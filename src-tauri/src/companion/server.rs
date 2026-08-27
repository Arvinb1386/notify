use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use super::protocol::{CompanionMessage, PairingQrData};
use crate::notifications::dumpsys_parser::{NotificationItem, NotificationStatus};
use crate::notifications::otp_detector::OtpDetector;
use crate::notifications::DesktopNotifier;
use crate::storage::Database;
use sha2::{Digest, Sha256};

pub const COMPANION_WS_PORT: u16 = 27890;
pub const COMPANION_UDP_PORT: u16 = 27891;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectedCompanion {
    pub device_id: String,
    pub device_name: String,
    pub manufacturer: String,
    pub model: String,
    pub android_version: String,
    pub ip_address: String,
    pub connected_at: i64,
}

pub struct CompanionServer {
    app_handle: AppHandle,
    database: Arc<Database>,
    pairing_secret: Arc<RwLock<String>>,
    connected_device: Arc<RwLock<Option<ConnectedCompanion>>>,
    outgoing_tx: Arc<RwLock<Option<mpsc::Sender<CompanionMessage>>>>,
    /// When true (user pressed Disconnect on the PC), incoming companion
    /// sessions are rejected immediately so the phone cannot auto-reconnect.
    paused: Arc<RwLock<bool>>,
    /// Cancels the currently active companion WebSocket session.
    session_cancel: Arc<RwLock<Option<CancellationToken>>>,
}

impl CompanionServer {
    pub fn new(app_handle: AppHandle, database: Arc<Database>) -> Self {
        let secret = uuid::Uuid::new_v4().to_string().replace('-', "")[..12].to_string();
        Self {
            app_handle,
            database,
            pairing_secret: Arc::new(RwLock::new(secret)),
            connected_device: Arc::new(RwLock::new(None)),
            outgoing_tx: Arc::new(RwLock::new(None)),
            paused: Arc::new(RwLock::new(false)),
            session_cancel: Arc::new(RwLock::new(None)),
        }
    }

    /// User-initiated disconnect: drops the active companion connection and
    /// blocks re-connections until resume_companion() is called again.
    pub async fn pause(&self) {
        info!("Companion server paused by user (disconnect requested)");
        *self.paused.write().await = true;

        if let Some(token) = self.session_cancel.write().await.take() {
            token.cancel();
        }

        let had_device = self.connected_device.write().await.take();
        *self.outgoing_tx.write().await = None;

        if let Some(dev) = had_device {
            let _ = self.app_handle.emit("companion-disconnected", &dev);
        }
    }

    /// Re-enables accepting companion connections (e.g. when the user opens
    /// the pairing wizard or explicitly connects a device again).
    pub async fn resume(&self) {
        if *self.paused.read().await {
            info!("Companion server resumed");
            *self.paused.write().await = false;
        }
    }

    /// Detects known VPN / virtual adapter names (keyword match on interface name)
    fn is_vpn_or_virtual_name(name_lower: &str) -> bool {
        const VPN_KEYWORDS: &[&str] = &[
            "tun", "tap", "vpn", "wireguard", "wintun", "tunnel",
            "nord", "tailscale", "mullvad", "proton", "warp", "cloudflare",
            "expressvpn", "surfshark", "zenmate", "zerotier", "hamachi", "openvpn",
            "hyper", "wsl", "docker", "vbox", "virtualbox", "vmware", "virtual",
            "microsoft host", "hosted network", "loopback pseudo",
        ];
        VPN_KEYWORDS.iter().any(|k| name_lower.contains(k))
    }

    /// Positive score for physical NIC names (Wi-Fi / Ethernet)
    fn physical_nic_bonus(name_lower: &str) -> i32 {
        const PHYSICAL_KEYWORDS: &[&str] = &[
            "wi-fi", "wifi", "wlan", "wireless lan", "ethernet", "eth", "lan connection",
        ];
        if PHYSICAL_KEYWORDS.iter().any(|k| name_lower.contains(k)) {
            40
        } else {
            0
        }
    }

