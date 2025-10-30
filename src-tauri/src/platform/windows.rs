// Windows-specific screen recording implementation using FFmpeg with gdigrab

use crate::models::recording::{
    PermissionResult, PermissionStatus, RecordingSource, RecordingSources, ScreenSource,
    WindowSource,
};
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref ACTIVE_RECORDINGS: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Request recording permissions on Windows
/// Windows handles permissions automatically when accessing capture devices
pub fn request_permissions(permissions: Vec<String>) -> Result<PermissionResult, String> {
    let mut status = PermissionStatus {
        screen: false,
        camera: false,
        microphone: false,
    };

    for permission in permissions {
        match permission.as_str() {
            "screen" => {
                // Windows doesn't require explicit screen recording permissions
                // Access is granted by default (with UAC prompt if needed)
                status.screen = true;
            }
            "camera" => {
                // Camera permissions are handled by Windows automatically
                status.camera = true;
            }
            "microphone" => {
                // Microphone permissions are handled by Windows
                status.microphone = true;
            }
            _ => {
                return Err(format!("Unknown permission: {}", permission));
            }
        }
    }

    Ok(PermissionResult { granted: status })
}

/// List available recording sources on Windows
pub fn list_sources() -> Result<RecordingSources, String> {
    let screens = list_screens()?;
    let windows = list_windows()?;
    let cameras = list_cameras()?;

    Ok(RecordingSources {
        screens,
        windows,
        cameras,
    })
}

fn list_screens() -> Result<Vec<ScreenSource>, String> {
    // Use FFmpeg to list DirectShow devices
    // In a full implementation, would use Windows API to enumerate displays

    // For MVP, return primary display
    Ok(vec![ScreenSource {
        id: "desktop".to_string(),
        name: "Primary Display".to_string(),
        resolution: "1920x1080".to_string(), // Would be detected from system
    }])
}

fn list_windows() -> Result<Vec<WindowSource>, String> {
    // In a full implementation, would use EnumWindows API
    // For MVP, return empty (screen recording is priority)
    Ok(vec![])
}

fn list_cameras() -> Result<Vec<RecordingSource>, String> {
    // Use FFmpeg to list DirectShow video devices
    let output = Command::new("ffmpeg")
        .args(&["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
        .output()
        .map_err(|e| format!("Failed to list cameras: {}", e))?;

    // FFmpeg outputs device list to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse camera devices from output
    // Format: [dshow @ ...] "Camera Name" (video)
    let mut cameras = Vec::new();

    for line in stderr.lines() {
        if line.contains("(video)") && line.contains("\"") {
            // Extract device name between quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let name = &line[start + 1..start + 1 + end];
                    cameras.push(RecordingSource {
                        id: name.to_string(),
                        name: name.to_string(),
                    });
                }
            }
        }
    }

    Ok(cameras)
}

/// Start recording using FFmpeg with gdigrab (screen) and dshow (camera) on Windows
pub fn start_recording(
    session_id: String,
    output_path: String,
    screen_source: Option<String>,
    camera_source: Option<String>,
    audio_sources: Vec<String>,
    resolution: String,
    fps: u32,
) -> Result<(), String> {
    let mut ffmpeg_args = vec!["-y".to_string()]; // Overwrite output file

    let has_screen = screen_source.is_some();
    let has_camera = camera_source.is_some();
    let has_audio = !audio_sources.is_empty();

    if has_screen {
        // Screen capture using gdigrab (Windows GDI-based screen capture)
        ffmpeg_args.extend_from_slice(&[
            "-f".to_string(),
            "gdigrab".to_string(),
            "-framerate".to_string(),
            fps.to_string(),
            "-i".to_string(),
            "desktop".to_string(), // Capture entire desktop
        ]);
    }

    if has_camera {
        // Camera capture using DirectShow
        let camera_name = camera_source.unwrap_or_else(|| "video=0".to_string());

        ffmpeg_args.extend_from_slice(&[
            "-f".to_string(),
            "dshow".to_string(),
            "-i".to_string(),
            format!("video={}", camera_name),
        ]);
    }

    if has_audio {
        // Audio capture using DirectShow
        if audio_sources.contains(&"microphone".to_string()) {
            // Default microphone
            ffmpeg_args.extend_from_slice(&[
                "-f".to_string(),
                "dshow".to_string(),
                "-i".to_string(),
                "audio=Microphone".to_string(),
            ]);
        }
    }

    // Video codec settings - web-compatible H.264
    ffmpeg_args.extend_from_slice(&[
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "ultrafast".to_string(),
        "-crf".to_string(),
        "23".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(), // Critical: ensures web/QuickTime compatibility
    ]);

    // Audio codec settings
    if has_audio {
        ffmpeg_args.extend_from_slice(&[
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "192k".to_string(),
        ]);
    }

    // Output resolution
    if resolution != "source" {
        ffmpeg_args.extend_from_slice(&["-s".to_string(), resolution.clone()]);
    }

    // MP4-specific flags for proper file structure
    ffmpeg_args.extend_from_slice(&[
        "-movflags".to_string(),
        "+faststart".to_string(), // Enable progressive playback and proper MP4 structure
    ]);

    // Output file
    ffmpeg_args.push(output_path.clone());

    // Start FFmpeg process with stdin pipe for graceful shutdown
    let child = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .stdin(std::process::Stdio::piped()) // Enable stdin for 'q' command
        .stdout(std::process::Stdio::null()) // Suppress stdout
        .stderr(std::process::Stdio::piped()) // Capture progress/errors
        .spawn()
        .map_err(|e| format!("Failed to start FFmpeg: {}", e))?;

    // Store the process handle
    let mut recordings = ACTIVE_RECORDINGS.lock().unwrap();
    recordings.insert(session_id, child);

    Ok(())
}

/// Stop an active recording gracefully
pub fn stop_recording(session_id: String) -> Result<(), String> {
    let mut recordings = ACTIVE_RECORDINGS.lock().unwrap();

    if let Some(mut child) = recordings.remove(&session_id) {
        // Try graceful shutdown first by sending 'q' to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            // Send 'q' to FFmpeg to trigger graceful shutdown
            let _ = stdin.write_all(b"q\n");
            let _ = stdin.flush();
            drop(stdin); // Close stdin to signal end
        }

        // Wait up to 5 seconds for graceful shutdown
        use std::time::{Duration, Instant};
        let start = Instant::now();
        let timeout = Duration::from_secs(5);

        loop {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Process exited gracefully
                    return Ok(());
                }
                Ok(None) => {
                    // Still running
                    if start.elapsed() > timeout {
                        // Timeout - force kill as last resort
                        let _ = child.kill();
                        child
                            .wait()
                            .map_err(|e| format!("Failed to wait for FFmpeg: {}", e))?;
                        return Ok(());
                    }
                    // Wait a bit before checking again
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(format!("Error checking FFmpeg process: {}", e));
                }
            }
        }
    } else {
        Err(format!("Recording session '{}' not found", session_id))
    }
}

/// Check if a recording is still active
pub fn is_recording_active(session_id: &str) -> bool {
    let recordings = ACTIVE_RECORDINGS.lock().unwrap();
    recordings.contains_key(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_permissions() {
        let permissions = vec!["screen".to_string(), "camera".to_string()];
        let result = request_permissions(permissions);

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.granted.screen);
        assert!(status.granted.camera);
    }

    #[test]
    fn test_list_screens() {
        let result = list_screens();
        assert!(result.is_ok());

        let screens = result.unwrap();
        assert!(!screens.is_empty());
    }
}
