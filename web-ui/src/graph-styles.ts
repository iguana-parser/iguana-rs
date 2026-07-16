import cytoscape from "cytoscape";

// Use any for stylesheet types since cytoscape doesn't export them correctly
type Stylesheet = any;
type ElementDefinition = any;
type Core = cytoscape.Core;

// Label truncation constants
export const LABEL_MAX_LENGTH = 20;           // Default for terminals/nonterminals
export const INTERMEDIATE_MAX_LENGTH = 30;    // Longer for intermediate nodes (grammar slots)

export function truncateLabel(label: string, maxLen: number = LABEL_MAX_LENGTH): string {
  if (label.length <= maxLen) return label;
  return label.substring(0, maxLen - 3) + "...";
}

// Tooltip management for graph nodes
export function setupGraphTooltip(
  cy: Core,
  container: HTMLElement
): () => void {
  // Create tooltip element
  const tooltip = document.createElement("div");
  tooltip.className = "graph-tooltip";
  tooltip.style.cssText = `
    position: fixed;
    background: #252526;
    border: 1px solid #454545;
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 11px;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    color: #d4d4d4;
    pointer-events: none;
    z-index: 10000;
    display: none;
    max-width: 400px;
    word-wrap: break-word;
    white-space: pre-wrap;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  `;
  document.body.appendChild(tooltip);

  const showTooltip = (event: cytoscape.EventObject) => {
    const node = event.target;
    const fullLabel = node.data("fullLabel");
    const label = node.data("label");

    // Only show tooltip if label was truncated
    if (!fullLabel || fullLabel === label) return;

    tooltip.textContent = fullLabel;
    tooltip.style.display = "block";

    // Position near cursor
    const renderedPos = event.renderedPosition || event.position;
    const containerRect = container.getBoundingClientRect();
    tooltip.style.left = `${containerRect.left + renderedPos.x + 15}px`;
    tooltip.style.top = `${containerRect.top + renderedPos.y + 15}px`;
  };

  const hideTooltip = () => {
    tooltip.style.display = "none";
  };

  const moveTooltip = (event: cytoscape.EventObject) => {
    if (tooltip.style.display === "none") return;
    const renderedPos = event.renderedPosition || event.position;
    const containerRect = container.getBoundingClientRect();
    tooltip.style.left = `${containerRect.left + renderedPos.x + 15}px`;
    tooltip.style.top = `${containerRect.top + renderedPos.y + 15}px`;
  };

  cy.on("mouseover", "node", showTooltip);
  cy.on("mouseout", "node", hideTooltip);
  cy.on("mousemove", "node", moveTooltip);

  // Return cleanup function
  return () => {
    cy.off("mouseover", "node", showTooltip);
    cy.off("mouseout", "node", hideTooltip);
    cy.off("mousemove", "node", moveTooltip);
    tooltip.remove();
  };
}

// Base styles shared by all graph nodes
const baseNodeStyle = {
  label: "data(label)",
  "text-valign": "center",
  "text-halign": "center",
  "font-size": "9px",
  color: "#d4d4d4",
  "border-width": 1,
  width: "label",
  shape: "round-rectangle",
};

// Base styles shared by all graph edges
const baseEdgeStyle = {
  width: 1,
  "line-color": "#555",
  "target-arrow-color": "#555",
  "target-arrow-shape": "triangle",
  "curve-style": "bezier",
  "arrow-scale": 0.8,
};

// Color definitions for node types
const nodeColors = {
  nonterminal: { bg: "#2d4a3d", border: "#4ec9b0", selectedBg: "#3a5f50", selectedBorder: "#7fffaa" },
  intermediate: { bg: "#2d3a4d", border: "#569cd6", selectedBg: "#3a4d60", selectedBorder: "#7eb8ff" },
  terminal: { bg: "#4d3a2d", border: "#ce9178", selectedBg: "#5f4a3a", selectedBorder: "#ffb07a" },
  packed: { bg: "#666", selectedBg: "#888", selectedBorder: "#aaa" },
  ambiguous: { bg: "#4d2d2d", border: "#e05050", selectedBg: "#5f3a3a", selectedBorder: "#ff7a7a" },
};

// Disable the default "active" state indicator (black overlay on click)
const disableActiveStyles: Stylesheet[] = [
  {
    selector: "node:active",
    style: { "overlay-opacity": 0 },
  },
  {
    selector: "edge:active",
    style: { "overlay-opacity": 0 },
  },
  {
    selector: "core",
    style: { "active-bg-opacity": 0 },
  },
];