    /// Scores a candidate IPv4 address for LAN-pairing suitability.
    /// Returns None for addresses that can never reach a phone on the local network.
    fn lan_candidate_score(name_lower: &str, ip_str: &str) -> Option<i32> {
        let octets: Vec<u32> = ip_str.split('.').filter_map(|o| o.parse::<u32>().ok()).collect();
        if octets.len() != 4 || octets.iter().any(|o| *o > 255) {
            return None;
        }

        // Loopback (127.x), link-local (169.254.x) — never usable
        if octets[0] == 127 || (octets[0] == 169 && octets[1] == 254) {
            return None;
        }
        // Multicast (224.x) & reserved (240.x+) — never usable
        if octets[0] >= 224 {
            return None;
        }
        // Only private / CGNAT ranges matter for pairing:
        //   - private: real LANs
        //   - CGNAT 100.64/10: Tailscale/ZeroTier-style meshes where both devices
        //     may share the same overlay when their VPNs are connected to the
        //     same network — kept as an ultra-low priority escape hatch.
        let is_private = octets[0] == 10
            || (octets[0] == 172 && (16..=31).contains(&octets[1]))
            || (octets[0] == 192 && octets[1] == 168);
        let is_cgnat = octets[0] == 100 && (64..=127).contains(&octets[1]);
        if !is_private && !is_cgnat {
            return None;
        }

        // Base score by subnet class:
        //   192.168.x  -> classic home LAN routers (strongest signal)
        //   172.16-31  -> valid private LAN, slightly unusual
        //   10.x       -> ambiguous: real corporate LANs AND most consumer VPNs use it
        //   CGNAT      -> same-VPN mesh fallback only
        let mut score = if octets[0] == 192 && octets[1] == 168 {
            30
        } else if octets[0] == 172 {
            12
        } else if is_cgnat {
            -40
        } else {
            5
        };

        // VirtualBox host-only adapter (192.168.56.x) — deprioritize heavily
        if octets[0] == 192 && octets[1] == 168 && octets[2] == 56 {
            score -= 60;
        }

        // Name-based signals
        if Self::is_vpn_or_virtual_name(name_lower) {
            score -= 50;
        } else {
            score += Self::physical_nic_bonus(name_lower);
        }

        Some(score)
    }

    /// Returns ALL viable LAN IPv4 addresses ranked best-first.
    /// Under an active VPN the tunnel usually wins the routing table, so we never
    /// rely on a single guess: callers hand the whole list to the phone, which
    /// probes each address and skips the unreachable (VPN) ones.
    pub fn resolve_lan_candidates() -> Vec<String> {
        let mut candidates: Vec<(String, i32)> = Vec::new();

        if let Ok(interfaces) = if_addrs::get_if_addrs() {
            for iface in interfaces {
                if iface.is_loopback() {
                    continue;
                }
                if let std::net::IpAddr::V4(ipv4) = iface.ip() {
                    let ip_str = ipv4.to_string();
                    let name_lower = iface.name.to_lowercase();
                    if let Some(score) = Self::lan_candidate_score(&name_lower, &ip_str) {
                        if !candidates.iter().any(|(existing, _)| existing == &ip_str) {
                            candidates.push((ip_str, score));
                        }
                    }
                }
            }
        }

        // Best score first; deterministic tiebreak by IP string
        candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        candidates.into_iter().map(|(ip, _)| ip).take(6).collect()
    }

    /// Best single guess for the LAN IP (primary candidate).
    pub fn resolve_lan_ip() -> String {
        Self::resolve_lan_candidates()
            .first()
            .cloned()
            // Last-resort fallback: default-route interface (may be a VPN IP under tunnel,
            // but still better than nothing — the phone-side probing will discard it anyway).
            .or_else(|| local_ip_address::local_ip().ok().map(|ip| ip.to_string()))
            .unwrap_or_else(|| "192.168.1.4".to_string())
    }

    pub async fn get_pairing_qr_data(&self) -> PairingQrData {
        let candidates = Self::resolve_lan_candidates();
        let primary = candidates.first().cloned().unwrap_or_else(Self::resolve_lan_ip);
        let secret = self.pairing_secret.read().await.clone();

        PairingQrData {
            server_ip: primary,
            server_ips: candidates,
            port: COMPANION_WS_PORT,
            secret_token: secret,
            server_name: "Notify PC Companion".to_string(),
        }
    }

    pub async fn get_connected_companion(&self) -> Option<ConnectedCompanion> {
        self.connected_device.read().await.clone()
    }

    /// Sends a quick reply back to the Android phone for a specific notification
    pub async fn send_quick_reply(&self, key: String, reply_text: String) -> Result<(), String> {
        if let Some(ref tx) = *self.outgoing_tx.read().await {
            tx.send(CompanionMessage::QuickReply { key, reply_text })
                .await
                .map_err(|e| format!("Failed to send reply to phone: {}", e))?;
            Ok(())
        } else {
            Err("No companion app connected".to_string())
        }
    }

