<script lang="ts">
  // TODO: T038 - Wire up playback controls

  let videoElement: HTMLVideoElement;
  let isPlaying = false;
  let currentTime = 0;
  let duration = 0;

  function handlePlayPause() {
    if (isPlaying) {
      videoElement?.pause();
    } else {
      videoElement?.play();
    }
  }

  function handleTimeUpdate() {
    if (videoElement) {
      currentTime = videoElement.currentTime;
    }
  }

  function handleLoadedMetadata() {
    if (videoElement) {
      duration = videoElement.duration;
    }
  }

  function handleSeek(event: Event) {
    const input = event.target as HTMLInputElement;
    if (videoElement) {
      videoElement.currentTime = parseFloat(input.value);
    }
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div class="video-preview">
  <div class="video-preview__player">
    <video
      bind:this={videoElement}
      on:timeupdate={handleTimeUpdate}
      on:loadedmetadata={handleLoadedMetadata}
      on:play={() => (isPlaying = true)}
      on:pause={() => (isPlaying = false)}
    >
      <!-- Video source will be set dynamically -->
      <track kind="captions" />
    </video>
  </div>

  <div class="video-preview__controls">
    <button on:click={handlePlayPause} class="btn-play">
      {isPlaying ? '⏸' : '▶'}
    </button>

    <input
      type="range"
      min="0"
      max={duration || 0}
      value={currentTime}
      on:input={handleSeek}
      class="scrubber"
    />

    <div class="time-display">
      {formatTime(currentTime)} / {formatTime(duration)}
    </div>
  </div>
</div>

<style>
  .video-preview {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #000;
  }

  .video-preview__player {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #000;
  }

  video {
    max-width: 100%;
    max-height: 100%;
  }

  .video-preview__controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: #1e1e1e;
    border-top: 1px solid #333;
  }

  .btn-play {
    width: 40px;
    height: 40px;
    background: #0078d4;
    border: none;
    border-radius: 50%;
    color: white;
    font-size: 1.2rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-play:hover {
    background: #106ebe;
  }

  .scrubber {
    flex: 1;
  }

  .time-display {
    color: #fff;
    font-size: 0.9rem;
    min-width: 100px;
    text-align: right;
  }
</style>

