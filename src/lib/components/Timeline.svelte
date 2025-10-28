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
  
  // Subscribe to timeline store - use derived store for better reactivity
  let tracks: Track[] = [];
  $: tracks = $tracksStore;
  
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
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }
  
  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    
    if (!event.dataTransfer) return;
    
    const clipData = event.dataTransfer.getData('application/json');
    if (!clipData) return;
    
    try {
      const mediaClip: MediaClip = JSON.parse(clipData);
      const rect = timelineContainer.getBoundingClientRect();
      const x = event.clientX - rect.left + scrollLeft;
      let dropTime = x / pixelsPerSecond;
      
      // If dropping near the left edge (within 2 seconds), auto-position after existing clips
      if (dropTime < 2 && tracks.length > 0 && tracks[0].clips.length > 0) {
        const existingClips = tracks[0].clips;
        // Find the end time of the last clip
        const lastClipEnd = Math.max(
          ...existingClips.map(clip => clip.start_time + (clip.out_point - clip.in_point))
        );
        // Position new clip 0.5 seconds after the last clip
        dropTime = lastClipEnd + 0.5;
        console.log('Auto-positioning clip after existing clips at:', dropTime);
      }
      
      console.log('Dropping clip:', mediaClip.name, 'at time:', dropTime, 'duration:', mediaClip.duration);
      
      // Add clip to first track
      if (tracks.length > 0) {
        console.log('Adding to track:', tracks[0].id, 'existing clips:', tracks[0].clips.length);
        const result = await timelineStore.addClipToTimeline(
          mediaClip.id,
          tracks[0].id,
          dropTime,
          0,
          mediaClip.duration
        );
        console.log('Clip added successfully:', result);
        console.log('Current tracks state:', tracks);
      } else {
        console.error('No tracks available!');
      }
    } catch (error) {
      console.error('Failed to drop clip:', error);
    }
  }
  
  // Redraw timeline when current time changes
  $: if (ctx) {
    drawTimeline();
  }
  
  // Debug: Log track and clip state
  $: {
    console.log('Timeline tracks updated:', tracks.length);
    tracks.forEach((track, i) => {
      console.log(`Track ${i} (${track.name}): ${track.clips.length} clips`);
      track.clips.forEach((clip, j) => {
        console.log(`  Clip ${j}: ${clip.id.substring(0,8)} at ${clip.start_time}s (${clip.in_point}-${clip.out_point})`);
      });
    });
  }
</script>

<div class="timeline-container" 
     role="region"
     aria-label="Timeline editor"
     bind:this={timelineContainer}
     on:dragover={handleDragOver}
     on:drop={handleDrop}>
  
  <!-- Debug panel -->
  <div style="position: absolute; top: 5px; right: 5px; background: rgba(0,0,0,0.9); color: #0f0; padding: 8px; font-family: monospace; font-size: 11px; z-index: 1000; border: 1px solid #0f0; max-width: 300px;">
    <strong>DEBUG</strong><br/>
    Tracks: {tracks.length}<br/>
    {#each tracks as track, i}
      Track {i} "{track.name}": {track.clips.length} clips<br/>
      {#each track.clips as clip, j}
        &nbsp;&nbsp;Clip {j}: {clip.id.substring(0,6)}... at {clip.start_time.toFixed(1)}s (dur: {(clip.out_point - clip.in_point).toFixed(1)}s)<br/>
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
        on:click={handleCanvasClick}
        on:mousedown={handleCanvasMouseDown}
        on:mousemove={handleCanvasMouseMove}
        on:mouseup={handleCanvasMouseUp}
        on:mouseleave={handleCanvasMouseUp}
      />
    </div>
    
    <div class="timeline-tracks">
      {#each tracks as track (track.id)}
        <TrackView 
          {track} 
          {pixelsPerSecond} 
          {scrollLeft}
          bind:currentTime
        />
      {/each}
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
    overflow: auto;
  }
  
  .timeline-ruler {
    height: 30px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    position: relative;
  }
  
  .timeline-ruler canvas {
    width: 100%;
    height: 100%;
    cursor: pointer;
  }
  
  .timeline-tracks {
    flex: 1;
    overflow-y: auto;
  }
</style>
