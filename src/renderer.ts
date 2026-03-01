// Generate 32 team colors procedurally via HSL
// First 6 match the original LW5 colors, rest are evenly spaced hues
function generateTeamColors(count: number): { css: string[]; hueRotate: number[]; rgb: Array<[number, number, number]> } {
  const baseRGB: Array<[number, number, number]> = [
    [0.26, 0.53, 0.96], // Blue
    [0.96, 0.26, 0.26], // Red
    [0.26, 0.96, 0.38], // Green
    [0.96, 0.84, 0.26], // Yellow
    [0.86, 0.26, 0.96], // Purple
    [0.26, 0.96, 0.90], // Cyan
  ];
  const baseHue = [220, 0, 100, 50, 280, 170];

  const css: string[] = [];
  const hueRotate: number[] = [];
  const rgb: Array<[number, number, number]> = [];

  for (let i = 0; i < count; i++) {
    if (i < 6) {
      const [r, g, b] = baseRGB[i];
      css.push(`rgb(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(b * 255)})`);
      hueRotate.push(baseHue[i]);
      rgb.push(baseRGB[i]);
    } else {
      // Spread hues evenly, offset to avoid clashing with first 6
      const hue = ((i - 6) * 360 / (count - 6) + 15) % 360;
      const s = 0.75, l = 0.55;
      // HSL to RGB
      const c = (1 - Math.abs(2 * l - 1)) * s;
      const x = c * (1 - Math.abs(((hue / 60) % 2) - 1));
      const m = l - c / 2;
      let r = 0, g = 0, b = 0;
      if (hue < 60) { r = c; g = x; }
      else if (hue < 120) { r = x; g = c; }
      else if (hue < 180) { g = c; b = x; }
      else if (hue < 240) { g = x; b = c; }
      else if (hue < 300) { r = x; b = c; }
      else { r = c; b = x; }
      r += m; g += m; b += m;
      css.push(`rgb(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(b * 255)})`);
      hueRotate.push(Math.round(hue));
      rgb.push([r, g, b]);
    }
  }
  return { css, hueRotate, rgb };
}

const TEAM_DATA = generateTeamColors(32);
const TEAM_COLORS_CSS = TEAM_DATA.css;
const TEAM_HUE_ROTATE = TEAM_DATA.hueRotate;

const VERT_SRC = `#version 300 es
in vec2 a_pos;
out vec2 v_uv;
void main() {
  v_uv = a_pos * 0.5 + 0.5;
  v_uv.y = 1.0 - v_uv.y;
  gl_Position = vec4(a_pos, 0.0, 1.0);
}`;

// Build GLSL team color array from generated colors
function buildFragShader(): string {
  let colorArray = "";
  for (let i = 0; i < 32; i++) {
    const [r, g, b] = TEAM_DATA.rgb[i];
    colorArray += `  vec3(${r.toFixed(3)}, ${g.toFixed(3)}, ${b.toFixed(3)})${i < 31 ? "," : ""}\n`;
  }
  return `#version 300 es
precision mediump float;
uniform sampler2D u_bitmap;
uniform sampler2D u_wallMap;
uniform sampler2D u_bgTexture;
uniform sampler2D u_fgTexture;
uniform vec2 u_texScale;
uniform int u_hasTextures;
in vec2 v_uv;
out vec4 fragColor;

const vec3 teamColors[32] = vec3[32](
${colorArray});
const vec3 wallColor = vec3(0.16, 0.16, 0.20);
const vec3 bgColor = vec3(0.06, 0.06, 0.10);

void main() {
  float raw = texture(u_bitmap, v_uv).r * 255.0;
  int byte = int(raw + 0.5);

  // Bitmap encoding: 0=empty, 254=wall, 1-224=fighter (team*7+health+1)
  if (byte >= 1 && byte <= 224) {
    int v = byte - 1;
    int team = v / 7;
    int health = v - team * 7;
    float brightness = 0.4 + 0.6 * float(health) / 6.0;
    vec3 color = teamColors[team] * brightness;
    fragColor = vec4(color, 1.0);
  } else if (byte == 254) {
    if (u_hasTextures == 1) {
      float wallVal = texture(u_wallMap, v_uv).r;
      vec2 tiledUV = fract(v_uv * u_texScale);
      if (wallVal > 0.5) {
        fragColor = texture(u_bgTexture, tiledUV);
      } else {
        fragColor = texture(u_fgTexture, tiledUV);
      }
    } else {
      fragColor = vec4(wallColor, 1.0);
    }
  } else {
    // Empty pixel (0 or 255)
    if (u_hasTextures == 1) {
      float wallVal = texture(u_wallMap, v_uv).r;
      vec2 tiledUV = fract(v_uv * u_texScale);
      if (wallVal > 0.5) {
        fragColor = texture(u_bgTexture, tiledUV);
      } else {
        fragColor = texture(u_fgTexture, tiledUV);
      }
    } else {
      fragColor = vec4(bgColor, 1.0);
    }
  }
}`;
}

