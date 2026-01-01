<script lang="ts">
  import { onMount, tick } from "svelte";
  import { page } from "$app/stores";
  import { listen, emit } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import cytoscape from "cytoscape";
  import dagre from "cytoscape-dagre";
  import { ZoomIn, ZoomOut, Maximize2 } from "lucide-svelte";
  import {
    sppfNodeStyles,
    gssNodeStyles,
    edgeStyles,
    gssEdgeStyles,
    capZoom,
    createGraph,
  } from "$lib/graph-styles";
  import { createMaximizeToggle } from "$lib/window-utils";
  import type {
    SPPF,
    GSS,
    DebugSPPFNode,
    DebugGSSNode,
    DebugGSSEdge,
  } from "../../bindings";

  cytoscape.use(dagre);

  const toggleMaximize = createMaximizeToggle();

  function startDrag(e: MouseEvent) {
    if (e.button === 0) {
      getCurrentWindow().startDragging();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      getCurrentWindow().close();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      emit("debug-step-back");
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      emit("debug-step-forward");
    }
  }

  // Get graph type from URL params
  const graphType = $page.url.searchParams.get("type") as
    | "sppf"
    | "gss"
    | "debugSppf"
    | "debugGss"
    | null;

  let container: HTMLDivElement;
  let cy: cytoscape.Core | null = null;

  // Data for different graph types (received via events)
  let sppfData: SPPF | null = $state(null);
  let gssData: GSS | null = $state(null);
  let debugSppfNodes: DebugSPPFNode[] = $state([]);
  let debugSppfCurrentNodeId: number | null = $state(null);
  let debugGssNodes: DebugGSSNode[] = $state([]);
  let debugGssEdges: DebugGSSEdge[] = $state([]);
  let debugGssCurrentNodeId: number | null = $state(null);

  function getTitle(): string {
    switch (graphType) {
      case "sppf":
      case "debugSppf":
        return "SPPF";
      case "gss":
      case "debugGss":
        return "GSS";
      default:
        return "Graph";
    }
  }

  function buildElements(): cytoscape.ElementDefinition[] {
    if (graphType === "sppf" && sppfData) {
      return [
        ...sppfData.nodes.map((node) => ({
          data: {
            id: `n${node.id}`,
            label: node.label || (node.kind === "Packed" ? "" : ""),
          },
          classes: node.kind.toLowerCase(),
        })),
        ...sppfData.edges.map((edge, i) => ({
          data: {
            id: `e${i}`,
            source: `n${edge.src}`,
            target: `n${edge.dest}`,
          },
        })),
      ];
    } else if (graphType === "gss" && gssData) {
      return [
        ...gssData.nodes.map((node) => ({
          data: {
            id: `n${node.id}`,
            label: node.label,
          },
        })),
        ...gssData.edges.map((edge, i) => ({
          data: {
            id: `e${i}`,
            source: `n${edge.src}`,
            target: `n${edge.dest}`,
            label: edge.label,
          },
        })),
      ];
    } else if (graphType === "debugSppf") {
      const elements: cytoscape.ElementDefinition[] = [];
      const nodeMap = new Map<number, DebugSPPFNode>();
      for (const node of debugSppfNodes) {
        nodeMap.set(node.id, node);
      }
      const reachableIds = new Set<number>();
      if (
        debugSppfCurrentNodeId !== null &&
        nodeMap.has(debugSppfCurrentNodeId)
      ) {
        const queue = [debugSppfCurrentNodeId];
        while (queue.length > 0) {
          const id = queue.shift()!;
          if (reachableIds.has(id)) continue;
          reachableIds.add(id);
          const node = nodeMap.get(id);
          if (node) {
            for (const childId of node.children) {
              queue.push(childId);
            }
          }
        }
      }
      for (const node of debugSppfNodes) {
        if (!reachableIds.has(node.id)) continue;
        elements.push({
          data: {
            id: `n${node.id}`,
            label: `(${node.label}, ${node.left_extent}, ${node.right_extent})`,
            kind: node.kind,
          },
          classes: node.kind.toLowerCase(),
        });
      }
      for (const node of debugSppfNodes) {
        if (!reachableIds.has(node.id)) continue;
        for (const childId of node.children) {
          if (reachableIds.has(childId)) {
            elements.push({
              data: {
                id: `e${node.id}-${childId}`,
                source: `n${node.id}`,
                target: `n${childId}`,
              },
            });
          }
        }
      }
      return elements;
    } else if (graphType === "debugGss") {
      const elements: cytoscape.ElementDefinition[] = [];
      for (const node of debugGssNodes) {
        elements.push({
          data: {
            id: `n${node.id}`,
            label: node.label,
          },
          classes: debugGssCurrentNodeId === node.id ? "current" : "",
        });
      }
      for (let i = 0; i < debugGssEdges.length; i++) {
        const edge = debugGssEdges[i];
        elements.push({
          data: {
            id: `e${i}`,
            source: `n${edge.src}`,
            target: `n${edge.dest}`,
            label: edge.label,
          },
        });
      }
      return elements;
    }
    return [];
  }

  function renderGraph() {
    if (!container) return;

    const elements = buildElements();
    if (elements.length === 0) {
      // Clear the graph if no data
      if (cy) {
        cy.destroy();
        cy = null;
      }
      return;
    }

    if (cy) {
      cy.destroy();
    }

    const isGss = graphType === "gss" || graphType === "debugGss";
    cy = createGraph({
      container,
      elements,
      styles: isGss
        ? [...gssNodeStyles, gssEdgeStyles]
        : [...sppfNodeStyles, edgeStyles],
      layout: isGss ? "gss" : "sppf",
    });
  }

  // Re-render when data changes
  $effect(() => {
    // Track all data sources
    sppfData;
    gssData;
    debugSppfNodes;
    debugSppfCurrentNodeId;
    debugGssNodes;
    debugGssEdges;
    debugGssCurrentNodeId;

    if (container) {
      tick().then(() => renderGraph());
    }
  });

  onMount(() => {
    const unlisteners: Promise<() => void>[] = [];

    // Listen for data events based on graph type
    if (graphType === "sppf") {
      unlisteners.push(
        listen<SPPF>("graph-data-sppf", (event) => {
          sppfData = event.payload;
        })
      );
    } else if (graphType === "gss") {
      unlisteners.push(
        listen<GSS>("graph-data-gss", (event) => {
          gssData = event.payload;
        })
      );
    } else if (graphType === "debugSppf") {
      unlisteners.push(
        listen<{ nodes: DebugSPPFNode[]; current_node_id: number | null }>(
          "graph-data-debug-sppf",
          (event) => {
            debugSppfNodes = event.payload.nodes;
            debugSppfCurrentNodeId = event.payload.current_node_id;
          }
        )
      );
    } else if (graphType === "debugGss") {
      unlisteners.push(
        listen<{
          nodes: DebugGSSNode[];
          edges: DebugGSSEdge[];
          current_gss_node_id: number | null;
        }>("graph-data-debug-gss", (event) => {
          debugGssNodes = event.payload.nodes;
          debugGssEdges = event.payload.edges;
          debugGssCurrentNodeId = event.payload.current_gss_node_id;
        })
      );
    }

    // Signal that we're ready to receive data
    emit("graph-window-ready", { graphType });

    // Fit graph to container on resize
    let resizeTimeout: ReturnType<typeof setTimeout>;
    const resizeObserver = new ResizeObserver(() => {
      clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(() => {
        if (cy) {
          cy.resize();
          cy.fit();
          capZoom(cy);
        }
      }, 50);
    });
    if (container) {
      resizeObserver.observe(container);
    }

    return () => {
      unlisteners.forEach((p) => p.then((fn) => fn()));
      resizeObserver.disconnect();
      if (cy) cy.destroy();
    };
  });

  function zoomIn() {
    if (cy) cy.zoom(cy.zoom() * 1.2);
  }

  function zoomOut() {
    if (cy) cy.zoom(cy.zoom() / 1.2);
  }

  function resetView() {
    if (cy) {
      cy.fit();
      capZoom(cy);
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<svelte:head>
  <title>{getTitle()} - Terrarium</title>
</svelte:head>

<div class="graph-window">
  <!-- Title bar area for dragging, leaves space for traffic lights -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="title-bar" onmousedown={startDrag} ondblclick={toggleMaximize}>
    <div class="title-bar-left"></div>
    <div class="title-bar-center">
      <span class="title">{getTitle()}</span>
    </div>
    <div class="title-bar-right"></div>
  </div>
  <div class="graph-area">
    <div class="graph-container" bind:this={container}></div>
    <div class="graph-controls">
      <button onclick={zoomIn} title="Zoom In">
        <ZoomIn size={14} />
      </button>
      <button onclick={zoomOut} title="Zoom Out">
        <ZoomOut size={14} />
      </button>
      <button onclick={resetView} title="Reset View">
        <Maximize2 size={14} />
      </button>
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .graph-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1e1e1e;
    overflow: hidden;
  }

  .title-bar {
    display: flex;
    align-items: center;
    height: 38px;
    background: #1e1e1e;
    user-select: none;
  }

  .title-bar-left {
    width: 70px; /* Space for traffic lights */
    flex-shrink: 0;
  }

  .title-bar-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .title {
    font-size: 12px;
    color: #888;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .title-bar-right {
    width: 70px;
    flex-shrink: 0;
  }

  .graph-area {
    flex: 1;
    position: relative;
    min-height: 0;
  }

  .graph-container {
    width: 100%;
    height: 100%;
  }

  .graph-controls {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .graph-area:hover .graph-controls {
    opacity: 1;
  }

  .graph-controls button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: rgba(60, 60, 60, 0.8);
    border: none;
    border-radius: 4px;
    color: #888;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .graph-controls button:hover {
    background: rgba(80, 80, 80, 0.9);
    color: #fff;
  }
</style>
