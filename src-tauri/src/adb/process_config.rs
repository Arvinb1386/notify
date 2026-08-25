#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// On Windows, prevents command prompt flashing during background ADB operations.
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn configure_tokio_command(cmd: &mut tokio::process::Command) {
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

pub fn configure_std_command(cmd: &mut std::process::Command) {
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}
