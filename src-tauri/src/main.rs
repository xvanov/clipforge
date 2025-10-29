// Clipforge Desktop Video Editor
// Main entry point

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod ffmpeg;
mod models;
mod platform;
mod storage;

use commands::media::AppState;
use commands::{export, media, playback, project, recording, timeline};
use std::sync::{Arc, Mutex};
use storage::CacheDb;

fn main() {
    // Initialize cache database
    let cache_path = dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".clipforge")
        .join("cache")
        .join("clipforge.db");

    std::fs::create_dir_all(cache_path.parent().unwrap())
        .expect("Failed to create cache directory");

    let cache_db = CacheDb::new(&cache_path).expect("Failed to initialize cache database");

    // Initialize app state with empty project
    let app_state = AppState {
        cache_db: Arc::new(Mutex::new(cache_db)),
        media_library: Arc::new(Mutex::new(Vec::new())),
        project: Arc::new(Mutex::new(None)),
    };

    // Initialize export state
    let export_state = export::ExportState::new();

    tauri::Builder::default()
        .manage(app_state)
        .manage(export_state)
        .invoke_handler(tauri::generate_handler![
            // Media commands
            media::import_media_files,
            media::get_media_metadata,
            media::generate_thumbnail_for_clip,
            // Playback commands
            playback::load_clip_for_playback,
            // Project commands
            project::create_new_project,
            project::save_project,
            project::load_project,
            // Timeline commands
            timeline::add_clip_to_timeline,
            timeline::update_timeline_clip,
            timeline::split_timeline_clip,
            timeline::delete_timeline_clip,
            timeline::create_track,
            // Export commands
            export::export_timeline,
            export::cancel_export,
            // Recording commands
            recording::request_recording_permissions,
            recording::list_recording_sources,
            recording::start_recording,
            recording::stop_recording,
            recording::get_recording_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
