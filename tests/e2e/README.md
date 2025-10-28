# E2E Tests for ClipForge

## Current Status

**Manual Testing**: ✅ Available  
**Automated E2E**: 🚧 Not yet implemented

## Manual Testing

For now, use the manual test guide:

```bash
# Start the app in dev mode
npm run tauri:dev

# Follow the manual test guide
cat tests/e2e/manual-timeline-test.md
```

The manual test guide provides:

- Step-by-step instructions
- Expected results at each step
- Debug verification points
- Bug diagnosis flowchart

## Future: Automated E2E Tests

### Option 1: WebDriver (Recommended for Tauri)

Tauri supports WebDriver testing for true E2E tests.

**Setup**:

```bash
# Install dependencies
npm install --save-dev @wdio/cli @wdio/local-runner @wdio/mocha-framework

# Configure WebDriver
npx wdio config
```

**Example Test Structure**:

```typescript
// tests/e2e/timeline-multi-clip.spec.ts
import { remote } from 'webdriverio';

describe('Timeline: Multiple Clips', () => {
  let browser;

  before(async () => {
    browser = await remote({
      capabilities: {
        'tauri:options': {
          application: './src-tauri/target/release/clipforge',
        },
      },
    });
  });

  it('should add multiple videos to timeline track', async () => {
    // Wait for app to load
    await browser.waitForExist('.media-library', 5000);

    // Import videos
    // ... test implementation
  });

  after(async () => {
    await browser.deleteSession();
  });
});
```

### Option 2: Playwright (Visual Testing)

Playwright can be configured to work with Tauri apps.

**Setup**:

```bash
npm install --save-dev @playwright/test
npx playwright install
```

**Example Test**:

```typescript
// tests/e2e/timeline-visual.spec.ts
import { test, expect } from '@playwright/test';

test('multiple clips appear on timeline', async ({ page }) => {
  // Launch Tauri app
  // ... configuration needed

  // Verify clips appear
  const clips = await page.locator('.timeline-clip');
  await expect(clips).toHaveCount(3);
});
```

### Option 3: Testing Library (Component E2E)

For component-level E2E without full app:

```bash
npm install --save-dev @testing-library/svelte @testing-library/jest-dom
```

## Setting Up Automated E2E (Future Task)

1. **Choose Framework**: WebDriver (recommended) or Playwright
2. **Install Dependencies**: See options above
3. **Configure**: Create `wdio.conf.js` or `playwright.config.ts`
4. **Write Tests**: Convert manual tests to automated
5. **Add to CI**: Run in GitHub Actions

## Running Tests

### Manual Tests

```bash
npm run tauri:dev
# Follow manual-timeline-test.md
```

### Automated Tests (when implemented)

```bash
# WebDriver
npm run test:e2e

# Playwright
npx playwright test
```

## Test Files

- `manual-timeline-test.md` - Manual test guide for multiple clips
- `timeline-multi-clip.spec.ts` - (Future) Automated WebDriver test
- `timeline-visual.spec.ts` - (Future) Playwright visual test

## Resources

- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/webdriver/introduction)
- [WebDriver for Desktop Apps](https://webdriver.io/)
- [Playwright for Desktop](https://playwright.dev/)

---

**Status**: Manual testing available, automated E2E planned for later phase
**Last Updated**: October 28, 2025

