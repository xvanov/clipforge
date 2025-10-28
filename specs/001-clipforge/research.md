# Research & Technical Decisions: ClipForge

**Feature**: ClipForge Desktop Video Editor  
**Date**: 2025-10-27  
**Purpose**: Document technology research, architectural decisions, and rationale for implementation choices

## Core Technology Stack

### Desktop Framework: Tauri

**Decision**: Use Tauri 1.5+ as desktop application framework

**Rationale**:

- **Small bundle size**: Tauri apps are 10-20MB vs Electron's 100-200MB (uses system webview instead of bundling Chromium)
- **Performance**: Rust backend provides native performance for FFmpeg integration and file I/O
- **Security**: Strong isolation between frontend and backend, explicit command API
- **Cross-platform**: Single codebase builds for macOS, Windows, Linux
- **Active development**: Strong community, regular updates, production-ready

**Alternatives Considered**:

- **Electron**: Rejected due to large bundle size, higher memory usage, slower startup
- **Flutter Desktop**: Rejected due to limited video manipulation ecosystem, Canvas API limitations for timeline rendering
- **Native (Swift/C++/C#)**: Rejected due to need for separate codebases per platform, longer development time

**Implementation Notes**:

- Use `tauri-plugin-dialog` for native file pickers
- Use `tauri-plugin-fs-extra` for enhanced file operations
- Platform-specific screen recording requires conditional compilation (`cfg(target_os)`)

---

### Frontend: Svelte + TypeScript

**Decision**: Use Svelte 4.0+ with TypeScript for UI layer

**Rationale**:

- **Reactive by design**: Svelte's reactivity model ideal for timeline state synchronization
- **Performance**: Compiles to vanilla JavaScript, no virtual DOM overhead (critical for 60fps timeline)
- **Small bundle**: Smaller than React/Vue, faster load times
- **TypeScript**: Type safety for complex timeline state management
- **Learning curve**: Simpler than React for component-based UI

**Alternatives Considered**:

- **React**: Rejected due to virtual DOM overhead (not ideal for Canvas-heavy timeline), larger bundle
- **Vue 3**: Viable alternative but Svelte's compiler approach better for performance-critical timeline
- **Vanilla JS**: Rejected due to complexity of state management for timeline editing

**Implementation Notes**:

- Use Svelte stores for global state (project, timeline, media library)
- Leverage Svelte's reactive declarations (`$:`) for derived timeline state
- Use `bind:this` for Canvas element refs in timeline component

---

### Media Processing: FFmpeg

**Decision**: Use FFmpeg 6.0+ CLI invoked from Rust backend

**Rationale**:

- **Industry standard**: Most battle-tested video processing tool
- **Comprehensive codec support**: Handles MP4, MOV, WebM, and virtually all formats
- **Hardware acceleration**: Supports VideoToolbox (macOS), NVENC (NVIDIA), QuickSync (Intel)
- **CLI stability**: Stable interface, extensive documentation
- **No licensing issues**: LGPL when using pre-built binaries

**Alternatives Considered**:

- **GStreamer**: Rejected due to steeper learning curve, more complex API, less documentation
- **libav (FFmpeg libraries)**: Rejected due to C API complexity, Rust FFI overhead, harder to debug
- **Browser APIs (WebCodecs)**: Rejected - limited codec support, no hardware acceleration, experimental status

**Implementation Strategy**:

- **Bundle static FFmpeg binary** with app (platform-specific builds)
- **Invoke via Rust `tokio::process::Command`** for async execution
- **Stream progress** via stderr parsing (FFmpeg outputs progress to stderr)
- **Generate proxies** on import (H.264 1080p mezzanines) for smooth timeline scrubbing
- **Hardware acceleration**: Use `-hwaccel videotoolbox` (macOS) for faster encoding

**Key FFmpeg Commands**:

```bash
# Thumbnail extraction (import)
ffmpeg -i input.mp4 -vf "select=eq(n\,0)" -q:v 2 -f image2 thumbnail.jpg

# Proxy generation (1080p H.264)
ffmpeg -i input.mp4 -vf "scale=1920:1080:force_original_aspect_ratio=decrease" \
  -c:v libx264 -preset fast -crf 23 -c:a aac -b:a 128k proxy.mp4

# Export (multi-clip timeline)
ffmpeg -f concat -safe 0 -i inputs.txt -c:v libx264 -crf 18 -c:a aac -b:a 192k output.mp4

# Hardware-accelerated export (macOS)
ffmpeg -hwaccel videotoolbox -i input.mp4 -c:v h264_videotoolbox -b:v 8M output.mp4
```

**Performance Considerations**:

- **Proxy videos**: Generate on import to avoid decoding original 4K files during scrubbing
- **Thumbnail caching**: Extract first frame on import, cache to SQLite + disk
- **Batch operations**: Queue FFmpeg tasks, run sequentially to avoid resource contention

---

### Timeline UI: HTML5 Canvas

**Decision**: Use HTML5 Canvas for custom timeline renderer

**Rationale**:

- **Performance**: Direct pixel manipulation, 60fps possible with optimized rendering
- **Flexibility**: Full control over timeline visuals (clip blocks, waveforms, markers)
- **Dirty-rect optimization**: Only redraw changed regions during scrubbing/dragging
- **HiDPI support**: `devicePixelRatio` scaling for sharp rendering on Retina displays

**Alternatives Considered**:

- **SVG**: Rejected due to performance degradation with 50+ clip elements
- **DOM-based (divs)**: Rejected due to layout thrashing with complex timelines
- **WebGL**: Overkill for 2D timeline, added complexity without benefit

**Implementation Approach**:

```typescript
// Timeline renderer architecture
class TimelineRenderer {
  - render(clips: Clip[], playhead: number): void
  - renderClip(clip: Clip, x: number, width: number): void
  - renderPlayhead(position: number): void
  - hitTest(mouseX: number, mouseY: number): Clip | null
  - zoom(level: number): void
}

// Optimization strategies:
- Use requestAnimationFrame for smooth 60fps rendering
- Implement dirty-rect tracking (only redraw changed regions)
- Cache clip thumbnails as ImageBitmap for fast blitting
- Use layered canvases (static background, dynamic playhead)
```

**Rendering Layers**:

1. **Background layer**: Grid lines, time markers (redrawn only on zoom)
2. **Clip layer**: Video clip blocks with thumbnails (redrawn on timeline changes)
3. **Playhead layer**: Current position indicator (redrawn every frame during playback)

---

### Video Playback: HTML5 Video Element

**Decision**: Use HTML5 `<video>` element for preview playback

**Rationale**:

- **Native performance**: Hardware-accelerated decoding via browser engine
- **Simple API**: Easy play/pause/seek controls
- **Codec support**: Safari/WebKit supports H.264, HEVC, ProRes natively on macOS
- **Synchronized audio**: Built-in audio/video sync

**Limitations & Workarounds**:

- **Seeking precision**: Time-based, not frame-accurate (acceptable for MVP)
- **Codec support**: Use H.264 proxies for consistent playback across clips
- **Multi-clip playback**: Switch video source when transitioning between clips on timeline

**Implementation**:

```typescript
// Video player service
class VideoPlayer {
  - loadClip(clipPath: string): void
  - play(): void
  - pause(): void
  - seek(timeSeconds: number): void
  - setVolume(level: number): void
  - getCurrentTime(): number
}

// Sync with timeline playhead
timeline.on('scrub', (time) => videoPlayer.seek(time));
timeline.on('play', () => videoPlayer.play());
```

**Future Enhancement**: For frame-accurate previews (P5+), consider libmpv or FFmpeg image pipe.

---

## Platform-Specific Considerations

### macOS Screen Recording (AVFoundation)

**Approach**: Use macOS native APIs via Rust FFI

**Key APIs**:

- `AVCaptureSession`: Coordinate screen/webcam recording
- `AVCaptureScreenInput`: Capture screen content
- `AVCaptureDeviceInput`: Capture webcam input
- `AVAssetWriter`: Write recordings to MP4 file

**Rust Integration**:

- Use `objc` crate for Objective-C bridge
- Wrap AVFoundation calls in safe Rust API
- Request screen recording permission via `CGPreflightScreenCaptureAccess`

**Implementation Notes**:

```rust
#[cfg(target_os = "macos")]
mod macos {
    use objc::*;

    pub fn request_screen_recording_permission() -> bool {
        // Call CGPreflightScreenCaptureAccess
    }

    pub fn start_screen_recording(output_path: &str) -> Result<(), Error> {
        // Initialize AVCaptureSession
        // Add AVCaptureScreenInput
        // Start recording
    }
}
```

---

### Windows Screen Recording (Windows.Graphics.Capture)

**Approach**: Use Windows Runtime (WinRT) APIs via Rust

**Key APIs**:

- `Windows.Graphics.Capture.GraphicsCaptureSession`: Capture screen/window
- `Windows.Media.MediaCapture`: Access webcam
- `Windows.Media.Transcoding.MediaTranscoder`: Encode recordings

**Rust Integration**:

- Use `windows` crate (official Microsoft Rust bindings)
- Request permissions via OS permission dialogs

**Implementation Notes**:

```rust
#[cfg(target_os = "windows")]
mod windows {
    use windows::Graphics::Capture::*;

    pub fn start_screen_recording(output_path: &str) -> Result<(), Error> {
        // Initialize GraphicsCaptureSession
        // Create MediaTranscoder
        // Start recording
    }
}
```

---

## Storage & Persistence

### Project Files: JSON

**Decision**: Store projects as JSON files on local filesystem

**Rationale**:

- **Human-readable**: Easy debugging, version control friendly
- **Simple serialization**: Rust `serde` + TypeScript JSON.parse()
- **Cross-platform**: Works identically on all platforms
- **No database overhead**: Lightweight for single-user desktop app

**Project Schema**:

```json
{
  "version": "1.0.0",
  "name": "My Video Project",
  "created_at": "2025-10-27T10:00:00Z",
  "modified_at": "2025-10-27T12:30:00Z",
  "timeline": {
    "tracks": [
      {
        "id": "track-1",
        "type": "main",
        "clips": [
          {
            "id": "clip-1",
            "source_path": "/path/to/video.mp4",
            "start_time": 0.0,
            "end_time": 10.5,
            "in_point": 2.0,
            "out_point": 8.0
          }
        ]
      }
    ],
    "duration": 10.5
  },
  "media_library": [
    {
      "id": "media-1",
      "path": "/path/to/video.mp4",
      "duration": 30.0,
      "resolution": "1920x1080",
      "fps": 30,
      "codec": "h264"
    }
  ]
}
```

---

### Media Cache: SQLite + Files

**Decision**: Use SQLite for metadata, filesystem for thumbnails/proxies

**Rationale**:

- **SQLite**: Fast queries for clip metadata, auto-save timestamps
- **Filesystem**: Store large binary data (thumbnails, proxy videos) as files
- **Hybrid approach**: Best of both worlds (structured data + blob storage)

**Cache Schema**:

```sql
CREATE TABLE clips (
    id TEXT PRIMARY KEY,
    source_path TEXT NOT NULL,
    duration REAL NOT NULL,
    resolution TEXT NOT NULL,
    fps INTEGER NOT NULL,
    codec TEXT NOT NULL,
    thumbnail_path TEXT,
    proxy_path TEXT,
    imported_at INTEGER NOT NULL
);

CREATE TABLE auto_saves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_path TEXT NOT NULL,
    snapshot_data TEXT NOT NULL,  -- JSON
    saved_at INTEGER NOT NULL
);
```

---

## Testing Strategy

### Unit Tests: Cargo Test (Rust)

**Scope**: Test individual Rust functions (FFmpeg wrappers, file I/O, models)

**Examples**:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_ffmpeg_progress() {
        let stderr = "frame=  100 fps= 30 q=-1.0 size=    1024kB time=00:00:03.33";
        let progress = parse_progress(stderr);
        assert_eq!(progress.frame, 100);
        assert_eq!(progress.time_seconds, 3.33);
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Test");
        let json = serde_json::to_string(&project).unwrap();
        let deserialized: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(project, deserialized);
    }
}
```

**Performance Target**: Under 5 seconds for full Rust test suite

---

### Component Tests: Vitest (Svelte)

**Scope**: Test Svelte components in isolation (Timeline, MediaLibrary, VideoPreview)

**Examples**:

```typescript
import { render, fireEvent } from '@testing-library/svelte';
import Timeline from '$lib/components/Timeline.svelte';

