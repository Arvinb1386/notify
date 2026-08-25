pub mod adb;
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

use adb::{AdbClient, AdbCommands, ConnectionManager, ConnectionState, DeviceInfo};
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
    let dev = state.connection_manager.connect(&host, port).await?;
    let _ = state.notification_engine.start_monitoring(dev.serial.clone()).await;
    Ok(dev)
}

#[tauri::command]
async fn disconnect_device(state: State<'_, AppState>) -> Result<(), AppError> {
    state.notification_engine.stop().await;
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
async fn get_notification_history(limit: Option<u32>, state: State<'_, AppState>) -> Result<Vec<NotificationItem>, AppError> {
    state.database.get_recent_notifications(limit.unwrap_or(100))
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

            // Setup system tray
            let _ = tray::setup_tray(&app_handle);

            // Forward connection status events to frontend
            let mut conn_rx = conn_manager.subscribe();
            let handle_conn = app_handle.clone();
            tokio::spawn(async move {
                while let Ok(event) = conn_rx.recv().await {
                    let _ = handle_conn.emit("connection-status-changed", event);
                }
            });

            // Forward notification events to frontend & save to DB
            let mut notif_rx = notif_engine.subscribe();
            let handle_notif = app_handle.clone();
            let db_ref = Arc::clone(&database);
            tokio::spawn(async move {
                while let Ok(item) = notif_rx.recv().await {
                    let _ = db_ref.insert_notification(&item);
                    let _ = handle_notif.emit("notification-received", &item);
                }
            });

            app.manage(AppState {
                adb_client,
                connection_manager: conn_manager,
                notification_engine: notif_engine,
                database,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_adb_status,
            pair_device,
            connect_device,
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
            clear_notification_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
