# project-100

Rust + Three.js hybrid WASM project. Particle wave simulation computed in Rust/WebAssembly, rendered with Three.js.

## Prerequisites

- Rust 1.85+ (`rustup update`)
- Bun 1.3+ (`curl -fsSL https://bun.sh/install | bash`)

## Quick Start

```
bun install
bun run setup       # install wasm-pack and cargo-watch (one-time)
bun run dev          # builds WASM, then starts Vite + cargo-watch in parallel
```

## Scripts

| Command                  | Description                                |
| ------------------------ | ------------------------------------------ |
| `bun run dev`            | Build WASM, start Vite dev + WASM watch    |
| `bun run build`          | Production build (WASM + Vite)             |
| `bun run wasm:build`     | Build WASM only (debug)                    |
| `bun run wasm:build:release` | Build WASM only (release, optimized)   |
| `bun run preview`        | Preview production build                   |
| `bun run setup`          | Install wasm-pack and cargo-watch          |

## Architecture

- `crate/` — Rust library compiled to WASM via wasm-pack
- `crate/pkg/` — Generated WASM + JS bindings (gitignored)
- `src/` — TypeScript + Three.js frontend
- Vite bundles everything; `vite-plugin-wasm` handles WASM module loading
