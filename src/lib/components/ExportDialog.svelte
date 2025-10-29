<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { save } from '@tauri-apps/api/dialog';
  import {
    DEFAULT_EXPORT_SETTINGS,
    type ExportSettings,
    type ExportProgressEvent,
    type ExportCompleteEvent,
    type ExportErrorEvent,
    type ExportRequest,
    type ExportJobResponse,
  } from '../types/export';

  export let visible = false;
  export let onClose: () => void = () => {};

  let settings: ExportSettings = { ...DEFAULT_EXPORT_SETTINGS };
  let exporting = false;
  let progress = 0;
  let currentJobId: string | null = null;
  let eta = 0;
  let currentFrame = 0;
  let totalFrames = 0;
  let fps = 0;
  let errorMessage = '';
  let successMessage = '';
  let unlistenProgress: UnlistenFn | null = null;
  let unlistenComplete: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;

  const resolutionOptions: Array<{ value: ExportSettings['resolution']; label: string }> = [
    { value: 'source', label: 'Source Resolution' },
    { value: '2160p', label: '4K (3840x2160)' },
    { value: '1440p', label: '2K (2560x1440)' },
    { value: '1080p', label: 'Full HD (1920x1080)' },
    { value: '720p', label: 'HD (1280x720)' },
    { value: '480p', label: 'SD (854x480)' },
  ];

  const codecOptions: Array<{ value: ExportSettings['codec']; label: string }> = [
    { value: 'h264', label: 'H.264 (MP4)' },
    { value: 'hevc', label: 'H.265/HEVC (MP4)' },
    { value: 'vp9', label: 'VP9 (WebM)' },
  ];

  const qualityOptions: Array<{ value: ExportSettings['quality']; label: string }> = [
    { value: 'high', label: 'High Quality' },
    { value: 'medium', label: 'Medium Quality' },
    { value: 'low', label: 'Low Quality (Smaller File)' },
  ];

  const audioBitrateOptions = [128, 192, 320];

  // Set up event listeners
  async function setupEventListeners() {
    unlistenProgress = await listen<ExportProgressEvent>('export_progress', (event) => {
      if (event.payload.job_id === currentJobId) {
        progress = event.payload.progress * 100;
        currentFrame = event.payload.current_frame;
        totalFrames = event.payload.total_frames;
        fps = event.payload.fps;
        eta = event.payload.eta_seconds;
      }
    });

    unlistenComplete = await listen<ExportCompleteEvent>('export_complete', (event) => {
      if (event.payload.job_id === currentJobId) {
        exporting = false;
        progress = 100;
        successMessage = `Video exported successfully to ${event.payload.output_path}`;
        currentJobId = null;
        cleanupListeners();
      }
    });

    unlistenError = await listen<ExportErrorEvent>('export_error', (event) => {
      if (event.payload.job_id === currentJobId) {
        exporting = false;
        errorMessage = `Export failed: ${event.payload.error}`;
        currentJobId = null;
        cleanupListeners();
      }
    });
  }

  function cleanupListeners() {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
    if (unlistenComplete) {
      unlistenComplete();
      unlistenComplete = null;
    }
    if (unlistenError) {
      unlistenError();
      unlistenError = null;
    }
  }

  async function handleExport() {
    errorMessage = '';
    successMessage = '';

    // Open save dialog
    const outputPath = await save({
      defaultPath: 'output.mp4',
      filters: [
        {
          name: 'Video Files',
          extensions: settings.codec === 'vp9' ? ['webm'] : ['mp4'],
        },
      ],
    });

    if (!outputPath) {
      return; // User cancelled
    }

    try {
      exporting = true;
      progress = 0;
      currentFrame = 0;
      totalFrames = 0;
      eta = 0;

      // Set up event listeners
      await setupEventListeners();

      const request: ExportRequest = {
        output_path: outputPath,
        settings,
      };

      const response = await invoke<ExportJobResponse>('export_timeline', { request });
      currentJobId = response.job_id;
    } catch (error) {
      exporting = false;
      errorMessage = `Failed to start export: ${error}`;
      cleanupListeners();
    }
  }

  async function handleCancel() {
    if (!currentJobId) return;

    try {
      await invoke('cancel_export', { job_id: currentJobId });
      exporting = false;
      progress = 0;
      currentJobId = null;
      errorMessage = 'Export cancelled';
      cleanupListeners();
    } catch (error) {
      errorMessage = `Failed to cancel export: ${error}`;
    }
  }

  function handleClose() {
    if (exporting) {
      const confirm = window.confirm('Export is in progress. Are you sure you want to close?');
      if (!confirm) return;
      handleCancel();
    }
    cleanupListeners();
    onClose();
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="export-dialog-overlay" on:click={handleClose}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="export-dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h2>Export Video</h2>
        <button class="close-button" on:click={handleClose}>&times;</button>
      </div>

      <div class="dialog-content">
        {#if !exporting}
          <!-- Export Settings -->
          <div class="settings-section">
            <div class="form-group">
              <label for="resolution">Resolution</label>
              <select id="resolution" bind:value={settings.resolution}>
                {#each resolutionOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>

            <div class="form-group">
              <label for="codec">Codec</label>
              <select id="codec" bind:value={settings.codec}>
                {#each codecOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>

            <div class="form-group">
              <label for="quality">Quality</label>
              <select id="quality" bind:value={settings.quality}>
                {#each qualityOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>

            <div class="form-group">
              <label for="audioBitrate">Audio Bitrate (kbps)</label>
              <select id="audioBitrate" bind:value={settings.audio_bitrate}>
                {#each audioBitrateOptions as bitrate}
                  <option value={bitrate}>{bitrate} kbps</option>
                {/each}
              </select>
            </div>

            <div class="form-group checkbox-group">
              <label>
                <input type="checkbox" bind:checked={settings.hardware_acceleration} />
                Enable Hardware Acceleration
              </label>
            </div>
          </div>

          <!-- Messages -->
          {#if errorMessage}
            <div class="message error-message">{errorMessage}</div>
          {/if}
          {#if successMessage}
            <div class="message success-message">{successMessage}</div>
          {/if}

          <!-- Actions -->
          <div class="dialog-actions">
            <button class="secondary-button" on:click={handleClose}>Cancel</button>
            <button class="primary-button" on:click={handleExport}>Export</button>
          </div>
        {:else}
          <!-- Export Progress -->
          <div class="progress-section">
            <h3>Exporting...</h3>

            <div class="progress-bar-container">
              <div class="progress-bar" style="width: {progress}%"></div>
            </div>

            <div class="progress-stats">
              <p>Progress: {progress.toFixed(1)}%</p>
              <p>Frame: {currentFrame} / {totalFrames}</p>
              <p>Speed: {fps.toFixed(1)} fps</p>
              {#if eta > 0}
                <p>Time Remaining: {formatTime(eta)}</p>
              {/if}
            </div>

            {#if errorMessage}
              <div class="message error-message">{errorMessage}</div>
            {/if}

            <div class="dialog-actions">
              <button class="secondary-button" on:click={handleCancel}>Cancel Export</button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .export-dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .export-dialog {
    background: white;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid #e0e0e0;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 28px;
    cursor: pointer;
    color: #666;
    padding: 0;
    width: 32px;
    height: 32px;
    line-height: 1;
  }

  .close-button:hover {
    color: #000;
  }

  .dialog-content {
    padding: 20px;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group label {
    font-weight: 500;
    font-size: 14px;
    color: #333;
  }

  .form-group select {
    padding: 8px 12px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 14px;
    background: white;
  }

  .checkbox-group label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .checkbox-group input[type='checkbox'] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .message {
    padding: 12px;
    border-radius: 4px;
    margin-top: 16px;
    font-size: 14px;
  }

  .error-message {
    background: #fee;
    color: #c00;
    border: 1px solid #fcc;
  }

  .success-message {
    background: #efe;
    color: #060;
    border: 1px solid #cfc;
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 20px;
  }

  button {
    padding: 10px 20px;
    border-radius: 4px;
    border: none;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }

  .primary-button {
    background: #007bff;
    color: white;
  }

  .primary-button:hover {
    background: #0056b3;
  }

  .secondary-button {
    background: #6c757d;
    color: white;
  }

  .secondary-button:hover {
    background: #545b62;
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .progress-section h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .progress-bar-container {
    width: 100%;
    height: 30px;
    background: #e0e0e0;
    border-radius: 15px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #4caf50, #8bc34a);
    transition: width 0.3s ease;
  }

  .progress-stats {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-stats p {
    margin: 0;
    font-size: 14px;
    color: #666;
  }
</style>
