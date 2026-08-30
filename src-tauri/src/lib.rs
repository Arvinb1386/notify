pub mod adb;
pub mod companion;
pub mod controls;
pub mod error;
pub mod notifications;
pub mod security;
pub mod storage;
pub mod telemetry;
pub mod tray;

use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tauri_plugin_notification::NotificationExt;

use adb::{AdbClient, AdbCommands, ConnectionManager, ConnectionState, DeviceInfo};
use companion::{CompanionServer, ConnectedCompanion, PairingQrData};
use controls::{DeviceCapabilities, RemoteControls};
use error::{AppError, AppResult};
use notifications::{NotificationEngine, NotificationItem};
use security::clipboard::SecureClipboard;
use storage::Database;
use telemetry::{DeviceTelemetry, TelemetryCollector};

pub struct AppState {
    pub adb_client: AdbClient,
    pub connection_manager: Arc<ConnectionManager>,
    pub notification_engine: Arc<NotificationEngine>,
    pub companion_server: Arc<CompanionServer>,
    pub database: Arc<Database>,
}

#[tauri::command]
async fn check_adb_status(state: State<'_, AppState>) -> Result<String, AppError> {
    state.adb_client.check_version().await
}

#[tauri::command]
async fn pair_device(
    host: String,
    port: u16,
    code: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    AdbCommands::pair(&state.adb_client, &host, port, &code).await
}

#[tauri::command]
async fn connect_device(
    host: String,
    port: u16,
    state: State<'_, AppState>,
) -> Result<DeviceInfo, AppError> {
    state.companion_server.resume().await;
    let dev = state.connection_manager.connect(&host, port).await?;
    let _ = state.database.save_device(&dev);
    let _ = state.notification_engine.start_monitoring(dev.serial.clone()).await;
    Ok(dev)
}

#[tauri::command]
async fn connect_by_serial(
    serial: String,
    state: State<'_, AppState>,
) -> Result<DeviceInfo, AppError> {
    state.companion_server.resume().await;
    // If serial contains host:port
    if let Some((host, port_str)) = serial.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            let dev = state.connection_manager.connect(host, port).await?;
            let _ = state.database.save_device(&dev);
            let _ = state.notification_engine.start_monitoring(dev.serial.clone()).await;
            return Ok(dev);
        }
    }

    // Direct adb attach if already paired/connected
    let dev = AdbCommands::get_device_info(&state.adb_client, &serial).await?;
    let _ = state.connection_manager.connect_existing(dev.clone()).await;
    let _ = state.database.save_device(&dev);
    let _ = state.notification_engine.start_monitoring(serial).await;
    Ok(dev)
}

#[tauri::command]
async fn get_saved_devices(state: State<'_, AppState>) -> Result<Vec<storage::SavedDevice>, AppError> {
    state.database.get_saved_devices()
}

#[tauri::command]
async fn delete_saved_device(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.database.delete_saved_device(&serial)
}

#[tauri::command]
async fn disconnect_device(state: State<'_, AppState>) -> Result<(), AppError> {
    state.notification_engine.stop().await;
    // Drop the companion WebSocket session AND reject auto-reconnect attempts
    // from the phone until the user explicitly pairs/connects again.
    state.companion_server.pause().await;
    state.connection_manager.disconnect().await
}

#[tauri::command]
async fn get_connection_state(state: State<'_, AppState>) -> Result<ConnectionState, AppError> {
    Ok(state.connection_manager.get_current_state().await)
}

#[tauri::command]
async fn get_active_device(state: State<'_, AppState>) -> Result<Option<DeviceInfo>, AppError> {
    Ok(state.connection_manager.get_active_device().await)
}

#[tauri::command]
async fn scan_mdns(state: State<'_, AppState>) -> Result<Vec<adb::mdns::DiscoveredService>, AppError> {
    adb::mdns::MdnsScanner::scan(&state.adb_client).await
}

#[tauri::command]
async fn get_telemetry(serial: String, state: State<'_, AppState>) -> Result<DeviceTelemetry, AppError> {
    TelemetryCollector::collect(&state.adb_client, &serial).await
}

#[tauri::command]
async fn send_keyevent(serial: String, keycode: u32, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::send_keyevent(&state.adb_client, &serial, keycode).await
}

