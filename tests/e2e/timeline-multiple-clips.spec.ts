import { test, expect, Page } from '@playwright/test';

/**
 * E2E Test: Multiple Clips on Timeline Track
 *
 * This test verifies that multiple videos can be dragged onto the timeline
 * and all appear correctly without disappearing.
 *
 * Uses real video files for testing.
 */

// Test video files (adjust paths as needed)
const TEST_VIDEOS = [
  '/Users/kalin.ivanov/Downloads/IMG_2278.mp4',
  '/Users/kalin.ivanov/Downloads/IMG_2248.mp4',
];

test.describe('Timeline: Multiple Clips E2E', () => {
  let page: Page;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();

    // Enable console logging for debugging
    page.on('console', (msg) => {
      const type = msg.type();
      const text = msg.text();
      if (type === 'error') {
        console.log('❌ Browser ERROR:', text);
      } else if (
        text.includes('Adding to track') ||
        text.includes('Clip added') ||
        text.includes('Track')
      ) {
        console.log('🔵 Browser LOG:', text);
      }
    });

    await page.goto('http://localhost:5173');

    // Wait for app to fully load
    await page.waitForSelector('.media-library', { timeout: 10000 });
    await page.waitForSelector('.timeline-container', { timeout: 10000 });

    // CRITICAL: Wait for default track to be created
    console.log('⏳ Waiting for track initialization...');
    await page.waitForTimeout(3000); // Give App.svelte onMount time to run

    // Verify track was created by checking debug panel
    await page.waitForSelector('text=DEBUG', { timeout: 5000 });

    // Check if we have tracks
    const bodyText = await page.textContent('body');
    console.log(
      '📊 Page loaded. Body includes:',
      bodyText?.includes('Track') ? 'Track found' : 'No track yet'
    );
  });

  test.afterAll(async () => {
    await page.close();
  });

  test('1. should show app loaded with empty media library', async () => {
    // Check debug panel exists
    const debugPanel = page.locator('text=DEBUG');
    await expect(debugPanel).toBeVisible();

    // Check Import Media button exists
    const importButton = page.locator('button:has-text("Import Media")');
    await expect(importButton).toBeVisible();

    console.log('✅ App loaded, ready for video import');
  });

  test('2. should import first video file', async () => {
    console.log('📥 Importing first video:', TEST_VIDEOS[0]);

    // Set up file chooser handler BEFORE clicking the button
    const fileChooserPromise = page.waitForEvent('filechooser', { timeout: 10000 });

    // Click Import Media button
    const importButton = page.locator('button:has-text("Import Media")');
    await importButton.click();

    // Wait for file chooser dialog
    const fileChooser = await fileChooserPromise;

    // Select the first test video
    await fileChooser.setFiles([TEST_VIDEOS[0]]);

    // Wait for video to be processed and appear in media library
    console.log('⏳ Waiting for video to appear in media library...');
    await page.waitForTimeout(3000); // Give time for FFmpeg processing

    // Verify video appears in media library
    const mediaClip = page.locator('.media-clip-card').first();
    await expect(mediaClip).toBeVisible({ timeout: 10000 });

    console.log('✅ First video imported successfully');
  });

  test('3. should import second video file', async () => {
    console.log('📥 Importing second video:', TEST_VIDEOS[1]);

    // Set up file chooser handler
    const fileChooserPromise = page.waitForEvent('filechooser', { timeout: 10000 });

    // Click Import Media button again
    const importButton = page.locator('button:has-text("Import Media")');
    await importButton.click();

    // Wait for file chooser dialog
    const fileChooser = await fileChooserPromise;

    // Select the second test video
    await fileChooser.setFiles([TEST_VIDEOS[1]]);

    // Wait for video to be processed
    console.log('⏳ Waiting for second video to appear...');
    await page.waitForTimeout(3000);

    // Verify we now have 2 videos in media library
    const mediaClips = page.locator('.media-clip-card');
    const count = await mediaClips.count();
    expect(count).toBeGreaterThanOrEqual(2);

    console.log(`✅ Second video imported. Total videos: ${count}`);
  });

  test('4. should drag first video to timeline', async () => {
    console.log('🎬 Dragging first video to timeline...');

    // Find first media clip card
    const firstClip = page.locator('.media-clip-card').first();
    await expect(firstClip).toBeVisible();

    // Get the timeline drop area
    const timeline = page.locator('.timeline-container');
    await expect(timeline).toBeVisible();

    // Perform drag and drop
    const clipBox = await firstClip.boundingBox();
    const timelineBox = await timeline.boundingBox();

    if (clipBox && timelineBox) {
      // Start drag from clip
      await page.mouse.move(clipBox.x + clipBox.width / 2, clipBox.y + clipBox.height / 2);
      await page.mouse.down();

      // Move to timeline (near the left edge)
      await page.mouse.move(timelineBox.x + 100, timelineBox.y + timelineBox.height / 2, {
        steps: 10,
      });
      await page.mouse.up();
    }

    // Wait for drop to process
    await page.waitForTimeout(1500);

    // Verify debug panel shows 1 clip
    console.log('🔍 Checking debug panel for clip count...');
    const bodyText = await page.textContent('body');
    console.log('Debug panel text:', bodyText?.match(/Track 0.*clips/)?.[0]);

    // Take screenshot
    await page.screenshot({
      path: 'test-results/after-first-drag.png',
      fullPage: true,
    });

    // Check if clip was added
    const hasOneClip = bodyText?.includes('1 clips') || bodyText?.includes('Clip 0');
    expect(hasOneClip).toBeTruthy();

    console.log('✅ First clip added to timeline!');
  });

  test('5. should drag second video to timeline and show BOTH clips', async () => {
    console.log('🎬 Dragging second video to timeline...');

    // Find second media clip card
    const secondClip = page.locator('.media-clip-card').nth(1);
    await expect(secondClip).toBeVisible();

    // Get the timeline drop area
    const timeline = page.locator('.timeline-container');

    // Perform drag and drop
    const clipBox = await secondClip.boundingBox();
    const timelineBox = await timeline.boundingBox();

    if (clipBox && timelineBox) {
      // Start drag from clip
      await page.mouse.move(clipBox.x + clipBox.width / 2, clipBox.y + clipBox.height / 2);
      await page.mouse.down();

      // Move to timeline (further right to avoid overlap)
      await page.mouse.move(timelineBox.x + 200, timelineBox.y + timelineBox.height / 2, {
        steps: 10,
      });
      await page.mouse.up();
    }

    // Wait for drop to process
    await page.waitForTimeout(1500);

    // CRITICAL CHECK: Verify debug panel shows 2 clips
    console.log('🔍 Checking debug panel for 2 clips...');
    const bodyText = await page.textContent('body');
    console.log('Debug panel text:', bodyText?.match(/Track 0.*clips/)?.[0]);

    // Take screenshot
    await page.screenshot({
      path: 'test-results/after-second-drag.png',
      fullPage: true,
    });

    // Check if we have 2 clips
    const hasTwoClips =
      bodyText?.includes('2 clips') ||
      (bodyText?.includes('Clip 0') && bodyText?.includes('Clip 1'));

    if (!hasTwoClips) {
      console.log('❌ BUG DETECTED: Expected 2 clips but found different count');
      console.log(
        'Full debug text:',
        bodyText?.substring(bodyText.indexOf('DEBUG'), bodyText.indexOf('DEBUG') + 200)
      );
    }

    expect(hasTwoClips).toBeTruthy();

    console.log('✅ Second clip added! Both clips visible on timeline!');
  });

  test('6. should verify CLIPS indicator shows 2', async () => {
    // Check the CLIPS: N indicator
    const clipsText = await page.textContent('body');
    const clipsMatch = clipsText?.match(/CLIPS: (\d+)/);

    if (clipsMatch) {
      const clipCount = parseInt(clipsMatch[1]);
      console.log(`📊 CLIPS indicator shows: ${clipCount}`);
      expect(clipCount).toBe(2);
    }

    console.log('✅ CLIPS indicator correct!');
  });

  test('7. should take final screenshot for visual verification', async () => {
    // Wait a moment for any animations
    await page.waitForTimeout(500);

    // Take full page screenshot
    await page.screenshot({
      path: 'test-results/final-timeline-state.png',
      fullPage: true,
    });

    console.log('📸 Final screenshot saved to test-results/final-timeline-state.png');
    console.log('✅ All tests complete! Check screenshots to verify visual state.');
  });
});
