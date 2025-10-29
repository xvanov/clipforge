<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api';
  import { convertFileSrc } from '@tauri-apps/api/tauri';
  import type { MediaClip } from '$lib/types/clip';

  const dispatch = createEventDispatcher();

  export let currentClip: MediaClip | null = null;
  export let currentTime: number = 0;
  export let clipStartTime: number = 0;
  export let clipInPoint: number = 0;
  export let clipOutPoint: number = 0;

  let videoElement: HTMLVideoElement;
  let isPlaying = false;
  let localTime = 0;
  let duration = 0;
  let volume = 1.0;
  let playbackError = '';
  let isLoadingProxy = false;
  let previousClipId: string | null = null;

  // When clip changes or playback starts, seek to the correct in-point
  $: if (currentClip && currentClip.id !== previousClipId && videoElement) {
    previousClipId = currentClip.id;
    const videoTime = clipInPoint + (currentTime - clipStartTime);
    if (videoElement.currentTime !== videoTime) {
      videoElement.currentTime = Math.max(0, Math.min(videoTime, clipOutPoint));
    }
  }

  // Sync external currentTime prop with video element during scrubbing
  $: if (videoElement && !isPlaying && currentClip) {
    const relativeTime = currentTime - clipStartTime;
    const videoTime = clipInPoint + relativeTime;
    const timeDiff = Math.abs(videoTime - localTime);

    // Only update if there's a significant difference (>0.1s) to avoid jitter
    if (timeDiff > 0.1) {
      videoElement.currentTime = Math.max(clipInPoint, Math.min(videoTime, clipOutPoint));
    }
  }

  // T037: Load clip for playback
  async function loadClip(clip: MediaClip) {
    try {
      playbackError = '';
      isLoadingProxy = false;

      // Remember if we were playing
      const wasPlaying = isPlaying;

      // Get playback path from backend (returns proxy if available, or source path)
      const playbackPath = await invoke<string>('load_clip_for_playback', {
        clipId: clip.id,
        useProxy: true, // Request proxy for better web compatibility
      });

      console.log('Playback path from backend:', playbackPath);

      if (videoElement) {
        // Show loading state if proxy doesn't exist yet but might be generated
        if (!clip.proxy_path && needsProxy(clip.codec)) {
          isLoadingProxy = true;
          // Poll for proxy availability
          pollForProxy(clip.id);
        }

        // Use the path returned from backend and convert to Tauri asset URL
        const assetUrl = convertFileSrc(playbackPath);
        console.log('Loading video from asset URL:', assetUrl);
        videoElement.src = assetUrl;

        // Set up one-time loadeddata listener to resume playback and seek
        const handleLoaded = () => {
          if (videoElement) {
            // Seek to correct position
            const videoTime = clipInPoint + (currentTime - clipStartTime);
            videoElement.currentTime = Math.max(clipInPoint, Math.min(videoTime, clipOutPoint));

            // Resume playback if we were playing
            if (wasPlaying) {
              videoElement.play().catch((err) => {
                console.error('Failed to resume playback:', err);
              });
            }
          }
          videoElement?.removeEventListener('loadeddata', handleLoaded);
        };

        videoElement.addEventListener('loadeddata', handleLoaded);
        videoElement.load();
      }
    } catch (err) {
      playbackError = `Failed to load clip: ${err}`;
      console.error(playbackError);
    }
  }

  // Check if codec needs a proxy
  function needsProxy(codec: string): boolean {
    const codecLower = codec.toLowerCase();
    const webCompatible = ['h264', 'vp8', 'vp9', 'av1'];
    return !webCompatible.some((c) => codecLower.includes(c));
  }

  // Poll for proxy generation completion
  async function pollForProxy(clipId: string) {
    const maxAttempts = 60; // 60 seconds max
    let attempts = 0;

    const interval = setInterval(async () => {
      attempts++;

      try {
        // Re-fetch clip metadata to check if proxy is ready
        const updatedClip = await invoke<MediaClip>('get_media_metadata', { clipId });

        if (updatedClip.proxy_path) {
          // Proxy is ready! Reload video with proxy
          isLoadingProxy = false;
          currentClip = updatedClip;
          clearInterval(interval);

          if (videoElement) {
            videoElement.src = convertFileSrc(updatedClip.proxy_path);
            videoElement.load();
          }
        }
      } catch (err) {
        console.error('Error checking proxy status:', err);
      }

      if (attempts >= maxAttempts) {
        isLoadingProxy = false;
        playbackError = 'Proxy generation timed out. Using original file.';
        clearInterval(interval);
      }
    }, 1000); // Check every second
  }

  // T038: Playback controls
  function togglePlay() {
    if (!videoElement) return;

    if (isPlaying) {
      videoElement.pause();
    } else {
      videoElement.play().catch((err) => {
        playbackError = `Playback failed: ${err}`;
      });
    }
  }

  function seek(event: Event) {
    const target = event.target as HTMLInputElement;
    if (videoElement) {
      videoElement.currentTime = parseFloat(target.value);
    }
  }

  function updateVolume(event: Event) {
    const target = event.target as HTMLInputElement;
    volume = parseFloat(target.value);
    if (videoElement) {
      videoElement.volume = volume;
    }
  }

  function handleTimeUpdate() {
    if (videoElement) {
      localTime = videoElement.currentTime;

      // Use trimmed duration (out_point - in_point) instead of full video duration
      duration = clipOutPoint > 0 ? clipOutPoint - clipInPoint : videoElement.duration || 0;

      // Dispatch time updates to parent (App.svelte handles clip transitions)
      dispatch('timeupdate', { time: localTime });

      // Only pause at out_point if we're beyond it (safety check)
      // Don't pause exactly at out_point during timeline playback - let parent handle transitions
      if (clipOutPoint > 0 && localTime > clipOutPoint + 0.1) {
        videoElement.pause();
        videoElement.currentTime = clipOutPoint;
      }
    }
  }

  function handlePlay() {
    isPlaying = true;
    dispatch('playpause', { playing: true });
  }

  function handlePause() {
    isPlaying = false;
    dispatch('playpause', { playing: false });
  }

  function formatTime(seconds: number): string {
    if (!isFinite(seconds)) return '0:00';

    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  // React to currentClip changes
  $: if (currentClip) {
    loadClip(currentClip);
  }

  // Listen for clip selection from MediaLibrary
  if (typeof window !== 'undefined') {
    const handleClipSelect = (event: Event) => {
      const customEvent = event as CustomEvent<MediaClip>;
      currentClip = customEvent.detail;
    };

    window.addEventListener('clipselect', handleClipSelect);
  }
</script>

<div class="video-preview">
  <div class="video-container">
    {#if playbackError}
      <div class="error">{playbackError}</div>
    {/if}

    {#if isLoadingProxy}
      <div class="loading-overlay">
        <div class="spinner"></div>
        <p>Generating web-compatible proxy...</p>
        <p class="hint">This may take a moment for the first playback</p>
      </div>
    {/if}

    {#if !currentClip}
      <div class="placeholder">
        <p>No clip selected</p>
        <p class="hint">Select a clip from the media library to preview</p>
      </div>
    {:else}
      <video
        bind:this={videoElement}
        on:timeupdate={handleTimeUpdate}
        on:play={handlePlay}
        on:pause={handlePause}
        on:error={() => (playbackError = 'Video playback error')}
      >
        <track kind="captions" />
      </video>
    {/if}
  </div>

  <div class="controls">
    <button
      class="play-btn"
      on:click={togglePlay}
      disabled={!currentClip}
      title={isPlaying ? 'Pause' : 'Play'}
    >
      {isPlaying ? '⏸' : '▶️'}
    </button>

    <div class="timeline">
      <span class="time">{formatTime(localTime)}</span>
      <input
        type="range"
        min={clipInPoint || 0}
        max={clipOutPoint || duration}
        value={localTime}
        on:input={seek}
        disabled={!currentClip}
        class="seek-slider"
      />
      <span class="time">{formatTime(duration)}</span>
    </div>

    <div class="volume">
      <span class="volume-icon">{volume > 0.5 ? '🔊' : volume > 0 ? '🔉' : '🔇'}</span>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={volume}
        on:input={updateVolume}
        class="volume-slider"
      />
    </div>
  </div>

  {#if currentClip}
    <div class="info-bar">
      <span class="clip-name">{currentClip.name}</span>
      <span class="clip-info">{currentClip.resolution} • {Math.round(currentClip.fps)} fps</span>
    </div>
  {/if}
</div>

<style>
  .video-preview {
    display: flex;
    flex-direction: column;
    background: #1a1a1a;
    height: 100%;
  }

  .video-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #000;
    position: relative;
    min-height: 0;
  }

  video {
    max-width: 100%;
    max-height: 100%;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .placeholder {
    text-align: center;
    color: #666;
    padding: 2rem;
  }

  .placeholder p {
    margin: 0.5rem 0;
  }

  .hint {
    font-size: 0.9rem;
    color: #555;
  }

  .error {
    padding: 1rem;
    background: #d13438;
    color: white;
    border-radius: 4px;
    margin: 1rem;
    text-align: center;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: #252525;
    border-top: 1px solid #333;
  }

  .play-btn {
    width: 40px;
    height: 40px;
    background: #0078d4;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    font-size: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .play-btn:hover:not(:disabled) {
    background: #106ebe;
  }

  .play-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .timeline {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .time {
    color: #aaa;
    font-size: 0.85rem;
    font-variant-numeric: tabular-nums;
    min-width: 40px;
  }

  .seek-slider {
    flex: 1;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: #444;
    border-radius: 3px;
    outline: none;
  }

  .seek-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    background: #0078d4;
    border-radius: 50%;
    cursor: pointer;
  }

  .seek-slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    background: #0078d4;
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .volume {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .volume-icon {
    font-size: 1.2rem;
  }

  .volume-slider {
    width: 80px;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: #444;
    border-radius: 3px;
    outline: none;
  }

  .volume-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    background: #0078d4;
    border-radius: 50%;
    cursor: pointer;
  }

  .volume-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    background: #0078d4;
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .info-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: #2a2a2a;
    border-top: 1px solid #333;
    color: #aaa;
    font-size: 0.85rem;
  }

  .clip-name {
    font-weight: 500;
    color: #fff;
  }

  .clip-info {
    color: #888;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.8);
    color: #fff;
    z-index: 10;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid #333;
    border-top-color: #0078d4;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-overlay p {
    margin: 0.25rem 0;
  }

  .loading-overlay .hint {
    font-size: 0.85rem;
    color: #999;
  }
</style>
