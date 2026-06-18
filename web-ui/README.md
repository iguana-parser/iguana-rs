# @iguana-parser/web-ui

The shared web UI for iguana's frontends. Its `ParseView` component presents a parse tree three ways: a collapsible tree, a graph, and an interactive s-expression, with selection synced to an input editor. The same component drives Terrarium's parse mode and the web viewer, so there is one renderer to maintain.

The renderer is host-agnostic. It reaches a parser through a `ParserBackend` interface rather than calling any host API directly, so it runs unchanged against either a subprocess parser or a wasm one. The package ships the wasm-backed implementation (`WasmBackend`); Terrarium supplies its own subprocess-backed one. Host-only chrome (graph pop-out, profiling, status and log hooks, PNG export) is taken as optional injected callbacks, so a host that omits them simply does not render the corresponding controls.

## What it exports

- `ParseView` — the renderer component.
- `InputEditor`, `NonterminalPicker` — the input editor and start-nonterminal picker it pairs with.
- `ParserBackend`, `ParseOutput`, `ParseTreeResult` — the backend contract and its data types.
- `WasmBackend` — a `ParserBackend` over a wasm-compiled parser, taking the module's `parse` as an injected function.
- The cytoscape graph helpers, `GraphCollapseManager`, `buildParseTreeElements`, and `downloadPng`.

## Usage

The package is consumed as source through the repository's npm workspace; there is no build step. It declares `svelte`, `cytoscape`, `cytoscape-tidytree`, `monaco-editor`, and `lucide-svelte` as peer dependencies, so the host provides a single copy of each. A host registers the cytoscape `tidytree` layout and sets up Monaco's worker before mounting `ParseView`.

## License

Licensed under either of MIT (`LICENSE-MIT`) or Apache 2.0 (`LICENSE-APACHE`), at your option.
