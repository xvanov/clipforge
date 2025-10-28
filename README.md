# ClipForge 🎬

A powerful desktop video editor built with Tauri, Svelte, and FFmpeg. Import videos, edit on a timeline, and export professional content - all from a native desktop app.

![ClipForge](docs/screenshot-placeholder.png)

## Features

### MVP (Currently Implementing)

- ✅ **Phase 1 & 2 Complete**: Project structure and foundation
- ✅ **User Story 1**: Import and Basic Video Playback (with MOV/ProRes support)
- 🚧 **User Story 2**: Timeline Editing and Trimming
- 🚧 **User Story 3**: Video Export

### Planned Features

- 📹 Screen and webcam recording (macOS & Windows)
- 🎞️ Multi-track timeline with overlays
- 💬 AI-powered captions (Whisper.cpp)
- ✨ Effects and transitions
- 💾 Project save/load

## Tech Stack

- **Desktop Framework**: Tauri 1.5+ (Rust backend)
- **Frontend**: Svelte 4.0+ with TypeScript
- **Media Processing**: FFmpeg 6.0+
- **Timeline**: HTML5 Canvas (custom renderer)
- **State Management**: Svelte stores

## Prerequisites

- **Rust**: 1.75+ ([Install](https://rustup.rs/))
- **Node.js**: 18+ LTS ([Install](https://nodejs.org/))
- **FFmpeg**: 6.0+ ([Install](https://ffmpeg.org/))
- **Platform-specific**:
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio 2022 Build Tools

## Quick Start

### 1. Install Dependencies

```bash
# Install Node.js dependencies
npm install

# The Rust dependencies will be installed automatically when running Tauri
```

### 2. Run Development Server

```bash
npm run tauri:dev
```

This will:

1. Start the Vite dev server (frontend with hot reload)
2. Build the Rust backend
3. Launch the Tauri window

**Note**: First run takes 2-5 minutes due to Rust compilation. Subsequent runs are much faster (~30 seconds).

### 3. Build for Production

```bash
npm run tauri:build
```

**Output**:

- **macOS**: `src-tauri/target/release/bundle/macos/ClipForge.app`
- **Windows**: `src-tauri/target/release/bundle/msi/ClipForge.msi`

## Development Workflow

### Project Structure

```
clipforge/
├── src-tauri/          # Rust backend (Tauri)
│   ├── src/
│   │   ├── commands/   # Tauri command handlers
│   │   ├── ffmpeg/     # FFmpeg integration
│   │   ├── models/     # Data models
│   │   └── storage/    # Persistence layer
│   └── Cargo.toml
│
├── src/                # Svelte frontend
│   ├── lib/
│   │   ├── components/ # UI components
│   │   ├── stores/     # State management
│   │   └── types/      # TypeScript types
│   └── App.svelte
│
├── specs/              # Design documentation
│   └── 001-clipforge/
│       ├── spec.md     # Feature specification
│       ├── plan.md     # Implementation plan
│       ├── tasks.md    # Task breakdown (124 tasks)
│       └── ...
│
└── tests/              # Tests
    ├── integration/
    └── e2e/
```

### Running Tests

```bash
# Frontend tests
npm test

# Frontend tests (watch mode for development)
npm run test:watch

# Rust tests
cd src-tauri && cargo test
```

### Code Quality Checks

```bash
# Frontend linting
npm run lint

# Frontend type checking
npm run type-check

# Frontend formatting check
npm run format:check

# Frontend formatting fix
npm run format:write

# Rust linting (clippy)
cd src-tauri && cargo clippy

# Rust linting (strict mode - CI/CD uses this)
cd src-tauri && cargo clippy -- -D warnings

# Rust formatting check
cd src-tauri && cargo fmt --check

# Rust formatting fix
cd src-tauri && cargo fmt
```

### CI/CD Validation (Local)

Run all CI/CD checks locally before pushing to ensure the pipeline will pass:

#### Quick Method (All Checks)

```bash
# Frontend checks
npm test && npm run lint && npm run type-check && npm run format:check

# Rust checks (from src-tauri directory)
cd src-tauri && cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

#### Step-by-Step Method

**Frontend Checks** (runs on Ubuntu in CI/CD):

```bash
# 1. Install dependencies (if not already installed)
npm install

# 2. Build frontend (required for Rust tests)
npm run build

# 3. Run tests
npm test

# 4. Run linter
npm run lint

# 5. Run type checker
npm run type-check

# 6. Check formatting
npm run format:check
```

**Rust Checks** (runs on macOS & Windows in CI/CD):

```bash
# Navigate to Rust directory
cd src-tauri

# 1. Run tests
cargo test

# 2. Run clippy (with warnings as errors)
cargo clippy -- -D warnings

# 3. Check formatting
cargo fmt --check

# Go back to project root
cd ..
```

#### Complete CI/CD Simulation

Run this comprehensive command to simulate the entire CI/CD pipeline:

```bash
# From project root
echo "=== FRONTEND CHECKS ===" && \
npm test && \
npm run lint && \
npm run type-check && \
npm run format:check && \
echo "=== RUST CHECKS ===" && \
cd src-tauri && \
cargo test && \
cargo clippy -- -D warnings && \
cargo fmt --check && \
cd .. && \
echo "✅ ALL CI/CD CHECKS PASSED!"
```

**Expected Output:**
- ✅ Frontend: 27 tests passing, 0 linting errors, 0 type errors, formatting valid
- ✅ Rust: 9 tests passing, 0 clippy warnings, formatting valid

#### Fixing Issues

If checks fail:

```bash
# Fix frontend formatting
npm run format:write

# Fix Rust formatting
cd src-tauri && cargo fmt

# Fix linting issues (review and fix manually)
npm run lint
cd src-tauri && cargo clippy
```

### Full Build Test

```bash
# Test production build
npm run tauri:build
```

## Implementation Status

### Phase 1: Setup ✅ COMPLETE

- [x] T001-T010: Project initialization, configuration, CI/CD

### Phase 2: Foundational ✅ COMPLETE

- [x] T011-T026: Data models, stores, infrastructure

### Phase 3: User Story 1 (Import & Playback) ✅ COMPLETE

- [x] T027-T040b: Media import, thumbnails, video preview, proxy generation
- **Achievement**: Full video import and playback with MOV/ProRes support

### Phase 4: User Story 2 (Timeline Editing) 📋 TODO

- [ ] T041-T061: Canvas timeline, drag-drop, trim/split

### Phase 5: User Story 3 (Export) 📋 TODO

- [ ] T062-T075: FFmpeg export pipeline

See [`specs/001-clipforge/tasks.md`](specs/001-clipforge/tasks.md) for complete task list.

## Architecture Overview

### Data Flow

```
User Interaction (Svelte)
    ↓
Svelte Stores (State Management)
    ↓
Tauri Commands (Rust IPC)
    ↓
Rust Backend (FFmpeg, Storage, Platform APIs)
```

### Key Components

**Rust Backend** (`src-tauri/src/`):

- **Commands**: Tauri IPC handlers (media, project, timeline, export, recording)
- **Models**: Data structures (Project, MediaClip, TimelineClip, Track)
- **FFmpeg**: Media processing (import, thumbnail, proxy generation, export, transcode)
- **Storage**: Persistence (JSON projects, SQLite cache)

**Svelte Frontend** (`src/`):

- **Components**: UI (MediaLibrary, VideoPreview, Timeline, ExportDialog)
- **Stores**: State management (project, mediaLibrary, timeline)
- **Canvas**: Custom timeline renderer (60fps, drag-drop, zoom)

## Contributing

### Development Guidelines

1. **Follow the task list**: See `specs/001-clipforge/tasks.md`
2. **TDD approach**: Write tests before implementation (when applicable)
3. **Constitution compliance**: Follow `.specify/memory/constitution.md`
4. **Local validation**: Run all checks before committing
5. **Atomic commits**: One task per commit when possible

### Task Implementation Pattern

Each task follows this pattern:

1. Read task description in `tasks.md`
2. Implement the feature
3. Mark task as complete: `- [x] TXXX ...`
4. Commit: `feat(taskXXX): description`

### Example: Implementing Media Import (T027)

```rust
// src-tauri/src/commands/media.rs
#[tauri::command]
pub async fn import_media_files(paths: Vec<String>) -> Result<Vec<MediaClip>, String> {
    // 1. Validate file paths
    // 2. Extract metadata using FFmpeg
    // 3. Generate thumbnails (async)
    // 4. Return MediaClip structs
}
```

## Troubleshooting

### npm Permission Errors

```bash
sudo chown -R $(whoami) ~/.npm
```

### FFmpeg Not Found

**macOS**:

```bash
brew install ffmpeg
```

**Windows**:

```bash
choco install ffmpeg
```

### Slow Rust Compilation

Use faster linker (macOS):

```bash
brew install llvm
# Add to ~/.cargo/config.toml:
# [target.x86_64-apple-darwin]
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Hot Reload Not Working

```bash
# Clean and rebuild
rm -rf target node_modules dist
npm install
npm run tauri:dev
```

## License

MIT © 2025 ClipForge Team

## Resources

- **Documentation**: [`specs/001-clipforge/`](specs/001-clipforge/)
- **Task List**: [`specs/001-clipforge/tasks.md`](specs/001-clipforge/tasks.md)
- **Tauri Docs**: https://tauri.app/v1/guides/
- **Svelte Tutorial**: https://svelte.dev/tutorial
- **FFmpeg Wiki**: https://trac.ffmpeg.org/wiki

---

**Status**: 🚧 Active Development - MVP in progress (Phases 1-2 complete)
