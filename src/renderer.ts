// LW5 team colors (CSS strings for overlay drawing)
const TEAM_COLORS_CSS = [
  "#4287f5", // Blue
  "#f54242", // Red
  "#42f560", // Green
  "#f5d742", // Yellow
  "#dc42f5", // Purple
  "#42f5e6", // Cyan
];

// Hue rotation in degrees to tint the red cursor sprite per team
const TEAM_HUE_ROTATE = [
  220,  // Blue
  0,    // Red (original)
  100,  // Green
  50,   // Yellow
  280,  // Purple
  170,  // Cyan
];

const VERT_SRC = `#version 300 es
in vec2 a_pos;
out vec2 v_uv;
void main() {
  v_uv = a_pos * 0.5 + 0.5;
  v_uv.y = 1.0 - v_uv.y;
  gl_Position = vec4(a_pos, 0.0, 1.0);
}`;

const FRAG_SRC = `#version 300 es
precision mediump float;
uniform sampler2D u_bitmap;
uniform sampler2D u_wallMap;
uniform sampler2D u_bgTexture;
uniform sampler2D u_fgTexture;
uniform vec2 u_texScale;
uniform int u_hasTextures;
in vec2 v_uv;
out vec4 fragColor;

const vec3 teamColors[6] = vec3[6](
  vec3(0.26, 0.53, 0.96),
  vec3(0.96, 0.26, 0.26),
  vec3(0.26, 0.96, 0.38),
  vec3(0.96, 0.84, 0.26),
  vec3(0.86, 0.26, 0.96),
  vec3(0.26, 0.96, 0.90)
);
const vec3 wallColor = vec3(0.16, 0.16, 0.20);
const vec3 bgColor = vec3(0.06, 0.06, 0.10);

void main() {
  float raw = texture(u_bitmap, v_uv).r * 255.0;
  int byte = int(raw + 0.5);

  if (byte < 254) {
    // Fighter pixel: team color with health brightness
    int team = (byte >> 4) & 0xF;
    int health = byte & 0xF;
    float brightness = 0.4 + 0.6 * float(health) / 15.0;
    vec3 color = teamColors[team % 6] * brightness;
    fragColor = vec4(color, 1.0);
  } else if (u_hasTextures == 1) {
    // Textured mode: use wall map to distinguish wall vs floor
    float wallVal = texture(u_wallMap, v_uv).r;
    vec2 tiledUV = fract(v_uv * u_texScale);
    if (wallVal > 0.5) {
      fragColor = texture(u_bgTexture, tiledUV);
    } else {
      fragColor = texture(u_fgTexture, tiledUV);
    }
  } else {
    // Flat color fallback
    if (byte == 254) {
      fragColor = vec4(wallColor, 1.0);
    } else {
      fragColor = vec4(bgColor, 1.0);
    }
  }
}`;

export class Renderer {
  private gl: WebGL2RenderingContext;
  private prog: WebGLProgram;
  private bitmapTexture: WebGLTexture;
  private wallMapTexture: WebGLTexture;
  private bgTexture: WebGLTexture;
  private fgTexture: WebGLTexture;
  private mapWidth: number;
  private mapHeight: number;
  private canvas: HTMLCanvasElement;

  // Overlay canvas for cursors and HUD
  private overlay: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private cursorImg: HTMLImageElement;
  private cursorLoaded = false;

  private hasTextures = false;
  private uHasTextures: WebGLUniformLocation | null = null;
  private uTexScale: WebGLUniformLocation | null = null;

