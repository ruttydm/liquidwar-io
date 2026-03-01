#!/usr/bin/env node
// Extract map metadata from index.json + PNG headers + metadata.xml files
// Outputs docs-site/data/maps_data.json
import { readFileSync, writeFileSync, readdirSync, statSync } from "fs";
import { join, dirname, basename } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const mapsDir = join(root, "public", "maps");
const extraDir = join(root, "tmp", "liquidwar6-extra-maps");
const outPath = join(root, "docs-site", "data", "maps_data.json");

// 1. Read index.json
const index = JSON.parse(readFileSync(join(mapsDir, "index.json"), "utf-8"));

// 2. Read PNG dimensions from file header (first 24 bytes contain IHDR)
function pngDimensions(filePath) {
  try {
    const buf = readFileSync(filePath);
    // PNG header: 8 bytes signature, then IHDR chunk (4 len + 4 type + 4 width + 4 height)
    if (buf.length < 24) return { width: 0, height: 0 };
    const width = buf.readUInt32BE(16);
    const height = buf.readUInt32BE(20);
    return { width, height };
  } catch {
    return { width: 0, height: 0 };
  }
}

// 3. Parse metadata.xml files — format: <string key="X" value="Y" />
function parseMetadataXml(filePath) {
  try {
    const xml = readFileSync(filePath, "utf-8");
    const meta = {};
    const re = /<string\s+key="([^"]+)"\s+value="([^"]*)"\s*\/>/g;
    let m;
    while ((m = re.exec(xml)) !== null) {
      meta[m[1]] = m[2];
    }
    return meta;
  } catch {
    return null;
  }
}

// 4. Walk extra maps dir to collect all metadata.xml keyed by directory basename
function collectMetadata(dir) {
  const result = new Map(); // basename -> metadata object
  function walk(d) {
    let entries;
    try {
      entries = readdirSync(d, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      const full = join(d, e.name);
      if (e.isDirectory()) {
        walk(full);
      } else if (e.name === "metadata.xml") {
        const meta = parseMetadataXml(full);
        if (meta) {
          const mapName = basename(d);
          // Don't overwrite if already seen (first match wins)
          if (!result.has(mapName)) {
            result.set(mapName, meta);
          }
        }
      }
    }
  }
  walk(dir);
  return result;
}

// Also check lw5 legacy maps which are nested under legacy/lw5/kasper/ etc
const metadata = collectMetadata(extraDir);

// 5. Build output
const maps = index.map((entry) => {
  const dims = pngDimensions(join(mapsDir, `${entry.id}.png`));
  const meta = metadata.get(entry.id);

  return {
    id: entry.id,
    name: entry.name,
    author: meta?.author || null,
    description: meta?.description || null,
    license: meta?.license || null,
    width: dims.width,
    height: dims.height,
  };
});

writeFileSync(outPath, JSON.stringify(maps, null, 2) + "\n");

// Stats
const withAuthor = maps.filter((m) => m.author).length;
const authors = new Map();
for (const m of maps) {
  if (m.author) {
    authors.set(m.author, (authors.get(m.author) || 0) + 1);
  }
}
console.log(`Generated ${outPath}`);
console.log(`  ${maps.length} maps total, ${withAuthor} with metadata`);
console.log(`  Authors:`);
[...authors.entries()]
  .sort((a, b) => b[1] - a[1])
  .forEach(([name, count]) => console.log(`    ${name}: ${count}`));