test('Timeline renders clips correctly', () => {
  const clips = [{ id: '1', start: 0, end: 10 }];
  const { getByText } = render(Timeline, { clips });
  expect(getByText('00:00:10')).toBeInTheDocument();
});

test('Timeline scrubbing updates playhead', async () => {
  const { component } = render(Timeline);
  await fireEvent.mouseDown(canvas, { clientX: 100 });
  expect(component.playheadPosition).toBe(5.0); // 100px = 5 seconds
});
```

**Performance Target**: Under 10 seconds for full frontend test suite

---

### Integration Tests: Rust + Sample Videos

**Scope**: Test FFmpeg pipelines end-to-end with real video files

**Examples**:

```rust
#[test]
fn test_thumbnail_extraction() {
    let input = "tests/fixtures/sample-1080p.mp4";
    let output = "/tmp/thumbnail.jpg";
    extract_thumbnail(input, output).unwrap();
    assert!(Path::new(output).exists());
    // Verify image dimensions
}

#[test]
fn test_export_multi_clip_timeline() {
    let clips = vec![
        Clip { path: "tests/fixtures/clip1.mp4", start: 0, end: 5 },
        Clip { path: "tests/fixtures/clip2.mp4", start: 5, end: 10 },
    ];
    export_timeline(clips, "/tmp/output.mp4").unwrap();
    // Verify output duration = 10 seconds
}
```

**Performance Target**: Under 30 seconds (use short test videos)

---

### E2E Tests: Playwright

**Scope**: Test complete user workflows in packaged app

**Examples**:

```typescript
test('Import and play video', async ({ page }) => {
  await page.click('button:has-text("Import")');
  await page.setInputFiles('input[type="file"]', 'tests/fixtures/sample.mp4');
  await expect(page.locator('.media-library')).toContainText('sample.mp4');
  await page.click('.media-library >> text=sample.mp4');
  await page.click('button:has-text("Play")');
  // Wait for video to start playing
  await expect(page.locator('video')).toHaveJSProperty('paused', false);
});

