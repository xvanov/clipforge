# AI Speech-to-Text Captions Setup

## Overview

ClipForge uses **Whisper.cpp** for local AI-powered speech-to-text caption generation. This document explains how to set up Whisper.cpp to enable the caption generation feature.

## Prerequisites

- FFmpeg (already required for ClipForge)
- C++ compiler (for building Whisper.cpp)
- Git

## Installation

### macOS

1. **Install Whisper.cpp:**

```bash
# Clone whisper.cpp repository
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp

# Build whisper.cpp using CMake (recommended)
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release

# Find the main executable (it's in build/bin/)
ls -la build/bin/

# The main executable for CLI transcription is in build/bin/main
# Due to whisper.cpp naming changes, the simplest approach is to:
# 1. Use the full path (no copying/renaming needed), OR
# 2. Copy with a simple name and configure ClipForge

# Option 1: Copy to a simple name (recommended)
sudo cp build/bin/main /usr/local/bin/whisper-cli

# Option 2: Use full path (configure in ClipForge config later)
# No need to copy, just note the full path:
# /path/to/whisper.cpp/build/bin/main

# Verify installation (if you copied it)
whisper-cli --help

# Note: Whisper.cpp now shows deprecation warnings for any name except
# 'whisper-whisper' prefixed names. These warnings are safe to ignore,
# or you can use the full path to 'main' to avoid them entirely.
```

