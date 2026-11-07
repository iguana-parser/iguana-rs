import { defineConfig } from "vite";

export default defineConfig({
  build: {
    lib: {
      entry: "src/main.ts",
      formats: ["es"],
      // A stable name (no content hash): the host page imports this path
      // directly, and a rebuild should rewrite the same file.
      fileName: () => "grammar-view.js",
    },
    // CodeMirror is bundled, and library mode does not minify ES output by
    // default.
    rollupOptions: { output: { minify: true } },
  },
});
