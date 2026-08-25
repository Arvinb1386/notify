use serde::{Deserialize, Serialize};
use tracing::info;

use crate::adb::client::AdbClient;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceCapabilities {
    pub supports_volume: bool,
    pub supports_media: bool,
    pub supports_wake: bool,
    pub supports_lock: bool,
}

pub struct RemoteControls;

impl RemoteControls {
    pub async fn send_keyevent(client: &AdbClient, serial: &str, keycode: u32) -> AppResult<()> {
        info!("Sending keyevent {} to {}", keycode, serial);
        client.shell(serial, &["input", "keyevent", &keycode.to_string()]).await?;
        Ok(())
    }

    pub async fn volume_up(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 24).await
    }

    pub async fn volume_down(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 25).await
    }

    pub async fn volume_mute(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 164).await
    }

    pub async fn media_play_pause(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 85).await
    }

    pub async fn media_next(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 87).await
    }

    pub async fn media_prev(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 88).await
    }

    pub async fn wake_screen(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 224).await
    }

    pub async fn lock_screen(client: &AdbClient, serial: &str) -> AppResult<()> {
        Self::send_keyevent(client, serial, 26).await
    }

    pub async fn check_capabilities(client: &AdbClient, serial: &str) -> DeviceCapabilities {
        DeviceCapabilities {
            supports_volume: true,
            supports_media: true,
            supports_wake: true,
            supports_lock: true,
        }
    }
}
