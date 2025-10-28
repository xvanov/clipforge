# API Contracts: Tauri Commands

**Feature**: ClipForge Desktop Video Editor  
**Date**: 2025-10-27  
**Purpose**: Define Tauri command API for frontend-backend communication

## Overview

Tauri commands provide the bridge between the Svelte frontend and Rust backend. All commands are async and return `Result<T, String>` for error handling.

**Command Categories**:

- Media Management (import, metadata extraction)
- Playback Control (video player state)
- Timeline Operations (CRUD for clips, tracks)
- Project Management (save, load, auto-save)
- Export (FFmpeg pipeline)
- Recording (screen/webcam capture)
- Effects (apply filters, transitions, captions)

---

## Media Management Commands

### import_media_files

Import video files into media library.

**Command**: `import_media_files`

**Request**:

```typescript
{
  paths: string[]  // Absolute paths to video files
}
```

**Response**:

```typescript
{
  clips: MediaClip[],  // Successfully imported clips
  errors: Array<{      // Failed imports
    path: string,
    error: string
  }>
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn import_media_files(
    paths: Vec<String>,
    state: State<'_, AppState>
) -> Result<ImportResult, String>
```

**Side Effects**:

- Generates thumbnail (async background task)
- Generates proxy video (async background task, optional)
- Inserts metadata into SQLite cache
- Emits `media_imported` event with clip IDs

**Errors**:

- "File not found: {path}"
- "Unsupported format: {codec}"
- "Failed to read metadata: {error}"

---

### get_media_metadata

Extract detailed metadata from video file.

**Command**: `get_media_metadata`

**Request**:

```typescript
{
  clip_id: string; // MediaClip ID
}
```

**Response**:

```typescript
{
  duration: number,
  resolution: string,
  width: number,
  height: number,
  fps: number,
  codec: string,
  audio_codec: string | null,
  bitrate: number,
  has_audio: boolean
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn get_media_metadata(
    clip_id: String,
    state: State<'_, AppState>
) -> Result<MediaMetadata, String>
```

---

### generate_thumbnail

Generate thumbnail image for clip.

**Command**: `generate_thumbnail`

**Request**:

```typescript
{
  clip_id: string,
  timestamp: number  // Time in seconds (default: 0)
}
```

**Response**:

```typescript
{
  thumbnail_path: string; // Absolute path to JPEG file
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn generate_thumbnail(
    clip_id: String,
    timestamp: f64,
    state: State<'_, AppState>
) -> Result<ThumbnailResult, String>
```

**FFmpeg Command**:

```bash
ffmpeg -ss {timestamp} -i {input} -vframes 1 -q:v 2 -f image2 {output}
```

---

### generate_proxy

Generate lower-resolution proxy for smooth editing.

**Command**: `generate_proxy`

**Request**:

```typescript
{
  clip_id: string,
  resolution: "1080p" | "720p"  // Proxy resolution
}
```

**Response**:

```typescript
{
  proxy_path: string; // Absolute path to proxy MP4
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn generate_proxy(
    clip_id: String,
    resolution: ProxyResolution,
    state: State<'_, AppState>
) -> Result<ProxyResult, String>
```

**FFmpeg Command**:

```bash
ffmpeg -i {input} -vf "scale=1920:1080:force_original_aspect_ratio=decrease" \
  -c:v libx264 -preset fast -crf 23 -c:a aac -b:a 128k {output}
```

---

## Playback Control Commands

### load_clip_for_playback

Load clip into video player.

**Command**: `load_clip_for_playback`

**Request**:

```typescript
{
  clip_id: string,
  use_proxy: boolean  // Use proxy instead of original
}
```

**Response**:

```typescript
{
  playback_url: string; // File URL (file://) or asset protocol
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn load_clip_for_playback(
    clip_id: String,
    use_proxy: bool,
    state: State<'_, AppState>
) -> Result<PlaybackResult, String>
```

**Notes**:

- Uses Tauri's asset protocol for secure file access
- Returns `asset://` URL that frontend can use in `<video>` element

---

## Timeline Operations Commands

### add_clip_to_timeline

Add clip to timeline track.

**Command**: `add_clip_to_timeline`

**Request**:

```typescript
{
  media_clip_id: string,
  track_id: string,
  start_time: number,    // Position on timeline (seconds)
  in_point: number,      // Trim start (seconds)
  out_point: number      // Trim end (seconds)
}
```

