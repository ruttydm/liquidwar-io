import * as THREE from "three";
import {
  init_particle_grid,
  update_particle_positions,
} from "../crate/pkg/crate";

const PARTICLE_COUNT = 10_000; // 100x100 grid
const SPACING = 0.5;

export async function createScene() {
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setPixelRatio(window.devicePixelRatio);
  document.body.appendChild(renderer.domElement);

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x0a0a1a);

  const camera = new THREE.PerspectiveCamera(
    60,
    window.innerWidth / window.innerHeight,
    0.1,
    1000,
  );
  camera.position.set(30, 20, 30);
  camera.lookAt(25, 0, 25);

  // Initialize particle positions from Rust
  const positions = init_particle_grid(PARTICLE_COUNT, SPACING);

  const geometry = new THREE.BufferGeometry();
  const positionAttribute = new THREE.Float32BufferAttribute(positions, 3);
  geometry.setAttribute("position", positionAttribute);

  const material = new THREE.PointsMaterial({
    size: 0.15,
    color: 0x44aaff,
    sizeAttenuation: true,
  });

  const points = new THREE.Points(geometry, material);
  scene.add(points);

  window.addEventListener("resize", () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
  });

  const clock = new THREE.Clock();

  function animate() {
    requestAnimationFrame(animate);

    const time = clock.getElapsedTime();

    // Rust computes wave positions each frame
    update_particle_positions(
      positionAttribute.array as Float32Array,
      PARTICLE_COUNT,
      time,
    );
    positionAttribute.needsUpdate = true;

    // Slow camera orbit
    camera.position.x = 25 + 20 * Math.cos(time * 0.1);
    camera.position.z = 25 + 20 * Math.sin(time * 0.1);
    camera.lookAt(25, 0, 25);

    renderer.render(scene, camera);
  }

  return { animate };
}
