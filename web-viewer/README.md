# @iguana-parser/web-viewer

A static web app that runs a wasm-compiled Iguana parser in the browser and renders the result. It is a thin host around the `ParseView` renderer from [`web-ui`](../web-ui): on load it reads `manifest.json`, loads the wasm module, constructs a `WasmBackend`, and mounts `ParseView`. There is no server and no Tauri; parsing happens in the page. Grammar editing is out of scope (that is Terrarium's job).

The app targets a single, fixed grammar per deployment. The wasm module and the manifest are grammar-specific and ship alongside the viewer; the viewer code itself is grammar-independent.

## Running it

The viewer needs a bundle (the wasm module plus its manifest) served as static files next to it. Produce one and place it in `public/`:

```
# Build the iggy wasm bundle into target/wasm/iggy
cargo xtask wasm

# Copy the bundle into the viewer's static dir
mkdir -p web-viewer/public/wasm
cp target/wasm/iggy/manifest.json web-viewer/public/manifest.json
cp -r target/wasm/iggy/wasm/pkg web-viewer/public/wasm/pkg
```

Then run the dev server or build the static site:

```
npm run dev      # from web-viewer/, serves on http://localhost:5174
npm run build    # emits ../iguana/viewer-dist/ (committed, embedded in the iguana binary)
```

The `public/` bundle is git-ignored, since it is generated output rather than source.

## License

Licensed under either of MIT (`LICENSE-MIT`) or Apache 2.0 (`LICENSE-APACHE`), at your option.
