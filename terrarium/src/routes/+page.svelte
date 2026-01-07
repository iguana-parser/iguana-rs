<script lang="ts">
  import { commands, type SPPF, type GSS, type DebugInfo, type DebugSPPFNode, type DebugSPPFInfo, type DebugGSSNode, type DebugGSSEdge, type DebugGSSInfo } from "../bindings";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { createMaximizeToggle } from "$lib/window-utils";
  import { onMount, tick } from "svelte";
  import { FolderOpen, Hammer, X, AlertTriangle, CheckCircle, Loader2, ChevronDown, ChevronRight, ZoomIn, ZoomOut, Maximize2, Minimize2, Expand, Fullscreen, GitFork, Bug, Braces, PanelBottom, Trash2, ChevronsDown, Copy, ClipboardCheck, UnfoldHorizontal, FoldHorizontal, Download } from "lucide-svelte";
  import cytoscape from "cytoscape";
  import dagre from "cytoscape-dagre";
  import {
    sppfNodeStyles,
    gssNodeStyles,
    edgeStyles,
    gssEdgeStyles,
    capZoom,
    createGraph,
    getViewport,
    truncateLabel,
    setupGraphTooltip,
    LABEL_MAX_LENGTH,
    INTERMEDIATE_MAX_LENGTH,
  } from "$lib/graph-styles";
  import { GraphCollapseManager, buildDebugSppfElements, exportGraphPng } from "$lib/graph-utils";

  // Parse Tree types (manually defined, not via specta)
  interface ParseTreeNode {
    id: number;
    kind: "Nonterminal" | "Token";
    label: string;
    start: number;
    end: number;
  }
  interface ParseTreeEdge {
    src: number;
    dest: number;
  }
  interface ParseTree {
    nodes: ParseTreeNode[];
    edges: ParseTreeEdge[];
  }

  cytoscape.use(dagre);

  // Event listeners for build progress
  onMount(() => {
    const unlistenProgress = listen<{ stage: string; message: string }>("build-progress", (event) => {
      // Progress is shown in title bar status, not status bar
    });

    const unlistenResult = listen<{ success: boolean; message: string }>("build-result", async (event) => {
      isBuilding = false;
      statusMessage = null;  // Clear status message
      if (event.payload.success) {
        buildStatus = "success";
        logOutput("Build successful");
        if (parserDirectory) {
          // Fetch parser name and nonterminals in parallel
          const [nameResult, ntResult] = await Promise.all([
            commands.getParserName(parserDirectory),
            commands.getNonterminals(parserDirectory)
          ]);
          if (nameResult.status === "ok") {
            parserName = nameResult.data;
          }
          if (ntResult.status === "ok") {
            nonterminals = ntResult.data;
            if (nonterminals.length > 0 && !startNonterminal) {
              startNonterminal = nonterminals[0];
            }
            // Show nonterminals in output panel
            logCommand(`${parserName} --list-nonterminals`);
            logOutput(nonterminals.join('\n'));
          }
        }
        showReadyStatus = true;
        if (readyStatusTimeout) clearTimeout(readyStatusTimeout);
        readyStatusTimeout = setTimeout(() => {
          showReadyStatus = false;
        }, 3000);
      } else {
        buildStatus = "error";
        buildError = event.payload.message;
        logError(`Build failed\n${event.payload.message}`);
        outputPanelOpen = true;
      }
    });

    const unlistenGenerateResult = listen<{ success: boolean; message: string }>("generate-result", (event) => {
      isGenerating = false;
      statusMessage = null;  // Clear status message
      if (event.payload.success) {
        generateStatus = "success";
        generateError = null;
        logOutput("Parser generated successfully");
      } else {
        generateStatus = "error";
        generateError = event.payload.message;
        logError(event.payload.message);
        outputPanelOpen = true;
      }
    });

    // Track window width for proportional column resizing
    let lastWindowWidth = window.innerWidth;

    function handleWindowResize() {
      const newWidth = window.innerWidth;
      const delta = newWidth - lastWindowWidth;

      if (delta !== 0 && activeMode === "debug") {
        // Distribute the extra/reduced width to the right column
        // This makes the graph area grow when window expands
        const minCol3 = 200;
        const maxCol3 = newWidth - 48 - debugCol1Width - 250;
        debugCol3Width = Math.max(minCol3, Math.min(maxCol3, debugCol3Width + delta));
      }

      lastWindowWidth = newWidth;
    }

    window.addEventListener('resize', handleWindowResize);

    // Listen for graph window ready events to send initial data
    const unlistenGraphWindowReady = listen<{ graphType: string }>("graph-window-ready", (event) => {
      const graphType = event.payload.graphType as GraphType;
      sendGraphData(graphType);
    });

    // Listen for step events from popup windows
    const unlistenStepBack = listen("debug-step-back", () => stepBack());
    const unlistenStepForward = listen("debug-step-forward", () => stepForward());

    // Listen for node selection from popup windows
    const unlistenNodeSelected = listen<{ left: number | null; right: number | null; nodeId: string | null }>("sppf-node-selected", (event) => {
      const { left, right, nodeId } = event.payload;
      if (left !== null && right !== null) {
        selectedSpan = { left, right };
      } else {
        selectedSpan = null;
      }
      // Update selection in main window's graph if it exists
      if (debugSppfCy) {
        if (selectedNodeId) {
          debugSppfCy.getElementById(selectedNodeId).removeClass('selected');
        }
        if (nodeId) {
          debugSppfCy.getElementById(nodeId).addClass('selected');
        }
        selectedNodeId = nodeId;
      }
    });

    // Listen for spans toggle from popup windows
    const unlistenSpansToggled = listen<{ show_spans: boolean }>("spans-toggled", (event) => {
      showSpans = event.payload.show_spans;
      // Preserve selection state before re-rendering
      const savedParseTreeSelection = parseTreeSelectedNodeId;
      const savedParseTreeSpan = parseTreeSelectedSpan;
      const savedSppfSelection = sppfSelectedNodeId;
      const savedSppfSpan = sppfSelectedSpan;
      const savedDebugSelection = selectedNodeId;
      const savedDebugSpan = selectedSpan;

      // Re-render main window graphs
      if (parseTree) {
        tick().then(() => {
          renderParseTree();
          // Restore selection
          if (savedParseTreeSelection && parseTreeCy) {
            parseTreeSelectedNodeId = savedParseTreeSelection;
            parseTreeSelectedSpan = savedParseTreeSpan;
            parseTreeCy.getElementById(savedParseTreeSelection).addClass('selected');
          }
        });
      }
      if (sppf) {
        tick().then(() => {
          renderSPPF();
          // Restore selection
          if (savedSppfSelection && cy) {
            sppfSelectedNodeId = savedSppfSelection;
            sppfSelectedSpan = savedSppfSpan;
            cy.getElementById(savedSppfSelection).addClass('selected');
          }
        });
      }
      if (debugSppfNodes.length > 0) {
        tick().then(() => {
          renderDebugSppf();
          // Restore selection
          if (savedDebugSelection && debugSppfCy) {
            selectedNodeId = savedDebugSelection;
            selectedSpan = savedDebugSpan;
            debugSppfCy.getElementById(savedDebugSelection).addClass('selected');
          }
        });
      }
    });

    // Close all graph windows when main window closes
    const mainWindow = getCurrentWindow();
    const unlistenMainClose = mainWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      await closeAllGraphWindows();
      await mainWindow.destroy();
    });

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenResult.then(fn => fn());
      unlistenGenerateResult.then(fn => fn());
      unlistenGraphWindowReady.then(fn => fn());
      unlistenStepBack.then(fn => fn());
      unlistenStepForward.then(fn => fn());
      unlistenNodeSelected.then(fn => fn());
      unlistenSpansToggled.then(fn => fn());
      unlistenMainClose.then(fn => fn());
      window.removeEventListener('resize', handleWindowResize);
    };
  });

  // Parser directory state
  let parserDirectory = $state<string | null>(null);
  let parserName = $state<string | null>(null);
  let isBuilding = $state(false);
  let buildStatus = $state<"none" | "success" | "error">("none");
  let buildError = $state<string | null>(null);
  let showReadyStatus = $state(false);
  let readyStatusTimeout: ReturnType<typeof setTimeout> | null = null;

  // Modal state
  let showErrorModal = $state(false);
  let errorModalMessage = $state("");

  // Generation state
  let isGenerating = $state(false);
  let generateStatus = $state<"none" | "success" | "error">("none");
  let generateError = $state<string | null>(null);

  // Status bar state
  let statusMessage = $state<string | null>(null);
  let statusType = $state<"info" | "error" | "success">("info");
  let showStatusDetails = $state(false);
  let statusTimeout: ReturnType<typeof setTimeout> | null = null;

  function setStatus(message: string, type: "info" | "error" | "success") {
    // Clear any existing timeout
    if (statusTimeout) {
      clearTimeout(statusTimeout);
      statusTimeout = null;
    }

    statusMessage = message;
    statusType = type;

    // Auto-dismiss success messages after 3 seconds
    if (type === "success") {
      statusTimeout = setTimeout(() => {
        statusMessage = null;
        statusTimeout = null;
      }, 3000);
    }
  }

  // State
  let inputText = $state("");
  let lastParsedInput = $state<string | null>(null);
  let startNonterminal = $state<string | null>(null);
  let nonterminals = $state<string[]>([]);
  let dropdownOpen = $state(false);

  // Playback state
  let currentStep = $state(0);
  let totalSteps = $state(0);

  // Parser state
  let currentAction = $state<string | null>(null);
  // svelte-ignore non_reactive_update
  let actionBoxEl: HTMLDivElement | null = null;
  let descriptorSet = $state<string[]>([]);
  let callStack = $state<string[]>([]);
  let debugLoaded = $state(false);
  let inputIndex = $state<number | null>(null);
  let selectedSpan = $state<{ left: number; right: number } | null>(null);
  let selectedNodeId = $state<string | null>(null);
  let debugSppfNodes = $state<DebugSPPFNode[]>([]);

  // Debug SPPF visualization
  // svelte-ignore non_reactive_update
  let debugSppfContainer: HTMLElement;
  let debugSppfCy: cytoscape.Core | null = null;
  let currentSppfNodeId = $state<number | null>(null);
  const debugSppfCollapseManager = new GraphCollapseManager();

  // Debug GSS visualization
  let debugGssNodes = $state<DebugGSSNode[]>([]);
  let debugGssEdges = $state<DebugGSSEdge[]>([]);
  let currentGssNodeId = $state<number | null>(null);
  // svelte-ignore non_reactive_update
  let debugGssContainer: HTMLElement;
  let debugGssCy: cytoscape.Core | null = null;

  // Graph tab
  let activeTab = $state<"gss" | "sppf" | "parse-tree">("parse-tree");

  // Show spans in graph labels (hidden by default)
  let showSpans = $state(false);

  // App mode
  let activeMode = $state<"parse" | "debug" | "design">("parse");

  // SPPF data
  let sppf = $state<SPPF | null>(null);
  // svelte-ignore non_reactive_update
  let sppfContainer: HTMLDivElement;
  const sppfCollapseManager = new GraphCollapseManager();

  // GSS data
  let gss = $state<GSS | null>(null);
  // svelte-ignore non_reactive_update
  let gssContainer: HTMLDivElement;

  // Parse Tree data
  let parseTree = $state<ParseTree | null>(null);
  // svelte-ignore non_reactive_update
  let parseTreeContainer: HTMLDivElement;
  let parseTreeCy: cytoscape.Core | null = null;
  const parseTreeCollapseManager = new GraphCollapseManager();

  // Parse tree node selection (for highlighting span in input)
  let parseTreeSelectedSpan = $state<{ start: number; end: number } | null>(null);
  let parseTreeSelectedNodeId = $state<string | null>(null);

  // SPPF node selection (for highlighting span in input)
  let sppfSelectedSpan = $state<{ left: number; right: number } | null>(null);
  let sppfSelectedNodeId = $state<string | null>(null);

  // Track if parse result is available
  let parseResultAvailable = $state(false);

  // Graph window management (separate OS windows)
  type GraphType = 'sppf' | 'gss' | 'debugSppf' | 'debugGss';
  let graphWindows = $state<Map<GraphType, WebviewWindow>>(new Map());

  // Output panel state
  let outputPanelOpen = $state(false);
  let outputPanelHeight = $state(150);

  // Output log entries: each entry has a type and content
  type LogEntry = { type: "command" | "output" | "error"; content: string };
  let outputLog = $state<LogEntry[]>([]);
  // svelte-ignore non_reactive_update
  let outputContentEl: HTMLDivElement | null = null;

  function scrollOutputToBottom() {
    if (outputContentEl) {
      // Use setTimeout to ensure DOM has updated
      setTimeout(() => {
        if (outputContentEl) {
          outputContentEl.scrollTop = outputContentEl.scrollHeight;
        }
      }, 0);
    }
  }

  function logCommand(cmd: string) {
    outputLog = [...outputLog, { type: "command", content: cmd }];
    scrollOutputToBottom();
  }

  function logOutput(text: string) {
    outputLog = [...outputLog, { type: "output", content: text }];
    scrollOutputToBottom();
  }

  function logError(text: string) {
    outputLog = [...outputLog, { type: "error", content: text }];
    scrollOutputToBottom();
  }

  function clearOutput() {
    outputLog = [];
  }
  let cy: cytoscape.Core | null = null;
  let gssCy: cytoscape.Core | null = null;

  let sppfTooltipCleanup: (() => void) | null = null;

  function renderSPPF() {
    if (!sppf || !sppfContainer) return;

    // Reset collapsed nodes when rendering new SPPF
    sppfCollapseManager.reset();
    // Clear selection when re-rendering
    sppfSelectedSpan = null;
    sppfSelectedNodeId = null;

    // Cleanup previous tooltip
    if (sppfTooltipCleanup) {
      sppfTooltipCleanup();
      sppfTooltipCleanup = null;
    }

    const elements: cytoscape.ElementDefinition[] = [
      ...sppf.nodes.map((node) => {
        const baseLabel = node.label || (node.kind === "Packed" ? "●" : "");
        // Intermediate nodes get longer max length since they show grammar slots
        const maxLen = node.kind === "Intermediate" ? INTERMEDIATE_MAX_LENGTH : LABEL_MAX_LENGTH;
        // Optionally add span to label (skip for packed nodes which have no real span)
        const span = `(${node.left_extent}, ${node.right_extent})`;
        const displayLabel = showSpans && node.kind !== "Packed"
          ? `${truncateLabel(baseLabel, maxLen)}\n${span}`
          : truncateLabel(baseLabel, maxLen);
        const fullLabel = showSpans && node.kind !== "Packed"
          ? `${baseLabel}\n${span}`
          : baseLabel;
        return {
          data: {
            id: `n${node.id}`,
            label: displayLabel,
            fullLabel: fullLabel,
            leftExtent: node.left_extent,
            rightExtent: node.right_extent,
          },
          classes: node.kind.toLowerCase(),
        };
      }),
      ...sppf.edges.map((edge, i) => ({
        data: {
          id: `e${i}`,
          source: `n${edge.src}`,
          target: `n${edge.dest}`,
        },
      })),
    ];

    // Save viewport before destroying
    const savedViewport = cy ? getViewport(cy) : undefined;

    if (cy) {
      cy.destroy();
    }

    cy = createGraph({
      container: sppfContainer,
      elements,
      styles: [...sppfNodeStyles, edgeStyles],
      layout: 'sppf',
      viewport: savedViewport,
    });

    sppfCollapseManager.setCy(cy);

    // Setup tooltip for long labels
    sppfTooltipCleanup = setupGraphTooltip(cy, sppfContainer);

    // Add double-click handler for collapse/expand
    cy.on('dbltap', 'node', (event) => {
      const node = event.target;
      sppfCollapseManager.toggleCollapse(node.id());
    });

    // Click on node to highlight span in input and select node
    cy.on('tap', 'node', (event) => {
      const node = event.target;
      const left = node.data('leftExtent');
      const right = node.data('rightExtent');
      if (left !== undefined && right !== undefined) {
        sppfSelectedSpan = { left, right };
      }
      // Update node selection styling
      if (sppfSelectedNodeId) {
        cy?.getElementById(sppfSelectedNodeId).removeClass('selected');
      }
      sppfSelectedNodeId = node.id();
      node.addClass('selected');
    });

    // Click on background to clear selection
    cy.on('tap', (event) => {
      if (event.target === cy) {
        sppfSelectedSpan = null;
        if (sppfSelectedNodeId) {
          cy?.getElementById(sppfSelectedNodeId).removeClass('selected');
          sppfSelectedNodeId = null;
        }
      }
    });
  }

  function renderGSS() {
    if (!gss || !gssContainer) return;

    const elements: cytoscape.ElementDefinition[] = [
      ...gss.nodes.map((node) => ({
        data: {
          id: `n${node.id}`,
          label: node.label,
        },
      })),
      ...gss.edges.map((edge, i) => ({
        data: {
          id: `e${i}`,
          source: `n${edge.src}`,
          target: `n${edge.dest}`,
          label: edge.label,
        },
      })),
    ];

    // Save viewport before destroying
    const savedViewport = gssCy ? getViewport(gssCy) : undefined;

    if (gssCy) {
      gssCy.destroy();
    }

    gssCy = createGraph({
      container: gssContainer,
      elements,
      styles: [...gssNodeStyles, gssEdgeStyles],
      layout: 'gss',
      viewport: savedViewport,
    });
  }

  let parseTreeTooltipCleanup: (() => void) | null = null;

  function renderParseTree() {
    if (!parseTree || !parseTreeContainer) return;

    parseTreeCollapseManager.reset();
    // Clear selection when re-rendering
    parseTreeSelectedSpan = null;
    parseTreeSelectedNodeId = null;

    if (parseTreeTooltipCleanup) {
      parseTreeTooltipCleanup();
      parseTreeTooltipCleanup = null;
    }

    const elements: cytoscape.ElementDefinition[] = [
      ...parseTree.nodes.map((node) => {
        const maxLen = LABEL_MAX_LENGTH;
        // Optionally add span to label: "label\n(start, end)"
        const span = `(${node.start}, ${node.end})`;
        const displayLabel = showSpans
          ? `${truncateLabel(node.label, maxLen)}\n${span}`
          : truncateLabel(node.label, maxLen);
        const fullLabel = showSpans
          ? `${node.label}\n${span}`
          : node.label;
        return {
          data: {
            id: `n${node.id}`,
            label: displayLabel,
            fullLabel: fullLabel,
            start: node.start,
            end: node.end,
          },
          classes: node.kind.toLowerCase(),
        };
      }),
      ...parseTree.edges.map((edge, i) => ({
        data: {
          id: `e${i}`,
          source: `n${edge.src}`,
          target: `n${edge.dest}`,
        },
      })),
    ];

    // Save viewport before destroying
    const savedViewport = parseTreeCy ? getViewport(parseTreeCy) : undefined;

    if (parseTreeCy) {
      parseTreeCy.destroy();
    }

    parseTreeCy = createGraph({
      container: parseTreeContainer,
      elements,
      styles: [...sppfNodeStyles, edgeStyles],  // Reuse SPPF styles (nonterminal/token)
      layout: 'sppf',  // Top-to-bottom tree layout
      viewport: savedViewport,
    });

    parseTreeCollapseManager.setCy(parseTreeCy);

    // Setup tooltip for long labels
    parseTreeTooltipCleanup = setupGraphTooltip(parseTreeCy, parseTreeContainer);

    // Add double-click handler for collapse/expand
    parseTreeCy.on('dbltap', 'node', (event) => {
      const node = event.target;
      parseTreeCollapseManager.toggleCollapse(node.id());
    });

    // Add click handler for node selection and span highlighting
    parseTreeCy.on('tap', 'node', (event) => {
      const node = event.target;
      const start = node.data('start');
      const end = node.data('end');
      if (start !== undefined && end !== undefined) {
        parseTreeSelectedSpan = { start, end };
      }
      // Update node selection styling
      if (parseTreeSelectedNodeId) {
        parseTreeCy?.getElementById(parseTreeSelectedNodeId).removeClass('selected');
      }
      parseTreeSelectedNodeId = node.id();
      node.addClass('selected');
    });

    // Click on background clears selection
    parseTreeCy.on('tap', (event) => {
      if (event.target === parseTreeCy) {
        parseTreeSelectedSpan = null;
        if (parseTreeSelectedNodeId) {
          parseTreeCy?.getElementById(parseTreeSelectedNodeId).removeClass('selected');
          parseTreeSelectedNodeId = null;
        }
      }
    });
  }

  $effect(() => {
    // Track activeTab so effect re-runs when switching tabs
    if (activeTab === "sppf" && sppf) {
      // Wait for DOM to update after tab switch
      tick().then(() => {
        if (sppfContainer) {
          renderSPPF();
        }
      });
    } else if (activeTab === "gss" && gss) {
      tick().then(() => {
        if (gssContainer) {
          renderGSS();
        }
      });
    } else if (activeTab === "parse-tree" && parseTree) {
      tick().then(() => {
        if (parseTreeContainer) {
          renderParseTree();
        }
      });
    }
  });

  // Fetch data when switching tabs
  $effect(() => {
    if (parseResultAvailable) {
      if (activeTab === "sppf" && !sppf) {
        fetchSppf();
      } else if (activeTab === "gss" && !gss) {
        fetchGss();
      } else if (activeTab === "parse-tree" && !parseTree) {
        fetchParseTree();
      }
    }
  });

  // ResizeObservers for debug graph containers
  let debugSppfResizeObserver: ResizeObserver | null = null;
  let debugGssResizeObserver: ResizeObserver | null = null;
  let sppfResizeTimeout: ReturnType<typeof setTimeout> | null = null;
  let gssResizeTimeout: ReturnType<typeof setTimeout> | null = null;

  // Render debug SPPF when nodes or current node changes
  $effect(() => {
    // Track both debugSppfNodes and currentSppfNodeId
    const _nodes = debugSppfNodes;
    const _currentId = currentSppfNodeId;
    if (debugSppfContainer) {
      tick().then(() => renderDebugSppf());
    }
  });

  let debugSppfTooltipCleanup: (() => void) | null = null;

  function renderDebugSppf() {
    if (!debugSppfContainer) return;

    // Cleanup previous tooltip
    if (debugSppfTooltipCleanup) {
      debugSppfTooltipCleanup();
      debugSppfTooltipCleanup = null;
    }

    const elements = buildDebugSppfElements(debugSppfNodes, currentSppfNodeId, showSpans);

    // If no reachable nodes, clear the graph
    if (!elements) {
      if (debugSppfCy) {
        debugSppfCy.destroy();
        debugSppfCy = null;
      }
      return;
    }

    // Save viewport before destroying
    const savedViewport = debugSppfCy ? getViewport(debugSppfCy) : undefined;

    if (debugSppfCy) {
      debugSppfCy.destroy();
    }

    debugSppfCollapseManager.reset();

    debugSppfCy = createGraph({
      container: debugSppfContainer,
      elements,
      styles: [...sppfNodeStyles, edgeStyles],
      layout: 'sppf',
      viewport: savedViewport,
    });

    debugSppfCollapseManager.setCy(debugSppfCy);

    // Setup tooltip for long labels
    debugSppfTooltipCleanup = setupGraphTooltip(debugSppfCy, debugSppfContainer);

    // Double-click to collapse/expand node
    debugSppfCy.on('dbltap', 'node', (event) => {
      const node = event.target;
      debugSppfCollapseManager.toggleCollapse(node.id());
    });

    // Click on node to highlight span in input and select node
    debugSppfCy.on('tap', 'node', (event) => {
      const node = event.target;
      const left = node.data('leftExtent');
      const right = node.data('rightExtent');
      if (left !== undefined && right !== undefined) {
        selectedSpan = { left, right };
      }
      // Update node selection styling
      if (selectedNodeId) {
        debugSppfCy?.getElementById(selectedNodeId).removeClass('selected');
      }
      selectedNodeId = node.id();
      node.addClass('selected');
    });

    // Click on background to clear selection
    debugSppfCy.on('tap', (event) => {
      if (event.target === debugSppfCy) {
        selectedSpan = null;
        if (selectedNodeId) {
          debugSppfCy?.getElementById(selectedNodeId).removeClass('selected');
          selectedNodeId = null;
        }
      }
    });

    // Set up ResizeObserver to recenter graph when container resizes (debounced)
    if (debugSppfResizeObserver) {
      debugSppfResizeObserver.disconnect();
    }
    debugSppfResizeObserver = new ResizeObserver(() => {
      if (sppfResizeTimeout) clearTimeout(sppfResizeTimeout);
      sppfResizeTimeout = setTimeout(() => {
        if (debugSppfCy) {
          debugSppfCy.resize();
          debugSppfCy.center();
        }
      }, 50);
    });
    debugSppfResizeObserver.observe(debugSppfContainer);
  }

  // Render debug GSS when nodes or current node changes
  $effect(() => {
    const _nodes = debugGssNodes;
    const _edges = debugGssEdges;
    const _currentId = currentGssNodeId;
    if (debugGssContainer) {
      tick().then(() => renderDebugGSS());
    }
  });

  function renderDebugGSS() {
    if (!debugGssContainer) return;

    const elements: cytoscape.ElementDefinition[] = [];

    // Add all nodes
    for (const node of debugGssNodes) {
      elements.push({
        data: {
          id: `n${node.id}`,
          label: node.label,
        },
        classes: currentGssNodeId === node.id ? 'current' : '',
      });
    }

    // Add edges
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

    // If no nodes, clear the graph
    if (elements.length === 0) {
      if (debugGssCy) {
        debugGssCy.destroy();
        debugGssCy = null;
      }
      return;
    }

    // Save viewport before destroying
    const savedViewport = debugGssCy ? getViewport(debugGssCy) : undefined;

    if (debugGssCy) {
      debugGssCy.destroy();
    }

    debugGssCy = createGraph({
      container: debugGssContainer,
      elements,
      styles: [...gssNodeStyles, gssEdgeStyles],
      layout: 'gss',
      viewport: savedViewport,
    });

    // Set up ResizeObserver to recenter graph when container resizes (debounced)
    if (debugGssResizeObserver) {
      debugGssResizeObserver.disconnect();
    }
    debugGssResizeObserver = new ResizeObserver(() => {
      if (gssResizeTimeout) clearTimeout(gssResizeTimeout);
      gssResizeTimeout = setTimeout(() => {
        if (debugGssCy) {
          debugGssCy.resize();
          debugGssCy.center();
        }
      }, 50);
    });
    debugGssResizeObserver.observe(debugGssContainer);
  }

  // Resizable panes
  let leftPanelWidth = $state(350);
  let callStackHeight = $state(200);
  let inputHeight = $state(200);
  let currentDescHeight = $state(120);  // ~25% of column
  let isDraggingVertical = $state(false);
  let isDraggingHorizontal = $state(false);
  let isDraggingInput = $state(false);
  let isDraggingCurrent = $state(false);
  let isDraggingOutput = $state(false);

  // Debug mode column widths
  let debugCol1Width = $state(300);
  let debugCol3Width = $state(400);
  let debugActionHeight = $state<number | null>(null);  // null = use CSS flex default (25%)
  // Note: Stack section always uses flex: 1 to absorb resize changes from Action and Pending
  let debugPendingHeight = $state<number | null>(null); // null = use CSS flex default (25%)
  let debugSppfHeight = $state<number | null>(null);   // null = use CSS flex default (50%)

  // Resize start tracking for delta-based resizing
  let resizeStartY = 0;
  let resizeStartActionHeight = 0;
  let resizeStartStackHeight = 0;
  let resizeStartPendingHeight = 0;
  let isDraggingDebug1 = $state(false);
  let isDraggingDebug2 = $state(false);
  let isDraggingDebugAction = $state(false);
  let isDraggingDebugStack = $state(false);
  let isDraggingDebugGraph = $state(false);

  async function selectDirectory() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Parser Directory",
    });
    if (selected) {
      parserDirectory = selected as string;
      buildStatus = "none";
      buildError = null;
      sppf = null;
      gss = null;
      parseTree = null;
      parseResultAvailable = false;
      lastParsedInput = null;
      nonterminals = [];
      startNonterminal = null;

      // Log the working directory
      logOutput(`Working directory: ${parserDirectory}`);

      // Try to get parser name (might not exist yet if empty directory)
      const result = await commands.getParserName(parserDirectory);
      if (result.status === "ok") {
        parserName = result.data;
        logOutput(`Parser: ${parserName}`);
      } else {
        parserName = null;
      }

      // Auto-build the parser
      await buildParser();
    }
  }

  function closeErrorModal() {
    showErrorModal = false;
    errorModalMessage = "";
  }

  async function buildParser() {
    if (!parserDirectory) return;
    isBuilding = true;
    buildError = null;
    statusMessage = null;  // Let isBuilding control the status display
    logCommand(`cargo build --features debug-trace`);
    // Command returns immediately, results come via events
    await commands.buildParser(parserDirectory);
  }

  async function generateParser() {
    if (!parserDirectory) return;
    isGenerating = true;
    generateError = null;
    generateStatus = "none";
    logCommand(`iguana generate --output .`);
    // Command returns immediately, results come via events
    await commands.generateParser(parserDirectory);
  }

  function clearStatus() {
    if (statusTimeout) {
      clearTimeout(statusTimeout);
      statusTimeout = null;
    }
    statusMessage = null;
    showStatusDetails = false;
  }

  async function parse() {
    if (!parserDirectory || buildStatus !== "success") return;
    setStatus("Parsing...", "info");

    // Reset previous results
    sppf = null;
    gss = null;
    parseTree = null;
    parseResultAvailable = false;
    parseTreeSelectedSpan = null;
    sppfSelectedSpan = null;

    logCommand(`${parserName} <input> --start ${startNonterminal}`);

    const result = await commands.parse(parserDirectory, inputText, startNonterminal!);
    if (result.status === "ok") {
      parseResultAvailable = true;
      lastParsedInput = inputText;
      logOutput("Parse successful");
      setStatus("Parse successful", "success");

      // Fetch the data for the active tab
      if (activeTab === "sppf") {
        await fetchSppf();
      } else if (activeTab === "gss") {
        await fetchGss();
      } else if (activeTab === "parse-tree") {
        await fetchParseTree();
      }
    } else {
      parseResultAvailable = false;
      logError(result.error);
      outputPanelOpen = true;  // Only auto-open on error
      setStatus("Parse failed", "error");
    }
  }

  async function fetchSppf() {
    if (!parseResultAvailable) return;
    const result = await commands.getSppf();
    if (result.status === "ok") {
      sppf = result.data;
      logOutput(`SPPF: ${result.data.nodes.length} nodes, ${result.data.edges.length} edges`);
    } else {
      logError(`Failed to load SPPF: ${result.error}`);
    }
  }

  async function fetchGss() {
    if (!parseResultAvailable) return;
    const result = await commands.getGss();
    if (result.status === "ok") {
      gss = result.data;
      logOutput(`GSS: ${result.data.nodes.length} nodes, ${result.data.edges.length} edges`);
    } else {
      logError(`Failed to load GSS: ${result.error}`);
    }
  }

  async function fetchParseTree() {
    if (!parseResultAvailable) return;
    const result = await commands.getParseTree();
    if (result.status === "ok") {
      try {
        parseTree = JSON.parse(result.data) as ParseTree;
        logOutput(`Parse Tree: ${parseTree.nodes.length} nodes, ${parseTree.edges.length} edges`);
      } catch (e) {
        logError(`Failed to parse parse tree JSON: ${e}`);
      }
    } else {
      logError(`Failed to load parse tree: ${result.error}`);
    }
  }

  // Graph controls (work with active graph)
  // Generic graph control functions
  function adjustZoomGraph(graph: cytoscape.Core | null, factor: number) {
    if (graph) {
      graph.zoom(graph.zoom() * factor);
    }
  }

  function resetViewGraph(graph: cytoscape.Core | null) {
    if (graph) {
      graph.fit();
      capZoom(graph);
    }
  }

  // Parse mode convenience functions
  function getActiveGraph(): cytoscape.Core | null {
    switch (activeTab) {
      case "sppf": return cy;
      case "gss": return gssCy;
      case "parse-tree": return parseTreeCy;
    }
  }

  function zoomIn() {
    adjustZoomGraph(getActiveGraph(), 1.2);
  }

  function zoomOut() {
    adjustZoomGraph(getActiveGraph(), 1/1.2);
  }

  function resetView() {
    resetViewGraph(getActiveGraph());
  }

  function expandAll() {
    if (activeTab === "sppf") {
      sppfCollapseManager.expandAll();
    } else if (activeTab === "parse-tree") {
      parseTreeCollapseManager.expandAll();
    }
  }

  function exportGraph() {
    const graph = getActiveGraph();
    const filename = activeTab === "parse-tree" ? "parse-tree" : activeTab;
    exportGraphPng(graph, filename);
  }

  function toggleSpans() {
    showSpans = !showSpans;
    // Preserve selection state before re-rendering
    const savedParseTreeSelection = parseTreeSelectedNodeId;
    const savedParseTreeSpan = parseTreeSelectedSpan;
    const savedSppfSelection = sppfSelectedNodeId;
    const savedSppfSpan = sppfSelectedSpan;
    const savedDebugSelection = selectedNodeId;
    const savedDebugSpan = selectedSpan;

    // Re-render affected graphs
    if (parseTree) {
      tick().then(() => {
        renderParseTree();
        // Restore selection
        if (savedParseTreeSelection && parseTreeCy) {
          parseTreeSelectedNodeId = savedParseTreeSelection;
          parseTreeSelectedSpan = savedParseTreeSpan;
          parseTreeCy.getElementById(savedParseTreeSelection).addClass('selected');
        }
      });
    }
    if (sppf) {
      tick().then(() => {
        renderSPPF();
        // Restore selection
        if (savedSppfSelection && cy) {
          sppfSelectedNodeId = savedSppfSelection;
          sppfSelectedSpan = savedSppfSpan;
          cy.getElementById(savedSppfSelection).addClass('selected');
        }
      });
    }
    if (debugSppfNodes.length > 0) {
      tick().then(() => {
        renderDebugSppf();
        // Restore selection
        if (savedDebugSelection && debugSppfCy) {
          selectedNodeId = savedDebugSelection;
          selectedSpan = savedDebugSpan;
          debugSppfCy.getElementById(savedDebugSelection).addClass('selected');
        }
      });
    }
    // Notify popup windows about the change
    notifyPopupWindowsSpansChanged();
  }

  function notifyPopupWindowsSpansChanged() {
    // Re-send data to popup windows with updated showSpans
    for (const [graphType, webview] of graphWindows) {
      if (graphType === 'debugSppf') {
        webview.emit('graph-data-debug-sppf', {
          nodes: debugSppfNodes,
          current_node_id: currentSppfNodeId,
          show_spans: showSpans,
        });
      }
    }
  }

  // Graph window management functions (separate OS windows)
  function getGraphTitle(graphType: GraphType): string {
    switch (graphType) {
      case 'sppf': return 'SPPF';
      case 'gss': return 'GSS';
      case 'debugSppf': return 'SPPF (Debug)';
      case 'debugGss': return 'GSS (Debug)';
    }
  }

  async function openGraphWindow(graphType: GraphType) {
    // If window already exists, focus it
    const existing = graphWindows.get(graphType);
    if (existing) {
      await existing.setFocus();
      return;
    }

    const webview = new WebviewWindow(`graph-${graphType}`, {
      url: `/graph?type=${graphType}`,
      title: getGraphTitle(graphType),
      width: 800,
      height: 600,
      center: true,
      titleBarStyle: 'overlay',
      hiddenTitle: true,
    });

    webview.once('tauri://created', () => {
      graphWindows.set(graphType, webview);
      // Send initial data after a short delay to ensure the window is ready
      setTimeout(() => sendGraphData(graphType), 100);
    });

    webview.once('tauri://destroyed', () => {
      graphWindows.delete(graphType);
    });
  }

  async function sendGraphData(graphType: GraphType) {
    const webview = graphWindows.get(graphType);
    if (!webview) return;

    switch (graphType) {
      case 'sppf':
        if (sppf) {
          await webview.emit('graph-data-sppf', sppf);
        }
        break;
      case 'gss':
        if (gss) {
          await webview.emit('graph-data-gss', gss);
        }
        break;
      case 'debugSppf':
        await webview.emit('graph-data-debug-sppf', {
          nodes: debugSppfNodes,
          current_node_id: currentSppfNodeId,
          show_spans: showSpans,
        });
        break;
      case 'debugGss':
        await webview.emit('graph-data-debug-gss', {
          nodes: debugGssNodes,
          edges: debugGssEdges,
          current_gss_node_id: currentGssNodeId,
        });
        break;
    }
  }

  // Notify all open debug graph windows of updates
  async function notifyDebugGraphWindows() {
    if (graphWindows.has('debugSppf')) {
      await sendGraphData('debugSppf');
    }
    if (graphWindows.has('debugGss')) {
      await sendGraphData('debugGss');
    }
  }

  // Close all graph windows
  async function closeAllGraphWindows() {
    for (const [, webview] of graphWindows) {
      await webview.close();
    }
    graphWindows.clear();
  }

  function stopDebug() {
    debugLoaded = false;
    currentStep = 0;
    totalSteps = 0;
    currentAction = null;
    descriptorSet = [];
    callStack = [];
    inputIndex = null;
    selectedSpan = null;
    selectedNodeId = null;
    debugSppfNodes = [];
    currentSppfNodeId = null;
    if (debugSppfCy) {
      debugSppfCy.destroy();
      debugSppfCy = null;
    }
    debugGssNodes = [];
    debugGssEdges = [];
    currentGssNodeId = null;
    if (debugGssCy) {
      debugGssCy.destroy();
      debugGssCy = null;
    }
    // Reset to default proportions
    debugActionHeight = null;
    debugPendingHeight = null;
    debugSppfHeight = null;
  }

  // Copy functionality
  let copiedFrame = $state<number | null>(null);
  let copiedAll = $state(false);

  async function copyStackFrame(frame: string, index: number) {
    await navigator.clipboard.writeText(frame);
    copiedFrame = index;
    setTimeout(() => { copiedFrame = null; }, 1500);
  }

  async function copyAllStackFrames() {
    const text = callStack.join('\n');
    await navigator.clipboard.writeText(text);
    copiedAll = true;
    setTimeout(() => { copiedAll = false; }, 1500);
  }

  function clearNodeSelection() {
    selectedSpan = null;
    selectedNodeId = null;
  }

  async function startDebug() {
    if (!parserDirectory || buildStatus !== "success" || !startNonterminal) return;

    setStatus("Loading debug trace...", "info");
    debugLoaded = false;
    callStack = [];
    currentStep = 0;
    totalSteps = 0;
    currentAction = null;
    descriptorSet = [];
    inputIndex = null;

    const result = await commands.loadDebugTrace(parserDirectory, inputText, startNonterminal);
    if (result.status === "ok") {
      const { input_path, symbols_path, trace_path, current_action, descriptor_set, input_index } = result.data;
      logCommand(`${parserName} --write-symbols ${symbols_path}`);
      logCommand(`${parserName} ${input_path} --start ${startNonterminal} --trace ${trace_path} --format json`);
      debugLoaded = true;
      currentStep = result.data.current_step;
      totalSteps = result.data.total_steps;
      currentAction = current_action;
      descriptorSet = descriptor_set;
      inputIndex = input_index ?? null;
      logOutput(`Loaded ${totalSteps} steps`);
      setStatus(`Loaded ${totalSteps} steps`, "success");
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    } else {
      logCommand(`${parserName} --write-symbols <symbols.json>`);
      logCommand(`${parserName} <input> --start ${startNonterminal} --trace <trace.json> --format json`);
      setStatus("Debug failed", "error");
      logError(result.error);
      outputPanelOpen = true;
    }
  }

  async function stepBack() {
    if (!debugLoaded || currentStep === 0) return;
    clearNodeSelection();
    const result = await commands.debugStepTo(currentStep - 1);
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function stepForward() {
    if (!debugLoaded || currentStep >= totalSteps - 1) return;
    clearNodeSelection();
    const result = await commands.debugStepForward();
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function stepTo(target: number) {
    if (!debugLoaded) return;
    clearNodeSelection();
    const result = await commands.debugStepTo(target);
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function fetchStackTrace() {
    const result = await commands.getStackTrace();
    if (result.status === "ok") {
      callStack = result.data;
    } else {
      callStack = [];
    }
  }

  async function fetchDebugSppf() {
    const result = await commands.getDebugSppf();
    if (result.status === "ok") {
      debugSppfNodes = result.data.nodes;
      currentSppfNodeId = result.data.current_node_id;
    } else {
      debugSppfNodes = [];
      currentSppfNodeId = null;
    }
  }

  async function fetchDebugGSS() {
    const result = await commands.getDebugGss();
    if (result.status === "ok") {
      debugGssNodes = result.data.nodes;
      debugGssEdges = result.data.edges;
      currentGssNodeId = result.data.current_gss_node_id;
    } else {
      debugGssNodes = [];
      debugGssEdges = [];
      currentGssNodeId = null;
    }
  }

  function startVerticalDrag(e: MouseEvent) {
    isDraggingVertical = true;
    e.preventDefault();
  }

  function startHorizontalDrag(e: MouseEvent) {
    isDraggingHorizontal = true;
    e.preventDefault();
  }

  function startInputDrag(e: MouseEvent) {
    isDraggingInput = true;
    e.preventDefault();
  }

  function startCurrentDrag(e: MouseEvent) {
    isDraggingCurrent = true;
    e.preventDefault();
  }

  function startOutputDrag(e: MouseEvent) {
    isDraggingOutput = true;
    e.preventDefault();
  }

  function startDebugResize1(e: MouseEvent) {
    isDraggingDebug1 = true;
    e.preventDefault();
  }

  function startDebugResize2(e: MouseEvent) {
    isDraggingDebug2 = true;
    e.preventDefault();
  }

  function startDebugActionResize(e: MouseEvent) {
    isDraggingDebugAction = true;
    resizeStartY = e.clientY;
    // Capture current Action height (from grid or default)
    resizeStartActionHeight = debugActionHeight ?? 120;
    e.preventDefault();
  }

  function startDebugStackResize(e: MouseEvent) {
    isDraggingDebugStack = true;
    resizeStartY = e.clientY;
    // Capture current Pending height (from grid or default)
    resizeStartPendingHeight = debugPendingHeight ?? 150;
    e.preventDefault();
  }

  function startDebugGraphResize(e: MouseEvent) {
    isDraggingDebugGraph = true;
    e.preventDefault();
  }

  function onMouseMove(e: MouseEvent) {
    if (isDraggingVertical) {
      leftPanelWidth = Math.max(250, Math.min(600, e.clientX));
    }
    if (isDraggingHorizontal) {
      const container = document.querySelector('.right-panel');
      if (container) {
        const rect = container.getBoundingClientRect();
        callStackHeight = Math.max(100, Math.min(400, rect.bottom - e.clientY));
      }
    }
    if (isDraggingInput) {
      const header = document.querySelector('.header');
      if (header) {
        const rect = header.getBoundingClientRect();
        inputHeight = Math.max(100, Math.min(400, e.clientY - rect.bottom));
      }
    }
    if (isDraggingCurrent) {
      const section = document.querySelector('.current-section');
      if (section) {
        const rect = section.getBoundingClientRect();
        currentDescHeight = Math.max(50, Math.min(200, e.clientY - rect.top));
      }
    }
    if (isDraggingOutput) {
      outputPanelHeight = Math.max(80, Math.min(400, window.innerHeight - e.clientY - 30));
    }
    if (isDraggingDebug1) {
      // Resize first debug column (account for activity bar width of 48px)
      // Max width leaves room for: min col2 (250px) + col3
      const maxCol1 = window.innerWidth - 48 - 250 - debugCol3Width;
      debugCol1Width = Math.max(200, Math.min(maxCol1, e.clientX - 48));
    }
    if (isDraggingDebug2) {
      // Resize third debug column from the right edge
      // Max width leaves room for: activity bar (48px) + col1 + min col2 (250px)
      const maxCol3 = window.innerWidth - 48 - debugCol1Width - 250;
      debugCol3Width = Math.max(200, Math.min(maxCol3, window.innerWidth - e.clientX));
    }
    if (isDraggingDebugAction) {
      // Resize Action - Call Stack (1fr) absorbs the change
      const delta = e.clientY - resizeStartY;
      const newHeight = resizeStartActionHeight + delta;
      // Get container height to calculate max (leave room for Call Stack min 100px + Pending + handles + playback)
      const container = document.querySelector('.debug-col-stack') as HTMLElement;
      const playback = document.querySelector('.debug-col-stack .playback-controls') as HTMLElement;
      const playbackHeight = playback?.getBoundingClientRect().height ?? 40;
      const pendingHeight = debugPendingHeight ?? 150;
      const containerHeight = container?.getBoundingClientRect().height ?? 600;
      const maxAction = containerHeight - playbackHeight - 100 - 8 - pendingHeight; // 100 = min Call Stack, 8 = handles
      debugActionHeight = Math.max(80, Math.min(maxAction, newHeight));
    }
    if (isDraggingDebugStack) {
      // Resize Pending - Call Stack (1fr) absorbs the change
      const delta = e.clientY - resizeStartY;
      // Note: dragging DOWN should SHRINK pending (delta is positive, height decreases)
      const newHeight = resizeStartPendingHeight - delta;
      // Get container height to calculate max (leave room for Call Stack min 100px + Action + handles + playback)
      const container = document.querySelector('.debug-col-stack') as HTMLElement;
      const playback = document.querySelector('.debug-col-stack .playback-controls') as HTMLElement;
      const playbackHeight = playback?.getBoundingClientRect().height ?? 40;
      const actionHeight = debugActionHeight ?? 120;
      const containerHeight = container?.getBoundingClientRect().height ?? 600;
      const maxPending = containerHeight - playbackHeight - 100 - 8 - actionHeight; // 100 = min Call Stack, 8 = handles
      debugPendingHeight = Math.max(80, Math.min(maxPending, newHeight));
    }
    if (isDraggingDebugGraph) {
      // Resize SPPF height within right column
      const graphSection = document.querySelector('.debug-col-graphs');
      if (graphSection) {
        const rect = graphSection.getBoundingClientRect();
        debugSppfHeight = Math.max(100, Math.min(500, e.clientY - rect.top));
      }
    }
  }

  function onMouseUp() {
    isDraggingVertical = false;
    isDraggingHorizontal = false;
    isDraggingInput = false;
    isDraggingCurrent = false;
    isDraggingOutput = false;
    isDraggingDebug1 = false;
    isDraggingDebug2 = false;
    isDraggingDebugAction = false;
    isDraggingDebugStack = false;
    isDraggingDebugGraph = false;
  }

  function handleWindowClick(e: MouseEvent) {
    // Close dropdown when clicking outside
    const target = e.target as HTMLElement;
    if (!target.closest('.custom-dropdown')) {
      dropdownOpen = false;
    }
  }

  function startWindowDrag() {
    getCurrentWindow().startDragging();
  }

  const toggleMaximize = createMaximizeToggle();

  function handleKeyDown(e: KeyboardEvent) {
    // Escape to deselect text and blur active element
    if (e.key === 'Escape') {
      window.getSelection()?.removeAllRanges();
      (document.activeElement as HTMLElement)?.blur();
      return;
    }

    // Prevent Cmd+A from selecting all unless in a valid text area
    if ((e.metaKey || e.ctrlKey) && e.key === 'a') {
      const target = e.target as HTMLElement;
      // Allow Cmd+A in textareas and inputs
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
      // Allow Cmd+A in our custom selectable containers (they have their own handler)
      if (target.classList.contains('output-content') || target.classList.contains('current-action-box')) return;
      // Block Cmd+A everywhere else
      e.preventDefault();
      return;
    }

    // Only handle keyboard shortcuts when debugging is active
    if (!debugLoaded) return;

    // Ignore if typing in an input/textarea
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      stepBack();
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      stepForward();
    }
  }

  // Handle Cmd+A to select all text within a container only
  function handleSelectAllInContainer(e: KeyboardEvent, container: HTMLElement | null) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'a') {
      e.preventDefault();
      e.stopPropagation();
      if (container) {
        const selection = window.getSelection();
        const range = document.createRange();
        range.selectNodeContents(container);
        selection?.removeAllRanges();
        selection?.addRange(range);
      }
    }
  }
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} onclick={handleWindowClick} onkeydown={handleKeyDown} />

