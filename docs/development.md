# Development

This guide covers the development workflows for iguana-rs. See the
[architecture guide](architecture.md) for the repository layout and component
relationships.

## Prerequisites

- **Rust.** The workspace uses Rust edition 2024 and the current stable
  toolchain.
- **Node.js and npm.** Node.js 22.12 or newer is required for the
  web packages, Terrarium, and the VS Code extension.
- **Graphviz.** The `dot` command is required only when a generated parser
  writes SPPF or GSS graphs as SVG.
- **WebAssembly tools.** `wasm-pack` and the `wasm32-unknown-unknown` target are
  required only for WebAssembly builds.

## Setup

After cloning, run once:

```sh
./setup.sh
```

The script checks for Cargo, installs `cargo-nextest`, installs the root npm
workspace dependencies when npm is available, reports a missing Graphviz
installation, and points Git at the tracked `.githooks/` directory. The VS
Code extension has a separate npm installation under `editors/vscode`.

The pre-commit hook rejects unformatted commits through
`cargo fmt --check --all`. The commit-message hook rejects `Co-Authored-By`,
`Signed-off-by`, and `Generated-by` trailers and AI-attribution lines. CI
applies the same checks to pull requests.

## Build and check

The root Cargo workspace contains the Rust crates, `xtask`, and the generated
grammar-test crates. Use `-p` to limit an iteration to the package you changed:

```sh
cargo fmt --all --check
cargo build --workspace
cargo check -p <package>
cargo clippy -p <package> --all-targets
cargo test -p <package>
cargo xtask test
```

Replace `<package>` with a Cargo package name such as `iguana-compiler`,
`iguana-runtime`, or `iguana-lsp`. Use the package-level commands while
iterating, then run `cargo xtask test` when the change can affect generated
parsers or more than one crate. This command uses `cargo-nextest` when it is
available and otherwise falls back to `cargo test --workspace`.

