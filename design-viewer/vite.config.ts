import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Monaco is not bundled. It loads at runtime from a CDN via the importmap in
// index.html, so the deployed app stays small. Keep this in sync with that
// importmap.
const external = ["monaco-editor"];

export default defineConfig({
  // The app deploys to a fixed mount on the website, so its assets and the
  // sibling lsp-wasm module resolve under this base.
  base: "/design-view/",
  plugins: [svelte()],
  optimizeDeps: { exclude: external },
  build: {
    rollupOptions: {
      external,
      // Stable output names (no content hash), so a rebuild rewrites the same
      // files rather than churning the deployed bundle.
      output: {
        entryFileNames: "assets/[name].js",
        chunkFileNames: "assets/[name].js",
        assetFileNames: "assets/[name][extname]",
      },
    },
  },
  server: {
    port: 5175,
    strictPort: true,
  },
});
