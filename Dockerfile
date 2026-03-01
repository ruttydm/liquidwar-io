# ── Stage 1: Builder ──
FROM rust:bookworm AS builder

# System deps
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl nodejs npm \
    && rm -rf /var/lib/apt/lists/*

# Rust nightly (for docs-site Leptos) + wasm target
RUN rustup toolchain install nightly \
    && rustup target add wasm32-unknown-unknown --toolchain nightly \
    && rustup target add wasm32-unknown-unknown --toolchain stable

# Build tools
RUN cargo install wasm-pack \
    && cargo install cargo-leptos

WORKDIR /build

# Copy dependency manifests first for caching
COPY Cargo.toml Cargo.lock ./
COPY game/Cargo.toml game/Cargo.toml
COPY client-wasm/Cargo.toml client-wasm/Cargo.toml
COPY server/Cargo.toml server/Cargo.toml
COPY docs-site/Cargo.toml docs-site/Cargo.toml
COPY docs-site/rust-toolchain.toml docs-site/rust-toolchain.toml
COPY package.json ./

# Create stub source files so cargo can resolve the workspace
RUN mkdir -p game/src client-wasm/src server/src docs-site/src \
    && echo "pub fn stub() {}" > game/src/lib.rs \
    && echo "pub fn stub() {}" > client-wasm/src/lib.rs \
    && echo "fn main() {}" > server/src/main.rs \
    && echo "pub fn stub() {}" > docs-site/src/lib.rs \
    && echo "fn main() {}" > docs-site/src/main.rs

# Pre-fetch cargo deps (cached layer)
RUN cargo fetch

# Now copy actual source
COPY . .

# Restore real source timestamps so cargo rebuilds
RUN touch game/src/lib.rs client-wasm/src/lib.rs server/src/main.rs \
    docs-site/src/lib.rs docs-site/src/main.rs

# Install Node deps
RUN npm install

# 1. Build client WASM
RUN wasm-pack build client-wasm --target web --out-dir pkg --release

# 2. Build game frontend
RUN npx vite build

# 3. Generate docs data (map index, metadata, sitemap)
RUN node scripts/update-map-index.mjs \
    && node scripts/extract-map-data.mjs \
    && node scripts/generate-sitemap.mjs

# 4. Build docs site (Leptos SSR + WASM hydration)
RUN cd docs-site && cargo leptos build --release

# 5. Build game server
RUN cargo build --release -p server

# ── Stage 2: Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    nginx ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Game frontend (Vite build)
COPY --from=builder /build/dist/ /app/dist/

# Public assets (maps, music, textures, sfx, etc.)
COPY --from=builder /build/public/ /app/public/

# Game WebSocket server
COPY --from=builder /build/target/release/server /app/server

# Docs site binary
COPY --from=builder /build/target/release/docs-site /app/docs-site

# Docs site compiled assets (CSS, JS, WASM, robots.txt, sitemap.xml, favicon, og-image)
# cargo leptos copies public/ into target/site/ and adds compiled pkg/
COPY --from=builder /build/target/site/ /app/docs-site-assets/

# Nginx config
COPY nginx.conf /etc/nginx/nginx.conf

# Entrypoint
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Server needs MAPS_DIR env
ENV MAPS_DIR=/app/public/maps
ENV LEPTOS_SITE_ROOT=/app/docs-site-assets
ENV LEPTOS_SITE_ADDR=127.0.0.1:3002
ENV LEPTOS_SITE_PKG_DIR=pkg

EXPOSE 80

CMD ["/app/entrypoint.sh"]
