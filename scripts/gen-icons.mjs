#!/usr/bin/env node
// Generate favicon, PWA icons, and OG image from lw5back.png
import sharp from "sharp";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const src = join(__dirname, "..", "public", "assets", "lw5back.png");
const out = join(__dirname, "..", "public");

const BG = { r: 0, g: 0, b: 32 }; // #000020

async function main() {
  // Load source image (640x480)
  const img = sharp(src);

  // Crop center square (480x480) for icon variants
  const iconBase = img.clone().extract({
    left: 80,
    top: 0,
    width: 480,
    height: 480,
  });

  // favicon.ico (32x32 PNG saved as .ico — browsers accept PNG favicons)
  await iconBase
    .clone()
    .resize(32, 32, { fit: "cover" })
    .png()
    .toFile(join(out, "favicon.ico"));
  console.log("  favicon.ico (32x32)");

  // icon-192.png
  await iconBase
    .clone()
    .resize(192, 192, { fit: "cover" })
    .png()
    .toFile(join(out, "icon-192.png"));
  console.log("  icon-192.png");

  // icon-512.png
  await iconBase
    .clone()
    .resize(512, 512, { fit: "cover" })
    .png()
    .toFile(join(out, "icon-512.png"));
  console.log("  icon-512.png");

  // apple-touch-icon.png (180x180)
  await iconBase
    .clone()
    .resize(180, 180, { fit: "cover" })
    .png()
    .toFile(join(out, "apple-touch-icon.png"));
  console.log("  apple-touch-icon.png (180x180)");

  // og-image.png (1200x630) — letterbox the source onto the dark background
  const ogWidth = 1200;
  const ogHeight = 630;

  // Resize source to fit within 1200x630, maintaining aspect ratio
  const resized = await sharp(src)
    .resize(ogWidth, ogHeight, { fit: "contain", background: BG })
    .png()
    .toBuffer();

  await sharp(resized).toFile(join(out, "og-image.png"));
  console.log("  og-image.png (1200x630)");

  console.log("Done!");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