test('Edit timeline and export', async ({ page }) => {
  // Import clips, drag to timeline, trim, export
  // Verify exported file exists and has correct duration
});
```

**Performance Target**: Under 2 minutes for full E2E suite

---

## CI/CD Pipeline

### GitHub Actions Workflow

**Platforms**: macOS (Intel + ARM), Windows (x64)

**Pipeline Stages**:

```yaml
name: CI/CD

on: [push, pull_request]

jobs:
  test-rust:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
      - name: Install FFmpeg
        run: # Platform-specific FFmpeg install
      - name: Run tests
        run: cargo test
      - name: Lint
        run: cargo clippy -- -D warnings

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - name: Install dependencies
        run: npm ci
      - name: Run tests
        run: npm test
      - name: Lint
        run: npm run lint
      - name: Type check
        run: npm run type-check

  build:
    needs: [test-rust, test-frontend]
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - name: Build Tauri app
        run: npm run tauri build
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: clipforge-${{ matrix.os }}
          path: src-tauri/target/release/bundle/
```

**Local Validation (Before Push)**:

```bash
# Rust checks
cargo test
cargo clippy
cargo fmt --check

# Frontend checks
npm test
npm run lint
npm run type-check

# Build verification
npm run tauri build
```

---

## Performance Optimization Strategy

### Timeline Rendering

**Optimization Techniques**:

1. **Dirty-rect rendering**: Only redraw changed regions
2. **RequestAnimationFrame loop**: Smooth 60fps updates
3. **ImageBitmap caching**: Pre-load clip thumbnails as bitmaps
4. **Layered canvases**: Separate static (grid) from dynamic (playhead) layers
5. **devicePixelRatio scaling**: Sharp rendering on HiDPI displays

**Target**: 60fps timeline updates, sub-100ms click-to-feedback

---

### Video Import

**Optimization Techniques**:

1. **Background processing**: Generate thumbnails/proxies async (don't block UI)
2. **Batch operations**: Process multiple imports in parallel (tokio tasks)
3. **Progress streaming**: Show real-time progress via Tauri events
4. **Proxy generation**: Create 1080p H.264 proxies for smooth scrubbing

**Target**: Thumbnail visible within 3 seconds of import

---

### Video Export

**Optimization Techniques**:

1. **Hardware acceleration**: Use VideoToolbox (macOS) or NVENC (Windows) when available
2. **Streaming progress**: Parse FFmpeg stderr, emit progress events
3. **Multi-pass encoding**: Optional quality pass for final export
4. **Temp file cleanup**: Remove intermediate files after export

**Target**: Real-time encoding (1 minute video exports in ~1 minute)

---

## Security Considerations

### File System Access

**Approach**: Use Tauri's permission system

**Security Measures**:

- Scope filesystem access to user-selected directories
- Validate all file paths before FFmpeg operations
- Prevent directory traversal attacks (reject paths with `..`)
- Sandbox FFmpeg execution (no shell injection)

---

### FFmpeg Command Injection

**Approach**: Use Rust command builders, never shell strings

**Secure Pattern**:

```rust
// SAFE: Arguments are properly escaped
Command::new("ffmpeg")
    .arg("-i")
    .arg(input_path)  // No shell interpretation
    .arg("-c:v")
    .arg("libx264")
    .spawn()

