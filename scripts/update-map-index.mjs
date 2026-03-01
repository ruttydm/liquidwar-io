#!/usr/bin/env node
// Update public/maps/index.json to include all PNG maps in the directory
import { readFileSync, writeFileSync, readdirSync } from "fs";
import { join, dirname, basename } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const mapsDir = join(__dirname, "..", "public", "maps");
const indexPath = join(mapsDir, "index.json");

// Read existing index
const existing = JSON.parse(readFileSync(indexPath, "utf-8"));
const byId = new Map(existing.map((e) => [e.id, e.name]));

// Scan all PNGs
const pngs = readdirSync(mapsDir)
  .filter((f) => f.endsWith(".png"))
  .map((f) => f.replace(/\.png$/, ""))
  .sort();

// Derive display name from filename: "avoid-the-void" → "Avoid The Void"
function deriveName(id) {
  return id
    .replace(/[-_]/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

let added = 0;
for (const id of pngs) {
  if (!byId.has(id)) {
    byId.set(id, deriveName(id));
    added++;
  }
}

// Build sorted array
const result = [...byId.entries()]
  .sort(([a], [b]) => a.localeCompare(b))
  .map(([id, name]) => ({ id, name }));

writeFileSync(indexPath, JSON.stringify(result, null, 2) + "\n");
console.log(`Updated index.json: ${result.length} maps (${added} added)`);
