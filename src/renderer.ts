// Team colors [R, G, B] — 6 teams matching LW5
const TEAM_COLORS = [
  [66, 135, 245], // Blue
  [245, 66, 66], // Red
  [66, 245, 96], // Green
  [245, 215, 66], // Yellow
  [220, 66, 245], // Purple
  [66, 245, 230], // Cyan
];
const WALL_COLOR = [40, 40, 50];
const BG_COLOR = [15, 15, 25];

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

  if (byte == 254) {
    fragColor = vec4(wallColor, 1.0);
  } else if (byte >= 255) {
    fragColor = vec4(bgColor, 1.0);
  } else {
    int team = (byte >> 4) & 0xF;
    int health = byte & 0xF;
    float brightness = 0.4 + 0.6 * float(health) / 15.0;
    vec3 color = teamColors[team % 6] * brightness;
    fragColor = vec4(color, 1.0);
  }
}`;

export class Renderer {
  private gl: WebGL2RenderingContext;
  private texture: WebGLTexture;
  private mapWidth: number;
  private mapHeight: number;
  private canvas: HTMLCanvasElement;

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

    // Fullscreen quad
    const quad = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);
    const vbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, quad, gl.STATIC_DRAW);
    const aPos = gl.getAttribLocation(prog, "a_pos");
    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0);

    // Create bitmap texture
    this.texture = gl.createTexture()!;
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    const uBitmap = gl.getUniformLocation(prog, "u_bitmap");
    gl.uniform1i(uBitmap, 0);

    window.addEventListener("resize", () => this.resize());
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

  render(bitmap: Uint8Array, cursors: Array<[number, number] | null>) {
    const gl = this.gl;

    // Upload bitmap as R8 texture
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.R8,
      this.mapWidth,
      this.mapHeight,
      0,
      gl.RED,
      gl.UNSIGNED_BYTE,
      bitmap,
    );

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

    // Draw cursors as overlay using 2D canvas
    // (could be done in shader but this is simpler for the skeleton)
  }
}
