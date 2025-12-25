<script lang="ts">
  import { commands, type SPPF } from "../bindings";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { FolderOpen, Hammer, X, AlertTriangle, CheckCircle, Loader2 } from "lucide-svelte";
  import cytoscape from "cytoscape";
  import dagre from "cytoscape-dagre";

  cytoscape.use(dagre);

  // Event listeners for build progress
  onMount(() => {
    const unlistenProgress = listen<{ stage: string; message: string }>("build-progress", (event) => {
      setStatus(event.payload.message, "info");
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
        setStatus("Build successful", "success");
      } else {
        buildStatus = "error";
        buildError = event.payload.message;
        setStatus("Build failed", "error");
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
  let traceEnabled = $state(true);
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
  let activeTab = $state<"gss" | "sppf">("gss");

  // SPPF data
  let sppf = $state<SPPF | null>(null);
  let sppfContainer: HTMLDivElement;
  let cy: cytoscape.Core | null = null;

  function renderSPPF() {
    if (!sppf || !sppfContainer) return;

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
  }

  $effect(() => {
    if (sppf && sppfContainer) {
      renderSPPF();
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
      statusMessage = null;
      sppf = null;
      // Try to get parser name (might not exist yet if empty directory)
      const result = await commands.getParserName(parserDirectory);
      if (result.status === "ok") {
        parserName = result.data;
        setStatus(`Loaded parser: ${parserName}`, "success");
      } else {
        // No existing parser - that's fine, user can generate one
        parserName = null;
        setStatus("Ready to generate parser", "success");
      }
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
    const result = await commands.parse(parserDirectory, inputText);
    if (result.status === "ok") {
      sppf = result.data;
      setStatus("Parse successful", "success");
    } else {
      sppf = null;
      setStatus("Parse failed", "error");
      buildError = result.error;
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
  }

  function onMouseUp() {
    isDraggingVertical = false;
    isDraggingHorizontal = false;
    isDraggingInput = false;
    isDraggingCurrent = false;
  }
</script>

<svelte:window on:mousemove={onMouseMove} on:mouseup={onMouseUp} />

<div class="app" class:dragging={isDraggingVertical || isDraggingHorizontal || isDraggingInput || isDraggingCurrent}>
  <!-- Parser Directory Bar -->
  <div class="parser-bar">
    <span class="parser-label">Parser:</span>
    <span class="parser-path" class:placeholder={!parserDirectory}>
      {#if parserName}
        {parserName}
        <span class="parser-dir">({parserDirectory})</span>
      {:else if parserDirectory}
        {parserDirectory}
      {:else}
        No parser selected
      {/if}
    </span>
    <button class="icon-btn" onclick={selectDirectory} title="Select parser directory">
      <FolderOpen size={18} color="#e0e0e0" />
    </button>
    <button
      class="build-btn"
      onclick={buildParser}
      disabled={!parserDirectory || isBuilding}
      class:success={buildStatus === "success"}
      class:error={buildStatus === "error"}
    >
      {#if isBuilding}
        <Loader2 size={14} class="spinning" />
        Building...
      {:else}
        <Hammer size={14} />
        {#if buildStatus === "success"}
          Built
        {:else if buildStatus === "error"}
          Rebuild
        {:else}
          Build
        {/if}
      {/if}
    </button>
  </div>

  <!-- Main Content -->
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
          <div class="graph-placeholder">GSS Graph</div>
        {:else if sppf}
          <div class="cytoscape-container" bind:this={sppfContainer}></div>
        {:else}
          <div class="graph-placeholder">Parse input to see SPPF</div>
        {/if}
      </div>
    </div>

    <!-- Horizontal Resize Handle -->
    <div class="resize-handle-horizontal" onmousedown={startHorizontalDrag}></div>

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
  </div>

  <!-- Status Bar (always visible) -->
  <div class="status-bar" class:error={statusMessage && statusType === "error"} class:success={statusMessage && statusType === "success"}>
    <div class="status-content">
      {#if statusMessage}
        {#if statusType === "error"}
          <AlertTriangle size={14} />
        {:else if statusType === "success"}
          <CheckCircle size={14} />
        {:else}
          <Loader2 size={14} class="spinning" />
        {/if}
        <span class="status-text">{statusMessage}</span>
        {#if buildError && statusType === "error"}
          <button class="status-details-btn" onclick={() => showStatusDetails = !showStatusDetails}>
            {showStatusDetails ? "Hide" : "Details"}
          </button>
        {/if}
        {#if statusType === "error"}
          <button class="status-close" onclick={clearStatus}>
            <X size={14} />
          </button>
        {/if}
      {:else}
        <span class="status-text placeholder">Ready</span>
      {/if}
    </div>
    {#if showStatusDetails && buildError}
      <div class="status-details">
        <pre>{buildError}</pre>
      </div>
    {/if}
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
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 14px;
    background: #1e1e1e;
    color: #d4d4d4;
  }

  /* Parser Directory Bar */
  .parser-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .parser-label {
    font-weight: 600;
    color: #888;
  }

  .parser-path {
    flex: 1;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .parser-path.placeholder {
    color: #666;
    font-style: italic;
  }

  .parser-dir {
    color: #666;
    font-size: 11px;
    margin-left: 8px;
  }

  .icon-btn {
    padding: 4px 8px;
    background: #3c3c3c;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
  }

  .icon-btn:hover {
    background: #4c4c4c;
  }

  .build-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: #3c3c3c;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    min-width: 80px;
  }

  .build-btn:hover:not(:disabled) {
    background: #4c4c4c;
  }

  .build-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .build-btn.success {
    background: #2d5a2d;
    border-color: #3d7a3d;
  }

  .build-btn.error {
    background: #5a2d2d;
    border-color: #7a3d3d;
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

  .parse-btn:hover {
    background: #1177bb;
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
  }

  /* Graph Section */
  .graph-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
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
  }

  .graph-placeholder {
    color: #555;
    font-size: 24px;
  }

  .cytoscape-container {
    width: 100%;
    height: 100%;
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

  /* Status Bar */
  .status-bar {
    display: flex;
    flex-direction: column;
    background: #2d2d2d;
    border-top: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .status-bar > .status-content {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
  }

  .status-bar.error {
    background: #3d2020;
    border-top-color: #5a3030;
  }

  .status-bar.success {
    background: #203d20;
    border-top-color: #305a30;
  }

  .status-text {
    flex: 1;
    font-size: 13px;
  }

  .status-text.placeholder {
    color: #666;
  }

  .status-details-btn {
    padding: 2px 8px;
    background: rgba(255, 255, 255, 0.1);
    color: #d4d4d4;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    font-size: 12px;
  }

  .status-details-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .status-close {
    padding: 4px;
    background: transparent;
    color: #888;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .status-close:hover {
    color: #d4d4d4;
  }

  .status-details {
    padding: 8px 12px;
    background: rgba(0, 0, 0, 0.2);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    max-height: 200px;
    overflow: auto;
  }

  .status-details pre {
    margin: 0;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
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
