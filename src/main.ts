import { createScene } from "./scene";

async function main() {
  const { animate } = await createScene();
  animate();
}

main().catch(console.error);
