<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen } from '@tauri-apps/api/event';
  import type { Caption, CaptionGenerationProgress } from '$lib/types/caption';
  import { onMount, onDestroy } from 'svelte';

  export let clipId: string;
  export let captions: Caption[] = [];

  let generating = false;
  let progress = 0;
  let statusMessage = '';
  let selectedLanguage = 'en';
  let selectedCaption: Caption | null = null;
  let editingCaption: Caption | null = null;

  let unlistenProgress: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;

  const languages = [
    { code: 'en', name: 'English' },
    { code: 'es', name: 'Spanish' },
    { code: 'fr', name: 'French' },
    { code: 'de', name: 'German' },
    { code: 'it', name: 'Italian' },
    { code: 'pt', name: 'Portuguese' },
    { code: 'ja', name: 'Japanese' },
    { code: 'ko', name: 'Korean' },
    { code: 'zh', name: 'Chinese' },
    { code: 'auto', name: 'Auto-detect' },
  ];

  onMount(async () => {
    // Listen for caption generation events
    unlistenProgress = await listen<CaptionGenerationProgress>(
      'caption_generation_progress',
      (event) => {
        progress = event.payload.progress * 100;
        statusMessage = event.payload.message || '';
      }
    );

    unlistenComplete = await listen<{ captions: Caption[] }>(
      'caption_generation_complete',
      (event) => {
        generating = false;
        progress = 100;
        statusMessage = 'Captions generated successfully!';
        captions = event.payload.captions;
        setTimeout(() => {
          statusMessage = '';
          progress = 0;
        }, 3000);
      }
    );

    unlistenError = await listen<CaptionGenerationProgress>('caption_generation_error', (event) => {
      generating = false;
      progress = 0;
      statusMessage = `Error: ${event.payload.message || 'Unknown error'}`;
    });
  });

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress();
    if (unlistenComplete) unlistenComplete();
    if (unlistenError) unlistenError();
  });

  async function generateCaptions() {
    try {
      generating = true;
      progress = 0;
      statusMessage = 'Starting caption generation...';
      await invoke('generate_captions', {
        clipId,
        language: selectedLanguage,
      });
    } catch (error) {
      console.error('Failed to generate captions:', error);
      statusMessage = `Error: ${error}`;
      generating = false;
    }
  }

  function selectCaption(caption: Caption) {
    selectedCaption = caption;
    editingCaption = null;
  }

  function startEditingCaption(caption: Caption) {
    editingCaption = { ...caption };
  }

  async function saveCaption() {
    if (!editingCaption) return;

    try {
      const updated = await invoke<Caption>('update_caption', {
        clipId,
        captionId: editingCaption.id,
        text: editingCaption.text,
        startTime: editingCaption.start_time,
        endTime: editingCaption.end_time,
      });

      // Update in local state
      const index = captions.findIndex((c) => c.id === updated.id);
      if (index !== -1) {
        captions[index] = updated;
        captions = captions; // Trigger reactivity
      }

      selectedCaption = updated;
      editingCaption = null;
    } catch (error) {
      console.error('Failed to update caption:', error);
      alert(`Failed to update caption: ${error}`);
    }
  }

  function cancelEditing() {
    editingCaption = null;
  }

  async function deleteCaption(captionId: string) {
    if (!confirm('Are you sure you want to delete this caption?')) return;

    try {
      await invoke('delete_caption', {
        clipId,
        captionId,
      });

      captions = captions.filter((c) => c.id !== captionId);
      if (selectedCaption?.id === captionId) {
        selectedCaption = null;
      }
    } catch (error) {
      console.error('Failed to delete caption:', error);
      alert(`Failed to delete caption: ${error}`);
    }
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 1000);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${ms.toString().padStart(3, '0')}`;
  }
</script>

<div class="captions-panel">
  <div class="captions-header">
    <h3>AI Captions</h3>
    {#if captions.length > 0}
      <span class="caption-count">{captions.length} captions</span>
    {/if}
  </div>

  {#if captions.length === 0 && !generating}
    <div class="captions-empty">
      <div class="generate-section">
        <p>Generate AI-powered captions from audio</p>
        <div class="language-selector">
          <label for="language">Language:</label>
          <select id="language" bind:value={selectedLanguage}>
            {#each languages as lang}
              <option value={lang.code}>{lang.name}</option>
            {/each}
          </select>
        </div>
        <button class="btn-primary" on:click={generateCaptions}> Generate Captions </button>
      </div>
    </div>
  {/if}

  {#if generating}
    <div class="generation-progress">
      <div class="progress-bar">
        <div class="progress-fill" style="width: {progress}%"></div>
      </div>
      <p class="status-message">{statusMessage}</p>
    </div>
  {/if}

  {#if captions.length > 0 && !generating}
    <div class="captions-controls">
      <button class="btn-secondary" on:click={generateCaptions}> Regenerate Captions </button>
    </div>

    <div class="captions-list">
      {#each captions as caption}
        <button
          type="button"
          class="caption-item"
          class:selected={selectedCaption?.id === caption.id}
          on:click={() => selectCaption(caption)}
        >
          <div class="caption-time">
            {formatTime(caption.start_time)} - {formatTime(caption.end_time)}
          </div>
          <div class="caption-text">{caption.text}</div>
          {#if caption.confidence}
            <div class="caption-confidence">
              Confidence: {(caption.confidence * 100).toFixed(0)}%
            </div>
          {/if}
          <div class="caption-actions">
            <button class="btn-sm" on:click|stopPropagation={() => startEditingCaption(caption)}>
              Edit
            </button>
            <button
              class="btn-sm btn-danger"
              on:click|stopPropagation={() => deleteCaption(caption.id)}
            >
              Delete
            </button>
          </div>
        </button>
      {/each}
    </div>
  {/if}

  {#if editingCaption}
    <div class="caption-editor-modal">
      <div class="modal-content">
        <h4>Edit Caption</h4>
        <div class="form-group">
          <label for="caption-text">Text:</label>
          <textarea id="caption-text" bind:value={editingCaption.text} rows="3"></textarea>
        </div>
        <div class="form-row">
          <div class="form-group">
            <label for="start-time">Start Time (s):</label>
            <input
              id="start-time"
              type="number"
              step="0.001"
              bind:value={editingCaption.start_time}
            />
          </div>
          <div class="form-group">
            <label for="end-time">End Time (s):</label>
            <input id="end-time" type="number" step="0.001" bind:value={editingCaption.end_time} />
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn-primary" on:click={saveCaption}>Save</button>
          <button class="btn-secondary" on:click={cancelEditing}>Cancel</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .captions-panel {
    background: #1e1e1e;
    border-radius: 4px;
    padding: 16px;
    color: #fff;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .captions-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .captions-header h3 {
    margin: 0;
    font-size: 18px;
  }

  .caption-count {
    color: #888;
    font-size: 14px;
  }

  .captions-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .generate-section {
    text-align: center;
  }

  .language-selector {
    margin: 16px 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .language-selector select {
    padding: 8px;
    border-radius: 4px;
    border: 1px solid #444;
    background: #2a2a2a;
    color: #fff;
  }

  .generation-progress {
    margin: 16px 0;
  }

  .progress-bar {
    height: 8px;
    background: #333;
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #4a90e2, #50c878);
    transition: width 0.3s ease;
  }

  .status-message {
    color: #888;
    font-size: 14px;
    text-align: center;
  }

  .captions-controls {
    margin-bottom: 12px;
  }

  .captions-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .caption-item {
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 4px;
    padding: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .caption-item:hover {
    background: #333;
    border-color: #555;
  }

  .caption-item.selected {
    background: #3a3a3a;
    border-color: #4a90e2;
  }

  .caption-time {
    color: #4a90e2;
    font-size: 12px;
    margin-bottom: 4px;
    font-family: monospace;
  }

  .caption-text {
    color: #fff;
    margin-bottom: 8px;
    line-height: 1.4;
  }

  .caption-confidence {
    color: #888;
    font-size: 11px;
    margin-bottom: 8px;
  }

  .caption-actions {
    display: flex;
    gap: 8px;
  }

  .btn-primary,
  .btn-secondary,
  .btn-sm {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #4a90e2;
    color: #fff;
  }

  .btn-primary:hover {
    background: #357abd;
  }

  .btn-secondary {
    background: #444;
    color: #fff;
  }

  .btn-secondary:hover {
    background: #555;
  }

  .btn-sm {
    padding: 4px 12px;
    font-size: 12px;
    background: #444;
    color: #fff;
  }

  .btn-sm:hover {
    background: #555;
  }

  .btn-danger {
    background: #e74c3c;
  }

  .btn-danger:hover {
    background: #c0392b;
  }

  .caption-editor-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 8px;
    padding: 24px;
    max-width: 500px;
    width: 90%;
  }

  .modal-content h4 {
    margin: 0 0 16px 0;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 4px;
    color: #aaa;
    font-size: 14px;
  }

  .form-group textarea,
  .form-group input {
    width: 100%;
    padding: 8px;
    border: 1px solid #444;
    border-radius: 4px;
    background: #1e1e1e;
    color: #fff;
    font-family: inherit;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .modal-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 16px;
  }
</style>
