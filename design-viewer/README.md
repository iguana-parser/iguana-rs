# @iguana-parser/design-viewer

`@iguana-parser/design-viewer` is the read-only Iggy editor embedded by the
website's grammar pages. It mounts `DesignView` from [`web-ui`](../web-ui) and
runs the Iggy analysis functions in the browser through `iguana-lsp` compiled
to WebAssembly.

The app reads the grammar URL from the `src` query parameter. The website
embeds one build on every grammar page because the editor and language-server
module are independent of the grammar being displayed.

## Development

The development server needs a WebAssembly build of `iguana-lsp` and a grammar
under `public/`. Install the root npm workspace dependencies once with
`npm install`, then run from the repository root:

```sh
cd iguana-lsp/wasm
wasm-pack build --target web \
  --out-dir ../../design-viewer/public/wasm/pkg \
  --out-name iguana_lsp_wasm

cd ../../design-viewer
cp ../iggy/iggy.iggy public/iggy.iggy
npm run check
npm run dev
```

Open `http://localhost:5175/design-view/?src=/design-view/iggy.iggy`. Run
`npm run build` to write the static app and its local development assets to
`dist/`. The website deployment supplies its own grammar files beside the app.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