    /// Starts both the UDP Discovery Beacon and WebSocket listener server
    pub fn start(&self) {
        let db = Arc::clone(&self.database);
        let secret_lock = Arc::clone(&self.pairing_secret);
        let connected_lock = Arc::clone(&self.connected_device);
        let outgoing_lock = Arc::clone(&self.outgoing_tx);
        let paused_lock = Arc::clone(&self.paused);
        let session_cancel_lock = Arc::clone(&self.session_cancel);

        // 1. UDP Discovery Responder Loop
        tokio::spawn(async move {
            if let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", COMPANION_UDP_PORT)).await {
                info!("UDP Discovery Beacon listening on port {}", COMPANION_UDP_PORT);
                let mut buf = [0u8; 1024];

                loop {
                    if let Ok((len, src)) = socket.recv_from(&mut buf).await {
                        let msg_str = String::from_utf8_lossy(&buf[..len]);
                        if msg_str.starts_with("NOTIFY_DISCOVER") {
                            debug!("Received UDP discovery from {}", src);

                            // Answer with ONE packet per candidate LAN IP. Under VPN,
                            // some advertised addresses are tunnel IPs the phone can't
                            // reach — the phone probes each candidate and connects to
                            // whichever actually responds.
                            let candidates = Self::resolve_lan_candidates();
                            if candidates.is_empty() {
                                let response = format!(
                                    "NOTIFY_SERVER|{}|{}|Notify-PC",
                                    Self::resolve_lan_ip(),
                                    COMPANION_WS_PORT
                                );
                                let _ = socket.send_to(response.as_bytes(), src).await;
                            } else {
                                for ip in &candidates {
                                    let response = format!(
                                        "NOTIFY_SERVER|{}|{}|Notify-PC",
                                        ip, COMPANION_WS_PORT
                                    );
                                    let _ = socket.send_to(response.as_bytes(), src).await;
                                }
                            }
                        }
                    }
                }
            } else {
                warn!("Could not bind UDP discovery port {}", COMPANION_UDP_PORT);
            }
        });

    // 2. WebSocket Connection Server
        let app_ws = self.app_handle.clone();
        tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", COMPANION_WS_PORT);
            let listener = match TcpListener::bind(&addr).await {
                Ok(l) => {
                    info!("WebSocket Companion Server listening on {}", addr);
                    l
                }
                Err(e) => {
                    error!("Failed to bind WebSocket server on {}: {}", addr, e);
                    return;
                }
            };

