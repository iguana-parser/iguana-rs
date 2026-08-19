import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = join(scriptsDir, "..", "..");
const bundleRoot = join(repositoryRoot, "target", "wasm", "iggy");
const manifestSource = join(bundleRoot, "manifest.json");
const wasmSource = join(bundleRoot, "wasm", "pkg");
const publicDir = join(scriptsDir, "..", "public");

for (const source of [manifestSource, wasmSource]) {
  if (!existsSync(source)) {
    throw new Error(`Missing ${source}. Run cargo xtask wasm first.`);
  }
}

const manifestTarget = join(publicDir, "manifest.json");
const wasmTarget = join(publicDir, "wasm");
rmSync(manifestTarget, { force: true });
rmSync(wasmTarget, { recursive: true, force: true });
mkdirSync(wasmTarget, { recursive: true });
cpSync(manifestSource, manifestTarget);
cpSync(wasmSource, join(wasmTarget, "pkg"), { recursive: true });

console.log("Staged the Iggy WebAssembly bundle in web-viewer/public.");