The Terrarium backend is a separate Cargo workspace because its Tauri
dependencies slow rust-analyzer for the root workspace. Keep it outside the
root workspace and use the checks in the
[Terrarium workflow](../terrarium/README.md#development).

The root npm workspace contains `web-ui`, `web-viewer`, `design-viewer`,
`terrarium`, and `tree-widget`. Check all five packages, or select one while
iterating:

```sh
npm run check
npm run check --workspace terrarium
```

`npm run check` runs `svelte-check` across `web-ui`, `web-viewer`,
`design-viewer`, and `terrarium`, and runs `tsc` for `tree-widget`. The VS Code
extension is not part of the root npm workspace and must be compiled
separately:

```sh
npm --prefix editors/vscode run compile
```

Warnings fail the Svelte checks: an unused CSS selector or an accessibility
warning breaks the build.

Run the root check after any frontend change. Suppress a Svelte warning with
`<!-- svelte-ignore rule_name -->` above the element only when the warning is
wrong or the fix requires a separate feature. Explain each kind of suppression
once, at its first use.

## Application and web workflows

Install the root npm dependencies before working on a web package. These are
the shortest useful iteration commands; follow the linked README when a
component needs generated assets or platform setup.

| Component | Iterate | Build or install | Details |
|---|---|---|---|
| Terrarium | Run `cargo xtask terrarium`. | Run `npm --prefix terrarium run tauri -- build`. | [Terrarium workflow](../terrarium/README.md#development) |
| Shared web UI | Run `npm run check --workspace web-ui`. | No separate build is required; consumers compile the source. | [`web-ui` workflow](../web-ui/README.md#workspace-integration) |
| Web viewer | Run `npm run dev --workspace web-viewer`. | Run `npm run build --workspace web-viewer`. | [Web viewer workflow](../web-viewer/README.md#development) |
| Design viewer | Run `npm run dev --workspace design-viewer`. | Run `npm run build --workspace design-viewer`. | [Design viewer workflow](../design-viewer/README.md#development) |
| Tree widget | Run `npm run check --workspace tree-widget`. | Run `npm run build --workspace tree-widget`. | [Tree widget workflow](../tree-widget/README.md) |
| VS Code extension | Run `npm --prefix editors/vscode run compile`. | Run `cargo xtask install-vscode-extension`. | [VS Code workflow](../editors/vscode/README.md) |

## Public documentation

User-facing documentation and the published site live in the separate
[`iguana-parser/website`](https://github.com/iguana-parser/website) repository.
Its `content/` directory contains the documentation source. Install
[Zola](https://www.getzola.org/documentation/getting-started/installation/),
then run from the website repository:

```sh
zola check --skip-external-links
zola serve
```

The check validates the local site without depending on external network
access, and `zola serve` starts the local preview. Run `zola check` without the
flag before publishing to validate external links as well. Use the website for
public tone and terminology, but verify commands, features, and package
behavior against this repository before updating the site. The Markdown files
in this repository are written for contributors and cover components and
internal development; the published product documentation lives on the
website.

## xtask commands

`xtask` is the project's command runner. List subcommands with `cargo xtask --help`:

| Command | Purpose |
|---------|---------|
| `cargo xtask install` | Rebuild the web viewer (`npm run build`), then build `iguana` in release mode and install it into `$CARGO_HOME/bin`. Needs npm on PATH; a plain `cargo install` stays npm-free and embeds the committed `iguana/viewer-dist`. |
| `cargo xtask install-vscode-extension` | Build and install the current `iguana-lsp`, package the VS Code extension as a VSIX, and install or replace it through the `code` command. Reload VS Code afterward. Requires Node.js 22.12 or newer, npm, and `code` on PATH. |
| `cargo xtask bootstrap` | Regenerate `iggy` from `iggy/iggy.iggy`. |
| `cargo xtask test [args...]` | Run the Cargo tests (`cargo-nextest` if installed, otherwise `cargo test --workspace`), then build the grammar-test binaries and check every grammar's output against its golden files. Extra arguments are forwarded to the Rust test command. `--regen` rewrites the golden files instead of checking them and skips the Rust tests. |
| `cargo xtask test-new <name>` | Scaffold a new grammar test (directory + stub `.iggy`). Pure scaffolding; no generator. |
| `cargo xtask test-gen <name>` | Run the generator on the grammar (lib + `main.rs`), patch the Cargo.toml to workspace membership, and add the crate to workspace `members`. |
| `cargo xtask test-gen-all` | Run `test-gen` for every directory under `tests/` that has a grammar file. |
| `cargo xtask test-rm <name>` | Remove a grammar test: delete the directory and remove the crate from workspace `members`. |
| `cargo xtask wasm [test]` | Generate a WebAssembly bundle for Iggy, or for the named grammar test, under `target/wasm/` and build it with `wasm-pack` against the local runtime. Requires `wasm-pack` and the `wasm32-unknown-unknown` target. |
| `cargo xtask terrarium` | Install `iguana`, then launch the Terrarium dev server. |

## Bootstrapping

The Iggy grammar (`iggy/iggy.iggy`) is parsed by the committed `iggy` parser,
which Iguana generated from that same grammar. After a change to the grammar or
generator, run:

```sh
cargo xtask bootstrap
cargo xtask bootstrap   # second run to verify stability
```

The second run must produce no diff against the first. If it does, the generator is non-deterministic or the change is not a fixed point.

When a generator change breaks compilation of the existing `iggy` parser, the
bootstrap cannot continue because `iguana` needs that parser to read the
grammar. There are two recovery paths:

- **Preserve compatibility.** Make the generator accept both the old and new
  generated forms during the transition.
- **Bridge the broken generation.** After the first bootstrap, manually patch
  the regenerated parser until it compiles. The next bootstrap then replaces
  that temporary patch with output from the new generator.

## Regeneration by output

Choose regeneration commands by the output a change can alter, not by the
directory containing the changed code:

| Affected output | Command | Result |
|---|---|---|
| The committed Iggy parser, including its generated Rust sources, CLI, and `Cargo.toml` | Run `cargo xtask bootstrap` twice. | Rewrites generated files under `iggy/`; the second run must produce no diff. |
| One grammar-test parser or its scaffold | Run `cargo xtask test-gen <name>`. | Rewrites the generated crate under `tests/<name>/` and ensures it is a root workspace member. |
| Generated Rust or scaffolding shared by grammar tests | Run `cargo xtask test-gen-all`. | Rewrites every generated parser crate under `tests/`. |
| Grammar-test s-expression expectations | Run `cargo xtask test --regen`. | Rewrites the golden files without running the Rust tests. |
| A generated WebAssembly parser, wrapper, or manifest | Run `cargo xtask wasm [test]`. | Rebuilds the selected bundle under the ignored `target/wasm/` directory. |
| The static viewer embedded in the `iguana` binary | Run `npm run build --workspace web-viewer`. | Rewrites the committed `iguana/viewer-dist/` bundle. |
| Terrarium's Rust-to-TypeScript command bindings | Start a debug build with `npm run tauri dev` in `terrarium/`. | Specta rewrites `terrarium/src/bindings.ts` when the app starts. |

A change to grammar lowering, generated parser sources, or parser scaffolding
usually affects both the Iggy parser and the grammar-test parsers. For that
case, run the complete sequence:

1. Run `cargo fmt --all`.
2. Run `cargo check -p iguana-compiler` and
   `cargo clippy -p iguana-compiler --all-targets`.
3. Run `cargo xtask bootstrap` twice.
4. Run `cargo xtask test-gen-all`.
5. Run `cargo xtask test`.
6. Run `cargo xtask wasm` as well if the change can affect WebAssembly output.

The bootstrap and grammar-test generation commands rewrite committed files.
Review their diffs before committing. A change limited to validation or error
reporting does not require regeneration unless it changes one of the outputs
listed above.

## Testing

Iguana has unit and integration tests inside the crates and grammar tests under
`tests/<name>/`. Grammar tests compare generated-parser output with golden
s-expression files. `cargo xtask test` runs both kinds. See the
[testing guide](testing.md) for the grammar-test layout and workflow.

## LSP server

`cargo xtask install-vscode-extension` installs the current `iguana-lsp` binary
and the matching local VS Code extension. The extension checks
`$CARGO_HOME/bin` and then `PATH` for the server when an Iggy file opens. See
the [extension README](../editors/vscode/README.md) for the manual development
workflow.

## Generated code

Files marked `// @generated by iguana. Do not edit manually.` are produced by
the generator. To change them, edit the corresponding generator module under
`iguana-compiler/src/generator/` and rerun the relevant xtask command.

Comments in generated code use attribute placeholders that `post_process.rs`
rewrites: `#[comment = "..."]` becomes `//`, and `#[doc = "..."]` becomes
`///`. The post-processor also inserts blank lines before item-level `impl`,
`fn`, and `pub fn` definitions.

`terrarium/src/bindings.ts` is also generated. Specta rewrites it when a debug
build of the Terrarium backend starts.

## Release checklist

Before publishing 0.1.0, remove `--version 0.1.0-alpha` from the installation
commands in the `iguana` and `iguana-lsp` READMEs and on the website's
installation page. The explicit version is required while crates.io contains
only prerelease versions.
