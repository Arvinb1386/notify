use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use super::dumpsys_parser::{DumpsysParser, NotificationItem, NotificationStatus};
use crate::adb::client::AdbClient;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RawNotificationSignal {
    Posted(String), // Trigger hint from logcat or poll
    Removed(String),
}

/// Abstract Notification Source Trait for extensible architectures
#[async_trait]
pub trait NotificationSource: Send + Sync {
    async fn start(
        &self,
        client: AdbClient,
        serial: String,
        signal_tx: mpsc::Sender<RawNotificationSignal>,
        cancel: CancellationToken,
    ) -> AppResult<()>;
}

/// Logcat Event Streamer (Hybrid trigger)
pub struct LogcatSource;

#[async_trait]
impl NotificationSource for LogcatSource {
    async fn start(
        &self,
        client: AdbClient,
        serial: String,
        signal_tx: mpsc::Sender<RawNotificationSignal>,
        cancel: CancellationToken,
    ) -> AppResult<()> {
        info!("Starting Logcat notification stream for {}", serial);
        let mut cmd = client.create_command(&[
            "-s",
            &serial,
            "logcat",
            "-v",
            "time",
            "-b",
            "main",
            "-b",
            "events",
            "-s",
            "NotificationService:I",
            "StatusBarNotification:I",
        ]);

        cmd.stdout(Stdio::piped()).stderr(Stdio::null());

        let mut child = cmd
            .spawn()
            .map_err(|e| crate::error::AppError::CommandFailed(format!("Logcat spawn failed: {}", e)))?;

        let stdout = child.stdout.take().ok_or_else(|| {
            crate::error::AppError::CommandFailed("Failed to capture logcat stdout".to_string())
        })?;

        let mut reader = BufReader::new(stdout).lines();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        let _ = child.kill().await;
                        break;
                    }
                    line_res = reader.next_line() => {
                        match line_res {
                            Ok(Some(line)) => {
                                if line.contains("enqueueNotification") || line.contains("onNotificationPosted") {
                                    let _ = signal_tx.send(RawNotificationSignal::Posted(line)).await;
                                } else if line.contains("cancelNotification") || line.contains("onNotificationRemoved") {
                                    let _ = signal_tx.send(RawNotificationSignal::Removed(line)).await;
                                }
                            }
                            Ok(None) => {
                                warn!("Logcat stream ended");
                                break;
                            }
                            Err(e) => {
                                debug!("Logcat line read error: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

/// Notification Engine: Coordinates Logcat stream, periodic dumpsys snapshots, diff reconciliation & deduplication
pub struct NotificationEngine {
    client: AdbClient,
    active_notifications: Arc<RwLock<std::collections::HashMap<String, NotificationItem>>>,
    seen_fingerprints: Arc<RwLock<lru::LruCache<String, i64>>>,
    event_sender: broadcast::Sender<NotificationItem>,
    cancel_token: Arc<RwLock<Option<CancellationToken>>>,
}

impl NotificationEngine {
    pub fn new(client: AdbClient) -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            client,
            active_notifications: Arc::new(RwLock::new(std::collections::HashMap::new())),
            seen_fingerprints: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(200).unwrap(),
            ))),
            event_sender: tx,
            cancel_token: Arc::new(RwLock::new(None)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NotificationItem> {
        self.event_sender.subscribe()
    }

    pub async fn start_monitoring(&self, serial: String) -> AppResult<()> {
        self.stop().await;

        let cancel = CancellationToken::new();
        *self.cancel_token.write().await = Some(cancel.clone());

        let (signal_tx, mut signal_rx) = mpsc::channel::<RawNotificationSignal>(64);
        let logcat_source = LogcatSource;
        let _ = logcat_source.start(self.client.clone(), serial.clone(), signal_tx, cancel.clone()).await;

        let client = self.client.clone();
        let active_map = Arc::clone(&self.active_notifications);
        let fingerprints = Arc::clone(&self.seen_fingerprints);
        let event_tx = self.event_sender.clone();
        let cancel_child = cancel.clone();

        tokio::spawn(async move {
            info!("Notification reconciliation supervisor running for {}", serial);
            let mut periodic_interval = tokio::time::interval(Duration::from_secs(3));

            loop {
                tokio::select! {
                    _ = cancel_child.cancelled() => {
                        info!("Notification supervisor cancelled");
                        break;
                    }
                    _ = periodic_interval.tick() => {
                        Self::reconcile_state(&client, &serial, &active_map, &fingerprints, &event_tx).await;
                    }
                    signal_opt = signal_rx.recv() => {
                        if signal_opt.is_some() {
                            // Debounce burst notifications (50ms)
                            tokio::time::sleep(Duration::from_millis(50)).await;
                            Self::reconcile_state(&client, &serial, &active_map, &fingerprints, &event_tx).await;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        if let Some(token) = self.cancel_token.write().await.take() {
            token.cancel();
        }
    }

    /// Fetches dumpsys snapshot and computes Diff: NEW, UPDATED, REMOVED
    async fn reconcile_state(
        client: &AdbClient,
        serial: &str,
        active_map: &Arc<RwLock<std::collections::HashMap<String, NotificationItem>>>,
        fingerprints: &Arc<RwLock<lru::LruCache<String, i64>>>,
        event_tx: &broadcast::Sender<NotificationItem>,
    ) {
        let raw_dumpsys = match client.shell(serial, &["dumpsys", "notification", "--noredact"]).await {
            Ok(out) => out,
            Err(e) => {
                debug!("Dumpsys poll error: {}", e);
                return;
            }
        };

        let current_items = DumpsysParser::parse_snapshot(&raw_dumpsys);
        let mut active_lock = active_map.write().await;
        let mut fp_lock = fingerprints.write().await;

        let mut current_keys = std::collections::HashSet::new();

        for item in current_items {
            current_keys.insert(item.id.clone());

            let is_known = active_lock.contains_key(&item.id);
            let is_duplicate_fp = fp_lock.contains(&item.fingerprint);

            if !is_duplicate_fp {
                fp_lock.put(item.fingerprint.clone(), item.post_time);

                let event_item = if is_known {
                    let mut updated = item.clone();
                    updated.status = NotificationStatus::Updated;
                    updated
                } else {
                    item.clone()
                };

                active_lock.insert(item.id.clone(), item.clone());
                let _ = event_tx.send(event_item);
            }
        }

        // Detect REMOVED items
        let removed_keys: Vec<String> = active_lock
            .keys()
            .filter(|k| !current_keys.contains(*k))
            .cloned()
            .collect();

        for key in removed_keys {
            if let Some(mut removed_item) = active_lock.remove(&key) {
                removed_item.status = NotificationStatus::Removed;
                let _ = event_tx.send(removed_item);
            }
        }
    }
}
