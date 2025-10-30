# Test Fixtures for ClipForge

## Video Files for Testing

### Location

Place test video files in `/tests/fixtures/videos/`

### Required Test Videos

For testing multiple clips functionality, you'll need 2-3 short video files:

1. **video1.mp4**
   - Duration: 5-10 seconds
   - Resolution: 1920x1080 or 1280x720
   - Format: MP4 (H.264)

2. **video2.mp4**
   - Duration: 5-10 seconds
   - Resolution: 1920x1080 or 1280x720
   - Format: MP4 (H.264)

3. **video3.mp4** (optional)
   - Duration: 5-10 seconds
   - Resolution: 1920x1080 or 1280x720
   - Format: MP4 (H.264)

### Creating Test Videos

#### Option 1: Use FFmpeg to create test videos

```bash
# Create a 5-second test video with colored background
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
  -vf "drawtext=text='Test Video 1':fontsize=60:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2" \
  tests/fixtures/videos/video1.mp4

ffmpeg -f lavfi -i testsrc=duration=7:size=1280x720:rate=30 \
  -vf "drawtext=text='Test Video 2':fontsize=60:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2" \
  tests/fixtures/videos/video2.mp4

ffmpeg -f lavfi -i testsrc=duration=10:size=1280x720:rate=30 \
  -vf "drawtext=text='Test Video 3':fontsize=60:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2" \
  tests/fixtures/videos/video3.mp4
```

#### Option 2: Use existing short clips

Copy any short video files (under 30 seconds) to `tests/fixtures/videos/`.

### Using Test Videos

#### In Manual Tests

1. Start the app: `npm run tauri:dev`
2. Import test videos from `tests/fixtures/videos/`
3. Follow the manual test guide

#### In Automated Tests (future)

```typescript
const testVideo1 = path.join(__dirname, '../fixtures/videos/video1.mp4');
const testVideo2 = path.join(__dirname, '../fixtures/videos/video2.mp4');
```

### File Structure

```
tests/
  fixtures/
    videos/
      video1.mp4        # Test video 1 (5-10s)
      video2.mp4        # Test video 2 (5-10s)
      video3.mp4        # Test video 3 (5-10s)
      README.md         # This file
```

### .gitignore

Video files are typically large, so they're ignored by git.

To share test videos with team:

1. Upload to cloud storage
2. Document download link in team docs
3. Or use FFmpeg command above to generate locally

---

**Note**: The `tests/fixtures/videos/` directory may be empty in the repository. Create test videos using FFmpeg or add your own short clips for testing.
