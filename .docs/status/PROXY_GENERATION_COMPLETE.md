# Proxy Generation Feature Complete

## Overview

Implemented automatic proxy video generation to support MOV and other non-web-compatible video formats for browser-based playback.

## Problem Solved

- MOV files and other formats with codecs not natively supported by HTML5 video (ProRes, HEVC, etc.) were failing with "NotSupportedError: The operation is not supported"
- Browsers only support limited codecs: H.264, VP8, VP9, AV1

## Solution Implemented

### Backend (Rust)

1. **New Module**: `src-tauri/src/ffmpeg/proxy.rs`
   - `needs_proxy(codec: &str)` - Detects if a codec requires proxy generation
   - `generate_proxy(source_path, output_path)` - Converts videos to H.264/MP4 format
   - Uses FFmpeg with optimized settings:
     - H.264 video codec (libx264)
     - Fast encoding preset
     - CRF 23 for quality/size balance
     - Max 1080p resolution
     - AAC audio codec
     - Progressive download support

2. **Updated**: `src-tauri/src/commands/media.rs`
   - Import process now detects non-web-compatible formats
   - Spawns background task for proxy generation (non-blocking)
   - Updates MediaClip with proxy_path once generation completes
   - Persists proxy path to cache database

3. **Updated**: `src-tauri/src/commands/playback.rs`
   - Prefers proxy path when available for playback
   - Falls back to source path if no proxy exists

### Frontend (Svelte)

4. **Updated**: `src/lib/components/VideoPreview.svelte`
   - Detects when proxy is needed but not yet available
   - Shows loading overlay with spinner during proxy generation
   - Polls backend every second to check for proxy completion
   - Auto-switches to proxy video once ready
   - Timeout after 60 seconds if generation takes too long

### Dependencies

5. **Updated**: `src-tauri/Cargo.toml`
   - Added `tokio-test = "0.4"` for async testing

## User Experience

### For Web-Compatible Formats (MP4 with H.264)

- Import completes instantly
- Playback works immediately
- No proxy generation needed

### For Non-Compatible Formats (MOV, ProRes, HEVC, etc.)

- Import completes quickly (no blocking)
- Clip appears in media library with thumbnail
- On playback attempt:
  - Loading overlay shown: "Generating web-compatible proxy..."
  - Background: FFmpeg converts to H.264/MP4
  - Once ready: Video auto-loads and plays
- Subsequent playbacks: Use cached proxy (instant)

## Files Modified

```
src-tauri/
  ├── Cargo.toml (added tokio-test)
  └── src/
      ├── ffmpeg/
      │   ├── mod.rs (export proxy module)
      │   └── proxy.rs (NEW - proxy generation logic)
      └── commands/
          ├── media.rs (background proxy generation)
          └── playback.rs (prefer proxy for playback)

src/lib/components/
  └── VideoPreview.svelte (loading UI + polling logic)
```

## Testing

Run the app and test:

1. Import an MP4 file → Should play immediately
2. Import a MOV file → Should show loading, then play once proxy is ready
3. Re-select the MOV clip → Should play immediately (proxy cached)

## Technical Details

**Supported Web Codecs** (no proxy needed):

- H.264 (h264)
- VP8 (vp8)
- VP9 (vp9)
- AV1 (av1)

**Requires Proxy**:

- HEVC (h265)
- ProRes
- MPEG-4
- Motion JPEG
- DNxHD
- All other codecs

**Proxy Storage**:

- Location: `~/.clipforge/cache/proxies/`
- Format: `{clip_id}.mp4`
- Persistent across app restarts

## Benefits

✅ Universal format support - any FFmpeg-supported format works  
✅ Non-blocking imports - UI remains responsive  
✅ Transparent to user - automatic background processing  
✅ Cached proxies - one-time generation cost  
✅ Fallback handling - graceful timeout if generation fails  
✅ Progress feedback - loading indicator shows what's happening

## Future Enhancements (Optional)

- Progress bar showing proxy generation percentage
- Event-based updates instead of polling
- Parallel proxy generation for multiple clips
- User preference to generate proxies for all imports
- Proxy quality settings (resolution, bitrate)
