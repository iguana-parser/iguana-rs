import { execSync } from "node:child_process";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// The bundled extension inlines its production dependencies, and minification
// strips their license headers, so the VSIX must ship the notices separately.
// This script collects each production package's license text into
// ThirdPartyNotices.txt, which vsce includes because the file exists at
// packaging time. The file is generated on every package run rather than
// committed, so it cannot drift from the lockfile.
const extensionDir = fileURLToPath(new URL("..", import.meta.url));

// Each package is read from the path npm reports for it, not from
// node_modules/<name>: with nested installations one name can resolve to
// several versions in several directories, and the top-level copy is not
// necessarily the bundled one. Deduplication is by name and version for the
// same reason.
const tree = JSON.parse(
  execSync("npm ls --omit=dev --all --json --long", {
    cwd: extensionDir,
    encoding: "utf8",
  }),
);
const packages = new Map();
const collect = (dependencies) => {
  for (const [name, info] of Object.entries(dependencies ?? {})) {
    if (!info.path) {
      throw new Error(`npm ls reported no path for ${name}@${info.version}.`);
    }
    packages.set(`${name}@${info.version}`, {
      name,
      version: info.version,
      path: info.path,
    });
    collect(info.dependencies);
  }
};
collect(tree.dependencies);
if (packages.size === 0) {
  throw new Error("npm ls reported no production dependencies.");
}

const sections = [...packages.values()]
  .sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version))
  .map(({ name, version, path }) => {
    const entries = readdirSync(path);
    const licenseFiles = entries.filter((entry) =>
      /^(license|licence|copying)/i.test(entry),
    );
    if (licenseFiles.length === 0) {
      throw new Error(`${name}@${version} has no license file in ${path}.`);
    }
    // A NOTICE file holds attribution that its license (Apache 2.0 most
    // commonly) requires redistributing alongside the license text.
    const noticeFiles = entries.filter((entry) => /^notice/i.test(entry));
    const manifest = JSON.parse(readFileSync(join(path, "package.json"), "utf8"));
    const heading = `${name} ${version} (${manifest.license})`;
    const files = [...licenseFiles, ...noticeFiles].sort();
    const texts = files.map((file) => {
      const text = readFileSync(join(path, file), "utf8").trim();
      return files.length > 1 ? `${file}:\n\n${text}` : text;
    });
    return `${heading}\n${"-".repeat(heading.length)}\n\n${texts.join("\n\n")}\n`;
  });

writeFileSync(
  join(extensionDir, "ThirdPartyNotices.txt"),
  "Third-party licenses for packages bundled in the Iguana VS Code extension\n\n" +
    sections.join("\n"),
);
console.log(`ThirdPartyNotices.txt lists ${packages.size} packages.`);
