<script lang="ts">
  import { commands, type SPPF, type GSS } from "../bindings";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
  import { availableMonitors, currentMonitor } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";
  import { FolderOpen, Hammer, X, AlertTriangle, CheckCircle, Loader2, ChevronDown, ChevronRight, ZoomIn, ZoomOut, Maximize2, Expand, GitFork, Bug, Braces } from "lucide-svelte";
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
      if (event.payload.success) {
        buildStatus = "success";
        // Refresh parser name after successful build
        if (parserDirectory) {
          const result = await commands.getParserName(parserDirectory);
          if (result.status === "ok") {
            parserName = result.data;
          }
        }
        // Show "Ready" status briefly
        showReadyStatus = true;
        if (readyStatusTimeout) clearTimeout(readyStatusTimeout);
        readyStatusTimeout = setTimeout(() => {
          showReadyStatus = false;
        }, 3000);
      } else {
        buildStatus = "error";
        buildError = event.payload.message;
      }
    });

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenResult.then(fn => fn());
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
  let inputText = $state("1 + 2 * 3");
  let startNonterminal = $state("Expr");
  let traceEnabled = $state(false);
  let nonterminals = $state(["Expr", "Term", "Factor"]); // TODO: load from grammar

  // Playback state
  let currentStep = $state(0);
  let totalSteps = $state(0);
  let isPlaying = $state(false);

  // Parser state
  let currentDescriptor = $state<string | null>(null);
  let descriptorSet = $state<string[]>([]);
  let callStack = $state<string[]>([]);

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

  // Output panel state
  let outputPanelOpen = $state(false);
  let outputContent = $state<string | null>(null);
  let outputType = $state<"success" | "error" | "info">("info");
  let outputPanelHeight = $state(150);
  let cy: cytoscape.Core | null = null;
  let gssCy: cytoscape.Core | null = null;

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

    cy = cytoscape({
      container: sppfContainer,
      elements,
      style: [
        {
          selector: "node",
          style: {
            label: "data(label)",
            "text-valign": "center",
            "text-halign": "center",
            "font-size": "10px",
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
          selector: "node.nonterminal",
          style: {
            "background-color": "#2d4a3d",
            "border-color": "#4ec9b0",
          },
        },
        {
          selector: "node.intermediate",
          style: {
            "background-color": "#2d3a4d",
            "border-color": "#569cd6",
            shape: "rectangle",
          },
        },
        {
          selector: "node.terminal",
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
        {
          selector: "edge",
          style: {
            width: 1,
            "line-color": "#555",
            "target-arrow-color": "#555",
            "target-arrow-shape": "triangle",
            "curve-style": "bezier",
            "arrow-scale": 0.8,
          },
        },
      ],
      layout: {
        name: "dagre",
        rankDir: "TB",
        nodeSep: 30,
        rankSep: 50,
      } as any,
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

    gssCy = cytoscape({
      container: gssContainer,
      elements,
      style: [
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
        },
      ],
      layout: {
        name: "dagre",
        rankDir: "BT",  // Bottom to top for GSS
        nodeSep: 50,
        rankSep: 60,
      } as any,
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

  // Resizable panes
  let leftPanelWidth = $state(350);
  let callStackHeight = $state(200);
  let inputHeight = $state(200);
  let currentDescHeight = $state(80);
  let isDraggingVertical = $state(false);
  let isDraggingHorizontal = $state(false);
  let isDraggingInput = $state(false);
  let isDraggingCurrent = $state(false);
  let isDraggingOutput = $state(false);

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

      // Try to get parser name (might not exist yet if empty directory)
      const result = await commands.getParserName(parserDirectory);
      if (result.status === "ok") {
        parserName = result.data;
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
    setStatus("Starting build...", "info");
    // Command returns immediately, results come via events
    await commands.buildParser(parserDirectory);
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
    outputContent = "Parsing...";
    outputType = "info";

    // Reset previous results
    sppf = null;
    gss = null;
    parseResultAvailable = false;

    const result = await commands.parse(parserDirectory, inputText);
    if (result.status === "ok") {
      parseResultAvailable = true;
      outputContent = "Parse successful";
      outputType = "success";
      setStatus("Parse successful", "success");

      // Fetch the data for the active tab
      if (activeTab === "sppf") {
        await fetchSppf();
      } else {
        await fetchGss();
      }
    } else {
      parseResultAvailable = false;
      outputContent = `Parse failed\n\n${result.error}`;
      outputType = "error";
      outputPanelOpen = true;  // Only auto-open on error
      setStatus("Parse failed", "error");
    }
  }

  async function fetchSppf() {
    if (!parseResultAvailable) return;
    const result = await commands.getSppf();
    if (result.status === "ok") {
      sppf = result.data;
      outputContent = `Parse successful\n\nSPPF: ${result.data.nodes.length} nodes, ${result.data.edges.length} edges`;
    } else {
      outputContent = `Failed to load SPPF: ${result.error}`;
      outputType = "error";
    }
  }

  async function fetchGss() {
    if (!parseResultAvailable) return;
    const result = await commands.getGss();
    if (result.status === "ok") {
      gss = result.data;
      outputContent = `Parse successful\n\nGSS: ${result.data.nodes.length} nodes, ${result.data.edges.length} edges`;
    } else {
      outputContent = `Failed to load GSS: ${result.error}`;
      outputType = "error";
    }
  }

  // Graph controls (work with active graph)
  function zoomIn() {
    const activeCy = activeTab === "sppf" ? cy : gssCy;
    if (activeCy) {
      activeCy.zoom(activeCy.zoom() * 1.2);
    }
  }

  function zoomOut() {
    const activeCy = activeTab === "sppf" ? cy : gssCy;
    if (activeCy) {
      activeCy.zoom(activeCy.zoom() / 1.2);
    }
  }

  function resetView() {
    const activeCy = activeTab === "sppf" ? cy : gssCy;
    if (activeCy) {
      activeCy.fit();
      activeCy.center();
    }
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

  function stepBack() {
    if (currentStep > 0) currentStep--;
  }

  function stepForward() {
    if (currentStep < totalSteps) currentStep++;
  }

  function togglePlay() {
    isPlaying = !isPlaying;
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
  }

  function onMouseUp() {
    isDraggingVertical = false;
    isDraggingHorizontal = false;
    isDraggingInput = false;
    isDraggingCurrent = false;
    isDraggingOutput = false;
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

<svelte:window on:mousemove={onMouseMove} on:mouseup={onMouseUp} />

<div class="app" class:dragging={isDraggingVertical || isDraggingHorizontal || isDraggingInput || isDraggingCurrent || isDraggingOutput}>
  <!-- Activity Bar -->
  <div class="activity-bar">
    <div class="activity-bar-spacer" onmousedown={startWindowDrag}></div>
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
  <!-- Title Bar -->
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

  <!-- Mode Content -->
  {#if activeMode === "parse"}
  <!-- Parse Mode -->
  <div class="main-content">
    <!-- Left Panel -->
    <div class="left-panel" style="width: {leftPanelWidth}px">
      <!-- Header -->
      <div class="header">
        <label>
          Start:
          <select bind:value={startNonterminal} disabled={!parserDirectory}>
            {#each nonterminals as nt}
              <option value={nt}>{nt}</option>
            {/each}
          </select>
        </label>
        <label class="trace-checkbox">
          <input type="checkbox" bind:checked={traceEnabled} disabled={!parserDirectory} />
          Trace
        </label>
        <button class="parse-btn" onclick={parse} disabled={!parserDirectory || buildStatus !== "success"}>Parse</button>
      </div>

    <!-- Input Area -->
    <div class="input-section" style="flex: 0 0 {inputHeight}px">
      <textarea
        bind:value={inputText}
        placeholder="Enter code to parse..."
        spellcheck="false"
      ></textarea>
    </div>

    <!-- Input Resize Handle -->
    <div class="resize-handle-horizontal" onmousedown={startInputDrag}></div>

    <!-- Playback Controls -->
    <div class="playback-controls">
      <button onclick={stepBack} disabled={currentStep === 0}>◀</button>
      <button onclick={togglePlay}>{isPlaying ? "⏸" : "▶"}</button>
      <button onclick={stepForward} disabled={currentStep === totalSteps}>▶▶</button>
      <span class="step-counter">Step {currentStep}/{totalSteps}</span>
      <input
        type="range"
        min="0"
        max={totalSteps}
        bind:value={currentStep}
        class="step-slider"
      />
    </div>

    <!-- Current Descriptor -->
    <div class="section current-section" style="flex: 0 0 {currentDescHeight}px">
      <div class="section-header">Current</div>
      <div class="section-content current-descriptor">
        {#if currentDescriptor}
          <code>{currentDescriptor}</code>
        {:else}
          <span class="placeholder">No descriptor</span>
        {/if}
      </div>
    </div>

    <!-- Current Resize Handle -->
    <div class="resize-handle-horizontal" onmousedown={startCurrentDrag}></div>

    <!-- Descriptor Set -->
    <div class="section descriptor-set">
      <div class="section-header">Descriptor Set</div>
      <div class="section-content">
        {#if descriptorSet.length > 0}
          <ul>
            {#each descriptorSet as desc, i}
              <li class:current={i === 0}><code>{desc}</code></li>
            {/each}
          </ul>
        {:else}
          <span class="placeholder">Empty</span>
        {/if}
      </div>
    </div>

    <!-- Call Stack -->
    <div class="section call-stack" style="flex: 0 0 {callStackHeight}px">
      <div class="section-header">Call Stack</div>
      <div class="section-content">
        {#if callStack.length > 0}
          <ul>
            {#each callStack as call, i}
              <li style="padding-left: {i * 16}px">
                <span class="call-marker">{i === callStack.length - 1 ? "●" : "▼"}</span>
                <code>{call}</code>
              </li>
            {/each}
          </ul>
        {:else}
          <span class="placeholder">Empty</span>
        {/if}
      </div>
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
          </div>
        {:else}
          <div class="graph-placeholder">Parse input to see SPPF</div>
        {/if}
      </div>
    </div>
  </div>
  </div>

  <!-- Output Panel (collapsible) -->
  {#if outputPanelOpen}
    <div class="resize-handle-horizontal" onmousedown={startOutputDrag}></div>
  {/if}
  <div class="output-panel" class:open={outputPanelOpen}>
    <button class="output-header" onclick={() => outputPanelOpen = !outputPanelOpen}>
      {#if outputPanelOpen}
        <ChevronDown size={14} />
      {:else}
        <ChevronRight size={14} />
      {/if}
      <span>Output</span>
      {#if outputType === "error"}
        <AlertTriangle size={14} class="output-status-icon error" />
      {:else if outputType === "success"}
        <CheckCircle size={14} class="output-status-icon success" />
      {/if}
    </button>
    {#if outputPanelOpen}
      <div class="output-content" class:error={outputType === "error"} class:success={outputType === "success"} style="height: {outputPanelHeight}px">
        {#if outputContent}
          <pre>{outputContent}</pre>
        {:else}
          <span class="placeholder">No output</span>
        {/if}
      </div>
    {/if}
  </div>
  {:else if activeMode === "debug"}
  <!-- Debug Mode -->
  <div class="mode-placeholder">
    <Bug size={48} />
    <h2>Debug Mode</h2>
    <p>Trace visualization coming soon</p>
  </div>
  {:else if activeMode === "design"}
  <!-- Design Mode -->
  <div class="mode-placeholder">
    <Braces size={48} />
    <h2>Design Mode</h2>
    <p>Grammar editor coming soon</p>
  </div>
  {/if}

  <!-- Status Bar (always visible) -->
  <div class="status-bar">
    <div class="status-content">
      <span class="status-text">
        {#if isBuilding}
          Building...
        {:else if parserDirectory && buildStatus === "success"}
          Ready
        {:else}
          No parser selected
        {/if}
      </span>
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
  </div>
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
    display: flex;
    flex-direction: row;
    height: 100vh;
    width: 100vw;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 14px;
    background: #1e1e1e;
    color: #d4d4d4;
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

  .activity-bar-spacer {
    height: 52px; /* Match title bar height, space for macOS traffic lights */
    flex-shrink: 0;
    cursor: default;
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

  /* Title Bar */
  .title-bar {
    display: flex;
    align-items: center;
    height: 52px;
    background: #1e1e1e;
    border-bottom: 1px solid #3c3c3c;
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
    padding: 8px 16px;
    min-width: 300px;
    max-width: 550px;
    width: 100%;
    background: #2d2d2d;
    border: 1px solid #404040;
    border-radius: 8px;
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

  .header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-bottom: 1px solid #3c3c3c;
    background: #2d2d2d;
  }

  .header select {
    padding: 4px 8px;
    background: #3c3c3c;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
  }

  .trace-checkbox {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .parse-btn {
    margin-left: auto;
    padding: 6px 16px;
    background: #0e639c;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .parse-btn:hover:not(:disabled) {
    background: #1177bb;
  }

  .parse-btn:disabled {
    background: #3c3c3c;
    color: #888;
    cursor: not-allowed;
  }

  /* Input Section */
  .input-section {
    min-height: 100px;
    max-height: 400px;
    padding: 8px;
  }

  .input-section textarea {
    width: 100%;
    height: 100%;
    resize: none;
    background: #1e1e1e;
    color: #d4d4d4;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 8px;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 13px;
  }

  .input-section textarea:focus {
    outline: 1px solid #0e639c;
  }

  /* Playback Controls */
  .playback-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid #3c3c3c;
    background: #2d2d2d;
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

  .step-slider {
    flex: 1;
    height: 4px;
  }

  /* Sections */
  .section {
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid #3c3c3c;
  }

  .section-header {
    padding: 8px 12px;
    background: #2d2d2d;
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    color: #888;
  }

  .section-content {
    padding: 8px 12px;
    flex: 1;
    overflow-y: auto;
  }

  .current-section {
    min-height: 50px;
    max-height: 200px;
    display: flex;
    flex-direction: column;
  }

  .current-descriptor {
    font-family: "Fira Code", "Consolas", monospace;
  }

  .descriptor-set {
    flex: 1;
    min-height: 0;
  }

  .descriptor-set .section-content {
    max-height: none;
    flex: 1;
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

  .call-stack .section-content {
    max-height: none;
  }

  .call-stack li {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .call-marker {
    color: #888;
  }

  code {
    background: transparent;
  }

  /* Output Panel */
  .output-panel {
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    border-top: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .output-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: #2d2d2d;
    border: none;
    border-bottom: 1px solid #3c3c3c;
    color: #d4d4d4;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    text-align: left;
  }

  .output-header:hover {
    background: #383838;
  }

  .output-status-icon {
    margin-left: auto;
  }

  .output-status-icon.error {
    color: #f48771;
  }

  .output-status-icon.success {
    color: #89d185;
  }

  .output-content {
    padding: 12px;
    overflow: auto;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 12px;
  }

  .output-content pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .output-content.error {
    background: #2d1f1f;
    color: #f48771;
  }

  .output-content.success {
    background: #1f2d1f;
    color: #89d185;
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    background: #2d2d2d;
    border-top: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .status-bar > .status-content {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
  }

  .status-bar .status-text {
    font-size: 12px;
    color: #666;
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
