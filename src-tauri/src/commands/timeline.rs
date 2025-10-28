use crate::commands::media::AppState;
use crate::models::timeline::{TimelineClip, Track, TrackType};
use tauri::State;

#[derive(serde::Deserialize)]
pub struct TimelineClipUpdates {
    pub start_time: Option<f64>,
    pub in_point: Option<f64>,
    pub out_point: Option<f64>,
    pub track_id: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SplitResult {
    pub clip_before: TimelineClip,
    pub clip_after: TimelineClip,
}

/// T048: Add clip to timeline
#[tauri::command]
pub async fn add_clip_to_timeline(
    media_clip_id: String,
    track_id: String,
    start_time: f64,
    in_point: f64,
    out_point: f64,
    state: State<'_, AppState>,
) -> Result<TimelineClip, String> {
    println!("add_clip_to_timeline called: media_clip={}, track={}, start={}", media_clip_id, track_id, start_time);
    
    // Validate inputs
    if in_point >= out_point {
        return Err("in_point must be less than out_point".to_string());
    }
    if start_time < 0.0 {
        return Err("start_time must be non-negative".to_string());
    }
    
    // Check if media clip exists
    let media_library = state.media_library.lock().unwrap();
    if !media_library.iter().any(|c| c.id == media_clip_id) {
        return Err(format!("Media clip not found: {}", media_clip_id));
    }
    drop(media_library);
    
    // Create timeline clip
    let timeline_clip = TimelineClip::new(
        media_clip_id,
        track_id.clone(),
        start_time,
        in_point,
        out_point,
    );
    
    println!("Created timeline clip: {:?}", timeline_clip);
    
    // Store in project state
    let mut project_lock = state.project.lock().unwrap();
    if let Some(ref mut project) = *project_lock {
        // Find the track and add the clip
        let track_found = project.tracks.iter_mut()
            .find(|t| t.id == track_id)
            .map(|track| {
                track.clips.push(timeline_clip.clone());
                track.clips.len()
            });
        
        if let Some(clip_count) = track_found {
            project.mark_modified();
            println!("Added clip to track. Track now has {} clips", clip_count);
        } else {
            return Err(format!("Track not found: {}", track_id));
        }
    } else {
        return Err("No project loaded".to_string());
    }
    
    Ok(timeline_clip)
}

/// T049: Update timeline clip properties
#[tauri::command]
pub async fn update_timeline_clip(
    clip_id: String,
    _updates: TimelineClipUpdates,
    _state: State<'_, AppState>,
) -> Result<TimelineClip, String> {
    // TODO: Implement update logic with project state
    // For now, return error
    Err(format!("Not fully implemented yet: {}", clip_id))
}

/// T050: Split timeline clip at specified time
#[tauri::command]
pub async fn split_timeline_clip(
    clip_id: String,
    split_time: f64,
    _state: State<'_, AppState>,
) -> Result<SplitResult, String> {
    // TODO: Implement split logic with project state
    // For now, return error
    Err(format!("Not fully implemented yet: {} at {}", clip_id, split_time))
}

/// T051: Delete timeline clip
#[tauri::command]
pub async fn delete_timeline_clip(
    clip_id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement delete logic with project state
    // For now, return error
    Err(format!("Not fully implemented yet: {}", clip_id))
}

/// T052: Create new track
#[tauri::command]
pub async fn create_track(
    name: String,
    track_type: String,
    state: State<'_, AppState>,
) -> Result<Track, String> {
    println!("create_track called: name={}, type={}", name, track_type);
    
    // Parse track type
    let parsed_type = match track_type.to_lowercase().as_str() {
        "main" => TrackType::Main,
        "overlay" => TrackType::Overlay,
        _ => return Err(format!("Invalid track type: {}", track_type)),
    };
    
    // Create track
    let mut track = Track::new(name, parsed_type);
    
    // Store in project state
    let mut project_lock = state.project.lock().unwrap();
    if let Some(ref mut project) = *project_lock {
        track.order = project.tracks.len() as u32;
        project.tracks.push(track.clone());
        project.mark_modified();
        println!("Added track. Project now has {} tracks", project.tracks.len());
    } else {
        // Create a new project if none exists
        use crate::models::project::Project;
        let mut new_project = Project::new("Untitled Project".to_string());
        track.order = new_project.tracks.len() as u32;
        new_project.tracks.push(track.clone());
        *project_lock = Some(new_project);
        println!("Created new project with 1 track");
    }
    
    Ok(track)
}
