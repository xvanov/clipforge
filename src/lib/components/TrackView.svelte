<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import TimelineClipView from './TimelineClipView.svelte';
  import { mediaLibrary } from '$lib/stores/media-library';
  import type { Track } from '../types/timeline';

  export let track: Track;
  export let pixelsPerSecond: number = 50;
  export let currentTime: number = 0;

  const dispatch = createEventDispatcher();

  let trackElement: HTMLDivElement;

  function handleClipDragStart(_event: CustomEvent) {
    // Clip drag started
  }

  function handleClipDragEnd() {
    // Clip drag ended
  }

  function handleClipMoved(event: CustomEvent) {
    const { clipId, newStartTime } = event.detail;

    dispatch('clip-moved', {
      clipId,
      newStartTime,
      trackId: track.id,
    });
  }

  function handleClipTrimmed(event: CustomEvent) {
    dispatch('clip-trimmed', event.detail);
  }

  function handleClipSplit(event: CustomEvent) {
    dispatch('clip-split', event.detail);
  }

  function handleClipDeleted(event: CustomEvent) {
    dispatch('clip-deleted', event.detail);
  }

  // Handle drag and drop from media library
  let isHoveringTrack = false;

  function handleTrackDragOver(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    isHoveringTrack = true;
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  function handleTrackDragLeave(_event: DragEvent) {
    isHoveringTrack = false;
  }

  function handleTrackDrop(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    isHoveringTrack = false;

    // Bubble up to parent Timeline component by dispatching custom event
    dispatch('track-drop', { event, track });
  }
</script>

<div
  class="track-view"
  bind:this={trackElement}
  class:dropping={isHoveringTrack}
  on:dragover={handleTrackDragOver}
  on:dragleave={handleTrackDragLeave}
  on:drop={handleTrackDrop}
  role="region"
  aria-label="Track: {track.name}"
>
  <div class="track-header">
    <div class="track-name">{track.name}</div>
    <div class="track-controls">
      <!-- TODO: Wire up visibility toggle to update track.visible via store -->
      <button
        class="track-visibility"
        class:visible={track.visible}
        title={track.visible ? 'Hide track' : 'Show track'}
        on:click={() => console.warn('Track visibility toggle not yet implemented')}
      >
        {track.visible ? '👁️' : '👁️‍🗨️'}
      </button>
      <!-- TODO: Wire up lock toggle to update track.locked via store -->
      <button
        class="track-lock"
        class:locked={track.locked}
        title={track.locked ? 'Unlock track' : 'Lock track'}
        on:click={() => console.warn('Track lock toggle not yet implemented')}
      >
        {track.locked ? '🔒' : '🔓'}
      </button>
    </div>
  </div>

  <div class="track-content">
    <div class="track-clips">
      {#each track.clips as timelineClip (timelineClip.id)}
        {@const mediaClip = $mediaLibrary.find((mc) => mc.id === timelineClip.media_clip_id)}
        <TimelineClipView
          clip={timelineClip}
          {pixelsPerSecond}
          {currentTime}
          locked={track.locked}
          mediaDuration={mediaClip?.duration}
          on:drag-start={handleClipDragStart}
          on:moved={handleClipMoved}
          on:drag-end={handleClipDragEnd}
          on:trimmed={handleClipTrimmed}
          on:split={handleClipSplit}
          on:deleted={handleClipDeleted}
        />
      {/each}
    </div>
  </div>
</div>

<style>
  .track-view {
    min-height: 80px;
    border-bottom: 1px solid #333;
    display: flex;
    background: #0a0a0a;
  }

  .track-view.dropping {
    background: rgba(0, 200, 100, 0.1);
    border: 2px solid #00c864;
  }

  .track-header {
    width: 150px;
    background: #1a1a1a;
    border-right: 1px solid #333;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .track-name {
    font-size: 14px;
    font-weight: 500;
    color: #fff;
  }

  .track-controls {
    display: flex;
    gap: 4px;
  }

  .track-controls button {
    width: 28px;
    height: 28px;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .track-controls button:hover {
    background: #3a3a3a;
  }

  .track-visibility.visible {
    background: #2a4a2a;
  }

  .track-lock.locked {
    background: #4a2a2a;
  }

  .track-content {
    flex: 1;
    position: relative;
    overflow: visible;
    min-height: 80px;
  }

  .track-clips {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 80px;
  }
</style>
