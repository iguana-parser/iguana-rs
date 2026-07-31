<script lang="ts">
  import type { ParserBackend } from "./backend";
  import type * as monaco from "monaco-editor";
  import { tick, untrack } from "svelte";
  import { ChevronDown, ChevronRight, CornerRightUp, Minimize2, Expand, SlidersHorizontal, Copy, ClipboardCheck, Play } from "lucide-svelte";
  import cytoscape from "cytoscape";
  import {
    sppfNodeStyles,
    edgeStyles,
    adjustZoomGraph,
    resetViewGraph,
    createGraph,
    setupGraphTooltip,
    highlightOutgoingEdges,
    clearEdgeHighlights,
    highlightClickedEdge,
    PARSE_TREE_LAYOUT,
    PARSE_TREE_WEBGL_NODE_THRESHOLD,
  } from "./graph-styles";
  import { createGraphControls } from "./graph-controls";
  import {
    GraphCollapseManager,
    buildParseTreeElements,
    buildDisplayGraph,
    type DisplayOptions,
    type ParseTreeData as ParseTree,
    type ParseTreeNodeData as ParseTreeNode,
  } from "./parse-tree-graph";
  import { downloadPng } from "./png";
  import PlainEditor from "./PlainEditor.svelte";
  import NonterminalPicker from "./NonterminalPicker.svelte";
  import "./graph.css";
  import "./parse-view.css";

  // ParseTree / ParseTreeNode (the DAG to_json emits) and the display transform
  // are shared with the graph and web viewer, so they live in parse-tree-graph.

  // Hierarchical tree node for tree view
  interface TreeNode {
    id: number;
    label: string;
    kind: "Nonterminal" | "Token" | "Amb";
    start: number;
    end: number;
    children: TreeNode[];
    // A reference to a node already shown in full elsewhere (the parse tree is a
    // DAG: an ambiguous sub-forest can be shared between parents). Reference nodes
    // carry no children and link back to the definition instead of re-expanding it.
    ref?: boolean;
    // Unique per row. The definition uses `n${id}`; each reference gets its own
    // `ref-${id}-${n}`, so selecting a reference highlights only that row, not the
    // definition that shares its node id.
    key: string;
  }

  // A node of the interactive s-expression, mirroring to_sexpr's structure. A node
  // shared in the ambiguity DAG carries `shareLabel` at its first occurrence (printed
  // `#N=`) and becomes a `ref` (printed `#N#`) at later occurrences.
  interface SexprNode {
    id: number;
    label: string;
    start: number;
    end: number;
    children: SexprNode[];
    shareLabel?: number;
    ref?: number;
  }

  interface Props {
    backend: ParserBackend | null;
    parserName: string | null;
    buildStatus: "none" | "success" | "error";
    nonterminals: string[];
    startNonterminal: string | null;
    inputText: string;
    leftPanelWidth: number;
    // Split orientation. "horizontal" (default) places the input panel left of the
    // result and sizes it by width; "vertical" stacks input over result and sizes
    // it by height. leftPanelWidth is the input panel's size along the split axis
    // in both cases.
    orientation?: "horizontal" | "vertical";
    // Which result views to expose as tabs, in order; a single view hides the tab
    // bar. Defaults to all three.
    views?: Array<"tree" | "graph" | "sexpr">;
    // "monaco" (default) uses the full editor; "plain" swaps in a lightweight
    // syntax-highlighted textarea for a smaller footprint (embedded widgets).
    editor?: "monaco" | "plain";
    // Show the "Export as PNG" control in the graph toolbar. True by default;
    // the web viewer sets it false.
    exportable?: boolean;
    isProfiling?: boolean;
    // Host-specific hooks. Terrarium passes these; the web viewer omits them, so the
    // corresponding chrome (status bar, output log, profiling, graph pop-out) is dropped.
    onStatus?: (message: string, type?: "info" | "error" | "success", tooltip?: string) => void;
    // Fired at the start of each parse, before the result is known, so the host
    // can clear transient per-parse UI (Terrarium clears the unexpected-error banner).
    onParseStart?: () => void;
    onLogCommand?: (cmd: string) => void;
    onLogOutput?: (text: string) => void;
    onLogError?: (text: string) => void;
    // Fired when the parser crashed rather than failing to parse, so the host can
    // surface an unexpected-error notice (Terrarium shows a banner with a way to
    // save the log). The web viewer omits it and falls back to the status hook.
    onUnexpectedError?: (detail: string) => void;
    onProfile?: () => void;
    onPopOut?: () => void;
    // Exports the parse-tree graph as a PNG. Defaults to a browser download;
    // Terrarium injects a native save dialog.
    onExportPng?: (graph: cytoscape.Core | null, defaultName: string) => void;
    // Fired when the visible parse tree changes, so a host that popped the graph
    // out into its own window can re-send the data.
    onParseTreeChange?: () => void;
    startVerticalDrag?: (e: MouseEvent) => void;
    // Cursor, scroll, and selection of the input editor, threaded through so the
    // host can preserve it across mode switches that remount this component.
    initialInputViewState?: monaco.editor.ICodeEditorViewState | null;
    onInputViewState?: (state: monaco.editor.ICodeEditorViewState | null) => void;
  }

  let {
    backend,
    parserName,
    buildStatus,
    nonterminals,
    startNonterminal = $bindable(null),
    inputText = $bindable(""),
    leftPanelWidth,
    orientation = "horizontal",
    views = ["tree", "graph", "sexpr"],
    editor = "monaco",
    exportable = true,
    isProfiling = false,
    onStatus,
    onParseStart,
    onLogCommand,
    onLogOutput,
    onLogError,
    onUnexpectedError,
    onProfile,
    onPopOut,
    onExportPng,
    onParseTreeChange,
    startVerticalDrag,
    initialInputViewState,
    onInputViewState,
  }: Props = $props();

  // Convert the (already display-transformed) flat DAG to a hierarchical tree.
  // Returns the root plus a parent map: each node's parent in the tree as
  // actually rendered, so the reveal walk expands exactly the chain that leads
  // to where a node is shown in full.
  function buildTree(parseTree: ParseTree): { root: TreeNode | null; parentMap: Map<number, number> } {
    const parentMap = new Map<number, number>();
    if (parseTree.nodes.length === 0) return { root: null, parentMap };

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
    if (!rootNode) return { root: null, parentMap };

    // Build node lookup for efficient access
    const nodeMap = new Map(parseTree.nodes.map(n => [n.id, n]));

    // A node reachable from several parents (shared in the ambiguity DAG) is shown
    // in full at its first occurrence; later occurrences become reference nodes so
    // the shared sub-forest is not re-expanded into a tree.
    const seen = new Set<number>();
    let refCounter = 0;

    // Build tree recursively. `parentId` is the node this subtree hangs under,
    // or null at the root.
    function buildSubtree(nodeId: number, parentId: number | null): TreeNode {
      const node = nodeMap.get(nodeId)!;
      if (seen.has(nodeId)) {
        return { id: node.id, label: node.label, kind: node.kind, start: node.start, end: node.end, children: [], ref: true, key: `ref-${nodeId}-${refCounter++}` };
      }
      seen.add(nodeId);
      if (parentId !== null) parentMap.set(node.id, parentId);
      const childIds = childrenMap.get(nodeId) || [];
      return {
        id: node.id,
        label: node.label,
        kind: node.kind,
        start: node.start,
        end: node.end,
        children: childIds.map(childId => buildSubtree(childId, nodeId)),
        key: `n${node.id}`,
      };
    }

    return { root: buildSubtree(rootNode.id, null), parentMap };
  }

  // Outermost Amb spans. An Amb is outermost when no enclosing parse-tree
  // node is also an Amb. Used to drive Monaco warning markers without
  // doubling up for nested ambiguities under the same span.
  function collectOutermostAmbs(parseTree: ParseTree): { start: number; end: number; message: string }[] {
    if (parseTree.nodes.length === 0) return [];
    const childrenMap = new Map<number, number[]>();
    const hasParent = new Set<number>();
    for (const edge of parseTree.edges) {
      if (!childrenMap.has(edge.src)) childrenMap.set(edge.src, []);
      childrenMap.get(edge.src)!.push(edge.dest);
      hasParent.add(edge.dest);
    }
    const nodeMap = new Map(parseTree.nodes.map(n => [n.id, n]));
    const root = parseTree.nodes.find(n => !hasParent.has(n.id));
    if (!root) return [];
    const out: { start: number; end: number; message: string }[] = [];
    function visit(id: number) {
      const node = nodeMap.get(id)!;
      if (node.kind === "Amb") {
        const childIds = childrenMap.get(id) ?? [];
        const childLabel = childIds.length > 0
          ? nodeMap.get(childIds[0])?.label ?? "node"
          : "node";
        out.push({
          start: node.start,
          end: node.end,
          message: `Ambiguous ${childLabel}: ${childIds.length} derivations`,
        });
        return;
      }
      for (const childId of childrenMap.get(id) ?? []) visit(childId);
    }
    visit(root.id);
    return out;
  }

  // Tabs: the tree view, the graph view, and the interactive s-expression.
  // Initial view only; `views` is fixed for the component's life (set from the
  // host once), so capture it without making activeTab track it.
  let activeTab = $state<"tree" | "graph" | "sexpr">(untrack(() => views[0] ?? "tree"));

  // Show spans on tree rows and in graph labels. False by default, driven by the
  // "Show spans" checkbox in the View popover. Label-only, so unlike the structural
  // toggles below it does not pass through buildDisplayGraph.
  let showSpans = $state(false);

  // Whether the View-options popover (the presentation toggles) is open.
  let viewMenuOpen = $state(false);

  // Presentation toggles, named and defaulted like the runtime's `DisplayOptions`
  // and the parser's `--show-*` flags. All false is the simplified view: layout
  // hidden, empties dropped, wrappers spliced.
  let displayOptions = $state<DisplayOptions>({
    showLayout: false,
    showEmpty: false,
    showWrappers: false,
  });

  // Parse Tree data: the raw DAG from to_json, and the display DAG the three
  // tabs render (layout/empties/wrappers resolved). displayOptions drives the
  // transform, so toggling is instant, no re-parse.
  let parseTree = $state<ParseTree | null>(null);
  let displayTree = $derived(parseTree ? buildDisplayGraph(parseTree, displayOptions) : null);
  let ambiguityWarnings = $derived(parseTree ? collectOutermostAmbs(parseTree) : []);
  // svelte-ignore non_reactive_update
  let parseTreeContainer: HTMLDivElement;
  let parseTreeCy: cytoscape.Core | null = null;
  const parseTreeCollapseManager = new GraphCollapseManager();
  // The Cytoscape instance is kept alive across tab switches and reused on most
  // parses. `graphDirty` marks its elements stale (new parse or a view-options/
  // span toggle), so the next time the graph tab is shown it reloads instead of
  // just resizing. A plain flag, not reactive: the graph $effect reacts to activeTab.
  let graphDirty = true;

  // Signature of what the graph last rendered (node/edge set + labels + showSpans).
  // A View toggle only rebuilds when this changes, so a no-op toggle — e.g. hiding
  // empty nodes when there are none — doesn't redraw and flicker. Set on every build.
  let lastGraphSig: string | null = null;

  // The renderer the live parse-tree graph was built with (`{ webgl }`), read off
  // the instance for the WebGL badge. Null until the graph is first built.
  let graphRenderer = $state<{ webgl: boolean } | null>(null);

  // Parse tree node selection (for highlighting span in input)
  let parseTreeSelectedSpan = $state<{ start: number; end: number } | null>(null);
  let parseTreeSelectedNodeId = $state<string | null>(null);
  // The selected tree row, keyed per row (TreeNode.key) so a reference and its
  // definition (which share a node id) highlight independently. Cross-view
  // highlighting (input span, graph node) still keys on parseTreeSelectedNodeId.
  let selectedTreeRowKey = $state<string | null>(null);

  // The tree view plus its parent map, rebuilt whenever the display tree changes.
  // The reveal walk reads the parent map (also from the s-expr and graph tabs),
  // so it travels with the tree rather than living in a side-effect-filled map.
  let treeBuild = $derived(displayTree ? buildTree(displayTree) : null);
  let treeRoot = $derived(treeBuild?.root ?? null);
  let expandedNodes = $state(new Set<number>());
  // svelte-ignore non_reactive_update
  let treeContainerEl: HTMLDivElement;
  // svelte-ignore non_reactive_update
  let inputEditorRef: { focus: () => void } | undefined;

  // Load the Monaco-based editor on demand: hosts that use editor="plain" (the
  // embedded widgets) then never fetch Monaco at all. PlainEditor stays bundled
  // statically since it is tiny.
  let InputEditorComp = $state<typeof import("./InputEditor.svelte").default | null>(null);
  $effect(() => {
    if (editor === "monaco" && !InputEditorComp) {
      import("./InputEditor.svelte").then((m) => (InputEditorComp = m.default));
    }
  });

  // Interactive s-expression, derived from the same display tree as the other tabs.
  let sexprRoot = $derived(displayTree ? buildSexprModel(displayTree) : null);
  // The node whose subtree was last copied, cleared on a timer to flash the icon.
  let copiedSexprKey = $state<number | null>(null);
  // The s-expression node currently under the pointer. Only its row mounts the
  // copy button, so we never instantiate one lucide icon per node.
  let hoveredSexprId = $state<number | null>(null);
  // Collapsed s-expression nodes (empty = fully expanded). Kept separate from the
  // tree's expandedNodes so the s-expression reads fully expanded by default.
  let sexprCollapsed = $state(new Set<number>());
  // svelte-ignore non_reactive_update
  let sexprContainerEl: HTMLDivElement;

  // Parse error info for input editor markers
  let parseErrorInfo = $state<{ line: number; column: number; message: string } | null>(null);

  let parseTreeTooltipCleanup: (() => void) | null = null;

  function buildGraphElements() {
    return buildParseTreeElements(displayTree!, showSpans);
  }

  // A cheap fingerprint of what the graph renders: the node set (id/kind/label),
  // the edge set, and showSpans (which changes labels). Two toggles that leave all
  // of these equal produce the same drawing, so we can skip the rebuild.
  function graphSignature(): string {
    if (!displayTree) return "";
    const nodes = displayTree.nodes
      .map((n) => `${n.id}:${n.kind}:${n.label}`)
      .join(",");
    const edges = displayTree.edges.map((e) => `${e.src}>${e.dest}`).join(",");
    return `${showSpans ? 1 : 0}|${nodes}|${edges}`;
  }

  // Shared by every View toggle: rebuild and re-fit the graph only when the toggle
  // actually changes what it draws. A no-op toggle (e.g. hiding empty nodes when
  // there are none) leaves the signature unchanged, so nothing redraws or flickers.
  function refreshGraphAfterToggle() {
    if (!parseTree || graphSignature() === lastGraphSig) return;
    if (activeTab === "graph") rebuildAndFitGraph();
    else graphDirty = true; // rebuild lazily when the graph tab is next shown
  }


  // Create the Cytoscape instance once and wire its event handlers. The handlers
  // are delegated on the instance (by 'node'/'edge' selector), so they keep
  // working across the element swaps reloadGraph does — no need to re-bind.
  function buildGraph() {
    if (!parseTree || !parseTreeContainer) return;
    parseTreeCollapseManager.reset();

    parseTreeCy = createGraph({
      container: parseTreeContainer,
      elements: buildGraphElements(),
      styles: [...sppfNodeStyles, ...edgeStyles],  // Reuse SPPF styles (nonterminal/token)
      layout: 'tree',  // cytoscape-tidytree (see PARSE_TREE_LAYOUT)
    });
    graphRenderer = parseTreeCy.scratch("_renderer");

    parseTreeCollapseManager.setCy(parseTreeCy);
    parseTreeTooltipCleanup = setupGraphTooltip(parseTreeCy, parseTreeContainer);

    // Double-click: collapse/expand
    parseTreeCy.on('dbltap', 'node', (event) => {
      parseTreeCollapseManager.toggleCollapse(event.target.id());
    });

    // Click a node: select it, highlight its span and outgoing edges
    parseTreeCy.on('tap', 'node', (event) => {
      const node = event.target;
      const start = node.data('start');
      const end = node.data('end');
      if (start !== undefined && end !== undefined) {
        parseTreeSelectedSpan = { start, end };
      }
      if (parseTreeSelectedNodeId) {
        parseTreeCy?.getElementById(parseTreeSelectedNodeId).removeClass('selected');
      }
      if (parseTreeCy) {
        clearEdgeHighlights(parseTreeCy);
        parseTreeSelectedNodeId = node.id();
        node.addClass('selected');
        highlightOutgoingEdges(parseTreeCy, node.id());
      }
    });

    // Click the background: clear selection
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

    // Click an edge: highlight it
    parseTreeCy.on('tap', 'edge', (event) => {
      const edge = event.target;
      if (parseTreeSelectedNodeId && parseTreeCy) {
        parseTreeCy.getElementById(parseTreeSelectedNodeId).removeClass('selected');
        parseTreeSelectedNodeId = null;
      }
      parseTreeSelectedSpan = null;
      if (parseTreeCy) highlightClickedEdge(parseTreeCy, edge.id());
    });

    graphDirty = false;
  }

  // Swap fresh elements onto the existing instance and relayout, reusing the same
  // Cytoscape instance — no teardown, so repeated toggles don't churn it. The
  // caller frames the view (rebuildAndFitGraph fits to the panel); here we only
  // rebuild, re-apply the current selection, and force a redraw.
  function reloadGraph() {
    if (!parseTreeCy || !parseTree) return;
    parseTreeCollapseManager.reset();

    parseTreeCy.elements().remove();
    parseTreeCy.add(buildGraphElements());
    parseTreeCy.layout({ ...PARSE_TREE_LAYOUT, fit: true } as any).run();

    // Re-apply the cross-view selection to the matching node, if it survived.
    if (parseTreeSelectedNodeId) {
      const node = parseTreeCy.getElementById(parseTreeSelectedNodeId);
      if (node.length > 0) {
        node.addClass('selected');
        highlightOutgoingEdges(parseTreeCy, parseTreeSelectedNodeId);
      }
    }
    // On the WebGL path (a large tree, see PARSE_TREE_WEBGL_NODE_THRESHOLD) a
    // relayout doesn't repaint edge buffers until the next viewport change, so
    // the edges vanish until you pan or select. resize() forces a full redraw and
    // brings them back immediately. Harmless on the Canvas2D path.
    parseTreeCy.resize();
    graphDirty = false;
  }

  // Destroy the parse-tree graph instance and its container wheel listener (which
  // cy.destroy() does not remove). Called on a parse that crosses the renderer
  // threshold so the next build re-picks Canvas2D vs WebGL; toggles and tab
  // switches reuse the instance instead.
  function teardownParseTreeGraph() {
    if (!parseTreeCy) return;
    parseTreeCy.scratch("_disposeWheel")?.();
    parseTreeTooltipCleanup?.();
    parseTreeTooltipCleanup = null;
    parseTreeCy.destroy();
    parseTreeCy = null;
    graphRenderer = null;
  }

  // Build the instance the first time, reload it afterwards.
  function loadGraph() {
    if (parseTreeCy) reloadGraph();
    else buildGraph();
  }

  // Rebuild the graph's elements for the current view and fit the whole tree to
  // the panel. This is the fresh-parse framing, shared by the graph $effect and
  // every View toggle so a toggle reframes exactly like a new parse. The rAF
  // re-fit is needed because a just-unhidden flex pane hasn't reached its final
  // size on the first tick, so the initial fit would land on a too-small box.
  function rebuildAndFitGraph() {
    loadGraph();
    lastGraphSig = graphSignature();
    requestAnimationFrame(() => {
      if (parseTreeCy && activeTab === "graph") {
        parseTreeCy.resize();
        resetView();
      }
    });
  }

  // The graph view stays mounted across tab switches, so the instance is built
  // once per parse rather than rebuilt on every visit. When the graph tab is
  // shown: reload if its elements are stale, otherwise just resize (the container
  // was display:none while hidden, so Cytoscape needs to re-read its size).
  $effect(() => {
    if (activeTab === "graph" && parseTree) {
      tick().then(() => {
        if (!parseTreeContainer) return;
        // Stale elements (new parse or a view toggle) rebuild and re-fit to the
        // panel; revisiting the tab with fresh elements only resizes, keeping the
        // user's zoom/pan.
        if (graphDirty || !parseTreeCy) rebuildAndFitGraph();
        else parseTreeCy.resize();
      });
    }
  });

  // The shared control strip (createGraphControls) renders into the graph
  // container while a tree is shown, so the buttons match the tree widget
  // and any other host. Pop out appears only when the host provides the hook.
  $effect(() => {
    if (!parseTree || !parseTreeContainer) return;
    return createGraphControls(parseTreeContainer, {
      zoomIn,
      zoomOut,
      fit: resetView,
      expandAll,
      exportPng: exportable ? exportGraph : undefined,
      popOut: onPopOut,
    });
  });

  // Sequence number of the newest parse. Parses run concurrently, so without it
  // a slower earlier parse can land after a faster later one and leave the views
  // showing a tree for input the user has already edited.
  let parseSeq = 0;

  // Exported so the page-level Cmd+P keybinding can fire the same parse via bind:this.
  export async function parse() {
    if (!backend || buildStatus !== "success" || !startNonterminal) return;
    const seq = ++parseSeq;
    onStatus?.("Parsing...", "info");
    onParseStart?.();

    // Reset previous results
    parseTree = null;
    parseTreeSelectedSpan = null;
    parseErrorInfo = null;

    onLogCommand?.(`${parserName} <input> --start ${startNonterminal}`);

    const result = await backend.parse(inputText, startNonterminal);
    // A newer parse started while this one ran, and the newer one owns the views.
    if (seq !== parseSeq) return;

    if ("error" in result) {
      // The parser could not be run at all (missing binary, spawn failure): an
      // unexpected error, surfaced like a crash rather than a parse failure.
      reportUnexpectedError(result.error);
      teardownParseTreeGraph();
      return;
    }

    const { output, treeJson } = result;
    if (output.success) {
      parseErrorInfo = null;
      const totalMs = (output.duration_ms ?? 0) + (output.tree_construction_ms ?? 0);
      const durationStr = output.duration_ms != null ? ` (${totalMs}ms)` : "";
      onLogOutput?.(`Parse successful${durationStr}`);
      const tooltip = output.duration_ms != null
        ? `Parse: ${output.duration_ms}ms\nTree construction: ${output.tree_construction_ms ?? '?'}ms`
        : undefined;
      onStatus?.(`Parse successful${durationStr}`, "success", tooltip);
    } else if (output.unexpected_error) {
      // The parser crashed or produced no result; not a parse failure. Surface it
      // honestly and let the host offer to save the log.
      parseErrorInfo = null;
      reportUnexpectedError(output.error ?? "An unexpected error occurred.");
    } else {
      // Expected parse failure: the input does not match the grammar.
      parseErrorInfo = output.error_info ?? null;
      if (output.error) {
        onLogError?.(output.error);
      }
      onStatus?.("Parse failed", "error");
    }

    // An unexpected error means the parser crashed, so any partial tree it wrote
    // is unreliable; do not render it.
    if (treeJson != null && !output.unexpected_error) {
      loadParseTree(treeJson);
    } else {
      // No tree to show (parse failure or crash): clear any tree left from a prior
      // parse so every tab matches the error state instead of showing stale data.
      teardownParseTreeGraph();
    }
  }

  // An error that is not the input failing to match the grammar: the parser
  // crashed, could not be run, or produced output the view cannot read. The host
  // shows a banner offering the log, and no editor markers are set, since there
  // is no position in the input to point at.
  function reportUnexpectedError(detail: string) {
    onLogError?.(detail);
    onStatus?.("An unexpected error occurred", "error");
    onUnexpectedError?.(detail);
  }

  function loadParseTree(json: string) {
    try {
      parseTree = JSON.parse(json) as ParseTree;
      // treeRoot / displayTree recompute reactively off parseTree. New tree:
      // drop any stale selection and mark the graph for reload. The graph
      // $effect rebuilds it when the graph tab is (or becomes) active.
      clearParseModeInputSelection();
      // Re-pick the graph renderer for this parse. It flips only when the node
      // count crosses the threshold, and a live instance can't switch renderer,
      // so tear it down only then; otherwise reloadGraph reuses it. Toggles keep
      // the renderer because they don't run through here.
      const wantWebgl = (displayTree?.nodes.length ?? 0) >= PARSE_TREE_WEBGL_NODE_THRESHOLD;
      if (parseTreeCy && parseTreeCy.scratch("_renderer")?.webgl !== wantWebgl) {
        teardownParseTreeGraph();
      }
      graphDirty = true;
      // Expand root by default
      if (treeRoot) {
        expandedNodes = new Set([treeRoot.id]);
      }
      onParseTreeChange?.();
    } catch (e) {
      // The parser reported success, so a tree it wrote that will not parse is an
      // internal error, not a bad input.
      reportUnexpectedError(`Could not read the parse tree the parser wrote: ${e}`);
      parseTree = null;
      teardownParseTreeGraph();
    }
  }

  // Graph controls (parse-tree graph)
  function zoomIn() {
    adjustZoomGraph(parseTreeCy, 1.2);
  }

  function zoomOut() {
    adjustZoomGraph(parseTreeCy, 1 / 1.2);
  }

  function resetView() {
    resetViewGraph(parseTreeCy);
  }

  function expandAll() {
    parseTreeCollapseManager.expandAll();
  }

  function exportGraph() {
    (onExportPng ?? downloadPng)(parseTreeCy, "parse-tree");
  }

  // Flip one presentation toggle. The tree and s-expression recompute reactively
  // off displayTree; the imperative graph is reloaded here.
  function setDisplayOption(key: keyof DisplayOptions, show: boolean) {
    displayOptions = { ...displayOptions, [key]: show };
    // A toggle can change the root (splicing wrappers unwraps the Start wrapper;
    // showing layout re-wraps it). A root not in expandedNodes renders collapsed,
    // which reads as the whole tree collapsing, so keep the new root expanded.
    // Every other node keeps its expansion: ids are stable across the transform,
    // so expandedNodes still applies to the survivors.
    if (treeRoot) expandedNodes = new Set(expandedNodes).add(treeRoot.id);
    clearParseModeInputSelection();
    // A toggle that changes the node set reframes the graph the same way a fresh
    // parse does (rebuild + fit to the panel); a toggle that changes nothing is a
    // no-op. refreshGraphAfterToggle decides which by comparing the drawing.
    if (parseTree) tick().then(refreshGraphAfterToggle);
    onParseTreeChange?.();
  }

  // The display tree (transformed under the current view options), for the graph
  // pop-out. Exported so a host can feed it to a separate graph window via bind:this.
  export function getParseTreeForPopup(): ParseTree | null {
    return displayTree;
  }

  // Exported so a host can focus the input editor, e.g. on a mode switch.
  export function focusInput() {
    inputEditorRef?.focus();
  }

  // Show or hide spans on tree rows and in graph labels. The tree reflects
  // showSpans reactively; the imperative graph rebuilds and re-fits to the panel
  // through refreshGraphAfterToggle, the same fresh-parse framing every other View
  // toggle uses (and it skips the rebuild when the drawing is unchanged).
  function setShowSpans(show: boolean) {
    showSpans = show;
    if (parseTree) tick().then(refreshGraphAfterToggle);
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
    selectedTreeRowKey = node.key;
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

  // A single click selects a node, a double click toggles its collapse. The
  // dblclick event and e.detail are both unreliable in the WKWebView Tauri runs
  // in, so detect the double click here: a second click on the same node within
  // the threshold toggles. Tracking resets after a toggle, so a click that
  // follows a double click selects rather than re-toggling.
  const DOUBLE_CLICK_MS = 300;
  let lastTreeClick: { id: number; time: number } | null = null;

  function clickTreeNode(node: TreeNode) {
    const now = performance.now();
    if (lastTreeClick && lastTreeClick.id === node.id && now - lastTreeClick.time < DOUBLE_CLICK_MS) {
      lastTreeClick = null;
      if (node.children.length > 0) toggleTreeNode(node.id);
    } else {
      lastTreeClick = { id: node.id, time: now };
      selectTreeNode(node);
    }
  }

  // Smallest-span display-tree node whose half-open range [start, end) covers
  // the offset, i.e. the deepest enclosing node. A click on a token character
  // lands on that token (the smallest covering node); a click in the gap between
  // tokens (whitespace, or a span hidden layout left behind) is covered only by
  // nonterminals, so fall back to the nearest token rather than selecting a wide
  // enclosing parent.
  function findDeepestParseTreeNodeAt(offset: number): ParseTreeNode | null {
    if (!displayTree) return null;
    let best: ParseTreeNode | null = null;
    for (const node of displayTree.nodes) {
      const contains = node.start <= offset && offset < node.end;
      if (contains && (!best || node.end - node.start < best.end - best.start)) {
        best = node;
      }
    }
    if (best && best.kind === "Token") return best;
    let nearestToken: ParseTreeNode | null = null;
    let nearestDistance = Infinity;
    for (const node of displayTree.nodes) {
      if (node.kind !== "Token") continue;
      const distance = offset < node.start ? node.start - offset : offset - node.end;
      if (distance < nearestDistance) {
        nearestDistance = distance;
        nearestToken = node;
      }
    }
    return nearestToken ?? best;
  }

  function parseTreeAncestorsOf(nodeId: number): number[] {
    const parentMap = treeBuild?.parentMap;
    if (!parentMap) return [];
    const ancestors: number[] = [];
    let cur = parentMap.get(nodeId);
    while (cur !== undefined) {
      ancestors.push(cur);
      cur = parentMap.get(cur);
    }
    return ancestors;
  }

  // Briefly flash a tree row. Used on reveal so the jump lands visibly even when
  // the target row was already selected.
  function flashTreeRow(el: Element) {
    el.classList.remove('flash');
    // Force a reflow so the animation restarts on a repeated reveal of the same row.
    void (el as HTMLElement).offsetWidth;
    el.classList.add('flash');
    el.addEventListener('animationend', () => el.classList.remove('flash'), { once: true });
  }

  // Reveal the given parse tree node in both views: expand collapsed ancestors,
  // set selection, scroll the tree row into view, pan the graph node into view.
  function revealParseTreeNode(node: ParseTreeNode) {
    const ancestors = parseTreeAncestorsOf(node.id);

    // Tree view: ensure every ancestor is expanded so the row is visible.
    if (ancestors.length > 0) {
      const next = new Set(expandedNodes);
      for (const id of ancestors) next.add(id);
      expandedNodes = next;
    }

    // S-expression view: un-collapse every ancestor so the node is shown.
    if (ancestors.length > 0) {
      const next = new Set(sexprCollapsed);
      for (const id of ancestors) next.delete(id);
      sexprCollapsed = next;
    }

    const cyNodeId = `n${node.id}`;
    if (parseTreeCy && parseTreeSelectedNodeId) {
      parseTreeCy.getElementById(parseTreeSelectedNodeId).removeClass('selected');
    }
    parseTreeSelectedSpan = { start: node.start, end: node.end };
    parseTreeSelectedNodeId = cyNodeId;
    // Reveal targets the node itself, which is rendered at its definition row.
    selectedTreeRowKey = cyNodeId;

    // Scroll the tree row into view once the expanded-state update has rendered,
    // then flash it. The flash is the only cue when the target was already
    // selected, since the selection highlight does not change in that case.
    tick().then(() => {
      if (treeContainerEl) {
        const selectedEl = treeContainerEl.querySelector('.tree-item.selected');
        if (selectedEl) {
          selectedEl.scrollIntoView({ block: 'nearest' });
          flashTreeRow(selectedEl);
        }
      }
      if (sexprContainerEl) {
        const el = sexprContainerEl.querySelector('.sexpr-node.selected');
        if (el) {
          el.scrollIntoView({ block: 'nearest' });
          flashTreeRow(el);
        }
      }
    });

    // Graph view: only when it's the active tab. The instance now persists while
    // hidden, but its container is display:none (zero size), so the fit math
    // below would divide by zero. Centering applies when the graph is visible.
    if (parseTreeCy && activeTab === "graph") {
      parseTreeCollapseManager.expandAncestors(cyNodeId);
      clearEdgeHighlights(parseTreeCy);
      const cyNode = parseTreeCy.getElementById(cyNodeId);
      if (cyNode.length > 0) {
        cyNode.addClass('selected');
        highlightOutgoingEdges(parseTreeCy, cyNodeId);
        // Always center on the selected node; pick the zoom from a
        // hypothetical fit on the node plus a few ancestors, then clamp to
        // a comfortable range. This keeps the node in the viewport center
        // while letting the zoom level adapt to the local context size.
        const ANCESTOR_COUNT = 4;
        const MIN_ZOOM = 0.6;
        const MAX_ZOOM = 2.0;
        const padding = 80;
        const focusEles = cyNode.union(
          cyNode.predecessors('node').slice(0, ANCESTOR_COUNT),
        );
        const bb = focusEles.boundingBox();
        const containerW = parseTreeContainer.clientWidth;
        const containerH = parseTreeContainer.clientHeight;
        const fitZoom = Math.min(
          (containerW - 2 * padding) / bb.w,
          (containerH - 2 * padding) / bb.h,
        );
        const targetZoom = Math.max(MIN_ZOOM, Math.min(fitZoom, MAX_ZOOM));
        parseTreeCy.animate(
          { zoom: targetZoom, center: { eles: cyNode } },
          { duration: 300 },
        );
      }
    }
  }

  function onParseInputClick(offset: number) {
    const node = findDeepestParseTreeNodeAt(offset);
    if (node) revealParseTreeNode(node);
  }

  // Drop any input-side highlight driven by a tree or graph selection. Triggered
  // when the user edits the input or presses Esc, so stale highlights do not
  // linger across content changes.
  function clearParseModeInputSelection() {
    if (parseTreeSelectedNodeId && parseTreeCy) {
      parseTreeCy.getElementById(parseTreeSelectedNodeId).removeClass('selected');
    }
    parseTreeSelectedSpan = null;
    parseTreeSelectedNodeId = null;
    selectedTreeRowKey = null;
    if (parseTreeCy) clearEdgeHighlights(parseTreeCy);
  }

  // Escape clears the node selection (and the input highlight it drives) wherever
  // focus sits — the tree, the s-expression, the graph, or the editor. The editor's
  // own onescape covers the in-editor case; this covers selections made elsewhere.
  function handleParseViewKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && viewMenuOpen) {
      viewMenuOpen = false;
      return;
    }
    if (e.key === "Escape" && (parseTreeSelectedNodeId || parseTreeSelectedSpan || selectedTreeRowKey)) {
      clearParseModeInputSelection();
    }
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

  // Build the interactive s-expression model from the (already transformed)
  // display tree, mirroring the generated to_sexpr: a node reachable from several
  // parents (indegree > 1 in the rendered DAG) is written once with a `#N=` label,
  // and later occurrences become `#N#` refs.
  function buildSexprModel(parseTree: ParseTree): SexprNode | null {
    const childrenMap = new Map<number, number[]>();
    const indegree = new Map<number, number>();
    for (const edge of parseTree.edges) {
      if (!childrenMap.has(edge.src)) childrenMap.set(edge.src, []);
      childrenMap.get(edge.src)!.push(edge.dest);
      indegree.set(edge.dest, (indegree.get(edge.dest) ?? 0) + 1);
    }
    const root = parseTree.nodes.find(n => !indegree.has(n.id));
    if (!root) return null;
    const nodeMap = new Map(parseTree.nodes.map(n => [n.id, n]));

    const labels = new Map<number, number>();
    let nextLabel = 1;
    const expanded = new Set<number>();

    function build(id: number): SexprNode {
      const node = nodeMap.get(id)!;
      const shared = (indegree.get(id) ?? 0) > 1;
      if (shared && expanded.has(id)) {
        return { id, label: node.label, start: node.start, end: node.end, children: [], ref: labels.get(id) };
      }
      let shareLabel: number | undefined;
      if (shared) {
        shareLabel = nextLabel++;
        labels.set(id, shareLabel);
        expanded.add(id);
      }
      const childIds = childrenMap.get(id) ?? [];
      return { id, label: node.label, start: node.start, end: node.end, children: childIds.map(build), shareLabel };
    }

    return build(root.id);
  }

  // Serialize an s-expression node to text, matching the generated to_sexpr:
  // the closing paren hugs the last child, an all-leaf node prints on one line,
  // and a node shared in the ambiguity DAG carries a `#N=` / `#N#` datum label.
  function sexprToText(node: SexprNode): string {
    return sexprContent(node, 0) + "\n";
  }

  // The node's text with no leading indent and no trailing newline; nested lines
  // carry their own indent. A ref points at a node that has children, so it never
  // counts as a leaf and always forces its parent to break.
  function sexprContent(node: SexprNode, indent: number): string {
    if (node.ref !== undefined) return `#${node.ref}#`;
    const prefix = node.shareLabel !== undefined ? `#${node.shareLabel}=` : "";
    if (node.children.length === 0) return `${prefix}${node.label}`;
    if (node.children.every((c) => c.children.length === 0 && c.ref === undefined)) {
      const parts = node.children.map((c) => sexprContent(c, indent));
      return `${prefix}(${node.label} ${parts.join(" ")})`;
    }
    const pad = " ".repeat(indent + 2);
    let s = `${prefix}(${node.label}`;
    for (const child of node.children) s += `\n${pad}${sexprContent(child, indent + 2)}`;
    return s + ")";
  }

  function selectSexprNode(node: SexprNode) {
    if (node.ref !== undefined) return;
    parseTreeSelectedSpan = { start: node.start, end: node.end };
    parseTreeSelectedNodeId = `n${node.id}`;
    selectedTreeRowKey = `n${node.id}`;
  }

  // Single click selects, double click toggles collapse; see clickTreeNode for
  // why the double click is detected here instead of via dblclick / e.detail.
  let lastSexprClick: { id: number; time: number } | null = null;

  function clickSexprNode(node: SexprNode) {
    const now = performance.now();
    const internal = node.ref === undefined && node.children.length > 0;
    if (lastSexprClick && lastSexprClick.id === node.id && now - lastSexprClick.time < DOUBLE_CLICK_MS) {
      lastSexprClick = null;
      if (internal) toggleSexprNode(node.id);
    } else {
      lastSexprClick = { id: node.id, time: now };
      selectSexprNode(node);
    }
  }

  function toggleSexprNode(id: number) {
    if (sexprCollapsed.has(id)) {
      sexprCollapsed.delete(id);
    } else {
      sexprCollapsed.add(id);
    }
    sexprCollapsed = new Set(sexprCollapsed); // Trigger reactivity
  }

  function expandAllSexprNodes() {
    sexprCollapsed = new Set();
  }

  function collapseAllSexprNodes() {
    const collapsed = new Set<number>();
    function collect(node: SexprNode) {
      if (node.ref === undefined && node.children.length > 0) {
        collapsed.add(node.id);
        node.children.forEach(collect);
      }
    }
    if (sexprRoot) collect(sexprRoot);
    sexprCollapsed = collapsed;
  }

  async function copySexprNode(node: SexprNode) {
    await navigator.clipboard.writeText(sexprToText(node).trimEnd());
    copiedSexprKey = node.id;
    setTimeout(() => {
      if (copiedSexprKey === node.id) copiedSexprKey = null;
    }, 1200);
  }