<div class="app" class:dragging={isDraggingVertical || isDraggingHorizontal || isDraggingInput || isDraggingCurrent || isDraggingOutput || isDraggingDebug1 || isDraggingDebug2 || isDraggingDebugAction || isDraggingDebugStack || isDraggingDebugGraph} class:dragging-horizontal={isDraggingHorizontal || isDraggingInput || isDraggingCurrent || isDraggingOutput || isDraggingDebugAction || isDraggingDebugStack || isDraggingDebugGraph}>
  <!-- Title Bar (full width) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="title-bar" onmousedown={startWindowDrag} ondblclick={toggleMaximize}>
    <div class="title-bar-left">
      <!-- Space for macOS traffic lights -->
    </div>
    <div class="title-bar-center">
      <button class="command-palette" onclick={selectDirectory} onmousedown={(e) => e.stopPropagation()}>
        <div class="palette-content">
          {#if parserName && parserDirectory}
            <span class="palette-name">{parserName}</span>
            <span class="palette-separator">—</span>
            <span class="palette-path">{parserDirectory}</span>
          {:else if parserDirectory}
            <span class="palette-path">{parserDirectory}</span>
          {:else}
            <FolderOpen size={14} />
            <span class="palette-placeholder">Open Parser...</span>
          {/if}
        </div>
        <div class="palette-status-area">
          {#if isBuilding}
            <Loader2 size={14} class="spinning" />
          {:else if showReadyStatus}
            <CheckCircle size={14} class="palette-status-success" />
          {:else if buildStatus === "error"}
            <AlertTriangle size={14} class="palette-status-error" />
          {/if}
        </div>
      </button>
    </div>
    <div class="title-bar-right">
    </div>
  </div>

  <!-- Middle Area (activity bar + content) -->
  <div class="middle-area">
    <!-- Activity Bar -->
    <div class="activity-bar">
      <button
        class="activity-btn"
        class:active={activeMode === "parse"}
        onclick={() => activeMode = "parse"}
        title="Parse"
      >
        <GitFork size={24} style="transform: rotate(180deg)" />
      </button>
      <button
        class="activity-btn"
        class:active={activeMode === "debug"}
        onclick={() => activeMode = "debug"}
        title="Debug"
      >
        <Bug size={24} />
      </button>
      <button
        class="activity-btn"
        class:active={activeMode === "design"}
        onclick={() => activeMode = "design"}
        title="Design"
      >
        <Braces size={24} />
      </button>
    </div>

    <!-- Main Area -->
    <div class="main-area">
    <!-- Mode Content -->
  {#if activeMode === "parse"}
  <!-- Parse Mode -->
  <div class="main-content">
    <!-- Left Panel -->
    <div class="left-panel" style="width: {leftPanelWidth}px">
      <!-- Header -->
      <div class="header">
        <div class="dropdown-wrapper">
          <span class="dropdown-label">Start:</span>
          <div class="custom-dropdown" class:disabled={!parserDirectory || nonterminals.length === 0}>
            <button
              class="dropdown-trigger"
              onclick={() => dropdownOpen = !dropdownOpen}
              disabled={!parserDirectory || nonterminals.length === 0}
            >
              <span class="dropdown-value">{startNonterminal || "Select..."}</span>
              <ChevronDown size={14} class="dropdown-chevron" />
            </button>
            {#if dropdownOpen}
              <div class="dropdown-menu">
                {#each nonterminals as nt}
                  <button
                    class="dropdown-item"
                    class:selected={nt === startNonterminal}
                    onclick={() => { startNonterminal = nt; dropdownOpen = false; }}
                  >
                    {nt}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        <button class="parse-btn" onclick={parse} disabled={!parserDirectory || buildStatus !== "success" || !startNonterminal}>Parse</button>
      </div>

    <!-- Input Area -->
    <div class="input-section">
      {#if parseTreeSelectedSpan !== null}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="input-viewer" onclick={() => parseTreeSelectedSpan = null}>{#each inputText.split('') as char, i}<span class="input-char" class:selected={i >= parseTreeSelectedSpan.start && i < parseTreeSelectedSpan.end}>{char}</span>{/each}</div>
      {:else if sppfSelectedSpan !== null}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="input-viewer" onclick={() => sppfSelectedSpan = null}>{#each inputText.split('') as char, i}<span class="input-char" class:selected={i >= sppfSelectedSpan.left && i < sppfSelectedSpan.right}>{char}</span>{/each}</div>
      {:else}
        <textarea
          bind:value={inputText}
          placeholder="Enter code to parse..."
          spellcheck="false"
          onkeydown={(e) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
              e.preventDefault();
              if (inputText !== lastParsedInput) {
                parse();
              }
            }
          }}
        ></textarea>
      {/if}
    </div>
  </div>

  <!-- Vertical Resize Handle -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="resize-handle-vertical" onmousedown={startVerticalDrag}></div>

  <!-- Right Panel -->
  <div class="right-panel">
    <!-- Graph Tabs -->
    <div class="graph-section">
      <div class="tabs">
        <button
          class:active={activeTab === "parse-tree"}
          onclick={() => activeTab = "parse-tree"}
        >Parse Tree</button>
        <button
          class:active={activeTab === "sppf"}
          onclick={() => activeTab = "sppf"}
        >SPPF</button>
        <button
          class:active={activeTab === "gss"}
          onclick={() => activeTab = "gss"}
        >GSS</button>
      </div>
      <div class="graph-container">
        {#if activeTab === "parse-tree"}
          {#if parseTree}
            <div class="cytoscape-container" bind:this={parseTreeContainer}></div>
            <div class="graph-controls">
              <button onclick={zoomIn} title="Zoom in">
                <ZoomIn size={16} />
              </button>
              <button onclick={zoomOut} title="Zoom out">
                <ZoomOut size={16} />
              </button>
              <button onclick={resetView} title="Reset view">
                <Maximize2 size={16} />
              </button>
              <button onclick={expandAll} title="Expand all (double-click node to collapse)">
                <Expand size={16} />
              </button>
              <button onclick={toggleSpans} title={showSpans ? "Hide spans" : "Show spans"}>
                {#if showSpans}
                  <FoldHorizontal size={16} />
                {:else}
                  <UnfoldHorizontal size={16} />
                {/if}
              </button>
              <button onclick={exportGraph} title="Export as PNG">
                <Download size={16} />
              </button>
            </div>
          {:else}
            <div class="graph-placeholder">Parse input to see Parse Tree</div>
          {/if}
        {:else if activeTab === "sppf"}
          {#if sppf}
            <div class="cytoscape-container" bind:this={sppfContainer}></div>
            <div class="graph-controls">
              <button onclick={zoomIn} title="Zoom in">
                <ZoomIn size={16} />
              </button>
              <button onclick={zoomOut} title="Zoom out">
                <ZoomOut size={16} />
              </button>
              <button onclick={resetView} title="Reset view">
                <Maximize2 size={16} />
              </button>
              <button onclick={expandAll} title="Expand all (double-click node to collapse)">
                <Expand size={16} />
              </button>
              <button onclick={toggleSpans} title={showSpans ? "Hide spans" : "Show spans"}>
                {#if showSpans}
                  <FoldHorizontal size={16} />
                {:else}
                  <UnfoldHorizontal size={16} />
                {/if}
              </button>
              <button onclick={exportGraph} title="Export as PNG">
                <Download size={16} />
              </button>
              <button onclick={() => openGraphWindow('sppf')} title="Pop out">
                <Fullscreen size={16} />
              </button>
            </div>
          {:else}
            <div class="graph-placeholder">Parse input to see SPPF</div>
          {/if}
        {:else if activeTab === "gss"}
          {#if gss}
            <div class="cytoscape-container" bind:this={gssContainer}></div>
            <div class="graph-controls">
              <button onclick={zoomIn} title="Zoom in">
                <ZoomIn size={16} />
              </button>
              <button onclick={zoomOut} title="Zoom out">
                <ZoomOut size={16} />
              </button>
              <button onclick={resetView} title="Reset view">
                <Maximize2 size={16} />
              </button>
              <button onclick={exportGraph} title="Export as PNG">
                <Download size={16} />
              </button>
              <button onclick={() => openGraphWindow('gss')} title="Pop out">
                <Fullscreen size={16} />
              </button>
            </div>
          {:else}
            <div class="graph-placeholder">Parse input to see GSS</div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
  </div>
  {:else if activeMode === "debug"}
  <!-- Debug Mode - Three Column Layout -->
  <div class="debug-layout">
    <!-- Column 1: Input -->
    <div class="debug-column debug-col-input" style="width: {debugCol1Width}px">
      <!-- Header -->
      <div class="header">
        <div class="dropdown-wrapper">
          <span class="dropdown-label">Start:</span>
          <div class="custom-dropdown" class:disabled={!parserDirectory || nonterminals.length === 0}>
            <button
              class="dropdown-trigger"
              onclick={() => dropdownOpen = !dropdownOpen}
              disabled={!parserDirectory || nonterminals.length === 0}
            >
              <span class="dropdown-value">{startNonterminal || "Select..."}</span>
              <ChevronDown size={14} class="dropdown-chevron" />
            </button>
            {#if dropdownOpen}
              <div class="dropdown-menu">
                {#each nonterminals as nt}
                  <button
                    class="dropdown-item"
                    class:selected={nt === startNonterminal}
                    onclick={() => { startNonterminal = nt; dropdownOpen = false; }}
                  >
                    {nt}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        {#if debugLoaded}
          <button class="parse-btn" onclick={stopDebug}>Stop</button>
        {:else}
          <button class="parse-btn" onclick={startDebug} disabled={!parserDirectory || buildStatus !== "success" || !startNonterminal}>Debug</button>
        {/if}
      </div>

      <!-- Input Area -->
      <div class="input-section">
        {#if debugLoaded}
          <div class="input-viewer">{#each inputText.split('') as char, i}<span class="input-char" class:consumed={inputIndex !== null && i < inputIndex} class:current={inputIndex !== null && i === inputIndex} class:selected={selectedSpan !== null && i >= selectedSpan.left && i < selectedSpan.right}>{char}</span>{/each}</div>
        {:else}
          <textarea
            bind:value={inputText}
            placeholder="Enter code to parse..."
            spellcheck="false"
          ></textarea>
        {/if}
      </div>
    </div>

    <!-- Resize Handle 1 -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="resize-handle-vertical" onmousedown={startDebugResize1}></div>

    <!-- Column 2: Stack + Pending -->
    <div class="debug-column debug-col-stack" style={`display: grid; grid-template-rows: auto minmax(80px, ${debugActionHeight ?? 120}px) 4px minmax(100px, 1fr) 4px minmax(80px, ${debugPendingHeight ?? 150}px);`}>
      <!-- Playback Controls -->
      <div class="playback-controls">
        <button onclick={stepBack} disabled={!debugLoaded || currentStep === 0}>◀</button>
        <button onclick={stepForward} disabled={!debugLoaded || currentStep >= totalSteps - 1}>▶</button>
        {#if debugLoaded}
          <span class="step-counter">Step {currentStep + 1} / {totalSteps}</span>
        {/if}
      </div>

      <!-- Current Action -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div
        class="current-action-box"
        bind:this={actionBoxEl}
        tabindex="0"
        onkeydown={(e) => handleSelectAllInContainer(e, actionBoxEl)}
      >
        {#if currentAction}
          <pre class:match-failed={currentAction.startsWith('Match Failed')}>{currentAction}</pre>
        {:else}
          <span class="placeholder">No action</span>
        {/if}
      </div>

      <!-- Resize Handle (between action and stack) -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle-horizontal" onmousedown={startDebugActionResize}></div>

      <!-- Call Stack -->
      <div class="section call-stack">
        <div class="section-header">
          <span>Call Stack</span>
          {#if callStack.length > 0}
            <button class="copy-btn" onclick={copyAllStackFrames} title="Copy all">
              {#if copiedAll}
                <ClipboardCheck size={12} />
              {:else}
                <Copy size={12} />
              {/if}
            </button>
          {/if}
        </div>
        <div class="stack-list">
          {#if callStack.length > 0}
            {#each callStack as frame, i}
              <div
                class="stack-frame"
                class:current={i === 0}
                class:copied={copiedFrame === i}
              >
                <code>{frame}</code>
                <button
                  class="frame-copy-btn"
                  class:copied={copiedFrame === i}
                  onclick={() => copyStackFrame(frame, i)}
                  title="Copy"
                >
                  {#if copiedFrame === i}
                    <ClipboardCheck size={12} />
                  {:else}
                    <Copy size={12} />
                  {/if}
                </button>
              </div>
            {/each}
          {:else}
            <span class="placeholder">{debugLoaded ? "No stack at current step" : "Click Debug to start"}</span>
          {/if}
        </div>
      </div>

      <!-- Resize Handle (horizontal) -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle-horizontal" onmousedown={startDebugStackResize}></div>

      <!-- Pending Descriptors -->
      <div class="section pending-descriptors">
        <div class="section-header">Pending Descriptors ({descriptorSet.length})</div>
        <div class="section-content">
          {#if descriptorSet.length > 0}
            <ul>
              {#each descriptorSet as desc}
                <li><code>{desc}</code></li>
              {/each}
            </ul>
          {:else}
            <span class="placeholder">Empty</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Resize Handle 2 -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="resize-handle-vertical" onmousedown={startDebugResize2}></div>

    <!-- Column 3: SPPF (top) / GSS (bottom) -->
    <div class="debug-column debug-col-graphs" style="width: {debugCol3Width}px">
      <!-- SPPF Section -->
      <div class="debug-graph-section" style={debugSppfHeight !== null ? `height: ${debugSppfHeight}px; flex: 0 0 auto` : ''}>
        <div class="section-header">SPPF</div>
        <div class="graph-container">
          {#if debugSppfNodes.length === 0}
            <div class="graph-placeholder">No SPPF nodes yet</div>
          {:else}
            <div class="cytoscape-container" bind:this={debugSppfContainer}></div>
            <div class="graph-controls">
              <button onclick={() => adjustZoomGraph(debugSppfCy, 1.2)} title="Zoom in">
                <ZoomIn size={16} />
              </button>
              <button onclick={() => adjustZoomGraph(debugSppfCy, 1/1.2)} title="Zoom out">
                <ZoomOut size={16} />
              </button>
              <button onclick={() => resetViewGraph(debugSppfCy)} title="Reset view">
                <Maximize2 size={16} />
              </button>
              <button onclick={toggleSpans} title={showSpans ? "Hide spans" : "Show spans"}>
                {#if showSpans}
                  <FoldHorizontal size={16} />
                {:else}
                  <UnfoldHorizontal size={16} />
                {/if}
              </button>
              <button onclick={() => exportGraphPng(debugSppfCy, 'debug-sppf')} title="Export as PNG">
                <Download size={16} />
              </button>
              <button onclick={() => openGraphWindow('debugSppf')} title="Pop out">
                <Fullscreen size={16} />
              </button>
            </div>
          {/if}
        </div>
      </div>

      <!-- Resize Handle -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle-horizontal" onmousedown={startDebugGraphResize}></div>

      <!-- GSS Section -->
      <div class="debug-graph-section">
        <div class="section-header">GSS</div>
        <div class="graph-container">
          {#if debugGssNodes.length === 0}
            <div class="graph-placeholder">No GSS nodes yet</div>
          {:else}
            <div class="cytoscape-container" bind:this={debugGssContainer}></div>
            <div class="graph-controls">
              <button onclick={() => adjustZoomGraph(debugGssCy, 1.2)} title="Zoom in">
                <ZoomIn size={16} />
              </button>
              <button onclick={() => adjustZoomGraph(debugGssCy, 1/1.2)} title="Zoom out">
                <ZoomOut size={16} />
              </button>
              <button onclick={() => resetViewGraph(debugGssCy)} title="Reset view">
                <Maximize2 size={16} />
              </button>
              <button onclick={() => exportGraphPng(debugGssCy, 'debug-gss')} title="Export as PNG">
                <Download size={16} />
              </button>
              <button onclick={() => openGraphWindow('debugGss')} title="Pop out">
                <Fullscreen size={16} />
              </button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
  {:else if activeMode === "design"}
  <!-- Design Mode -->
  <div class="design-mode">
    <div class="design-header">
      <button
        class="generate-btn"
        onclick={generateParser}
        disabled={!parserDirectory || isGenerating}
      >
        {#if isGenerating}
          <Loader2 size={16} class="spinning" />
          Generating...
        {:else}
          Generate Parser
        {/if}
      </button>
      {#if generateStatus === "success"}
        <span class="generate-status success">
          <CheckCircle size={16} />
          Generated successfully
        </span>
      {:else if generateStatus === "error"}
        <span class="generate-status error">
          <AlertTriangle size={16} />
          Generation failed
        </span>
      {/if}
    </div>
    {#if generateError}
      <div class="generate-error">
        <pre>{generateError}</pre>
      </div>
    {/if}
    <div class="design-placeholder">
      <Braces size={48} />
      <p>Grammar editor coming soon</p>
    </div>
  </div>
  {/if}

    </div>
  </div>

  <!-- Output Panel (overlay) -->
  {#if outputPanelOpen}
    <div class="output-panel-overlay">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle-horizontal" onmousedown={startOutputDrag}></div>
      <div class="output-panel open">
        <div class="output-header">
          <span class="output-title">Output</span>
          <div class="output-header-buttons">
            <button class="output-header-btn" onclick={clearOutput} title="Clear output">
              <Trash2 size={20} />
            </button>
            <button class="output-header-btn" onclick={() => outputPanelOpen = false} title="Hide output">
              <ChevronsDown size={20} />
            </button>
          </div>
        </div>
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <div
          class="output-content"
          style="height: {outputPanelHeight}px"
          bind:this={outputContentEl}
          tabindex="0"
          onkeydown={(e) => handleSelectAllInContainer(e, outputContentEl)}
        >
          {#if outputLog.length > 0}
            {#each outputLog as entry}
              <pre class="log-entry {entry.type}">{entry.content}</pre>
            {/each}
          {:else}
            <span class="placeholder">No output</span>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Status Bar (full width) -->
  <div class="status-bar">
    <div class="status-left">
      <button class="status-text-btn" onclick={() => outputPanelOpen = !outputPanelOpen}>
        {#if isBuilding}
          Building...
        {:else if isGenerating}
          Generating...
        {:else if statusMessage}
          {statusMessage}
        {:else if parserDirectory && buildStatus === "success"}
          Ready
        {:else}
          No parser selected
        {/if}
      </button>
    </div>
    <div class="status-right">
      <button
        class="status-icon-btn"
        class:active={outputPanelOpen}
        onclick={() => outputPanelOpen = !outputPanelOpen}
        title="Toggle Output Panel"
      >
        <PanelBottom size={14} />
      </button>
    </div>
  </div>

  <!-- Error Modal -->
  {#if showErrorModal}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal-overlay" onclick={closeErrorModal}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="modal" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <AlertTriangle size={20} color="#f48771" />
          <span>Invalid Parser Directory</span>
          <button class="modal-close" onclick={closeErrorModal}>
            <X size={18} />
          </button>
        </div>
        <div class="modal-body">
          <p>{errorModalMessage}</p>
          <p class="modal-hint">Please select a directory containing a generated Iguana parser.</p>
        </div>
        <div class="modal-footer">
          <button class="modal-btn" onclick={closeErrorModal}>OK</button>
        </div>
      </div>
    </div>
  {/if}

</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  :global(*) {
    box-sizing: border-box;
  }

  .app {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 14px;
    background: #1e1e1e;
    color: #d4d4d4;
    user-select: none;  /* Prevent selection of UI elements */
  }

  /* Middle Area (activity bar + content) */
  .middle-area {
    display: flex;
    flex-direction: row;
    flex: 1;
    min-height: 0;
  }

  /* Activity Bar */
  .activity-bar {
    display: flex;
    flex-direction: column;
    width: 48px;
    background: #333333;
    border-right: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .activity-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    background: transparent;
    border: none;
    color: #858585;
    cursor: pointer;
    border-left: 2px solid transparent;
  }

  .activity-btn:hover {
    color: #d4d4d4;
  }

  .activity-btn.active {
    color: #d4d4d4;
    border-left-color: #d4d4d4;
  }

  /* Main Area (right of activity bar) */
  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  /* Mode Placeholder */
  .mode-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #666;
    gap: 16px;
  }

  /* Design Mode */
  .design-mode {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .design-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
  }

  .generate-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: transparent;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    transition: border-color 0.15s, color 0.15s, background 0.15s;
  }

  .generate-btn:hover:not(:disabled) {
    border-color: #888;
    color: #fff;
    background: rgba(255, 255, 255, 0.05);
  }

  .generate-btn:disabled {
    background: transparent;
    color: #555;
    border-color: #3c3c3c;
    cursor: not-allowed;
  }

  .generate-status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
  }

  .generate-status.success {
    color: #89d185;
  }

  .generate-status.error {
    color: #f48771;
  }

  .generate-error {
    padding: 12px 16px;
    background: #2d1f1f;
    border-bottom: 1px solid #3c3c3c;
    max-height: 200px;
    overflow: auto;
  }

  .generate-error pre {
    margin: 0;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 12px;
    color: #f48771;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .design-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #666;
    gap: 16px;
  }

  .design-placeholder p {
    margin: 0;
    font-size: 14px;
  }

  /* Title Bar */
  .title-bar {
    display: flex;
    align-items: center;
    height: 48px;
    background: #1e1e1e;
    border-bottom: 1px solid #454545;
    flex-shrink: 0;
    cursor: default;
  }

  .title-bar-left {
    width: 78px;  /* Space for macOS traffic lights */
    flex-shrink: 0;
  }

  .title-bar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0 16px;
  }

  .title-bar-right {
    width: 78px;  /* Balance with left */
    flex-shrink: 0;
  }

  /* Command Palette */
  .command-palette {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    min-width: 300px;
    max-width: 550px;
    width: 100%;
    background: #2d2d2d;
    border: 1px solid #404040;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    color: #888;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  .command-palette:hover {
    border-color: #555;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.4);
  }

  .command-palette:focus {
    outline: none;
    border-color: #0e639c;
  }

  .palette-content {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .palette-status-area {
    width: 20px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .palette-name {
    font-weight: 600;
    color: #e0e0e0;
    flex-shrink: 0;
  }

  .palette-separator {
    color: #444;
    flex-shrink: 0;
  }

  .palette-path {
    color: #666;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .palette-placeholder {
    color: #666;
  }

  :global(.palette-status-success) {
    color: #89d185;
  }

  :global(.palette-status-error) {
    color: #f48771;
  }

  /* Main Content */
  .main-content {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  /* Dragging state */
  .app.dragging {
    user-select: none;
    cursor: col-resize;
  }

  .app.dragging.dragging-horizontal {
    cursor: row-resize;
  }

  .app.dragging * {
    pointer-events: none;
  }

  .app.dragging .resize-handle-vertical,
  .app.dragging .resize-handle-horizontal {
    pointer-events: auto;
  }

  /* Resize Handles */
  .resize-handle-vertical {
    width: 4px;
    cursor: col-resize;
    background: #3c3c3c;
    transition: background 0.2s;
    flex-shrink: 0;
    position: relative;
    z-index: 5;
  }

  .resize-handle-vertical:hover,
  .app.dragging .resize-handle-vertical {
    background: #0e639c;
  }

  .resize-handle-horizontal {
    height: 4px;
    cursor: row-resize;
    background: #3c3c3c;
    transition: background 0.2s;
    flex-shrink: 0;
    position: relative;
    z-index: 5;
  }

  .resize-handle-horizontal:hover,
  .app.dragging .resize-handle-horizontal {
    background: #0e639c;
  }

  /* Left Panel */
  .left-panel {
    min-width: 250px;
    max-width: 600px;
    display: flex;
    flex-direction: column;
    background: #252526;
  }

  /* Debug Layout - Three Columns */
  .debug-layout {
    flex: 1;
    display: flex;
    flex-direction: row;
    min-height: 0;
    user-select: none;  /* Contain selection within debug layout */
  }

  .debug-column {
    display: flex;
    flex-direction: column;
    background: #252526;
    border-right: 1px solid #3c3c3c;
    user-select: none;  /* Contain selection within columns */
  }

  .debug-column:last-child {
    border-right: none;
  }

  .debug-col-input {
    flex-shrink: 0;
    min-width: 200px;
    max-width: 500px;
  }

  .debug-col-stack {
    flex: 1;
    min-width: 200px;
    /* Grid layout is set via inline style */
    overflow: hidden;
  }

  .debug-col-graphs {
    flex-shrink: 0;
    min-width: 200px;
    display: flex;
    flex-direction: column;
  }

  .debug-graph-section {
    display: flex;
    flex-direction: column;
    min-height: 100px;
    overflow: hidden;
    flex: 1;  /* 50% each (1:1 ratio) */
  }


  .debug-col-stack .call-stack {
    min-height: 0;  /* Allow grid to control size */
    border-bottom: none;
    overflow: auto;
    user-select: none;  /* Contain selection */
  }

  .debug-col-stack .pending-descriptors {
    min-height: 0;  /* Allow grid to control size */
    overflow: auto;
    user-select: none;  /* Contain selection */
  }


  .debug-col-stack .section-content {
    flex: 1;
  }

  .current-action-box {
    padding: 12px;
    background: #252526;
    border-bottom: 1px solid #3c3c3c;
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
    font-size: 13px;
    min-height: 0;  /* Allow grid to control size */
    overflow: auto;
    user-select: text;
  }

  .current-action-box:focus {
    outline: none;
  }

  .current-action-box pre {
    margin: 0;
    white-space: pre-wrap;
    color: #d4d4d4;
    line-height: 1.5;
  }

  .current-action-box pre::first-line {
    color: #4ec9b0;
    font-weight: 600;
  }

  .current-action-box pre.match-failed {
    color: #f48771;
  }

  .current-action-box pre.match-failed::first-line {
    color: #f14c4c;
    font-weight: 600;
  }

  .current-action-box .placeholder {
    color: #666;
    font-style: italic;
  }

  .graph-placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #666;
    font-style: italic;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-bottom: 1px solid #3c3c3c;
    background: #2d2d2d;
  }

  .dropdown-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dropdown-label {
    color: #d4d4d4;
    font-size: 13px;
  }

  .custom-dropdown {
    position: relative;
    width: 150px;
  }

  .custom-dropdown.disabled {
    opacity: 0.5;
  }

  .dropdown-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 5px 8px;
    background: #3c3c3c;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    text-align: left;
  }

  .dropdown-trigger:hover:not(:disabled) {
    background: #454545;
    border-color: #666;
  }

  .dropdown-trigger:focus {
    outline: none;
    border-color: #0e639c;
  }

  .dropdown-trigger:disabled {
    cursor: not-allowed;
  }

  .dropdown-value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  :global(.dropdown-chevron) {
    flex-shrink: 0;
    color: #888;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: #2d2d2d;
    border: 1px solid #454545;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    max-height: 200px;
    overflow-y: auto;
    z-index: 100;
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    color: #d4d4d4;
    border: none;
    cursor: pointer;
    font-size: 13px;
    text-align: left;
  }

  .dropdown-item:hover {
    background: #094771;
  }

  .dropdown-item.selected {
    background: #0e639c;
  }

  .parse-btn {
    margin-left: auto;
    padding: 6px 16px;
    background: transparent;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s, background 0.15s;
  }

  .parse-btn:hover:not(:disabled) {
    border-color: #888;
    color: #fff;
    background: rgba(255, 255, 255, 0.05);
  }

  .parse-btn:disabled {
    background: transparent;
    color: #555;
    border-color: #3c3c3c;
    cursor: not-allowed;
  }


  /* Input Section */
  .input-section {
    flex: 1;
    min-height: 100px;
    user-select: none;  /* Contain selection - allow only in textarea/input-viewer */
  }

  .input-section textarea {
    width: 100%;
    height: 100%;
    resize: none;
    background: #1e1e1e;
    color: #d4d4d4;
    border: none;
    padding: 8px;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 13px;
    user-select: text !important;  /* Allow text selection in textarea */
  }

  .input-section textarea:focus {
    outline: none;
  }

  /* Input Viewer for Debug Mode */
  .input-viewer {
    width: 100%;
    height: 100%;
    padding: 8px;
    background: #1e1e1e;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 13px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text !important;  /* Allow text selection in input viewer */
  }

  /* Ensure text selection works in input viewer */
  .input-section .input-viewer {
    user-select: text !important;
  }

  /* Debug mode specific - ensure text selection works */
  .debug-col-input .input-section textarea,
  .debug-col-input .input-section .input-viewer {
    user-select: text !important;
  }

  .input-char {
    color: #d4d4d4;
  }

  .input-char.consumed {
    color: #6a9955;
  }

  .input-char.current {
    background: #264f78;
    color: #fff;
  }

  .input-char.selected {
    background: #264f78;
    color: #fff;
  }

  /* Playback Controls */
  .playback-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid #3c3c3c;
    background: #2d2d2d;
    flex-shrink: 0;
  }

  .playback-controls button {
    padding: 4px 12px;
    background: #3c3c3c;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
  }

  .playback-controls button:hover:not(:disabled) {
    background: #4c4c4c;
  }

  .playback-controls button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .step-counter {
    font-size: 12px;
    color: #888;
    min-width: 80px;
  }

  /* Sections */
  .section {
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid #3c3c3c;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: #2d2d2d;
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    color: #888;
  }

  .copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    border-radius: 3px;
  }

  .copy-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #d4d4d4;
  }

  .section-content {
    padding: 8px 12px;
    flex: 1;
    overflow-y: auto;
  }

  .section-content ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .section-content li {
    padding: 4px 0;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 12px;
  }

  .placeholder {
    color: #666;
    font-style: italic;
  }

  /* Right Panel */
  .right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  /* Graph Section */
  .graph-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .tabs {
    display: flex;
    background: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
  }

  .tabs button {
    padding: 8px 20px;
    background: transparent;
    color: #888;
    border: none;
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }

  .tabs button.active {
    color: #d4d4d4;
    border-bottom-color: #0e639c;
  }

  .tabs button:hover:not(.active) {
    color: #d4d4d4;
  }

  .graph-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1e1e1e;
    overflow: hidden;
    position: relative;
    min-height: 0;
  }

  .graph-placeholder {
    color: #555;
    font-size: 24px;
  }

  .cytoscape-container {
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }

  .graph-controls {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .graph-container:hover .graph-controls {
    opacity: 1;
  }

  .graph-controls button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: rgba(45, 45, 45, 0.9);
    border: 1px solid #555;
    border-radius: 4px;
    color: #d4d4d4;
    cursor: pointer;
  }

  .graph-controls button:hover {
    background: rgba(60, 60, 60, 0.95);
    border-color: #888;
  }

  /* Call Stack */
  .call-stack {
    min-height: 100px;
    max-height: 400px;
  }

  .stack-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .stack-list .placeholder {
    padding: 8px 12px;
  }

  .stack-frame {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 12px;
    cursor: default;
    user-select: none;
  }

  .stack-frame:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .stack-frame.current {
    background: #094771;
    color: #fff;
  }

  .stack-frame.current:hover {
    background: #0a5286;
  }

  .stack-frame.copied {
    background: rgba(78, 201, 176, 0.2);
  }

  .stack-frame code {
    background: transparent;
    color: inherit;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .frame-copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    margin-left: 8px;
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    border-radius: 3px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .stack-frame:hover .frame-copy-btn {
    opacity: 1;
  }

  .frame-copy-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #d4d4d4;
  }

  .frame-copy-btn.copied {
    opacity: 1;
    color: #4ec9b0;
  }

  code {
    background: transparent;
  }

  /* Output Panel Overlay */
  .output-panel-overlay {
    position: absolute;
    bottom: 32px; /* Above status bar */
    left: 0;
    right: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    user-select: none;  /* Contain selection within overlay */
  }

  /* Output Panel */
  .output-panel {
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    border-top: 1px solid #454545;
    flex-shrink: 0;
  }

  .output-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
  }

  .output-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: #888;
  }

  .output-header-buttons {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .output-header-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    border-radius: 3px;
  }

  .output-header-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #d4d4d4;
  }

  .output-content {
    padding: 8px 12px;
    overflow: auto;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 12px;
    background: #1e1e1e;
    user-select: text;
  }

  .output-content:focus {
    outline: none;
  }

  .output-content .placeholder {
    color: #666;
    font-style: italic;
  }

  .log-entry {
    margin: 0 0 4px 0;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .log-entry.command {
    color: #569cd6;
  }

  .log-entry.command::before {
    content: "$ ";
    color: #6a9955;
  }

  .log-entry.output {
    color: #d4d4d4;
  }

  .log-entry.error {
    color: #f48771;
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #252526;
    border-top: 1px solid #454545;
    flex-shrink: 0;
    height: 32px;
    padding: 0 12px;
  }

  .status-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-text-btn {
    background: transparent;
    border: none;
    font-size: 12px;
    color: #888;
    cursor: pointer;
    padding: 2px 4px;
    margin: -2px -4px;
    border-radius: 3px;
  }

  .status-text-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #d4d4d4;
  }

  .status-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    border-radius: 3px;
  }

  .status-icon-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #d4d4d4;
  }

  .status-icon-btn.active {
    color: #d4d4d4;
  }

  /* Modal */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: #2d2d2d;
    border: 1px solid #3c3c3c;
    border-radius: 8px;
    min-width: 400px;
    max-width: 500px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }

  .modal-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 16px;
    border-bottom: 1px solid #3c3c3c;
    font-weight: 600;
  }

  .modal-header span {
    flex: 1;
  }

  .modal-close {
    padding: 4px;
    background: transparent;
    color: #888;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .modal-close:hover {
    color: #d4d4d4;
  }

  .modal-body {
    padding: 16px;
  }

  .modal-body p {
    margin: 0 0 12px 0;
  }

  .modal-hint {
    color: #888;
    font-size: 13px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 12px 16px;
    border-top: 1px solid #3c3c3c;
  }

  .modal-btn {
    padding: 8px 20px;
    background: #0e639c;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .modal-btn:hover {
    background: #1177bb;
  }

  /* Spinning animation for loader */
  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

</style>
