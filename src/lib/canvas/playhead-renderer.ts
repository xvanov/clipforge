export class PlayheadRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  /**
   * Render the playhead at the current time position
   */
  public render(
    currentTime: number,
    pixelsPerSecond: number,
    scrollLeft: number,
    canvasHeight: number
  ): void {
    const x = currentTime * pixelsPerSecond - scrollLeft;

    // Skip if playhead is outside visible area
    if (x < 0 || x > this.ctx.canvas.width) {
      return;
    }

    // Draw playhead line
    this.ctx.strokeStyle = '#ff4444';
    this.ctx.lineWidth = 2;
    this.ctx.beginPath();
    this.ctx.moveTo(x, 0);
    this.ctx.lineTo(x, canvasHeight);
    this.ctx.stroke();

    // Draw playhead handle (triangle at top)
    this.drawPlayheadHandle(x, 0);
  }

  /**
   * Draw the playhead handle (draggable triangle)
   */
  private drawPlayheadHandle(x: number, y: number): void {
    const handleSize = 8;

    this.ctx.fillStyle = '#ff4444';
    this.ctx.beginPath();
    this.ctx.moveTo(x, y + 10);
    this.ctx.lineTo(x - handleSize, y);
    this.ctx.lineTo(x + handleSize, y);
    this.ctx.closePath();
    this.ctx.fill();

    // Add handle border
    this.ctx.strokeStyle = '#ffffff';
    this.ctx.lineWidth = 1;
    this.ctx.stroke();
  }

  /**
   * Check if a point is near the playhead (for drag detection)
   */
  public isNearPlayhead(
    mouseX: number,
    currentTime: number,
    pixelsPerSecond: number,
    scrollLeft: number,
    threshold: number = 10
  ): boolean {
    const playheadX = currentTime * pixelsPerSecond - scrollLeft;
    return Math.abs(mouseX - playheadX) < threshold;
  }

  /**
   * Render playhead with time label
   */
  public renderWithLabel(
    currentTime: number,
    pixelsPerSecond: number,
    scrollLeft: number,
    canvasHeight: number,
    showLabel: boolean = true
  ): void {
    this.render(currentTime, pixelsPerSecond, scrollLeft, canvasHeight);

    if (showLabel) {
      const x = currentTime * pixelsPerSecond - scrollLeft;
      this.drawTimeLabel(x, 20, currentTime);
    }
  }

  /**
   * Draw time label above playhead
   */
  private drawTimeLabel(x: number, y: number, time: number): void {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    const milliseconds = Math.floor((time % 1) * 100);
    const label = `${minutes}:${seconds.toString().padStart(2, '0')}.${milliseconds.toString().padStart(2, '0')}`;

    // Measure text
    this.ctx.font = '11px monospace';
    const metrics = this.ctx.measureText(label);
    const padding = 4;
    const labelWidth = metrics.width + padding * 2;
    const labelHeight = 16;

    // Draw label background
    this.ctx.fillStyle = 'rgba(255, 68, 68, 0.9)';
    this.ctx.fillRect(x - labelWidth / 2, y, labelWidth, labelHeight);

    // Draw label text
    this.ctx.fillStyle = '#ffffff';
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'top';
    this.ctx.fillText(label, x, y + padding);
  }

  /**
   * Render playhead shadow (for preview while dragging)
   */
  public renderShadow(
    time: number,
    pixelsPerSecond: number,
    scrollLeft: number,
    canvasHeight: number
  ): void {
    const x = time * pixelsPerSecond - scrollLeft;

    // Draw shadow line
    this.ctx.strokeStyle = 'rgba(255, 68, 68, 0.3)';
    this.ctx.lineWidth = 2;
    this.ctx.setLineDash([5, 5]);
    this.ctx.beginPath();
    this.ctx.moveTo(x, 0);
    this.ctx.lineTo(x, canvasHeight);
    this.ctx.stroke();
    this.ctx.setLineDash([]);
  }
}