// SPPF node styles by type
export const sppfNodeStyles: Stylesheet[] = [
  ...disableActiveStyles,
  {
    selector: "node",
    style: {
      ...baseNodeStyle,
      "text-wrap": "wrap",  // Needed for \n to create line breaks
      "background-color": "#3c3c3c",
      "border-color": "#555",
      height: "label",
      "padding-left": "9px",
      "padding-right": "9px",
      "padding-top": "5px",
      "padding-bottom": "5px",
    },
  },
  {
    selector: "node.nonterminal, node[kind='Nonterminal']",
    style: {
      "background-color": nodeColors.nonterminal.bg,
      "border-color": nodeColors.nonterminal.border,
    },
  },
  {
    selector: "node.intermediate, node[kind='Intermediate']",
    style: {
      "background-color": nodeColors.intermediate.bg,
      "border-color": nodeColors.intermediate.border,
      shape: "rectangle",
    },
  },
  {
    selector: "node.terminal, node[kind='Terminal']",
    style: {
      "background-color": nodeColors.terminal.bg,
      "border-color": nodeColors.terminal.border,
    },
  },
  {
    selector: "node.token, node[kind='Token']",
    style: {
      "background-color": nodeColors.terminal.bg,
      "border-color": nodeColors.terminal.border,
    },
  },
  {
    selector: "node.packed",
    style: {
      width: 12,
      height: 12,
      "background-color": "#666",
      "border-width": 0,
      label: "",
    },
  },
  {
    selector: "node.collapsed",
    style: {
      // Use dashed border instead of double - keeps same width so arrows don't move
      "border-style": "dashed",
    },
  },
  // Selected nodes: a thicker double-line border on top of the per-kind
  // color shift below. Strong enough to spot at any zoom level.
  {
    selector: "node.selected",
    style: {
      "border-width": 3,
      "border-style": "double",
    },
  },
  {
    selector: "node.nonterminal.selected, node[kind='Nonterminal'].selected",
    style: {
      "background-color": nodeColors.nonterminal.selectedBg,
      "border-color": nodeColors.nonterminal.selectedBorder,
    },
  },
  {
    selector: "node.intermediate.selected, node[kind='Intermediate'].selected",
    style: {
      "background-color": nodeColors.intermediate.selectedBg,
      "border-color": nodeColors.intermediate.selectedBorder,
    },
  },
  {
    selector: "node.terminal.selected, node[kind='Terminal'].selected",
    style: {
      "background-color": nodeColors.terminal.selectedBg,
      "border-color": nodeColors.terminal.selectedBorder,
    },
  },
  {
    selector: "node.token.selected, node[kind='Token'].selected",
    style: {
      "background-color": nodeColors.terminal.selectedBg,
      "border-color": nodeColors.terminal.selectedBorder,
    },
  },
  {
    selector: "node.packed.selected, node[kind='Packed'].selected",
    style: {
      "background-color": nodeColors.packed.selectedBg,
      "border-width": 2,
      "border-color": nodeColors.packed.selectedBorder,
    },
  },
  // Ambiguous node styles (override the base colors with red). Covers both the
  // SPPF `.ambiguous` flag on Nonterminal/Intermediate nodes and the parse-tree
  // `.amb` kind (Amb wrappers around alternative derivations).
  {
    selector: "node.ambiguous, node.amb",
    style: {
      "background-color": nodeColors.ambiguous.bg,
      "border-color": nodeColors.ambiguous.border,
    },
  },
  {
    selector: "node.ambiguous.selected, node.amb.selected",
    style: {
      "background-color": nodeColors.ambiguous.selectedBg,
      "border-color": nodeColors.ambiguous.selectedBorder,
    },
  },
  // Shared span styles (nodes with same span as another node - orange-red border)
  {
    selector: "node.shared-span",
    style: {
      "border-color": "#e07030",
      "border-width": 2,
    },
  },
  {
    selector: "node.shared-span.selected",
    style: {
      "border-color": "#ff9050",
    },
  },
];

// GSS node styles
export const gssNodeStyles: Stylesheet[] = [
  ...disableActiveStyles,
  {
    selector: "node",
    style: {
      ...baseNodeStyle,
      "background-color": nodeColors.nonterminal.bg,
      "border-color": nodeColors.nonterminal.border,
      height: 22,
      "padding-left": "8px",
      "padding-right": "8px",
    },
  },
  {
    selector: "node.current",
    style: {
      "border-width": 3,
      "border-color": nodeColors.nonterminal.border,
      "background-color": "#3d5a4d",
    },
  },
];

