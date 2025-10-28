# Quick Start: Testing Multiple Clips on Timeline

## TL;DR - How to Test Right Now

### 1. Run the App

```bash
npm run tauri:dev
```

### 2. Follow the Manual Test Guide

```bash
cat tests/e2e/manual-timeline-test.md
```

### 3. What to Do

1. Import 2-3 videos into Media Library
2. Drag first video to timeline → Should see 1 clip
3. Drag second video to timeline → **Should see 2 clips** (BUG CHECK)
4. Drag third video to timeline → **Should see 3 clips** (BUG CHECK)

### 4. What to Watch

**Debug Panel (top-right, green text)**:

```
DEBUG
Tracks: 1
Track 0 "Main Track": 2 clips    ← Should increment!
  Clip 0: abc123... at 0.0s (dur: 5.0s)
  Clip 1: def456... at 5.5s (dur: 7.0s)
```

**Timeline CLIPS indicator**:

```
CLIPS: 2    ← Should increment!
```

**Browser Console** (F12 or Cmd+Option+I):

```
Dropping clip: video2.mp4 at time: 5.5 duration: 7.0
Adding to track: track-1 existing clips: 1  ← Should increment!
Auto-positioning clip after existing clips at: 5.5
Clip added successfully: { id: "...", ... }
```

**Terminal (where tauri:dev is running)**:

```
add_clip_to_timeline called: media=..., track=..., start=5.5
Created timeline clip: TimelineClip { id: "...", ... }
Added clip to track. Track now has 2 clips  ← Should increment!
```

### 5. Bug Indicators

❌ **BUG CONFIRMED** if:

- Debug panel shows "2 clips" but timeline shows only 1 clip
- OR debug panel shows "1 clips" always (doesn't increment)
- OR clips appear then disappear

✅ **WORKING** if:

- Debug panel increments: 1 clips → 2 clips → 3 clips
- Timeline shows all clips visually
- CLIPS indicator increments: 1 → 2 → 3

---

## Test Types Available

### ✅ Unit/Integration Tests (READY - ALL PASSING)

**What**: Tests the store logic without UI
**Command**: `npm test`
**Status**: ✅ 33 tests passing

**Purpose**: Verify the store can handle multiple clips
**Result**: All pass - store logic is correct

### ✅ Manual E2E Test (READY)

**What**: Step-by-step guide to test in the actual app
**Location**: `tests/e2e/manual-timeline-test.md`
**Command**: `npm run tauri:dev` + follow guide
**Status**: ✅ Ready to use

**Purpose**: See the bug happen live and debug it
**Result**: Use this to find where the bug is occurring

### 🚧 Automated E2E Tests (NOT YET IMPLEMENTED)

**What**: Automated visual tests with WebDriver/Playwright
**Status**: Planned for future
**See**: `tests/e2e/README.md` for setup instructions

---

## Debugging Workflow

### Step 1: Run Unit Tests

```bash
npm test
```

**Expected**: All 33 tests pass ✅  
**If they fail**: Fix store logic first

### Step 2: Run Manual E2E Test

```bash
npm run tauri:dev
```

Then follow `tests/e2e/manual-timeline-test.md`

**Expected**: Can drag multiple videos and see them all  
**If bug occurs**: Note which logs show the issue

### Step 3: Analyze Logs

Check all three log sources:

1. **Debug Panel** (UI) - Shows frontend state
2. **Browser Console** (F12) - Shows frontend logic
3. **Terminal** (backend) - Shows Rust commands

Compare them to find where data gets lost:

- Backend has clips but frontend doesn't → Sync issue
- Frontend store has clips but UI doesn't → Render issue
- Both have clips but disappear → State overwrite issue

### Step 4: Fix the Bug

Based on findings, fix in:

- `timeline.ts` - If store sync issue
- `Timeline.svelte` - If drag/drop issue
- `TrackView.svelte` - If rendering issue
- `timeline.rs` - If backend persistence issue

### Step 5: Verify Fix

```bash
npm test  # Unit tests still pass
npm run tauri:dev  # Manual test now works
```

---

## Files Created

### Test Files

- ✅ `src/lib/stores/timeline.test.ts` - Unit/integration tests (7 new tests)
- ✅ `tests/e2e/manual-timeline-test.md` - Manual E2E test guide
- ✅ `tests/e2e/README.md` - E2E test overview
- ✅ `tests/fixtures/README.md` - Test video fixtures guide

### Documentation

- ✅ `TEST_GUIDE.md` - Comprehensive debugging guide
- ✅ `TEST_IMPLEMENTATION_SUMMARY.md` - Implementation details
- ✅ `QUICK_START_TESTING.md` - This file

---

## Need Help?

1. **Tests failing?** → Check `TEST_GUIDE.md` for debugging steps
2. **Don't see videos?** → Check `tests/fixtures/README.md` for creating test videos
3. **Want to automate?** → Check `tests/e2e/README.md` for WebDriver setup

---

**Ready to Test**: ✅ Yes!  
**Command**: `npm run tauri:dev`  
**Guide**: `tests/e2e/manual-timeline-test.md`  
**Status**: All test infrastructure ready

