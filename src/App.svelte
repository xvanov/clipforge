<script lang="ts">
  import { onMount } from 'svelte';
  import MediaLibrary from '$lib/components/MediaLibrary.svelte';
  import VideoPreview from '$lib/components/VideoPreview.svelte';
  import Timeline from '$lib/components/Timeline.svelte';
  import { timelineStore } from '$lib/stores/timeline';

  let videoCurrentTime = 0;
  let timelineDuration = 0;

  // Initialize default track on mount
  onMount(async () => {
    try {
      console.log('App.svelte: Creating default track...');
      const track = await timelineStore.createTrack('Main Track', 'main');
      console.log('App.svelte: Track created successfully:', track);
    } catch (error) {
      console.error('Failed to create default track:', error);
    }
  });

  function handleTimeUpdate(event: CustomEvent<{ time: number }>) {
    // Only update timeline when user isn't playing video
    videoCurrentTime = event.detail.time;
  }
</script>

<main class="app">
  <header class="app-header">
    <div class="logo">
      <h1>ClipForge</h1>
      <span class="version">v0.1.0</span>
    </div>
    <div class="toolbar">
      <!-- Toolbar items will be added in later phases -->
    </div>
  </header>

  <div class="app-content">
    <aside class="sidebar">
      <MediaLibrary />
    </aside>

    <section class="main-area">
      <div class="preview-section">
        <VideoPreview currentTime={videoCurrentTime} on:timeupdate={handleTimeUpdate} />
      </div>

      <div class="timeline-section">
        <Timeline bind:currentTime={videoCurrentTime} bind:duration={timelineDuration} />
      </div>
    </section>
  </div>
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

  .toolbar {
    display: flex;
    gap: 0.5rem;
  }

  .app-content {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    width: 280px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    border-right: 1px solid #333;
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
