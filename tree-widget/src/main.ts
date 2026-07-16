import cytoscape from "cytoscape";
import tidytree from "cytoscape-tidytree";
import {
  parseSexprTree,
  buildParseTreeElements,
  createGraph,
  createGraphControls,
  sppfNodeStyles,
  edgeStyles,
  setupGraphTooltip,
  adjustZoomGraph,
  resetViewGraph,
  GraphCollapseManager,
  highlightOutgoingEdges,
  clearEdgeHighlights,
  highlightClickedEdge,
  downloadPng,
} from "@iguana-parser/web-ui/graph";

// The embeddable parse-tree graph: the graph tab of the parse view, distilled
// to a function a static page can call. The host owns the surrounding chrome
// (tabs, panel sizing) and the styling; this module owns the Cytoscape
// instance, its interactions, and the shared control strip, reusing the graph
// core so the docs site renders trees exactly as Terrarium and the playground
// do.

cytoscape.use(tidytree);

export interface ParseTreeGraphOptions {
  // Optional controls; zoom and fit always render. The shared strip
  // (createGraphControls) keeps the buttons consistent with the parse view.
  controls?: {
    expandAll?: boolean;
    exportPng?: boolean;
  };
}

export interface ParseTreeGraphHandle {
  zoomIn(): void;
  zoomOut(): void;
  resetView(): void;
  expandAll(): void;
  exportPng(name?: string): void;
  // Re-reads the container size; call after the container becomes visible or
  // is resized (Cytoscape does not observe it).
  resize(): void;
  destroy(): void;
}

// Parses the printed s-expression and mounts the interactive graph into
// `container`, which must be visible and sized. Interactions match the parse
// view's graph tab: click a node to highlight its outgoing edges, click an
// edge to highlight it, click the background to clear, double-click a node to
// collapse or expand its subtree. Throws if the s-expression does not parse.
export function mountParseTreeGraph(
  container: HTMLElement,
  sexprText: string,
  options: ParseTreeGraphOptions = {},
): ParseTreeGraphHandle {
  const tree = parseSexprTree(sexprText);
  const cy = createGraph({
    container,
    // No spans: the printed s-expression carries none (see parseSexprTree).
    elements: buildParseTreeElements(tree, false),
    styles: [...sppfNodeStyles, ...edgeStyles],
    layout: "tree",
  });

  const collapse = new GraphCollapseManager();
  collapse.setCy(cy);
  const disposeTooltip = setupGraphTooltip(cy, container);

  let selectedNodeId: string | null = null;
  function clearSelection() {
    if (selectedNodeId) {
      cy.getElementById(selectedNodeId).removeClass("selected");
      selectedNodeId = null;
    }
    clearEdgeHighlights(cy);
  }

  cy.on("dbltap", "node", (event) => {
    collapse.toggleCollapse(event.target.id());
  });

  cy.on("tap", "node", (event) => {
    clearSelection();
    selectedNodeId = event.target.id();
    event.target.addClass("selected");
    highlightOutgoingEdges(cy, selectedNodeId!);
  });

  cy.on("tap", "edge", (event) => {
    clearSelection();
    highlightClickedEdge(cy, event.target.id());
  });

  cy.on("tap", (event) => {
    if (event.target === cy) clearSelection();
  });

  const disposeControls = createGraphControls(container, {
    zoomIn: () => adjustZoomGraph(cy, 1.2),
    zoomOut: () => adjustZoomGraph(cy, 1 / 1.2),
    fit: () => resetViewGraph(cy),
    expandAll: options.controls?.expandAll ? () => collapse.expandAll() : undefined,
    exportPng: options.controls?.exportPng ? () => downloadPng(cy, "parse-tree") : undefined,
  });

  return {
    zoomIn: () => adjustZoomGraph(cy, 1.2),
    zoomOut: () => adjustZoomGraph(cy, 1 / 1.2),
    resetView: () => resetViewGraph(cy),
    expandAll: () => collapse.expandAll(),
    exportPng: (name = "parse-tree") => downloadPng(cy, name),
    resize: () => cy.resize(),
    destroy: () => {
      // The wheel listener lives on the container, not the instance, so
      // cy.destroy() does not remove it; createGraph stashes a disposer.
      cy.scratch("_disposeWheel")?.();
      disposeControls();
      disposeTooltip();
      cy.destroy();
    },
  };
}
