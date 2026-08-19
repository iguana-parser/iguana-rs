# Terrarium

Terrarium is an IDE for Iggy grammars. It combines grammar authoring, parser
generation, parse-tree visualization, and step-by-step parser debugging.

The implementation is a Tauri app with a Monaco editor and Cytoscape graph
views. Terrarium is experimental and is not released for general use. Its
interfaces and behavior may change at any time.

Terrarium has three modes:

- **Design mode.** Terrarium provides a Monaco grammar editor with
  highlighting, formatting, document symbols, diagnostics, folding,
  go-to-definition, and find-references navigation.
- **Parse mode.** Terrarium runs the generated parser and shows its parse tree
  as a collapsible tree, graph, or s-expression.
- **Debug mode.** Terrarium replays parser execution step by step and shows the
  SPPF and GSS as they are built. This mode requires a parser built with the
  `debug-trace` feature.

## Development

Terrarium requires Rust, Node.js and npm, and the platform dependencies listed
in the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). Install
the repository dependencies from the root, then start the app:

```sh
npm install
cargo xtask terrarium
```

The `xtask` command rebuilds the web viewer, installs the current `iguana`
binary into the Cargo bin directory, and launches `tauri dev`. Terrarium calls
that installed binary when it generates a parser.

To run the same steps separately:

```sh
cargo xtask install
cd terrarium
npm run tauri dev
```

Check the frontend from the repository root:

```sh
npm run check --workspace terrarium
```

The Rust backend is a separate workspace at `src-tauri/`. Check it through its
own manifest:

```sh
cargo fmt --manifest-path terrarium/src-tauri/Cargo.toml --check
cargo check --manifest-path terrarium/src-tauri/Cargo.toml
cargo clippy --manifest-path terrarium/src-tauri/Cargo.toml --all-targets
cargo test --manifest-path terrarium/src-tauri/Cargo.toml
```

Run `npm run check` at the repository root after changing `web-ui` or another
shared frontend package. Specta rewrites `src/bindings.ts` when a debug build
starts; review that file after changing the Tauri command interface.

An Iggy language feature must work in both the desktop and browser hosts. Add
the analysis function under `iguana-lsp/src/` and export it from
`iguana-lsp/wasm/src/lib.rs`. Add the method to `LspBackend` in
`web-ui/src/lsp-backend.ts`, implement it in
`web-ui/src/wasm-lsp-backend.ts`, and register the corresponding Monaco
provider in `web-ui/src/DesignView.svelte`. For Terrarium, expose the function
through a Tauri command in `src-tauri/src/lib.rs` and implement the method in
`src/lib/tauri-lsp-backend.ts`.

To create a platform application bundle, run `npm run tauri build` from
`terrarium/`.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
