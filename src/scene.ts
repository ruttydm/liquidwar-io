import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import initWasm, {
  create_simulation,
  step_simulation,
  get_positions,
  get_speeds,
} from "../crate/pkg/crate";

const PARTICLE_COUNT = 3000;
const DOMAIN_WIDTH = 4.0;
const DOMAIN_HEIGHT = 6.0;
const DOMAIN_DEPTH = 4.0;

const vertexShader = `
  attribute float speed;
  varying float vSpeed;

  void main() {
    vSpeed = speed;
    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
    gl_PointSize = 5.0 * (300.0 / -mvPosition.z);
    gl_Position = projectionMatrix * mvPosition;
  }
`;

const fragmentShader = `
  varying float vSpeed;

  void main() {
    float dist = length(gl_PointCoord - vec2(0.5));
    if (dist > 0.5) discard;
    float alpha = 1.0 - smoothstep(0.3, 0.5, dist);

    float t = clamp(vSpeed / 5.0, 0.0, 1.0);
    vec3 slow = vec3(0.05, 0.15, 0.6);
    vec3 mid  = vec3(0.2, 0.5, 1.0);
    vec3 fast = vec3(0.8, 0.95, 1.0);

    vec3 color;
    if (t < 0.5) {
      color = mix(slow, mid, t * 2.0);
    } else {
      color = mix(mid, fast, (t - 0.5) * 2.0);
    }

    gl_FragColor = vec4(color, alpha * 0.85);
  }
`;

export async function createScene() {
  await initWasm();

  const actualCount = create_simulation(PARTICLE_COUNT);

  const renderer = new THREE.WebGLRenderer({
    antialias: true,
    powerPreference: "high-performance",
  });
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  document.body.appendChild(renderer.domElement);

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x0a0a1a);

  const camera = new THREE.PerspectiveCamera(
    50,
    window.innerWidth / window.innerHeight,
    0.1,
    100,
  );
  camera.position.set(8, 5, 8);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.target.set(DOMAIN_WIDTH / 2, DOMAIN_HEIGHT / 3, DOMAIN_DEPTH / 2);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  controls.update();

  // Particle geometry
  const geometry = new THREE.BufferGeometry();
  const positionAttr = new THREE.Float32BufferAttribute(
    new Float32Array(actualCount * 3),
    3,
  );
  const speedAttr = new THREE.Float32BufferAttribute(
    new Float32Array(actualCount),
    1,
  );
  geometry.setAttribute("position", positionAttr);
  geometry.setAttribute("speed", speedAttr);

  const material = new THREE.ShaderMaterial({
    vertexShader,
    fragmentShader,
    transparent: true,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });

  const points = new THREE.Points(geometry, material);
  scene.add(points);

  // Container wireframe
  const boxGeom = new THREE.BoxGeometry(
    DOMAIN_WIDTH,
    DOMAIN_HEIGHT,
    DOMAIN_DEPTH,
  );
  const boxEdges = new THREE.EdgesGeometry(boxGeom);
  const boxLines = new THREE.LineSegments(
    boxEdges,
    new THREE.LineBasicMaterial({
      color: 0x334466,
      opacity: 0.4,
      transparent: true,
    }),
  );
  boxLines.position.set(
    DOMAIN_WIDTH / 2,
    DOMAIN_HEIGHT / 2,
    DOMAIN_DEPTH / 2,
  );
  scene.add(boxLines);

  window.addEventListener("resize", () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
  });

  // Set initial positions
  const initialPositions = get_positions();
  (positionAttr.array as Float32Array).set(initialPositions);
  positionAttr.needsUpdate = true;

  function animate() {
    requestAnimationFrame(animate);

    step_simulation();

    const positions = get_positions();
    const speeds = get_speeds();

    (positionAttr.array as Float32Array).set(positions);
    positionAttr.needsUpdate = true;

    (speedAttr.array as Float32Array).set(speeds);
    speedAttr.needsUpdate = true;

    controls.update();
    renderer.render(scene, camera);
  }

  return { animate };
}
