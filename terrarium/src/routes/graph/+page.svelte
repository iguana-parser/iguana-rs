<script lang="ts">
  import { onMount, tick } from "svelte";
  import { page } from "$app/stores";
  import { listen, emit } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import cytoscape from "cytoscape";
  import dagre from "cytoscape-dagre";
  import tidytree from "cytoscape-tidytree";
  import { ZoomIn, ZoomOut, Maximize2, UnfoldHorizontal, FoldHorizontal, Download } from "lucide-svelte";
  import {
    sppfNodeStyles,
    gssNodeStyles,
    edgeStyles,
    gssEdgeStyles,
    capZoom,
    createGraph,
    getViewport,
    setupGraphTooltip,
    highlightOutgoingEdges,
    clearEdgeHighlights,
    highlightClickedEdge,
    GraphCollapseManager,
    buildParseTreeElements,
    type ParseTreeData,
  } from "@iguana-parser/web-ui";
  import { buildDebugSppfElements, buildDebugGssElements, buildSppfElements, buildGssElements, exportGraphPng } from "$lib/graph-utils";
  import { createMaximizeToggle } from "$lib/window-utils";
  import type {
    SPPF,
    GSS,
    DebugSPPFNode,
    DebugGSSNode,
    DebugGSSEdge,
  } from "../../bindings";

  cytoscape.use(dagre);
  cytoscape.use(tidytree);

  const toggleMaximize = createMaximizeToggle();

  function startDrag(e: MouseEvent) {
    if (e.button === 0) {
      getCurrentWindow().startDragging();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      // Close context menu first, then clear selections, then close window
      if (contextMenu) {
        contextMenu = null;
      } else if (selectedNodeId || (cy && cy.edges('.edge-clicked').length > 0)) {
        if (selectedNodeId && cy) {
          cy.getElementById(selectedNodeId).removeClass('selected');
          selectedNodeId = null;
        }
        if (cy) clearEdgeHighlights(cy);
        emit('sppf-node-selected', { left: null, right: null, nodeId: null });
      } else {
        getCurrentWindow().close();
      }
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
    | "parseTree"
    | "debugSppf"
    | "debugGss"
    | null;

  let container: HTMLDivElement;
  let cy: cytoscape.Core | null = null;
  const collapseManager = new GraphCollapseManager();

  // Data for different graph types (received via events)
  let sppfData: SPPF | null = $state(null);
  let gssData: GSS | null = $state(null);
  let parseTreeData: ParseTreeData | null = $state(null);
  let debugSppfNodes: DebugSPPFNode[] = $state([]);
  let debugSppfCurrentNodeId: number | null = $state(null);
  let debugGssNodes: DebugGSSNode[] = $state([]);
  let debugGssEdges: DebugGSSEdge[] = $state([]);
  let debugGssCurrentNodeId: number | null = $state(null);
  let selectedNodeId: string | null = $state(null);
  let showSpans = $state(false);
  let contextMenu = $state<{ x: number; y: number; nodeId: string } | null>(null);
  let subtreeFocused = $state(false);

  function getTitle(): string {
    switch (graphType) {
      case "sppf":
      case "debugSppf":
        return "SPPF";
      case "gss":
      case "debugGss":
        return "GSS";
      case "parseTree":
        return "Parse Tree";
      default:
        return "Graph";
    }
  }

  function buildElements(): cytoscape.ElementDefinition[] {
    switch (graphType) {
      case "sppf":
        return sppfData ? buildSppfElements(sppfData) : [];
      case "gss":
        return gssData ? buildGssElements(gssData) : [];
      case "parseTree":
        return parseTreeData ? buildParseTreeElements(parseTreeData, showSpans) : [];
      case "debugSppf":
        return buildDebugSppfElements(debugSppfNodes, debugSppfCurrentNodeId, showSpans) || [];
      case "debugGss":
        return buildDebugGssElements(debugGssNodes, debugGssEdges, debugGssCurrentNodeId);
      default:
        return [];
    }
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

    // Save viewport before destroying
    const savedViewport = cy ? getViewport(cy) : undefined;

    if (cy) {
      cy.destroy();
    }

    // Reset collapsed nodes and focus when rendering new graph
    collapseManager.reset();
    subtreeFocused = false;

    const isGss = graphType === "gss" || graphType === "debugGss";
    const isSppf = graphType === "sppf" || graphType === "debugSppf";
    // The parse-tree graph reuses the SPPF styling and interactions, but its node
    // selection is not synced back to the main window (it has no leftExtent/rightExtent).
    const isParseTree = graphType === "parseTree";

    cy = createGraph({
      container,
      elements,
      styles: isGss
        ? [...gssNodeStyles, gssEdgeStyles]
        : [...sppfNodeStyles, ...edgeStyles],
      layout: isGss ? "gss" : isParseTree ? "tree" : "sppf",
      viewport: savedViewport,
    });

    collapseManager.setCy(cy);

    // Setup tooltip for long labels (SPPF only, since intermediate nodes have long labels)
    if (isSppf || isParseTree) {
      tooltipCleanup = setupGraphTooltip(cy, container);

      // Add double-click handler for collapse/expand
      cy.on('dbltap', 'node', (event) => {
        const node = event.target;
        collapseManager.toggleCollapse(node.id());
      });

      // Click on node to select and emit span to main window
      if (isSppf || isParseTree) {
        cy.on('tap', 'node', (event) => {
          const node = event.target;
          const left = node.data('leftExtent');
          const right = node.data('rightExtent');

          // Update node selection styling
          if (selectedNodeId && cy) {
            cy.getElementById(selectedNodeId).removeClass('selected');
          }
          // Clear previous edge highlights and highlight new outgoing edges
          if (cy) {
            clearEdgeHighlights(cy);
            selectedNodeId = node.id();
            node.addClass('selected');
            highlightOutgoingEdges(cy, node.id());
          }

          // Emit event to main window
          if (left !== undefined && right !== undefined) {
            emit('sppf-node-selected', { left, right, nodeId: node.id() });
          }
          // Close context menu on regular click
          contextMenu = null;
        });

        // Click on background to clear selection and close context menu
        cy.on('tap', (event) => {
          if (event.target === cy) {
            if (selectedNodeId && cy) {
              cy.getElementById(selectedNodeId).removeClass('selected');
              selectedNodeId = null;
            }
            if (cy) clearEdgeHighlights(cy);
            if (!isParseTree) emit('sppf-node-selected', { left: null, right: null, nodeId: null });
            contextMenu = null;
          }
        });

        // Click on edge to highlight it
        cy.on('tap', 'edge', (event) => {
          const edge = event.target;
          // Clear node selection
          if (selectedNodeId && cy) {
            cy.getElementById(selectedNodeId).removeClass('selected');
            selectedNodeId = null;
          }
          if (cy) {
            highlightClickedEdge(cy, edge.id());
          }
          if (!isParseTree) emit('sppf-node-selected', { left: null, right: null, nodeId: null });
        });

        // Right-click on node to show context menu
        cy.on('cxttap', 'node', (event) => {
          const node = event.target;
          const renderedPos = node.renderedPosition();
          const containerRect = container.getBoundingClientRect();
          // Hide tooltip when showing context menu
          const tooltip = document.querySelector('.graph-tooltip') as HTMLElement;
          if (tooltip) tooltip.style.display = 'none';
          contextMenu = {
            x: containerRect.left + renderedPos.x,
            y: containerRect.top + renderedPos.y,
            nodeId: node.id()
          };
        });

        // Right-click on background to close context menu
        cy.on('cxttap', (event) => {
          if (event.target === cy) {
            contextMenu = null;
          }
        });
      }
    }
  }

  function handleContextMenuAction(action: 'focus' | 'showAll') {
    if (action === 'focus' && contextMenu) {
      collapseManager.focusOnSubtree(contextMenu.nodeId);
      subtreeFocused = true;
    } else if (action === 'showAll') {
      collapseManager.clearFocus();
      subtreeFocused = false;
    }
    contextMenu = null;
  }

  // Re-render when data changes
  $effect(() => {
    // Track all data sources
    sppfData;
    gssData;
    parseTreeData;
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
    } else if (graphType === "parseTree") {
      unlisteners.push(
        listen<ParseTreeData>("graph-data-parse-tree", (event) => {
          parseTreeData = event.payload;
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

  function adjustZoom(factor: number) {
    if (cy) cy.zoom(cy.zoom() * factor);
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
        highlightOutgoingEdges(cy, savedSelection);
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
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="graph-container" bind:this={container} oncontextmenu={(e) => e.preventDefault()}></div>
    <div class="graph-controls">
      <button onclick={() => adjustZoom(1.2)} title="Zoom In">
        <ZoomIn size={14} />
      </button>
      <button onclick={() => adjustZoom(1/1.2)} title="Zoom Out">
        <ZoomOut size={14} />
      </button>
      <button onclick={resetView} title="Reset View">
        <Maximize2 size={14} />
      </button>
      {#if graphType === 'debugSppf' || graphType === 'parseTree'}
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
    {#if contextMenu}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="context-menu"
        style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
        onmousedown={(e) => e.stopPropagation()}
      >
        <button onclick={() => handleContextMenuAction('focus')}>Focus on subtree</button>
        {#if subtreeFocused}
          <button onclick={() => handleContextMenuAction('showAll')}>Show all nodes</button>
        {/if}
      </div>
    {/if}
    {#if subtreeFocused}
      <button class="show-all-button" onclick={() => handleContextMenuAction('showAll')}>
        Show all nodes
      </button>
    {/if}
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

  /* Context Menu */
  .context-menu {
    position: fixed;
    background: #2d2d2d;
    border: 1px solid #555;
    border-radius: 4px;
    padding: 2px 0;
    min-width: 130px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 10001;
  }

  .context-menu button {
    display: block;
    width: 100%;
    padding: 5px 10px;
    background: none;
    border: none;
    color: #d4d4d4;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
  }

  .context-menu button:hover {
    background: #3a3a3a;
  }

  /* Show all nodes button (when subtree is focused) */
  .show-all-button {
    position: absolute;
    top: 8px;
    left: 8px;
    padding: 6px 12px;
    background: rgba(45, 45, 45, 0.95);
    border: 1px solid #555;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
    cursor: pointer;
    z-index: 100;
  }

  .show-all-button:hover {
    background: rgba(60, 60, 60, 0.95);
    border-color: #888;
  }
</style>
