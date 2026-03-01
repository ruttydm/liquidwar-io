import nipplejs from "nipplejs";

export const KEY_UP = 1;
export const KEY_RIGHT = 2;
export const KEY_DOWN = 4;
export const KEY_LEFT = 8;

const DEFAULT_BINDINGS: Record<string, number> = {
  ArrowUp: KEY_UP,
  ArrowDown: KEY_DOWN,
  ArrowLeft: KEY_LEFT,
  ArrowRight: KEY_RIGHT,
  KeyW: KEY_UP,
  KeyS: KEY_DOWN,
  KeyA: KEY_LEFT,
  KeyD: KEY_RIGHT,
};

const isMobile = "ontouchstart" in window || navigator.maxTouchPoints > 0;

export class InputHandler {
  public keyState = 0;
  private bindings: Record<string, number>;
  private kbBits = 0;
  private mouseBits = 0;
  private joystickBits = 0;
  private mouseActive = false;
  private mouseAnchorX = 0;
  private mouseAnchorY = 0;
  private static readonly MOUSE_GAP = 18;
  private joystickManager: nipplejs.JoystickManager | null = null;

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

    // Mobile: prevent scroll/zoom on canvas and add virtual joystick
    if (isMobile) {
      canvas.addEventListener("touchstart", (e) => e.preventDefault(), { passive: false });
      canvas.addEventListener("touchmove", (e) => e.preventDefault(), { passive: false });
      this.setupJoystick();
    }
  }

  private setupJoystick() {
    const zone = document.createElement("div");
    zone.id = "joystick-zone";
    zone.style.cssText = "position:fixed;top:0;right:0;width:50%;height:100%;z-index:60;";
    document.body.appendChild(zone);

    this.joystickManager = nipplejs.create({
      zone,
      mode: "dynamic",
      size: 120,
      color: "rgba(100,176,255,0.5)",
      threshold: 0.1,
    });

    this.joystickManager.on("move", (_evt, data) => {
      this.joystickBits = 0;
      if (data.force < 0.15) {
        this.updateState();
        return;
      }
      // nipplejs: 0°=right, 90°=up, 180°=left, 270°=down
      const deg = data.angle.degree;
      if (deg >= 22.5 && deg < 157.5)  this.joystickBits |= KEY_UP;
      if (deg >= 202.5 && deg < 337.5) this.joystickBits |= KEY_DOWN;
      if (deg >= 112.5 && deg < 247.5) this.joystickBits |= KEY_LEFT;
      if (deg >= 337.5 || deg < 67.5)  this.joystickBits |= KEY_RIGHT;
      this.updateState();
    });

    this.joystickManager.on("end", () => {
      this.joystickBits = 0;
      this.updateState();
    });
  }

  destroy() {
    if (this.joystickManager) {
      this.joystickManager.destroy();
      this.joystickManager = null;
    }
    const zone = document.getElementById("joystick-zone");
    if (zone) zone.remove();
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
    this.keyState = this.kbBits | this.mouseBits | this.joystickBits;
  }

  setBindings(bindings: Record<string, number>) {
    this.bindings = bindings;
  }
}