**Response**:

```typescript
{
  timeline_clip: TimelineClip; // Created clip with ID
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn add_clip_to_timeline(
    media_clip_id: String,
    track_id: String,
    start_time: f64,
    in_point: f64,
    out_point: f64,
    state: State<'_, AppState>
) -> Result<TimelineClip, String>
```

**Validation**:

- Check media_clip exists
- Check track exists
- Validate in_point < out_point
- Check no overlap with existing clips on track

**Errors**:

- "Media clip not found: {id}"
- "Track not found: {id}"
- "Clip overlaps with existing clip at {time}"

---

### update_timeline_clip

Update timeline clip properties.

**Command**: `update_timeline_clip`

**Request**:

```typescript
{
  clip_id: string,
  updates: {
    start_time?: number,
    in_point?: number,
    out_point?: number,
    track_id?: string     // Move to different track
  }
}
```

**Response**:

```typescript
{
  timeline_clip: TimelineClip; // Updated clip
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn update_timeline_clip(
    clip_id: String,
    updates: TimelineClipUpdates,
    state: State<'_, AppState>
) -> Result<TimelineClip, String>
```

---

### split_timeline_clip

Split clip at specified time.

**Command**: `split_timeline_clip`

**Request**:

```typescript
{
  clip_id: string,
  split_time: number  // Time on timeline (seconds)
}
```

**Response**:

```typescript
{
  clip_before: TimelineClip,  // Original clip (trimmed to split point)
  clip_after: TimelineClip    // New clip (from split point to end)
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn split_timeline_clip(
    clip_id: String,
    split_time: f64,
    state: State<'_, AppState>
) -> Result<SplitResult, String>
```

**Validation**:

- `split_time` must be between clip start and end time
- Creates two clips referencing same MediaClip with adjusted in/out points

---

### delete_timeline_clip

Remove clip from timeline.

**Command**: `delete_timeline_clip`

**Request**:

```typescript
{
  clip_id: string;
}
```

**Response**:

```typescript
{
  success: boolean;
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn delete_timeline_clip(
    clip_id: String,
    state: State<'_, AppState>
) -> Result<(), String>
```

---

### create_track

Add new track to timeline.

**Command**: `create_track`

**Request**:

```typescript
{
  name: string,
  type: "main" | "overlay"
}
```

**Response**:

```typescript
{
  track: Track; // Created track with ID
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn create_track(
    name: String,
    track_type: TrackType,
    state: State<'_, AppState>
) -> Result<Track, String>
```

---

## Project Management Commands

### create_new_project

Create empty project with default track.

**Command**: `create_new_project`

**Request**:

```typescript
{
  name: string;
}
```

**Response**:

```typescript
{
  project: Project; // New project with ID
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn create_new_project(
    name: String,
    state: State<'_, AppState>
) -> Result<Project, String>
```

---

### save_project

Save project to file.

**Command**: `save_project`

**Request**:

```typescript
{
  path: string; // Absolute path to .clipforge file
}
```

**Response**:

```typescript
{
  success: boolean,
  path: string  // Saved file path
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn save_project(
    path: String,
    state: State<'_, AppState>
) -> Result<SaveResult, String>
```

**Side Effects**:

- Serializes project to JSON
- Writes to file atomically (temp file + rename)
- Updates project.file_path and project.modified_at

---

### load_project

Load project from file.

**Command**: `load_project`

**Request**:

```typescript
{
  path: string; // Absolute path to .clipforge file
}
```

**Response**:

```typescript
{
  project: Project; // Loaded project
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn load_project(
    path: String,
    state: State<'_, AppState>
) -> Result<Project, String>
```

**Validation**:

- Check all referenced media files still exist
- Warn if missing files (allow user to relocate)

**Errors**:

- "Invalid project file format"
- "Project version {version} not supported"
- "Missing media files: {paths}"

---

### auto_save_project

Perform auto-save (background operation).

**Command**: `auto_save_project`

**Request**: (none)

**Response**:

```typescript
{
  success: boolean,
  saved_at: string  // ISO timestamp
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn auto_save_project(
    state: State<'_, AppState>
) -> Result<AutoSaveResult, String>
```

**Side Effects**:

- Serializes project to JSON
- Stores in SQLite auto_saves table
- Keeps last 10 auto-saves (delete older)

---

## Export Commands

### export_timeline

Export timeline to video file.

