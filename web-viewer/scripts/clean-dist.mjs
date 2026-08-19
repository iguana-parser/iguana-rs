import { rmSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// Vite copies public/ verbatim into the configured output directory. The
// stage:iggy script puts one grammar's manifest and WebAssembly module in
// public/ for local development, but the committed viewer must remain
// grammar-independent because generated bundles supply those files themselves.
const viewerDist = fileURLToPath(
  new URL("../../iguana/viewer-dist/", import.meta.url),
);
const grammarAssets = [
  join(viewerDist, "manifest.json"),
  join(viewerDist, "wasm"),
];

for (const asset of grammarAssets) {
  rmSync(asset, { recursive: true, force: true });
}
