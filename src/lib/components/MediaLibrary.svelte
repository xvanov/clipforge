<script lang="ts">
  import { mediaLibrary } from '$lib/stores/media-library';
  import MediaClipCard from './MediaClipCard.svelte';

  // TODO: T035 - Implement drag-and-drop import
  // TODO: T036 - Implement file picker dialog

  function handleImportClick() {
    // TODO: Open file picker dialog
    console.log('Import clicked');
  }
</script>

<div class="media-library">
  <div class="media-library__header">
    <h2>Media Library</h2>
    <button on:click={handleImportClick} class="btn-import">Import</button>
  </div>

  <div class="media-library__grid">
    {#if $mediaLibrary.length === 0}
      <div class="empty-state">
        <p>No media files yet</p>
        <p class="empty-state__hint">Click Import or drag files here</p>
      </div>
    {:else}
      {#each $mediaLibrary as clip (clip.id)}
        <MediaClipCard {clip} />
      {/each}
    {/if}
  </div>
</div>

<style>
  .media-library {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    color: #fff;
    overflow: hidden;
  }

  .media-library__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #333;
  }

  .media-library__header h2 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 500;
  }

  .btn-import {
    padding: 0.5rem 1rem;
    background: #0078d4;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .btn-import:hover {
    background: #106ebe;
  }

  .media-library__grid {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1rem;
    align-content: start;
  }

  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 3rem 1rem;
    color: #888;
  }

  .empty-state__hint {
    font-size: 0.9rem;
    margin-top: 0.5rem;
  }
</style>

