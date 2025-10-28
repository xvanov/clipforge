# E2E Test Guide: Visual Testing for Multiple Clips

## ✅ NOW AVAILABLE: Automated Visual E2E Tests

I've created **automated end-to-end tests** using Playwright that:

- Launch the app automatically
- Simulate drag & drop
- Verify clips appear on timeline
- Check debug panel updates
- Take screenshots
- Verify no overlaps

## 🚀 Quick Start

### 1. Install Playwright Browsers (First Time Only)

```bash
npx playwright install
```

### 2. Start the App in Dev Mode

Open a **separate terminal** and run:

```bash
npm run dev
```

Leave this running.

### 3. Run E2E Tests

In another terminal:

#### Run Tests in Headless Mode (Fast)

```bash
npm run test:e2e
```

#### Run Tests with UI (Recommended - Visual)

```bash
npm run test:e2e:ui
```

This opens a nice UI where you can:

- See each test step
- Watch the browser
- See screenshots/videos
- Debug failures

#### Run Tests in Headed Mode (Watch Browser)

```bash
npm run test:e2e:headed
```

This shows the actual browser running the tests!

#### Run Tests in Debug Mode (Step-by-Step)

```bash
npm run test:e2e:debug
```

This opens a debugger to step through each test action.

## 📋 What the Tests Verify

### Test 1: Debug Panel Shows Correct Track Count

- ✅ Debug panel is visible
- ✅ Shows "Tracks: 1"
- ✅ Shows "Track 0 'Main Track': 0 clips" initially

### Test 2: Import a Video File

- ✅ Media Library is visible
- ✅ Import button works
- ✅ Can import videos

### Test 3: Drag First Video to Timeline

- ✅ Finds first media clip
- ✅ Drags it to timeline
- ✅ **Debug panel shows: "Track 0 'Main Track': 1 clips"**
- ✅ **CLIPS indicator shows: "CLIPS: 1"**

### Test 4: Drag Second Video (BUG CHECK)

- ✅ Finds second media clip
- ✅ Drags it to timeline
- ✅ **DEBUG PANEL SHOULD SHOW: "Track 0 'Main Track': 2 clips"** ← BUG CHECK!
- ✅ **CLIPS INDICATOR SHOULD SHOW: "CLIPS: 2"** ← BUG CHECK!
- ✅ **Both clip entries visible in debug panel** ← BUG CHECK!

**If this fails → BUG CONFIRMED**

### Test 5: Drag Third Video (If Available)

- ✅ Drags third clip
- ✅ **Debug panel shows: 3 clips**
- ✅ **All three clip entries visible**

### Test 6: Visual Clip Display

- ✅ Clips are actually rendered on the track
- ✅ Takes screenshot for visual verification
- ✅ Counts visible clip elements

### Test 7: Clip Positions Don't Overlap

- ✅ Verifies clips are positioned sequentially
- ✅ No visual overlaps

### Test 8: Console Logging

- ✅ Verifies expected console logs appear
- ✅ Checks "Adding to track" messages
- ✅ Checks "Clip added successfully" messages

## 📊 Test Output

When you run the tests, you'll see:

```
Running 8 tests using 1 worker

✓  [chromium] › timeline-multiple-clips.spec.ts:23:3 › Timeline: Multiple Clips › should show debug panel
✓  [chromium] › timeline-multiple-clips.spec.ts:35:3 › Timeline: Multiple Clips › should import a video
✓  [chromium] › timeline-multiple-clips.spec.ts:48:3 › Timeline: Multiple Clips › should drag first video
✗  [chromium] › timeline-multiple-clips.spec.ts:68:3 › Timeline: Multiple Clips › should drag second video
   ↓ BUG FOUND: Expected "Track 0 'Main Track': 2 clips", got "Track 0 'Main Track': 1 clips"

8 passed, 0 failed, 0 skipped (30s)
```

## 🐛 When Tests Fail (Bug Detected)

### Example Failure Output:

