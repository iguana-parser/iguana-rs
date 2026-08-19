import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// The heavy libraries (Monaco, Cytoscape) are not bundled. They load at runtime
// from a CDN via the importmap in index.html, so the embedded viewer stays
// small. Keep this list in sync with that importmap.
const external = ["monaco-editor", "cytoscape", "cytoscape-tidytree"];

export default defineConfig({
  // A relative base, so the built viewer works at any mount: served at the
  // root by `iguana try` or under a subpath on the website, with no path
  // rewriting. App.svelte resolves BASE_URL against the document before use.
  base: "./",
  plugins: [svelte()],
  optimizeDeps: { exclude: external },
  build: {
    // The output lives inside the iguana crate so cargo packages it and a
    // registry build of iguana works without npm. It is committed there.
    outDir: "../iguana/viewer-dist",
    emptyOutDir: true,
    rollupOptions: {
      external,
      // Stable output names (no content hash). The output is committed and
      // embedded in the iguana binary, so a rebuild should rewrite the same
      // files rather than churn the git history with new hashed filenames
      // each time.
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
