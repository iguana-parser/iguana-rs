# tree-widget

The embeddable parse-tree graph: `mountParseTreeGraph(container, sexprText)`
parses a printed parse-tree s-expression (the generated `to_sexpr` output,
sharing labels included) and renders it as the interactive Cytoscape tree the
parse view shows, reusing the shared graph core in `web-ui`.

Built for static hosts that show verified parse trees as text and want a
graph view next to them, like the iguana website's docs pages. The host owns the
surrounding chrome (tabs, panel sizing) and the styling; the widget owns the
Cytoscape instance and renders the shared control strip (zoom and fit by
default, expand-all and PNG export optional), unstyled, for the host's CSS
to skin.

Cytoscape and cytoscape-tidytree are not bundled: they resolve at runtime
through an importmap on the host page, pinned to the same versions as the
other embedded apps. Build with `npm run build`; the output is a single
`dist/tree-widget.js` ES module.
