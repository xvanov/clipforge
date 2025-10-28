# 🚨 ISSUE FOUND: Tests Running Against Web, Not Desktop

## The Problem

The E2E tests are currently running against:

```
http://localhost:5173/  ← Vite dev server (WEB MODE)
```

But they need to run against:

```
The actual Tauri desktop app  ← Where Tauri APIs work
```

**Error**: `window.__TAURI_IPC__ is not a function`  
**Reason**: Tauri APIs (`@tauri-apps/api`) don't work in web browsers  
**Also**: Debug shows "Tracks: 0" - track creation also uses Tauri

## Why This Happens

```
npm run dev          → Starts Vite (web browser mode)
npm run tauri:dev    → Starts Tauri (desktop app with APIs)
```

File import uses:

```typescript
import { open } from '@tauri-apps/api/dialog';  ← Only works in Tauri!
```

## Solutions

### Option 1: Test Desktop App (Proper E2E)

We need to use **Tauri WebDriver** to test the actual desktop app.

**Setup** (complex but proper):

```bash
# Install Tauri WebDriver
npm install --save-dev tauri-driver

# Install WebDriver
npm install --save-dev webdriverio @wdio/cli
```

**Configure WebDriver** to connect to Tauri app instead of browser.

**Pros**: Tests real desktop app  
**Cons**: More complex setup

### Option 2: Manual Testing (Current Best Option)

Since import works in desktop app, use the **manual test guide**:

```bash
# Run the desktop app
npm run tauri:dev

# Follow manual test guide
cat tests/e2e/manual-timeline-test.md
```

**Steps**:

1. Import 2-3 videos manually
2. Drag them to timeline
3. Watch debug panel
4. Verify clip counts

**Pros**: Works right now, tests real app  
**Cons**: Manual (but only takes 2 minutes)

### Option 3: Mock Tauri APIs for Web Tests

Create mocks so tests can run in browser:

```typescript
// Mock file import for web testing
if (!window.__TAURI__) {
  window.__TAURI__ = {
    // Mock implementations
  };
}
```

**Pros**: Tests run automatically  
**Cons**: Not testing real Tauri functionality

## Recommended: Manual Testing

Until we set up proper Tauri WebDriver, use **manual testing**:

### Quick Manual Test (2 minutes)

```bash
# 1. Start desktop app
npm run tauri:dev

# 2. Import 2-3 videos
#    Click "Import Media" → Select videos

# 3. Drag first video to timeline
#    → Check: Debug shows "1 clips"

# 4. Drag second video to timeline
#    → Check: Debug shows "2 clips" ← BUG CHECK!

# 5. Look at debug panel
#    If shows "1 clips" → BUG CONFIRMED
#    If shows "2 clips" → IT WORKS!
```

## Why Desktop App Works But Web Doesn't

| Feature            | Web (`npm run dev`) | Desktop (`npm run tauri:dev`) |
| ------------------ | ------------------- | ----------------------------- |
| File Import        | ❌ No Tauri APIs    | ✅ Tauri APIs work            |
| Track Creation     | ❌ Needs Tauri      | ✅ Works                      |
| Drag & Drop        | ✅ Works            | ✅ Works                      |
| Timeline Rendering | ✅ Works            | ✅ Works                      |

## Current State

### Unit Tests: ✅ PASS

```bash
npm test  # 33 tests pass
```

### E2E Tests: ❌ FAIL (wrong environment)

```bash
npm run test:e2e  # Fails - testing web instead of desktop
```

### Manual Desktop Test: ✅ WORKS

```bash
npm run tauri:dev  # Use this for actual testing
```

## Next Steps

### For You Right Now:

**Use manual testing**:

```bash
npm run tauri:dev
# Import videos, drag to timeline, check debug panel
```

This will show you if the bug exists in the actual desktop app.

### For Future (Proper E2E):

Set up Tauri WebDriver to test the actual desktop app. See:

- [Tauri WebDriver Guide](https://tauri.app/v1/guides/testing/webdriver/introduction)

## Files to Reference

- **Manual Test Guide**: `tests/e2e/manual-timeline-test.md`
- **Quick Start**: `QUICK_START_TESTING.md`

---

**TLDR**: E2E tests run against web (no Tauri APIs). Use `npm run tauri:dev` + manual testing until we set up Tauri WebDriver.

**Recommended Action**: Manual test in desktop app (takes 2 minutes, shows real bug status)

