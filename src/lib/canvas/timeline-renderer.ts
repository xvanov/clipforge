import type { Track } from '../types/timeline';
import { ClipRenderer } from './clip-renderer';
import { PlayheadRenderer } from './playhead-renderer';

export class TimelineRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private clipRenderer: ClipRenderer;
  private playheadRenderer: PlayheadRenderer;
  private animationFrameId: number | null = null;
  private isRendering: boolean = false;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Failed to get 2D context from canvas');
    }
    this.ctx = context;
    this.clipRenderer = new ClipRenderer(this.ctx);
    this.playheadRenderer = new PlayheadRenderer(this.ctx);
  }

  /**
   * Start the rendering loop
   */
  public start(): void {
    if (this.isRendering) return;
    this.isRendering = true;
    this.render();
  }

  /**
   * Stop the rendering loop
   */
  public stop(): void {
    this.isRendering = false;
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  /**
   * Main render loop using requestAnimationFrame
   */
  private render = (): void => {
    if (!this.isRendering) return;

    // Request next frame
    this.animationFrameId = requestAnimationFrame(this.render);
  };

  /**
   * Render timeline with tracks and clips
   */
  public renderTimeline(
    tracks: Track[],
    currentTime: number,
    pixelsPerSecond: number,
    scrollLeft: number
  ): void {
    // Clear canvas
    this.clear();

    // Draw timeline background
    this.drawBackground();

    // Draw time ruler
    this.drawTimeRuler(currentTime, pixelsPerSecond, scrollLeft);

    // Draw tracks and clips
    this.drawTracks(tracks, pixelsPerSecond, scrollLeft);

    // Draw playhead on top
    this.playheadRenderer.render(currentTime, pixelsPerSecond, scrollLeft, this.canvas.height);
  }

  /**
   * Clear the canvas
   */
  private clear(): void {
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
  }

  /**
   * Draw timeline background
   */
  private drawBackground(): void {
    this.ctx.fillStyle = '#0a0a0a';
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
  }

  /**
   * Draw time ruler with markers
   */
  private drawTimeRuler(
    currentTime: number,
    pixelsPerSecond: number,
    scrollLeft: number
  ): void {
    this.ctx.strokeStyle = '#444';
    this.ctx.fillStyle = '#999';
    this.ctx.font = '10px sans-serif';

    const rulerHeight = 30;
    const startTime = scrollLeft / pixelsPerSecond;
    const endTime = (scrollLeft + this.canvas.width) / pixelsPerSecond;

    // Draw ruler background
    this.ctx.fillStyle = '#1a1a1a';
    this.ctx.fillRect(0, 0, this.canvas.width, rulerHeight);

    // Draw time markers
    const majorInterval = this.calculateMajorInterval(pixelsPerSecond);
    const minorInterval = majorInterval / 5;

    this.ctx.strokeStyle = '#444';
    this.ctx.fillStyle = '#999';

    for (let t = Math.floor(startTime / minorInterval) * minorInterval; t <= endTime; t += minorInterval) {
      const x = t * pixelsPerSecond - scrollLeft;

      if (Math.abs(t % majorInterval) < 0.001) {
        // Major marker
        this.ctx.beginPath();
        this.ctx.moveTo(x, rulerHeight - 20);
        this.ctx.lineTo(x, rulerHeight);
        this.ctx.stroke();

        // Time label
        const minutes = Math.floor(t / 60);
        const seconds = Math.floor(t % 60);
        this.ctx.fillText(`${minutes}:${seconds.toString().padStart(2, '0')}`, x + 2, rulerHeight - 25);
      } else {
        // Minor marker
        this.ctx.beginPath();
        this.ctx.moveTo(x, rulerHeight - 10);
        this.ctx.lineTo(x, rulerHeight);
        this.ctx.stroke();
      }
    }

    // Draw ruler border
    this.ctx.strokeStyle = '#333';
    this.ctx.beginPath();
    this.ctx.moveTo(0, rulerHeight);
    this.ctx.lineTo(this.canvas.width, rulerHeight);
    this.ctx.stroke();
  }

  /**
   * Calculate appropriate marker interval based on zoom level
   */
  private calculateMajorInterval(pixelsPerSecond: number): number {
    if (pixelsPerSecond > 100) return 1; // 1 second
    if (pixelsPerSecond > 50) return 5; // 5 seconds
    if (pixelsPerSecond > 20) return 10; // 10 seconds
    if (pixelsPerSecond > 10) return 30; // 30 seconds
    return 60; // 1 minute
  }

  /**
   * Draw all tracks
   */
  private drawTracks(tracks: Track[], pixelsPerSecond: number, scrollLeft: number): void {
    const rulerHeight = 30;
    const trackHeight = 80;
    let yOffset = rulerHeight;

    for (const track of tracks) {
      if (!track.visible) continue;

      // Draw track background
      this.ctx.fillStyle = '#0a0a0a';
      this.ctx.fillRect(0, yOffset, this.canvas.width, trackHeight);

      // Draw track border
      this.ctx.strokeStyle = '#333';
      this.ctx.beginPath();
      this.ctx.moveTo(0, yOffset + trackHeight);
      this.ctx.lineTo(this.canvas.width, yOffset + trackHeight);
      this.ctx.stroke();

      // Draw clips on track
      for (const clip of track.clips) {
        this.clipRenderer.render(clip, pixelsPerSecond, scrollLeft, yOffset, trackHeight);
      }

      yOffset += trackHeight;
    }
  }

  /**
   * Resize canvas to match container
   */
  public resize(width: number, height: number): void {
    this.canvas.width = width;
    this.canvas.height = height;
  }

  /**
   * Convert pixel position to time
   */
  public pixelToTime(x: number, pixelsPerSecond: number, scrollLeft: number): number {
    return (x + scrollLeft) / pixelsPerSecond;
  }

  /**
   * Convert time to pixel position
   */
  public timeToPixel(time: number, pixelsPerSecond: number, scrollLeft: number): number {
    return time * pixelsPerSecond - scrollLeft;
  }
}

