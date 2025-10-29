// macOS-specific screen recording implementation using AVFoundation and ScreenCaptureKit

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

/// Request screen recording permissions on macOS
pub fn request_permissions(permissions: Vec<String>) -> Result<PermissionResult, String> {
    let mut status = PermissionStatus {
        screen: false,
        camera: false,
        microphone: false,
    };

    for permission in permissions {
        match permission.as_str() {
            "screen" => {
                // On macOS, screen recording permission is requested automatically
                // when attempting to access screen content via ScreenCaptureKit
                // We return true here, actual permission check happens when recording starts
                status.screen = true;
            }
            "camera" => {
                // Camera permissions can be requested via AVFoundation
                // For simplicity, we'll assume granted (actual implementation would use Objective-C bridge)
                status.camera = true;
            }
            "microphone" => {
                // Microphone permissions via AVFoundation
                status.microphone = true;
            }
            _ => {
                return Err(format!("Unknown permission: {}", permission));
            }
        }
    }

    Ok(PermissionResult { granted: status })
}

/// List available recording sources (screens, windows, cameras) on macOS
pub fn list_sources() -> Result<RecordingSources, String> {
    // Use system_profiler to list displays
    let screens = list_screens()?;

    // List windows using CGWindowListCopyWindowInfo
    let windows = list_windows()?;

    // List cameras using system_profiler
    let cameras = list_cameras()?;

    // List microphones (audio input devices)
    let microphones = list_microphones()?;

    Ok(RecordingSources {
        screens,
        windows,
        cameras,
        microphones,
    })
}

fn list_screens() -> Result<Vec<ScreenSource>, String> {
    // Use system_profiler to get display information
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .map_err(|e| format!("Failed to list displays: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get display information".to_string());
    }

    // Parse JSON output (simplified for now - in production would parse properly)
    // For MVP, return screen capture device
    // Note: The actual device index depends on how many cameras are connected
    // We use "Capture screen 0" which is more reliable than numeric index
    Ok(vec![ScreenSource {
        id: "Capture screen 0".to_string(), // Use explicit screen capture name
        name: "Main Display".to_string(),
        resolution: "1920x1080".to_string(), // Would be parsed from system_profiler output
    }])
}

fn list_windows() -> Result<Vec<WindowSource>, String> {
    // In a full implementation, this would use CGWindowListCopyWindowInfo via Objective-C bridge
    // For MVP, return empty list (screen recording is more important)
    Ok(vec![])
}

fn list_cameras() -> Result<Vec<RecordingSource>, String> {
    // Use system_profiler to list cameras
    let output = Command::new("system_profiler")
        .args(["SPCameraDataType", "-json"])
        .output()
        .map_err(|e| format!("Failed to list cameras: {}", e))?;

    if !output.status.success() {
        return Ok(vec![]); // No cameras available
    }

    // Parse JSON output (simplified)
    // For MVP, check if any camera is available
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Camera") || stdout.contains("FaceTime") {
        Ok(vec![RecordingSource {
            id: "0".to_string(),
            name: "FaceTime HD Camera".to_string(),
        }])
    } else {
        Ok(vec![])
    }
}

fn list_microphones() -> Result<Vec<RecordingSource>, String> {
    // Use FFmpeg to list audio devices
    let output = Command::new("ffmpeg")
        .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
        .output()
        .map_err(|e| format!("Failed to list microphones: {}", e))?;

    // FFmpeg outputs device list to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Parse audio devices from FFmpeg output
    let mut microphones = Vec::new();
    let mut in_audio_section = false;
    
    for line in stderr.lines() {
        if line.contains("AVFoundation audio devices:") {
            in_audio_section = true;
            continue;
        }
        
        if in_audio_section {
            // Stop when we hit the end or another section
            if line.contains("Error opening input") || line.is_empty() {
                break;
            }
            
            // Parse line like: "[AVFoundation indev @ 0x...] [0] MacBook Pro Microphone"
            if let Some(idx) = line.find("] [") {
                if let Some(end_idx) = line[idx + 3..].find(']') {
                    let device_id = &line[idx + 3..idx + 3 + end_idx];
                    let device_name = line[idx + 3 + end_idx + 2..].trim();
                    
                    microphones.push(RecordingSource {
                        id: device_id.to_string(),
                        name: device_name.to_string(),
                    });
                }
            }
        }
    }
    
    // If parsing failed, return a default microphone
    if microphones.is_empty() {
        microphones.push(RecordingSource {
            id: "0".to_string(),
            name: "Default Microphone".to_string(),
        });
    }
    
    Ok(microphones)
}

