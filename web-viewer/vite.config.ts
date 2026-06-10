import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// The heavy libraries (Monaco, Cytoscape) are not bundled. They load at runtime
// from a CDN via the importmap in index.html, so the embedded viewer stays
// small. Keep this list in sync with that importmap.
const external = ["monaco-editor", "cytoscape", "cytoscape-tidytree"];

export default defineConfig({
  plugins: [svelte()],
  optimizeDeps: { exclude: external },
  build: {
    rollupOptions: {
      external,
      // Stable output names (no content hash). dist/ is committed and embedded
      // in the iguana binary, so a rebuild should rewrite the same files rather
      // than churn the git history with new hashed filenames each time.
      output: {
        entryFileNames: "assets/[name].js",
        chunkFileNames: "assets/[name].js",
        assetFileNames: "assets/[name][extname]",
      },
    },
  },
  server: {
    port: 5174,
    strictPort: true,
  },
});
