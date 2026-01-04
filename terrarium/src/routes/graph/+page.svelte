<script lang="ts">
  import { onMount, tick } from "svelte";
  import { page } from "$app/stores";
  import { listen, emit } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import cytoscape from "cytoscape";
  import dagre from "cytoscape-dagre";
  import { ZoomIn, ZoomOut, Maximize2, UnfoldHorizontal, FoldHorizontal, Download } from "lucide-svelte";
  import {
    sppfNodeStyles,
    gssNodeStyles,
    edgeStyles,
    gssEdgeStyles,
    capZoom,
    createGraph,
    truncateLabel,
    setupGraphTooltip,
    LABEL_MAX_LENGTH,
    INTERMEDIATE_MAX_LENGTH,
  } from "$lib/graph-styles";
  import { GraphCollapseManager, buildDebugSppfElements, exportGraphPng } from "$lib/graph-utils";
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
  const collapseManager = new GraphCollapseManager();

  // Data for different graph types (received via events)
  let sppfData: SPPF | null = $state(null);
  let gssData: GSS | null = $state(null);
  let debugSppfNodes: DebugSPPFNode[] = $state([]);
  let debugSppfCurrentNodeId: number | null = $state(null);
  let debugGssNodes: DebugGSSNode[] = $state([]);
  let debugGssEdges: DebugGSSEdge[] = $state([]);
  let debugGssCurrentNodeId: number | null = $state(null);
  let selectedNodeId: string | null = $state(null);
  let showSpans = $state(false);

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
        ...sppfData.nodes.map((node) => {
          const fullLabel = node.label || (node.kind === "Packed" ? "" : "");
          // Intermediate nodes get longer max length since they show grammar slots
          const maxLen = node.kind === "Intermediate" ? INTERMEDIATE_MAX_LENGTH : LABEL_MAX_LENGTH;
          return {
            data: {
              id: `n${node.id}`,
              label: truncateLabel(fullLabel, maxLen),
              fullLabel: fullLabel,
            },
            classes: node.kind.toLowerCase(),
          };
        }),
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
      return buildDebugSppfElements(debugSppfNodes, debugSppfCurrentNodeId, showSpans) || [];
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

  let tooltipCleanup: (() => void) | null = null;

  function renderGraph() {
    if (!container) return;

    // Cleanup previous tooltip
    if (tooltipCleanup) {
      tooltipCleanup();
      tooltipCleanup = null;
    }

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

    // Reset collapsed nodes when rendering new graph
    collapseManager.reset();

    const isGss = graphType === "gss" || graphType === "debugGss";
    const isSppf = graphType === "sppf" || graphType === "debugSppf";

    cy = createGraph({
      container,
      elements,
      styles: isGss
        ? [...gssNodeStyles, gssEdgeStyles]
        : [...sppfNodeStyles, edgeStyles],
      layout: isGss ? "gss" : "sppf",
    });

    collapseManager.setCy(cy);

    // Setup tooltip for long labels (SPPF only, since intermediate nodes have long labels)
    if (isSppf) {
      tooltipCleanup = setupGraphTooltip(cy, container);

      // Add double-click handler for collapse/expand
      cy.on('dbltap', 'node', (event) => {
        const node = event.target;
        collapseManager.toggleCollapse(node.id());
      });

      // Click on node to select and emit span to main window
      if (graphType === 'debugSppf') {
        cy.on('tap', 'node', (event) => {
          const node = event.target;
          const left = node.data('leftExtent');
          const right = node.data('rightExtent');

          // Update node selection styling
          if (selectedNodeId && cy) {
            cy.getElementById(selectedNodeId).removeClass('selected');
          }
          selectedNodeId = node.id();
          node.addClass('selected');

          // Emit event to main window
          if (left !== undefined && right !== undefined) {
            emit('sppf-node-selected', { left, right, nodeId: node.id() });
          }
        });

        // Click on background to clear selection
        cy.on('tap', (event) => {
          if (event.target === cy) {
            if (selectedNodeId && cy) {
              cy.getElementById(selectedNodeId).removeClass('selected');
              selectedNodeId = null;
            }
            emit('sppf-node-selected', { left: null, right: null, nodeId: null });
          }
        });
      }
    }
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
        listen<{ nodes: DebugSPPFNode[]; current_node_id: number | null; show_spans?: boolean }>(
          "graph-data-debug-sppf",
          (event) => {
            debugSppfNodes = event.payload.nodes;
            debugSppfCurrentNodeId = event.payload.current_node_id;
            if (event.payload.show_spans !== undefined) {
              showSpans = event.payload.show_spans;
            }
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

  function toggleSpans() {
    showSpans = !showSpans;
    // Preserve selection state before re-rendering
    const savedSelection = selectedNodeId;

    // Re-render the graph
    tick().then(() => {
      renderGraph();
      // Restore selection
      if (savedSelection && cy) {
        selectedNodeId = savedSelection;
        cy.getElementById(savedSelection).addClass('selected');
      }
    });
    // Notify main window about the change (only for debug SPPF)
    if (graphType === 'debugSppf') {
      emit('spans-toggled', { show_spans: showSpans });
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
      {#if graphType === 'debugSppf'}
        <button onclick={toggleSpans} title={showSpans ? "Hide spans" : "Show spans"}>
          {#if showSpans}
            <FoldHorizontal size={14} />
          {:else}
            <UnfoldHorizontal size={14} />
          {/if}
        </button>
      {/if}
      <button onclick={() => exportGraphPng(cy, graphType ?? 'graph')} title="Export as PNG">
        <Download size={14} />
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
