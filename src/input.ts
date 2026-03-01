export class InputHandler {
  public cursorX = 0;
  public cursorY = 0;
  private scale: number;

  constructor(
    private canvas: HTMLCanvasElement,
    private mapWidth: number,
    private mapHeight: number,
  ) {
    this.scale = Math.min(canvas.width / mapWidth, canvas.height / mapHeight);

    canvas.addEventListener("mousemove", (e) =>
      this.handleMove(e.offsetX, e.offsetY),
    );

    canvas.addEventListener(
      "touchmove",
      (e) => {
        e.preventDefault();
        const touch = e.touches[0];
        const rect = canvas.getBoundingClientRect();
        this.handleMove(touch.clientX - rect.left, touch.clientY - rect.top);
      },
      { passive: false },
    );
  }

  updateScale() {
    this.scale = Math.min(
      this.canvas.width / this.mapWidth,
      this.canvas.height / this.mapHeight,
    );
  }

  private handleMove(pixelX: number, pixelY: number) {
    // Account for CSS scaling
    const rect = this.canvas.getBoundingClientRect();
    const scaleX = this.canvas.width / rect.width;
    const scaleY = this.canvas.height / rect.height;
    pixelX *= scaleX;
    pixelY *= scaleY;

    this.cursorX = Math.floor(pixelX / this.scale);
    this.cursorY = Math.floor(pixelY / this.scale);
    this.cursorX = Math.max(0, Math.min(this.mapWidth - 1, this.cursorX));
    this.cursorY = Math.max(0, Math.min(this.mapHeight - 1, this.cursorY));
  }
}
