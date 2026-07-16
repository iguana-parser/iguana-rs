import { defineConfig } from "vite";

// Cytoscape and its layout extension are not bundled. They load at runtime
// from a CDN via an importmap on the host page, so the widget stays small.
// Keep this list (and the pinned versions on the host) in sync with the
// importmaps in web-viewer and design-viewer.
const external = ["cytoscape", "cytoscape-tidytree"];

export default defineConfig({
  build: {
    lib: {
      entry: "src/main.ts",
      formats: ["es"],
      // A stable name (no content hash): the host page imports this path
      // directly, and a rebuild should rewrite the same file.
      fileName: () => "tree-widget.js",
    },
    rollupOptions: { external },
  },
});
