use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::adb::client::AdbClient;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTelemetry {
    pub battery_level: u8,
    pub battery_status: BatteryStatus,
    pub battery_temp_celsius: f32,
    pub storage_free_gb: f32,
    pub storage_total_gb: f32,
    pub storage_used_percent: u8,
    pub wifi_signal_dbm: Option<i32>,
    pub wifi_ssid: Option<String>,
}

pub struct TelemetryCollector;

impl TelemetryCollector {
    /// Collects full system telemetry: Battery, Storage, Wi-Fi
    pub async fn collect(client: &AdbClient, serial: &str) -> AppResult<DeviceTelemetry> {
        let battery_output = client.shell(serial, &["dumpsys", "battery"]).await.unwrap_or_default();
        let (battery_level, battery_status, battery_temp_celsius) = Self::parse_battery(&battery_output);

        let df_output = client.shell(serial, &["df", "/data"]).await.unwrap_or_default();
        let (storage_free_gb, storage_total_gb, storage_used_percent) = Self::parse_storage(&df_output);

        let wifi_output = client.shell(serial, &["dumpsys", "wifi"]).await.unwrap_or_default();
        let (wifi_ssid, wifi_signal_dbm) = Self::parse_wifi(&wifi_output);

        Ok(DeviceTelemetry {
            battery_level,
            battery_status,
            battery_temp_celsius,
            storage_free_gb,
            storage_total_gb,
            storage_used_percent,
            wifi_signal_dbm,
            wifi_ssid,
        })
    }

    fn parse_battery(output: &str) -> (u8, BatteryStatus, f32) {
        let mut level: u8 = 100;
        let mut status = BatteryStatus::Unknown;
        let mut temp: f32 = 25.0;

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("level:") {
                if let Some(val) = line.split(':').nth(1) {
                    level = val.trim().parse::<u8>().unwrap_or(100);
                }
            } else if line.starts_with("status:") {
                if let Some(val) = line.split(':').nth(1) {
                    status = match val.trim() {
                        "2" => BatteryStatus::Charging,
                        "3" => BatteryStatus::Discharging,
                        "4" => BatteryStatus::NotCharging,
                        "5" => BatteryStatus::Full,
                        _ => BatteryStatus::Unknown,
                    };
                }
            } else if line.starts_with("temperature:") {
                if let Some(val) = line.split(':').nth(1) {
                    let raw = val.trim().parse::<f32>().unwrap_or(250.0);
                    temp = raw / 10.0;
                }
            }
        }

        (level, status, temp)
    }

    fn parse_storage(output: &str) -> (f32, f32, u8) {
        // Output format: Filesystem 1K-blocks Used Available Use% Mounted on
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let total_kb = parts[1].parse::<f32>().unwrap_or(0.0);
                let used_kb = parts[2].parse::<f32>().unwrap_or(0.0);
                let free_kb = parts[3].parse::<f32>().unwrap_or(0.0);

                let total_gb = total_kb / (1024.0 * 1024.0);
                let free_gb = free_kb / (1024.0 * 1024.0);
                let percent = if total_kb > 0.0 {
                    ((used_kb / total_kb) * 100.0) as u8
                } else {
                    0
                };

                return (free_gb, total_gb, percent);
            }
        }
        (0.0, 0.0, 0)
    }

    fn parse_wifi(output: &str) -> (Option<String>, Option<i32>) {
        let mut ssid = None;
        let mut rssi = None;

        for line in output.lines() {
            if line.contains("SSID:") || line.contains("mWifiInfo SSID:") {
                if let Some(pos) = line.find("SSID:") {
                    let part = &line[pos + 5..].trim();
                    let end = part.find(',').unwrap_or(part.len());
                    let clean = part[..end].trim().trim_matches('"');
                    if clean != "<unknown ssid>" && !clean.is_empty() {
                        ssid = Some(clean.to_string());
                    }
                }
            }
            if line.contains("RSSI:") {
                if let Some(pos) = line.find("RSSI:") {
                    let part = &line[pos + 5..].trim();
                    let end = part.find(',').unwrap_or(part.find(' ').unwrap_or(part.len()));
                    if let Ok(val) = part[..end].trim().parse::<i32>() {
                        rssi = Some(val);
                    }
                }
            }
        }

        (ssid, rssi)
    }
}
