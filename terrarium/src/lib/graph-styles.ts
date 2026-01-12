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
  // Ambiguous node styles (override the base colors with red)
  {
    selector: "node.ambiguous",
    style: {
      "background-color": nodeColors.ambiguous.bg,
      "border-color": nodeColors.ambiguous.border,
    },
  },
  {
    selector: "node.ambiguous.selected",
    style: {
      "background-color": nodeColors.ambiguous.selectedBg,
      "border-color": nodeColors.ambiguous.selectedBorder,
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
  layout?: "sppf" | "gss";
  viewport?: Viewport;  // If provided, restore this viewport instead of auto-fitting
}

// Cap zoom level after fit to prevent huge nodes on small graphs
export const MAX_FIT_ZOOM = 1.0;

export function capZoom(cyInstance: Core) {
  if (cyInstance.zoom() > MAX_FIT_ZOOM) {
    cyInstance.zoom(MAX_FIT_ZOOM);
    cyInstance.center();
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

  const cyInstance = cytoscape({
    container,
    elements,
    style: styles,
    layout: {
      name: "dagre",
      rankDir: layout === "gss" ? "BT" : "TB",
      nodeSep: layout === "gss" ? 50 : 30,
      rankSep: layout === "gss" ? 60 : 50,
    } as any,
    userZoomingEnabled: false,  // Disable built-in wheel zoom, we handle it manually
    userPanningEnabled: true,
    boxSelectionEnabled: false,
  });

  // Enable two-finger trackpad scrolling to pan, pinch to zoom
  if (container) {
    container.addEventListener('wheel', (e: WheelEvent) => {
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
    }, { passive: false });
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
