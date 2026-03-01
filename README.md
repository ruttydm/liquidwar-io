# LiquidWar.io

A web-based multiplayer remake of [Liquid War](https://en.wikipedia.org/wiki/Liquid_War), the classic game where you control a cursor to lead your army of fighters and absorb the enemy.

**Play now at [liquidwar.io](https://liquidwar.io)**

## How It Works

Each player controls a cursor. Every fighter on the map follows the nearest friendly cursor using gradient-based pathfinding. When fighters from different teams collide, the outnumbered side loses health. The last team standing wins.

## Tech Stack

- **Game engine** — Rust, compiled to WebAssembly via `wasm-pack`
- **Renderer** — TypeScript + WebGL (Three.js)
- **Multiplayer server** — Rust WebSocket server
- **Docs site** — Leptos (Rust SSR + WASM hydration)
- **Deployment** — Docker (multi-stage build), Nginx reverse proxy

## Project Structure

```
game/           Rust game logic (fighters, mesh, gradient, AI)
client-wasm/    WASM bindings for the browser client
server/         WebSocket multiplayer server
docs-site/      Documentation site (Leptos SSR)
src/            TypeScript frontend (renderer, input, audio)
public/         Static assets (maps, music, textures, sfx)
scripts/        Build utilities (map index, sitemap generation)
```

## Development

### Prerequisites

- Rust 1.85+ with `wasm32-unknown-unknown` target
- Node.js or Bun
- `wasm-pack` and `cargo-watch`

```sh
# One-time setup
rustup target add wasm32-unknown-unknown
npm run setup

# Install JS dependencies
npm install

# Start dev server (WASM + Vite + game server)
npm run dev
```

### Scripts

| Command | Description |
| --- | --- |
| `npm run dev` | Build WASM, start Vite dev + WASM watch + game server |
| `npm run build` | Production build (WASM + Vite) |
| `npm run server` | Run game server standalone |
| `npm run docs:dev` | Run docs site in dev mode |
| `npm run docs:build` | Build docs site for production |

### Docker

```sh
docker build -t liquidwar .
docker run -p 80:80 liquidwar
```

This builds the game client, game server, and docs site in a single multi-stage Docker image with Nginx as the reverse proxy.

## Credits

Based on [Liquid War 5](https://www.nongnu.org/liquidwar/) by Thomas Colcombet and Christian Mauduit.

## License

See [LICENSE](LICENSE) for details.
