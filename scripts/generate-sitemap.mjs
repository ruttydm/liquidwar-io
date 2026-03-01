import { readFileSync, writeFileSync } from "fs";

const BASE = "https://liquidwar.io";
const today = new Date().toISOString().split("T")[0];

// Static routes
const staticRoutes = [
  { path: "/docs", priority: "1.0", changefreq: "weekly" },
  { path: "/docs/how-to-play", priority: "0.8", changefreq: "monthly" },
  { path: "/docs/mechanics", priority: "0.8", changefreq: "monthly" },
  { path: "/docs/maps", priority: "0.9", changefreq: "weekly" },
  { path: "/docs/settings", priority: "0.8", changefreq: "monthly" },
  { path: "/docs/multiplayer", priority: "0.8", changefreq: "monthly" },
  { path: "/docs/history", priority: "0.7", changefreq: "monthly" },
  { path: "/docs/credits", priority: "0.7", changefreq: "monthly" },
];

// Load map data for dynamic routes
const mapsData = JSON.parse(
  readFileSync("docs-site/data/maps_data.json", "utf-8")
);

const urls = [];

for (const route of staticRoutes) {
  urls.push(
    `  <url>\n    <loc>${BASE}${route.path}</loc>\n    <lastmod>${today}</lastmod>\n    <changefreq>${route.changefreq}</changefreq>\n    <priority>${route.priority}</priority>\n  </url>`
  );
}

for (const map of mapsData) {
  urls.push(
    `  <url>\n    <loc>${BASE}/docs/maps/${map.id}</loc>\n    <lastmod>${today}</lastmod>\n    <changefreq>monthly</changefreq>\n    <priority>0.5</priority>\n  </url>`
  );
}

const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls.join("\n")}
</urlset>
`;

writeFileSync("docs-site/public/sitemap.xml", sitemap);
console.log(`Sitemap generated: ${urls.length} URLs`);