// UNSAFE: Shell injection risk
Command::new("sh")
    .arg("-c")
    .arg(format!("ffmpeg -i {}", input_path))  // NEVER DO THIS
```

---

## Dependencies & Licensing

### Rust Crates (Cargo.toml)

```toml
[dependencies]
tauri = "1.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
rusqlite = "0.30"
tauri-plugin-dialog = "1.0"
tauri-plugin-fs-extra = "1.0"

[target.'cfg(target_os = "macos")'.dependencies]
objc = "0.2"

[target.'cfg(target_os = "windows")'.dependencies]
windows = "0.52"
```

---

### NPM Packages (package.json)

```json
{
  "dependencies": {
    "svelte": "^4.0.0",
    "@tauri-apps/api": "^1.5.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "vitest": "^1.0.0",
    "@testing-library/svelte": "^4.0.0",
    "@playwright/test": "^1.40.0",
    "eslint": "^8.0.0",
    "prettier": "^3.0.0"
  }
}
```

---

### FFmpeg Licensing

**License**: LGPL 2.1+ (when using pre-built binaries)

**Compliance**:

- Distribute FFmpeg binary separately (not statically linked)
- Include FFmpeg license file in app bundle
- Provide attribution in About dialog
- No modifications to FFmpeg source (use official builds)

**Distribution**:

- macOS: Bundle `ffmpeg` binary in `Contents/MacOS/`
- Windows: Bundle `ffmpeg.exe` in app directory
- Download from official FFmpeg builds (not GPL-licensed builds with extra codecs)

---

## Open Questions & Future Research

### For MVP (P1):

- ✅ All critical decisions resolved
- No blockers identified

### For P2 (Recording):

- **Question**: How to handle simultaneous screen + webcam recording?
  - **Answer**: Research AVFoundation's multi-input capability or composite in post-processing
- **Question**: System audio capture on macOS (requires signed app)?
  - **Answer**: Research Loopback Audio or BlackHole virtual audio devices

### For P4 (Captions):

- **Question**: Which speech-to-text engine (Whisper.cpp, Vosk, cloud APIs)?
  - **Answer**: Research Whisper.cpp for local processing (privacy + offline)
- **Question**: Language model size vs accuracy tradeoff?
  - **Answer**: Test "base" model (142MB) vs "small" model (466MB) on sample videos

### For P5 (Effects):

- **Question**: Real-time effects preview (FFmpeg filters)?
  - **Answer**: Research FFmpeg filter graphs for brightness/contrast adjustments
- **Question**: Text overlay rendering (Canvas vs FFmpeg drawtext)?
  - **Answer**: Canvas for preview, FFmpeg for export

---

## Summary

**Technology stack validated**: Tauri + Svelte + FFmpeg provides solid foundation for ClipForge. All MVP (P1) requirements achievable with chosen stack.

**Key strengths**:

- Native performance (Rust + hardware-accelerated FFmpeg)
- Small bundle size (Tauri vs Electron)
- Cross-platform with single codebase
- Proven technologies (FFmpeg industry standard)

**Risks identified**:

- Platform-specific recording APIs (P2) require careful implementation
- Canvas timeline performance must be optimized early
- FFmpeg bundling and licensing requires attention to detail

**Next steps**: Proceed to Phase 1 (data model, contracts, quickstart guide)
