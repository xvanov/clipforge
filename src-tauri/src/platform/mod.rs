// Platform-specific recording implementations

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

// Re-export platform-specific functions with a common interface
pub use platform_impl::*;

#[cfg(target_os = "macos")]
mod platform_impl {
    pub use super::macos::*;
}

#[cfg(target_os = "windows")]
mod platform_impl {
    pub use super::windows::*;
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform_impl {
    use crate::models::recording::{PermissionResult, PermissionStatus, RecordingSources};

    pub fn request_permissions(_permissions: Vec<String>) -> Result<PermissionResult, String> {
        Err("Recording not supported on this platform".to_string())
    }

    pub fn list_sources() -> Result<RecordingSources, String> {
        Err("Recording not supported on this platform".to_string())
    }

    pub fn start_recording(
        _session_id: String,
        _output_path: String,
        _screen_source: Option<String>,
        _camera_source: Option<String>,
        _audio_sources: Vec<String>,
        _resolution: String,
        _fps: u32,
    ) -> Result<(), String> {
        Err("Recording not supported on this platform".to_string())
    }

    pub fn stop_recording(_session_id: String) -> Result<(), String> {
        Err("Recording not supported on this platform".to_string())
    }
}
