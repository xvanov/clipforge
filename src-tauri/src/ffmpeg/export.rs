use crate::models::clip::MediaClip;
use crate::models::export::ExportSettings;
use crate::models::timeline::Track;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Export job tracking
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExportJob {
    pub id: String,
    pub output_path: String,
    pub status: ExportStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportStatus {
    Preparing,
    Rendering,
    Complete,
    Cancelled,
    Failed,
}

/// Export progress information
#[derive(Debug, Clone)]
pub struct ExportProgress {
    pub current_frame: u64,
    pub total_frames: u64,
    pub fps: f64,
    pub progress: f64, // 0.0 - 1.0
    pub eta_seconds: u64,
}

/// Generate FFmpeg concat file from timeline clips
pub fn generate_concat_file(
    tracks: &[Track],
    media_library: &[MediaClip],
    output_dir: &Path,
) -> Result<PathBuf, String> {
    // Debug: Log all tracks
    eprintln!("[Export] Analyzing {} tracks:", tracks.len());
    for (i, track) in tracks.iter().enumerate() {
        eprintln!(
            "[Export]   Track {}: name='{}', type={:?}, clips={}",
            i,
            track.name,
            track.track_type,
            track.clips.len()
        );
    }

    // For now, only process the main track
    // Multi-track support will be added in Phase 7 (User Story 5)
    // If multiple main tracks exist, use the one with the most clips
    let main_track = tracks
        .iter()
        .filter(|t| matches!(t.track_type, crate::models::timeline::TrackType::Main))
        .max_by_key(|t| t.clips.len())
        .ok_or_else(|| "No main track found".to_string())?;

    eprintln!(
        "[Export] Using main track '{}' with {} clips",
        main_track.name,
        main_track.clips.len()
    );

    // Sort clips by start time
    let mut clips = main_track.clips.clone();
    clips.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

    eprintln!("[Export] Found {} clips to export", clips.len());

    // Generate concat file content
    let mut content = String::from("ffconcat version 1.0\n");

    for (i, clip) in clips.iter().enumerate() {
        eprintln!(
            "[Export] Processing clip {}: media_clip_id={}",
            i, clip.media_clip_id
        );

        // Find media clip
        let media_clip = media_library
            .iter()
            .find(|m| m.id == clip.media_clip_id)
            .ok_or_else(|| format!("Media clip not found: {}", clip.media_clip_id))?;

        eprintln!("[Export]   Found media clip: {}", media_clip.source_path);

        // Use proxy if available, otherwise source
        let file_path = media_clip
            .proxy_path
            .as_ref()
            .unwrap_or(&media_clip.source_path);

        // Escape single quotes in path by replacing ' with '\''
        let escaped_path = file_path.replace("'", "'\\''");

        // Write file entry with proper escaping
        content.push_str(&format!("file '{}'\n", escaped_path));

        // Add in-point and out-point for trimming
        content.push_str(&format!("inpoint {:.6}\n", clip.in_point));
        content.push_str(&format!("outpoint {:.6}\n", clip.out_point));

        eprintln!(
            "[Export]   Added: inpoint={:.6}, outpoint={:.6}",
            clip.in_point, clip.out_point
        );
    }

    // Write concat file
    let concat_path = output_dir.join("concat.txt");

    // Log the concat file content for debugging
    eprintln!("[Export] Generated concat file:");
    eprintln!("{}", content);
    eprintln!("[Export] Concat file path: {}", concat_path.display());

    fs::write(&concat_path, content).map_err(|e| format!("Failed to write concat file: {}", e))?;

    Ok(concat_path)
}

/// Build FFmpeg command for export
pub fn build_export_command(
    concat_file: &Path,
    output_path: &Path,
    settings: &ExportSettings,
) -> Result<Command, String> {
    let mut cmd = Command::new("ffmpeg");

    // Input from concat file
    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(concat_file);

    // Video codec - choose hardware or software based on settings
    if settings.hardware_acceleration {
        match settings.codec {
            crate::models::export::VideoCodec::H264 => {
                #[cfg(target_os = "macos")]
                {
                    cmd.args(["-c:v", "h264_videotoolbox"]);
                }

                #[cfg(target_os = "windows")]
                {
                    cmd.args(["-c:v", "h264_nvenc"]);
                }

                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    // Fallback to software on other platforms
                    cmd.arg("-c:v").arg(settings.codec.ffmpeg_codec());
                }
            }
            _ => {
                // Other codecs use software encoding
                cmd.arg("-c:v").arg(settings.codec.ffmpeg_codec());
            }
        }
    } else {
        // Software encoding
        cmd.arg("-c:v").arg(settings.codec.ffmpeg_codec());
    }

    // Quality (CRF) - only for software encoders
    if !settings.hardware_acceleration || settings.codec != crate::models::export::VideoCodec::H264
    {
        cmd.arg("-crf")
            .arg(settings.quality.crf_value().to_string());
    } else {
        // For hardware encoders, use bitrate instead
        cmd.arg("-b:v").arg("5M"); // 5 Mbps default
    }

    // Preset for encoding speed/quality balance (software only)
    if !settings.hardware_acceleration {
        cmd.arg("-preset").arg("medium");
    }

    // Resolution scaling (if not source)
    if let Some((width, height)) = settings.resolution.dimensions() {
        cmd.arg("-vf").arg(format!(
            "scale={}:{}:force_original_aspect_ratio=decrease",
            width, height
        ));
    }

    // Frame rate override
    if let Some(fps) = settings.fps {
        cmd.arg("-r").arg(fps.to_string());
    }

    // Audio codec
    cmd.arg("-c:a").arg(settings.audio_codec.ffmpeg_codec());
    cmd.arg("-b:a").arg(format!("{}k", settings.audio_bitrate));

    // Output file
    cmd.arg("-y") // Overwrite output file
        .arg(output_path);

    // Configure for progress parsing
    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::piped());

    Ok(cmd)
}

