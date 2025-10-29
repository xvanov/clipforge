use crate::models::recording::*;
use crate::platform;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tokio::time::{interval, Duration};

lazy_static::lazy_static! {
    static ref RECORDING_SESSIONS: Arc<Mutex<HashMap<String, RecordingSession>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Request system permissions for screen/camera/microphone recording
#[tauri::command]
pub async fn request_recording_permissions(
    permissions: Vec<String>,
) -> Result<PermissionResult, String> {
    platform::request_permissions(permissions)
}

/// List available screens, windows, and cameras
#[tauri::command]
pub async fn list_recording_sources() -> Result<RecordingSources, String> {
    platform::list_sources()
}

/// Start a new recording session
#[tauri::command]
pub async fn start_recording(
    config: RecordingConfig,
    app_handle: AppHandle,
) -> Result<RecordingSession, String> {
    // Generate output path
    let output_dir = get_recordings_dir()?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("recording_{}.mp4", timestamp);
    let output_path = output_dir.join(&filename);
    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| "Invalid output path".to_string())?
        .to_string();

    // Create recording session
    let mut session = RecordingSession::new(
        config.recording_type.clone(),
        output_path_str.clone(),
        config.settings.resolution.clone(),
        config.settings.fps,
    );

    // Set sources
    session.screen_source = config.screen_source_id.clone();
    session.camera_device = config.camera_device_id.clone();
    session.audio_sources = config.audio_sources.clone();

    // Validate configuration
    session.validate()?;

    // Start platform-specific recording
    let session_id = session.id.clone();

    #[cfg(target_os = "macos")]
    platform::macos::start_recording(
        session_id.clone(),
        output_path_str,
        config.screen_source_id,
        config.camera_device_id,
        config.audio_sources,
        config.microphone_device_id,
        config.settings.resolution,
        config.settings.fps,
    )?;

    #[cfg(target_os = "windows")]
    platform::windows::start_recording(
        session_id.clone(),
        output_path_str,
        config.screen_source_id,
        config.camera_device_id,
        config.audio_sources,
        config.settings.resolution,
        config.settings.fps,
    )?;

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return Err("Recording not supported on this platform".to_string());

    // Update session status
    session.start();

    // Store session
    let session_clone = session.clone();
    {
        let mut sessions = RECORDING_SESSIONS.lock().unwrap();
        sessions.insert(session_id.clone(), session.clone());
    }

    // Emit recording_started event
    app_handle
        .emit_all(
            "recording_started",
            json!({
                "session_id": session_id
            }),
        )
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    // Start duration tracking task
    start_duration_tracking(session_id, app_handle);

    Ok(session_clone)
}

/// Stop an active recording session
#[tauri::command]
pub async fn stop_recording(
    session_id: String,
    app_handle: AppHandle,
) -> Result<crate::models::clip::MediaClip, String> {
    use crate::commands::media::AppState;
    use tauri::Manager;

    // Get session
    let mut session = {
        let mut sessions = RECORDING_SESSIONS.lock().unwrap();
        sessions
            .remove(&session_id)
            .ok_or_else(|| format!("Recording session not found: {}", session_id))?
    };

    // Stop platform-specific recording
    #[cfg(target_os = "macos")]
    platform::macos::stop_recording(session_id.clone())?;

    #[cfg(target_os = "windows")]
    platform::windows::stop_recording(session_id.clone())?;

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return Err("Recording not supported on this platform".to_string());

    // Update session status
    session.stop();

    // Create MediaClip from recording
    let media_clip = create_media_clip_from_recording(&session, &app_handle).await?;

    // Store the created clip ID
    session.created_media_clip_id = Some(media_clip.id.clone());

    // Add clip to AppState (so it can be played back)
    let app_state = app_handle.state::<AppState>();
    
    // Add to media library
    {
        let mut library = app_state.media_library.lock().unwrap();
        library.push(media_clip.clone());
    }

    // Add to cache database
    {
        let cache_db = app_state.cache_db.lock().unwrap();
        cache_db.insert_media_clip(&media_clip)?;
    }

    // Add to project's media library if a project is loaded
    {
        let mut project_lock = app_state.project.lock().unwrap();
        if let Some(ref mut project) = *project_lock {
            project.media_library.push(media_clip.clone());
        }
    }

    // Emit recording_stopped event
    app_handle
        .emit_all(
            "recording_stopped",
            json!({
                "session_id": session_id,
                "media_clip_id": media_clip.id
            }),
        )
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(media_clip)
}

