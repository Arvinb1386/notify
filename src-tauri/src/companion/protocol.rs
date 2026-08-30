use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum CompanionMessage {
    #[serde(rename = "handshake")]
    Handshake {
        device_id: String,
        device_name: String,
        manufacturer: String,
        model: String,
        android_version: String,
        pairing_token: String,
    },
    #[serde(rename = "notification_posted")]
    NotificationPosted {
        key: String,
        package_name: String,
        app_name: String,
        title: Option<String>,
        body: Option<String>,
        subtext: Option<String>,
        post_time: i64,
        icon_base64: Option<String>,
        can_reply: bool,
    },
    #[serde(rename = "notification_removed")]
    NotificationRemoved {
        key: String,
        package_name: String,
    },
    #[serde(rename = "telemetry")]
    Telemetry {
        battery_level: u8,
        battery_status: String,
        battery_temp: f32,
        wifi_ssid: Option<String>,
        wifi_signal: Option<i32>,
        /// Real storage stats (newer companion APKs); old APKs omit them
        #[serde(default)]
        storage_free_gb: Option<f64>,
        #[serde(default)]
        storage_total_gb: Option<f64>,
    },
    #[serde(rename = "clipboard")]
    Clipboard {
        text: String,
    },
    #[serde(rename = "quick_reply")]
    QuickReply {
        key: String,
        reply_text: String,
    },
    #[serde(rename = "media_control")]
    MediaControl {
        action: String, // "play_pause", "next", "prev", "volume_up", "volume_down"
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpBeaconPayload {
    pub device_id: String,
    pub device_name: String,
    pub port: u16,
    pub server_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingQrData {
    pub server_ip: String,
    /// All viable LAN IPv4 candidates ranked best-first (VPN-tolerant pairing).
    #[serde(default)]
    pub server_ips: Vec<String>,
    pub port: u16,
    pub secret_token: String,
    pub server_name: String,
}
