# Quickstart Guide: ClipForge Development

**Feature**: ClipForge Desktop Video Editor  
**Date**: 2025-10-27  
**Purpose**: Get development environment set up and running in under 30 minutes

## Prerequisites

Before you begin, ensure you have these tools installed:

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.75+ | Tauri backend development |
| **Node.js** | 18+ LTS | Frontend build tools |
| **npm** | 9+ | Package management |
| **FFmpeg** | 6.0+ | Media processing (runtime dependency) |
| **Git** | 2.30+ | Version control |

### Platform-Specific

**macOS**:
- Xcode Command Line Tools: `xcode-select --install`
- Homebrew (recommended): `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`

**Windows**:
- Visual Studio 2022 Build Tools with C++ workload
- Windows 10 SDK

**Linux** (future support):
- webkit2gtk-4.0
- libappindicator3-dev

---

## Installation Steps

### 1. Install Rust

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download and run: https://rustup.rs/

# Verify installation
rustc --version  # Should show 1.75+
cargo --version
```

---

### 2. Install Node.js and npm

**macOS** (Homebrew):
```bash
brew install node@18
```

**Windows**:
- Download from https://nodejs.org/
- Install LTS version (18.x)

**Verify**:
```bash
node --version  # Should show v18.x
npm --version   # Should show 9.x
```

---

### 3. Install FFmpeg

**macOS** (Homebrew):
```bash
brew install ffmpeg
```

**Windows** (Chocolatey):
```bash
choco install ffmpeg
```

**Windows** (Manual):
1. Download from https://ffmpeg.org/download.html
2. Extract to `C:\ffmpeg`
3. Add `C:\ffmpeg\bin` to PATH

**Verify**:
```bash
ffmpeg -version  # Should show 6.0+
```

---

### 4. Clone Repository

```bash
git clone https://github.com/yourusername/clipforge.git
cd clipforge
```

---

### 5. Install Dependencies

**Install Rust dependencies**:
```bash
cd src-tauri
cargo build
cd ..
```

**Install Node dependencies**:
```bash
npm install
```

This will install:
- Svelte + SvelteKit
- Vite (bundler)
- TypeScript
- Tauri CLI
- Testing frameworks (Vitest, Playwright)
- Linting tools (ESLint, Prettier)

---

## Development Workflow

### Running the App in Dev Mode

```bash
npm run tauri dev
```

This command:
1. Starts Vite dev server (frontend hot reload on port 1420)
2. Builds Rust backend
3. Launches Tauri window with dev tools

**First run takes 2-5 minutes** (Rust compilation). Subsequent runs are faster (~30 seconds).

---

### Project Structure Quick Tour

```
clipforge/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── main.rs    # Tauri app entry point
│   │   ├── commands/  # Tauri command handlers
│   │   └── ffmpeg/    # FFmpeg integration
│   └── Cargo.toml     # Rust dependencies
│
├── src/                # Svelte frontend
│   ├── lib/
│   │   ├── components/  # UI components
│   │   ├── stores/      # Svelte stores
│   │   └── services/    # API wrappers
│   └── App.svelte     # Root component
│
├── tests/             # Integration & E2E tests
│   ├── integration/   # Rust integration tests
│   └── e2e/           # Playwright E2E tests
│
└── package.json       # Node dependencies
```

---

### Making Your First Change

Let's add a simple button to the UI:

**1. Edit frontend component** (`src/App.svelte`):
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  
  async function greet() {
    const message = await invoke<string>('greet', { name: 'ClipForge' });
    alert(message);
  }
</script>

<button on:click={greet}>
  Greet ClipForge
</button>
```

