import { execSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// Regression check: a grammar bundle staged in public/ must not reach the
// committed viewer. Vite copies public/ verbatim into iguana/viewer-dist, and
// clean-dist.mjs removes the two grammar entries afterwards, but rmSync with
// force accepts a path that does not exist, so a wrong path silently turns
// the cleanup into a no-op. The check seeds the two entries, runs the build
// in its two halves, and asserts the middle state: present after the Vite
// build, absent after the cleanup.
const webViewerDir = fileURLToPath(new URL("..", import.meta.url));
const publicDir = join(webViewerDir, "public");
const viewerDist = join(webViewerDir, "..", "iguana", "viewer-dist");

// The Vite build empties and rewrites viewer-dist, which is committed, so the
// check snapshots it first and restores it on every exit. A failed build or a
// detected regression therefore reports the defect without leaving a partial
// or contaminated viewer behind, and the check as a whole modifies nothing.
const backupRoot = mkdtempSync(join(tmpdir(), "check-clean-dist-"));
const viewerDistBackup = join(backupRoot, "viewer-dist");
const hadViewerDist = existsSync(viewerDist);
if (hadViewerDist) {
  cpSync(viewerDist, viewerDistBackup, { recursive: true });
}

// A bundle staged by stage:iggy already exercises the copy, so the check
// seeds only the entries that are missing and removes only those. The exact
// seeded paths are asserted after the build; for a developer's staged bundle
// only the top-level entries are, since its layout is theirs. The seeding
// happens inside the try, so a failure mid-seed still reaches the cleanup.
const seeded = [];
const copied = ["manifest.json", "wasm"];
try {
  if (!existsSync(join(publicDir, "manifest.json"))) {
    seeded.push(join(publicDir, "manifest.json"));
    writeFileSync(join(publicDir, "manifest.json"), '{"grammarName":"seed"}\n');
  }
  if (!existsSync(join(publicDir, "wasm"))) {
    seeded.push(join(publicDir, "wasm"));
    copied.push(join("wasm", "pkg", "parser.wasm"));
    mkdirSync(join(publicDir, "wasm", "pkg"), { recursive: true });
    writeFileSync(join(publicDir, "wasm", "pkg", "parser.wasm"), "seed");
  }

  execSync("npx vite build", { cwd: webViewerDir, stdio: "inherit" });
  for (const name of copied) {
    if (!existsSync(join(viewerDist, name))) {
      throw new Error(
        `${name} is missing from ${viewerDist} after the Vite build; ` +
          "the build no longer copies public/, so the check does not exercise the cleanup.",
      );
    }
  }
  execSync("node scripts/clean-dist.mjs", { cwd: webViewerDir, stdio: "inherit" });
  for (const name of ["manifest.json", "wasm"]) {
    if (existsSync(join(viewerDist, name))) {
      throw new Error(
        `${name} survived in ${viewerDist}; the cleanup misses the build output.`,
      );
    }
  }
} finally {
  rmSync(viewerDist, { recursive: true, force: true });
  if (hadViewerDist) {
    cpSync(viewerDistBackup, viewerDist, { recursive: true });
  }
  rmSync(backupRoot, { recursive: true, force: true });
  for (const path of seeded) {
    rmSync(path, { recursive: true, force: true });
  }
}

console.log("The committed viewer excludes staged grammar files.");