/// Parse FFmpeg progress from stderr
pub fn parse_progress(line: &str, total_duration: f64) -> Option<ExportProgress> {
    // FFmpeg outputs progress like: frame= 1234 fps= 30 q=28.0 size= 1024kB time=00:00:41.40 bitrate= 202.3kbits/s speed=1.2x

    lazy_static::lazy_static! {
        static ref FRAME_RE: Regex = Regex::new(r"frame=\s*(\d+)").unwrap();
        static ref FPS_RE: Regex = Regex::new(r"fps=\s*([\d.]+)").unwrap();
        static ref TIME_RE: Regex = Regex::new(r"time=(\d+):(\d+):([\d.]+)").unwrap();
    }

    let current_frame = FRAME_RE
        .captures(line)
        .and_then(|cap| cap[1].parse::<u64>().ok())?;

    let fps = FPS_RE
        .captures(line)
        .and_then(|cap| cap[1].parse::<f64>().ok())
        .unwrap_or(30.0);

    // Parse current time
    let current_time = if let Some(cap) = TIME_RE.captures(line) {
        let hours = cap[1].parse::<f64>().unwrap_or(0.0);
        let minutes = cap[2].parse::<f64>().unwrap_or(0.0);
        let seconds = cap[3].parse::<f64>().unwrap_or(0.0);
        hours * 3600.0 + minutes * 60.0 + seconds
    } else {
        0.0
    };

    // Calculate progress
    let progress = if total_duration > 0.0 {
        (current_time / total_duration).min(1.0)
    } else {
        0.0
    };

    // Estimate total frames
    let total_frames = (total_duration * fps) as u64;

    // Calculate ETA
    let eta_seconds = if fps > 0.0 && current_frame > 0 {
        let remaining_frames = total_frames.saturating_sub(current_frame);
        (remaining_frames as f64 / fps) as u64
    } else {
        0
    };

    Some(ExportProgress {
        current_frame,
        total_frames,
        fps,
        progress,
        eta_seconds,
    })
}