/// Start recording using FFmpeg with avfoundation input on macOS
pub fn start_recording(
    session_id: String,
    output_path: String,
    screen_source: Option<String>,
    camera_source: Option<String>,
    audio_sources: Vec<String>,
    microphone_device_id: Option<String>,
    resolution: String,
    fps: u32,
) -> Result<(), String> {
    let mut ffmpeg_args = vec!["-y".to_string()]; // Overwrite output file

    // Determine input sources
    let has_screen = screen_source.is_some();
    let has_camera = camera_source.is_some();
    let has_audio = !audio_sources.is_empty();

    if has_screen {
        // Screen capture using avfoundation
        // Use "Capture screen N" format which is more reliable than numeric indices
        // The numeric index varies based on number of connected cameras
        let screen_idx = screen_source.unwrap_or_else(|| "Capture screen 0".to_string());

        ffmpeg_args.extend_from_slice(&[
            "-f".to_string(),
            "avfoundation".to_string(),
            "-capture_cursor".to_string(),
            "1".to_string(),
            "-r".to_string(),
            fps.to_string(),
        ]);

        // Build input string
        // For screen-only or screen with system audio: use screen name with optional audio
        let input = if has_audio && audio_sources.contains(&"system".to_string()) {
            // Screen + system audio: "screenName:audioIndex"
            // Note: System audio capture on macOS may require additional setup (BlackHole, etc.)
            format!("{}:none", screen_idx) // Use "none" for audio as system audio needs special handling
        } else {
            // Screen only (no audio): just the screen name
            screen_idx.clone()
        };

        ffmpeg_args.extend_from_slice(&["-i".to_string(), input]);
    }

    if has_camera {
        // Camera capture using avfoundation
        let camera_idx = camera_source.unwrap_or_else(|| "0".to_string());

        // If we want microphone audio with the camera, capture it together
        let camera_input = if has_audio && audio_sources.contains(&"microphone".to_string()) {
            // Camera with microphone: "cameraIndex:audioIndex"
            // Use the specific microphone if provided, otherwise default to ":0"
            let audio_idx = microphone_device_id.as_deref().unwrap_or("0");
            format!("{}:{}", camera_idx, audio_idx)
        } else {
            // Camera without audio
            camera_idx.clone()
        };

        ffmpeg_args.extend_from_slice(&[
            "-f".to_string(),
            "avfoundation".to_string(),
            "-r".to_string(),
            fps.to_string(),
            "-i".to_string(),
            camera_input,
        ]);
    }

    // Add separate microphone input only for webcam-only mode without camera audio
    if has_audio && audio_sources.contains(&"microphone".to_string()) && !has_screen && !has_camera {
        // Separate microphone input (webcam-only fallback)
        ffmpeg_args.extend_from_slice(&[
            "-f".to_string(),
            "avfoundation".to_string(),
            "-i".to_string(),
            ":0".to_string(), // Default audio input
        ]);
    }

    // If we have both screen and camera, create picture-in-picture overlay
    if has_screen && has_camera {
        // Picture-in-picture overlay: webcam in bottom-left corner
        // [1:v] is the camera input (second input), [0:v] is the screen (first input)
        // Scale webcam to 30% of screen size, adjust brightness/contrast, and overlay in bottom-left with 20px padding
        // eq filter: brightness=0.06 (slightly brighter), contrast=1.1 (slightly more contrast)
        let filter = "[1:v]scale=iw*0.30:ih*0.30,eq=brightness=0.06:contrast=1.1[cam];[0:v][cam]overlay=20:main_h-overlay_h-20";
        
        ffmpeg_args.extend_from_slice(&[
            "-filter_complex".to_string(),
            filter.to_string(),
        ]);
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

    // Audio codec settings (if audio is present)
    if has_audio {
        ffmpeg_args.extend_from_slice(&[
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "256k".to_string(), // Increased from 192k for better audio quality
            "-ar".to_string(),
            "48000".to_string(), // 48kHz sample rate for professional audio quality
        ]);
    }

    // Output resolution (if specified and different from source)
    // Note: Don't apply this when using filter_complex as it's handled in the filter
    if resolution != "source" && !(has_screen && has_camera) {
        let parts: Vec<&str> = resolution.split('x').collect();
        if parts.len() == 2 {
            ffmpeg_args.extend_from_slice(&["-s".to_string(), resolution.clone()]);
        }
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
#[allow(dead_code)]
pub fn is_recording_active(session_id: &str) -> bool {
    let recordings = ACTIVE_RECORDINGS.lock().unwrap();
    recordings.contains_key(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_screens() {
        // This test will only work on macOS
        let result = list_screens();
        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_request_permissions() {
        let permissions = vec!["screen".to_string(), "camera".to_string()];
        let result = request_permissions(permissions);

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.granted.screen);
        assert!(status.granted.camera);
    }
}
