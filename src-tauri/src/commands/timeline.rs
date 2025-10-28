use crate::commands::media::AppState;
use crate::models::timeline::{TimelineClip, Track, TrackType};
use tauri::State;

// TODO: This struct is used by update_timeline_clip which is not yet fully implemented
#[allow(dead_code)]
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
    println!(
        "add_clip_to_timeline called: media_clip={}, track={}, start={}",
        media_clip_id, track_id, start_time
    );

    // Validate inputs
    if in_point >= out_point {
        return Err("in_point must be less than out_point".to_string());
    }
    if start_time < 0.0 {
        return Err("start_time must be non-negative".to_string());
    }

    // Check if media clip exists
    let media_library = state
        .media_library
        .lock()
        .expect("Failed to acquire lock on media library");
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
    let mut project_lock = state
        .project
        .lock()
        .expect("Failed to acquire lock on project");
    if let Some(ref mut project) = *project_lock {
        // Find the track and add the clip
        let track_found = project
            .tracks
            .iter_mut()
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
    updates: TimelineClipUpdates,
    state: State<'_, AppState>,
) -> Result<TimelineClip, String> {
    println!("update_timeline_clip called: clip={}", clip_id);

    let mut project_lock = state
        .project
        .lock()
        .expect("Failed to acquire lock on project");

    if let Some(ref mut project) = *project_lock {
        // Find the clip across all tracks
        let mut updated_clip: Option<TimelineClip> = None;

        for track in &mut project.tracks {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                // Apply updates
                if let Some(start_time) = updates.start_time {
                    if start_time >= 0.0 {
                        clip.start_time = start_time;
                        println!("✓ Updated clip start_time to {}", start_time);
                    } else {
                        println!("✗ Rejected start_time update: {} (negative)", start_time);
                    }
                }
                if let Some(in_point) = updates.in_point {
                    if in_point >= 0.0 && in_point < clip.out_point {
                        clip.in_point = in_point;
                        println!("✓ Updated clip in_point to {}", in_point);
                    } else {
                        println!(
                            "✗ Rejected in_point update: {} (must be >= 0 and < out_point {})",
                            in_point, clip.out_point
                        );
                    }
                }
                if let Some(out_point) = updates.out_point {
                    if out_point > clip.in_point {
                        clip.out_point = out_point;
                        println!("✓ Updated clip out_point to {}", out_point);
                    } else {
                        println!(
                            "✗ Rejected out_point update: {} (must be > in_point {})",
                            out_point, clip.in_point
                        );
                    }
                }
                if let Some(track_id) = updates.track_id {
                    clip.track_id = track_id;
                    println!("✓ Updated clip track_id");
                }

                updated_clip = Some(clip.clone());
                break;
            }
        }

        if let Some(clip) = updated_clip {
            project.mark_modified();
            return Ok(clip);
        }

        Err(format!("Clip not found: {}", clip_id))
    } else {
        Err("No project loaded".to_string())
    }
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
    Err(format!(
        "Not fully implemented yet: {} at {}",
        clip_id, split_time
    ))
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
    let mut project_lock = state
        .project
        .lock()
        .expect("Failed to acquire lock on project");
    if let Some(ref mut project) = *project_lock {
        track.order = project.tracks.len() as u32;
        project.tracks.push(track.clone());
        project.mark_modified();
        println!(
            "Added track. Project now has {} tracks",
            project.tracks.len()
        );
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