/// Calculate total timeline duration
pub fn calculate_timeline_duration(tracks: &[Track]) -> f64 {
    tracks
        .iter()
        .map(|t| t.duration())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::clip::MediaClip;
    use crate::models::timeline::{TimelineClip, Track, TrackType};
    use chrono::Utc;
    use tempfile::TempDir;

    // ============================================================================
    // Test Helpers - Mock Data Creation (No I/O)
    // ============================================================================

    /// Create a mock MediaClip for testing (no file I/O)
    fn mock_media_clip(id: &str, duration: f64, path: &str) -> MediaClip {
        MediaClip {
            id: id.to_string(),
            name: format!("test_{}.mp4", id),
            source_path: path.to_string(),
            proxy_path: None,
            thumbnail_path: None,
            duration,
            resolution: "1920x1080".to_string(),
            width: 1920,
            height: 1080,
            fps: 30.0,
            codec: "h264".to_string(),
            audio_codec: Some("aac".to_string()),
            file_size: 1024 * 1024, // 1MB
            bitrate: Some(5000),
            has_audio: true,
            imported_at: Utc::now(),
            captions: vec![],
        }
    }

    /// Create a mock MediaClip with proxy path
    fn mock_media_clip_with_proxy(id: &str, duration: f64, source: &str, proxy: &str) -> MediaClip {
        let mut clip = mock_media_clip(id, duration, source);
        clip.proxy_path = Some(proxy.to_string());
        clip
    }

    /// Create a mock Track with clips
    fn mock_track_with_clips(name: &str, clips: Vec<TimelineClip>) -> Track {
        let track_id = uuid::Uuid::new_v4().to_string();
        Track {
            id: track_id,
            name: name.to_string(),
            track_type: TrackType::Main,
            order: 0,
            clips,
            visible: true,
            locked: false,
            volume: 1.0,
        }
    }

    /// Create a mock TimelineClip
    fn mock_timeline_clip(
        media_clip_id: &str,
        track_id: &str,
        start_time: f64,
        in_point: f64,
        out_point: f64,
    ) -> TimelineClip {
        TimelineClip {
            id: uuid::Uuid::new_v4().to_string(),
            media_clip_id: media_clip_id.to_string(),
            track_id: track_id.to_string(),
            start_time,
            in_point,
            out_point,
            layer_order: 0,
            transform: None,
        }
    }

    // ============================================================================
    // Test Suite 1: Concat File Generation (FAST - No I/O)
    // ============================================================================

    #[test]
    fn test_generate_concat_single_clip_full_duration() {
        let temp_dir = TempDir::new().unwrap();
        
        let media_clip = mock_media_clip("clip1", 10.0, "/path/to/video.mp4");
        let timeline_clip = mock_timeline_clip("clip1", "track1", 0.0, 0.0, 10.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline_clip]);
        let media_library = vec![media_clip];
        
        let result = generate_concat_file(&[track], &media_library, temp_dir.path());
        
        assert!(result.is_ok());
        let concat_path = result.unwrap();
        
        // Read and verify concat file content
        let content = std::fs::read_to_string(concat_path).unwrap();
        assert!(content.contains("ffconcat version 1.0"));
        assert!(content.contains("file '/path/to/video.mp4'"));
        assert!(content.contains("inpoint 0.000000"));
        assert!(content.contains("outpoint 10.000000"));
    }

    #[test]
    fn test_generate_concat_single_clip_trimmed() {
        let temp_dir = TempDir::new().unwrap();
        
        let media_clip = mock_media_clip("clip1", 10.0, "/path/to/video.mp4");
        let timeline_clip = mock_timeline_clip("clip1", "track1", 0.0, 2.0, 5.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline_clip]);
        let media_library = vec![media_clip];
        
        let result = generate_concat_file(&[track], &media_library, temp_dir.path());
        
        assert!(result.is_ok());
        let concat_path = result.unwrap();
        
        let content = std::fs::read_to_string(concat_path).unwrap();
        assert!(content.contains("inpoint 2.000000"));
        assert!(content.contains("outpoint 5.000000"));
    }

    #[test]
    fn test_generate_concat_multiple_clips_ordered() {
        let temp_dir = TempDir::new().unwrap();
        
        let media1 = mock_media_clip("clip1", 5.0, "/path/to/video1.mp4");
        let media2 = mock_media_clip("clip2", 7.0, "/path/to/video2.mp4");
        let media3 = mock_media_clip("clip3", 3.0, "/path/to/video3.mp4");
        
        // Add clips in non-chronological order (should be sorted by start_time)
        let timeline1 = mock_timeline_clip("clip1", "track1", 5.0, 0.0, 5.0);
        let timeline2 = mock_timeline_clip("clip2", "track1", 0.0, 0.0, 7.0);
        let timeline3 = mock_timeline_clip("clip3", "track1", 10.0, 0.0, 3.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline1, timeline2, timeline3]);
        let media_library = vec![media1, media2, media3];
        
        let result = generate_concat_file(&[track], &media_library, temp_dir.path());
        
        assert!(result.is_ok());
        let concat_path = result.unwrap();
        
        let content = std::fs::read_to_string(concat_path).unwrap();
        
        // Verify clips appear in chronological order (clip2, clip1, clip3)
        let clip2_pos = content.find("video2.mp4").unwrap();
        let clip1_pos = content.find("video1.mp4").unwrap();
        let clip3_pos = content.find("video3.mp4").unwrap();
        
        assert!(clip2_pos < clip1_pos);
        assert!(clip1_pos < clip3_pos);
    }

    #[test]
    fn test_generate_concat_escapes_paths_with_quotes() {
        let temp_dir = TempDir::new().unwrap();
        
        let media_clip = mock_media_clip("clip1", 5.0, "/path/to/my'video.mp4");
        let timeline_clip = mock_timeline_clip("clip1", "track1", 0.0, 0.0, 5.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline_clip]);
        let media_library = vec![media_clip];
        
        let result = generate_concat_file(&[track], &media_library, temp_dir.path());
        
        assert!(result.is_ok());
        let concat_path = result.unwrap();
        
        let content = std::fs::read_to_string(concat_path).unwrap();
        
        // Verify path is escaped: ' becomes '\''
        assert!(content.contains("my'\\''video.mp4"));
    }

    #[test]
    fn test_generate_concat_uses_proxy_when_available() {
        let temp_dir = TempDir::new().unwrap();
        
        let media_clip = mock_media_clip_with_proxy(
            "clip1",
            5.0,
            "/path/to/source.mov",
            "/path/to/proxy.mp4"
        );
        let timeline_clip = mock_timeline_clip("clip1", "track1", 0.0, 0.0, 5.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline_clip]);
        let media_library = vec![media_clip];
        
        let result = generate_concat_file(&[track], &media_library, temp_dir.path());
        
        assert!(result.is_ok());
        let concat_path = result.unwrap();
        
        let content = std::fs::read_to_string(concat_path).unwrap();
        
        // Should use proxy path, not source
        assert!(content.contains("proxy.mp4"));
        assert!(!content.contains("source.mov"));
    }

    #[test]
    fn test_generate_concat_fails_on_missing_media_clip() {
        let temp_dir = TempDir::new().unwrap();
        
        // Timeline references a clip that doesn't exist in media library
        let timeline_clip = mock_timeline_clip("nonexistent", "track1", 0.0, 0.0, 5.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline_clip]);
        let media_library = vec![]; // Empty - clip not found
        
        let result = generate_concat_file(&[track], &media_library, temp_dir.path());
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Media clip not found"));
    }

    #[test]
    fn test_generate_concat_fails_on_no_main_track() {
        let temp_dir = TempDir::new().unwrap();
        
        let media_clip = mock_media_clip("clip1", 5.0, "/path/to/video.mp4");
        let media_library = vec![media_clip];
        
        // Create overlay track instead of main
        let mut track = mock_track_with_clips("Overlay", vec![]);
        track.track_type = TrackType::Overlay;
        
        let result = generate_concat_file(&[track], &media_library, temp_dir.path());
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No main track found"));
    }

    // ============================================================================
    // Test Suite 2: Command Building (FAST - No execution)
    // ============================================================================

    #[test]
    fn test_build_command_hardware_accel_macos() {
        let temp_dir = TempDir::new().unwrap();
        let concat_path = temp_dir.path().join("concat.txt");
        let output_path = temp_dir.path().join("output.mp4");
        
        let settings = ExportSettings {
            hardware_acceleration: true,
            codec: crate::models::export::VideoCodec::H264,
            ..Default::default()
        };
        
        let result = build_export_command(&concat_path, &output_path, &settings);
        
        assert!(result.is_ok());
        let cmd = result.unwrap();
        let cmd_str = format!("{:?}", cmd);
        
        #[cfg(target_os = "macos")]
        {
            assert!(cmd_str.contains("h264_videotoolbox"));
            // Hardware encoder should use bitrate, not CRF
            assert!(cmd_str.contains("-b:v"));
        }
    }

    #[test]
    fn test_build_command_software_encoding() {
        let temp_dir = TempDir::new().unwrap();
        let concat_path = temp_dir.path().join("concat.txt");
        let output_path = temp_dir.path().join("output.mp4");
        
        let settings = ExportSettings {
            hardware_acceleration: false,
            codec: crate::models::export::VideoCodec::H264,
            ..Default::default()
        };
        
        let result = build_export_command(&concat_path, &output_path, &settings);
        
        assert!(result.is_ok());
        let cmd = result.unwrap();
        let cmd_str = format!("{:?}", cmd);
        
        // Software encoding should use libx264 and CRF
        assert!(cmd_str.contains("libx264"));
        assert!(cmd_str.contains("-crf"));
        assert!(cmd_str.contains("-preset"));
    }

    #[test]
    fn test_build_command_resolution_scaling() {
        let temp_dir = TempDir::new().unwrap();
        let concat_path = temp_dir.path().join("concat.txt");
        let output_path = temp_dir.path().join("output.mp4");
        
        let settings = ExportSettings {
            resolution: crate::models::export::ExportResolution::FullHD,
            ..Default::default()
        };
        
        let result = build_export_command(&concat_path, &output_path, &settings);
        
        assert!(result.is_ok());
        let cmd = result.unwrap();
        let cmd_str = format!("{:?}", cmd);
        
        // Should have scale filter
        assert!(cmd_str.contains("-vf"));
        assert!(cmd_str.contains("scale=1920:1080"));
    }

    #[test]
    fn test_build_command_includes_audio_settings() {
        let temp_dir = TempDir::new().unwrap();
        let concat_path = temp_dir.path().join("concat.txt");
        let output_path = temp_dir.path().join("output.mp4");
        
        let settings = ExportSettings::default();
        
        let result = build_export_command(&concat_path, &output_path, &settings);
        
        assert!(result.is_ok());
        let cmd = result.unwrap();
        let cmd_str = format!("{:?}", cmd);
        
        // Should have audio codec and bitrate
        assert!(cmd_str.contains("-c:a"));
        assert!(cmd_str.contains("-b:a"));
    }

    // ============================================================================
    // Test Suite 3: Duration Calculation (FAST - Pure math)
    // ============================================================================

    #[test]
    fn test_calculate_duration_single_track() {
        let timeline1 = mock_timeline_clip("clip1", "track1", 0.0, 0.0, 5.0);
        let timeline2 = mock_timeline_clip("clip2", "track1", 5.0, 0.0, 7.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline1, timeline2]);
        
        let duration = calculate_timeline_duration(&[track]);
        
        // Clip1: 0-5s, Clip2: 5-12s, total = 12s
        assert_eq!(duration, 12.0);
    }

    #[test]
    fn test_calculate_duration_multiple_tracks() {
        let timeline1 = mock_timeline_clip("clip1", "track1", 0.0, 0.0, 10.0);
        let track1 = mock_track_with_clips("Track 1", vec![timeline1]);
        
        let timeline2 = mock_timeline_clip("clip2", "track2", 0.0, 0.0, 15.0);
        let track2 = mock_track_with_clips("Track 2", vec![timeline2]);
        
        let duration = calculate_timeline_duration(&[track1, track2]);
        
        // Should use longest track duration
        assert_eq!(duration, 15.0);
    }

    #[test]
    fn test_calculate_duration_with_trimming() {
        // Clip at start_time=0, but inpoint=2, outpoint=8
        // Actual duration should be 6s (not 8s)
        let timeline = mock_timeline_clip("clip1", "track1", 0.0, 2.0, 8.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline]);
        
        let duration = calculate_timeline_duration(&[track]);
        
        // Duration = start_time + (outpoint - inpoint) = 0 + (8 - 2) = 6
        assert_eq!(duration, 6.0);
    }

    #[test]
    fn test_calculate_duration_empty_tracks() {
        let track = mock_track_with_clips("Main Track", vec![]);
        
        let duration = calculate_timeline_duration(&[track]);
        
        assert_eq!(duration, 0.0);
    }

    #[test]
    fn test_calculate_duration_with_gaps() {
        let timeline1 = mock_timeline_clip("clip1", "track1", 0.0, 0.0, 3.0);
        let timeline2 = mock_timeline_clip("clip2", "track1", 10.0, 0.0, 5.0);
        
        let track = mock_track_with_clips("Main Track", vec![timeline1, timeline2]);
        
        let duration = calculate_timeline_duration(&[track]);
        
        // Clip1: 0-3s, Clip2: 10-15s, total timeline duration = 15s
        assert_eq!(duration, 15.0);
    }

    // ============================================================================
    // Test Suite 4: Progress Parsing (FAST - String parsing)
    // ============================================================================

    #[test]
    fn test_parse_progress() {
        let line = "frame= 1234 fps= 30 q=28.0 size= 1024kB time=00:00:41.40 bitrate= 202.3kbits/s speed=1.2x";
        let total_duration = 120.0; // 2 minutes

        let progress = parse_progress(line, total_duration);
        assert!(progress.is_some());

        let progress = progress.unwrap();
        assert_eq!(progress.current_frame, 1234);
        assert_eq!(progress.fps, 30.0);
        assert!(progress.progress > 0.0 && progress.progress < 1.0);
    }

    #[test]
    fn test_parse_progress_returns_none_on_invalid() {
        let line = "Some random FFmpeg output without progress";
        let total_duration = 120.0;

        let progress = parse_progress(line, total_duration);
        assert!(progress.is_none());
    }

    #[test]
    fn test_parse_progress_calculates_eta() {
        let line = "frame= 100 fps= 25 q=28.0 size= 1024kB time=00:00:04.00 bitrate= 202.3kbits/s speed=1.0x";
        let total_duration = 100.0; // 100 seconds total

        let progress = parse_progress(line, total_duration);
        assert!(progress.is_some());

        let progress = progress.unwrap();
        assert!(progress.eta_seconds > 0);
    }

    // ============================================================================
    // Test Suite 5: Export Settings (FAST)
    // ============================================================================

    #[test]
    fn test_export_settings_defaults() {
        let settings = ExportSettings::default();
        assert_eq!(settings.audio_bitrate, 192);
        assert!(settings.hardware_acceleration);
    }

    // ============================================================================
    // Test Suite 6: Real E2E Test (SLOW - marked with #[ignore])
    // ============================================================================

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_full_import_edit_export_workflow() {
        // This test would require actual video files and FFmpeg execution
        // Only run manually or in CI
        
        // TODO: Implement when we have tiny test fixtures
        // 1. Import tiny test video (100ms)
        // 2. Add to timeline with trim
        // 3. Export
        // 4. Verify output file exists and is valid
        
        println!("E2E test requires real video fixtures - implement later");
    }
}