**2. Add Rust command** (`src-tauri/src/main.rs`):
```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**3. Save files** → Watch hot reload in dev window

---

## Testing

### Run Rust Tests

```bash
cd src-tauri
cargo test
```

**Expected output**: All tests pass in under 5 seconds

---

### Run Frontend Tests

```bash
npm test
```

Runs Vitest component tests.

---

### Run E2E Tests

```bash
npm run test:e2e
```

Launches Playwright for end-to-end testing.

---

## Linting & Formatting

### Check Code Quality

**Rust linting** (Clippy):
```bash
cd src-tauri
cargo clippy -- -D warnings
```

**Rust formatting**:
```bash
cargo fmt --check
```

**Frontend linting** (ESLint):
```bash
npm run lint
```

**Frontend formatting** (Prettier):
```bash
npm run format:check
```

---

### Auto-fix Issues

**Rust**:
```bash
cargo fmt  # Auto-format
```

**Frontend**:
```bash
npm run lint:fix      # Auto-fix ESLint issues
npm run format:write  # Auto-format with Prettier
```

---

## Building for Production

### Development Build

```bash
npm run tauri build
```

**Output location**:
- **macOS**: `src-tauri/target/release/bundle/macos/ClipForge.app`
- **Windows**: `src-tauri/target/release/bundle/msi/ClipForge.msi`

**Build time**: 5-10 minutes (first build), 2-3 minutes (incremental)

---

### Test Production Build

**macOS**:
```bash
open src-tauri/target/release/bundle/macos/ClipForge.app
```

**Windows**:
```bash
.\src-tauri\target\release\clipforge.exe
```

---

## CI/CD Validation (Pre-Push Checklist)

**Run ALL checks before committing** (Constitution requirement):

```bash
# 1. Rust tests
cd src-tauri && cargo test && cd ..

# 2. Rust linting
cd src-tauri && cargo clippy -- -D warnings && cd ..

# 3. Rust formatting
cd src-tauri && cargo fmt --check && cd ..

# 4. Frontend tests
npm test

# 5. Frontend linting
npm run lint

# 6. Frontend type checking
npm run type-check

# 7. Build verification
npm run tauri build
```

**All must pass** before `git push` (per Constitution CI/CD principle).

---

## Troubleshooting

### Issue: "Rust compiler not found"

**Solution**:
```bash
# Add Rust to PATH
source $HOME/.cargo/env

# Or restart terminal
```

---

### Issue: "FFmpeg not found" during runtime

**Solution macOS**:
```bash
brew install ffmpeg
```

**Solution Windows**:
- Download FFmpeg from https://ffmpeg.org/
- Add to PATH
- Restart terminal

---

### Issue: "webkit2gtk not found" (Linux)

**Solution Ubuntu/Debian**:
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

---

### Issue: Slow Rust compilation

**Solution**: Enable faster linker

**macOS**:
```bash
# Install lld
brew install llvm

# Add to ~/.cargo/config.toml
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**Linux**:
```bash
sudo apt install lld
```

---

### Issue: Hot reload not working

**Solution**:
1. Stop dev server (Ctrl+C)
2. Clear build cache: `rm -rf target node_modules`
3. Reinstall: `npm install && cd src-tauri && cargo clean && cargo build`
4. Restart: `npm run tauri dev`

---

## IDE Setup

### Recommended: VS Code

**Install Extensions**:
1. **rust-analyzer** - Rust LSP support
2. **Svelte for VS Code** - Svelte syntax
3. **Tauri** - Tauri project support
4. **ESLint** - JavaScript linting
5. **Prettier** - Code formatting

**Settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode"
  }
}
```

---

### Alternative: Other IDEs

**IntelliJ IDEA / WebStorm**:
- Install Rust plugin
- Install Svelte plugin
- Enable ESLint and Prettier

**Vim / Neovim**:
- Use rust-analyzer LSP
- Install svelte-language-server
- Configure ALE or CoC for linting

---

## Common Development Tasks

### Add New Tauri Command

**1. Define Rust handler** (`src-tauri/src/commands/media.rs`):
```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Processed: {}", param))
}
```

**2. Register command** (`src-tauri/src/main.rs`):
```rust
use crate::commands::media::my_command;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![my_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**3. Call from frontend** (`src/lib/services/tauri-api.ts`):
```typescript
import { invoke } from '@tauri-apps/api/tauri';

export async function myCommand(param: string): Promise<string> {
  return await invoke<string>('my_command', { param });
}
```

---

### Add New Svelte Component

**1. Create component** (`src/lib/components/MyComponent.svelte`):
```svelte
<script lang="ts">
  export let title: string;
</script>

<div class="my-component">
  <h2>{title}</h2>
</div>

<style>
  .my-component {
    padding: 1rem;
  }
</style>
```

**2. Use in parent**:
```svelte
<script>
  import MyComponent from '$lib/components/MyComponent.svelte';
</script>

<MyComponent title="Hello World" />
```

---

### Run FFmpeg Command from Rust

```rust
use std::process::Command;
use tokio::process::Command as AsyncCommand;

#[tauri::command]
async fn extract_thumbnail(input_path: String) -> Result<String, String> {
    let output_path = "/tmp/thumbnail.jpg";
    
    let output = AsyncCommand::new("ffmpeg")
        .arg("-i")
        .arg(&input_path)
        .arg("-vframes")
        .arg("1")
        .arg("-q:v")
        .arg("2")
        .arg(output_path)
        .output()
        .await
        .map_err(|e| format!("FFmpeg failed: {}", e))?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    
    Ok(output_path.to_string())
}
```

---

## Development Best Practices

### 1. Test-Driven Development

**Write tests BEFORE implementation** (Constitution requirement):

```rust
// tests/integration/thumbnail_test.rs
#[tokio::test]
async fn test_extract_thumbnail() {
    let input = "tests/fixtures/sample.mp4";
    let result = extract_thumbnail(input.to_string()).await;
    assert!(result.is_ok());
    assert!(std::path::Path::new(&result.unwrap()).exists());
}
```

**Then implement** to make test pass.

---

### 2. Local CI/CD Validation

**Before every commit** (Constitution requirement):

```bash
# Run full validation suite
./scripts/validate-local.sh
```

Create this script:
```bash
#!/bin/bash
set -e

echo "Running Rust tests..."
cd src-tauri && cargo test && cd ..

echo "Running Rust linter..."
cd src-tauri && cargo clippy -- -D warnings && cd ..

echo "Running Rust formatter..."
cd src-tauri && cargo fmt --check && cd ..

echo "Running frontend tests..."
npm test

echo "Running frontend linter..."
npm run lint

echo "Running type check..."
npm run type-check

echo "Building app..."
npm run tauri build

echo "✅ All checks passed!"
```

Make executable: `chmod +x scripts/validate-local.sh`

---

### 3. Git Workflow

```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Make changes, test locally
./scripts/validate-local.sh

# 3. Commit (only if validation passes)
git add .
git commit -m "feat: add my feature"

# 4. Push (CI/CD will run remotely)
git push origin feature/my-feature
```

---

## Next Steps

Now that your environment is set up:

1. **Read the spec**: `specs/001-clipforge/spec.md`
2. **Review data model**: `specs/001-clipforge/data-model.md`
3. **Study API contracts**: `specs/001-clipforge/contracts/tauri-commands.md`
4. **Start with MVP** (P1 user stories):
   - Import and playback
   - Timeline editing
   - Video export

---

## Resources

### Documentation

- **Tauri Docs**: https://tauri.app/v1/guides/
- **Svelte Tutorial**: https://svelte.dev/tutorial
- **FFmpeg Wiki**: https://trac.ffmpeg.org/wiki
- **Rust Book**: https://doc.rust-lang.org/book/

### Community

- **Tauri Discord**: https://discord.com/invite/tauri
- **Svelte Discord**: https://svelte.dev/chat
- **Project Issues**: https://github.com/yourusername/clipforge/issues

---

## Summary

**Setup time**: ~15-30 minutes (depending on download speeds)

**Development loop**:
1. Run `npm run tauri dev` → App launches
2. Edit code → Hot reload updates
3. Test changes → `npm test` / `cargo test`
4. Validate → `./scripts/validate-local.sh`
5. Commit → `git commit`
6. Push → CI/CD validates

**Ready to code!** Start with implementing media import (FR-005 through FR-009).

