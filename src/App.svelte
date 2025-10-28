<script lang="ts">
  import MediaLibrary from '$lib/components/MediaLibrary.svelte';
  import VideoPreview from '$lib/components/VideoPreview.svelte';
  import Timeline from '$lib/components/Timeline.svelte';

  // TODO: T074 - Add Export button
  // TODO: T090 - Add Recording panel (Phase 6)
</script>

<main class="app">
  <header class="app__header">
    <div class="app__logo">
      <h1>🎬 ClipForge</h1>
    </div>
    
    <div class="app__toolbar">
      <button class="btn btn--primary" disabled>New Project</button>
      <button class="btn" disabled>Open</button>
      <button class="btn" disabled>Save</button>
      <div class="toolbar__spacer" />
      <button class="btn btn--primary" disabled>Export</button>
    </div>
  </header>

  <div class="app__body">
    <aside class="app__sidebar">
      <MediaLibrary />
    </aside>

    <div class="app__main">
      <div class="app__preview">
        <VideoPreview />
      </div>
      
      <div class="app__timeline">
        <Timeline />
      </div>
    </div>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: #1e1e1e;
    color: #fff;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .app__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: #2a2a2a;
    border-bottom: 1px solid #333;
  }

  .app__logo h1 {
    margin: 0;
    font-size: 1.3rem;
    font-weight: 600;
  }

  .app__toolbar {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .toolbar__spacer {
    width: 1px;
    height: 24px;
    background: #555;
    margin: 0 0.5rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    background: #333;
    color: #fff;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.2s;
  }

  .btn:hover:not(:disabled) {
    background: #444;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn--primary {
    background: #0078d4;
    border-color: #0078d4;
  }

  .btn--primary:hover:not(:disabled) {
    background: #106ebe;
  }

  .app__body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .app__sidebar {
    width: 280px;
    border-right: 1px solid #333;
  }

  .app__main {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .app__preview {
    flex: 1;
    min-height: 0;
  }

  .app__timeline {
    height: 300px;
    border-top: 1px solid #333;
  }
</style>

