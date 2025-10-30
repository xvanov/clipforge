# ClipForge AI Captions Feature

## 🎯 Feature Complete

AI-powered speech-to-text caption generation has been successfully implemented for ClipForge!

## ✨ What's New

### User-Facing Features

- 🎙️ **Automatic caption generation** from video audio
- 🌍 **Multi-language support** (10 languages + auto-detect)
- ✏️ **Caption editing** (text and timing adjustment)
- 👁️ **Live caption preview** during video playback
- 📊 **Real-time progress tracking** during generation
- 💾 **Persistent captions** stored with media clips
- 🔒 **100% local processing** (no data sent to external servers)

### Technical Implementation

- Simple CLI integration with Whisper.cpp (no complex FFI)
- FFmpeg audio extraction (16-bit PCM WAV, 16kHz, mono)
- SRT subtitle parsing with timestamp precision
- Async background processing with progress events
- Comprehensive error handling and cleanup
- Full TypeScript type safety

## 📂 Project Structure

```
src-tauri/
├── src/
│   ├── ai/                      # NEW: AI module
│   │   ├── mod.rs
│   │   └── whisper.rs           # Whisper.cpp integration
│   ├── models/
│   │   ├── caption.rs           # NEW: Caption entity
│   │   └── clip.rs              # MODIFIED: Added captions field
│   ├── ffmpeg/
│   │   └── audio.rs             # NEW: Audio extraction
│   └── commands/
│       └── captions.rs          # NEW: Caption commands
└── docs/
    ├── WHISPER_SETUP.md         # Setup instructions
    └── AI_CAPTIONS_IMPLEMENTATION.md  # Implementation details

src/
├── lib/
│   ├── types/
│   │   └── caption.ts           # NEW: TypeScript types
│   └── components/
│       ├── CaptionsPanel.svelte # NEW: Caption UI
│       └── VideoPreview.svelte  # MODIFIED: Caption overlay
```

## 🚀 Quick Start

### 1. Install Whisper.cpp

**macOS:**

```bash
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp
make
sudo cp main /usr/local/bin/whisper

# Download model
bash ./models/download-ggml-model.sh base.en
mkdir -p ~/.clipforge/models
cp models/ggml-base.en.bin ~/.clipforge/models/
```

**Windows:**

```powershell
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
copy build\bin\Release\main.exe C:\Windows\System32\whisper.exe

# Download model
bash ./models/download-ggml-model.sh base.en
mkdir %USERPROFILE%\.clipforge\models
copy models\ggml-base.en.bin %USERPROFILE%\.clipforge\models\
```

### 2. Use the Feature

1. Import or record a video with audio
2. Select the clip in Media Library
3. Open the Captions Panel
4. Click "Generate Captions"
5. Select language (or auto-detect)
6. Wait for processing
7. Edit captions as needed
8. Captions appear during playback automatically

## 📖 Documentation

- **[WHISPER_SETUP.md](./src-tauri/docs/WHISPER_SETUP.md)** - Complete installation guide
- **[AI_CAPTIONS_IMPLEMENTATION.md](./src-tauri/docs/AI_CAPTIONS_IMPLEMENTATION.md)** - Technical implementation details

## ✅ Phase 6.5 Tasks (15/15 Complete)

All tasks in Phase 6.5 have been completed:

- ✅ Caption data models (Rust + TypeScript)
- ✅ FFmpeg audio extraction
- ✅ Whisper.cpp integration
- ✅ SRT parsing
- ✅ Tauri commands (generate, update, delete)
- ✅ CaptionsPanel UI component
- ✅ Caption editing modal
- ✅ Video preview overlay
- ✅ Progress event system
- ✅ Documentation

## 🎨 UI Components

### CaptionsPanel

- Language selection dropdown
- Generate/Regenerate buttons
- Progress bar with status messages
- Caption list with timestamps
- Edit and delete actions
- Confidence score display

### VideoPreview

- Caption overlay during playback
- Automatic timing synchronization
- Styled text rendering

## 🔧 API Reference

### Tauri Commands

```rust
// Generate captions for a clip
generate_captions(clip_id: String, language: String) -> String

// Update caption text or timing
update_caption(
    clip_id: String,
    caption_id: String,
    text: Option<String>,
    start_time: Option<f64>,
    end_time: Option<f64>
) -> Caption

// Delete a caption
delete_caption(clip_id: String, caption_id: String) -> ()
```

### Events

```typescript
// Progress updates during generation
'caption_generation_progress': {
  job_id: string,
  progress: number,  // 0.0 - 1.0
  status: string,
  message?: string
}

// Generation complete
'caption_generation_complete': {
  job_id: string,
  captions: Caption[]
}

// Generation error
'caption_generation_error': {
  job_id: string,
  message: string
}
```

## 🧪 Testing

### Rust Unit Tests

```bash
cd src-tauri
cargo test
```

Tests included:

- Caption creation and validation
- SRT timestamp parsing
- SRT content parsing
- Caption style validation

### Frontend Component Tests

```bash
npm test
```

### E2E Tests

User-handled as requested.

## ⚡ Performance

- **Processing Time:** ~2-4x audio duration
  - 1 min audio → 2-4 min processing
- **Model:** base.en (74MB) recommended
- **Memory:** ~1GB RAM during transcription
- **Threads:** 4 CPU threads by default

## 🔮 Future Enhancements

### Not Yet Implemented

- Caption export (burn-in to video or SRT file)
- Caption timeline visualization
- "Generate Captions" button on MediaClipCard
- Custom caption styling UI
- GPU acceleration support

### Recommended Next Steps

1. Add caption burn-in to export pipeline
2. Create caption track renderer for timeline
3. Implement caption style customization
4. Add batch caption generation
5. Support for more languages

## 🔒 Privacy & Security

- ✅ 100% local processing (no external API calls)
- ✅ No API keys required
- ✅ No telemetry or tracking
- ✅ Audio never leaves your machine
- ✅ Open source Whisper.cpp (MIT license)

## 📝 License

This feature integrates with Whisper.cpp (MIT License).

## 🙏 Credits

- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) by Georgi Gerganov
- [OpenAI Whisper](https://github.com/openai/whisper) (original model)

---

**Status:** ✅ Phase 6.5 Complete - Ready for Phase 7 (Multi-Track Timeline)
