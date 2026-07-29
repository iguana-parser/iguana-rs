# Terrarium

Grammar authoring, debugging, and parse-tree visualization tool for Iguana: a Tauri app with a Monaco grammar editor and Cytoscape graph views. Terrarium opens a grammar directory, generates and builds its parser, and shows how it parses an input.

Terrarium has three modes:

- **Design**: grammar editing in Monaco, with highlighting, formatting, symbols, and diagnostics from `iguana-lsp`.
- **Parse**: parse-tree, SPPF, GSS, and timing views over a run of the generated parser on input text.
- **Debug**: step-by-step trace replay of the parser's execution (requires a build with `--features debug-trace`).

The website's [Terrarium page](https://iguana-parser.org/terrarium/) covers the app in detail.

## Running it

From the repository root:

```
cargo xtask terrarium   # installs iguana, then launches the dev server
```

Or directly from this directory:

```
npm install
npm run tauri dev
```

The Rust backend is a separate workspace at `src-tauri/`, excluded from the root workspace by default.

## License

Licensed under the GNU General Public License v3 or later (`LICENSE-GPL`).
