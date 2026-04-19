<script lang="ts">
  import { commands, type SPPF, type GSS, type DebugInfo, type DebugSPPFNode, type DebugSPPFInfo, type DebugGSSNode, type DebugGSSEdge, type DebugGSSInfo, type ErrorInfo, type StatsData, type BuildFeatures, type DocumentSymbolData } from "../bindings";
  import { listen, emit } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { createMaximizeToggle } from "$lib/window-utils";
  import { onMount, tick } from "svelte";
  import { FolderOpen, Cog, Hammer, X, AlertTriangle, CheckCircle, Loader2, ChevronDown, ChevronRight, ZoomIn, ZoomOut, Maximize2, Minimize2, Expand, Fullscreen, GitFork, Bug, Braces, PanelBottom, Trash2, ChevronsDown, Copy, ClipboardCheck, UnfoldHorizontal, FoldHorizontal, Download, MoreHorizontal, Keyboard, List } from "lucide-svelte";
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
    highlightOutgoingEdges,
    clearEdgeHighlights,
    highlightClickedEdge,
    LABEL_MAX_LENGTH,
    INTERMEDIATE_MAX_LENGTH,
  } from "$lib/graph-styles";
  import { GraphCollapseManager, buildDebugSppfElements, exportGraphPng, parseNodeKind } from "$lib/graph-utils";
  import MonacoEditor from "$lib/MonacoEditor.svelte";

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

  // Hierarchical tree node for tree view
  interface TreeNode {
    id: number;
    label: string;
    kind: "Nonterminal" | "Token";
    start: number;
    end: number;
    children: TreeNode[];
  }

  // Convert flat parse tree to hierarchical structure
  function buildTree(parseTree: ParseTree): TreeNode | null {
    if (parseTree.nodes.length === 0) return null;

    // Build adjacency list from edges
    const childrenMap = new Map<number, number[]>();
    const hasParent = new Set<number>();

    for (const edge of parseTree.edges) {
      if (!childrenMap.has(edge.src)) childrenMap.set(edge.src, []);
      childrenMap.get(edge.src)!.push(edge.dest);
      hasParent.add(edge.dest);
    }

    // Find root (node with no parent)
    const rootNode = parseTree.nodes.find(n => !hasParent.has(n.id));
    if (!rootNode) return null;

    // Build node lookup for efficient access
    const nodeMap = new Map(parseTree.nodes.map(n => [n.id, n]));

    // Build tree recursively
    function buildSubtree(nodeId: number): TreeNode {
      const node = nodeMap.get(nodeId)!;
      const childIds = childrenMap.get(nodeId) || [];
      return {
        id: node.id,
        label: node.label,
        kind: node.kind,
        start: node.start,
        end: node.end,
        children: childIds.map(buildSubtree),
      };
    }

    return buildSubtree(rootNode.id);
  }

  cytoscape.use(dagre);

  // Event listeners for build progress
  onMount(() => {
    const unlistenProgress = listen<{ stage: string; message: string }>("build-progress", (event) => {
      // Progress is shown in title bar status, not status bar
    });

    const unlistenResult = listen<{ success: boolean; message: string; features?: BuildFeatures | null }>("build-result", async (event) => {
      statusMessage = null;  // Clear status message
      if (!event.payload.success) {
        isBuilding = false;
      }
      if (event.payload.success) {
        buildStatus = "success";
        buildDurationMs = buildStartTime != null ? Math.round(performance.now() - buildStartTime) : null;
        buildFeatures = event.payload.features ?? null;
        // If the new binary lacks debug-trace, leave Debug mode.
        // If it lacks instrument, clear stats and leave the Stats tab.
        if (buildFeatures && !buildFeatures.debug_trace && activeMode === "debug") {
          activeMode = "parse";
        }
        if (!buildFeatures?.instrument) {
          statsData = null;
          if (activeTab === "stats") activeTab = "parse-tree";
        }
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
        isBuilding = false;
      } else {
        buildStatus = "error";
        buildError = event.payload.message;
        logError(`Build failed\n${event.payload.message}`);
        outputPanelOpen = true;
      }
    });

    const unlistenGenerateResult = listen<{ success: boolean; message: string; duration_ms?: number }>("generate-result", async (event) => {
      isGenerating = false;
      statusMessage = null;  // Clear status message
      if (event.payload.success) {
        generateStatus = "success";
        generateDurationMs = event.payload.duration_ms ?? (generateStartTime != null ? Math.round(performance.now() - generateStartTime) : null);
        const durationStr = generateDurationMs != null ? ` (${generateDurationMs}ms)` : "";
        logOutput(`Parser generated successfully${durationStr}`);
        setStatus(`Generated${durationStr}`, "success");
        setTimeout(() => { generateStatus = "none"; }, 2000);
        // Chain into build with the current build config
        await buildParser();
      } else {
        generateStatus = "error";
        logError(event.payload.message);
        outputPanelOpen = true;
        setTimeout(() => { generateStatus = "none"; }, 3000);
      }
    });

    const unlistenProfileProgress = listen<{ stage: string; message: string }>("profile-progress", (event) => {
      setStatus(event.payload.message, "info");
    });

    const unlistenProfileResult = listen<{ success: boolean; message: string }>("profile-result", (event) => {
      isProfiling = false;
      if (event.payload.success) {
        setStatus("Flamegraph opened in browser", "success");
        logOutput(event.payload.message);
      } else {
        setStatus("Profiling failed", "error");
        logError(`Profiling failed\n${event.payload.message}`);
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
        clearEdgeHighlights(debugSppfCy);
        if (nodeId) {
          debugSppfCy.getElementById(nodeId).addClass('selected');
          highlightOutgoingEdges(debugSppfCy, nodeId);
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
            highlightOutgoingEdges(parseTreeCy, savedParseTreeSelection);
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
            highlightOutgoingEdges(cy, savedSppfSelection);
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
            highlightOutgoingEdges(debugSppfCy, savedDebugSelection);
          }
        });
      }
    });

    // Listen for step changes from event log window (when user clicks an entry)
    const unlistenDebugStepChanged = listen("debug-step-changed", async () => {
      // Refresh debug state from backend
      const result = await commands.getDebugInfo();
      if (result.status === "ok") {
        currentStep = result.data.current_step;
        currentErrorIndex = result.data.current_error_index ?? null;
        currentAction = result.data.current_action;
        descriptorSet = result.data.descriptor_set;
        inputIndex = result.data.input_index ?? null;
        await fetchStackTrace();
        await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
        await notifyDebugGraphWindows();
      }
    });

    // Close all graph windows when main window closes
    const mainWindow = getCurrentWindow();
    const unlistenMainClose = mainWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      await closeAllGraphWindows();
      await mainWindow.destroy();
    });

    // Listen for "Open Grammar" from command palette
    function handleTerrariumOpenGrammar() {
      selectDirectory();
    }
    window.addEventListener("terrarium-open-grammar", handleTerrariumOpenGrammar);

    // Listen for Cmd+G from Monaco editor (which can't bubble keyboard events)
    function handleTerrariumGenerate() {
      if (grammarFileName && !isGenerating) {
        generateParser();
      }
    }
    window.addEventListener("terrarium-generate", handleTerrariumGenerate);

    // Listen for Cmd+P from Monaco editor (only fires when editor has focus)
    function handleTerrariumParse() {
      if (activeMode === "design") {
        // In Design mode, re-run the same grammar analysis that fires on every
        // keystroke (uses linked iggy via lsp::build, no generated parser needed).
        commands.analyzeGrammar(grammarText).then(onGrammarAnalyze);
        return;
      }
      if (buildStatus === "success" && startNonterminal) {
        parse();
      }
    }
    window.addEventListener("terrarium-parse", handleTerrariumParse);

    // Listen for Cmd+1/2/3 from Monaco editor (only fires when editor has focus)
    function handleTerrariumMode(e: Event) {
      const mode = (e as CustomEvent).detail as "design" | "parse" | "debug";
      if (mode === "design") activeMode = "design";
      else if (mode === "parse" && buildStatus === "success") activeMode = "parse";
      else if (mode === "debug" && buildStatus === "success" && buildFeatures?.debug_trace) activeMode = "debug";
    }
    window.addEventListener("terrarium-mode", handleTerrariumMode);

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenResult.then(fn => fn());
      unlistenGenerateResult.then(fn => fn());
      unlistenGraphWindowReady.then(fn => fn());
      unlistenStepBack.then(fn => fn());
      unlistenStepForward.then(fn => fn());
      unlistenNodeSelected.then(fn => fn());
      unlistenSpansToggled.then(fn => fn());
      unlistenDebugStepChanged.then(fn => fn());
      unlistenMainClose.then(fn => fn());
      window.removeEventListener('resize', handleWindowResize);
      window.removeEventListener("terrarium-open-grammar", handleTerrariumOpenGrammar);
      window.removeEventListener("terrarium-generate", handleTerrariumGenerate);
      window.removeEventListener("terrarium-parse", handleTerrariumParse);
      window.removeEventListener("terrarium-mode", handleTerrariumMode);
    };
  });

  // Parser directory state
  let parserDirectory = $state<string | null>(null);
  let parserName = $state<string | null>(null);
  let isBuilding = $state(false);
  let isProfiling = $state(false);
  let buildStatus = $state<"none" | "success" | "error">("none");
  let buildError = $state<string | null>(null);
  let showReadyStatus = $state(false);
  let readyStatusTimeout: ReturnType<typeof setTimeout> | null = null;

  // Modal state
  let showErrorModal = $state(false);
  let errorModalMessage = $state("");
  let showShortcutsModal = $state(false);

  // Title bar menu state
  let titleBarMenuOpen = $state(false);

  // Generation state
  let isGenerating = $state(false);
  let generateStatus = $state<"none" | "success" | "error">("none");
  let grammarText = $state("");
  let grammarFileName = $state<string | null>(null);

  // Auto-save grammar file (debounced 500ms after edits)
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  function onGrammarEdit(text: string) {
    if (!parserDirectory || !grammarFileName) return;
    if (saveTimeout) clearTimeout(saveTimeout);
    const dir = parserDirectory;
    const file = grammarFileName;
    saveTimeout = setTimeout(() => {
      commands.saveGrammar(dir, file, text);
    }, 500);
  }

  // Outline panel
  let outlineOpen = $state(false);
  let outlinePanelWidth = $state(200);
  let isDraggingOutline = $state(false);
  let outlineSymbols = $state<DocumentSymbolData[]>([]);
  let outlineExpanded = $state(new Set<string>());
  let outlineSelectedIndex = $state(-1);
  let outlineListEl: HTMLDivElement;
  let editorInstance: import("monaco-editor").editor.IStandaloneCodeEditor | undefined;

  interface OutlineItem {
    sym: DocumentSymbolData;
    isChild: boolean;
  }

  // Flat list of visible outline items (respects expand/collapse).
  function visibleOutlineItems(): OutlineItem[] {
    const items: OutlineItem[] = [];
    for (const sym of outlineSymbols) {
      items.push({ sym, isChild: false });
      if (sym.children.length > 0 && outlineExpanded.has(sym.name)) {
        for (const child of sym.children) {
          items.push({ sym: child, isChild: true });
        }
      }
    }
    return items;
  }

  function toggleOutlineNode(name: string) {
    const next = new Set(outlineExpanded);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    outlineExpanded = next;
  }

  function scrollOutlineItemIntoView() {
    const el = outlineListEl?.querySelector('.outline-item.selected') as HTMLElement | null;
    el?.scrollIntoView({ block: 'nearest' });
  }

  function handleOutlineKeydown(e: KeyboardEvent) {
    const items = visibleOutlineItems();
    if (items.length === 0) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      outlineSelectedIndex = Math.min(outlineSelectedIndex + 1, items.length - 1);
      revealSymbol(items[outlineSelectedIndex].sym);
      tick().then(scrollOutlineItemIntoView);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      outlineSelectedIndex = Math.max(outlineSelectedIndex - 1, 0);
      revealSymbol(items[outlineSelectedIndex].sym);
      tick().then(scrollOutlineItemIntoView);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      if (outlineSelectedIndex >= 0 && outlineSelectedIndex < items.length) {
        const item = items[outlineSelectedIndex];
        if (!item.isChild && item.sym.children.length > 0 && !outlineExpanded.has(item.sym.name)) {
          toggleOutlineNode(item.sym.name);
        }
      }
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      if (outlineSelectedIndex >= 0 && outlineSelectedIndex < items.length) {
        const item = items[outlineSelectedIndex];
        if (!item.isChild && outlineExpanded.has(item.sym.name)) {
          toggleOutlineNode(item.sym.name);
        } else if (item.isChild) {
          // Jump to parent
          for (let j = outlineSelectedIndex - 1; j >= 0; j--) {
            if (!items[j].isChild) {
              outlineSelectedIndex = j;
              revealSymbol(items[j].sym);
              tick().then(scrollOutlineItemIntoView);
              break;
            }
          }
        }
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (outlineSelectedIndex >= 0 && outlineSelectedIndex < items.length) {
        const item = items[outlineSelectedIndex];
        if (!item.isChild && item.sym.children.length > 0) {
          toggleOutlineNode(item.sym.name);
        }
        revealSymbol(item.sym);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      editorInstance?.focus();
    }
  }

  // Called by MonacoEditor after each grammar analysis (parse)
  function onGrammarAnalyze(result: { success: boolean; parse_duration_ms: number; tree_construction_duration_ms: number }) {
    if (result.success) {
      const totalMs = result.parse_duration_ms + result.tree_construction_duration_ms;
      setStatus(
        `Parsed (${totalMs}ms)`,
        "success",
        `Parse: ${result.parse_duration_ms}ms\nTree construction: ${result.tree_construction_duration_ms}ms`,
      );
    }
    // Refresh outline symbols
    commands.getDocumentSymbols(grammarText).then((symbols) => {
      outlineSymbols = symbols;
    });
  }

  function onEditorReady(editor: import("monaco-editor").editor.IStandaloneCodeEditor) {
    editorInstance = editor;
  }

  function revealSymbol(sym: DocumentSymbolData, focusEditor = false) {
    if (!editorInstance) return;
    const range = {
      startLineNumber: sym.selection_range.start_line + 1,
      startColumn: sym.selection_range.start_char + 1,
      endLineNumber: sym.selection_range.end_line + 1,
      endColumn: sym.selection_range.end_char + 1,
    };
    editorInstance.revealRangeInCenter(range);
    editorInstance.setSelection(range);
    if (focusEditor) editorInstance.focus();
  }

  // Status bar state
  let statusMessage = $state<string | null>(null);
  let statusTooltip = $state<string | null>(null);
  let statusType = $state<"info" | "error" | "success">("info");
  let showStatusDetails = $state(false);

  function setStatus(message: string, type: "info" | "error" | "success", tooltip?: string) {
    statusMessage = message;
    statusTooltip = tooltip ?? null;
    statusType = type;
  }

  // State
  let inputText = $state("");
  let lastParsedInput = $state<string | null>(null);
  let startNonterminal = $state<string | null>(null);
  let nonterminals = $state<string[]>([]);
  let dropdownOpen = $state(false);

  // Helper for displaying nonterminal (strips "Start" prefix if present for backwards compatibility)
  function displayNonterminal(nt: string | null): string {
    if (!nt) return "Select...";
    return nt.replace(/^Start/, "");
  }

  // Playback state
  let currentStep = $state(0);
  let totalSteps = $state(0);
  let totalErrors = $state(0);
  let currentErrorIndex = $state<number | null>(null);
  let errorList = $state<ErrorInfo[]>([]);
  let errorDropdownOpen = $state(false);

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
  let activeTab = $state<"gss" | "sppf" | "parse-tree" | "stats">("parse-tree");

  // Build configuration (chosen by user in Generate dropdown)
  let buildConfig = $state({ ll1: true, instrument: false, debugTrace: false });
  let generateDurationMs = $state<number | null>(null);
  let buildDurationMs = $state<number | null>(null);
  let buildStartTime: number | null = null;
  let generateStartTime: number | null = null;
  // Features the current binary was actually built with (null = no successful build)
  let buildFeatures = $state<BuildFeatures | null>(null);
  // Generate dropdown popover
  let generateMenuOpen = $state(false);
  // Stats panel
  let statsData = $state<StatsData | null>(null);

  // Show spans in graph labels (hidden by default)
  let showSpans = $state(false);

  // App mode
  let activeMode = $state<"design" | "parse" | "debug">("design");

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

  // Parse tree view mode (graph or tree)
  let parseTreeViewMode = $state<"graph" | "tree">("tree");
  let treeRoot = $state<TreeNode | null>(null);
  let expandedNodes = $state(new Set<number>());
  // svelte-ignore non_reactive_update
  let treeContainerEl: HTMLDivElement;

  // SPPF node selection (for highlighting span in input)
  let sppfSelectedSpan = $state<{ left: number; right: number } | null>(null);
  let sppfSelectedNodeId = $state<string | null>(null);

  // Context menu state for SPPF
  let sppfContextMenu = $state<{ x: number; y: number; nodeId: string } | null>(null);
  let sppfSubtreeFocused = $state(false);

  // Track if parse result is available
  let parseResultAvailable = $state(false);

  // Graph window management (separate OS windows)
  type GraphType = 'sppf' | 'gss' | 'debugSppf' | 'debugGss';
  let graphWindows = $state<Map<GraphType, WebviewWindow>>(new Map());

  // Event log window
  let eventLogWindow = $state<WebviewWindow | null>(null);

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

    // Reset collapsed nodes and focus when rendering new SPPF
    sppfCollapseManager.reset();
    sppfSubtreeFocused = false;
    // Clear selection when re-rendering
    sppfSelectedSpan = null;
    sppfSelectedNodeId = null;

    // Cleanup previous tooltip
    if (sppfTooltipCleanup) {
      sppfTooltipCleanup();
      sppfTooltipCleanup = null;
    }

    // Build a map of node IDs to their ambiguous status for edge coloring
    const nodeAmbiguousMap = new Map<number, boolean>();

    // Count terminal nodes per span to detect shared spans
    const spanCounts = new Map<string, number>();
    for (const node of sppf.nodes) {
      const { name: kindName } = parseNodeKind(node.kind);
      if (kindName === "Terminal") {
        const spanKey = `${node.left_extent},${node.right_extent}`;
        spanCounts.set(spanKey, (spanCounts.get(spanKey) || 0) + 1);
      }
    }

    const elements: cytoscape.ElementDefinition[] = [
      ...sppf.nodes.map((node) => {
        const { name: kindName, ambiguous } = parseNodeKind(node.kind);
        nodeAmbiguousMap.set(node.id, ambiguous);
        const baseLabel = node.label || (kindName === "Packed" ? "●" : "");
        // Intermediate nodes get longer max length since they show grammar slots
        const maxLen = kindName === "Intermediate" ? INTERMEDIATE_MAX_LENGTH : LABEL_MAX_LENGTH;
        // Optionally add span to label (skip for packed nodes which have no real span)
        const span = `(${node.left_extent}, ${node.right_extent})`;
        const displayLabel = showSpans && kindName !== "Packed"
          ? `${truncateLabel(baseLabel, maxLen)}\n${span}`
          : truncateLabel(baseLabel, maxLen);
        const fullLabel = showSpans && kindName !== "Packed"
          ? `${baseLabel}\n${span}`
          : baseLabel;
        // Check if this terminal node shares its span with other terminals
        const spanKey = `${node.left_extent},${node.right_extent}`;
        const hasSharedSpan = kindName === "Terminal" && (spanCounts.get(spanKey) || 0) > 1;
        let classes = kindName.toLowerCase();
        if (ambiguous) classes += ' ambiguous';
        if (hasSharedSpan) classes += ' shared-span';
        return {
          data: {
            id: `n${node.id}`,
            label: displayLabel,
            fullLabel: fullLabel,
            kind: kindName,
            ambiguous: ambiguous,
            leftExtent: node.left_extent,
            rightExtent: node.right_extent,
          },
          classes: classes,
        };
      }),
      ...sppf.edges.map((edge, i) => {
        const sourceAmbiguous = nodeAmbiguousMap.get(edge.src) || false;
        return {
          data: {
            id: `e${i}`,
            source: `n${edge.src}`,
            target: `n${edge.dest}`,
          },
          classes: sourceAmbiguous ? "edge-ambiguous" : "",
        };
      }),
    ];

    // Save viewport before destroying
    const savedViewport = cy ? getViewport(cy) : undefined;

    if (cy) {
      cy.destroy();
    }

    cy = createGraph({
      container: sppfContainer,
      elements,
      styles: [...sppfNodeStyles, ...edgeStyles],
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
      // Clear previous edge highlights and highlight new outgoing edges
      if (cy) {
        clearEdgeHighlights(cy);
        sppfSelectedNodeId = node.id();
        node.addClass('selected');
        highlightOutgoingEdges(cy, node.id());
      }
      // Close context menu on regular click
      sppfContextMenu = null;
    });

    // Click on background to clear selection and close context menu
    cy.on('tap', (event) => {
      if (event.target === cy) {
        sppfSelectedSpan = null;
        if (sppfSelectedNodeId && cy) {
          cy.getElementById(sppfSelectedNodeId).removeClass('selected');
          sppfSelectedNodeId = null;
        }
        if (cy) clearEdgeHighlights(cy);
        sppfContextMenu = null;
      }
    });

    // Click on edge to highlight it
    cy.on('tap', 'edge', (event) => {
      const edge = event.target;
      // Clear node selection
      if (sppfSelectedNodeId && cy) {
        cy.getElementById(sppfSelectedNodeId).removeClass('selected');
        sppfSelectedNodeId = null;
      }
      sppfSelectedSpan = null;
      if (cy) highlightClickedEdge(cy, edge.id());
    });

    // Right-click on node to show context menu
    cy.on('cxttap', 'node', (event) => {
      const node = event.target;
      const renderedPos = node.renderedPosition();
      const containerRect = sppfContainer.getBoundingClientRect();
      // Hide tooltip when showing context menu
      const tooltip = document.querySelector('.graph-tooltip') as HTMLElement;
      if (tooltip) tooltip.style.display = 'none';
      sppfContextMenu = {
        x: containerRect.left + renderedPos.x,
        y: containerRect.top + renderedPos.y,
        nodeId: node.id()
      };
    });

    // Right-click on background to close context menu
    cy.on('cxttap', (event) => {
      if (event.target === cy) {
        sppfContextMenu = null;
      }
    });
  }

  function handleSppfContextMenuAction(action: 'focus' | 'showAll') {
    if (action === 'focus' && sppfContextMenu) {
      sppfCollapseManager.focusOnSubtree(sppfContextMenu.nodeId);
      sppfSubtreeFocused = true;
    } else if (action === 'showAll') {
      sppfCollapseManager.clearFocus();
      sppfSubtreeFocused = false;
    }
    sppfContextMenu = null;
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
      styles: [...sppfNodeStyles, ...edgeStyles],  // Reuse SPPF styles (nonterminal/token)
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
      // Clear previous edge highlights and highlight new outgoing edges
      if (parseTreeCy) {
        clearEdgeHighlights(parseTreeCy);
        parseTreeSelectedNodeId = node.id();
        node.addClass('selected');
        highlightOutgoingEdges(parseTreeCy, node.id());
      }
    });

    // Click on background clears selection
    parseTreeCy.on('tap', (event) => {
      if (event.target === parseTreeCy) {
        parseTreeSelectedSpan = null;
        if (parseTreeSelectedNodeId && parseTreeCy) {
          parseTreeCy.getElementById(parseTreeSelectedNodeId).removeClass('selected');
          parseTreeSelectedNodeId = null;
        }
        if (parseTreeCy) clearEdgeHighlights(parseTreeCy);
      }
    });

    // Click on edge to highlight it
    parseTreeCy.on('tap', 'edge', (event) => {
      const edge = event.target;
      // Clear node selection
      if (parseTreeSelectedNodeId && parseTreeCy) {
        parseTreeCy.getElementById(parseTreeSelectedNodeId).removeClass('selected');
        parseTreeSelectedNodeId = null;
      }
      parseTreeSelectedSpan = null;
      if (parseTreeCy) highlightClickedEdge(parseTreeCy, edge.id());
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
    } else if (activeTab === "parse-tree" && parseTree && parseTreeViewMode === "graph") {
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
      styles: [...sppfNodeStyles, ...edgeStyles],
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
      // Clear previous edge highlights and highlight new outgoing edges
      if (debugSppfCy) {
        clearEdgeHighlights(debugSppfCy);
        selectedNodeId = node.id();
        node.addClass('selected');
        highlightOutgoingEdges(debugSppfCy, node.id());
      }
    });

    // Click on background to clear selection
    debugSppfCy.on('tap', (event) => {
      if (event.target === debugSppfCy) {
        selectedSpan = null;
        if (selectedNodeId && debugSppfCy) {
          debugSppfCy.getElementById(selectedNodeId).removeClass('selected');
          selectedNodeId = null;
        }
        if (debugSppfCy) clearEdgeHighlights(debugSppfCy);
      }
    });

    // Click on edge to highlight it
    debugSppfCy.on('tap', 'edge', (event) => {
      const edge = event.target;
      // Clear node selection
      if (selectedNodeId && debugSppfCy) {
        debugSppfCy.getElementById(selectedNodeId).removeClass('selected');
        selectedNodeId = null;
      }
      selectedSpan = null;
      if (debugSppfCy) highlightClickedEdge(debugSppfCy, edge.id());
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
  let leftPanelWidth = $state(400);
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
      title: "Select Grammar Directory",
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
      grammarText = "";
      grammarFileName = null;


      // Log the working directory
      logOutput(`Working directory: ${parserDirectory}`);

      // Try to get parser name (might not exist yet if empty directory)
      const nameResult = await commands.getParserName(parserDirectory);
      if (nameResult.status === "ok") {
        parserName = nameResult.data;
        logOutput(`Parser: ${parserName}`);
      } else {
        parserName = null;
      }

      // Load grammar file into editor
      const grammarResult = await commands.loadGrammar(parserDirectory);
      if (grammarResult.status === "ok") {
        const [filename, content] = grammarResult.data;
        grammarFileName = filename;
        grammarText = content;
        editorInstance?.focus();

        logOutput(`Grammar: ${filename}`);
      }

      // No auto-build: user must press Generate explicitly.
      buildFeatures = null;
      // Force back to Design mode since Parse/Debug are gated on a successful build.
      activeMode = "design";
    }
  }

  function closeErrorModal() {
    showErrorModal = false;
    errorModalMessage = "";
  }

  async function buildParser() {
    if (!parserDirectory) return;
    isBuilding = true;
    buildStartTime = performance.now();
    buildError = null;
    statusMessage = null;  // Let isBuilding control the status display
    const featList = ["profile"];
    if (buildConfig.instrument) featList.push("instrument");
    if (buildConfig.debugTrace) featList.push("debug-trace");
    logCommand(`cargo build --release --features ${featList.join(",")}`);
    // Command returns immediately, results come via events
    await commands.buildParser(parserDirectory, buildConfig.instrument, buildConfig.debugTrace);
  }

  async function generateParser() {
    if (!parserDirectory) return;
    isGenerating = true;
    generateStartTime = performance.now();
    generateDurationMs = null;
    buildDurationMs = null;
    generateStatus = "none";
    const ll1Flag = buildConfig.ll1 ? "" : " --no-ll1";
    logCommand(`iguana generate --output .${ll1Flag}`);
    // Command returns immediately, results come via events; build chains after generate succeeds
    await commands.generateParser(parserDirectory, !buildConfig.ll1);
  }

  function clearStatus() {
    statusMessage = null;
    statusTooltip = null;
    showStatusDetails = false;
  }

  async function fetchStats() {
    if (!parserDirectory || !startNonterminal) return;
    const startSymbol = `Start${startNonterminal}`;
    const result = await commands.getStats(parserDirectory, inputText, startSymbol);
    if (result.status === "ok") {
      statsData = result.data;
    } else {
      statsData = null;
      logError(result.error);
      outputPanelOpen = true;
    }
  }

  async function parse() {
    if (!parserDirectory || buildStatus !== "success") return;
    setStatus("Parsing...", "info");

    // Reset previous results
    sppf = null;
    gss = null;
    parseTree = null;
    statsData = null;
    parseResultAvailable = false;
    parseTreeSelectedSpan = null;
    sppfSelectedSpan = null;

    const startSymbol = `Start${startNonterminal}`;
    logCommand(`${parserName} <input> --start ${startSymbol}`);

    const result = await commands.parse(parserDirectory, inputText, startSymbol);
    if (result.status === "error") {
      // Command itself failed (couldn't run parser)
      logError(result.error);
      outputPanelOpen = true;
      setStatus("Parse failed", "error");
      return;
    }

    const output = result.data;
    lastParsedInput = inputText;

    // Some data may be available even if parsing or parse tree creation failed
    parseResultAvailable = output.has_sppf || output.has_gss || output.has_parse_tree;

    if (output.success) {
      const totalMs = (output.duration_ms ?? 0) + (output.tree_construction_ms ?? 0);
      const durationStr = output.duration_ms != null ? ` (${totalMs}ms)` : "";
      logOutput(`Parse successful${durationStr}`);
      const tooltip = output.duration_ms != null
        ? `Parse: ${output.duration_ms}ms\nTree construction: ${output.tree_construction_ms ?? '?'}ms`
        : undefined;
      setStatus(`Parse successful${durationStr}`, "success", tooltip);
    } else {
      // Partial success - show error but still display available data
      if (output.error) {
        logError(output.error);
      }
      if (parseResultAvailable) {
        const available = [
          output.has_sppf ? "SPPF" : null,
          output.has_gss ? "GSS" : null,
          output.has_parse_tree ? "Parse Tree" : null,
        ].filter(Boolean).join(", ");
        logOutput(`Partial data available: ${available}`);
        setStatus("Parse error (partial data)", "error");
      } else {
        setStatus("Parse failed", "error");
      }
      outputPanelOpen = true;
    }

    // Fetch the data for the active tab if available
    if (activeTab === "sppf" && output.has_sppf) {
      await fetchSppf();
    } else if (activeTab === "gss" && output.has_gss) {
      await fetchGss();
    } else if (activeTab === "parse-tree" && output.has_parse_tree) {
      await fetchParseTree();
    } else if (activeTab === "stats" && buildFeatures?.instrument) {
      await fetchStats();
    } else if (output.has_sppf) {
      // If active tab data not available, try to show something
      await fetchSppf();
    } else if (output.has_gss) {
      await fetchGss();
    }

    // Always fetch stats when instrument is enabled (independent of active tab)
    if (buildFeatures?.instrument && activeTab !== "stats") {
      await fetchStats();
    }
  }

  function profileParser() {
    if (!parserDirectory || buildStatus !== "success" || !startNonterminal) return;
    isProfiling = true;
    const startSymbol = `Start${startNonterminal}`;
    logCommand(`${parserName} <input> --start ${startSymbol} --profile 1000`);
    commands.profile(parserDirectory, inputText, startSymbol, 1000);
  }

  async function setupVscodeDebug() {
    if (!parserDirectory || !startNonterminal) return;

    const startSymbol = `Start${startNonterminal}`;
    const result = await commands.setupVscodeDebug(parserDirectory, inputText, startSymbol);
    if (result.status === "ok") {
      logOutput(`Debug config: .vscode/launch.json`);
      logOutput(`Debug input: .vscode/debug-input.txt`);
      logOutput(`→ Open ${parserDirectory} in VS Code, press F5`);
      outputPanelOpen = true;
    } else {
      logError(`Failed to setup debug config: ${result.error}`);
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
        // Build hierarchical tree for tree view
        treeRoot = buildTree(parseTree);
        // Expand root by default
        if (treeRoot) {
          expandedNodes = new Set([treeRoot.id]);
        }
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
      default: return null;
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
          highlightOutgoingEdges(parseTreeCy, savedParseTreeSelection);
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
          highlightOutgoingEdges(cy, savedSppfSelection);
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
          highlightOutgoingEdges(debugSppfCy, savedDebugSelection);
        }
      });
    }
    // Notify popup windows about the change
    notifyPopupWindowsSpansChanged();
  }

  // Tree view functions
  function toggleTreeNode(nodeId: number) {
    if (expandedNodes.has(nodeId)) {
      expandedNodes.delete(nodeId);
    } else {
      expandedNodes.add(nodeId);
    }
    expandedNodes = new Set(expandedNodes); // Trigger reactivity
  }

  function expandAllTreeNodes() {
    if (!treeRoot) return;
    const allWithChildren = new Set<number>();
    function collect(node: TreeNode) {
      if (node.children.length > 0) {
        allWithChildren.add(node.id);
        node.children.forEach(collect);
      }
    }
    collect(treeRoot);
    expandedNodes = allWithChildren;
  }

  function collapseAllTreeNodes() {
    expandedNodes = new Set();
  }

  function selectTreeNode(node: TreeNode) {
    parseTreeSelectedSpan = { start: node.start, end: node.end };
    parseTreeSelectedNodeId = `n${node.id}`;
    // Scroll selected node into view
    tick().then(() => {
      if (treeContainerEl) {
        const selectedEl = treeContainerEl.querySelector('.tree-item.selected');
        if (selectedEl) {
          selectedEl.scrollIntoView({ block: 'nearest' });
        }
      }
    });
  }

  // Get visible nodes in display order (for keyboard navigation)
  function getVisibleTreeNodes(): TreeNode[] {
    if (!treeRoot) return [];
    const visible: TreeNode[] = [];
    function collect(node: TreeNode) {
      visible.push(node);
      if (expandedNodes.has(node.id)) {
        node.children.forEach(collect);
      }
    }
    collect(treeRoot);
    return visible;
  }

  // Find a TreeNode by ID
  function findTreeNodeById(nodeId: number): TreeNode | null {
    if (!treeRoot) return null;
    function find(node: TreeNode): TreeNode | null {
      if (node.id === nodeId) return node;
      for (const child of node.children) {
        const found = find(child);
        if (found) return found;
      }
      return null;
    }
    return find(treeRoot);
  }

  // Handle keyboard navigation in tree view
  function handleTreeKeydown(e: KeyboardEvent) {
    if (!treeRoot) return;

    const visibleNodes = getVisibleTreeNodes();
    if (visibleNodes.length === 0) return;

    // Get currently selected node ID (strip 'n' prefix)
    const currentId = parseTreeSelectedNodeId ? parseInt(parseTreeSelectedNodeId.slice(1)) : null;
    const currentIndex = currentId !== null
      ? visibleNodes.findIndex(n => n.id === currentId)
      : -1;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (currentIndex < visibleNodes.length - 1) {
        selectTreeNode(visibleNodes[currentIndex + 1]);
      } else if (currentIndex === -1 && visibleNodes.length > 0) {
        selectTreeNode(visibleNodes[0]);
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (currentIndex > 0) {
        selectTreeNode(visibleNodes[currentIndex - 1]);
      } else if (currentIndex === -1 && visibleNodes.length > 0) {
        selectTreeNode(visibleNodes[visibleNodes.length - 1]);
      }
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      if (currentId !== null) {
        const node = findTreeNodeById(currentId);
        if (node && node.children.length > 0 && !expandedNodes.has(node.id)) {
          toggleTreeNode(node.id);
        }
      }
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      if (currentId !== null) {
        const node = findTreeNodeById(currentId);
        if (node && node.children.length > 0 && expandedNodes.has(node.id)) {
          toggleTreeNode(node.id);
        }
      }
    }
  }

  function notifyPopupWindowsSpansChanged() {
    // Re-send data to popup windows with updated showSpans
    for (const [graphType] of graphWindows) {
      if (graphType === 'debugSppf') {
        emit('graph-data-debug-sppf', {
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
    if (!graphWindows.has(graphType)) return;

    switch (graphType) {
      case 'sppf':
        if (sppf) {
          await emit('graph-data-sppf', sppf);
        }
        break;
      case 'gss':
        if (gss) {
          await emit('graph-data-gss', gss);
        }
        break;
      case 'debugSppf':
        await emit('graph-data-debug-sppf', {
          nodes: debugSppfNodes,
          current_node_id: currentSppfNodeId,
          show_spans: showSpans,
        });
        break;
      case 'debugGss':
        await emit('graph-data-debug-gss', {
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
    // Event log window polls for updates from the backend
  }

  // Close all graph windows
  async function closeAllGraphWindows() {
    for (const [, webview] of graphWindows) {
      await webview.close();
    }
    graphWindows.clear();
    // Also close event log window
    if (eventLogWindow) {
      await eventLogWindow.close();
      eventLogWindow = null;
    }
  }

  // Event log window functions
  async function openEventLogWindow() {
    // If window already exists, focus it
    if (eventLogWindow) {
      await eventLogWindow.setFocus();
      return;
    }

    const webview = new WebviewWindow('eventlog', {
      url: '/eventlog',
      title: 'Event Log',
      width: 500,
      height: 700,
      center: true,
      titleBarStyle: 'overlay',
      hiddenTitle: true,
    });

    webview.once('tauri://created', () => {
      eventLogWindow = webview;
    });

    webview.once('tauri://destroyed', () => {
      eventLogWindow = null;
    });
  }

  function stopDebug() {
    debugLoaded = false;
    currentStep = 0;
    totalSteps = 0;
    totalErrors = 0;
    currentErrorIndex = null;
    errorList = [];
    errorDropdownOpen = false;
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
    totalErrors = 0;
    currentErrorIndex = null;
    errorList = [];
    errorDropdownOpen = false;
    currentAction = null;
    descriptorSet = [];
    inputIndex = null;

    const startSymbol = `Start${startNonterminal}`;
    const result = await commands.loadDebugTrace(parserDirectory, inputText, startSymbol);
    if (result.status === "ok") {
      const { input_path, symbols_path, trace_path, current_action, descriptor_set, input_index, total_errors, current_error_index } = result.data;
      logCommand(`${parserName} --write-symbols ${symbols_path}`);
      logCommand(`${parserName} ${input_path} --start ${startSymbol} --trace ${trace_path} --format json`);
      debugLoaded = true;
      currentStep = result.data.current_step;
      totalSteps = result.data.total_steps;
      totalErrors = total_errors;
      currentErrorIndex = current_error_index ?? null;
      currentAction = current_action;
      descriptorSet = descriptor_set;
      inputIndex = input_index ?? null;
      logOutput(`Loaded ${totalSteps} steps, ${totalErrors} errors`);
      setStatus(`Loaded ${totalSteps} steps`, "success");
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS(), fetchErrorList()]);
      await notifyDebugGraphWindows();
    } else {
      logCommand(`${parserName} --write-symbols <symbols.json>`);
      logCommand(`${parserName} <input> --start ${startSymbol} --trace <trace.json> --format json`);
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
      currentErrorIndex = result.data.current_error_index ?? null;
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
      currentErrorIndex = result.data.current_error_index ?? null;
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
      currentErrorIndex = result.data.current_error_index ?? null;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function goToFurthestError() {
    if (!debugLoaded || totalErrors === 0) return;
    clearNodeSelection();
    errorDropdownOpen = false;
    const result = await commands.debugGoToFurthestError();
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentErrorIndex = result.data.current_error_index ?? null;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function goToError(stepIndex: number) {
    if (!debugLoaded) return;
    clearNodeSelection();
    errorDropdownOpen = false;
    const result = await commands.debugStepTo(stepIndex);
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentErrorIndex = result.data.current_error_index ?? null;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function nextError() {
    if (!debugLoaded || totalErrors === 0) return;
    clearNodeSelection();
    errorDropdownOpen = false;
    const result = await commands.debugNextError();
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentErrorIndex = result.data.current_error_index ?? null;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function prevError() {
    if (!debugLoaded || totalErrors === 0) return;
    clearNodeSelection();
    errorDropdownOpen = false;
    const result = await commands.debugPrevError();
    if (result.status === "ok") {
      currentStep = result.data.current_step;
      currentErrorIndex = result.data.current_error_index ?? null;
      currentAction = result.data.current_action;
      descriptorSet = result.data.descriptor_set;
      inputIndex = result.data.input_index ?? null;
      await fetchStackTrace();
      await Promise.all([fetchDebugSppf(), fetchDebugGSS()]);
      await notifyDebugGraphWindows();
    }
  }

  async function fetchErrorList() {
    const result = await commands.getDebugErrors();
    if (result.status === "ok") {
      errorList = result.data;
    } else {
      errorList = [];
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

  function startOutlineDrag(e: MouseEvent) {
    isDraggingOutline = true;
    e.preventDefault();
  }

  function onMouseMove(e: MouseEvent) {
    if (isDraggingVertical) {
      leftPanelWidth = Math.max(250, Math.min(600, e.clientX));
    }
    if (isDraggingOutline) {
      const mainArea = document.querySelector('.main-area');
      if (mainArea) {
        const rect = mainArea.getBoundingClientRect();
        outlinePanelWidth = Math.max(120, Math.min(400, rect.right - e.clientX));
      }
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
    isDraggingOutline = false;
  }

  function handleWindowClick(e: MouseEvent) {
    // Close dropdowns when clicking outside
    const target = e.target as HTMLElement;
    if (!target.closest('.custom-dropdown')) {
      dropdownOpen = false;
    }
    if (!target.closest('.title-bar-menu')) {
      titleBarMenuOpen = false;
    }
    if (!target.closest('.generate-split')) {
      generateMenuOpen = false;
    }
  }

  function startWindowDrag() {
    getCurrentWindow().startDragging();
  }

  const toggleMaximize = createMaximizeToggle();

  function handleKeyDown(e: KeyboardEvent) {
    // Cmd+Shift+P: command palette (global — focus editor first so Monaco can open it)
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'p') {
      e.preventDefault();
      if (editorInstance) {
        editorInstance.focus();
        editorInstance.trigger("keyboard", "editor.action.quickCommand", null);
      }
      return;
    }

    // Cmd+O: open grammar when no grammar is loaded (otherwise Monaco handles it for symbols)
    if ((e.metaKey || e.ctrlKey) && e.key === 'o' && !e.shiftKey) {
      if (!grammarFileName) {
        e.preventDefault();
        selectDirectory();
      }
      return;
    }

    // Cmd+G to generate & build parser (any mode)
    if ((e.metaKey || e.ctrlKey) && e.key === 'g') {
      e.preventDefault();
      if (grammarFileName && !isGenerating && !isBuilding) {
        generateParser();
      }
      return;
    }

    // Cmd+1/2/3 to switch modes (if enabled)
    if ((e.metaKey || e.ctrlKey) && (e.key === '1' || e.key === '2' || e.key === '3')) {
      e.preventDefault();
      if (e.key === '1') activeMode = "design";
      else if (e.key === '2' && buildStatus === "success") activeMode = "parse";
      else if (e.key === '3' && buildStatus === "success" && buildFeatures?.debug_trace) activeMode = "debug";
      return;
    }

    // Cmd+P to parse (always, in any mode).
    // In Design mode this re-runs grammar analysis (linked iggy, no build needed);
    // elsewhere it runs the generated parser on the test input.
    if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
      e.preventDefault();
      if (activeMode === "design") {
        commands.analyzeGrammar(grammarText).then(onGrammarAnalyze);
      } else if (buildStatus === "success" && startNonterminal) {
        parse();
      }
      return;
    }

    // Cmd+/ — toggle line comment (handled by Monaco, just don't intercept it)

    // Cmd+Shift+/ (⌘?) — show keyboard shortcuts
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === '/') {
      e.preventDefault();
      showShortcutsModal = !showShortcutsModal;
      return;
    }

    // Escape to close modals, deselect text, blur active element, and clear graph selections
    if (e.key === 'Escape') {
      // If focus is inside Monaco, let the editor handle Escape exclusively
      // (closing the quick outline / suggest / find widgets, etc.) and don't
      // run any of our global Escape side effects. Otherwise dismissing a
      // Monaco overlay would also clear graph selections, deselect text, etc.
      const active = document.activeElement as HTMLElement | null;
      if (active?.closest('.monaco-editor')) {
        return;
      }
      // Close context menu first
      if (sppfContextMenu) {
        sppfContextMenu = null;
        return;
      }
      // Close modals
      if (showShortcutsModal) {
        showShortcutsModal = false;
        return;
      }
      if (showErrorModal) {
        showErrorModal = false;
        return;
      }
      window.getSelection()?.removeAllRanges();
      active?.blur();
      // Clear all graph selections and edge highlights
      if (cy) {
        if (sppfSelectedNodeId) cy.getElementById(sppfSelectedNodeId).removeClass('selected');
        clearEdgeHighlights(cy);
      }
      if (parseTreeCy) {
        if (parseTreeSelectedNodeId) parseTreeCy.getElementById(parseTreeSelectedNodeId).removeClass('selected');
        clearEdgeHighlights(parseTreeCy);
      }
      if (debugSppfCy) {
        if (selectedNodeId) debugSppfCy.getElementById(selectedNodeId).removeClass('selected');
        clearEdgeHighlights(debugSppfCy);
      }
      sppfSelectedNodeId = null;
      sppfSelectedSpan = null;
      parseTreeSelectedNodeId = null;
      parseTreeSelectedSpan = null;
      selectedNodeId = null;
      selectedSpan = null;
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

<div class="app" class:dragging={isDraggingVertical || isDraggingHorizontal || isDraggingInput || isDraggingCurrent || isDraggingOutput || isDraggingDebug1 || isDraggingDebug2 || isDraggingDebugAction || isDraggingDebugStack || isDraggingDebugGraph || isDraggingOutline} class:dragging-horizontal={isDraggingHorizontal || isDraggingInput || isDraggingCurrent || isDraggingOutput || isDraggingDebugAction || isDraggingDebugStack || isDraggingDebugGraph}>
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
            <span class="palette-placeholder">Open Grammar...</span>
          {/if}
        </div>
        <div class="palette-status-area">
          {#if buildStatus === "error"}
            <AlertTriangle size={14} class="palette-status-error" />
          {/if}
        </div>
      </button>
      {#if grammarFileName}
        <div
          class="generate-split"
          class:generate-success={generateStatus === "success"}
          class:generate-error={generateStatus === "error"}
          class:busy={isGenerating || isBuilding}
          onmousedown={(e) => e.stopPropagation()}
          ondblclick={(e) => e.stopPropagation()}
        >
          <button
            class="generate-main"
            onclick={generateParser}
            disabled={isGenerating || isBuilding}
            title={isBuilding ? "Building..." : isGenerating ? "Generating..." : "Generate & Build Parser"}
          >
            {#if isGenerating || isBuilding}
              <Loader2 size={15} class="spinning" />
            {:else}
              <Hammer size={15} />
            {/if}
          </button>
          <button
            class="generate-chevron"
            onclick={() => generateMenuOpen = !generateMenuOpen}
            title="Generate options"
          >
            <ChevronDown size={11} />
          </button>
          {#if generateMenuOpen}
            <div class="generate-menu">
              <label class="generate-menu-item">
                <input type="checkbox" bind:checked={buildConfig.ll1} />
                <span>LL(1) optimization</span>
              </label>
              <label class="generate-menu-item">
                <input type="checkbox" bind:checked={buildConfig.instrument} />
                <span>Instrument (stats)</span>
              </label>
              <label class="generate-menu-item">
                <input type="checkbox" bind:checked={buildConfig.debugTrace} />
                <span>Debug trace</span>
              </label>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    <div class="title-bar-right">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="title-bar-menu" onmousedown={(e) => e.stopPropagation()}>
        <button
          class="title-bar-menu-btn"
          onclick={() => titleBarMenuOpen = !titleBarMenuOpen}
          title="More options"
        >
          <MoreHorizontal size={18} />
        </button>
        {#if titleBarMenuOpen}
          <div class="title-bar-menu-dropdown">
            <button
              class="menu-item"
              onclick={() => { setupVscodeDebug(); titleBarMenuOpen = false; }}
              disabled={!parserDirectory || buildStatus !== "success" || !startNonterminal}
            >
              Debug in VSCode...
            </button>
            <div class="menu-divider"></div>
            <button
              class="menu-item"
              onclick={() => { showShortcutsModal = true; titleBarMenuOpen = false; }}
            >
              Keyboard Shortcuts
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Middle Area (activity bar + content) -->
  <div class="middle-area" class:output-open={outputPanelOpen}>
    <!-- Activity Bar -->
    <div class="activity-bar">
      <button
        class="activity-btn"
        class:active={activeMode === "design"}
        onclick={() => activeMode = "design"}
        title="Design"
      >
        <Braces size={24} />
      </button>
      <button
        class="activity-btn"
        class:active={activeMode === "parse"}
        onclick={() => { if (buildStatus === "success") activeMode = "parse"; }}
        disabled={buildStatus !== "success"}
        title={buildStatus === "success" ? "Parse" : "Generate the parser to enable Parse mode"}
      >
        <GitFork size={24} style="transform: rotate(180deg)" />
      </button>
      <button
        class="activity-btn"
        class:active={activeMode === "debug"}
        onclick={() => { if (buildStatus === "success" && buildFeatures?.debug_trace) activeMode = "debug"; }}
        disabled={buildStatus !== "success" || !buildFeatures?.debug_trace}
        title={buildStatus !== "success"
          ? "Generate the parser to enable Debug mode"
          : buildFeatures?.debug_trace
            ? "Debug"
            : "Enable Debug Trace in Generate options to use Debug mode"}
      >
        <Bug size={24} />
      </button>
    </div>

    <!-- Main Area -->
    <div class="main-area">
    <!-- Mode Content -->
  {#if activeMode === "design"}
  <!-- Design Mode -->
  <div class="design-mode">
    <div class="design-editor">
      <MonacoEditor bind:value={grammarText} language="iggy" disabled={!grammarFileName} onchange={onGrammarEdit} onanalyze={onGrammarAnalyze} onready={onEditorReady} />
      {#if !grammarFileName}
        <div class="editor-placeholder">Open a grammar to get started</div>
      {/if}
    </div>
    {#if outlineOpen}
    <div class="resize-handle-vertical" onmousedown={startOutlineDrag}></div>
    <div class="outline-panel" style="width: {outlinePanelWidth}px">
      <div class="outline-header">
        <span class="outline-title">Outline</span>
      </div>
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="outline-list" tabindex="0" onkeydown={handleOutlineKeydown} bind:this={outlineListEl}>
        {#each visibleOutlineItems() as item, i}
          <button
            class="outline-item"
            class:outline-child={item.isChild}
            class:selected={i === outlineSelectedIndex}
            onmousedown={(e) => { e.preventDefault(); outlineSelectedIndex = i; revealSymbol(item.sym); outlineListEl?.focus(); }}
          >
            {#if !item.isChild && item.sym.children.length > 0}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span class="outline-chevron" onclick={(e) => { e.stopPropagation(); toggleOutlineNode(item.sym.name); }}>
                {#if outlineExpanded.has(item.sym.name)}
                  <ChevronDown size={14} />
                {:else}
                  <ChevronRight size={14} />
                {/if}
              </span>
            {:else if !item.isChild}
              <span class="outline-chevron-placeholder"></span>
            {/if}
            <span class="outline-icon" class:outline-icon-label={item.isChild}>{item.isChild ? '#' : item.sym.kind === 5 ? 'S' : item.sym.kind === 11 ? 'N' : 'R'}</span>
            <span class="outline-name">{item.sym.name}</span>
          </button>
        {/each}
      </div>
    </div>
    {/if}
    <div class="outline-strip">
      <button class="outline-strip-btn" class:active={outlineOpen} onclick={() => outlineOpen = !outlineOpen} title="Toggle outline">
        <List size={18} />
      </button>
    </div>
  </div>
  {:else if activeMode === "parse"}
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
              <span class="dropdown-value">{displayNonterminal(startNonterminal)}</span>
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
                    {displayNonterminal(nt)}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        <div class="parse-actions">
          <button class="parse-btn" onclick={parse} disabled={!parserDirectory || buildStatus !== "success" || !startNonterminal}>Parse</button>
          <button class="parse-btn" onclick={profileParser} disabled={!parserDirectory || buildStatus !== "success" || !startNonterminal || isProfiling}>
            {isProfiling ? "Profiling..." : "Profile"}
          </button>
        </div>
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
        {#if buildFeatures?.instrument}
          <button
            class:active={activeTab === "stats"}
            onclick={async () => { activeTab = "stats"; if (!statsData && parseResultAvailable) await fetchStats(); }}
          >Stats</button>
        {/if}
      </div>
      {#if activeTab === "parse-tree"}
        <div class="view-toggle-row">
          <div class="view-toggle">
            <button
              class:active={parseTreeViewMode === "graph"}
              onclick={() => parseTreeViewMode = "graph"}
            >Graph</button>
            <button
              class:active={parseTreeViewMode === "tree"}
              onclick={() => parseTreeViewMode = "tree"}
            >Tree</button>
          </div>
        </div>
      {/if}
      <div class="graph-container">
        {#if activeTab === "parse-tree"}
          {#if parseTree}
            {#if parseTreeViewMode === "graph"}
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
              <div class="tree-view">
                <div class="tree-controls">
                  <button onclick={expandAllTreeNodes} title="Expand All">
                    <Expand size={16} />
                  </button>
                  <button onclick={collapseAllTreeNodes} title="Collapse All">
                    <Minimize2 size={16} />
                  </button>
                </div>
                <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                <div class="tree-container" tabindex="0" onkeydown={handleTreeKeydown} bind:this={treeContainerEl}>
                  {#if treeRoot}
                    {#snippet treeNode(node: TreeNode, depth: number)}
                      <!-- svelte-ignore a11y_click_events_have_key_events -->
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="tree-item"
                        class:selected={parseTreeSelectedNodeId === `n${node.id}`}
                        style="padding-left: {depth * 16 + 8}px"
                        onclick={() => selectTreeNode(node)}
                        ondblclick={() => { if (node.children.length > 0) toggleTreeNode(node.id); }}
                      >
                        {#if node.children.length > 0}
                          <span class="expand-icon" onclick={(e) => { e.stopPropagation(); toggleTreeNode(node.id); }}>
                            {#if expandedNodes.has(node.id)}
                              <ChevronDown size={14} />
                            {:else}
                              <ChevronRight size={14} />
                            {/if}
                          </span>
                        {:else}
                          <span class="expand-icon-placeholder"></span>
                        {/if}
                        <span
                          class="tree-label"
                          class:nonterminal={node.kind === "Nonterminal"}
                          class:token={node.kind === "Token"}
                        >
                          {node.label}
                        </span>
                        <span class="tree-span">[{node.start}:{node.end}]</span>
                      </div>
                      {#if expandedNodes.has(node.id)}
                        {#each node.children as child}
                          {@render treeNode(child, depth + 1)}
                        {/each}
                      {/if}
                    {/snippet}
                    {@render treeNode(treeRoot, 0)}
                  {/if}
                </div>
              </div>
            {/if}
          {/if}
        {:else if activeTab === "sppf"}
          {#if sppf}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="cytoscape-container" bind:this={sppfContainer} oncontextmenu={(e) => e.preventDefault()}></div>
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
            {#if sppfContextMenu}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="context-menu"
                style="left: {sppfContextMenu.x}px; top: {sppfContextMenu.y}px;"
                onmousedown={(e) => e.stopPropagation()}
              >
                <button onclick={() => handleSppfContextMenuAction('focus')}>Focus on subtree</button>
                {#if sppfSubtreeFocused}
                  <button onclick={() => handleSppfContextMenuAction('showAll')}>Show all nodes</button>
                {/if}
              </div>
            {/if}
            {#if sppfSubtreeFocused}
              <button class="show-all-button" onclick={() => handleSppfContextMenuAction('showAll')}>
                Show all nodes
              </button>
            {/if}
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
          {/if}
        {:else if activeTab === "stats"}
          <div class="stats-panel">
            {#if !buildFeatures?.instrument}
              <div class="stats-empty">Rebuild with the Instrument option enabled to collect stats.</div>
            {:else if !statsData}
              <div class="stats-empty">
                Run a parse to collect stats.
                {#if parseResultAvailable}
                  <div style="margin-top: 8px;">
                    <button class="parse-btn" onclick={fetchStats}>Collect now</button>
                  </div>
                {/if}
              </div>
            {:else}
              <div class="stats-counters">
                <div><span class="stats-label">descriptors</span><span class="stats-value">{statsData.descriptors_count}</span></div>
                <div><span class="stats-label">gss_nodes</span><span class="stats-value">{statsData.gss_nodes_count}</span></div>
                <div><span class="stats-label">gss_edges</span><span class="stats-value">{statsData.gss_edges_count}</span></div>
                <div><span class="stats-label">nonterminal_nodes</span><span class="stats-value">{statsData.nonterminal_nodes_count}</span></div>
                <div><span class="stats-label">intermediate_nodes</span><span class="stats-value">{statsData.intermediate_nodes_count}</span></div>
                <div><span class="stats-label">terminal_nodes</span><span class="stats-value">{statsData.terminal_nodes_count}</span></div>
                <div><span class="stats-label">ambiguous_nodes</span><span class="stats-value">{statsData.ambiguous_nodes_count}</span></div>
              </div>
              {#if Object.keys(statsData.histograms).length > 0}
                <div class="stats-histograms">
                  <h4>Size histograms</h4>
                  {#each Object.entries(statsData.histograms) as [name, lens] (name)}
                    {@const lensArr = lens as number[]}
                    {@const buckets = (() => {
                      const b = [0, 0, 0, 0, 0, 0, 0, 0];
                      for (const l of lensArr) {
                        if (l === 0) b[0]++;
                        else if (l === 1) b[1]++;
                        else if (l === 2) b[2]++;
                        else if (l <= 4) b[3]++;
                        else if (l <= 8) b[4]++;
                        else if (l <= 16) b[5]++;
                        else if (l <= 32) b[6]++;
                        else b[7]++;
                      }
                      return b;
                    })()}
                    {@const labels = ['0', '1', '2', '3-4', '5-8', '9-16', '17-32', '33+']}
                    {@const max = Math.max(1, ...buckets)}
                    {@const n = lensArr.length}
                    {@const sum = lensArr.reduce((a: number, b: number) => a + b, 0)}
                    {@const maxv = Math.max(0, ...lensArr)}
                    <div class="histogram">
                      <div class="histogram-name">{name}</div>
                      <div class="histogram-meta">n={n}  max={maxv}  avg={(sum / Math.max(1, n)).toFixed(2)}</div>
                      {#each buckets as count, i}
                        <div class="histogram-row">
                          <span class="histogram-bucket">{labels[i]}</span>
                          <div class="histogram-bar-container">
                            <div class="histogram-bar" style="width: {(count * 100) / max}%"></div>
                          </div>
                          <span class="histogram-count">{count}</span>
                        </div>
                      {/each}
                    </div>
                  {/each}
                </div>
              {/if}
            {/if}
          </div>
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
              <span class="dropdown-value">{displayNonterminal(startNonterminal)}</span>
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
                    {displayNonterminal(nt)}
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
          <div class="input-viewer">{#each inputText.split('') as char, i}<span class="input-char" class:consumed={inputIndex !== null && i < inputIndex} class:current={inputIndex !== null && i === inputIndex} class:selected={selectedSpan !== null && i >= selectedSpan.left && i < selectedSpan.right} class:whitespace={char === ' ' || char === '\t' || char === '\n'}>{#if char === ' '}<span class="ws-marker">·</span>{:else if char === '\t'}<span class="ws-marker">→</span>{:else if char === '\n'}<span class="ws-marker">↵</span>{'\n'}{:else}{char}{/if}</span>{/each}</div>
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
      <!-- Controls Container (both rows) -->
      <div class="debug-controls-container">
        <!-- Playback Controls -->
        <div class="playback-controls">
          <button onclick={stepBack} disabled={!debugLoaded || currentStep === 0} title="Step back">◀</button>
          <button onclick={stepForward} disabled={!debugLoaded || currentStep >= totalSteps - 1} title="Step forward">▶</button>
          {#if debugLoaded}
            <span class="step-counter">Step {currentStep + 1} / {totalSteps}</span>
            <button onclick={openEventLogWindow} class="event-log-btn" title="Open Event Log">
              <List size={14} />
            </button>
          {/if}
        </div>

        <!-- Error Navigation -->
        {#if debugLoaded && totalErrors > 0}
          <div class="error-controls">
            <button onclick={prevError} title="Previous error" class="error-nav-btn">◀</button>
            <button onclick={goToFurthestError} title="Go to furthest error (max input index)" class="error-nav-btn furthest-btn">Furthest</button>
            <button onclick={nextError} title="Next error" class="error-nav-btn">▶</button>
            <span class="error-counter">Error {currentErrorIndex ?? '-'} / {totalErrors}</span>

            <!-- Error Dropdown -->
            <div class="error-dropdown-container">
              <button
                class="error-dropdown-trigger"
                onclick={() => errorDropdownOpen = !errorDropdownOpen}
                title="Show all errors"
              >
                <ChevronDown size={14} />
              </button>
              {#if errorDropdownOpen}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="error-dropdown" onmouseleave={() => errorDropdownOpen = false}>
                  {#each errorList as error}
                    <button
                      class="error-dropdown-item"
                      class:active={error.step_index === currentStep}
                      onclick={() => goToError(error.step_index)}
                    >
                      <span class="error-index">Index {error.input_index}</span>
                      <span class="error-terminal">{error.description}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
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
          {#if debugSppfNodes.length > 0}
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
          {#if debugGssNodes.length > 0}
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
  {/if}

    </div>
  </div>

  <!-- Output Panel (overlay) -->
  {#if outputPanelOpen}
    <div class="output-panel-overlay">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle-horizontal" onmousedown={startOutputDrag}></div>
      <div class="output-panel">
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
      <button
        class="status-text-btn"
        onclick={() => outputPanelOpen = !outputPanelOpen}
      >
        {#if isGenerating || isBuilding}
          Generating Parser...
        {:else if statusMessage}
          {statusMessage}
        {:else if parserDirectory && buildStatus === "success"}
          Parser Generated
        {:else}
          No grammar selected
        {/if}
        {#if statusTooltip}
          <span class="status-tooltip">{statusTooltip}</span>
        {:else if parserDirectory && buildStatus === "success" && !isBuilding && !isGenerating && !statusMessage}
          <span class="status-tooltip">Generation: {generateDurationMs ?? '?'}ms
Compilation: {buildDurationMs ?? '?'}ms</span>
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
          <p class="modal-hint">Please select a directory containing an Iguana grammar.</p>
        </div>
        <div class="modal-footer">
          <button class="modal-btn" onclick={closeErrorModal}>OK</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Keyboard Shortcuts Modal -->
  {#if showShortcutsModal}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal-overlay" onclick={() => showShortcutsModal = false}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="modal shortcuts-modal" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <Keyboard size={20} color="#4ec9b0" />
          <span>Keyboard Shortcuts</span>
          <button class="modal-close" onclick={() => showShortcutsModal = false}>
            <X size={18} />
          </button>
        </div>
        <div class="modal-body shortcuts-body">
          <div class="shortcuts-section">
            <h4>Global</h4>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>⇧</kbd><kbd>P</kbd></span>
              <span class="shortcut-desc">Command palette</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>G</kbd></span>
              <span class="shortcut-desc">Generate &amp; build parser</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>P</kbd></span>
              <span class="shortcut-desc">Parse input</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>1</kbd></span>
              <span class="shortcut-desc">Switch to Design mode</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>2</kbd></span>
              <span class="shortcut-desc">Switch to Parse mode</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>3</kbd></span>
              <span class="shortcut-desc">Switch to Debug mode</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>?</kbd></span>
              <span class="shortcut-desc">Show keyboard shortcuts</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>Esc</kbd></span>
              <span class="shortcut-desc">Clear selection / close modal</span>
            </div>
          </div>
          <div class="shortcuts-section">
            <h4>Editor</h4>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>O</kbd></span>
              <span class="shortcut-desc">Show symbols in file</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>F3</kbd></span>
              <span class="shortcut-desc">Go to definition</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⇧</kbd><kbd>F12</kbd></span>
              <span class="shortcut-desc">Find all references</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>[</kbd> / <kbd>⌘</kbd><kbd>]</kbd></span>
              <span class="shortcut-desc">Navigate back / forward</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>⇧</kbd><kbd>F</kbd></span>
              <span class="shortcut-desc">Format grammar</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>F</kbd></span>
              <span class="shortcut-desc">Find</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>/</kbd></span>
              <span class="shortcut-desc">Toggle comment</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>⌘</kbd><kbd>D</kbd></span>
              <span class="shortcut-desc">Delete line</span>
            </div>
          </div>
          <div class="shortcuts-section">
            <h4>Debug Mode</h4>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>←</kbd></span>
              <span class="shortcut-desc">Step back</span>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-keys"><kbd>→</kbd></span>
              <span class="shortcut-desc">Step forward</span>
            </div>
          </div>
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

  /* Add padding to scrollable areas when output panel is open */
  .middle-area.output-open .input-section textarea,
  .middle-area.output-open .input-viewer,
  .middle-area.output-open .stack-list,
  .middle-area.output-open .section-content,
  .middle-area.output-open .tree-container {
    padding-bottom: 200px;  /* Space for output panel */
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
    flex-direction: row;
    min-height: 0;
  }

  .design-editor {
    flex: 1;
    min-height: 0;
    min-width: 0;
    position: relative;
  }

  .editor-placeholder {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: #808080;
    font-size: 16px;
    pointer-events: none;
  }

  .outline-panel {
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: #1e1e1e;
    border-left: 1px solid #3c3c3c;
    overflow: hidden;
  }

  .outline-header {
    display: flex;
    align-items: center;
    height: 32px;
    padding: 0 8px 0 12px;
    border-bottom: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .outline-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: #bbbbbb;
    letter-spacing: 0.5px;
    flex: 1;
  }

  .outline-strip {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 36px;
    background: #252526;
    border: none;
    border-left: 1px solid #3c3c3c;
    padding: 0;
    flex-shrink: 0;
  }

  .outline-strip-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: none;
    color: #888;
    cursor: pointer;
  }

  .outline-strip-btn:hover {
    background: #2a2d2e;
    color: #d4d4d4;
  }

  .outline-strip-btn.active {
    color: #d4d4d4;
  }

  .outline-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
    outline: none;
  }

  .outline-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 2px 4px 2px 4px;
    border: none;
    background: none;
    color: #d4d4d4;
    font-size: 13px;
    font-family: inherit;
    cursor: pointer;
    gap: 4px;
    text-align: left;
  }

  .outline-item:hover {
    background: #2a2d2e;
  }

  .outline-item.selected {
    background: #04395e;
  }

  .outline-item.selected:hover {
    background: #04395e;
  }

  .outline-child {
    padding-left: 32px;
  }

  .outline-chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    color: #888;
    cursor: pointer;
  }

  .outline-chevron:hover {
    color: #d4d4d4;
  }

  .outline-chevron-placeholder {
    width: 16px;
    flex-shrink: 0;
  }

  .outline-icon {
    font-size: 11px;
    font-weight: 700;
    color: #4ec9b0;
    width: 14px;
    text-align: center;
    flex-shrink: 0;
  }

  .outline-icon-label {
    color: #dcdcaa;
  }

  .outline-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    flex-shrink: 0;
    display: flex;
    justify-content: flex-end;
    align-items: center;
    padding-right: 12px;
    gap: 4px;
  }

  .title-bar-icon-btn {
    background: transparent;
    border: none;
    color: #999;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
  }

  .title-bar-icon-btn:hover:not(:disabled) {
    color: #d4d4d4;
    background: rgba(255, 255, 255, 0.1);
  }

  .title-bar-icon-btn:disabled {
    color: #555;
    cursor: not-allowed;
  }

  .title-bar-icon-btn.generate-success {
    color: #89d185;
    transition: color 0.5s ease;
  }

  .title-bar-icon-btn.generate-error {
    color: #f48771;
    transition: color 0.5s ease;
  }

  /* Title Bar Menu */
  .title-bar-menu {
    position: relative;
  }

  .title-bar-menu-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #888;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .title-bar-menu-btn:hover {
    background: #404040;
    color: #ccc;
  }

  .title-bar-menu-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 180px;
    background: #2d2d2d;
    border: 1px solid #454545;
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    padding: 4px 0;
    z-index: 1000;
  }

  .menu-item {
    display: block;
    width: 100%;
    padding: 8px 14px;
    background: transparent;
    border: none;
    color: #ccc;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .menu-item:hover:not(:disabled) {
    background: #094771;
  }

  .menu-item:disabled {
    color: #666;
    cursor: not-allowed;
  }

  .menu-divider {
    height: 1px;
    background: #3c3c3c;
    margin: 4px 0;
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
    flex-wrap: wrap;
    gap: 8px 12px;
    padding: 12px;
    border-bottom: 1px solid #3c3c3c;
    background: #2d2d2d;
  }

  .parse-actions {
    display: flex;
    gap: 6px;
    margin-left: auto;
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

  .input-char .ws-marker {
    font-size: 0.9em;
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

  /* Debug Controls Container */
  .debug-controls-container {
    border-bottom: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  /* Playback Controls */
  .playback-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
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

  .event-log-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px 8px;
    background: #3c3c3c;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    margin-left: auto;
  }

  .event-log-btn:hover {
    background: #4c4c4c;
  }

  /* Error Navigation Row */
  .error-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: #2a2020;
    flex-shrink: 0;
  }

  .error-nav-btn {
    padding: 4px 12px;
    background: #4a3030;
    color: #d4d4d4;
    border: 1px solid #6a4040;
    border-radius: 4px;
    cursor: pointer;
  }

  .error-nav-btn:hover {
    background: #5a4040;
  }

  .error-nav-btn.furthest-btn {
    background: #5a3535;
    border-color: #8a5050;
    font-weight: 500;
  }

  .error-nav-btn.furthest-btn:hover {
    background: #6a4545;
  }

  .error-counter {
    font-size: 12px;
    color: #e05050;
    min-width: 80px;
  }

  /* Error Dropdown */
  .error-dropdown-container {
    position: relative;
    margin-left: auto;
  }

  .error-dropdown-trigger {
    padding: 4px 8px;
    background: #4a3030;
    color: #d4d4d4;
    border: 1px solid #6a4040;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .error-dropdown-trigger:hover {
    background: #5a4040;
  }

  .error-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    background: #2d2d2d;
    border: 1px solid #555;
    border-radius: 4px;
    max-height: 300px;
    overflow-y: auto;
    min-width: 200px;
    z-index: 100;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .error-dropdown-item {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: #d4d4d4;
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-size: 12px;
  }

  .error-dropdown-item:hover {
    background: #3c3c3c;
  }

  .error-dropdown-item.active {
    background: #4a3030;
  }

  .error-index {
    color: #e05050;
    font-weight: 500;
  }

  .error-terminal {
    color: #ce9178;
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

  /* View toggle row (Graph/Tree) */
  .view-toggle-row {
    display: flex;
    padding: 6px 8px;
    background: #252526;
    border-bottom: 1px solid #3c3c3c;
  }

  .view-toggle {
    display: flex;
  }

  .view-toggle button {
    padding: 3px 10px;
    background: #2d2d2d;
    color: #888;
    border: 1px solid #3c3c3c;
    cursor: pointer;
    font-size: 11px;
  }

  .view-toggle button:first-child {
    border-radius: 3px 0 0 3px;
  }

  .view-toggle button:last-child {
    border-radius: 0 3px 3px 0;
    border-left: none;
  }

  .view-toggle button.active {
    background: #3c3c3c;
    color: #fff;
    border-color: #555;
  }

  .view-toggle button:hover:not(.active) {
    background: #3c3c3c;
    color: #d4d4d4;
  }

  /* Tree View */
  .tree-view {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }

  .tree-controls {
    padding: 8px;
    border-bottom: 1px solid #3c3c3c;
    display: flex;
    gap: 4px;
    background: #252526;
  }

  .tree-controls button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: #2d2d2d;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    color: #888;
    cursor: pointer;
  }

  .tree-controls button:hover {
    background: #3c3c3c;
    color: #d4d4d4;
  }

  .tree-container {
    flex: 1;
    overflow: auto;
    padding: 8px 0;
    font-family: 'SF Mono', 'Fira Code', Consolas, monospace;
    font-size: 13px;
    outline: none;
  }

  .tree-container:focus {
    outline: 1px solid #0e639c;
    outline-offset: -1px;
  }

  .tree-item {
    display: flex;
    align-items: center;
    padding: 2px 8px;
    cursor: pointer;
    white-space: nowrap;
  }

  .tree-item:hover {
    background: #2a2d2e;
  }

  .tree-item.selected {
    background: #094771;
  }

  .expand-icon {
    width: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
    cursor: pointer;
    flex-shrink: 0;
  }

  .expand-icon:hover {
    color: #d4d4d4;
  }

  .expand-icon-placeholder {
    width: 18px;
    flex-shrink: 0;
  }

  .tree-label {
    margin-right: 8px;
  }

  .tree-label.nonterminal {
    color: #4ec9b0;
  }

  .tree-label.token {
    color: #ce9178;
  }

  .tree-span {
    color: #6a9955;
    font-size: 11px;
    margin-left: auto;
    padding-left: 12px;
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
    pointer-events: none;  /* Let events pass through to content below */
  }

  /* Re-enable pointer events on interactive children */
  .output-panel-overlay > * {
    pointer-events: auto;
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
    position: relative;
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

  .status-tooltip {
    display: none;
    position: absolute;
    bottom: 100%;
    left: 0;
    margin-bottom: 6px;
    padding: 4px 8px;
    background: #252526;
    border: 1px solid #454545;
    border-radius: 4px;
    font-size: 12px;
    color: #ccc;
    white-space: pre;
    text-align: left;
    pointer-events: none;
  }

  .status-text-btn:hover .status-tooltip {
    display: block;
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

  /* Shortcuts Modal */
  .shortcuts-modal {
    min-width: 320px;
  }

  .shortcuts-body {
    padding: 8px 16px 16px;
  }

  .shortcuts-section {
    margin-bottom: 16px;
  }

  .shortcuts-section:last-child {
    margin-bottom: 0;
  }

  .shortcuts-section h4 {
    margin: 0 0 8px 0;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: #888;
    letter-spacing: 0.5px;
  }

  .shortcut-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 0;
  }

  .shortcut-keys {
    display: flex;
    gap: 4px;
  }

  .shortcut-keys kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 22px;
    padding: 0 6px;
    background: #3c3c3c;
    border: 1px solid #555;
    border-radius: 4px;
    font-family: inherit;
    font-size: 12px;
    color: #d4d4d4;
  }

  .shortcut-desc {
    color: #d4d4d4;
    font-size: 13px;
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

  .generate-split {
    position: relative;
    display: inline-flex;
    align-items: stretch;
    margin-left: 8px;
    height: 24px;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    background: #2d2d2d;
  }
  .generate-split.busy { cursor: wait; pointer-events: none; }
  .generate-main:first-child { border-top-left-radius: 3px; border-bottom-left-radius: 3px; }
  .generate-chevron:last-child { border-top-right-radius: 3px; border-bottom-right-radius: 3px; }
  .generate-split:hover:not(.busy) { background: #3a3a3a; }
  .generate-split.disabled { opacity: 0.5; }
  .generate-split.generate-error { border-color: #e05050; }
  .generate-main,
  .generate-chevron {
    background: transparent;
    border: 0;
    color: #cccccc;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }
  .generate-main { padding: 0 7px; }
  .generate-chevron {
    padding: 0 4px;
    border-left: 1px solid #3c3c3c;
  }
  .generate-main:hover,
  .generate-chevron:hover { background: rgba(255, 255, 255, 0.06); }
  .generate-main:disabled,
  .generate-chevron:disabled { cursor: default; }
  .generate-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    background: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 6px 4px;
    min-width: 200px;
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }
  .generate-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    cursor: pointer;
    color: #cccccc;
    font-size: 12px;
  }
  .generate-menu-item:hover {
    background: #2a2d2e;
  }
  .generate-menu-item input[type="checkbox"] {
    margin: 0;
    cursor: pointer;
  }

  .stats-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: #cccccc;
    font-size: 12px;
    cursor: pointer;
    padding: 0 6px;
  }
  .stats-toggle input { margin: 0; cursor: pointer; }

  .activity-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .stats-panel {
    padding: 16px 20px;
    overflow: auto;
    color: #cccccc;
    font-family: Menlo, monospace;
    font-size: 12px;
    width: 100%;
    height: 100%;
    align-self: stretch;
    box-sizing: border-box;
  }
  .stats-empty { color: #888; padding: 8px 0; }
  .stats-counters > div {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
    border-bottom: 1px solid #2d2d2d;
  }
  .stats-label { color: #888; }
  .stats-value { color: #4ec9b0; font-weight: 600; }
  .stats-histograms { margin-top: 18px; }
  .stats-histograms h4 { margin: 0 0 10px 0; color: #ddd; font-size: 12px; font-weight: 600; }
  .histogram { margin-bottom: 14px; }
  .histogram-name { color: #569cd6; margin-bottom: 2px; }
  .histogram-meta { color: #888; margin-bottom: 4px; font-size: 11px; }
  .histogram-row {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 16px;
  }
  .histogram-bucket {
    width: 38px;
    text-align: right;
    color: #888;
  }
  .histogram-bar-container {
    flex: 1;
    background: #1e1e1e;
    height: 10px;
    border-radius: 2px;
    overflow: hidden;
  }
  .histogram-bar {
    background: #4ec9b0;
    height: 100%;
  }
  .histogram-count {
    width: 36px;
    color: #aaa;
  }
</style>