#[tauri::command]
async fn volume_up(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::volume_up(&state.adb_client, &serial).await
}

#[tauri::command]
async fn volume_down(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::volume_down(&state.adb_client, &serial).await
}

#[tauri::command]
async fn media_play_pause(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::media_play_pause(&state.adb_client, &serial).await
}

#[tauri::command]
async fn media_next(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::media_next(&state.adb_client, &serial).await
}

#[tauri::command]
async fn media_prev(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::media_prev(&state.adb_client, &serial).await
}

#[tauri::command]
async fn wake_screen(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::wake_screen(&state.adb_client, &serial).await
}

#[tauri::command]
async fn lock_screen(serial: String, state: State<'_, AppState>) -> Result<(), AppError> {
    RemoteControls::lock_screen(&state.adb_client, &serial).await
}

#[tauri::command]
async fn check_capabilities(serial: String, state: State<'_, AppState>) -> Result<DeviceCapabilities, AppError> {
    Ok(RemoteControls::check_capabilities(&state.adb_client, &serial).await)
}

#[tauri::command]
async fn copy_otp_to_clipboard(code: String, ttl_secs: Option<u64>) -> Result<(), String> {
    SecureClipboard::copy_with_ttl(code, ttl_secs.unwrap_or(45))
}

#[tauri::command]
async fn delete_notification(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.database.delete_notification(&id)
}

#[tauri::command]
async fn get_notification_history(limit: Option<u32>, state: State<'_, AppState>) -> Result<Vec<NotificationItem>, AppError> {
    state.database.get_recent_notifications(limit.unwrap_or(100))
}

#[tauri::command]
async fn get_companion_pairing_qr(state: State<'_, AppState>) -> Result<PairingQrData, AppError> {
    // Opening the pairing wizard means the user wants to (re)connect —
    // clear any user-disconnect pause so the phone can reach us again.
    state.companion_server.resume().await;
    Ok(state.companion_server.get_pairing_qr_data().await)
}

#[tauri::command]
async fn get_connected_companion(state: State<'_, AppState>) -> Result<Option<ConnectedCompanion>, AppError> {
    Ok(state.companion_server.get_connected_companion().await)
}

#[tauri::command]
async fn send_companion_reply(key: String, reply_text: String, state: State<'_, AppState>) -> Result<(), String> {
    state.companion_server.send_quick_reply(key, reply_text).await
}

#[tauri::command]
async fn clear_notification_history(state: State<'_, AppState>) -> Result<(), AppError> {
    state.database.clear_notifications()
}