**Command**: `export_timeline`

**Request**:

```typescript
{
  output_path: string,
  settings: {
    resolution: "source" | "2160p" | "1440p" | "1080p" | "720p",
    codec: "h264" | "hevc" | "vp9",
    quality: "high" | "medium" | "low",
    hardware_acceleration: boolean
  }
}
```

**Response**:

```typescript
{
  job_id: string; // Export job ID for progress tracking
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn export_timeline(
    output_path: String,
    settings: ExportSettings,
    state: State<'_, AppState>
) -> Result<ExportJob, String>
```

**Side Effects**:

- Creates export job (background task)
- Generates FFmpeg concat file from timeline clips
- Emits progress events: `export_progress`, `export_complete`, `export_error`

**Progress Events**:

```typescript
// Emitted periodically during export
{
  job_id: string,
  progress: number,      // 0.0 - 1.0
  current_frame: number,
  total_frames: number,
  fps: number,
  eta_seconds: number
}
```

---

### cancel_export

Cancel ongoing export.

**Command**: `cancel_export`

**Request**:

```typescript
{
  job_id: string;
}
```

**Response**:

```typescript
{
  success: boolean;
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn cancel_export(
    job_id: String,
    state: State<'_, AppState>
) -> Result<(), String>
```

**Side Effects**:

- Terminates FFmpeg process
- Deletes partial output file
- Emits `export_cancelled` event

---

## Recording Commands

### request_recording_permissions

Request system permissions for screen/camera/mic.

**Command**: `request_recording_permissions`

**Request**:

```typescript
{
  permissions: Array<'screen' | 'camera' | 'microphone'>;
}
```

**Response**:

```typescript
{
  granted: {
    screen: boolean,
    camera: boolean,
    microphone: boolean
  }
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn request_recording_permissions(
    permissions: Vec<Permission>,
    state: State<'_, AppState>
) -> Result<PermissionResult, String>
```

**Platform-Specific**:

- macOS: Uses `CGPreflightScreenCaptureAccess`, `AVCaptureDevice.requestAccess`
- Windows: Shows OS permission dialogs automatically

---

### list_recording_sources

Get available screens, windows, and cameras.

**Command**: `list_recording_sources`

**Request**: (none)

**Response**:

```typescript
{
  screens: Array<{
    id: string,
    name: string,      // "Display 1", "Display 2"
    resolution: string // "1920x1080"
  }>,
  windows: Array<{
    id: string,
    name: string,      // Window title
    app: string        // Application name
  }>,
  cameras: Array<{
    id: string,
    name: string       // "FaceTime HD Camera"
  }>
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn list_recording_sources(
    state: State<'_, AppState>
) -> Result<RecordingSources, String>
```

---

### start_recording

Begin screen/webcam recording.

**Command**: `start_recording`

**Request**:

```typescript
{
  type: "screen" | "webcam" | "screen_webcam",
  screen_source_id?: string,    // Required if type includes screen
  camera_device_id?: string,    // Required if type includes webcam
  audio_sources: Array<"microphone" | "system">,
  settings: {
    resolution: string,  // "1920x1080"
    fps: 30 | 60
  }
}
```

**Response**:

```typescript
{
  session_id: string; // Recording session ID
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn start_recording(
    config: RecordingConfig,
    state: State<'_, AppState>
) -> Result<RecordingSession, String>
```

**Side Effects**:

- Creates RecordingSession (status = "recording")
- Starts platform-specific capture
- Emits `recording_started` event
- Emits periodic `recording_progress` events (duration update)

---

### stop_recording

End recording and create MediaClip.

**Command**: `stop_recording`

**Request**:

```typescript
{
  session_id: string;
}
```

**Response**:

```typescript
{
  media_clip: MediaClip; // Created from recording
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn stop_recording(
    session_id: String,
    state: State<'_, AppState>
) -> Result<MediaClip, String>
```

**Side Effects**:

- Stops capture
- Finalizes video file
- Creates MediaClip (adds to media_library)
- Generates thumbnail (async)
- Emits `recording_stopped` event

---

## Effects Commands

### apply_effect

Add effect to timeline clip.

**Command**: `apply_effect`

**Request**:

```typescript
{
  clip_id: string,
  effect: {
    type: "transition" | "filter" | "text_overlay" | "audio_adjustment",
    name: string,
    parameters: Record<string, any>  // Effect-specific params
  }
}
```

