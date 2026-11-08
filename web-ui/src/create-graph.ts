// The one module in the package that imports Cytoscape as a value. Keeping it
// apart from graph-styles lets ParseView load Cytoscape on demand.
import cytoscape from "cytoscape";
import {
  capZoom,
  PARSE_TREE_LAYOUT,
  PARSE_TREE_WEBGL_NODE_THRESHOLD,
  type GraphOptions,
} from "./graph-styles";

type Core = cytoscape.Core;

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
