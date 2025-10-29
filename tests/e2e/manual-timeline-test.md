# Manual E2E Test: Multiple Videos on Timeline Track

## Purpose

Verify that multiple videos can be dragged onto the timeline track and all appear correctly.

## Prerequisites

- App is running in dev mode: `npm run tauri:dev`
- At least 2-3 video files available (MP4, MOV, etc.)

## Test Scenario: Multiple Clips on Timeline

### Setup

1. Launch the app: `npm run tauri:dev`
2. Wait for app to fully load
3. Verify debug panel appears (top-right, green text)

### Test Steps

#### Step 1: Import Videos

**Action**: Import 2-3 video files

- Option A: Drag & drop video files onto Media Library (left panel)
- Option B: Click "Import Media" button and select files

**Expected Result**:

- ✅ Video thumbnails appear in Media Library
- ✅ Each thumbnail shows: filename, resolution, fps, duration
- ✅ Console shows: "Media clip imported: [filename]"

**Verification**:

```
Media Library count: [X] clips
```

---

#### Step 2: Drag First Video to Timeline

**Action**: Drag first video from Media Library onto the timeline (bottom section)

**Expected Result**:

- ✅ Clip appears on "Main Track"
- ✅ Debug panel shows: "Track 0 'Main Track': 1 clips"
- ✅ Console shows: "Adding to track: track-1, existing clips: 0"
- ✅ Console shows: "Clip added successfully: [clip-data]"
- ✅ Timeline shows green "CLIPS: 1" indicator

**Debug Panel Should Show**:

```
DEBUG
Tracks: 1
Track 0 "Main Track": 1 clips
  Clip 0: [id]... at 0.0s (dur: [X]s)
```

**If This Fails**: Check console for errors

---

#### Step 3: Drag Second Video to Timeline

**Action**: Drag second video from Media Library onto the timeline

**Expected Result**:

- ✅ **BOTH clips appear on "Main Track"** (not just the new one)
- ✅ Debug panel shows: "Track 0 'Main Track': 2 clips"
- ✅ Console shows: "Adding to track: track-1, existing clips: 1"
- ✅ Console shows: "Auto-positioning clip after existing clips at: [time]"
- ✅ Timeline shows green "CLIPS: 2" indicator
- ✅ Second clip appears AFTER first clip (with small gap)

**Debug Panel Should Show**:

```
DEBUG
Tracks: 1
Track 0 "Main Track": 2 clips
  Clip 0: [id]... at 0.0s (dur: [X]s)
  Clip 1: [id]... at [Y]s (dur: [X]s)
```

**🐛 BUG CHECK**:

- If only 1 clip shows → BUG CONFIRMED
- If debug panel shows "2 clips" but UI shows 1 → Rendering issue
- If console shows error → Check error message

---

#### Step 4: Drag Third Video to Timeline

**Action**: Drag third video from Media Library onto the timeline

**Expected Result**:

- ✅ **ALL THREE clips appear on "Main Track"**
- ✅ Debug panel shows: "Track 0 'Main Track': 3 clips"
- ✅ Console shows: "Adding to track: track-1, existing clips: 2"
- ✅ Timeline shows green "CLIPS: 3" indicator
- ✅ All three clips visible and positioned sequentially

**Debug Panel Should Show**:

```
DEBUG
Tracks: 1
Track 0 "Main Track": 3 clips
  Clip 0: [id]... at 0.0s (dur: [X]s)
  Clip 1: [id]... at [Y]s (dur: [X]s)
  Clip 2: [id]... at [Z]s (dur: [X]s)
```

---

### Additional Verification

#### Check 1: Visual Timeline

- Open browser DevTools (if running in browser)
- All 3 clip rectangles should be visible on the track
- Each clip should have different position/color

#### Check 2: Console Logs

Look for these log patterns:

```
Dropping clip: [name] at time: [X] duration: [Y]
Adding to track: track-1 existing clips: [N]
Auto-positioning clip after existing clips at: [X]
Clip added successfully: [clip-data]
Current tracks state: [tracks-array]
```

#### Check 3: Store State (Browser Console)

If running in dev mode, you can inspect the store:

```javascript
// In browser console:
window.__TAURI__.invoke('create_track', { name: 'Test', trackType: 'main' });
```

#### Check 4: Rust Backend Logs

In the terminal where `npm run tauri:dev` is running, look for:

```
add_clip_to_timeline called: media=[id], track=[id], start=[X]
Created timeline clip: TimelineClip { id: "[uuid]", ... }
Added clip to track. Track now has 1 clips
Added clip to track. Track now has 2 clips  ← Should increment!
Added clip to track. Track now has 3 clips
```

**🐛 BUG INDICATORS**:

- If backend always shows "Track now has 1 clips" → Backend state issue
- If backend increments but UI doesn't → Frontend sync issue
- If no backend logs → Command not being called

---

## Bug Diagnosis

### Scenario A: Debug Panel Shows Multiple Clips, UI Doesn't

**Problem**: State is correct, rendering is broken
**Location**: TrackView.svelte or TimelineClipView.svelte
**Fix**: Check component reactivity, CSS positioning

### Scenario B: Debug Panel Shows 1 Clip, Backend Shows Multiple

**Problem**: Frontend store not syncing with backend
**Location**: timeline.ts `addClipToTimeline` method
**Fix**: Check store update logic

### Scenario C: Backend Shows 1 Clip Always

**Problem**: Backend state being reset/not persisting
**Location**: timeline.rs `add_clip_to_timeline` command
**Fix**: Check project state persistence

### Scenario D: No Errors, But Clips Disappear

**Problem**: Store update logic overwrites previous clips
**Location**: timeline.ts line 44-55 (update method)
**Fix**: Ensure using `[...track.clips, timelineClip]`

---

## Success Criteria

✅ **PASS**: All 3 clips visible on timeline track
✅ **PASS**: Debug panel shows "3 clips"
✅ **PASS**: Backend logs show incrementing clip counts
✅ **PASS**: Console logs show all clips in state
✅ **PASS**: Clips positioned sequentially (no overlap)

❌ **FAIL**: Only last clip visible
❌ **FAIL**: Debug shows 3, UI shows 1
❌ **FAIL**: Backend shows 1 clip always

---

## Quick Test Command

For rapid iteration:

1. Start app: `npm run tauri:dev`
2. Follow steps 1-4
3. Take screenshot when bug occurs
4. Check all logs above
5. Report findings with: debug panel output + console logs + backend logs

---

## Automated Test (Future)

Once the bug is fixed, this manual test should be automated using:

- WebDriver for Tauri
- Or Playwright for visual testing
- See `/tests/e2e/README.md` for setup

---

**Test Created**: October 28, 2025
**Purpose**: Verify Phase 4 multiple clips functionality
**Issue**: Multiple videos not showing on track
