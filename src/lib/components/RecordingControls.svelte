<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import type { RecordingSources, RecordingSession } from '../types/recording';
  import type { MediaClip } from '../types/clip';
  import { mediaLibrary } from '../stores/media-library';

  // Recording state
  let isRecording = false;
  let isPreparing = false;
  let currentSession: RecordingSession | null = null;
  let recordingDuration = 0;
  let error: string | null = null;

  // Recording sources
  let sources: RecordingSources | null = null;
  let selectedScreenId: string | null = null;
  let selectedCameraId: string | null = null;
  let selectedMicrophoneId: string | null = null; // Add microphone selection
  let browserCameraDeviceId: string | null = null; // Actual browser device ID (UUID)
  let recordingType: 'screen' | 'webcam' | 'screen_webcam' = 'screen';
  let includeAudio = true;
  let includeMicrophone = true;

  // Settings
  let resolution = '1920x1080';
  let fps = 30;

  // Event listeners
  let unlistenRecordingStarted: (() => void) | null = null;
  let unlistenRecordingProgress: (() => void) | null = null;
  let unlistenRecordingStopped: (() => void) | null = null;

  // Webcam preview
  let videoPreviewElement: HTMLVideoElement | null = null;
  let webcamStream: MediaStream | null = null;

  onMount(async () => {
    // Request permissions
    await requestPermissions();

    // Load recording sources
    await loadSources();

    // Listen for recording events
    unlistenRecordingStarted = await listen(
      'recording_started',
      (event: { payload: { session_id: string } }) => {
        console.log('Recording started:', event.payload);
        isRecording = true;
        isPreparing = false;
      }
    );

    unlistenRecordingProgress = await listen(
      'recording_progress',
      (event: { payload: { session_id: string; duration: number } }) => {
        if (currentSession && event.payload.session_id === currentSession.id) {
          recordingDuration = event.payload.duration;
        }
      }
    );

    unlistenRecordingStopped = await listen(
      'recording_stopped',
      (event: { payload: { session_id: string; media_clip_id: string } }) => {
        console.log('Recording stopped:', event.payload);
        // Note: stopRecording() handles adding clip to library and resetting state
      }
    );
  });

  onDestroy(() => {
    // Clean up event listeners
    if (unlistenRecordingStarted) unlistenRecordingStarted();
    if (unlistenRecordingProgress) unlistenRecordingProgress();
    if (unlistenRecordingStopped) unlistenRecordingStopped();
    
    // Stop webcam stream
    stopWebcamPreview();
  });

  async function requestPermissions() {
    try {
      const permissions = ['screen', 'camera', 'microphone'];
      const result = await invoke<{
        granted: { screen: boolean; camera: boolean; microphone: boolean };
      }>('request_recording_permissions', { permissions });
      console.log('Permissions:', result.granted);
    } catch (err) {
      error = `Failed to request permissions: ${err}`;
      console.error(error);
    }
  }

  async function loadSources() {
    try {
      sources = await invoke<RecordingSources>('list_recording_sources');

      // Select first available sources by default
      if (sources.screens.length > 0) {
        selectedScreenId = sources.screens[0].id;
      }
      if (sources.cameras.length > 0) {
        selectedCameraId = sources.cameras[0].id;
      }
      if (sources.microphones.length > 0) {
        selectedMicrophoneId = sources.microphones[0].id;
      }

      // Get browser's actual camera device IDs for preview
      if (navigator.mediaDevices && navigator.mediaDevices.enumerateDevices) {
        const devices = await navigator.mediaDevices.enumerateDevices();
        const videoDevices = devices.filter(device => device.kind === 'videoinput');
        if (videoDevices.length > 0) {
          browserCameraDeviceId = videoDevices[0].deviceId; // Use first camera for preview
        }
      }
    } catch (err) {
      error = `Failed to load sources: ${err}`;
      console.error(error);
    }
  }

  async function startWebcamPreview() {
    if (webcamStream) {
      stopWebcamPreview();
    }

    try {
      // Request webcam access from browser using browser device ID
      // Don't constrain to specific device - just get any available camera
      webcamStream = await navigator.mediaDevices.getUserMedia({
        video: {
          width: { ideal: 1280 },
          height: { ideal: 720 }
        },
        audio: false // Don't capture audio in preview
      });

      if (videoPreviewElement) {
        videoPreviewElement.srcObject = webcamStream;
        videoPreviewElement.play();
      }
    } catch (err) {
      console.error('Failed to start webcam preview:', err);
      error = `Failed to access webcam: ${err}`;
    }
  }

  function stopWebcamPreview() {
    if (webcamStream) {
      webcamStream.getTracks().forEach(track => track.stop());
      webcamStream = null;
    }
    if (videoPreviewElement) {
      videoPreviewElement.srcObject = null;
    }
  }

  // Start webcam preview when webcam mode is selected
  $: if (recordingType === 'webcam' || recordingType === 'screen_webcam') {
    if (!isRecording && !isPreparing) {
      startWebcamPreview();
    }
  } else {
    stopWebcamPreview();
  }

  // Update preview when camera changes (simplified - just restart preview)
  $: if (selectedCameraId && (recordingType === 'webcam' || recordingType === 'screen_webcam') && !isRecording && !isPreparing) {
    // Restart preview when camera selection changes
    startWebcamPreview();
  }

  async function startRecording() {
    if (isRecording || isPreparing) return;

    error = null;
    isPreparing = true;

    // CRITICAL: Stop webcam preview before starting FFmpeg recording
    // This releases the microphone and camera so FFmpeg can access them
    stopWebcamPreview();

    try {
      // Build audio sources list
      const audioSources: string[] = [];
      if (includeAudio) audioSources.push('system');
      if (includeMicrophone) audioSources.push('microphone');

      // Build recording config
      const config = {
        type: recordingType,
        screen_source_id: recordingType !== 'webcam' ? selectedScreenId : null,
        camera_device_id: recordingType !== 'screen' ? selectedCameraId : null,
        audio_sources: audioSources,
        microphone_device_id: includeMicrophone ? selectedMicrophoneId : null,
        settings: {
          resolution,
          fps,
        },
      };

      currentSession = await invoke<RecordingSession>('start_recording', { config });
      console.log('Recording session started:', currentSession);
    } catch (err) {
      error = `Failed to start recording: ${err}`;
      console.error(error);
      isPreparing = false;
      
      // Restart preview if recording failed
      if (recordingType === 'webcam' || recordingType === 'screen_webcam') {
        startWebcamPreview();
      }
    }
  }

  async function stopRecording() {
    if (!isRecording || !currentSession) return;

    try {
      const mediaClip = await invoke<MediaClip>('stop_recording', {
        sessionId: currentSession.id, // Use camelCase - Tauri converts to snake_case automatically
      });
      console.log('Recording stopped, media clip created:', mediaClip);

      // Add clip to media library
      mediaLibrary.update((clips) => [...clips, mediaClip]);

      // Reset recording state
      isRecording = false;
      currentSession = null;
      recordingDuration = 0;
      
      // Restart webcam preview after recording stops (if in webcam mode)
      if (recordingType === 'webcam' || recordingType === 'screen_webcam') {
        startWebcamPreview();
      }
    } catch (err) {
      error = `Failed to stop recording: ${err}`;
      console.error(error);
      isRecording = false;
      currentSession = null;
      recordingDuration = 0;
      
      // Restart preview even on error
      if (recordingType === 'webcam' || recordingType === 'screen_webcam') {
        startWebcamPreview();
      }
    }
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  }
</script>

