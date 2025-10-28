// Playback control commands
use crate::commands::media::AppState;
use tauri::State;

/// T037: Load clip for playback in video preview
#[tauri::command]
pub async fn load_clip_for_playback(
    clip_id: String,
    use_proxy: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let library = state.media_library.lock().unwrap();
    let clip = library
        .iter()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| format!("Media clip not found: {}", clip_id))?;

    // Prefer proxy if available for better web compatibility
    // Otherwise fall back to source path
    let playback_path = if use_proxy && clip.proxy_path.is_some() {
        clip.proxy_path.as_ref().unwrap().clone()
    } else if clip.proxy_path.is_some() {
        // Even if not explicitly requested, use proxy if available (better compatibility)
        clip.proxy_path.as_ref().unwrap().clone()
    } else {
        clip.source_path.clone()
    };

    println!("load_clip_for_playback: clip_id={}, use_proxy={}, has_proxy={}, returning: {}", 
             clip_id, use_proxy, clip.proxy_path.is_some(), playback_path);

    Ok(playback_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_url_encoding() {
        let path = "/Users/test/Videos/my video.mp4";
        let encoded = urlencoding::encode(path);
        assert!(encoded.contains("%20")); // Space should be encoded
    }
}

