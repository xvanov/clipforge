<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import MediaLibrary from '$lib/components/MediaLibrary.svelte';
  import VideoPreview from '$lib/components/VideoPreview.svelte';
  import Timeline from '$lib/components/Timeline.svelte';
  import ExportDialog from '$lib/components/ExportDialog.svelte';
  import RecordingControls from '$lib/components/RecordingControls.svelte';
  import CaptionsPanel from '$lib/components/CaptionsPanel.svelte';
  import { timelineStore, tracks } from '$lib/stores/timeline';
  import { mediaLibrary } from '$lib/stores/media-library';
  import type { MediaClip } from '$lib/types/clip';

  let videoCurrentTime = 0;
  let timelineDuration = 0;
  let isTimelinePlaying = false;
  let currentClipForPreview: MediaClip | null = null;
  let currentTimelineClipId: string | null = null;
  let currentClipStartTime: number = 0;
  let currentClipInPoint: number = 0;
  let currentClipOutPoint: number = 0;
  let showExportDialog = false;
  let showDebugPanel = false; // Global debug toggle
  let showViewMenu = false; // View menu dropdown
  let sidebarTab: 'media' | 'recording' | 'captions' = 'media'; // Sidebar tab selection

  // Subscribe to tracks to find current clip
  $: if ($tracks.length > 0 && $tracks[0].clips.length > 0) {
    // Find the clip at current playhead position
    const currentClip = $tracks[0].clips.find((clip) => {
      const clipStart = clip.start_time;
      const clipEnd = clip.start_time + (clip.out_point - clip.in_point);
      return videoCurrentTime >= clipStart && videoCurrentTime < clipEnd;
    });

    if (currentClip) {
      // Only update if clip changed
      if (currentClip.id !== currentTimelineClipId) {
        // Find the MediaClip from media library
        const mediaClip = $mediaLibrary.find((mc) => mc.id === currentClip.media_clip_id);
        if (mediaClip) {
          currentClipForPreview = mediaClip;
          currentTimelineClipId = currentClip.id;
          currentClipStartTime = currentClip.start_time;
          currentClipInPoint = currentClip.in_point;
          currentClipOutPoint = currentClip.out_point;
          console.log(
            `Switching to clip ${mediaClip.name} at timeline ${videoCurrentTime.toFixed(2)}s`
          );
        }
      }
    }
  }

  // Initialize default track on mount
  onMount(async () => {
    try {
      console.log('App.svelte: Creating default track...');
      const track = await timelineStore.createTrack('Main Track', 'main');
      console.log('App.svelte: Track created successfully:', track);
    } catch (error) {
      console.error('Failed to create default track:', error);
    }

    // Add global keyboard handler
    window.addEventListener('keydown', handleGlobalKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleGlobalKeyDown);
  });

  function handleGlobalKeyDown(event: KeyboardEvent) {
    // Toggle debug panel with Cmd+Shift+D (Mac) or Ctrl+Shift+D (Windows/Linux)
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === 'D') {
      event.preventDefault();
      showDebugPanel = !showDebugPanel;
      console.log('Debug panel toggled:', showDebugPanel);
    }
  }

  function handleTimeUpdate(event: CustomEvent<{ time: number }>) {
    // Update timeline when video plays
    if (isTimelinePlaying) {
      // Map video time to timeline time
      // Video plays from in_point to out_point, mapped to timeline start_time
      const videoTime = event.detail.time;
      const relativeVideoTime = videoTime - currentClipInPoint;
      videoCurrentTime = currentClipStartTime + relativeVideoTime;

      // Check if we've reached the end of this clip
      const clipEnd = currentClipStartTime + (currentClipOutPoint - currentClipInPoint);
      if (videoCurrentTime >= clipEnd - 0.05) {
        // Small buffer to catch the end
        // Try to move to next clip
        const sortedClips = [...$tracks[0].clips].sort((a, b) => a.start_time - b.start_time);
        const currentIndex = sortedClips.findIndex((c) => c.id === currentTimelineClipId);
        if (currentIndex >= 0 && currentIndex < sortedClips.length - 1) {
          // Move to next clip
          const nextClip = sortedClips[currentIndex + 1];
          videoCurrentTime = nextClip.start_time;
        } else {
          // No more clips, stop playback
          isTimelinePlaying = false;
        }
      }
    } else {
      // When not playing timeline, sync playhead position with video scrubbing
      const videoTime = event.detail.time;
      const relativeVideoTime = videoTime - currentClipInPoint;
      videoCurrentTime = currentClipStartTime + relativeVideoTime;
    }
  }

  function handlePlayPause(playing: boolean) {
    isTimelinePlaying = playing;
  }

  function handleExportClick() {
    // Check if there are clips on the timeline
    if ($tracks.length === 0 || $tracks.every((track) => track.clips.length === 0)) {
      alert('Please add clips to the timeline before exporting');
      return;
    }
    showExportDialog = true;
  }

  function handleCloseExportDialog() {
    showExportDialog = false;
  }

  function toggleDebugPanel() {
    showDebugPanel = !showDebugPanel;
    showViewMenu = false; // Close menu after selection
  }

  function toggleViewMenu() {
    showViewMenu = !showViewMenu;
  }

  // Close menus when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.menu-container')) {
      showViewMenu = false;
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<main class="app" on:click={handleClickOutside}>
  <header class="app-header">
    <div class="logo">
      <h1>ClipForge</h1>
      <span class="version">v0.1.0</span>
    </div>

    <nav class="menubar">
      <!-- View Menu -->
      <div class="menu-container">
        <button class="menu-button" on:click|stopPropagation={toggleViewMenu}>
          View
          <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor" style="margin-left: 4px;">
            <path d="M0 2L4 6L8 2H0Z" />
          </svg>
        </button>
        {#if showViewMenu}
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div class="menu-dropdown" on:click|stopPropagation>
            <button class="menu-item" on:click={toggleDebugPanel}>
              <span class="menu-check">{showDebugPanel ? '✓' : ''}</span>
              Debug Panel
              <span class="menu-shortcut">⌘⇧D</span>
            </button>
          </div>
        {/if}
      </div>
    </nav>

    <div class="toolbar">
      <button class="export-button" on:click={handleExportClick}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 12L3 7h3V1h4v6h3L8 12z" />
          <path d="M14 14H2v-2h12v2z" />
        </svg>
        Export
      </button>
    </div>
  </header>

  <div class="app-content">
    <aside class="sidebar">
      <div class="sidebar-tabs">
        <button
          class="tab-button"
          class:active={sidebarTab === 'media'}
          on:click={() => (sidebarTab = 'media')}
        >
          Media Library
        </button>
        <button
          class="tab-button"
          class:active={sidebarTab === 'recording'}
          on:click={() => (sidebarTab = 'recording')}
        >
          Recording
        </button>
        <button
          class="tab-button"
          class:active={sidebarTab === 'captions'}
          on:click={() => (sidebarTab = 'captions')}
        >
          AI Captions
        </button>
      </div>

      <div class="sidebar-content">
        {#if sidebarTab === 'media'}
          <MediaLibrary />
        {:else if sidebarTab === 'recording'}
          <RecordingControls />
        {:else if sidebarTab === 'captions'}
          {#if currentClipForPreview}
            <CaptionsPanel
              clipId={currentClipForPreview.id}
              captions={currentClipForPreview.captions || []}
            />
          {:else}
            <div class="no-clip-message">
              <p>No clip selected</p>
              <p class="hint">Add a clip to the timeline to generate captions</p>
            </div>
          {/if}
        {/if}
      </div>
    </aside>

    <section class="main-area">
      <div class="preview-section">
        <VideoPreview
          currentClip={currentClipForPreview}
          currentTime={videoCurrentTime}
          clipStartTime={currentClipStartTime}
          clipInPoint={currentClipInPoint}
          clipOutPoint={currentClipOutPoint}
          on:timeupdate={handleTimeUpdate}
          on:playpause={(e) => handlePlayPause(e.detail.playing)}
        />
      </div>

      <div class="timeline-section">
        <Timeline
          bind:currentTime={videoCurrentTime}
          bind:duration={timelineDuration}
          bind:showDebug={showDebugPanel}
        />
      </div>
    </section>
  </div>

  <!-- Export Dialog -->
  <ExportDialog visible={showExportDialog} onClose={handleCloseExportDialog} />
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: #1a1a1a;
    color: #fff;
    overflow: hidden;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: #2a2a2a;
    border-bottom: 1px solid #333;
    height: 50px;
    flex-shrink: 0;
  }

  .logo {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  .logo h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: #0078d4;
  }

  .version {
    font-size: 0.75rem;
    color: #666;
  }

  .menubar {
    display: flex;
    gap: 0.25rem;
    margin-left: 1rem;
  }

  .menu-container {
    position: relative;
  }

  .menu-button {
    display: flex;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background: transparent;
    color: #ccc;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    cursor: pointer;
    transition:
      background 0.2s,
      color 0.2s;
  }

  .menu-button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .menu-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 0.25rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    min-width: 200px;
    z-index: 1000;
    padding: 0.25rem 0;
  }

  .menu-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 0.5rem 1rem;
    background: transparent;
    color: #ccc;
    border: none;
    text-align: left;
    font-size: 0.875rem;
    cursor: pointer;
    transition:
      background 0.2s,
      color 0.2s;
  }

  .menu-item:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .menu-check {
    width: 20px;
    margin-right: 8px;
    color: #0078d4;
    font-weight: bold;
  }

  .menu-shortcut {
    margin-left: auto;
    font-size: 0.75rem;
    color: #888;
    padding-left: 1rem;
  }

  .toolbar {
    display: flex;
    gap: 0.5rem;
  }

  .export-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: #0078d4;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }

  .export-button:hover {
    background: #005a9e;
  }

  .export-button svg {
    flex-shrink: 0;
  }

  .app-content {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    width: 320px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    border-right: 1px solid #333;
  }

  .sidebar-tabs {
    display: flex;
    border-bottom: 1px solid #333;
    background: #252525;
  }

  .tab-button {
    flex: 1;
    padding: 0.75rem 1rem;
    background: transparent;
    color: #888;
    border: none;
    border-bottom: 2px solid transparent;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tab-button:hover {
    color: #ccc;
    background: rgba(255, 255, 255, 0.05);
  }

  .tab-button.active {
    color: #0078d4;
    border-bottom-color: #0078d4;
    background: #1e1e1e;
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .no-clip-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #888;
    text-align: center;
    padding: 2rem;
  }

  .no-clip-message p {
    margin: 0.5rem 0;
  }

  .no-clip-message .hint {
    font-size: 0.875rem;
    color: #666;
  }

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .preview-section {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .timeline-section {
    height: 250px;
    flex-shrink: 0;
    background: #0a0a0a;
    border-top: 1px solid #333;
  }
</style>
