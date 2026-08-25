use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::client::AdbClient;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscoveredService {
    pub instance_name: String,
    pub service_type: String, // e.g. "_adb-tls-connect._tcp" or "_adb-tls-pairing._tcp"
    pub host: String,
    pub port: u16,
}

pub struct MdnsScanner;

impl MdnsScanner {
    /// Scans local network for ADB Wireless Debugging services using `adb mdns services`
    pub async fn scan(client: &AdbClient) -> AppResult<Vec<DiscoveredService>> {
        debug!("Scanning for mDNS ADB services...");
        let output = match client.execute_raw(&["mdns", "services"]).await {
            Ok(out) => out,
            Err(e) => {
                warn!("mDNS scan output error: {}", e);
                return Ok(Vec::new());
            }
        };

        let mut services = Vec::new();
        for line in output.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Output format typically: <instance_name> \t <service_type> \t <ip:port>
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let instance_name = parts[0].to_string();
                let service_type = parts[1].to_string();
                let endpoint = parts[2];

                if let Some((host, port_str)) = endpoint.rsplit_once(':') {
                    if let Ok(port) = port_str.parse::<u16>() {
                        services.push(DiscoveredService {
                            instance_name,
                            service_type,
                            host: host.to_string(),
                            port,
                        });
                    }
                }
            }
        }

        Ok(services)
    }
}
