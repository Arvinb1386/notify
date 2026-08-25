use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tracing::{debug, error, info};

use super::process_config::configure_tokio_command;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct AdbClient {
    pub binary_path: PathBuf,
}

impl AdbClient {
    /// Detects the best available ADB binary (custom path, system PATH, or winget/bundled scrcpy).
    pub fn new(custom_path: Option<String>) -> AppResult<Self> {
        if let Some(path_str) = custom_path {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                info!("Using custom ADB path: {:?}", path);
                return Ok(Self { binary_path: path });
            }
        }

        // Try standard PATH
        if let Ok(which_path) = which::which("adb") {
            info!("Found system ADB in PATH: {:?}", which_path);
            return Ok(Self {
                binary_path: which_path,
            });
        }

        // Try common fallback locations on Windows
        #[cfg(target_os = "windows")]
        {
            let candidates = [
                // Winget / scrcpy location
                dirs::data_local_dir().map(|p| {
                    p.join("Microsoft/WinGet/Packages/Genymobile.scrcpy_Microsoft.Winget.Source_8wekyb3d8bbwe/scrcpy-win64-v4.1/adb.exe")
                }),
                // Android SDK Platform Tools standard path
                dirs::data_local_dir().map(|p| p.join("Android/Sdk/platform-tools/adb.exe")),
                // Program files
                Some(PathBuf::from("C:\\Program Files\\Android\\platform-tools\\adb.exe")),
            ];

            for candidate in candidates.into_iter().flatten() {
                if candidate.exists() {
                    info!("Found ADB at fallback candidate: {:?}", candidate);
                    return Ok(Self {
                        binary_path: candidate,
                    });
                }
            }
        }

        // Check relative resources/platform-tools
        let bundled = Path::new("resources/platform-tools/adb.exe");
        if bundled.exists() {
            return Ok(Self {
                binary_path: bundled.to_path_buf(),
            });
        }

        error!("No valid ADB binary found on system");
        Err(AppError::AdbNotFound)
    }

    /// Verifies that ADB binary works and returns its version string.
    pub async fn check_version(&self) -> AppResult<String> {
        let output = self.execute_raw(&["version"]).await?;
        Ok(output)
    }

    /// Creates a configured tokio command for the given arguments.
    pub fn create_command(&self, args: &[&str]) -> TokioCommand {
        let mut cmd = TokioCommand::new(&self.binary_path);
        cmd.args(args);
        configure_tokio_command(&mut cmd);
        cmd
    }

    /// Executes an ADB command asynchronously and returns stdout string.
    pub async fn execute_raw(&self, args: &[&str]) -> AppResult<String> {
        debug!("Executing ADB command: {:?}", args);
        let mut cmd = self.create_command(args);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| AppError::CommandFailed(format!("Failed to spawn ADB process: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let err_msg = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else {
                stdout.trim().to_string()
            };
            return Err(AppError::CommandFailed(err_msg));
        }

        Ok(stdout)
    }

    /// Executes a shell command on the target device: adb -s <serial> shell <subcommand...>
    pub async fn shell(&self, serial: &str, shell_args: &[&str]) -> AppResult<String> {
        let mut args = vec!["-s", serial, "shell"];
        args.extend_from_slice(shell_args);
        self.execute_raw(&args).await
    }
}