**Alternative with Make (if CMake doesn't work):**

```bash
cd whisper.cpp
make

# Executables will be in the root directory or build/bin/
# Check both locations:
ls -la main
ls -la build/bin/main

# Copy whichever exists
sudo cp main /usr/local/bin/whisper-main
# OR
sudo cp build/bin/main /usr/local/bin/whisper-main
```

2. **Download Whisper Models:**

Whisper.cpp requires language models. Download the recommended `base.en` model (English only) or `base` model (multilingual):

```bash
# Download English model (74MB, good balance of speed/accuracy)
bash ./models/download-ggml-model.sh base.en

# OR download multilingual model (141MB)
bash ./models/download-ggml-model.sh base

# Copy models to ClipForge directory
mkdir -p ~/.clipforge/models
cp models/ggml-*.bin ~/.clipforge/models/
```

### Windows

1. **Install Whisper.cpp:**

```powershell
# Clone whisper.cpp repository
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp

# Build using Visual Studio or MinGW
# With Visual Studio Developer Command Prompt:
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release

# Copy executable to PATH
copy build\bin\Release\main.exe C:\Windows\System32\whisper.exe
```

2. **Download models (same as macOS):**

```powershell
# Download English model
bash ./models/download-ggml-model.sh base.en

# Copy to ClipForge directory
mkdir %USERPROFILE%\.clipforge\models
copy models\ggml-*.bin %USERPROFILE%\.clipforge\models\
```

## Configuration

### Model Selection

ClipForge supports multiple Whisper model sizes. Choose based on your hardware:

| Model  | Size   | Speed    | Accuracy | RAM  |
| ------ | ------ | -------- | -------- | ---- |
| tiny   | 75 MB  | Fastest  | Basic    | 1 GB |
| base   | 141 MB | Fast     | Good     | 1 GB |
| small  | 466 MB | Moderate | Better   | 2 GB |
| medium | 1.5 GB | Slow     | High     | 4 GB |
| large  | 3.0 GB | Slowest  | Highest  | 8 GB |

**Recommended:** `base.en` for English-only content, or `base` for multilingual support.

### Custom Whisper Path

Whisper.cpp has changed their binary naming. The easiest approach is to configure the full path to the `main` executable.

Create a configuration file at `~/.clipforge/config.json`:

```json
{
  "whisper": {
    "executable_path": "/full/path/to/whisper.cpp/build/bin/main",
    "model_path": "~/.clipforge/models/ggml-base.en.bin",
    "default_language": "en"
  }
}
```

**Alternative names if you copied the binary:**

- `whisper-main` (our recommended name from setup)
- `whisper-whisper` (whisper.cpp's official name, no warnings)
- Full path to `build/bin/main` (avoids all naming issues)

**Example with copied binary:**

```json
{
  "whisper": {
    "executable_path": "/usr/local/bin/whisper-main",
    "model_path": "~/.clipforge/models/ggml-base.en.bin",
    "default_language": "en"
  }
}
```

## Usage

Once Whisper.cpp is installed:

1. **Import or record a video** in ClipForge with audio
2. **Select the clip** in the Media Library
3. **Click "Generate Captions"** in the Captions Panel
4. **Choose language** (or use auto-detect)
5. **Wait for processing** - progress will be shown
6. **Review and edit** generated captions as needed

## Performance

Caption generation speed depends on:

- **Model size:** Smaller models (tiny, base) are faster
- **CPU:** Whisper.cpp uses CPU by default; faster CPUs = faster processing
- **Audio duration:** Processing time is typically 2-4x audio duration
  - Example: 60 seconds of audio → 2-4 minutes processing time

### Optimization Tips

- Use `base.en` model for English content (faster than multilingual)
- Close other applications to free up CPU resources
- For large videos, generate captions for shorter clips
- Consider upgrading to `tiny` model for fastest results (lower accuracy)

## Troubleshooting

### "main executable not found after build"

**Solution:** Check your build output for errors

```bash
cd whisper.cpp

# Clean and rebuild with verbose output
make clean
make

# Check if main was created
ls -l main

# If main exists, you should see something like:
# -rwxr-xr-x  1 user  staff  1234567 Oct 30 12:00 main

# If build failed, you may need to install build tools:
# macOS:
xcode-select --install

# Or try alternative build with cmake:
mkdir build
cd build
cmake ..
make
# The executable will be in: ./bin/main
```

### "Whisper.cpp not found" Error

**Solution:** Configure the full path to the whisper.cpp `main` executable

```bash
# Find where whisper.cpp is installed
cd /path/to/whisper.cpp
pwd  # Note this path

# Verify the main executable exists
ls -la build/bin/main

# Create ClipForge config with full path
mkdir -p ~/.clipforge
cat > ~/.clipforge/config.json << 'EOF'
{
  "whisper": {
    "executable_path": "/full/path/to/whisper.cpp/build/bin/main",
    "model_path": "~/.clipforge/models/ggml-base.en.bin",
    "default_language": "en"
  }
}
EOF

# Replace "/full/path/to" with your actual whisper.cpp path
```

**If you copied the binary and get "not found":**

```bash
# Check what name you used
which whisper-main
which whisper-cli
which whisper

# Update ClipForge config to match:
# executable_path: "/usr/local/bin/whisper-main"
# or whatever name shows up in the 'which' command
```

### "Model file not found" Error

**Solution:** Download the model and place it in the correct location

```bash
# Check model location
ls ~/.clipforge/models/

# Re-download if missing
cd whisper.cpp
bash ./models/download-ggml-model.sh base.en
cp models/ggml-base.en.bin ~/.clipforge/models/
```

### Slow Processing

**Solutions:**

- Use smaller model (`tiny` or `base` instead of `large`)
- Reduce thread count in Whisper.cpp (default: 4 threads)
- Ensure no other CPU-intensive apps are running

### Poor Caption Accuracy

**Solutions:**

- Upgrade to larger model (`small`, `medium`, or `large`)
- Ensure audio quality is good (no heavy background noise)
- Verify correct language is selected
- Try manual editing for technical terms or names

## Advanced Configuration

### Custom Whisper.cpp Build

For optimal performance, build Whisper.cpp with CPU-specific optimizations:

```bash
# macOS (Intel)
make clean
make CFLAGS="-O3 -march=native"

# macOS (Apple Silicon - Metal acceleration)
make clean
WHISPER_METAL=1 make

# Linux
make clean
make CFLAGS="-O3 -march=native"
```

### GPU Acceleration (Optional)

Whisper.cpp supports GPU acceleration for faster processing:

- **NVIDIA (CUDA):** Build with `WHISPER_CUBLAS=1`
- **Apple Silicon (Metal):** Build with `WHISPER_METAL=1`
- **OpenCL:** Build with `WHISPER_OPENBLAS=1`

Refer to [Whisper.cpp documentation](https://github.com/ggerganov/whisper.cpp) for details.

## Resources

- [Whisper.cpp GitHub](https://github.com/ggerganov/whisper.cpp)
- [OpenAI Whisper](https://github.com/openai/whisper) (original Python version)
- [Model performance comparison](https://github.com/ggerganov/whisper.cpp#model-performance)

## Privacy Note

**All caption generation happens locally on your machine.** No audio is sent to external servers. Your privacy is fully protected.
