<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import TimelineClipView from './TimelineClipView.svelte';
  import type { Track } from '../types/timeline';
  
  export let track: Track;
  export let pixelsPerSecond: number = 50;
  export let scrollLeft: number = 0;
  export let currentTime: number = 0;
  
  const dispatch = createEventDispatcher();
  
  let trackElement: HTMLDivElement;
  let isDraggingClip: boolean = false;
  let draggedClipId: string | null = null;
  
  function handleClipDragStart(event: CustomEvent) {
    isDraggingClip = true;
    draggedClipId = event.detail.clipId;
  }
  
  function handleClipDragEnd() {
    isDraggingClip = false;
    draggedClipId = null;
  }
  
  function handleClipDrag(event: CustomEvent) {
    if (!isDraggingClip || !draggedClipId) return;
    
    const newStartTime = event.detail.startTime;
    dispatch('clip-moved', {
      clipId: draggedClipId,
      newStartTime,
      trackId: track.id
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
  
  // Debug logging
  $: {
    console.log(`TrackView for track ${track.name} has ${track.clips.length} clips:`, track.clips);
  }
</script>

<div class="track-view" bind:this={trackElement}>
  <div class="track-header">
    <div class="track-name">{track.name}</div>
    <div class="track-controls">
      <button 
        class="track-visibility" 
        class:visible={track.visible}
        title={track.visible ? 'Hide track' : 'Show track'}
      >
        {track.visible ? '👁️' : '👁️‍🗨️'}
      </button>
      <button 
        class="track-lock" 
        class:locked={track.locked}
        title={track.locked ? 'Unlock track' : 'Lock track'}
      >
        {track.locked ? '🔒' : '🔓'}
      </button>
    </div>
  </div>
  
  <div class="track-content">
    <div class="track-clips" style="position: relative; height: 100%;">
      {#each track.clips as clip (clip.id)}
        <TimelineClipView
          {clip}
          {pixelsPerSecond}
          {scrollLeft}
          {currentTime}
          locked={track.locked}
          on:drag-start={handleClipDragStart}
          on:drag={handleClipDrag}
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
    overflow: hidden;
  }
  
  .track-clips {
    width: 100%;
    height: 100%;
  }
</style>

