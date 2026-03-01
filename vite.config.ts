import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  build: {
    target: "esnext",
  },
  server: {
    proxy: {
      "/docs": "http://127.0.0.1:3002",
      "/pkg": "http://127.0.0.1:3002",
    },
  },
});