// SPPF edge styles (no labels)
export const edgeStyles: Stylesheet[] = [
  {
    selector: "edge",
    style: { ...baseEdgeStyle },
  },
  // Edges into a shared node (a node with several parents in the DAG), in
  // the pink of the s-expression's #N=/#N# sharing marks, so sharing reads
  // the same in both views. The selection styles below override it.
  {
    selector: "edge.shared",
    style: {
      "line-color": "#c586c0",
      "target-arrow-color": "#c586c0",
    },
  },
  {
    selector: "edge.edge-selected-nonterminal",
    style: {
      "line-color": nodeColors.nonterminal.selectedBorder,
      "target-arrow-color": nodeColors.nonterminal.selectedBorder,
    },
  },
  {
    selector: "edge.edge-selected-intermediate",
    style: {
      "line-color": nodeColors.intermediate.selectedBorder,
      "target-arrow-color": nodeColors.intermediate.selectedBorder,
    },
  },
  {
    selector: "edge.edge-selected-terminal",
    style: {
      "line-color": nodeColors.terminal.selectedBorder,
      "target-arrow-color": nodeColors.terminal.selectedBorder,
    },
  },
  {
    selector: "edge.edge-selected-packed",
    style: {
      "line-color": nodeColors.packed.selectedBorder,
      "target-arrow-color": nodeColors.packed.selectedBorder,
    },
  },
  {
    selector: "edge.edge-clicked",
    style: {
      "line-color": "#999",
      "target-arrow-color": "#999",
    },
  },
  {
    selector: "edge.edge-selected-ambiguous",
    style: {
      "line-color": nodeColors.ambiguous.selectedBorder,
      "target-arrow-color": nodeColors.ambiguous.selectedBorder,
    },
  },
  // Edges from ambiguous nodes (shown in red)
  {
    selector: "edge.edge-ambiguous",
    style: {
      "line-color": nodeColors.ambiguous.border,
      "target-arrow-color": nodeColors.ambiguous.border,
    },
  },
];

// GSS edge styles (with labels)
export const gssEdgeStyles: Stylesheet = {
  selector: "edge",
  style: {
    ...baseEdgeStyle,
    label: "data(label)",
    "font-size": "9px",
    color: "#888",
    "text-rotation": "autorotate",
    "text-margin-y": -10,
  },
};

// Viewport state for preserving zoom/pan across re-renders
export interface Viewport {
  zoom: number;
  pan: { x: number; y: number };
}

// Graph creation options
export interface GraphOptions {
  container: HTMLElement;
  elements: ElementDefinition[];
  styles: Stylesheet[];
  // "sppf"/"gss" use dagre (general DAG layout). "tree" uses the cytoscape-tidytree
  // extension (van der Ploeg's linear-time Reingold-Tilford), far cheaper than
  // dagre on the near-tree-shaped parse-tree graph.
  layout?: "sppf" | "gss" | "tree";
  viewport?: Viewport;  // If provided, restore this viewport instead of auto-fitting
}

// Layout config for the parse-tree graph, shared by the initial build and the
// in-place reload (toggles) so a relayout matches the first layout. `fit` is
// overridden per call: the reload preserves the viewport on a toggle.
export const PARSE_TREE_LAYOUT = {
  name: "tidytree",
  direction: "TB",
  horizontalSpacing: 16,
  verticalSpacing: 30,
  nodeDimensionsIncludeLabels: true,  // nodes are label-sized
  fit: true,
  padding: 30,
};

// Node count past which the parse-tree graph switches from Canvas2D to the
// WebGL renderer. Canvas2D keeps text crisp but redraws on the CPU each frame,
// so a large tree pans more smoothly on the GPU even with softer atlas text.
export const PARSE_TREE_WEBGL_NODE_THRESHOLD = 1000;

// Cap zoom level after fit to prevent huge nodes on small graphs
export const MAX_FIT_ZOOM = 1.0;

export function capZoom(cyInstance: Core) {
  if (cyInstance.zoom() > MAX_FIT_ZOOM) {
    cyInstance.zoom(MAX_FIT_ZOOM);
    cyInstance.center();
  }
}

// Multiply a graph's zoom by a factor (zoom in/out controls)
export function adjustZoomGraph(graph: Core | null, factor: number) {
  if (graph) {
    graph.zoom(graph.zoom() * factor);
  }
}

// Fit a graph to its contents, then cap the zoom so small graphs don't over-zoom
export function resetViewGraph(graph: Core | null) {
  if (graph) {
    graph.fit();
    capZoom(graph);
  }
}

// Get current viewport from a graph instance
export function getViewport(cyInstance: Core): Viewport {
  return {
    zoom: cyInstance.zoom(),
    pan: cyInstance.pan(),
  };
}