</script>

<svelte:window onkeydown={handleParseViewKeydown} onclick={() => viewMenuOpen = false} />

<div class="main-content" class:vertical={orientation === "vertical"}>
  <!-- Left Panel -->
  <div class="left-panel" style={orientation === "vertical" ? `height: ${leftPanelWidth}px` : `width: ${leftPanelWidth}px`}>
    <!-- Header: shown only when it has content — a start chooser (more than one
         start nonterminal) or a Profile action. A single-start grammar needs no
         chooser, and Parse now lives in the input pane, so the header collapses. -->
    {#if nonterminals.length > 1 || onProfile}
    <div class="header">
      {#if nonterminals.length > 1}
        <div class="dropdown-wrapper">
          <span class="dropdown-label">Start:</span>
          <NonterminalPicker
            bind:value={startNonterminal}
            options={nonterminals}
            disabled={!backend || nonterminals.length === 0}
          />
        </div>
      {/if}
      {#if onProfile}
        <div class="parse-actions">
          <button class="parse-btn" onclick={onProfile} disabled={!backend || buildStatus !== "success" || !startNonterminal || isProfiling}>
            {isProfiling ? "Profiling..." : "Profile"}
          </button>
        </div>
      {/if}
      <!-- Parse trigger lives on the header bar when there is one, using its empty
           right side, rather than floating over the editor. -->
      <button
        class="parse-play parse-play--bar"
        onclick={parse}
        disabled={!backend || buildStatus !== "success" || !startNonterminal}
        title="Parse"
        aria-label="Parse"
      >
        <Play size={16} fill="currentColor" strokeWidth={0} />
      </button>
    </div>
    {/if}

    <!-- Input Area -->
    <div class="input-section">
      {#if editor === "plain"}
        <PlainEditor
          bind:this={inputEditorRef}
          bind:value={inputText}
          highlightSpan={parseTreeSelectedSpan}
          onclick={onParseInputClick}
          onchange={clearParseModeInputSelection}
          onescape={clearParseModeInputSelection}
          placeholder="Enter code to parse..."
        />
      {:else if InputEditorComp}
        <InputEditorComp
          bind:this={inputEditorRef}
          bind:value={inputText}
          error={parseErrorInfo}
          ambiguities={ambiguityWarnings}
          highlightSpan={parseTreeSelectedSpan}
          onclick={onParseInputClick}
          onchange={clearParseModeInputSelection}
          onescape={clearParseModeInputSelection}
          placeholder="Enter code to parse..."
          initialViewState={initialInputViewState}
          onSaveViewState={onInputViewState}
        />
      {/if}
      <!-- Parse trigger: floats in the input pane's corner only when there is no
           header bar to host it (a single-start grammar with no Profile action). -->
      {#if !(nonterminals.length > 1 || onProfile)}
      <button
        class="parse-play"
        onclick={parse}
        disabled={!backend || buildStatus !== "success" || !startNonterminal}
        title="Parse"
        aria-label="Parse"
      >
        <Play size={16} fill="currentColor" strokeWidth={0} />
      </button>
      {/if}
    </div>
  </div>

  <!-- Vertical Resize Handle -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="resize-handle-vertical" onmousedown={startVerticalDrag}></div>

  <!-- Right Panel -->
  <div class="right-panel">
    <div class="graph-section">
      {#if views.length > 1}
      <div class="tabs">
        {#if views.includes("tree")}<button class:active={activeTab === "tree"} onclick={() => (activeTab = "tree")}>Tree</button>{/if}
        {#if views.includes("graph")}<button class:active={activeTab === "graph"} onclick={() => (activeTab = "graph")}>Graph</button>{/if}
        {#if views.includes("sexpr")}<button class:active={activeTab === "sexpr"} onclick={() => (activeTab = "sexpr")}>S-expr</button>{/if}
        {#if parseTree}
          <div class="view-options">
            <button
              class="view-options-btn"
              class:active={viewMenuOpen}
              onclick={(e) => { e.stopPropagation(); viewMenuOpen = !viewMenuOpen; }}
              title="View options"
            >
              <SlidersHorizontal size={14} />
              <span>View</span>
              <ChevronDown size={12} />
            </button>
            {#if viewMenuOpen}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="view-options-menu" onclick={(e) => e.stopPropagation()}>
                {#if parseTree.layout_name}
                  <label>
                    <input type="checkbox" checked={displayOptions.showLayout}
                      onchange={(e) => setDisplayOption("showLayout", e.currentTarget.checked)} />
                    Show layout
                  </label>
                {/if}
                <label>
                  <input type="checkbox" checked={displayOptions.showEmpty}
                    onchange={(e) => setDisplayOption("showEmpty", e.currentTarget.checked)} />
                  Show empty nodes
                </label>
                <label>
                  <input type="checkbox" checked={displayOptions.showWrappers}
                    onchange={(e) => setDisplayOption("showWrappers", e.currentTarget.checked)} />
                  Show wrappers
                </label>
                <label>
                  <input type="checkbox" checked={showSpans}
                    onchange={(e) => setShowSpans(e.currentTarget.checked)} />
                  Show spans
                </label>
              </div>
            {/if}
          </div>
        {/if}
      </div>
      {/if}
      <div class="graph-container">
        {#if activeTab === "tree"}
          {#if treeRoot}
            <div class="tree-view">
              <div class="tree-controls">
                <button onclick={expandAllTreeNodes} title="Expand All">
                  <Expand size={16} />
                </button>
                <button onclick={collapseAllTreeNodes} title="Collapse All">
                  <Minimize2 size={16} />
                </button>
              </div>
              <!-- The container takes arrow-key navigation over its rows, but the
                   rows are not ARIA treeitems, so role="tree" here would announce
                   a tree with no items. -->
              <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="tree-container" tabindex="0" onkeydown={handleTreeKeydown} bind:this={treeContainerEl}>
                {#snippet treeNode(node: TreeNode, depth: number)}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div
                    class="tree-item"
                    class:selected={selectedTreeRowKey === node.key}
                    style="padding-left: {depth * 16 + 8}px"
                    onmousedown={(e) => { if (e.detail > 1) e.preventDefault(); }}
                    onclick={() => clickTreeNode(node)}
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
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span
                      class="tree-label"
                      class:nonterminal={node.kind === "Nonterminal"}
                      class:token={node.kind === "Token"}
                      class:amb={node.kind === "Amb"}
                      class:reference={node.ref}
                      title={node.ref ? "Jump to definition" : undefined}
                      onclick={node.ref ? (e) => { e.stopPropagation(); revealParseTreeNode(node); } : undefined}
                    >
                      {node.label}{#if node.ref}<span class="tree-ref-icon"><CornerRightUp size={13} /></span>{/if}
                    </span>
                    {#if showSpans}
                      <span class="tree-span">[{node.start}:{node.end}]</span>
                    {/if}
                  </div>
                  {#if expandedNodes.has(node.id)}
                    {#each node.children as child}
                      {@render treeNode(child, depth + 1)}
                    {/each}
                  {/if}
                {/snippet}
                {@render treeNode(treeRoot, 0)}
              </div>
            </div>
          {/if}
        {:else if activeTab === "sexpr"}
          {#if sexprRoot}
            <div class="sexpr-view">
              <div class="tree-controls">
                <button onclick={expandAllSexprNodes} title="Expand All">
                  <Expand size={16} />
                </button>
                <button onclick={collapseAllSexprNodes} title="Collapse All">
                  <Minimize2 size={16} />
                </button>
              </div>
              <!-- svelte-ignore a11y_mouse_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="sexpr-scroll"
                bind:this={sexprContainerEl}
                onmouseover={(e) => {
                  const line = (e.target as HTMLElement).closest(".sexpr-line");
                  hoveredSexprId = line ? Number((line as HTMLElement).dataset.sid) : null;
                }}
                onmouseleave={() => hoveredSexprId = null}
              >
              {#snippet sexprNode(node: SexprNode, indent: number, trailing: string)}
                {@const internal = node.ref === undefined && node.children.length > 0}
                {@const collapsed = sexprCollapsed.has(node.id)}
                {@const expanded = internal && !collapsed}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="sexpr-line" data-sid={node.id} style="padding-left: {indent * 9 + 8}px">
                  {#if internal}
                    <span class="sexpr-toggle" onclick={() => toggleSexprNode(node.id)}>
                      <span class="sexpr-caret" class:expanded={!collapsed}></span>
                    </span>
                  {:else}
                    <span class="sexpr-toggle-placeholder"></span>
                  {/if}
                  {#if node.ref !== undefined}
                    <span class="sexpr-token reference">#{node.ref}#</span>{#if trailing}<span class="sexpr-paren">{trailing}</span>{/if}
                  {:else}
                    <span
                      class="sexpr-node"
                      class:selected={parseTreeSelectedNodeId === `n${node.id}`}
                      onmousedown={(e) => { if (e.detail > 1) e.preventDefault(); }}
                      onclick={() => clickSexprNode(node)}
                    >
                      {#if node.shareLabel !== undefined}<span class="sexpr-label">#{node.shareLabel}=</span>{/if}{#if internal}<span class="sexpr-paren">(</span>{/if}<span class="sexpr-token" class:amb={node.label === "Amb"}>{node.label}</span>{#if internal && collapsed}<span class="sexpr-ellipsis"> … </span><span class="sexpr-paren">)</span>{/if}
                    </span>{#if !expanded && trailing}<span class="sexpr-paren">{trailing}</span>{/if}
                    {#if hoveredSexprId === node.id}
                      <button class="sexpr-copy" title="Copy s-expression" onclick={() => copySexprNode(node)}>
                        {#if copiedSexprKey === node.id}
                          <ClipboardCheck size={12} />
                        {:else}
                          <Copy size={12} />
                        {/if}
                      </button>
                    {/if}
                  {/if}
                </div>
                {#if expanded}
                  {#each node.children as child, i}
                    {@render sexprNode(child, indent + 1, i === node.children.length - 1 ? ")" + trailing : "")}
                  {/each}
                {/if}
              {/snippet}
              {@render sexprNode(sexprRoot, 0, "")}
              </div>
            </div>
          {/if}
        {/if}
        <!-- The graph view stays mounted across tab switches (hidden when another
             tab is active) so its Cytoscape instance is built once and reused,
             not rebuilt on every visit or re-parse. The container is always
             present so its bind:this stays stable while parseTree resets to null
             mid-parse; only the controls are gated on having a parse tree. -->
        <div class="graph-view" class:hidden={activeTab !== "graph"}>
          <div class="cytoscape-container" bind:this={parseTreeContainer}></div>
          {#if graphRenderer?.webgl}
            <div class="renderer-badge" title="Rendered on the GPU (WebGL) because this tree exceeds the node threshold">WebGL</div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .main-content {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  /* Left Panel */
  .left-panel {
    min-width: 250px;
    max-width: 600px;
    display: flex;
    flex-direction: column;
    background: #252526;
    /* Clip the editor to the pane. Monaco's automatic layout can momentarily lag
       a shrink, and a short stacked pane must never let the editor spill over the
       result tabs below it. */
    overflow: hidden;
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

  /* Input Section */
  .input-section {
    flex: 1;
    min-height: 0;  /* shrink to the pane; a taller floor overflows short stacked panes over the result tabs */
    user-select: none;  /* Contain selection - allow only in the input editor */
    position: relative;  /* anchor the floating Parse button */
  }

  /* Parse trigger: a minimal play triangle in the input pane's top-right corner,
     replacing the old header Parse button. */
  .parse-play {
    position: absolute;
    top: 8px;
    right: 12px;
    z-index: 4;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border: none;
    background: transparent;
    color: #8a8a8a;
    cursor: pointer;
    transition: color 0.15s;
  }
  .parse-play:hover:not(:disabled) {
    color: #d4d4d4;
  }
  .parse-play:disabled {
    opacity: 0.4;
    cursor: default;
  }
  /* On the header bar the button is a normal flex item at the right end, not the
     absolutely-positioned overlay it is in the bare input pane. */
  .parse-play--bar {
    position: static;
    margin-left: auto;
    padding: 4px 6px;
  }

  /* Resize Handle */
  .resize-handle-vertical {
    width: 4px;
    cursor: col-resize;
    background: #3c3c3c;
    transition: background 0.2s;
    flex-shrink: 0;
    position: relative;
    z-index: 5;
  }

  .resize-handle-vertical:hover {
    background: #0e639c;
  }

  /* Vertical orientation: stack input over result and turn the divider into a
     horizontal row-resize bar. The input panel's height is set inline. */
  .main-content.vertical {
    flex-direction: column;
  }

  .main-content.vertical .left-panel {
    min-width: 0;
    max-width: none;
    min-height: 100px;
  }

  .main-content.vertical .resize-handle-vertical {
    width: auto;
    height: 4px;
    cursor: row-resize;
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

  /* Direct-child combinator so the tab padding stays on the Tree/Graph tabs and
     does not leak onto the nested .view-options-btn (which is a button too). */
  .tabs > button {
    padding: 8px 20px;
    background: transparent;
    color: #888;
    border: none;
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }

  .tabs > button.active {
    color: #d4d4d4;
    border-bottom-color: #0e639c;
  }

  .tabs > button:hover:not(.active) {
    color: #d4d4d4;
  }

  /* View-options popover, right-aligned in the tab row */
  .view-options {
    position: relative;
    margin-left: auto;
    margin-right: 2px;
    align-self: center;
  }

  .view-options-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 6px;
    background: #2d2d2d;
    color: #888;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    font-size: 11px;
  }

  .view-options-btn.active {
    background: #3c3c3c;
    color: #fff;
  }

  .view-options-btn:hover:not(.active) {
    background: #3c3c3c;
    color: #d4d4d4;
  }

  .view-options-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 20;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    background: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    white-space: nowrap;
  }

  .view-options-menu label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #d4d4d4;
    font-size: 11px;
    cursor: pointer;
  }

  .view-options-menu input {
    cursor: pointer;
    margin: 0;
  }

  /* Graph view: absolute-fill like the other views, stays mounted across tab
     switches. `display: none` while hidden so it doesn't overlay the active tab
     (Cytoscape re-reads its size via cy.resize() when it's shown again). */
  .graph-view {
    position: absolute;
    inset: 0;
  }

  .graph-view.hidden {
    display: none;
  }

  /* Marks the parse-tree graph as GPU-rendered; shown only on the WebGL path. */
  .renderer-badge {
    position: absolute;
    bottom: 8px;
    left: 8px;
    padding: 2px 7px;
    font-family: 'SF Mono', 'Fira Code', Consolas, monospace;
    font-size: 11px;
    color: #888;
    background: rgba(37, 37, 38, 0.8);
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    pointer-events: none;
    user-select: none;
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

  /* Float the expand/collapse controls in the view's top-right corner (like the
     graph controls) instead of a full toolbar row, so the tree/sexpr content gets
     the whole panel. */
  .tree-controls {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 2;
    display: flex;
    gap: 4px;
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

  /* `flash` is toggled imperatively by flashTreeRow, so scope only `.tree-item`. */
  .tree-item:global(.flash) {
    animation: tree-row-flash 0.35s ease-out;
  }

  @keyframes tree-row-flash {
    0% { background: #2b7fc4; }
    100% { background: transparent; }
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

  .tree-label.amb {
    color: #e05050;
  }

  /* A reference is its own category (a link to a shared node), not the kind it
     points to, so it gets one dedicated color regardless of the target's kind.
     The label is the jump link: pointer cursor and an underline on hover make it
     read as clickable, and clicking it navigates to the definition. */
  .tree-label.reference {
    color: #c586c0;
    font-style: italic;
    cursor: pointer;
  }

  .tree-label.reference:hover {
    text-decoration: underline;
    color: #e0a0db;
  }

  .tree-ref-icon {
    display: inline-flex;
    vertical-align: middle;
    margin-left: 4px;
  }

  .tree-span {
    color: #6a9955;
    font-size: 12px;
    margin-left: 8px;
  }

  /* S-expression View */
  .sexpr-view {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
  }

  .sexpr-scroll {
    flex: 1;
    overflow: auto;
    padding: 8px 0;
    font-family: 'SF Mono', 'Fira Code', Consolas, monospace;
    font-size: 13px;
    line-height: 1.5;
  }

  .sexpr-line {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: pre;
  }

  .sexpr-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 13px;
    color: #888;
    cursor: pointer;
    flex-shrink: 0;
  }

  .sexpr-toggle:hover {
    color: #d4d4d4;
  }

  /* CSS caret in place of a lucide chevron, so internal nodes mount no SVG. */
  .sexpr-caret {
    width: 5px;
    height: 5px;
    border-right: 1.5px solid currentColor;
    border-bottom: 1.5px solid currentColor;
    transform: rotate(-45deg);
    transition: transform 0.1s ease;
  }

  .sexpr-caret.expanded {
    transform: rotate(45deg);
  }

  .sexpr-toggle-placeholder {
    display: inline-block;
    width: 13px;
    flex-shrink: 0;
  }

  .sexpr-ellipsis {
    color: #888;
  }

  .sexpr-node {
    cursor: pointer;
    border-radius: 3px;
  }

  .sexpr-node:hover {
    background: #2a2d2e;
  }

  .sexpr-node.selected {
    background: #094771;
  }

  /* `flash` is toggled imperatively by flashTreeRow on input-click reveal. */
  .sexpr-node:global(.flash) {
    animation: tree-row-flash 0.35s ease-out;
    border-radius: 3px;
  }

  .sexpr-paren {
    color: #b59a6e;
  }

  .sexpr-label {
    color: #c586c0;
  }

  .sexpr-token {
    color: #4ec9b0;
  }

  .sexpr-token.amb {
    color: #e05050;
  }

  .sexpr-token.reference {
    color: #c586c0;
    font-style: italic;
  }

  /* Only the hovered row renders this button, so no per-node icon is mounted. */
  .sexpr-copy {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 18px;
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    flex-shrink: 0;
  }

  .sexpr-copy:hover {
    color: #fff;
  }
</style>
