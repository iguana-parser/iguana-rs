# tree-widget

`tree-widget` embeds Iguana's interactive parse-tree graph in a static page.
`mountParseTreeGraph(container, sexprText)` reads the s-expression produced by
generated `to_sexpr` functions, including subtree-sharing markers, and renders
the same Cytoscape graph used by `ParseView`.

The host owns the surrounding controls, sizing, and styles. The widget owns the
Cytoscape instance and its interactions. Zoom and fit controls are always
shown; expand-all and PNG export controls are optional. The mount function
returns a handle for controlling, resizing, and destroying the graph.

Cytoscape and `cytoscape-tidytree` are external dependencies resolved by the
host's import map. Install dependencies at the repository root, then check and
build the widget:

```sh
cd tree-widget
npm run check
npm run build
```

The build output is the single ES module `dist/tree-widget.js`.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
