<script lang="ts">
  import { createEventDispatcher, onDestroy } from 'svelte';
  import type { TimelineClip } from '../types/timeline';

  export let clip: TimelineClip;
  export let pixelsPerSecond: number = 50;
  export let scrollLeft: number = 0;
  export let currentTime: number = 0;
  export let locked: boolean = false;
  // Optional: pass the source media duration to validate maxOutPoint
  export let mediaDuration: number | undefined = undefined;

  const dispatch = createEventDispatcher();

  let clipElement: HTMLDivElement;
  let isDragging: boolean = false;
  let isTrimmingStart: boolean = false;
  let isTrimmingEnd: boolean = false;
  let dragStartX: number = 0;
  let originalStartTime: number = 0;
  let originalInPoint: number = 0;
  let originalOutPoint: number = 0;

  $: duration = clip.out_point - clip.in_point;
  $: endTime = clip.start_time + duration;
  $: leftPosition = clip.start_time * pixelsPerSecond - scrollLeft;
  $: width = duration * pixelsPerSecond;

  function handleMouseDown(event: MouseEvent) {
    if (locked) return;

    event.stopPropagation();

    const rect = clipElement.getBoundingClientRect();
    const offsetX = event.clientX - rect.left;

    // Check if clicking on trim handles
    if (offsetX < 10) {
      isTrimmingStart = true;
      originalInPoint = clip.in_point;
      originalStartTime = clip.start_time;
    } else if (offsetX > rect.width - 10) {
      isTrimmingEnd = true;
      originalOutPoint = clip.out_point;
    } else {
      isDragging = true;
      originalStartTime = clip.start_time;
    }

    dragStartX = event.clientX;
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);

    dispatch('drag-start', {
      clipId: clip.id,
      offsetX,
    });
  }

  function handleMouseMove(event: MouseEvent) {
    if (locked) return;

    const deltaX = event.clientX - dragStartX;
    const deltaTime = deltaX / pixelsPerSecond;

    if (isDragging) {
      const newStartTime = Math.max(0, originalStartTime + deltaTime);
      dispatch('drag', {
        clipId: clip.id,
        startTime: newStartTime,
      });
    } else if (isTrimmingStart) {
      const newInPoint = Math.max(0, Math.min(originalInPoint + deltaTime, clip.out_point - 0.1));
      const newStartTime = originalStartTime + (newInPoint - originalInPoint);
      dispatch('trimmed', {
        clipId: clip.id,
        inPoint: newInPoint,
        startTime: newStartTime,
      });
    } else if (isTrimmingEnd) {
      // Constrain maxOutPoint by media duration if provided, otherwise use current out_point
      const maxOutPoint = mediaDuration !== undefined ? mediaDuration : clip.out_point;
      const newOutPoint = Math.max(
        clip.in_point + 0.1,
        Math.min(originalOutPoint + deltaTime, maxOutPoint)
      );
      dispatch('trimmed', {
        clipId: clip.id,
        outPoint: newOutPoint,
      });
    }
  }

  function handleMouseUp() {
    if (isDragging || isTrimmingStart || isTrimmingEnd) {
      dispatch('drag-end');
    }

    isDragging = false;
    isTrimmingStart = false;
    isTrimmingEnd = false;

    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }

  // Cleanup event listeners on component destroy
  onDestroy(() => {
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  });

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();

    // Show context menu with options (split, delete)
    // For now, just log
    console.log('Context menu for clip:', clip.id);
  }

  function handleDoubleClick() {
    // Zoom to clip or open clip properties
    console.log('Double click on clip:', clip.id);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (locked) return;

    if (event.key === 'Delete' || event.key === 'Backspace') {
      dispatch('deleted', { clipId: clip.id });
    } else if (event.key === 's' || event.key === 'S') {
      // Split at playhead
      if (currentTime >= clip.start_time && currentTime <= endTime) {
        dispatch('split', {
          clipId: clip.id,
          splitTime: currentTime,
        });
      }
    }
  }
</script>

<div
  bind:this={clipElement}
  class="timeline-clip"
  class:dragging={isDragging}
  class:trimming={isTrimmingStart || isTrimmingEnd}
  class:locked
  style="left: {leftPosition}px; width: {width}px;"
  on:mousedown={handleMouseDown}
  on:contextmenu={handleContextMenu}
  on:dblclick={handleDoubleClick}
  on:keydown={handleKeyDown}
  role="button"
  tabindex="0"
>
  <div class="clip-content">
    <div class="clip-name">{clip.id.substring(0, 8)}</div>
    <div class="clip-duration">{duration.toFixed(2)}s</div>
  </div>

  {#if !locked}
    <div class="clip-trim-handle left" title="Trim start" />
    <div class="clip-trim-handle right" title="Trim end" />
  {/if}
</div>

<style>
  .timeline-clip {
    position: absolute;
    top: 10px;
    height: 60px;
    background: linear-gradient(135deg, #3a3a7a 0%, #2a2a5a 100%);
    border: 2px solid #4a4a8a;
    border-radius: 4px;
    cursor: grab;
    overflow: hidden;
    user-select: none;
    min-width: 20px; /* Ensure clip is always visible */
  }

  .timeline-clip:hover {
    border-color: #6a6aaa;
  }

  .timeline-clip.dragging {
    cursor: grabbing;
    opacity: 0.8;
  }

  .timeline-clip.trimming {
    cursor: ew-resize;
  }

  .timeline-clip.locked {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .clip-content {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    pointer-events: none;
  }

  .clip-name {
    font-size: 12px;
    font-weight: 500;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .clip-duration {
    font-size: 10px;
    color: #aaa;
  }

  .clip-trim-handle {
    position: absolute;
    top: 0;
    width: 10px;
    height: 100%;
    background: rgba(255, 255, 255, 0.1);
    cursor: ew-resize;
  }

  .clip-trim-handle.left {
    left: 0;
  }

  .clip-trim-handle.right {
    right: 0;
  }

  .clip-trim-handle:hover {
    background: rgba(255, 255, 255, 0.3);
  }
</style>
