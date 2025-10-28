<script lang="ts">
  import { invoke } from '@tauri-apps/api';
  import { open } from '@tauri-apps/api/dialog';
  import { convertFileSrc } from '@tauri-apps/api/tauri';
  import { mediaLibrary, addClipToLibrary } from '$lib/stores/media-library';
  import MediaClipCard from './MediaClipCard.svelte';
  import type { MediaClip } from '$lib/types/clip';

  let importing = false;
  let errorMessage = '';

  // T035: Drag-and-drop file import
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    
    if (!event.dataTransfer) return;
    
    const files = Array.from(event.dataTransfer.files);
    const videoPaths = files
      .filter((file: File) => {
        const ext = file.name.toLowerCase();
        return ext.endsWith('.mp4') || ext.endsWith('.mov') || 
               ext.endsWith('.avi') || ext.endsWith('.webm');
      })
      .map((file: File & { path?: string }) => file.path || file.name);

    if (videoPaths.length > 0) {
      await importFiles(videoPaths);
    }
  }

  // T036: File picker dialog integration
  async function openFilePicker() {
    try {
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'Video',
          extensions: ['mp4', 'mov', 'avi', 'webm', 'mkv']
        }]
      });

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        await importFiles(paths);
      }
    } catch (err: any) {
      errorMessage = `Failed to open file picker: ${err}`;
    }
  }

  async function importFiles(paths: string[]) {
    importing = true;
    errorMessage = '';

    try {
      const result = await invoke<{
        clips: MediaClip[],
        errors: Array<{ path: string, error: string }>
      }>('import_media_files', { paths });

      // Add successfully imported clips to store
      result.clips.forEach((clip: MediaClip) => {
        addClipToLibrary(clip);
      });

      // Show errors if any
      if (result.errors.length > 0) {
        const errorPaths = result.errors.map((e: { path: string }) => e.path).join(', ');
        errorMessage = `Failed to import: ${errorPaths}`;
      }
    } catch (err: any) {
      errorMessage = `Import failed: ${err}`;
    } finally {
      importing = false;
    }
  }

  function handleClipSelect(clip: MediaClip) {
    // Dispatch event to notify parent component
    const event = new CustomEvent('clipselect', { detail: clip });
    window.dispatchEvent(event);
  }
</script>

<div 
  class="media-library"
  role="region"
  aria-label="Media Library"
  on:dragover={handleDragOver}
  on:drop={handleDrop}
>
  <div class="header">
    <h2>Media Library</h2>
    <button on:click={openFilePicker} disabled={importing} class="import-btn">
      {importing ? 'Importing...' : 'Import Media'}
    </button>
  </div>

  {#if errorMessage}
    <div class="error">{errorMessage}</div>
  {/if}

  <div class="clips-grid">
    {#if $mediaLibrary.length === 0}
      <div class="empty-state">
        <p>No media files imported yet</p>
        <p class="hint">Drag and drop video files here or click Import Media</p>
      </div>
    {:else}
      {#each $mediaLibrary as clip (clip.id)}
        <MediaClipCard 
          {clip} 
          on:click={() => handleClipSelect(clip)} 
        />
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
    border-right: 1px solid #333;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #333;
  }

  .header h2 {
    margin: 0;
    color: #fff;
    font-size: 1.2rem;
  }

  .import-btn {
    padding: 0.5rem 1rem;
    background: #0078d4;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .import-btn:hover:not(:disabled) {
    background: #106ebe;
  }

  .import-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error {
    padding: 0.75rem 1rem;
    background: #d13438;
    color: white;
    margin: 1rem;
    border-radius: 4px;
  }

  .clips-grid {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
    align-content: start;
  }

  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 3rem 1rem;
    color: #888;
  }

  .empty-state p {
    margin: 0.5rem 0;
  }

  .hint {
    font-size: 0.9rem;
    color: #666;
  }
</style>
