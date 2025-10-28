<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/tauri';
  import type { MediaClip } from '$lib/types/clip';

  export let clip: MediaClip;

  // Format duration as MM:SS or HH:MM:SS
  function formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (hours > 0) {
      return `${hours}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  // Format file size
  function formatFileSize(bytes: number): string {
    const mb = bytes / (1024 * 1024);
    if (mb > 1024) {
      return `${(mb / 1024).toFixed(1)} GB`;
    }
    return `${mb.toFixed(1)} MB`;
  }

  // Get thumbnail URL
  function getThumbnailSrc(clip: MediaClip): string {
    if (clip.thumbnail_path) {
      return convertFileSrc(clip.thumbnail_path);
    }
    // Fallback placeholder
    return 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="200" height="112"%3E%3Crect width="200" height="112" fill="%23333"/%3E%3Ctext x="50%25" y="50%25" fill="%23666" text-anchor="middle" dy=".3em" font-family="Arial" font-size="14"%3ENo Thumbnail%3C/text%3E%3C/svg%3E';
  }

  // Enable drag to timeline
  function handleDragStart(event: DragEvent) {
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'copy';
      event.dataTransfer.setData('application/json', JSON.stringify(clip));
    }
  }
</script>

<button class="clip-card" on:click draggable="true" on:dragstart={handleDragStart}>
  <div class="thumbnail">
    <img src={getThumbnailSrc(clip)} alt={clip.name} />
    <div class="duration">{formatDuration(clip.duration)}</div>
  </div>

  <div class="info">
    <div class="name" title={clip.name}>{clip.name}</div>
    <div class="metadata">
      <span class="resolution">{clip.resolution}</span>
      <span class="fps">{Math.round(clip.fps)} fps</span>
    </div>
    <div class="details">
      <span class="file-size">{formatFileSize(clip.file_size)}</span>
      {#if clip.has_audio}
        <span class="audio-indicator" title="Has audio">🔊</span>
      {:else}
        <span class="audio-indicator muted" title="No audio">🔇</span>
      {/if}
    </div>
  </div>
</button>

<style>
  .clip-card {
    display: flex;
    flex-direction: column;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    padding: 0;
    width: 100%;
  }

  .clip-card:hover {
    border-color: #0078d4;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 120, 212, 0.3);
  }

  .thumbnail {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 9;
    background: #1a1a1a;
    overflow: hidden;
  }

  .thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .duration {
    position: absolute;
    bottom: 4px;
    right: 4px;
    background: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .info {
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .name {
    color: #fff;
    font-size: 0.9rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .metadata {
    display: flex;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: #aaa;
  }

  .details {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
    color: #888;
  }

  .audio-indicator {
    font-size: 1rem;
  }

  .audio-indicator.muted {
    opacity: 0.5;
  }
</style>
