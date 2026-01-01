<script lang="ts">
  import { commands, type SPPF, type GSS, type DebugInfo, type DebugSPPFNode, type DebugSPPFInfo, type DebugGSSNode, type DebugGSSEdge, type DebugGSSInfo } from "../bindings";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
  import { availableMonitors, currentMonitor } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";
  import { FolderOpen, Hammer, X, AlertTriangle, CheckCircle, Loader2, ChevronDown, ChevronRight, ZoomIn, ZoomOut, Maximize2, Minimize2, Expand, Fullscreen, GitFork, Bug, Braces, PanelBottom, Trash2, ChevronsDown, Copy, ClipboardCheck } from "lucide-svelte";
  import cytoscape from "cytoscape";
  import dagre from "cytoscape-dagre";

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

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenResult.then(fn => fn());
      unlistenGenerateResult.then(fn => fn());
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
  let startNonterminal = $state<string | null>(null);
  let nonterminals = $state<string[]>([]);
  let dropdownOpen = $state(false);

  // Playback state
  let currentStep = $state(0);
  let totalSteps = $state(0);

  // Parser state
  let currentAction = $state<string | null>(null);
  let descriptorSet = $state<string[]>([]);
  let callStack = $state<string[]>([]);
  let debugLoaded = $state(false);
  let inputIndex = $state<number | null>(null);
  let debugSppfNodes = $state<DebugSPPFNode[]>([]);

  // Debug SPPF visualization
  let debugSppfContainer: HTMLElement;
  let debugSppfCy: cytoscape.Core | null = null;
  let currentSppfNodeId = $state<number | null>(null);

  // Debug GSS visualization
  let debugGssNodes = $state<DebugGSSNode[]>([]);
  let debugGssEdges = $state<DebugGSSEdge[]>([]);
  let currentGssNodeId = $state<number | null>(null);
  let debugGssContainer: HTMLElement;
  let debugGssCy: cytoscape.Core | null = null;

  // Graph tab
  let activeTab = $state<"gss" | "sppf">("sppf");

  // App mode
  let activeMode = $state<"parse" | "debug" | "design">("parse");

  // SPPF data
  let sppf = $state<SPPF | null>(null);
  let sppfContainer: HTMLDivElement;
  let collapsedNodes = $state<Set<string>>(new Set());

  // GSS data
  let gss = $state<GSS | null>(null);
  let gssContainer: HTMLDivElement;
  let gssCollapsedNodes = $state<Set<string>>(new Set());

  // Track if parse result is available
  let parseResultAvailable = $state(false);

  // Pop-out modal for graphs
  type PopoutGraph = 'sppf' | 'gss' | 'debugSppf' | 'debugGss' | null;
  let popoutGraph = $state<PopoutGraph>(null);
  let popoutContainer: HTMLDivElement;
  let popoutCy: cytoscape.Core | null = null;

  // Popout modal position for dragging
  let popoutX = $state(40);
  let popoutY = $state(40);
  let isDraggingPopout = $state(false);
  let popoutDragStartX = 0;
  let popoutDragStartY = 0;

  // Output panel state
  let outputPanelOpen = $state(false);
  let outputPanelHeight = $state(150);

  // Output log entries: each entry has a type and content
  type LogEntry = { type: "command" | "output" | "error"; content: string };
  let outputLog = $state<LogEntry[]>([]);
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

  // ============ Shared Graph Styles ============
  const sppfNodeStyles: cytoscape.Stylesheet[] = [
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

  const gssNodeStyles: cytoscape.Stylesheet[] = [
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

  const edgeStyles: cytoscape.Stylesheet = {
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

  const gssEdgeStyles: cytoscape.Stylesheet = {
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

  // Helper to create a Cytoscape graph with common options
  interface GraphOptions {
    container: HTMLElement;
    elements: cytoscape.ElementDefinition[];
    styles: cytoscape.Stylesheet[];
    layout?: 'sppf' | 'gss';
  }

  // Cap zoom level after fit to prevent huge nodes on small graphs
  const MAX_FIT_ZOOM = 1.5;

  function capZoom(cyInstance: cytoscape.Core) {
    if (cyInstance.zoom() > MAX_FIT_ZOOM) {
      cyInstance.zoom(MAX_FIT_ZOOM);
      cyInstance.center();
    }
  }

  function createGraph(options: GraphOptions): cytoscape.Core {
    const { container, elements, styles, layout = 'sppf' } = options;

    const cyInstance = cytoscape({
      container,
      elements,
      style: styles,
      layout: {
        name: "dagre",
        rankDir: layout === 'gss' ? "BT" : "TB",
        nodeSep: layout === 'gss' ? 50 : 30,
        rankSep: layout === 'gss' ? 60 : 50,
      } as any,
      userZoomingEnabled: true,
      userPanningEnabled: true,
      boxSelectionEnabled: false,
    });

    // Cap initial zoom after layout
    capZoom(cyInstance);

    return cyInstance;
  }

  // ============ End Shared Graph Styles ============

  // Find the root node (node with no incoming edges)
  function findRoot(): string | null {
    if (!cy) return null;
    const roots = cy.nodes().filter((node: cytoscape.NodeSingular) => node.incomers('edge').length === 0);
    return roots.length > 0 ? roots.first().id() : null;
  }

  // Get all nodes reachable from root, respecting collapsed nodes (their outgoing edges are "cut")
  function getReachableNodes(): Set<string> {
    if (!cy) return new Set();
    const reachable = new Set<string>();
    const root = findRoot();
    if (!root) return reachable;

    const queue = [root];
    while (queue.length > 0) {
      const nodeId = queue.shift()!;
      if (reachable.has(nodeId)) continue;
      reachable.add(nodeId);

      // If this node is collapsed, don't traverse its children
      if (collapsedNodes.has(nodeId)) continue;

      const node = cy.getElementById(nodeId);
      node.outgoers('node').forEach((child: cytoscape.NodeSingular) => {
        if (!reachable.has(child.id())) {
          queue.push(child.id());
        }
      });
    }

    return reachable;
  }

  // Update visibility based on reachability
  function updateVisibility() {
    if (!cy) return;

    const reachable = getReachableNodes();

    cy.nodes().forEach((node: cytoscape.NodeSingular) => {
      if (reachable.has(node.id())) {
        node.style('display', 'element');
      } else {
        node.style('display', 'none');
      }
    });

    cy.edges().forEach((edge: cytoscape.EdgeSingular) => {
      const sourceId = edge.source().id();
      const targetId = edge.target().id();
      // Show edge only if both endpoints are visible AND source is not collapsed
      if (reachable.has(sourceId) && reachable.has(targetId) && !collapsedNodes.has(sourceId)) {
        edge.style('display', 'element');
      } else {
        edge.style('display', 'none');
      }
    });
  }

  // Toggle collapse/expand a node
  function toggleCollapse(nodeId: string) {
    if (!cy) return;

    const node = cy.getElementById(nodeId);

    // Check if node has children
    if (node.outgoers('node').length === 0) return;

    const isCollapsed = collapsedNodes.has(nodeId);

    if (isCollapsed) {
      collapsedNodes.delete(nodeId);
      node.removeClass('collapsed');
    } else {
      collapsedNodes.add(nodeId);
      node.addClass('collapsed');
    }

    // Trigger reactivity
    collapsedNodes = new Set(collapsedNodes);

    // Update all visibility based on new reachability
    updateVisibility();
  }

  function renderSPPF() {
    if (!sppf || !sppfContainer) return;

    // Reset collapsed nodes when rendering new SPPF
    collapsedNodes = new Set();

    const elements: cytoscape.ElementDefinition[] = [
      ...sppf.nodes.map((node) => ({
        data: {
          id: `n${node.id}`,
          label: node.label || (node.kind === "Packed" ? "●" : ""),
        },
        classes: node.kind.toLowerCase(),
      })),
      ...sppf.edges.map((edge, i) => ({
        data: {
          id: `e${i}`,
          source: `n${edge.src}`,
          target: `n${edge.dest}`,
        },
      })),
    ];

    if (cy) {
      cy.destroy();
    }

    cy = createGraph({
      container: sppfContainer,
      elements,
      styles: [...sppfNodeStyles, edgeStyles],
      layout: 'sppf',
    });

    // Add double-click handler for collapse/expand
    cy.on('dbltap', 'node', (event) => {
      const node = event.target;
      toggleCollapse(node.id());
    });
  }

  function renderGSS() {
    if (!gss || !gssContainer) return;

    // Reset collapsed nodes when rendering new GSS
    gssCollapsedNodes = new Set();

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

    if (gssCy) {
      gssCy.destroy();
    }

    gssCy = createGraph({
      container: gssContainer,
      elements,
      styles: [...gssNodeStyles, gssEdgeStyles],
      layout: 'gss',
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
    }
  });

  // Fetch data when switching tabs
  $effect(() => {
    if (parseResultAvailable) {
      if (activeTab === "sppf" && !sppf) {
        fetchSppf();
      } else if (activeTab === "gss" && !gss) {
        fetchGss();
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

  function renderDebugSppf() {
    if (!debugSppfContainer) return;

    const elements: cytoscape.ElementDefinition[] = [];

    // Build a map for quick lookup
    const nodeMap = new Map<number, typeof debugSppfNodes[0]>();
    for (const node of debugSppfNodes) {
      nodeMap.set(node.id, node);
    }

    // Find all nodes reachable from current descriptor's node (the subtree to show)
    const reachableIds = new Set<number>();
    if (currentSppfNodeId !== null && nodeMap.has(currentSppfNodeId)) {
      const queue = [currentSppfNodeId];
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

    // If no current node or no reachable nodes, show nothing
    if (reachableIds.size === 0) {
      if (debugSppfCy) {
        debugSppfCy.destroy();
        debugSppfCy = null;
      }
      return;
    }

    // Add only nodes in the current subtree
    for (const node of debugSppfNodes) {
      if (!reachableIds.has(node.id)) continue;

      const label = `(${node.label}, ${node.left_extent}, ${node.right_extent})`;

      elements.push({
        data: {
          id: `n${node.id}`,
          label,
          kind: node.kind,
        },
        classes: node.kind.toLowerCase(),
      });
    }

    // Add edges only within the subtree
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

    if (debugSppfCy) {
      debugSppfCy.destroy();
    }

    debugSppfCy = createGraph({
      container: debugSppfContainer,
      elements,
      styles: [...sppfNodeStyles, edgeStyles],
      layout: 'sppf',
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

    if (debugGssCy) {
      debugGssCy.destroy();
    }

    debugGssCy = createGraph({
      container: debugGssContainer,
      elements,
      styles: [...gssNodeStyles, gssEdgeStyles],
      layout: 'gss',
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
      parseResultAvailable = false;
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
    parseResultAvailable = false;

    logCommand(`${parserName} <input> --start ${startNonterminal}`);

    const result = await commands.parse(parserDirectory, inputText, startNonterminal!);
    if (result.status === "ok") {
      parseResultAvailable = true;
      logOutput("Parse successful");
      setStatus("Parse successful", "success");

      // Fetch the data for the active tab
      if (activeTab === "sppf") {
        await fetchSppf();
      } else {
        await fetchGss();
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

  // Graph controls (work with active graph)
  // Generic graph control functions
  function zoomInGraph(graph: cytoscape.Core | null) {
    if (graph) {
      graph.zoom(graph.zoom() * 1.2);
    }
  }

  function zoomOutGraph(graph: cytoscape.Core | null) {
    if (graph) {
      graph.zoom(graph.zoom() / 1.2);
    }
  }

  function resetViewGraph(graph: cytoscape.Core | null) {
    if (graph) {
      graph.fit();
      capZoom(graph);
    }
  }

  // Parse mode convenience functions
  function zoomIn() {
    zoomInGraph(activeTab === "sppf" ? cy : gssCy);
  }

  function zoomOut() {
    zoomOutGraph(activeTab === "sppf" ? cy : gssCy);
  }

  function resetView() {
    resetViewGraph(activeTab === "sppf" ? cy : gssCy);
  }

  function expandAll() {
    if (activeTab === "sppf") {
      if (!cy) return;
      cy.nodes().removeClass('collapsed');
      collapsedNodes = new Set();
      updateVisibility();
    } else {
      if (!gssCy) return;
      gssCy.nodes().removeClass('collapsed');
      gssCollapsedNodes = new Set();
    }
  }

  // Pop-out modal functions
  function handlePopoutKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closePopout();
    }
  }

  function openPopout(graphType: PopoutGraph) {
    popoutGraph = graphType;
    // Reset position for each open
    popoutX = 40;
    popoutY = 40;
    // Add escape key listener
    window.addEventListener('keydown', handlePopoutKeydown);
  }

  function closePopout() {
    if (popoutCy) {
      popoutCy.destroy();
      popoutCy = null;
    }
    popoutGraph = null;
    window.removeEventListener('keydown', handlePopoutKeydown);
  }

  function startPopoutDrag(e: MouseEvent) {
    isDraggingPopout = true;
    popoutDragStartX = e.clientX - popoutX;
    popoutDragStartY = e.clientY - popoutY;
    window.addEventListener('mousemove', handlePopoutDrag);
    window.addEventListener('mouseup', stopPopoutDrag);
  }

  function handlePopoutDrag(e: MouseEvent) {
    if (!isDraggingPopout) return;
    popoutX = e.clientX - popoutDragStartX;
    popoutY = e.clientY - popoutDragStartY;
  }

  function stopPopoutDrag() {
    isDraggingPopout = false;
    window.removeEventListener('mousemove', handlePopoutDrag);
    window.removeEventListener('mouseup', stopPopoutDrag);
  }

  function getPopoutElements(): cytoscape.ElementDefinition[] {
    if (popoutGraph === 'sppf' && sppf) {
      return [
        ...sppf.nodes.map((node) => ({
          data: {
            id: `n${node.id}`,
            label: node.label || (node.kind === "Packed" ? "●" : ""),
          },
          classes: node.kind.toLowerCase(),
        })),
        ...sppf.edges.map((edge, i) => ({
          data: {
            id: `e${i}`,
            source: `n${edge.src}`,
            target: `n${edge.dest}`,
          },
        })),
      ];
    } else if (popoutGraph === 'gss' && gss) {
      return [
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
    } else if (popoutGraph === 'debugSppf') {
      const elements: cytoscape.ElementDefinition[] = [];
      const nodeMap = new Map<number, typeof debugSppfNodes[0]>();
      for (const node of debugSppfNodes) {
        nodeMap.set(node.id, node);
      }
      const reachableIds = new Set<number>();
      if (currentSppfNodeId !== null && nodeMap.has(currentSppfNodeId)) {
        const queue = [currentSppfNodeId];
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
    } else if (popoutGraph === 'debugGss') {
      const elements: cytoscape.ElementDefinition[] = [];
      for (const node of debugGssNodes) {
        elements.push({
          data: {
            id: `n${node.id}`,
            label: node.label,
          },
          classes: currentGssNodeId === node.id ? 'current' : '',
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

  function renderPopout() {
    if (!popoutContainer || !popoutGraph) return;

    const elements = getPopoutElements();
    if (elements.length === 0) return;

    if (popoutCy) {
      popoutCy.destroy();
    }

    const isGss = popoutGraph === 'gss' || popoutGraph === 'debugGss';
    popoutCy = createGraph({
      container: popoutContainer,
      elements,
      styles: isGss ? [...gssNodeStyles, gssEdgeStyles] : [...sppfNodeStyles, edgeStyles],
      layout: isGss ? 'gss' : 'sppf',
    });
  }

  // Effect to render popout when container becomes available
  $effect(() => {
    if (popoutGraph && popoutContainer) {
      tick().then(() => renderPopout());
    }
  });

  function stopDebug() {
    debugLoaded = false;
    currentStep = 0;
    totalSteps = 0;
    currentAction = null;
    descriptorSet = [];
    callStack = [];
    inputIndex = null;
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
    const result = await commands.debugStepTo(currentStep - 1);
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
    }
  }

  async function stepForward() {
    if (!debugLoaded || currentStep >= totalSteps - 1) return;
    const result = await commands.debugStepForward();
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
    }
  }

  async function stepTo(target: number) {
    if (!debugLoaded) return;
    const result = await commands.debugStepTo(target);
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
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

  // Store pre-maximize bounds for custom maximize behavior
  let savedBounds: { x: number; y: number; width: number; height: number } | null = null;
  let isCustomMaximized = false;
  let isAnimating = false;

  // Easing function for smooth animation (ease-out quint - even smoother)
  function easeOutQuint(t: number): number {
    return 1 - Math.pow(1 - t, 5);
  }

  async function animateWindow(
    from: { x: number; y: number; width: number; height: number },
    to: { x: number; y: number; width: number; height: number },
    duration: number = 500
  ) {
    const window = getCurrentWindow();
    const startTime = performance.now();

    return new Promise<void>((resolve) => {
      function step(currentTime: number) {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / duration, 1);
        const eased = easeOutQuint(progress);

        const currentX = Math.round(from.x + (to.x - from.x) * eased);
        const currentY = Math.round(from.y + (to.y - from.y) * eased);
        const currentWidth = Math.round(from.width + (to.width - from.width) * eased);
        const currentHeight = Math.round(from.height + (to.height - from.height) * eased);

        window.setPosition({ type: "Physical", x: currentX, y: currentY });
        window.setSize({ type: "Physical", width: currentWidth, height: currentHeight });

        if (progress < 1) {
          requestAnimationFrame(step);
        } else {
          resolve();
        }
      }

      requestAnimationFrame(step);
    });
  }

  function handleKeyDown(e: KeyboardEvent) {
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

  async function toggleMaximize() {
    if (isAnimating) return;

    const window = getCurrentWindow();
    const monitor = await currentMonitor();

    if (!monitor) return;

    isAnimating = true;

    const pos = await window.outerPosition();
    const size = await window.outerSize();
    const currentBounds = { x: pos.x, y: pos.y, width: size.width, height: size.height };

    if (isCustomMaximized && savedBounds) {
      // Animate to saved bounds
      await animateWindow(currentBounds, savedBounds);
      isCustomMaximized = false;
      savedBounds = null;
    } else {
      // Save current bounds
      savedBounds = currentBounds;

      // Animate to monitor bounds
      const { position, size: monitorSize } = monitor;
      const targetBounds = { x: position.x, y: position.y, width: monitorSize.width, height: monitorSize.height };
      await animateWindow(currentBounds, targetBounds);
      isCustomMaximized = true;
    }

    isAnimating = false;
  }
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} onclick={handleWindowClick} onkeydown={handleKeyDown} />

<div class="app" class:dragging={isDraggingVertical || isDraggingHorizontal || isDraggingInput || isDraggingCurrent || isDraggingOutput || isDraggingDebug1 || isDraggingDebug2 || isDraggingDebugAction || isDraggingDebugStack || isDraggingDebugGraph} class:dragging-horizontal={isDraggingHorizontal || isDraggingInput || isDraggingCurrent || isDraggingOutput || isDraggingDebugAction || isDraggingDebugStack || isDraggingDebugGraph}>
  <!-- Title Bar (full width) -->
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
      <textarea
        bind:value={inputText}
        placeholder="Enter code to parse..."
        spellcheck="false"
      ></textarea>
    </div>
  </div>

  <!-- Vertical Resize Handle -->
  <div class="resize-handle-vertical" onmousedown={startVerticalDrag}></div>

  <!-- Right Panel -->
  <div class="right-panel">
    <!-- Graph Tabs -->
    <div class="graph-section">
      <div class="tabs">
        <button
          class:active={activeTab === "gss"}
          onclick={() => activeTab = "gss"}
        >GSS</button>
        <button
          class:active={activeTab === "sppf"}
          onclick={() => activeTab = "sppf"}
        >SPPF</button>
      </div>
      <div class="graph-container">
        {#if activeTab === "gss"}
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
              <button onclick={() => openPopout('gss')} title="Pop out">
                <Fullscreen size={16} />
              </button>
            </div>
          {:else}
            <div class="graph-placeholder">Parse input to see GSS</div>
          {/if}
        {:else if sppf}
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
            <button onclick={() => openPopout('sppf')} title="Pop out">
              <Fullscreen size={16} />
            </button>
          </div>
        {:else}
          <div class="graph-placeholder">Parse input to see SPPF</div>
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
          <div class="input-viewer">{#each inputText.split('') as char, i}<span class="input-char" class:current={i === inputIndex} class:consumed={inputIndex !== null && i < inputIndex}>{char}</span>{/each}{#if inputIndex !== null && inputIndex >= inputText.length}<span class="input-char current">&nbsp;</span>{/if}</div>
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
      <div class="current-action-box">
        {#if currentAction}
          <pre>{currentAction}</pre>
        {:else}
          <span class="placeholder">No action</span>
        {/if}
      </div>

      <!-- Resize Handle (between action and stack) -->
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
              <button onclick={() => zoomInGraph(debugSppfCy)} title="Zoom in">
                <ZoomIn size={16} />
              </button>
              <button onclick={() => zoomOutGraph(debugSppfCy)} title="Zoom out">
                <ZoomOut size={16} />
              </button>
              <button onclick={() => resetViewGraph(debugSppfCy)} title="Reset view">
                <Maximize2 size={16} />
              </button>
              <button onclick={() => openPopout('debugSppf')} title="Pop out">
                <Fullscreen size={16} />
              </button>
            </div>
          {/if}
        </div>
      </div>

      <!-- Resize Handle -->
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
              <button onclick={() => zoomInGraph(debugGssCy)} title="Zoom in">
                <ZoomIn size={16} />
              </button>
              <button onclick={() => zoomOutGraph(debugGssCy)} title="Zoom out">
                <ZoomOut size={16} />
              </button>
              <button onclick={() => resetViewGraph(debugGssCy)} title="Reset view">
                <Maximize2 size={16} />
              </button>
              <button onclick={() => openPopout('debugGss')} title="Pop out">
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
        <div class="output-content" style="height: {outputPanelHeight}px" bind:this={outputContentEl}>
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
    <div class="modal-overlay" onclick={closeErrorModal}>
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

  <!-- Pop-out modal for graphs -->
  {#if popoutGraph}
    <div class="popout-overlay" onclick={closePopout}>
      <div class="popout-modal" style="left: {popoutX}px; top: {popoutY}px; right: auto; bottom: auto;" onclick={(e) => e.stopPropagation()}>
        <div class="popout-header" onmousedown={startPopoutDrag}>
          <span class="popout-title">
            {#if popoutGraph === 'sppf'}SPPF{:else if popoutGraph === 'gss'}GSS{:else if popoutGraph === 'debugSppf'}SPPF{:else if popoutGraph === 'debugGss'}GSS{/if}
          </span>
          <div class="graph-controls popout-controls">
            <button title="Zoom In" onclick={(e) => { e.stopPropagation(); popoutCy?.zoom(popoutCy.zoom() * 1.2); }}>
              <ZoomIn size={16} />
            </button>
            <button title="Zoom Out" onclick={(e) => { e.stopPropagation(); popoutCy?.zoom(popoutCy.zoom() / 1.2); }}>
              <ZoomOut size={16} />
            </button>
            <button title="Reset View" onclick={(e) => { e.stopPropagation(); if (popoutCy) { popoutCy.fit(); capZoom(popoutCy); } }}>
              <Maximize2 size={16} />
            </button>
            <button title="Close" onclick={(e) => { e.stopPropagation(); closePopout(); }}>
              <X size={16} />
            </button>
          </div>
        </div>
        <div class="popout-container" bind:this={popoutContainer}></div>
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

  .mode-placeholder h2 {
    margin: 0;
    font-weight: 500;
  }

  .mode-placeholder p {
    margin: 0;
    font-size: 14px;
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
  }

  .debug-column {
    display: flex;
    flex-direction: column;
    background: #252526;
    border-right: 1px solid #3c3c3c;
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
  }

  .debug-col-stack .pending-descriptors {
    min-height: 0;  /* Allow grid to control size */
    overflow: auto;
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

  .section-content li.current {
    color: #4ec9b0;
  }

  .section-content li.current::before {
    content: "● ";
    color: #4ec9b0;
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

  .status-left .status-text {
    margin-left: 8px;
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-bar .status-text {
    font-size: 12px;
    color: #888;
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

  /* Pop-out modal for graphs */
  .popout-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    z-index: 1000;
  }

  .popout-modal {
    position: absolute;
    background: #1e1e1e;
    border: 1px solid #3c3c3c;
    border-radius: 8px;
    width: calc(100% - 80px);
    height: calc(100% - 80px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .popout-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
    cursor: move;
    user-select: none;
  }

  .popout-title {
    font-size: 14px;
    font-weight: 500;
    color: #d4d4d4;
  }

  .popout-controls {
    position: static;
  }

  .popout-container {
    flex: 1;
    min-height: 0;
  }
</style>
