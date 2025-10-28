<script lang="ts">
  import { onMount } from 'svelte';
  import MediaLibrary from '$lib/components/MediaLibrary.svelte';
  import VideoPreview from '$lib/components/VideoPreview.svelte';
  import Timeline from '$lib/components/Timeline.svelte';
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
  });

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
