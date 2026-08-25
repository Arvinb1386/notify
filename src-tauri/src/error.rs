use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "code", content = "message")]
pub enum AppError {
    #[error("ADB binary not found on system or bundled path")]
    AdbNotFound,

    #[error("Pairing failed: {0}")]
    PairingFailed(String),

    #[error("Pairing timed out")]
    PairingTimeout,

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("Device is offline or disconnected: {0}")]
    DeviceOffline(String),

    #[error("mDNS discovery failed or timed out: {0}")]
    MdnsUnavailable(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse output: {0}")]
    ParserFailed(String),

    #[error("Feature not supported on device: {0}")]
    UnsupportedFeature(String),

    #[error("Permission denied on device")]
    PermissionDenied,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type AppResult<T> = Result<T, AppError>;
