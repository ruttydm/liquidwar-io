export const KEY_UP = 1;
export const KEY_RIGHT = 2;
export const KEY_DOWN = 4;
export const KEY_LEFT = 8;

const DEFAULT_BINDINGS: Record<string, number> = {
  ArrowUp: KEY_UP,
  ArrowDown: KEY_DOWN,
  ArrowLeft: KEY_LEFT,
  ArrowRight: KEY_RIGHT,
};

export class InputHandler {
  public keyState = 0;
  private bindings: Record<string, number>;
  private kbBits = 0;
  private mouseBits = 0;
  private mouseActive = false;
  private mouseAnchorX = 0;
  private mouseAnchorY = 0;
  private static readonly MOUSE_GAP = 18;

  constructor(canvas: HTMLCanvasElement) {
    this.bindings = { ...DEFAULT_BINDINGS };

    document.addEventListener("keydown", (e) => this.onKeyDown(e));
    document.addEventListener("keyup", (e) => this.onKeyUp(e));

    // LW5-style mouse gap mode: hold click, move away from anchor to steer
    canvas.addEventListener("mousedown", (e) => {
      this.mouseActive = true;
      this.mouseAnchorX = e.clientX;
      this.mouseAnchorY = e.clientY;
      this.mouseBits = 0;
      this.updateState();
    });
    document.addEventListener("mouseup", () => {
      this.mouseActive = false;
      this.mouseBits = 0;
      this.updateState();
    });
    document.addEventListener("mousemove", (e) => {
      if (!this.mouseActive) return;
      const dx = e.clientX - this.mouseAnchorX;
      const dy = e.clientY - this.mouseAnchorY;
      this.mouseBits = 0;
      if (dy < -InputHandler.MOUSE_GAP) this.mouseBits |= KEY_UP;
      if (dy > InputHandler.MOUSE_GAP) this.mouseBits |= KEY_DOWN;
      if (dx < -InputHandler.MOUSE_GAP) this.mouseBits |= KEY_LEFT;
      if (dx > InputHandler.MOUSE_GAP) this.mouseBits |= KEY_RIGHT;
      this.updateState();
    });
  }

  private onKeyDown(e: KeyboardEvent) {
    const bit = this.bindings[e.code] ?? this.bindings[e.key];
    if (bit !== undefined) {
      e.preventDefault();
      this.kbBits |= bit;
      this.updateState();
    }
  }

  private onKeyUp(e: KeyboardEvent) {
    const bit = this.bindings[e.code] ?? this.bindings[e.key];
    if (bit !== undefined) {
      e.preventDefault();
      this.kbBits &= ~bit;
      this.updateState();
    }
  }

  private updateState() {
    this.keyState = this.kbBits | this.mouseBits;
  }

  setBindings(bindings: Record<string, number>) {
    this.bindings = bindings;
  }
}
