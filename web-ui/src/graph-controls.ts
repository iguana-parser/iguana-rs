// The graph control strip, shared by every host that shows a Cytoscape
// graph (Terrarium, the web viewer, the tree widget), so the buttons, their
// icons, and their order are one implementation. A button renders only if
// its action is provided, which is how a host picks its set: the docs pass
// zoom and fit, the parse view passes everything. Styling is the host's:
// the strip renders as .graph-controls with plain button children, and each
// host skins those selectors.

export interface GraphControlsActions {
  zoomIn: () => void;
  zoomOut: () => void;
  // Fits the whole graph back into the viewport.
  fit: () => void;
  // Undoes double-click collapses. The parse view passes this. The docs
  // omit it, since collapsing is undiscoverable there.
  expandAll?: () => void;
  exportPng?: () => void;
  popOut?: () => void;
}

const ICON_ATTRS =
  'width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" ' +
  'stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"';

const ICONS = {
  zoomIn: `<svg ${ICON_ATTRS}><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35M11 8v6M8 11h6"/></svg>`,
  zoomOut: `<svg ${ICON_ATTRS}><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35M8 11h6"/></svg>`,
  fit: `<svg ${ICON_ATTRS}><path d="M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2"/></svg>`,
  expandAll: `<svg ${ICON_ATTRS}><path d="m15 15 6 6m0 0v-4.8m0 4.8h-4.8M9 9 3 3m0 0v4.8M3 3h4.8M15 9l6-6m0 0v4.8M21 3h-4.8M9 15l-6 6m0 0v-4.8M3 21h4.8"/></svg>`,
  exportPng: `<svg ${ICON_ATTRS}><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>`,
  popOut: `<svg ${ICON_ATTRS}><path d="M15 3h6v6M10 14 21 3M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>`,
};

// Builds the strip inside `container` and returns a disposer that removes it.
export function createGraphControls(
  container: HTMLElement,
  actions: GraphControlsActions,
): () => void {
  const strip = document.createElement("div");
  strip.className = "graph-controls";

  function button(title: string, icon: string, action: () => void) {
    const b = document.createElement("button");
    b.type = "button";
    b.title = title;
    b.setAttribute("aria-label", title);
    b.innerHTML = icon;
    b.addEventListener("click", action);
    strip.appendChild(b);
  }

  button("Zoom in", ICONS.zoomIn, actions.zoomIn);
  button("Zoom out", ICONS.zoomOut, actions.zoomOut);
  button("Fit to view", ICONS.fit, actions.fit);
  if (actions.expandAll) {
    button("Expand all (double-click a node to collapse)", ICONS.expandAll, actions.expandAll);
  }
  if (actions.exportPng) button("Export as PNG", ICONS.exportPng, actions.exportPng);
  if (actions.popOut) button("Pop out", ICONS.popOut, actions.popOut);

  container.appendChild(strip);
  return () => strip.remove();
}
