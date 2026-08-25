use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use super::client::AdbClient;
use super::commands::{AdbCommands, DeviceInfo};
use super::mdns::MdnsScanner;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Disconnected,
    Discovering,
    Connecting,
    Connected,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatusEvent {
    pub state: ConnectionState,
    pub device: Option<DeviceInfo>,
    pub message: Option<String>,
}

pub struct ConnectionManager {
    client: AdbClient,
    state: Arc<RwLock<ConnectionState>>,
    active_device: Arc<RwLock<Option<DeviceInfo>>>,
    last_known_endpoint: Arc<RwLock<Option<(String, u16)>>>,
    event_sender: broadcast::Sender<ConnectionStatusEvent>,
    watchdog_cancel: Arc<RwLock<Option<CancellationToken>>>,
}

impl ConnectionManager {
    pub fn new(client: AdbClient) -> Self {
        let (tx, _) = broadcast::channel(32);
        Self {
            client,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            active_device: Arc::new(RwLock::new(None)),
            last_known_endpoint: Arc::new(RwLock::new(None)),
            event_sender: tx,
            watchdog_cancel: Arc::new(RwLock::new(None)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConnectionStatusEvent> {
        self.event_sender.subscribe()
    }

    pub async fn get_current_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    pub async fn get_active_device(&self) -> Option<DeviceInfo> {
        self.active_device.read().await.clone()
    }

    async fn set_state(&self, new_state: ConnectionState, device: Option<DeviceInfo>, msg: Option<String>) {
        *self.state.write().await = new_state.clone();
        if let Some(ref dev) = device {
            *self.active_device.write().await = Some(dev.clone());
        } else if new_state == ConnectionState::Disconnected {
            *self.active_device.write().await = None;
        }

        let event = ConnectionStatusEvent {
            state: new_state,
            device,
            message: msg,
        };
        let _ = self.event_sender.send(event);
    }

    /// Explicitly connect to a host:port endpoint and start the watchdog supervisor
    pub async fn connect(&self, host: &str, port: u16) -> AppResult<DeviceInfo> {
        self.set_state(ConnectionState::Connecting, None, Some(format!("Connecting to {}:{}...", host, port))).await;
        *self.last_known_endpoint.write().await = Some((host.to_string(), port));

        match AdbCommands::connect(&self.client, host, port).await {
            Ok(device) => {
                info!("Device connected successfully: {:?}", device);
                self.set_state(ConnectionState::Connected, Some(device.clone()), None).await;
                self.spawn_watchdog(device.serial.clone()).await;
                Ok(device)
            }
            Err(e) => {
                warn!("Connect error: {:?}", e);
                self.set_state(ConnectionState::Disconnected, None, Some(format!("Connection failed: {}", e))).await;
                Err(e)
            }
        }
    }

    /// Disconnects active session and aborts watchdog
    pub async fn disconnect(&self) -> AppResult<()> {
        if let Some(token) = self.watchdog_cancel.write().await.take() {
            token.cancel();
        }

        if let Some(device) = self.active_device.read().await.clone() {
            let _ = AdbCommands::disconnect(&self.client, &device.serial).await;
        }

        self.set_state(ConnectionState::Disconnected, None, Some("Disconnected by user".to_string())).await;
        Ok(())
    }

    /// Background Watchdog & Self-Healing Reconnect Loop with exponential backoff & jitter
    async fn spawn_watchdog(&self, serial: String) {
        if let Some(old_token) = self.watchdog_cancel.write().await.take() {
            old_token.cancel();
        }

        let cancel_token = CancellationToken::new();
        *self.watchdog_cancel.write().await = Some(cancel_token.clone());

        let client = self.client.clone();
        let state_lock = Arc::clone(&self.state);
        let active_device_lock = Arc::clone(&self.active_device);
        let last_endpoint_lock = Arc::clone(&self.last_known_endpoint);
        let event_tx = self.event_sender.clone();

        tokio::spawn(async move {
            info!("Watchdog supervisor started for {}", serial);
            let mut failure_count = 0;

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Watchdog loop cancelled for {}", serial);
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {
                        let is_alive = AdbCommands::ping_device(&client, &serial).await;
                        if is_alive {
                            if failure_count > 0 {
                                info!("Device connection recovered for {}", serial);
                                failure_count = 0;
                                *state_lock.write().await = ConnectionState::Connected;
                                let dev = active_device_lock.read().await.clone();
                                let _ = event_tx.send(ConnectionStatusEvent {
                                    state: ConnectionState::Connected,
                                    device: dev,
                                    message: None,
                                });
                            }
                        } else {
                            failure_count += 1;
                            warn!("Watchdog ping failed for {} (streak: {})", serial, failure_count);

                            if failure_count == 1 {
                                *state_lock.write().await = ConnectionState::Degraded;
                                let dev = active_device_lock.read().await.clone();
                                let _ = event_tx.send(ConnectionStatusEvent {
                                    state: ConnectionState::Degraded,
                                    device: dev,
                                    message: Some("Connection degraded, attempting recovery...".to_string()),
                                });
                            } else if failure_count >= 3 {
                                *state_lock.write().await = ConnectionState::Discovering;
                                let _ = event_tx.send(ConnectionStatusEvent {
                                    state: ConnectionState::Discovering,
                                    device: None,
                                    message: Some("Device dropped. Searching network via mDNS & direct reconnect...".to_string()),
                                });

                                // Try auto-reconnecting with backoff
                                let backoff_secs = std::cmp::min(1 << failure_count, 30);
                                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;

                                // 1. Try last known IP/port
                                let endpoint = last_endpoint_lock.read().await.clone();
                                if let Some((host, port)) = endpoint {
                                    if let Ok(dev) = AdbCommands::connect(&client, &host, port).await {
                                        info!("Successfully reconnected to {}:{}", host, port);
                                        failure_count = 0;
                                        *state_lock.write().await = ConnectionState::Connected;
                                        *active_device_lock.write().await = Some(dev.clone());
                                        let _ = event_tx.send(ConnectionStatusEvent {
                                            state: ConnectionState::Connected,
                                            device: Some(dev),
                                            message: None,
                                        });
                                        continue;
                                    }
                                }

                                // 2. Try mDNS discovery
                                if let Ok(services) = MdnsScanner::scan(&client).await {
                                    for service in services {
                                        if service.service_type.contains("connect") {
                                            if let Ok(dev) = AdbCommands::connect(&client, &service.host, service.port).await {
                                                info!("mDNS reconnected to {}:{}", service.host, service.port);
                                                failure_count = 0;
                                                *state_lock.write().await = ConnectionState::Connected;
                                                *active_device_lock.write().await = Some(dev.clone());
                                                let _ = event_tx.send(ConnectionStatusEvent {
                                                    state: ConnectionState::Connected,
                                                    device: Some(dev),
                                                    message: None,
                                                });
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
