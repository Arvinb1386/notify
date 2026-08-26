use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
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
        }
    }

    /// Finds the best Local LAN IP address (192.168.x.x or 10.x.x.x), bypassing VPN virtual adapters (tun0, wireguard, 172.x)
    pub fn resolve_lan_ip() -> String {
        if let Ok(interfaces) = if_addrs::get_if_addrs() {
            let mut best_192 = None;
            let mut best_10 = None;
            let mut other_lan = None;

            for iface in interfaces {
                // Ignore loopback
                if iface.is_loopback() {
                    continue;
                }

                if let std::net::IpAddr::V4(ipv4) = iface.ip() {
                    let ip_str = ipv4.to_string();
                    let name_lower = iface.name.to_lowercase();

                    // Skip known VPN / Virtual adapter keywords
                    if name_lower.contains("tun")
                        || name_lower.contains("tap")
                        || name_lower.contains("vpn")
                        || name_lower.contains("wireguard")
                        || name_lower.contains("wsl")
                        || name_lower.contains("docker")
                        || name_lower.contains("vbox")
                        || name_lower.contains("vmware")
                    {
                        continue;
                    }

                    // Priority 1: Standard Home LAN (192.168.x.x)
                    if ip_str.starts_with("192.168.") {
                        // Prefer standard physical Wi-Fi/Ethernet subnets over VirtualBox 192.168.56.x
                        if !ip_str.starts_with("192.168.56.") {
                            best_192 = Some(ip_str);
                            break;
                        } else if best_192.is_none() {
                            best_192 = Some(ip_str);
                        }
                    }
                    // Priority 2: 10.x.x.x private range
                    else if ip_str.starts_with("10.") {
                        best_10 = Some(ip_str);
                    }
                    // Priority 3: Non-VPN private IP
                    else if !ip_str.starts_with("172.") && !ip_str.starts_with("169.254.") {
                        other_lan = Some(ip_str);
                    }
                }
            }

            if let Some(ip) = best_192 {
                return ip;
            }
            if let Some(ip) = best_10 {
                return ip;
            }
            if let Some(ip) = other_lan {
                return ip;
            }
        }

        // Fallback
        local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "192.168.1.4".to_string())
    }

    pub async fn get_pairing_qr_data(&self) -> PairingQrData {
        let ip = Self::resolve_lan_ip();
        let secret = self.pairing_secret.read().await.clone();

        PairingQrData {
            server_ip: ip,
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
                            let local_ip_str = Self::resolve_lan_ip();

                            let response = format!(
                                "NOTIFY_SERVER|{}|{}|Notify-PC",
                                local_ip_str, COMPANION_WS_PORT
                            );
                            let _ = socket.send_to(response.as_bytes(), src).await;
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

                tokio::spawn(async move {
                    info!("Incoming Companion connection from {}", peer_addr);
                    if let Ok(ws_stream) = accept_async(stream).await {
                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                        let (tx, mut rx) = mpsc::channel::<CompanionMessage>(32);

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
                        while let Some(Ok(msg)) = ws_receiver.next().await {
                            if let Message::Text(text) = msg {
                                if let Ok(parsed) = serde_json::from_str::<CompanionMessage>(text.as_str()) {
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
                                                id: key,
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
                                                fingerprint,
                                            };

                                            let _ = db_ref.insert_notification(&notif_item);
                                            let _ = app_h.emit("notification-received", &notif_item);

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
                                        CompanionMessage::NotificationRemoved { key, package_name } => {
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
