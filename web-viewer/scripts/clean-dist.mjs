import { rmSync } from "node:fs";

// Vite copies public/ verbatim into dist. When a developer runs the viewer
// locally, public/ holds a grammar's wasm module and manifest (see .gitignore),
// which are grammar-specific. The committed dist is the grammar-independent
// viewer the iguana binary embeds, so drop them: a generated bundle supplies its
// own manifest and wasm.
rmSync("dist/manifest.json", { force: true });
rmSync("dist/wasm", { recursive: true, force: true });
