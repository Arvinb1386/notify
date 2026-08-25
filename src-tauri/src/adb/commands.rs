use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::client::AdbClient;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DeviceInfo {
    pub serial: String,       // e.g. "192.168.1.50:41235"
    pub model: String,        // e.g. "Pixel 8 Pro"
    pub manufacturer: String, // e.g. "Google"
    pub android_version: String, // e.g. "14"
    pub sdk_version: String,  // e.g. "34"
    pub is_wireless: bool,
    pub is_connected: bool,
}

impl DeviceInfo {
    pub fn new_placeholder(serial: &str) -> Self {
        Self {
            serial: serial.to_string(),
            model: "Android Device".to_string(),
            manufacturer: "Generic".to_string(),
            android_version: "Unknown".to_string(),
            sdk_version: "Unknown".to_string(),
            is_wireless: serial.contains(':'),
            is_connected: true,
        }
    }
}

pub struct AdbCommands;

impl AdbCommands {
    /// Pairs with an Android 11+ device using pairing code: adb pair <ip>:<port> <code>
    pub async fn pair(client: &AdbClient, host: &str, port: u16, code: &str) -> AppResult<String> {
        let target = format!("{}:{}", host, port);
        info!("Pairing with device at {} with code", target);
        let output = client.execute_raw(&["pair", &target, code]).await?;

        if output.contains("Successfully paired to") {
            info!("Pairing successful: {}", output.trim());
            Ok(output.trim().to_string())
        } else {
            warn!("Pairing failed output: {}", output);
            Err(AppError::PairingFailed(output.trim().to_string()))
        }
    }

    /// Connects to a device via Wi-Fi: adb connect <ip>:<port>
    pub async fn connect(client: &AdbClient, host: &str, port: u16) -> AppResult<DeviceInfo> {
        let target = format!("{}:{}", host, port);
        info!("Connecting to device at {}", target);
        let output = client.execute_raw(&["connect", &target]).await?;

        if output.contains("connected to") || output.contains("already connected to") {
            info!("Successfully connected to {}", target);
            Self::get_device_info(client, &target).await
        } else {
            warn!("Connection failed output: {}", output);
            Err(AppError::ConnectionRefused(output.trim().to_string()))
        }
    }

    /// Disconnects from an ADB device: adb disconnect <ip>:<port>
    pub async fn disconnect(client: &AdbClient, serial: &str) -> AppResult<()> {
        info!("Disconnecting from device {}", serial);
        let _ = client.execute_raw(&["disconnect", serial]).await;
        Ok(())
    }

    /// Lists currently attached ADB devices
    pub async fn list_devices(client: &AdbClient) -> AppResult<Vec<String>> {
        let output = client.execute_raw(&["devices"]).await?;
        let mut devices = Vec::new();

        for line in output.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                devices.push(parts[0].to_string());
            }
        }

        Ok(devices)
    }

    /// Fetches rich device telemetry & identity: model, manufacturer, android version, sdk
    pub async fn get_device_info(client: &AdbClient, serial: &str) -> AppResult<DeviceInfo> {
        let model = client
            .shell(serial, &["getprop", "ro.product.model"])
            .await
            .unwrap_or_else(|_| "Unknown Model".to_string())
            .trim()
            .to_string();

        let manufacturer = client
            .shell(serial, &["getprop", "ro.product.manufacturer"])
            .await
            .unwrap_or_else(|_| "Android".to_string())
            .trim()
            .to_string();

        let android_version = client
            .shell(serial, &["getprop", "ro.build.version.release"])
            .await
            .unwrap_or_else(|_| "14".to_string())
            .trim()
            .to_string();

        let sdk_version = client
            .shell(serial, &["getprop", "ro.build.version.sdk"])
            .await
            .unwrap_or_else(|_| "34".to_string())
            .trim()
            .to_string();

        Ok(DeviceInfo {
            serial: serial.to_string(),
            model: if model.is_empty() { "Android Device".to_string() } else { model },
            manufacturer: if manufacturer.is_empty() { "Generic".to_string() } else { manufacturer },
            android_version: if android_version.is_empty() { "14".to_string() } else { android_version },
            sdk_version: if sdk_version.is_empty() { "34".to_string() } else { sdk_version },
            is_wireless: serial.contains(':'),
            is_connected: true,
        })
    }

    /// Pings device state (returns true if 'device' state)
    pub async fn ping_device(client: &AdbClient, serial: &str) -> bool {
        match client.execute_raw(&["-s", serial, "get-state"]).await {
            Ok(out) => out.trim() == "device",
            Err(_) => false,
        }
    }
}
