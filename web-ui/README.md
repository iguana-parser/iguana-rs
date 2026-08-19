# @iguana-parser/web-ui

`@iguana-parser/web-ui` contains the shared Svelte components and TypeScript
interfaces used by Iguana's web frontends. `ParseView` presents a parse tree as
a collapsible tree, a graph, and an interactive s-expression. `DesignView`
provides the Monaco-based Iggy editor used by Terrarium and the website.

The components reach parser and language-server functions through backend
interfaces rather than host APIs. `WasmBackend` runs a WebAssembly parser, and
`WasmLspBackend` runs the Iggy analysis functions compiled to WebAssembly.
Terrarium supplies Tauri-backed implementations of the same interfaces.

## What the package exports

- **Parser UI.** `ParseView`, `InputEditor`, and `NonterminalPicker` provide the
  parser input and parse-tree views.
- **Grammar UI.** `DesignView` provides grammar editing and navigation.
- **Parser backends.** `ParserBackend`, `ParseOutput`, `ParseTreeResult`, and
  `WasmBackend` define and implement parser access.
- **Language backends.** `LspBackend`, its result types, and `WasmLspBackend`
  define and implement grammar analysis.
- **Graph API.** The `@iguana-parser/web-ui/graph` export provides the plain
  TypeScript parse-tree graph helpers without importing the Svelte components.

## Workspace integration

The repository's npm workspace consumes the package as source, so the package
has no separate build step. A host provides the peer dependencies, registers
the Cytoscape `tidytree` layout, and configures Monaco's worker before mounting
the components.

Install dependencies at the repository root. Run the package check while
iterating, then check every npm workspace because Terrarium, both viewers, and
the tree widget consume this package directly:

```sh
npm run check --workspace web-ui
npm run check
```

Before changing a shared component, identify its class bindings, event
handlers, and conditional behavior in every consumer. Keep Tauri-specific code
in `terrarium/src/lib/`. Put a component or helper in this package when a
browser application also needs it.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
