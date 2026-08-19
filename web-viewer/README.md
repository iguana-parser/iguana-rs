# @iguana-parser/web-viewer

`@iguana-parser/web-viewer` runs a WebAssembly parser in the browser and shows
its parse tree. The app reads `manifest.json`, loads the parser module, and
mounts `ParseView` from [`web-ui`](../web-ui) through a `WasmBackend`. Parsing
runs in the page and does not require a server-side parser.

Each deployment serves one grammar-specific WebAssembly module and manifest.
The viewer code is independent of that grammar.

## Development

The viewer needs a bundle, consisting of the WebAssembly module and its
manifest, served as static files beside it. Produce one and place it in
`public/`. Install the root npm workspace dependencies once with `npm install`,
then run:

```sh
# Build the Iggy WebAssembly bundle into target/wasm/iggy
cargo xtask wasm

# Refresh the viewer's ignored local bundle
npm run stage:iggy --workspace web-viewer
```

Change to `web-viewer`, then run the development server or build the static
site:

```sh
cd web-viewer
npm run check
npm run dev      # Serve the app on http://localhost:5174
npm run build    # Write the app embedded by the `iguana` binary
```

The `public/` bundle is ignored by Git because it is generated output. The
viewer build under `iguana/viewer-dist/` is committed because the `iguana`
binary embeds it. The build removes the local manifest and WebAssembly module
from that committed output, leaving the embedded viewer grammar-independent.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