  constructor(canvas: HTMLCanvasElement, mapWidth: number, mapHeight: number) {
    this.canvas = canvas;
    this.mapWidth = mapWidth;
    this.mapHeight = mapHeight;

    const gl = canvas.getContext("webgl2", { antialias: false })!;
    this.gl = gl;

    this.resize();

    // Compile shaders
    const vs = this.compileShader(gl.VERTEX_SHADER, VERT_SRC);
    const fs = this.compileShader(gl.FRAGMENT_SHADER, FRAG_SRC);
    const prog = gl.createProgram()!;
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    gl.useProgram(prog);
    this.prog = prog;

    // Fullscreen quad
    const quad = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);
    const vbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, quad, gl.STATIC_DRAW);
    const aPos = gl.getAttribLocation(prog, "a_pos");
    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0);

    // Bitmap texture (unit 0)
    this.bitmapTexture = this.createR8Texture(0);
    gl.uniform1i(gl.getUniformLocation(prog, "u_bitmap"), 0);

    // Wall map texture (unit 1)
    this.wallMapTexture = this.createR8Texture(1);
    gl.uniform1i(gl.getUniformLocation(prog, "u_wallMap"), 1);

    // BG texture placeholder (unit 2)
    this.bgTexture = this.createRGBATexture(2);
    gl.uniform1i(gl.getUniformLocation(prog, "u_bgTexture"), 2);

    // FG texture placeholder (unit 3)
    this.fgTexture = this.createRGBATexture(3);
    gl.uniform1i(gl.getUniformLocation(prog, "u_fgTexture"), 3);

    this.uHasTextures = gl.getUniformLocation(prog, "u_hasTextures");
    this.uTexScale = gl.getUniformLocation(prog, "u_texScale");
    gl.uniform1i(this.uHasTextures, 0);
    gl.uniform2f(this.uTexScale!, 1.0, 1.0);

    // Create overlay canvas for cursors
    this.overlay = document.createElement("canvas");
    this.overlay.style.position = "absolute";
    this.overlay.style.pointerEvents = "none";
    this.overlay.style.imageRendering = "pixelated";
    canvas.parentElement?.appendChild(this.overlay);
    this.ctx = this.overlay.getContext("2d")!;
    this.syncOverlay();

    // Load cursor sprite
    this.cursorImg = new Image();
    this.cursorImg.src = "/assets/cursor.png";
    this.cursorImg.onload = () => { this.cursorLoaded = true; };

    window.addEventListener("resize", () => {
      this.resize();
      this.syncOverlay();
    });
  }

  private createR8Texture(unit: number): WebGLTexture {
    const gl = this.gl;
    const tex = gl.createTexture()!;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return tex;
  }

  private createRGBATexture(unit: number): WebGLTexture {
    const gl = this.gl;
    const tex = gl.createTexture()!;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
    // Init with 1x1 pixel
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array([0, 0, 0, 255]));
    return tex;
  }

  private syncOverlay() {
    this.overlay.width = this.canvas.width;
    this.overlay.height = this.canvas.height;
    this.overlay.style.width = this.canvas.style.width;
    this.overlay.style.height = this.canvas.style.height;
    // Match position of the game canvas
    const rect = this.canvas.getBoundingClientRect();
    this.overlay.style.left = rect.left + "px";
    this.overlay.style.top = rect.top + "px";
  }

  private resize() {
    const aspect = this.mapWidth / this.mapHeight;
    let w = window.innerWidth;
    let h = window.innerHeight;

    if (w / h > aspect) {
      w = Math.floor(h * aspect);
    } else {
      h = Math.floor(w / aspect);
    }

    this.canvas.width = w;
    this.canvas.height = h;
    this.canvas.style.width = w + "px";
    this.canvas.style.height = h + "px";
    this.gl.viewport(0, 0, w, h);
  }

  private compileShader(type: number, src: string): WebGLShader {
    const gl = this.gl;
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, src);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      throw new Error(gl.getShaderInfoLog(shader)!);
    }
    return shader;
  }

  /** Upload map passability data as wall map texture */
  setMapData(mapData: number[]) {
    const gl = this.gl;
    // Convert: 0=passable→0, 1=wall→255
    const data = new Uint8Array(mapData.length);
    for (let i = 0; i < mapData.length; i++) {
      data[i] = mapData[i] ? 255 : 0;
    }
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.wallMapTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, this.mapWidth, this.mapHeight, 0, gl.RED, gl.UNSIGNED_BYTE, data);
  }

  /** Load and set background (wall) and foreground (floor) textures */
  async setTextures(bgUrl: string, fgUrl: string) {
    const gl = this.gl;

    const loadImg = (url: string): Promise<HTMLImageElement> =>
      new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = reject;
        img.src = url;
      });

    try {
      const [bgImg, fgImg] = await Promise.all([loadImg(bgUrl), loadImg(fgUrl)]);

      // Upload BG texture (walls)
      gl.activeTexture(gl.TEXTURE2);
      gl.bindTexture(gl.TEXTURE_2D, this.bgTexture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, bgImg);

      // Upload FG texture (floors)
      gl.activeTexture(gl.TEXTURE3);
      gl.bindTexture(gl.TEXTURE_2D, this.fgTexture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, fgImg);

      // Compute tiling scale
      const scaleX = this.mapWidth / bgImg.width;
      const scaleY = this.mapHeight / bgImg.height;
      gl.useProgram(this.prog);
      gl.uniform2f(this.uTexScale!, scaleX, scaleY);
      gl.uniform1i(this.uHasTextures, 1);
      this.hasTextures = true;
    } catch {
      // Keep flat color fallback
    }
  }

  render(bitmap: Uint8Array, cursors: Array<[number, number] | null>, scores?: number[]) {
    const gl = this.gl;

    gl.useProgram(this.prog);

    // Upload bitmap as R8 texture
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.bitmapTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, this.mapWidth, this.mapHeight, 0, gl.RED, gl.UNSIGNED_BYTE, bitmap);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

    // Draw overlay (cursors + info bar)
    this.syncOverlay();
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.overlay.width, this.overlay.height);

    this.drawCursors(ctx, cursors);
    if (scores) {
      this.drawInfoBar(ctx, scores);
    }
  }

  private drawCursors(ctx: CanvasRenderingContext2D, cursors: Array<[number, number] | null>) {
    if (!this.cursorLoaded) return;

    const scaleX = this.canvas.width / this.mapWidth;
    const scaleY = this.canvas.height / this.mapHeight;
    const cursorSize = 24; // Fixed screen pixels

    for (let i = 0; i < cursors.length; i++) {
      const c = cursors[i];
      if (!c) continue;
      const [cx, cy] = c;
      const px = cx * scaleX;
      const py = cy * scaleY;

      ctx.save();
      // Tint cursor to team color via hue rotation
      ctx.filter = `hue-rotate(${TEAM_HUE_ROTATE[i % 6]}deg) saturate(1.5) brightness(1.2)`;
      ctx.drawImage(this.cursorImg, px, py, cursorSize, cursorSize);
      ctx.restore();
    }
  }

  private drawInfoBar(ctx: CanvasRenderingContext2D, scores: number[]) {
    const total = scores.reduce((a, b) => a + b, 0);
    if (total === 0) return;

    const barW = 16;
    const barX = 6;
    const barY = 6;
    const barH = this.overlay.height - 12;

    // Background
    ctx.fillStyle = "rgba(0, 0, 0, 0.4)";
    ctx.fillRect(barX - 2, barY - 2, barW + 4, barH + 4);

    // Team segments
    let currentY = barY;
    for (let i = 0; i < scores.length; i++) {
      if (scores[i] === 0) continue;
      const pct = scores[i] / total;
      const segH = pct * barH;
      ctx.fillStyle = TEAM_COLORS_CSS[i % 6];
      ctx.fillRect(barX, currentY, barW, segH);
      currentY += segH;
    }
  }
}
