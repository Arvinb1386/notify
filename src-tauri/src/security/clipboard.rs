use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct SecureClipboard;

impl SecureClipboard {
    /// Copies text to OS clipboard and schedules an auto-clear task after TTL seconds
    pub fn copy_with_ttl(text: String, ttl_secs: u64) -> Result<(), String> {
        let mut ctx = arboard::Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
        ctx.set_text(text.clone()).map_err(|e| format!("Failed to set clipboard: {}", e))?;
        info!("Text copied to clipboard with {}s TTL", ttl_secs);

        let target_text = text;
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(ttl_secs)).await;
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(current) = clipboard.get_text() {
                    if current == target_text {
                        let _ = clipboard.clear();
                        info!("Auto-cleared sensitive OTP from clipboard after TTL");
                    }
                }
            }
        });

        Ok(())
    }

    /// Pure manual copy without TTL
    pub fn copy_text(text: String) -> Result<(), String> {
        let mut ctx = arboard::Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
        ctx.set_text(text).map_err(|e| format!("Failed to set clipboard: {}", e))
    }
}