/// Start duration tracking task (runs every second)
fn start_duration_tracking(session_id: String, app_handle: AppHandle) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
        let mut elapsed = 0.0;

        loop {
            ticker.tick().await;

            // Check if session still exists
            let session_exists = {
                let sessions = RECORDING_SESSIONS.lock().unwrap();
                sessions.contains_key(&session_id)
            };

            if !session_exists {
                // Recording stopped, exit tracking
                break;
            }

            // Update elapsed time
            elapsed += 1.0;

            // Update session duration
            {
                let mut sessions = RECORDING_SESSIONS.lock().unwrap();
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.update_duration(elapsed);
                }
            }

            // Emit progress event
            let _ = app_handle.emit_all(
                "recording_progress",
                json!({
                    "session_id": session_id,
                    "duration": elapsed
                }),
            );
        }
    });
}

/// Create MediaClip from completed recording
async fn create_media_clip_from_recording(
    session: &RecordingSession,
    app_handle: &AppHandle,
) -> Result<crate::models::clip::MediaClip, String> {
    use crate::ffmpeg::metadata::extract_metadata;
    use crate::models::clip::MediaClip;

    // Validate file exists and has content
    let metadata_fs = std::fs::metadata(&session.output_path)
        .map_err(|e| format!("Recording file not found: {}", e))?;

    if metadata_fs.len() == 0 {
        return Err("Recording file is empty - recording may have failed".to_string());
    }

    // Extract metadata from recorded file
    let metadata = extract_metadata(&session.output_path)
        .await
        .map_err(|e| format!("Failed to get metadata from recording: {}", e))?;

    // Create MediaClip
    let clip = MediaClip {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("Recording {}", chrono::Utc::now().format("%Y-%m-%d %H:%M")),
        source_path: session.output_path.clone(),
        proxy_path: None,
        thumbnail_path: None,
        duration: session.duration.unwrap_or(0.0),
        resolution: metadata.resolution,
        width: metadata.width as i32,
        height: metadata.height as i32,
        fps: metadata.fps,
        codec: metadata.codec,
        audio_codec: metadata.audio_codec,
        file_size: metadata_fs.len() as i64,
        bitrate: metadata.bitrate.map(|b| b as i32),
        has_audio: metadata.has_audio,
        imported_at: chrono::Utc::now(),
        captions: Vec::new(),
    };

    // Generate thumbnail asynchronously
    let clip_id = clip.id.clone();
    let output_path = session.output_path.clone();
    let app_handle_clone = app_handle.clone();

    tokio::spawn(async move {
        // Create thumbnail directory
        let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let thumbnail_dir = home_dir.join(".clipforge").join("thumbnails");
        std::fs::create_dir_all(&thumbnail_dir).ok();

        let thumbnail_path = thumbnail_dir.join(format!("{}.jpg", clip_id));
        let thumbnail_path_str = thumbnail_path.to_str().unwrap_or("").to_string();

        if crate::ffmpeg::thumbnails::generate_thumbnail(&output_path, &thumbnail_path_str, 0.0)
            .await
            .is_ok()
        {
            // Update clip with thumbnail path
            let _ = app_handle_clone.emit_all(
                "thumbnail_generated",
                json!({
                    "clip_id": clip_id,
                    "thumbnail_path": thumbnail_path_str
                }),
            );
        }
    });

    Ok(clip)
}

/// Get the recordings directory (platform-specific)
fn get_recordings_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Cannot find home directory".to_string())?;

    // Use platform-specific default directories
    #[cfg(target_os = "macos")]
    let recordings_dir = home_dir.join("Movies").join("ClipForge Recordings");

    #[cfg(not(target_os = "macos"))]
    let recordings_dir = home_dir.join("Videos").join("ClipForge Recordings");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {}", e))?;

    Ok(recordings_dir)
}

/// Get an active recording session (for testing/debugging)
#[tauri::command]
pub async fn get_recording_session(session_id: String) -> Result<RecordingSession, String> {
    let sessions = RECORDING_SESSIONS.lock().unwrap();
    sessions
        .get(&session_id)
        .cloned()
        .ok_or_else(|| format!("Recording session not found: {}", session_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_recordings_dir() {
        let result = get_recordings_dir();

        // Should either succeed or fail with a known error
        // (on some systems the Videos directory might not exist)
        match result {
            Ok(dir) => {
                // Directory should be created if it doesn't exist
                assert!(dir.to_str().unwrap().contains("ClipForge Recordings"));
                assert!(dir.exists());
            }
            Err(_) => {
                // It's acceptable to fail if we can't create the directory
                // (e.g., in sandboxed environments)
            }
        }
    }

    #[tokio::test]
    async fn test_request_recording_permissions() {
        let permissions = vec!["screen".to_string()];
        let result = request_recording_permissions(permissions).await;

        // Should succeed on all platforms
        assert!(result.is_ok());
    }
}
