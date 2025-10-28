import type { TimelineClip } from '../types/timeline';

export class ClipRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  /**
   * Render a timeline clip
   */
  public render(
    clip: TimelineClip,
    pixelsPerSecond: number,
    scrollLeft: number,
    trackY: number,
    trackHeight: number
  ): void {
    const duration = clip.out_point - clip.in_point;
    const x = clip.start_time * pixelsPerSecond - scrollLeft;
    const width = duration * pixelsPerSecond;
    const y = trackY + 10;
    const height = trackHeight - 20;

    // Skip if clip is outside visible area
    if (x + width < 0 || x > this.ctx.canvas.width) {
      return;
    }

    // Draw clip background
    const gradient = this.ctx.createLinearGradient(x, y, x, y + height);
    gradient.addColorStop(0, '#3a3a7a');
    gradient.addColorStop(1, '#2a2a5a');
    this.ctx.fillStyle = gradient;
    this.ctx.fillRect(x, y, width, height);

    // Draw clip border
    this.ctx.strokeStyle = '#4a4a8a';
    this.ctx.lineWidth = 2;
    this.ctx.strokeRect(x, y, width, height);

    // Draw clip label
    if (width > 50) {
      this.ctx.fillStyle = '#ffffff';
      this.ctx.font = '12px sans-serif';
      this.ctx.textAlign = 'left';
      this.ctx.textBaseline = 'top';
      
      const clipName = clip.id.substring(0, 8) + '...';
      this.ctx.fillText(clipName, x + 8, y + 8);

      // Draw duration
      this.ctx.fillStyle = '#aaaaaa';
      this.ctx.font = '10px sans-serif';
      this.ctx.fillText(`${duration.toFixed(2)}s`, x + 8, y + 25);
    }

    // Draw trim handles
    this.drawTrimHandles(x, y, width, height);

    // Draw waveform placeholder (if clip has audio)
    // TODO: Implement waveform rendering
  }

  /**
   * Draw trim handles at clip edges
   */
  private drawTrimHandles(x: number, y: number, width: number, height: number): void {
    const handleWidth = 10;

    // Left handle
    this.ctx.fillStyle = 'rgba(255, 255, 255, 0.1)';
    this.ctx.fillRect(x, y, handleWidth, height);

    // Right handle
    this.ctx.fillRect(x + width - handleWidth, y, handleWidth, height);
  }

  /**
   * Render clip selection highlight
   */
  public renderSelection(
    clip: TimelineClip,
    pixelsPerSecond: number,
    scrollLeft: number,
    trackY: number,
    trackHeight: number
  ): void {
    const duration = clip.out_point - clip.in_point;
    const x = clip.start_time * pixelsPerSecond - scrollLeft;
    const width = duration * pixelsPerSecond;
    const y = trackY + 10;
    const height = trackHeight - 20;

    // Draw selection border
    this.ctx.strokeStyle = '#6a6aaa';
    this.ctx.lineWidth = 3;
    this.ctx.strokeRect(x - 2, y - 2, width + 4, height + 4);
  }

  /**
   * Render clip with overlay transform
   */
  public renderOverlay(
    clip: TimelineClip,
    pixelsPerSecond: number,
    scrollLeft: number,
    trackY: number,
    trackHeight: number,
    transform?: { x: number; y: number; width: number; height: number; rotation: number }
  ): void {
    if (!transform) {
      this.render(clip, pixelsPerSecond, scrollLeft, trackY, trackHeight);
      return;
    }

    // Save context state
    this.ctx.save();

    const duration = clip.out_point - clip.in_point;
    const x = clip.start_time * pixelsPerSecond - scrollLeft;
    const width = duration * pixelsPerSecond;
    const y = trackY + 10;
    const height = trackHeight - 20;

    // Apply transformations
    const centerX = x + width / 2;
    const centerY = y + height / 2;

    this.ctx.translate(centerX, centerY);
    this.ctx.rotate((transform.rotation * Math.PI) / 180);
    this.ctx.scale(transform.width / width, transform.height / height);
    this.ctx.translate(-centerX, -centerY);

    // Render clip
    this.render(clip, pixelsPerSecond, scrollLeft, trackY, trackHeight);

    // Restore context state
    this.ctx.restore();
  }
}

