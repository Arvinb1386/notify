use tracing::{error, info};

pub struct DesktopNotifier;

impl DesktopNotifier {
    /// Registers the AppUserModelId and Icon for Notify in the Windows Registry (HKCU) on app launch
    pub fn ensure_app_id_registered() {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            use crate::adb::process_config::configure_std_command;

            let script = r#"
                $appId = 'com.notify.desktop'
                $regPath = "HKCU:\Software\Classes\AppUserModelId\$appId"
                if (!(Test-Path $regPath)) {
                    New-Item -Path $regPath -Force | Out-Null
                }
                Set-ItemProperty -Path $regPath -Name 'DisplayName' -Value 'Notify'
                Set-ItemProperty -Path $regPath -Name 'IconUri' -Value 'D:\git-project\notify\src-tauri\icons\icon.ico'
                Set-ItemProperty -Path $regPath -Name 'ShowInSettings' -Value 1
            "#;

            let utf16: Vec<u8> = script.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
            let encoded = {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(&utf16)
            };

            let mut cmd = Command::new("powershell");
            cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", &encoded]);
            configure_std_command(&mut cmd);
            let _ = cmd.spawn();
        }
    }

    /// Dispatches a native Windows Toast notification displaying "Notify" app logo & branding
    pub fn show(title: &str, body: &str) {
        #[cfg(target_os = "windows")]
        {
            let t = title.to_string();
            let b = body.to_string();

            std::thread::spawn(move || {
                Self::show_native_windows_toast(&t, &b);
            });
        }
    }

    #[cfg(target_os = "windows")]
    fn show_native_windows_toast(title: &str, body: &str) {
        use std::process::Command;
        use crate::adb::process_config::configure_std_command;

        // Escape XML entities
        let safe_title = title
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;");

        let safe_body = body
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;");

        // Resolve absolute path to the icon, correctly formatted for URI scheme
        let exe_dir = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        // Look up to project root icons folder during dev, or use packaged path in release
        let possible_icon_paths = [
            exe_dir.join("icons/128x128.png"),
            exe_dir.join("../../icons/128x128.png"),
            std::path::PathBuf::from("D:/git-project/notify/src-tauri/icons/128x128.png")
        ];

        let mut final_icon_path = "D:/git-project/notify/src-tauri/icons/128x128.png".to_string();
        for p in possible_icon_paths {
            if p.exists() {
                final_icon_path = p.to_string_lossy().replace('\\', "/");
                break;
            }
        }

        let icon_uri = format!("file:///{}", final_icon_path);

        // Build PowerShell script using a single-line XML template to completely avoid PowerShell here-string parsing bugs
        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null;\
            [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null;\
            $xml = [Windows.Data.Xml.Dom.XmlDocument]::new();\
            $xml.LoadXml('<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text><image placement=\"appLogoOverride\" src=\"{}\" hint-crop=\"none\"/></binding></visual></toast>');\
            $toast = [Windows.UI.Notifications.ToastNotification]::new($xml);\
            [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('com.notify.desktop').Show($toast);",
            safe_title, safe_body, icon_uri
        );

        // Encode as UTF-16LE Base64 (Standard required by powershell -EncodedCommand)
        let utf16: Vec<u8> = script
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();

        let encoded = {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(&utf16)
        };

        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", &encoded]);
        configure_std_command(&mut cmd);

        if let Err(e) = cmd.spawn() {
            error!("Native Windows toast notification dispatch failed: {}", e);
        } else {
            info!("Windows Toast with App Logo dispatched successfully for: {}", title);
        }
    }
}