**Response**:

```typescript
{
  effect: Effect; // Created effect with ID
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn apply_effect(
    clip_id: String,
    effect: EffectConfig,
    state: State<'_, AppState>
) -> Result<Effect, String>
```

---

### generate_captions

Generate captions using speech-to-text.

**Command**: `generate_captions`

**Request**:

```typescript
{
  clip_id: string,
  language: string  // ISO 639-1 code (e.g., "en")
}
```

**Response**:

```typescript
{
  job_id: string; // Caption generation job ID
}
```

**Rust Signature**:

```rust
#[tauri::command]
async fn generate_captions(
    clip_id: String,
    language: String,
    state: State<'_, AppState>
) -> Result<CaptionJob, String>
```

**Side Effects**:

- Extracts audio from clip
- Sends to Whisper.cpp (background task)
- Emits `captions_generated` event when complete with Caption array

**Progress Events**:

```typescript
{
  job_id: string,
  progress: number,  // 0.0 - 1.0
  status: "extracting_audio" | "transcribing" | "complete"
}
```

---

## Event System

### Events Emitted by Backend

**Event**: `media_imported`

```typescript
{
  clip_ids: string[]
}
```

**Event**: `export_progress`

```typescript
{
  job_id: string,
  progress: number,      // 0.0 - 1.0
  current_frame: number,
  eta_seconds: number
}
```

**Event**: `export_complete`

```typescript
{
  job_id: string,
  output_path: string
}
```

**Event**: `export_error`

```typescript
{
  job_id: string,
  error: string
}
```

**Event**: `recording_started`

```typescript
{
  session_id: string;
}
```

**Event**: `recording_progress`

```typescript
{
  session_id: string,
  duration: number  // Seconds
}
```

**Event**: `recording_stopped`

```typescript
{
  session_id: string,
  media_clip_id: string
}
```

**Event**: `captions_generated`

```typescript
{
  clip_id: string,
  captions: Caption[]
}
```

---

## Frontend Usage Examples

### Importing Media

```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

async function importVideos(paths: string[]) {
  const result = await invoke<ImportResult>('import_media_files', { paths });

  // Add successfully imported clips to UI
  result.clips.forEach((clip) => {
    mediaLibrary.add(clip);
  });

  // Show errors for failed imports
  result.errors.forEach(({ path, error }) => {
    console.error(`Failed to import ${path}: ${error}`);
  });

  // Listen for thumbnail generation
  await listen('media_imported', (event) => {
    const { clip_ids } = event.payload;
    // Refresh UI with thumbnails
  });
}
```

---

### Exporting Timeline

```typescript
async function exportVideo(outputPath: string, settings: ExportSettings) {
  const { job_id } = await invoke<ExportJob>('export_timeline', {
    output_path: outputPath,
    settings,
  });

  // Listen for progress updates
  await listen<ExportProgress>('export_progress', (event) => {
    if (event.payload.job_id === job_id) {
      updateProgressBar(event.payload.progress);
    }
  });

  // Handle completion
  await listen<ExportComplete>('export_complete', (event) => {
    if (event.payload.job_id === job_id) {
      showSuccessMessage(`Video exported to ${event.payload.output_path}`);
    }
  });
}
```

---

## Error Handling

All commands return `Result<T, String>`. Frontend should handle errors gracefully:

```typescript
try {
  const clip = await invoke<MediaClip>('import_media_files', { paths });
} catch (error) {
  // error is a string with error message
  showErrorDialog(`Import failed: ${error}`);
}
```

**Common Error Patterns**:

- "File not found: {path}" → Show file picker again
- "Unsupported format: {codec}" → Show codec help message
- "Permission denied: screen recording" → Guide user to System Preferences
- "FFmpeg error: {message}" → Show detailed error in dialog

---

## Summary

**Total Commands**: 28

**Command Categories**:

- Media Management: 4 commands
- Playback Control: 1 command
- Timeline Operations: 6 commands
- Project Management: 4 commands
- Export: 2 commands
- Recording: 4 commands
- Effects: 2 commands

**Event System**: 8 event types for async notifications

**Design Principles**:

- Async by default (all commands return Promises)
- Strong typing (TypeScript interfaces + Rust structs)
- Error handling via Result type
- Progress tracking via events (long-running operations)
- Validation at command boundaries

**Next Steps**: Generate quickstart guide for setting up development environment