            while let Ok((stream, peer_addr)) = listener.accept().await {
                let app_h = app_ws.clone();
                let db_ref = Arc::clone(&db);
                let _secret_ref = Arc::clone(&secret_lock);
                let conn_dev = Arc::clone(&connected_lock);
                let out_tx_lock = Arc::clone(&outgoing_lock);
                let paused_flag = Arc::clone(&paused_lock);
                let session_slot = Arc::clone(&session_cancel_lock);

                tokio::spawn(async move {
                    // Reject sessions while the user has disconnected on purpose
                    if *paused_flag.read().await {
                        debug!("Companion connection from {} rejected (server paused)", peer_addr);
                        if let Ok(mut ws) = accept_async(stream).await {
                            let _ = ws.send(Message::Close(None)).await;
                        }
                        return;
                    }

                    info!("Incoming Companion connection from {}", peer_addr);
                    if let Ok(ws_stream) = accept_async(stream).await {
                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                        let (tx, mut rx) = mpsc::channel::<CompanionMessage>(32);

                        // Register a cancellation token so pause() can kill this session
                        let cancel_token = CancellationToken::new();
                        *session_slot.write().await = Some(cancel_token.clone());

                        // Save outgoing sender
                        *out_tx_lock.write().await = Some(tx);

                        // Forward outgoing messages to client
                        let out_handle = tokio::spawn(async move {
                            while let Some(msg) = rx.recv().await {
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    if ws_sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        });

                        let mut device_info_holder: Option<ConnectedCompanion> = None;

                        // Process incoming messages from Android app
                        loop {
                            tokio::select! {
                                _ = cancel_token.cancelled() => {
                                    info!("Companion session cancelled (user disconnect)");
                                    break;
                                }
                                msg = ws_receiver.next() => {
                                    let text = match msg {
                                        Some(Ok(Message::Text(t))) => t,
                                        _ => break,
                                    };
                                    let parsed = match serde_json::from_str::<CompanionMessage>(text.as_str()) {
                                        Ok(p) => p,
                                        Err(e) => {
                                            debug!("Ignoring malformed companion message: {}", e);
                                            continue;
                                        }
                                    };
                                    match parsed {
                                        CompanionMessage::Handshake {
                                            device_id,
                                            device_name,
                                            manufacturer,
                                            model,
                                            android_version,
                                            pairing_token: _,
                                        } => {
                                            info!("Handshake accepted for Android device: {} ({})", device_name, model);
                                            let companion = ConnectedCompanion {
                                                device_id: device_id.clone(),
                                                device_name: device_name.clone(),
                                                manufacturer: manufacturer.clone(),
                                                model: model.clone(),
                                                android_version: android_version.clone(),
                                                ip_address: peer_addr.ip().to_string(),
                                                connected_at: chrono::Utc::now().timestamp_millis(),
                                            };
                                            *conn_dev.write().await = Some(companion.clone());
                                            device_info_holder = Some(companion.clone());
                                            let _ = app_h.emit("companion-connected", &companion);

                                            DesktopNotifier::show(
                                                "Companion Connected",
                                                &format!("Connected to {} ({})", device_name, model),
                                            );
                                        }
                                        CompanionMessage::NotificationPosted {
                                            key,
                                            package_name,
                                            app_name,
                                            title,
                                            body,
                                            subtext,
                                            post_time,
                                            icon_base64: _,
                                            can_reply: _,
                                        } => {
                                            let otp_res = OtpDetector::detect(title.as_deref(), body.as_deref());
                                            let (is_otp, otp_code) = match otp_res {
                                                Some(otp) => (true, Some(otp.code)),
                                                None => (false, None),
                                            };

                                            let mut hasher = Sha256::new();
                                            hasher.update(format!("{}:{}:{}:{}", package_name, title.as_deref().unwrap_or(""), body.as_deref().unwrap_or(""), post_time));
                                            let fingerprint = format!("{:x}", hasher.finalize());

                                            let notif_item = NotificationItem {
                                                id: key.clone(),
                                                package_name: package_name.clone(),
                                                app_name: Some(app_name.clone()),
                                                title: title.clone(),
                                                body: body.clone(),
                                                subtext: subtext.clone(),
                                                channel_id: None,
                                                post_time,
                                                is_otp,
                                                otp_code: otp_code.clone(),
                                                status: NotificationStatus::Posted,
                                                fingerprint: fingerprint.clone(),
                                            };

                                            // Collapse chatty updates (progress bars etc.):
                                            // identical repeat payloads are dropped entirely.
                                            let signature = format!(
                                                "{}|{}|{}|{}|{}",
                                                title.as_deref().unwrap_or(""),
                                                body.as_deref().unwrap_or(""),
                                                subtext.as_deref().unwrap_or(""),
                                                otp_code.as_deref().unwrap_or(""),
                                                post_time
                                            );
                                            if !crate::notifications::UpdateGate::should_forward(&key, &signature) {
                                                continue;
                                            }

                                            let _ = db_ref.insert_notification(&notif_item);
                                            let _ = app_h.emit("notification-received", &notif_item);

                                            // Toast only for genuinely new / changed notifications,
                                            // never once-per-second progress ticks.
                                            if crate::notifications::UpdateGate::should_show_toast(&key, &signature, is_otp) {
                                                let display_title = if let Some(ref t) = title {
                                                    format!("{}: {}", app_name, t)
                                                } else {
                                                    app_name
                                                };

                                                let body_text = if let Some(ref otp) = otp_code {
                                                    format!("Verification Code: {}\n{}", otp, body.clone().unwrap_or_default())
                                                } else {
                                                    body.clone().unwrap_or_else(|| "New message".to_string())
                                                };

                                                DesktopNotifier::show(&display_title, &body_text);
                                            }
                                        }
                                        CompanionMessage::NotificationRemoved { key, package_name } => {
                                            crate::notifications::UpdateGate::forget(&key);
                                            let notif_item = NotificationItem {
                                                id: key,
                                                package_name,
                                                app_name: None,
                                                title: None,
                                                body: None,
                                                subtext: None,
                                                channel_id: None,
                                                post_time: 0,
                                                is_otp: false,
                                                otp_code: None,
                                                status: NotificationStatus::Removed,
                                                fingerprint: "".to_string(),
                                            };
                                            let _ = app_h.emit("notification-received", &notif_item);
                                        }
                                        CompanionMessage::Telemetry {
                                            battery_level,
                                            battery_status,
                                            battery_temp,
                                            wifi_ssid,
                                            wifi_signal,
                                        } => {
                                            let telemetry_payload = serde_json::json!({
                                                "battery_level": battery_level,
                                                "battery_status": battery_status.to_lowercase(),
                                                "battery_temp_celsius": battery_temp,
                                                "storage_free_gb": 0.0,
                                                "storage_total_gb": 0.0,
                                                "storage_used_percent": 0,
                                                "wifi_signal_dbm": wifi_signal,
                                                "wifi_ssid": wifi_ssid,
                                            });
                                            let _ = app_h.emit("telemetry-updated", telemetry_payload);
                                        }
                                        CompanionMessage::Clipboard { text } => {
                                            let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(text));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // Cleanup on disconnect
                        out_handle.abort();
                        *out_tx_lock.write().await = None;
                        *conn_dev.write().await = None;
                        if let Some(dev) = device_info_holder {
                            let _ = app_h.emit("companion-disconnected", &dev);
                        }
                        info!("Companion device disconnected");
                    }
                });
            }
        });
    }
}
