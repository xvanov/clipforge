<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { timelineStore, tracks as tracksStore } from '../stores/timeline';
  import TrackView from './TrackView.svelte';
  import type { Track } from '../types/timeline';
  import type { MediaClip } from '../types/clip';

  export let currentTime: number = 0;
  export let duration: number = 0;

  let timelineContainer: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let pixelsPerSecond: number = 50; // Zoom level
  let scrollLeft: number = 0;
  let isDraggingPlayhead: boolean = false;
  let isDraggingOverTimeline: boolean = false;

  // Subscribe to timeline store - use derived store for better reactivity
  let tracks: Track[] = [];
  $: tracks = $tracksStore;

  // Calculate total timeline duration from all clips
  $: totalDuration =
    Math.max(
      duration,
      ...tracks.flatMap((track) =>
        track.clips.map((clip) => clip.start_time + (clip.out_point - clip.in_point))
      )
    ) || 60; // Minimum 60 seconds

  // Calculate minimum width needed to show all content
  $: timelineContentWidth = Math.max(
    totalDuration * pixelsPerSecond,
    2000 // Minimum width of 2000px
  );

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
      resizeCanvas();
      window.addEventListener('resize', resizeCanvas);
    }
  });

  onDestroy(() => {
    window.removeEventListener('resize', resizeCanvas);
  });

  function resizeCanvas() {
    if (canvas && timelineContainer) {
      canvas.width = timelineContainer.clientWidth;
      canvas.height = timelineContainer.clientHeight;
      drawTimeline();
    }
  }

  function drawTimeline() {
    if (!ctx || !canvas) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw timeline background
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw time markers
    drawTimeMarkers();

    // Draw playhead
    drawPlayhead();
  }

  function drawTimeMarkers() {
    if (!ctx || !canvas) return;

    ctx.strokeStyle = '#444';
    ctx.fillStyle = '#999';
    ctx.font = '10px sans-serif';

    const startTime = scrollLeft / pixelsPerSecond;
    const endTime = (scrollLeft + canvas.width) / pixelsPerSecond;

    // Draw major markers every 5 seconds
    const majorInterval = 5;
    const minorInterval = 1;

    for (let t = Math.floor(startTime); t <= Math.ceil(endTime); t += minorInterval) {
      const x = t * pixelsPerSecond - scrollLeft;

      if (t % majorInterval === 0) {
        // Major marker
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, 20);
        ctx.stroke();

        // Time label
        const minutes = Math.floor(t / 60);
        const seconds = t % 60;
        ctx.fillText(`${minutes}:${seconds.toString().padStart(2, '0')}`, x + 2, 15);
      } else {
        // Minor marker
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, 10);
        ctx.stroke();
      }
    }
  }

  function drawPlayhead() {
    if (!ctx || !canvas) return;

    const x = currentTime * pixelsPerSecond - scrollLeft;

    // Draw playhead line
    ctx.strokeStyle = '#ff4444';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, canvas.height);
    ctx.stroke();

    // Draw playhead handle
    ctx.fillStyle = '#ff4444';
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x - 8, -10);
    ctx.lineTo(x + 8, -10);
    ctx.closePath();
    ctx.fill();
  }

  function handleCanvasClick(event: MouseEvent) {
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left + scrollLeft;
    const time = x / pixelsPerSecond;

    // Update current time (seek)
    currentTime = Math.max(0, Math.min(time, duration));
    drawTimeline();
  }

  function handleCanvasMouseDown(event: MouseEvent) {
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const playheadX = currentTime * pixelsPerSecond - scrollLeft;

    // Check if clicking near playhead
    if (Math.abs(x - playheadX) < 10) {
      isDraggingPlayhead = true;
    }
  }

  function handleCanvasMouseMove(event: MouseEvent) {
    if (!isDraggingPlayhead || !canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left + scrollLeft;
    const time = x / pixelsPerSecond;

    currentTime = Math.max(0, Math.min(time, duration));
    drawTimeline();
  }

  function handleCanvasMouseUp() {
    isDraggingPlayhead = false;
  }

  function handleZoomIn() {
    pixelsPerSecond = Math.min(pixelsPerSecond * 1.5, 200);
    drawTimeline();
  }

  function handleZoomOut() {
    pixelsPerSecond = Math.max(pixelsPerSecond / 1.5, 10);
    drawTimeline();
  }

  // Handle drag-and-drop from media library
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    isDraggingOverTimeline = true;
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  function handleDragLeave(event: DragEvent) {
    // Only set isDraggingOverTimeline to false if we're leaving the timeline entirely
    // Don't set it to false when moving between child elements
    const target = event.currentTarget as HTMLElement;
    const relatedTarget = event.relatedTarget as HTMLElement;

    if (!target.contains(relatedTarget)) {
      isDraggingOverTimeline = false;
    }
  }

  async function handleDrop(event: DragEvent | CustomEvent) {
    // Handle both native DragEvent and CustomEvent from TrackView
    let dragEvent: DragEvent;
    let targetTrack: Track | undefined;

    if (event instanceof CustomEvent) {
      dragEvent = event.detail.event;
      targetTrack = event.detail.track;
    } else {
      dragEvent = event;
    }

    dragEvent.preventDefault();
    dragEvent.stopPropagation();
    isDraggingOverTimeline = false;

    if (!dragEvent.dataTransfer) {
      console.error('No dataTransfer object');
      return;
    }

    const clipData = dragEvent.dataTransfer.getData('application/json');

    if (!clipData) {
      console.error('No clip data in dataTransfer');
      return;
    }

    try {
      const mediaClip: MediaClip = JSON.parse(clipData);

      const rect = timelineContainer.getBoundingClientRect();
      const x = dragEvent.clientX - rect.left + scrollLeft;
      let dropTime = x / pixelsPerSecond;

      // Auto-position: place clips sequentially after existing clips
      const targetTrackObj = targetTrack || tracks[0];
      if (targetTrackObj && targetTrackObj.clips.length > 0) {
        // Find the end time of the last clip on this track
        const sortedClips = [...targetTrackObj.clips].sort((a, b) => a.start_time - b.start_time);
        const lastClip = sortedClips[sortedClips.length - 1];
        const lastClipEnd = lastClip.start_time + (lastClip.out_point - lastClip.in_point);
        // Position new clip immediately after the last clip (no gap)
        dropTime = lastClipEnd;
      } else {
        // If track is empty, start at beginning
        dropTime = 0;
      }

      // Add clip to first track (or specified track)
      const trackToUse = targetTrack || tracks[0];
      if (trackToUse) {
        // in_point starts at 0, out_point is the media duration (full clip)
        await timelineStore.addClipToTimeline(
          mediaClip.id,
          trackToUse.id,
          dropTime,
          0, // in_point: start of media
          mediaClip.duration // out_point: end of media (use full clip duration)
        );

        console.log(`Added "${mediaClip.name}" to timeline at ${dropTime.toFixed(2)}s`);
      } else {
        console.error('No tracks available');
      }
    } catch (error) {
      console.error('Failed to drop clip:', error);
    }
  }

  // Redraw timeline when dependencies change
  $: if (ctx && (currentTime || pixelsPerSecond || scrollLeft !== undefined || tracks)) {
    drawTimeline();
  }

  // Handle clip movement (drag to reorder)
  async function handleClipMoved(event: CustomEvent) {
    const { clipId, newStartTime } = event.detail;

    try {
      await timelineStore.updateClip(clipId, {
        startTime: newStartTime,
      });
    } catch (error) {
      console.error('Failed to move clip:', error);
    }
  }

  // Handle clip trimming
  async function handleClipTrimmed(event: CustomEvent) {
    const { clipId, inPoint, outPoint, startTime } = event.detail;

    try {
      const updates: Record<string, number> = {};
      if (inPoint !== undefined) updates.inPoint = inPoint;
      if (outPoint !== undefined) updates.outPoint = outPoint;
      if (startTime !== undefined) updates.startTime = startTime;

      await timelineStore.updateClip(clipId, updates);
    } catch (error) {
      console.error('Failed to trim clip:', error);
    }
  }
</script>

<div
  class="timeline-container"
  role="region"
  aria-label="Timeline editor"
  bind:this={timelineContainer}
>
  <!-- Debug panel -->
  <div
    style="position: absolute; top: 5px; right: 5px; background: rgba(0,0,0,0.9); color: #0f0; padding: 8px; font-family: monospace; font-size: 11px; z-index: 1000; border: 1px solid #0f0; max-width: 300px;"
  >
    <strong>DEBUG</strong><br />
    Tracks: {tracks.length}<br />
    Dragging Over: {isDraggingOverTimeline ? 'YES' : 'NO'}<br />
    {#each tracks as trackItem, i}
      Track {i} "{trackItem.name}": {trackItem.clips.length} clips<br />
      {#each trackItem.clips as clipItem, j}
        &nbsp;&nbsp;Clip {j}: {clipItem.id.substring(0, 6)}... at {clipItem.start_time.toFixed(1)}s
        (dur: {(clipItem.out_point - clipItem.in_point).toFixed(1)}s)<br />
      {/each}
    {/each}
  </div>

  <div class="timeline-header">
    <div class="timeline-controls">
      <button on:click={handleZoomIn} title="Zoom In">+</button>
      <button on:click={handleZoomOut} title="Zoom Out">-</button>
      <span class="zoom-level">{Math.round(pixelsPerSecond)}px/s</span>
      <span style="margin-left: 20px; color: #0f0; font-weight: bold;">
        CLIPS: {tracks.length > 0 ? tracks[0].clips.length : 0}
      </span>
    </div>
  </div>

  <div class="timeline-content">
    <div class="timeline-ruler">
      <canvas
        bind:this={canvas}
        style="width: {timelineContentWidth}px;"
        on:click={handleCanvasClick}
        on:mousedown={handleCanvasMouseDown}
        on:mousemove={handleCanvasMouseMove}
        on:mouseup={handleCanvasMouseUp}
        on:mouseleave={handleCanvasMouseUp}
      />
    </div>

    <div
      class="timeline-tracks"
      class:dragging-over={isDraggingOverTimeline}
      role="region"
      aria-label="Timeline tracks drop zone"
      on:dragover={handleDragOver}
      on:dragleave={handleDragLeave}
      on:drop={handleDrop}
    >
      <div
        class="timeline-tracks-inner"
        style="width: {timelineContentWidth}px; min-height: 100px;"
      >
        {#each tracks as trackItem (trackItem.id)}
          <TrackView
            track={trackItem}
            {pixelsPerSecond}
            bind:currentTime
            on:track-drop={handleDrop}
            on:clip-moved={handleClipMoved}
            on:clip-trimmed={handleClipTrimmed}
          />
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .timeline-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #0a0a0a;
    color: #fff;
    overflow: hidden;
  }

  .timeline-header {
    height: 40px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    display: flex;
    align-items: center;
    padding: 0 10px;
  }

  .timeline-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .timeline-controls button {
    width: 30px;
    height: 30px;
    background: #2a2a2a;
    border: 1px solid #444;
    color: #fff;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .timeline-controls button:hover {
    background: #3a3a3a;
  }

  .zoom-level {
    font-size: 12px;
    color: #999;
  }

  .timeline-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-x: auto;
    overflow-y: auto;
  }

  .timeline-ruler {
    height: 30px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    position: relative;
    overflow: visible;
  }

  .timeline-ruler canvas {
    height: 100%;
    cursor: pointer;
    display: block;
  }

  .timeline-tracks {
    flex: 1;
    overflow-x: auto;
    overflow-y: auto;
    min-height: 100px;
    position: relative;
  }

  .timeline-tracks-inner {
    position: relative;
  }

  .timeline-tracks.dragging-over {
    background: rgba(0, 120, 212, 0.2);
    border: 2px dashed #0078d4;
  }
</style>
