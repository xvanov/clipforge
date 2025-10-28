<script lang="ts">
  import type { MediaClip } from '$lib/types/clip';

  export let clip: MediaClip;

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatFileSize(bytes: number): string {
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }

  function handleClick() {
    // TODO: Load clip into preview player
    console.log('Clicked clip:', clip.id);
  }
</script>

<div class="clip-card" on:click={handleClick} role="button" tabindex="0">
  <div class="clip-card__thumbnail">
    {#if clip.thumbnail_path}
      <img src={clip.thumbnail_path} alt={clip.name} />
    {:else}
      <div class="clip-card__placeholder">
        <span>🎬</span>
      </div>
    {/if}
  </div>
  
  <div class="clip-card__info">
    <div class="clip-card__name" title={clip.name}>
      {clip.name}
    </div>
    <div class="clip-card__details">
      <span>{clip.resolution}</span>
      <span>{formatDuration(clip.duration)}</span>
    </div>
  </div>
</div>

<style>
  .clip-card {
    background: #2a2a2a;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .clip-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .clip-card__thumbnail {
    width: 100%;
    aspect-ratio: 16 / 9;
    background: #1a1a1a;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .clip-card__thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .clip-card__placeholder {
    font-size: 2rem;
  }

  .clip-card__info {
    padding: 0.75rem;
  }

  .clip-card__name {
    font-size: 0.85rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 0.25rem;
  }

  .clip-card__details {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: #888;
  }
</style>

