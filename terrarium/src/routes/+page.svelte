<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

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

  // Resizable panes
  let leftPanelWidth = $state(350);
  let callStackHeight = $state(200);
  let inputHeight = $state(200);
  let currentDescHeight = $state(80);
  let isDraggingVertical = $state(false);
  let isDraggingHorizontal = $state(false);
  let isDraggingInput = $state(false);
  let isDraggingCurrent = $state(false);

  async function parse() {
    // TODO: invoke Tauri command
    console.log("Parsing:", inputText, "from", startNonterminal, "trace:", traceEnabled);
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
  <!-- Left Panel -->
  <div class="left-panel" style="width: {leftPanelWidth}px">
    <!-- Header -->
    <div class="header">
      <label>
        Start:
        <select bind:value={startNonterminal}>
          {#each nonterminals as nt}
            <option value={nt}>{nt}</option>
          {/each}
        </select>
      </label>
      <label class="trace-checkbox">
        <input type="checkbox" bind:checked={traceEnabled} />
        Trace
      </label>
      <button class="parse-btn" onclick={parse}>Parse</button>
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
        {:else}
          <div class="graph-placeholder">SPPF Graph</div>
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
    height: 100vh;
    width: 100vw;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 14px;
    background: #1e1e1e;
    color: #d4d4d4;
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
</style>
