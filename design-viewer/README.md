# @iguana-parser/design-viewer

A static web app that mounts the shared read-only design view over a grammar: Monaco driven by the iggy language server compiled to wasm. It is a thin host around the design view in [`web-ui`](../web-ui); the website's grammar pages embed the built app.

## Running it

```
npm run dev      # from design-viewer/, dev server
npm run build    # static build into dist/
```

## License

Licensed under either of MIT (`LICENSE-MIT`) or Apache 2.0 (`LICENSE-APACHE`), at your option.