const FRAG_SRC = buildFragShader();

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

  // Overlay canvas for cursors
  private overlay: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private cursorImg: HTMLImageElement;
  private cursorLoaded = false;

  // Info bar canvas (bottom, horizontal)
  private infoBar: HTMLCanvasElement;
  private infoCtx: CanvasRenderingContext2D;
  private static readonly BAR_H = 28;

  private teamNames: string[] = [];
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

    // Create info bar canvas (fixed at bottom, full width)
    this.infoBar = document.createElement("canvas");
    this.infoBar.style.position = "fixed";
    this.infoBar.style.bottom = "0";
    this.infoBar.style.left = "0";
    this.infoBar.style.width = "100%";
    this.infoBar.style.height = Renderer.BAR_H + "px";
    this.infoBar.style.zIndex = "10";
    this.infoBar.style.pointerEvents = "none";
    this.infoBar.width = window.innerWidth;
    this.infoBar.height = Renderer.BAR_H;
    document.body.appendChild(this.infoBar);
    this.infoCtx = this.infoBar.getContext("2d")!;

    // Load cursor sprite
    this.cursorImg = new Image();
    this.cursorImg.src = "/assets/cursor.png";
    this.cursorImg.onload = () => { this.cursorLoaded = true; };

    window.addEventListener("resize", () => {
      this.resize();
      this.syncOverlay();
      this.infoBar.width = window.innerWidth;
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
    const barH = Renderer.BAR_H;
    const aspect = this.mapWidth / this.mapHeight;
    let w = window.innerWidth;
    let h = window.innerHeight - barH; // reserve space for bottom info bar

    if (w / h > aspect) {
      w = Math.floor(h * aspect);
    } else {
      h = Math.floor(w / aspect);
    }

    this.canvas.width = w;
    this.canvas.height = h;
    this.canvas.style.width = w + "px";
    this.canvas.style.height = h + "px";

    // Position canvas centered in the space above the info bar
    const left = Math.floor((window.innerWidth - w) / 2);
    const top = Math.floor((window.innerHeight - barH - h) / 2);
    this.canvas.style.position = "fixed";
    this.canvas.style.left = left + "px";
    this.canvas.style.top = top + "px";

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

      // Tile the wall texture as the page background (fills margins around map)
      // Match the in-map tiling scale: each texture tile = texPx * (canvasPx / mapPx)
      const tileSizeX = Math.round(bgImg.width * this.canvas.width / this.mapWidth);
      const tileSizeY = Math.round(bgImg.height * this.canvas.height / this.mapHeight);
      document.body.style.backgroundImage = `url('${bgUrl}')`;
      document.body.style.backgroundRepeat = "repeat";
      document.body.style.backgroundSize = `${tileSizeX}px ${tileSizeY}px`;
    } catch (e) {
      console.error("Failed to load textures:", e);
    }
  }

  setTeamNames(names: string[]) {
    this.teamNames = names;
  }

  destroy() {
    this.infoBar.remove();
    this.overlay.remove();
    // Reset canvas positioning
    this.canvas.style.position = "";
    this.canvas.style.left = "";
    this.canvas.style.top = "";
    // Reset body background
    document.body.style.backgroundImage = "";
    document.body.style.backgroundRepeat = "";
    document.body.style.backgroundSize = "";
  }

  render(bitmap: Uint8Array, cursors: Array<[number, number] | null>, scores?: number[]) {
    const gl = this.gl;

    gl.useProgram(this.prog);

    // Re-bind all texture units defensively
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.wallMapTexture);
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.bgTexture);
    gl.activeTexture(gl.TEXTURE3);
    gl.bindTexture(gl.TEXTURE_2D, this.fgTexture);

    // Upload bitmap as R8 texture
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.bitmapTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, this.mapWidth, this.mapHeight, 0, gl.RED, gl.UNSIGNED_BYTE, bitmap);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

    // Draw overlay (cursors)
    this.syncOverlay();
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.overlay.width, this.overlay.height);
    this.drawCursors(ctx, cursors);

    // Draw info bar on separate top canvas
    if (scores) {
      this.infoCtx.clearRect(0, 0, this.infoBar.width, this.infoBar.height);
      this.drawInfoBar(this.infoCtx, scores);
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
      ctx.filter = `hue-rotate(${TEAM_HUE_ROTATE[i % 32]}deg) saturate(1.5) brightness(1.2)`;
      ctx.drawImage(this.cursorImg, px, py, cursorSize, cursorSize);
      ctx.restore();
    }
  }

  private drawInfoBar(ctx: CanvasRenderingContext2D, scores: number[]) {
    const total = scores.reduce((a, b) => a + b, 0);
    if (total === 0) return;

    const active: { idx: number; pct: number }[] = [];
    for (let i = 0; i < scores.length; i++) {
      if (scores[i] > 0) active.push({ idx: i, pct: scores[i] / total });
    }
    if (active.length === 0) return;

    const W = this.infoBar.width;
    const barH = Renderer.BAR_H;

    // Dark background
    ctx.fillStyle = "rgba(0, 0, 20, 0.85)";
    ctx.fillRect(0, 0, W, barH);

    // Horizontal stacked colored segments
    let curX = 0;
    for (const { idx, pct } of active) {
      const segW = pct * W;
      ctx.fillStyle = TEAM_COLORS_CSS[idx % 32];
      ctx.fillRect(curX, 0, segW, barH);

      const fontSize = Math.max(7, Math.min(11, Math.floor(barH * 0.4)));
      ctx.font = `${fontSize}px 'Press Start 2P', monospace`;
      ctx.textBaseline = "middle";
      ctx.textAlign = "center";
      const cy = barH / 2;
      const label = this.teamNames[idx] || `P${idx + 1}`;
      const pctText = `${(pct * 100).toFixed(0)}%`;
      const fullText = `${label} ${pctText}`;
      const textW = ctx.measureText(fullText).width;

      if (segW > textW + 8) {
        ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
        ctx.fillText(fullText, curX + segW / 2, cy);
        ctx.fillStyle = "#fff";
        ctx.fillText(fullText, curX + segW / 2 - 1, cy - 1);
      } else if (segW > 30) {
        ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
        ctx.fillText(pctText, curX + segW / 2, cy);
        ctx.fillStyle = "#fff";
        ctx.fillText(pctText, curX + segW / 2 - 1, cy - 1);
      }

      curX += segW;
    }
  }
}
