import cytoscape from "cytoscape";

// Use any for stylesheet types since cytoscape doesn't export them correctly
type Stylesheet = any;
type ElementDefinition = any;
type Core = cytoscape.Core;

// SPPF node styles by type
export const sppfNodeStyles: Stylesheet[] = [
  {
    selector: "node",
    style: {
      label: "data(label)",
      "text-valign": "center",
      "text-halign": "center",
      "font-size": "10px",
      "text-wrap": "wrap",
      "text-max-width": "80px",
      color: "#d4d4d4",
      "background-color": "#3c3c3c",
      "border-width": 1,
      "border-color": "#555",
      width: "label",
      height: 24,
      "padding-left": "8px",
      "padding-right": "8px",
      shape: "round-rectangle",
    },
  },
  {
    selector: "node.nonterminal, node[kind='Nonterminal']",
    style: {
      "background-color": "#2d4a3d",
      "border-color": "#4ec9b0",
    },
  },
  {
    selector: "node.intermediate, node[kind='Intermediate']",
    style: {
      "background-color": "#2d3a4d",
      "border-color": "#569cd6",
      shape: "rectangle",
    },
  },
  {
    selector: "node.terminal, node[kind='Terminal']",
    style: {
      "background-color": "#4d3a2d",
      "border-color": "#ce9178",
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
      "border-width": 3,
      "border-style": "double",
    },
  },
];

// GSS node styles
export const gssNodeStyles: Stylesheet[] = [
  {
    selector: "node",
    style: {
      label: "data(label)",
      "text-valign": "center",
      "text-halign": "center",
      "font-size": "10px",
      color: "#d4d4d4",
      "background-color": "#2d4a3d",
      "border-width": 1,
      "border-color": "#4ec9b0",
      width: "label",
      height: 24,
      "padding-left": "8px",
      "padding-right": "8px",
      shape: "round-rectangle",
    },
  },
  {
    selector: "node.current",
    style: {
      "border-width": 3,
      "border-color": "#4ec9b0",
      "background-color": "#3d5a4d",
    },
  },
];

// SPPF edge styles (no labels)
export const edgeStyles: Stylesheet = {
  selector: "edge",
  style: {
    width: 1,
    "line-color": "#555",
    "target-arrow-color": "#555",
    "target-arrow-shape": "triangle",
    "curve-style": "bezier",
    "arrow-scale": 0.8,
  },
};

// GSS edge styles (with labels)
export const gssEdgeStyles: Stylesheet = {
  selector: "edge",
  style: {
    label: "data(label)",
    "font-size": "9px",
    color: "#888",
    "text-rotation": "autorotate",
    "text-margin-y": -10,
    width: 1,
    "line-color": "#555",
    "target-arrow-color": "#555",
    "target-arrow-shape": "triangle",
    "curve-style": "bezier",
    "arrow-scale": 0.8,
  },
};

// Graph creation options
export interface GraphOptions {
  container: HTMLElement;
  elements: ElementDefinition[];
  styles: Stylesheet[];
  layout?: "sppf" | "gss";
}

// Cap zoom level after fit to prevent huge nodes on small graphs
export const MAX_FIT_ZOOM = 1.0;

export function capZoom(cyInstance: Core) {
  if (cyInstance.zoom() > MAX_FIT_ZOOM) {
    cyInstance.zoom(MAX_FIT_ZOOM);
    cyInstance.center();
  }
}

export function createGraph(options: GraphOptions): Core {
  const { container, elements, styles, layout = "sppf" } = options;

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
    userZoomingEnabled: true,
    userPanningEnabled: true,
    boxSelectionEnabled: false,
  });

  // Cap initial zoom and center after layout
  capZoom(cyInstance);

  return cyInstance;
}