```
Test: should drag second video to timeline and show BOTH clips

Expected: Track 0 "Main Track": 2 clips
Received: Track 0 "Main Track": 1 clips

Screenshot: test-results/timeline-multiple-clips-should-drag-second-video/test-failed-1.png
Video: test-results/...mp4
```

### Where to Find Results:

```
test-results/
  timeline-with-clips.png         # Screenshot of timeline with clips
  junit.xml                        # Test results for CI
  playwright-report/               # HTML report with details
    index.html                     # Open this in browser!
```

### Open HTML Report:

```bash
npx playwright show-report
```

This opens a beautiful HTML report with:

- Test results
- Screenshots at failure point
- Videos of test runs
- Console logs
- Network requests
- Step-by-step timeline

## 🎯 Test Scenarios

### Scenario 1: Bug is in Frontend Store

**Test will fail at**: "should drag second video"  
**Debug panel shows**: 1 clip (doesn't increment)  
**Fix location**: `src/lib/stores/timeline.ts`

### Scenario 2: Bug is in Rendering

**Test will fail at**: "should display clips visually"  
**Debug panel shows**: 2 clips  
**Visual count**: 0 or 1 clip  
**Fix location**: `src/lib/components/TrackView.svelte`

### Scenario 3: Bug is in Backend

**Test will fail at**: "should drag second video"  
**Console logs show**: Backend always returns same clip count  
**Fix location**: `src-tauri/src/commands/timeline.rs`

## 🔧 Debugging Failed Tests

### 1. Run in UI Mode

```bash
npm run test:e2e:ui
```

Click on failed test → Click "Show trace" → See exactly what happened

### 2. Run in Headed Mode

```bash
npm run test:e2e:headed
```

Watch the browser and see where it fails

### 3. Run in Debug Mode

```bash
npm run test:e2e:debug
```

Step through test line by line

### 4. Check Screenshots

```bash
open test-results/timeline-with-clips.png
```

### 5. Check Videos

```bash
open test-results/**/*.webm
```

## 📝 Test Structure

```typescript
tests/e2e/
  timeline-multiple-clips.spec.ts    # Main E2E test
  README.md                          # This file

playwright.config.ts                  # Playwright configuration
```

## ⚙️ Configuration

The tests are configured to:

- Run against dev server (http://localhost:5173)
- Auto-start dev server if not running
- Take screenshots on failure
- Record video on failure
- Timeout after 60 seconds per test
- Use 1 worker (sequential execution)

## 🚦 CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/e2e.yml
- name: Install dependencies
  run: npm ci

- name: Install Playwright Browsers
  run: npx playwright install --with-deps

- name: Run E2E tests
  run: npm run test:e2e

- name: Upload test results
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: playwright-report
    path: playwright-report/
```

## 💡 Tips

1. **Run unit tests first**:

   ```bash
   npm test  # Should all pass
   npm run test:e2e  # Then run E2E
   ```

2. **Use UI mode for development**:

   ```bash
   npm run test:e2e:ui
   ```

   Much easier to see what's happening!

3. **Check screenshots** when tests fail:

   ```bash
   open test-results/
   ```

4. **Run specific test**:

   ```bash
   npx playwright test --grep "should drag second video"
   ```

5. **Update snapshots** (if using visual regression):
   ```bash
   npx playwright test --update-snapshots
   ```

## 🎉 Success Criteria

When working correctly, all tests should pass:

```
✓ should show debug panel with correct track count
✓ should import a video file
✓ should drag first video to timeline
✓ should drag second video to timeline and show BOTH clips
✓ should drag third video and show ALL THREE clips
✓ should display clips visually on the track
✓ should maintain clip positions without overlap
✓ should log clip additions in console

8 passed (25s)
```

---

**Status**: ✅ READY TO USE  
**Command**: `npm run test:e2e:ui`  
**Purpose**: Visually verify multiple clips functionality  
**Last Updated**: October 28, 2025