export function createGraph(options: GraphOptions): Core {
  const { container, elements, styles, layout = "sppf", viewport } = options;

  // Edges carry a `source`; everything else is a node.
  const nodeCount = elements.filter((el) => el.data.source === undefined).length;

  const layoutConfig = layout === "tree"
    ? PARSE_TREE_LAYOUT
    : {
        name: "dagre",
        rankDir: layout === "gss" ? "BT" : "TB",
        nodeSep: layout === "gss" ? 50 : 30,
        rankSep: layout === "gss" ? 60 : 50,
      };

  // Renderer choice. SPPF and GSS use the WebGL path (Cytoscape 3.31+): it
  // GPU-batches drawing and scales to large, edge-dense graphs, but it
  // rasterizes labels into a fixed texture atlas and resamples them, which
  // drops font hinting and reads as soft text. The parse-tree graph uses the
  // plain Canvas2D renderer, which redraws text each frame at devicePixelRatio,
  // so the primary text-heavy view stays crisp and scrolls smoothly. A tree past
  // PARSE_TREE_WEBGL_NODE_THRESHOLD nodes switches back to WebGL, trading text
  // sharpness for GPU scroll speed. The caller rebuilds the instance when a new
  // parse crosses the threshold, so the choice tracks the current tree.
  const webgl = layout !== "tree" || nodeCount >= PARSE_TREE_WEBGL_NODE_THRESHOLD;

  const cyInstance = cytoscape({
    container,
    elements,
    style: styles,
    layout: layoutConfig as any,
    userZoomingEnabled: false,  // Disable built-in wheel zoom, we handle it manually
    userPanningEnabled: true,
    boxSelectionEnabled: false,
    // Spread as `any` because the bundled @types/cytoscape predates `renderer`.
    ...({ renderer: { name: "canvas", webgl } } as any),
  });

  // Expose the chosen renderer for the caller: it reads `webgl` back to show a
  // badge and to decide whether a new parse needs a rebuild (the renderer can't
  // change on a live instance). The public scratch API keeps this off the typed
  // surface.
  cyInstance.scratch("_renderer", { webgl });

  // Enable two-finger trackpad scrolling to pan, pinch to zoom. The listener is
  // on the container, not the instance, so cy.destroy() does not remove it; stash
  // a disposer so a caller that rebuilds the graph (a new parse) can strip it and
  // avoid stacking one listener per rebuild.
  if (container) {
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      if (e.ctrlKey) {
        // Pinch-to-zoom (ctrlKey is set for pinch gestures on macOS)
        const zoomFactor = 1 - e.deltaY * 0.01;
        const newZoom = cyInstance.zoom() * zoomFactor;
        cyInstance.zoom({
          level: newZoom,
          renderedPosition: { x: e.offsetX, y: e.offsetY },
        });
      } else {
        // Two-finger scroll to pan
        const pan = cyInstance.pan();
        cyInstance.pan({
          x: pan.x - e.deltaX,
          y: pan.y - e.deltaY,
        });
      }
    };
    container.addEventListener('wheel', onWheel, { passive: false });
    cyInstance.scratch("_disposeWheel", () => container.removeEventListener('wheel', onWheel));
  }

  // Restore viewport if provided, otherwise cap initial zoom
  if (viewport) {
    cyInstance.zoom(viewport.zoom);
    cyInstance.pan(viewport.pan);
  } else {
    capZoom(cyInstance);
  }

  return cyInstance;
}

// Edge selection class names
const EDGE_SELECTED_CLASSES = ['edge-selected-nonterminal', 'edge-selected-intermediate', 'edge-selected-terminal', 'edge-selected-packed', 'edge-selected-ambiguous', 'edge-clicked'];

// Get the appropriate edge selection class based on node type
function getEdgeClassForNode(node: cytoscape.NodeSingular): string {
  // Ambiguous nodes take priority for edge coloring
  if (node.data('ambiguous') || node.hasClass('ambiguous')) {
    return 'edge-selected-ambiguous';
  }
  const kind = node.data('kind');
  if (kind === 'Packed' || node.hasClass('packed')) {
    return 'edge-selected-packed';
  } else if (kind === 'Nonterminal' || node.hasClass('nonterminal')) {
    return 'edge-selected-nonterminal';
  } else if (kind === 'Intermediate' || node.hasClass('intermediate')) {
    return 'edge-selected-intermediate';
  } else if (kind === 'Terminal' || kind === 'Token' || node.hasClass('terminal') || node.hasClass('token')) {
    return 'edge-selected-terminal';
  }
  return 'edge-selected-nonterminal'; // default
}

// Highlight outgoing edges of a selected node
export function highlightOutgoingEdges(cy: Core, nodeId: string) {
  const node = cy.getElementById(nodeId);
  if (node.empty()) return;

  const edgeClass = getEdgeClassForNode(node);
  const outgoingEdges = node.outgoers('edge');
  outgoingEdges.addClass(edgeClass);
}

// Clear all edge selection highlighting
export function clearEdgeHighlights(cy: Core) {
  cy.edges().removeClass(EDGE_SELECTED_CLASSES);
}

// Highlight a single clicked edge
export function highlightClickedEdge(cy: Core, edgeId: string) {
  clearEdgeHighlights(cy);
  cy.getElementById(edgeId).addClass('edge-clicked');
}
