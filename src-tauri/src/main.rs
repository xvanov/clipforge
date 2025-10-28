// Clipforge Desktop Video Editor
// Main entry point

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod ffmpeg;
mod models;
mod storage;

#[cfg(target_os = "macos")]
mod platform;

use commands::{media, project, timeline};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Media commands
            media::import_media_files,
            media::get_media_metadata,
            media::generate_thumbnail,
            
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