<div class="recording-controls">
  <h2>Recording</h2>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if !isRecording && !isPreparing}
    <!-- Recording type selector -->
    <div class="section">
      <div class="section-header">
        <strong>Recording Type:</strong>
      </div>
      <div class="recording-type-buttons">
        <button
          class:active={recordingType === 'screen'}
          on:click={() => (recordingType = 'screen')}
        >
          Screen Only
        </button>
        <button
          class:active={recordingType === 'webcam'}
          on:click={() => (recordingType = 'webcam')}
        >
          Webcam Only
        </button>
        <button
          class:active={recordingType === 'screen_webcam'}
          on:click={() => (recordingType = 'screen_webcam')}
        >
          Screen + Webcam
        </button>
      </div>
    </div>

    <!-- Screen source selector -->
    {#if (recordingType === 'screen' || recordingType === 'screen_webcam') && sources}
      <div class="section">
        <label for="screen-select">
          <strong>Screen:</strong>
        </label>
        <select id="screen-select" bind:value={selectedScreenId}>
          {#each sources.screens as screen}
            <option value={screen.id}>
              {screen.name} ({screen.resolution})
            </option>
          {/each}
        </select>
      </div>
    {/if}

    <!-- Camera source selector -->
    {#if (recordingType === 'webcam' || recordingType === 'screen_webcam') && sources}
      <div class="section">
        <label for="camera-select">
          <strong>Camera:</strong>
        </label>
        <select id="camera-select" bind:value={selectedCameraId}>
          {#each sources.cameras as camera}
            <option value={camera.id}>
              {camera.name}
            </option>
          {/each}
        </select>
      </div>

      <!-- Webcam preview -->
      <div class="section webcam-preview">
        {#if webcamStream}
          <!-- svelte-ignore a11y-media-has-caption -->
          <video
            bind:this={videoPreviewElement}
            class="webcam-video"
            autoplay
            playsinline
            muted
          />
        {:else}
          <div class="preview-placeholder">
            <p>📹 Webcam Preview</p>
            <p class="preview-note">Requesting camera access...</p>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Audio settings -->
    <div class="section">
      <div class="section-header">
        <strong>Audio:</strong>
      </div>
      <div class="checkbox-group">
        <label>
          <input type="checkbox" bind:checked={includeAudio} />
          System Audio
        </label>
        <label>
          <input type="checkbox" bind:checked={includeMicrophone} />
          Microphone
        </label>
      </div>
      
      <!-- Microphone selector -->
      {#if includeMicrophone && sources && sources.microphones.length > 0}
        <div class="microphone-selector">
          <label for="microphone-select">
            <strong>Microphone:</strong>
          </label>
          <select id="microphone-select" bind:value={selectedMicrophoneId}>
            {#each sources.microphones as microphone}
              <option value={microphone.id}>
                {microphone.name}
              </option>
            {/each}
          </select>
        </div>
      {/if}
    </div>

    <!-- Recording settings -->
    <div class="section">
      <div class="section-header">
        <strong>Settings:</strong>
      </div>
      <div class="settings-row">
        <select bind:value={resolution}>
          <option value="1920x1080">1080p</option>
          <option value="1280x720">720p</option>
          <option value="2560x1440">1440p</option>
          <option value="3840x2160">4K</option>
        </select>
        <select bind:value={fps}>
          <option value={30}>30 FPS</option>
          <option value={60}>60 FPS</option>
        </select>
      </div>
    </div>

    <!-- Start recording button -->
    <div class="section">
      <button class="record-button" on:click={startRecording}>
        <span class="record-icon">⏺</span>
        Start Recording
      </button>
    </div>
  {:else if isPreparing}
    <div class="preparing">
      <p>Preparing to record...</p>
    </div>
  {:else if isRecording}
    <!-- Recording indicator -->
    <div class="recording-status">
      <div class="recording-indicator">
        <span class="recording-dot"></span>
        <span class="recording-label">RECORDING</span>
      </div>

      <div class="recording-timer">
        <span class="timer-display">{formatDuration(recordingDuration)}</span>
      </div>

      <button class="stop-button" on:click={stopRecording}>
        <span class="stop-icon">⏹</span>
        Stop Recording
      </button>
    </div>
  {/if}
</div>

<style>
  .recording-controls {
    padding: 20px;
    background: #1a1a1a;
    border-radius: 8px;
    color: #ffffff;
  }

  h2 {
    margin: 0 0 20px 0;
    font-size: 20px;
    font-weight: 600;
  }

  .error {
    background: #ff4444;
    color: white;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 16px;
  }

  .section {
    margin-bottom: 20px;
  }

  label {
    display: block;
    margin-bottom: 8px;
    color: #cccccc;
  }

  .section-header {
    margin-bottom: 8px;
    color: #cccccc;
  }

  .recording-type-buttons {
    display: flex;
    gap: 8px;
  }

  .recording-type-buttons button {
    flex: 1;
    padding: 10px 16px;
    background: #2a2a2a;
    border: 1px solid #444444;
    border-radius: 4px;
    color: #ffffff;
    cursor: pointer;
    transition: all 0.2s;
  }

  .recording-type-buttons button:hover {
    background: #333333;
    border-color: #555555;
  }

  .recording-type-buttons button.active {
    background: #0066cc;
    border-color: #0077ee;
  }

  select {
    width: 100%;
    padding: 8px 12px;
    background: #2a2a2a;
    border: 1px solid #444444;
    border-radius: 4px;
    color: #ffffff;
    cursor: pointer;
  }

  select:hover {
    border-color: #555555;
  }

  .checkbox-group {
    display: flex;
    gap: 16px;
  }

  .checkbox-group label {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
  }

  input[type='checkbox'] {
    cursor: pointer;
  }

  .microphone-selector {
    margin-top: 12px;
  }

  .microphone-selector label {
    display: block;
    margin-bottom: 6px;
    font-size: 14px;
  }

  .microphone-selector select {
    width: 100%;
  }

  .settings-row {
    display: flex;
    gap: 12px;
  }

  .settings-row select {
    flex: 1;
  }

  .webcam-preview {
    margin-top: 16px;
  }

  .webcam-video {
    width: 100%;
    height: auto;
    border-radius: 8px;
    background: #000000;
    aspect-ratio: 16/9;
    object-fit: cover;
  }

  .preview-placeholder {
    background: #2a2a2a;
    border: 2px dashed #444444;
    border-radius: 8px;
    padding: 40px;
    text-align: center;
    color: #888888;
    aspect-ratio: 16/9;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .preview-placeholder p {
    margin: 8px 0;
  }

  .preview-placeholder p:first-child {
    font-size: 32px;
    margin-bottom: 12px;
  }

  .preview-note {
    font-size: 13px;
    color: #666666;
  }

  .record-button {
    width: 100%;
    padding: 16px;
    background: #cc0000;
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    transition: background 0.2s;
  }

  .record-button:hover {
    background: #dd0000;
  }

  .record-icon {
    font-size: 20px;
  }

  .preparing {
    text-align: center;
    padding: 40px;
    color: #cccccc;
  }

  .recording-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    padding: 20px;
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .recording-dot {
    width: 16px;
    height: 16px;
    background: #ff0000;
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }

  .recording-label {
    font-size: 18px;
    font-weight: 600;
    color: #ff0000;
  }

  .recording-timer {
    font-size: 48px;
    font-weight: 300;
    font-family: 'Courier New', monospace;
    color: #ffffff;
  }

  .timer-display {
    display: inline-block;
    min-width: 120px;
    text-align: center;
  }

  .stop-button {
    padding: 14px 32px;
    background: #444444;
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    transition: background 0.2s;
  }

  .stop-button:hover {
    background: #555555;
  }

  .stop-icon {
    font-size: 18px;
  }
</style>
