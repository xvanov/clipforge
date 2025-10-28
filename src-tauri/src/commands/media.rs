// Media command implementation for import, metadata extraction, and thumbnail generation

use crate::ffmpeg::{extract_metadata, generate_proxy, generate_thumbnail, needs_proxy};
use crate::models::clip::MediaClip;
use crate::storage::cache::CacheDb;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    pub cache_db: Arc<Mutex<CacheDb>>,
    pub media_library: Arc<Mutex<Vec<MediaClip>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub clips: Vec<MediaClip>,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportError {
    pub path: String,
    pub error: String,
}

/// T027: Import media files into media library
#[tauri::command]
pub async fn import_media_files(
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    let mut clips = Vec::new();
    let mut errors = Vec::new();

    for path in paths {
        match import_single_file(&path, &state).await {
            Ok(clip) => clips.push(clip),
            Err(e) => errors.push(ImportError {
                path: path.clone(),
                error: e,
            }),
        }
    }

    // Add successfully imported clips to media library
    if !clips.is_empty() {
        let mut library = state.media_library.lock().unwrap();
        library.extend(clips.clone());
    }

    Ok(ImportResult { clips, errors })
}

async fn import_single_file(path: &str, state: &State<'_, AppState>) -> Result<MediaClip, String> {
    // Validate file exists
    let file_path = PathBuf::from(path);
    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    // Extract metadata using FFmpeg
    let metadata = extract_metadata(path).await?;

    // Generate clip ID and thumbnail path
    let clip_id = Uuid::new_v4().to_string();
    let cache_dir = get_cache_dir()?;
    let thumbnail_dir = cache_dir.join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir)
        .map_err(|e| format!("Failed to create thumbnail directory: {}", e))?;
    let thumbnail_path = thumbnail_dir.join(format!("{}.jpg", clip_id));

    // Generate thumbnail at 1 second mark (or 0 if video is shorter)
    let timestamp = if metadata.duration > 1.0 { 1.0 } else { 0.0 };
    let thumbnail_path_str = thumbnail_path
        .to_str()
        .ok_or("Invalid thumbnail path")?
        .to_string();

    match generate_thumbnail(path, &thumbnail_path_str, timestamp).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Warning: Failed to generate thumbnail: {}", e);
            // Continue without thumbnail
        }
    }

    // Check if we need to generate a proxy for web playback
    let proxy_path = if needs_proxy(&metadata.codec) {
        let proxy_dir = cache_dir.join("proxies");
        std::fs::create_dir_all(&proxy_dir)
            .map_err(|e| format!("Failed to create proxy directory: {}", e))?;
        let proxy_file = proxy_dir.join(format!("{}.mp4", clip_id));
        let proxy_path_str = proxy_file
            .to_str()
            .ok_or("Invalid proxy path")?
            .to_string();

        // Generate proxy in background (don't block import)
        let path_clone = path.to_string();
        let proxy_clone = proxy_path_str.clone();
        let clip_id_clone = clip_id.clone();
        let state_clone = state.inner().clone();
        
        tokio::spawn(async move {
            match generate_proxy(&path_clone, &proxy_clone).await {
                Ok(_) => {
                    println!("✓ Proxy generated for clip {}", clip_id_clone);
                    println!("  Proxy path: {}", proxy_clone);
                    
                    // Update the clip in the library with the proxy path
                    let mut library = state_clone.media_library.lock().unwrap();
                    if let Some(clip) = library.iter_mut().find(|c| c.id == clip_id_clone) {
                        clip.proxy_path = Some(proxy_clone.clone());
                        println!("  Updated clip in library with proxy path");
                        
                        // Update cache database
                        let cache_db = state_clone.cache_db.lock().unwrap();
                        if let Err(e) = cache_db.insert_media_clip(clip) {
                            eprintln!("Failed to update clip with proxy path: {}", e);
                        } else {
                            println!("  Updated cache database with proxy path");
                        }
                    } else {
                        eprintln!("  ERROR: Could not find clip {} in library to update proxy path", clip_id_clone);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to generate proxy for {}: {}", clip_id_clone, e);
                }
            }
        });

        // Return None for now - will be updated when proxy generation completes
        None
    } else {
        // No proxy needed for web-compatible formats
        None
    };

    // Get file size
    let file_size = std::fs::metadata(&file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Get file name for display
    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Create MediaClip
    let clip = MediaClip {
        id: clip_id,
        name,
        source_path: path.to_string(),
        proxy_path,
        thumbnail_path: if thumbnail_path.exists() {
            Some(thumbnail_path_str)
        } else {
            None
        },
        duration: metadata.duration,
        resolution: metadata.resolution,
        width: metadata.width as i32,
        height: metadata.height as i32,
        fps: metadata.fps,
        codec: metadata.codec,
        audio_codec: metadata.audio_codec,
        file_size: file_size as i64,
        bitrate: metadata.bitrate.map(|b| b as i32),
        has_audio: metadata.has_audio,
        imported_at: chrono::Utc::now(),
        captions: vec![],
    };

    // Store in cache database
    let cache_db = state.cache_db.lock().unwrap();
    cache_db.insert_media_clip(&clip)?;

    Ok(clip)
}

/// T028: Get metadata for a specific clip
#[tauri::command]
pub async fn get_media_metadata(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<MediaClip, String> {
    let library = state.media_library.lock().unwrap();
    library
        .iter()
        .find(|c| c.id == clip_id)
        .cloned()
        .ok_or_else(|| format!("Media clip not found: {}", clip_id))
}

/// T030: Generate thumbnail for existing clip
#[tauri::command]
pub async fn generate_thumbnail_for_clip(
    clip_id: String,
    timestamp: f64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Get the source path from the library, then drop the lock before async operation
    let source_path = {
        let library = state.media_library.lock().unwrap();
        let clip = library
            .iter()
            .find(|c| c.id == clip_id)
            .ok_or_else(|| format!("Media clip not found: {}", clip_id))?;
        clip.source_path.clone()
    }; // MutexGuard is dropped here

    let cache_dir = get_cache_dir()?;
    let thumbnail_dir = cache_dir.join("thumbnails");
    let thumbnail_path = thumbnail_dir.join(format!("{}.jpg", clip_id));
    let thumbnail_path_str = thumbnail_path
        .to_str()
        .ok_or("Invalid thumbnail path")?
        .to_string();

    generate_thumbnail(&source_path, &thumbnail_path_str, timestamp).await?;

    Ok(thumbnail_path_str)
}

/// Get cache directory path
fn get_cache_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let cache_dir = home_dir.join(".clipforge").join("cache");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    Ok(cache_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_dir() {
        let result = get_cache_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().contains(".clipforge"));
    }
}
