import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Monaco is not bundled. It loads at runtime from a CDN via the importmap in
// index.html, so the deployed app stays small. Keep this in sync with that
// importmap.
const external = ["monaco-editor"];

export default defineConfig({
  // A relative base, so the built app works at any mount with no path
  // rewriting. The sibling lsp-wasm module resolves relative to the page;
  // App.svelte resolves BASE_URL against the document before use.
  base: "./",
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