pub fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info,notify=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Notify Application...");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            let adb_client = match AdbClient::new(None) {
                Ok(c) => c,
                Err(e) => {
                    error!("ADB initialization warning: {:?}", e);
                    AdbClient {
                        binary_path: std::path::PathBuf::from("adb"),
                    }
                }
            };

            let conn_manager = Arc::new(ConnectionManager::new(adb_client.clone()));
            let notif_engine = Arc::new(NotificationEngine::new(adb_client.clone()));

            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let db_path = app_data_dir.join("notify.db");
            let database = Arc::new(Database::new(db_path).expect("Failed to initialize SQLite Database"));

            let companion_server = Arc::new(CompanionServer::new(app_handle.clone(), Arc::clone(&database)));
            companion_server.start();

            // Setup system tray & ensure AppUserModelId registered
            let _ = tray::setup_tray(&app_handle);
            notifications::DesktopNotifier::ensure_app_id_registered();

            // Auto-attach if an ADB device is already connected on startup
            let init_adb = adb_client.clone();
            let init_conn_mgr = Arc::clone(&conn_manager);
            let init_notif_eng = Arc::clone(&notif_engine);
            tauri::async_runtime::spawn(async move {
                if let Ok(devices) = AdbCommands::list_devices(&init_adb).await {
                    if let Some(first_serial) = devices.first() {
                        if let Ok(dev) = AdbCommands::get_device_info(&init_adb, first_serial).await {
                            info!("Auto-attached to active ADB device on startup: {}", first_serial);
                            let _ = init_conn_mgr.connect_existing(dev.clone()).await;
                            let _ = init_notif_eng.start_monitoring(first_serial.clone()).await;
                        }
                    }
                }
            });

            // Forward connection status events to frontend & manage telemetry loop
            let mut conn_rx = conn_manager.subscribe();
            let handle_conn = app_handle.clone();
            let adb_for_telemetry = adb_client.clone();
            let conn_mgr_ref = Arc::clone(&conn_manager);
            tauri::async_runtime::spawn(async move {
                while let Ok(event) = conn_rx.recv().await {
                    let _ = handle_conn.emit("connection-status-changed", &event);
                }
            });

            // Fast Live Telemetry Background Poller (Battery & Storage every 3 seconds when connected)
            let handle_telemetry = app_handle.clone();
            let adb_telemetry_loop = adb_client.clone();
            let conn_mgr_for_poll = Arc::clone(&conn_manager);
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
                loop {
                    interval.tick().await;
                    if let Some(dev) = conn_mgr_for_poll.get_active_device().await {
                        if let Ok(telemetry) = TelemetryCollector::collect(&adb_telemetry_loop, &dev.serial).await {
                            let _ = handle_telemetry.emit("telemetry-updated", &telemetry);
                        }
                    }
                }
            });

            // Forward notification events to frontend & save to DB & dispatch Windows OS Toast
            let mut notif_rx = notif_engine.subscribe();
            let handle_notif = app_handle.clone();
            let db_ref = Arc::clone(&database);
            let companion_for_forwarder = Arc::clone(&companion_server);
            tauri::async_runtime::spawn(async move {
                while let Ok(item) = notif_rx.recv().await {
                    // If the companion app is connected, it already delivers every
                    // notification — skip the ADB pipeline to avoid duplicates.
                    if companion_for_forwarder.has_connected_device().await {
                        continue;
                    }

                    // Collapse chatty updates (download progress etc.):
                    // identical repeat payloads are dropped entirely.
                    // NOTE: post_time is intentionally NOT part of the signature —
                    // apps bump it when re-posting an unchanged notification.
                    let signature = format!(
                        "{}|{}|{}",
                        item.title.as_deref().unwrap_or(""),
                        item.body.as_deref().unwrap_or(""),
                        item.subtext.as_deref().unwrap_or("")
                    );

                    if item.status == notifications::NotificationStatus::Removed {
                        notifications::UpdateGate::forget(&item.id);
                    } else if !notifications::UpdateGate::should_forward(&item.id, &signature) {
                        continue;
                    }

                    if item.status != notifications::NotificationStatus::Removed {
                        let _ = db_ref.insert_notification(&item);
                        let _ = handle_notif.emit("notification-received", &item);

                        // Show Windows Desktop Toast Notification via Native App ID
                        // Only for genuinely new/changed messages or OTP codes —
                        // never once-per-second progress ticks.
                        if item.status == notifications::NotificationStatus::Posted
                            || item.status == notifications::NotificationStatus::Updated
                        {
                            if !notifications::UpdateGate::should_show_toast(&item.id, &signature, item.is_otp) {
                                continue;
                            }

                            let sender_title = item.app_name.clone().unwrap_or_else(|| item.package_name.clone());
                            let display_title = if let Some(ref t) = item.title {
                                format!("{}: {}", sender_title, t)
                            } else {
                                sender_title
                            };

                            let body_text = if let Some(ref otp) = item.otp_code {
                                format!("Verification Code: {}\n{}", otp, item.body.clone().unwrap_or_default())
                            } else {
                                item.body.clone().unwrap_or_else(|| "New notification received".to_string())
                            };

                            // Single Native Windows Toast with "Notify" Identity
                            notifications::DesktopNotifier::show(&display_title, &body_text);
                        }
                    }
                }
            });

            app.manage(AppState {
                adb_client,
                connection_manager: conn_manager,
                notification_engine: notif_engine,
                companion_server,
                database,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_adb_status,
            pair_device,
            connect_device,
            connect_by_serial,
            get_saved_devices,
            delete_saved_device,
            get_companion_pairing_qr,
            get_connected_companion,
            send_companion_reply,
            disconnect_device,
            get_connection_state,
            get_active_device,
            scan_mdns,
            get_telemetry,
            send_keyevent,
            volume_up,
            volume_down,
            media_play_pause,
            media_next,
            media_prev,
            wake_screen,
            lock_screen,
            check_capabilities,
            copy_otp_to_clipboard,
            get_notification_history,
            delete_notification,
            clear_notification_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
