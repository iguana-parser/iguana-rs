<script lang="ts">
  import { commands } from "../../bindings";
  import { tick } from "svelte";
  import { ChevronDown, ChevronRight, CornerRightUp, ZoomIn, ZoomOut, Maximize2, Minimize2, Expand, Fullscreen, UnfoldHorizontal, FoldHorizontal, Download, Eye, EyeOff, Copy, ClipboardCheck } from "lucide-svelte";
  import cytoscape from "cytoscape";
  import {
    sppfNodeStyles,
    edgeStyles,
    adjustZoomGraph,
    resetViewGraph,
    createGraph,
    getViewport,
    setupGraphTooltip,
    highlightOutgoingEdges,
    clearEdgeHighlights,
    highlightClickedEdge,
    PARSE_TREE_LAYOUT,
  } from "$lib/graph-styles";
  import { GraphCollapseManager, exportGraphPng, buildParseTreeElements } from "$lib/graph-utils";
  import InputEditor from "$lib/InputEditor.svelte";
  import NonterminalPicker from "$lib/NonterminalPicker.svelte";
  import "$lib/graph.css";
  import "$lib/parse-view/parse-view.css";

  // Parse Tree types (manually defined, not via specta)
  interface ParseTreeNode {
    id: number;
    kind: "Nonterminal" | "Token" | "Amb";
    label: string;
    start: number;
    end: number;
  }
  interface ParseTreeEdge {
    src: number;
    dest: number;
  }
  interface ParseTree {
    layout_name?: string | null;
    nodes: ParseTreeNode[];
    edges: ParseTreeEdge[];
  }

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
    parserDirectory: string | null;
    parserName: string | null;
    buildStatus: "none" | "success" | "error";
    nonterminals: string[];
    startNonterminal: string | null;
    inputText: string;
    leftPanelWidth: number;
    isProfiling?: boolean;
    // Host-specific hooks. Terrarium passes these; the web viewer omits them, so the
    // corresponding chrome (status bar, output log, profiling, graph pop-out) is dropped.
    onStatus?: (message: string, type?: "info" | "error" | "success", tooltip?: string) => void;
    onLogCommand?: (cmd: string) => void;
    onLogOutput?: (text: string) => void;
    onLogError?: (text: string) => void;
    onProfile?: () => void;
    onPopOut?: () => void;
    // Fired when the visible parse tree changes, so a host that popped the graph
    // out into its own window can re-send the data.
    onParseTreeChange?: () => void;
    startVerticalDrag?: (e: MouseEvent) => void;
  }

  let {
    parserDirectory,
    parserName,
    buildStatus,
    nonterminals,
    startNonterminal = $bindable(null),
    inputText = $bindable(""),
    leftPanelWidth,
    isProfiling = false,
    onStatus,
    onLogCommand,
    onLogOutput,
    onLogError,
    onProfile,
    onPopOut,
    onParseTreeChange,
    startVerticalDrag,
  }: Props = $props();

  // Convert flat parse tree to hierarchical structure, skipping any node id
  // in `hidden` (and therefore any subtree rooted at one). Fills `parentMap`
  // with each node's parent in the tree as actually rendered, so the reveal
  // walk expands exactly the chain that leads to where a node is shown in full.
  function buildTree(parseTree: ParseTree, hidden: Set<number>, parentMap: Map<number, number>): TreeNode | null {
    if (parseTree.nodes.length === 0) return null;

    // Build adjacency list from edges
    const childrenMap = new Map<number, number[]>();
    const hasParent = new Set<number>();

    for (const edge of parseTree.edges) {
      if (hidden.has(edge.src) || hidden.has(edge.dest)) continue;
      if (!childrenMap.has(edge.src)) childrenMap.set(edge.src, []);
      childrenMap.get(edge.src)!.push(edge.dest);
      hasParent.add(edge.dest);
    }

    // Find root (node with no parent)
    const rootNode = parseTree.nodes.find(n => !hidden.has(n.id) && !hasParent.has(n.id));
    if (!rootNode) return null;

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

    return buildSubtree(rootNode.id, null);
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

  // Layout nonterminal (per parseTree.layout_name) and every descendant.
  // Returns an empty set when hide is false or the grammar has no layout rule.
  function computeHiddenLayoutNodes(parseTree: ParseTree, hide: boolean): Set<number> {
    const hidden = new Set<number>();
    if (!hide || !parseTree.layout_name) return hidden;
    const layoutName = parseTree.layout_name;

    const childrenMap = new Map<number, number[]>();
    for (const edge of parseTree.edges) {
      if (!childrenMap.has(edge.src)) childrenMap.set(edge.src, []);
      childrenMap.get(edge.src)!.push(edge.dest);
    }

    for (const node of parseTree.nodes) {
      if (node.kind !== "Nonterminal" || node.label !== layoutName) continue;
      if (hidden.has(node.id)) continue;
      const queue = [node.id];
      while (queue.length > 0) {
        const id = queue.shift()!;
        if (hidden.has(id)) continue;
        hidden.add(id);
        const children = childrenMap.get(id);
        if (children) queue.push(...children);
      }
    }
    return hidden;
  }

  // Tabs: the tree view, the graph view, and the interactive s-expression.
  let activeTab = $state<"tree" | "graph" | "sexpr">("tree");

  // Show spans in graph labels (hidden by default)
  let showSpans = $state(false);

  // Parse Tree data
  let parseTree = $state<ParseTree | null>(null);
  let ambiguityWarnings = $derived(parseTree ? collectOutermostAmbs(parseTree) : []);
  // child node id → parent node id in the rendered tree, filled by buildTree
  let parseTreeParentMap = new Map<number, number>();
  // Layout nonterminal and its descendants, hidden when hideLayout is on.
  // On by default: layout nodes are rarely what you want to see, so the parse
  // structure reads better with them hidden.
  let hiddenLayoutNodes = $state(new Set<number>());
  let hideLayout = $state(true);
  // svelte-ignore non_reactive_update
  let parseTreeContainer: HTMLDivElement;
  let parseTreeCy: cytoscape.Core | null = null;
  const parseTreeCollapseManager = new GraphCollapseManager();
  // The Cytoscape instance is built once and kept alive across tab switches.
  // `graphDirty` marks that its elements are stale (new parse or a layout/span
  // toggle), so the next time the graph tab is shown it reloads instead of just
  // resizing. A plain flag, not reactive: the graph $effect reacts to activeTab.
  let graphDirty = true;

  // Parse tree node selection (for highlighting span in input)
  let parseTreeSelectedSpan = $state<{ start: number; end: number } | null>(null);
  let parseTreeSelectedNodeId = $state<string | null>(null);
  // The selected tree row, keyed per row (TreeNode.key) so a reference and its
  // definition (which share a node id) highlight independently. Cross-view
  // highlighting (input span, graph node) still keys on parseTreeSelectedNodeId.
  let selectedTreeRowKey = $state<string | null>(null);

  let treeRoot = $state<TreeNode | null>(null);
  let expandedNodes = $state(new Set<number>());
  // svelte-ignore non_reactive_update
  let treeContainerEl: HTMLDivElement;

  // Interactive s-expression, derived from the parse tree and the hide-layout toggle.
  let sexprRoot = $derived(parseTree ? buildSexprModel(parseTree, hiddenLayoutNodes) : null);
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

  // Track if parse result is available
  let parseResultAvailable = $state(false);

  // Parse error info for input editor markers
  let parseErrorInfo = $state<{ line: number; column: number; message: string } | null>(null);

  let parseTreeTooltipCleanup: (() => void) | null = null;

  function buildGraphElements() {
    return buildParseTreeElements(parseTree!, showSpans, hiddenLayoutNodes);
  }

  // Park the root at the top-center of the viewport, instead of fitting the whole
  // tree (which zooms a few-thousand-node tree down to dots). The zoom is based on
  // the vertical level spacing, not the tree width: tidy-tree centers each child
  // over its subtree, so the root's children span nearly the whole tree width,
  // which would make a width-based fit collapse to dots on a wide tree. Vertical
  // spacing is uniform per level, so framing to a fixed level count is consistent
  // whether layout is shown or hidden. The user scrolls down from there.
  const ROOT_FRAME_TOP_PADDING = 60;
  const ROOT_FRAME_MIN_ZOOM = 0.3;
  const ROOT_FRAME_MAX_ZOOM = 1.5;
  const ROOT_FRAME_LEVELS_VISIBLE = 12;
  function frameOnRoot() {
    if (!parseTreeCy || !parseTreeContainer) return;
    const root = parseTreeCy.nodes().roots().first();
    if (root.length === 0) return;

    const rootPos = root.position();
    const child = root.outgoers("node").first() as cytoscape.NodeSingular;
    const levelStep = child.length > 0 ? Math.abs(child.position().y - rootPos.y) : 60;
    const h = parseTreeContainer.clientHeight;
    const pad = 60;
    const fitZoom = (h - 2 * pad) / (levelStep * ROOT_FRAME_LEVELS_VISIBLE);
    const zoom = Math.max(ROOT_FRAME_MIN_ZOOM, Math.min(fitZoom, ROOT_FRAME_MAX_ZOOM));

    parseTreeCy.zoom(zoom);
    parseTreeCy.pan({
      x: parseTreeContainer.clientWidth / 2 - rootPos.x * zoom,
      y: ROOT_FRAME_TOP_PADDING - rootPos.y * zoom,
    });
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

    frameOnRoot();
    graphDirty = false;
  }

  // Swap fresh elements onto the existing instance and relayout, reusing the same
  // Cytoscape instance — no teardown, so repeated toggles don't churn it. `fit`
  // true re-fits the view (a new parse); false preserves the current viewport (a
  // toggle the user is mid-inspecting) and re-applies the current node selection.
  function reloadGraph(fit: boolean) {
    if (!parseTreeCy || !parseTree) return;
    parseTreeCollapseManager.reset();
    const savedViewport = fit ? undefined : getViewport(parseTreeCy);

    parseTreeCy.elements().remove();
    parseTreeCy.add(buildGraphElements());
    parseTreeCy.layout({ ...PARSE_TREE_LAYOUT, fit } as any).run();

    if (savedViewport) {
      parseTreeCy.zoom(savedViewport.zoom);
      parseTreeCy.pan(savedViewport.pan);
    } else {
      // fit=true: frame on the root at a readable zoom rather than fitting the
      // whole tree to the viewport.
      frameOnRoot();
    }
    // Re-apply the cross-view selection to the matching node, if it survived.
    if (parseTreeSelectedNodeId) {
      const node = parseTreeCy.getElementById(parseTreeSelectedNodeId);
      if (node.length > 0) {
        node.addClass('selected');
        highlightOutgoingEdges(parseTreeCy, parseTreeSelectedNodeId);
      }
    }
    // WebGL renderer: after a relayout it doesn't repaint edge buffers until the
    // next viewport change, so the edges vanish until you pan or select. resize()
    // forces a full redraw and brings them back immediately.
    parseTreeCy.resize();
    graphDirty = false;
  }

  // Build the instance the first time, reload it afterwards.
  function loadGraph(fit: boolean) {
    if (parseTreeCy) reloadGraph(fit);
    else buildGraph();
  }

  // The graph view stays mounted across tab switches, so the instance is built
  // once per parse rather than rebuilt on every visit. When the graph tab is
  // shown: reload if its elements are stale, otherwise just resize (the container
  // was display:none while hidden, so Cytoscape needs to re-read its size).
  $effect(() => {
    if (activeTab === "graph" && parseTree) {
      tick().then(() => {
        if (!parseTreeContainer) return;
        if (graphDirty || !parseTreeCy) loadGraph(true);
        else parseTreeCy.resize();
      });
    }
  });

  // Exported so the page-level Cmd+P keybinding can fire the same parse via bind:this.
  export async function parse() {
    if (!parserDirectory || buildStatus !== "success" || !startNonterminal) return;
    onStatus?.("Parsing...", "info");

    // Reset previous results
    parseTree = null;
    parseResultAvailable = false;
    parseTreeSelectedSpan = null;
    parseErrorInfo = null;

    onLogCommand?.(`${parserName} <input> --start ${startNonterminal}`);

    const result = await commands.parse(parserDirectory, inputText, startNonterminal);
    if (result.status === "error") {
      // Command itself failed (couldn't run parser)
      onLogError?.(result.error);
      onStatus?.("Parse failed", "error");
      return;
    }

    const output = result.data;
    // The parse view only renders the parse tree; SPPF/GSS live in debug mode.
    parseResultAvailable = output.has_parse_tree;

    if (output.success) {
      parseErrorInfo = null;
      const totalMs = (output.duration_ms ?? 0) + (output.tree_construction_ms ?? 0);
      const durationStr = output.duration_ms != null ? ` (${totalMs}ms)` : "";
      onLogOutput?.(`Parse successful${durationStr}`);
      const tooltip = output.duration_ms != null
        ? `Parse: ${output.duration_ms}ms\nTree construction: ${output.tree_construction_ms ?? '?'}ms`
        : undefined;
      onStatus?.(`Parse successful${durationStr}`, "success", tooltip);
    } else {
      parseErrorInfo = output.error_info ?? null;
      if (output.error) {
        onLogError?.(output.error);
      }
      if (parseResultAvailable) {
        onLogOutput?.("Partial data available: Parse Tree");
        onStatus?.("Parse error (partial data)", "error");
      } else {
        onStatus?.("Parse failed", "error");
      }
    }

    if (output.has_parse_tree) {
      await fetchParseTree();
    }
  }

  async function fetchParseTree() {
    if (!parseResultAvailable) return;
    const result = await commands.getParseTree();
    if (result.status === "ok") {
      try {
        parseTree = JSON.parse(result.data) as ParseTree;
        rebuildLayoutDerivedState();
        // New tree: drop any stale selection and mark the graph for reload. The
        // graph $effect rebuilds it when the graph tab is (or becomes) active.
        clearParseModeInputSelection();
        graphDirty = true;
        // Expand root by default
        if (treeRoot) {
          expandedNodes = new Set([treeRoot.id]);
        }
        onParseTreeChange?.();
      } catch (e) {
        // JSON parse error — ignore
      }
    }
  }

  // Re-derive everything that depends on (parseTree, hideLayout):
  // hidden-node set, parent map for the reveal walk, and treeRoot. The parent
  // map is filled by buildTree from the tree as rendered, so the reveal walk
  // follows the chain to where each shared node is shown in full.
  function rebuildLayoutDerivedState() {
    if (!parseTree) return;
    hiddenLayoutNodes = computeHiddenLayoutNodes(parseTree, hideLayout);
    parseTreeParentMap = new Map();
    treeRoot = buildTree(parseTree, hiddenLayoutNodes, parseTreeParentMap);
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
    exportGraphPng(parseTreeCy, "parse-tree");
  }

  function toggleHideLayout() {
    hideLayout = !hideLayout;
    rebuildLayoutDerivedState();
    // Selection may have pointed at a now-hidden node; drop it.
    clearParseModeInputSelection();
    graphDirty = true;
    // Re-fit on a layout toggle: the node set changed substantially, so re-center
    // on the root rather than holding the old viewport.
    if (parseTree && activeTab === "graph") {
      tick().then(() => loadGraph(true));
    }
    onParseTreeChange?.();
  }

  // The visible parse tree (layout-hidden nodes removed), for the graph pop-out.
  // Exported so a host can feed it to a separate graph window via bind:this.
  export function getParseTreeForPopup(): ParseTree | null {
    if (!parseTree) return null;
    return {
      layout_name: parseTree.layout_name,
      nodes: parseTree.nodes.filter((n) => !hiddenLayoutNodes.has(n.id)),
      edges: parseTree.edges.filter((e) => !hiddenLayoutNodes.has(e.src) && !hiddenLayoutNodes.has(e.dest)),
    };
  }

  function toggleSpans() {
    showSpans = !showSpans;
    // reloadGraph preserves the viewport and re-applies the current selection.
    graphDirty = true;
    if (parseTree && activeTab === "graph") {
      tick().then(() => loadGraph(false));
    }
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

  // Smallest-span parse tree node whose half-open range [start, end) covers
  // the offset, i.e. the deepest enclosing node. When the click lands inside
  // a hidden layout subtree, fall back to the nearest visible token instead
  // of returning the wide enclosing parent.
  function findDeepestParseTreeNodeAt(offset: number): ParseTreeNode | null {
    if (!parseTree) return null;
    let best: ParseTreeNode | null = null;
    let clickInsideHidden = false;
    for (const node of parseTree.nodes) {
      const contains = node.start <= offset && offset < node.end;
      if (hiddenLayoutNodes.has(node.id)) {
        if (contains) clickInsideHidden = true;
        continue;
      }
      if (contains && (!best || node.end - node.start < best.end - best.start)) {
        best = node;
      }
    }
    if (clickInsideHidden) {
      let nearestToken: ParseTreeNode | null = null;
      let nearestDistance = Infinity;
      for (const node of parseTree.nodes) {
        if (node.kind !== "Token" || hiddenLayoutNodes.has(node.id)) continue;
        const distance = offset < node.start ? node.start - offset : offset - node.end;
        if (distance < nearestDistance) {
          nearestDistance = distance;
          nearestToken = node;
        }
      }
      if (nearestToken) return nearestToken;
    }
    return best;
  }

  function parseTreeAncestorsOf(nodeId: number): number[] {
    const ancestors: number[] = [];
    let cur = parseTreeParentMap.get(nodeId);
    while (cur !== undefined) {
      ancestors.push(cur);
      cur = parseTreeParentMap.get(cur);
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

  // Build the interactive s-expression model, mirroring the generated to_sexpr:
  // a node reachable from several parents (indegree > 1 in the rendered DAG) is
  // written once with a `#N=` label, and later occurrences become `#N#` refs.
  // Honors hideLayout so all three tabs show the same structure.
  function buildSexprModel(parseTree: ParseTree, hidden: Set<number>): SexprNode | null {
    const childrenMap = new Map<number, number[]>();
    const indegree = new Map<number, number>();
    for (const edge of parseTree.edges) {
      if (hidden.has(edge.src) || hidden.has(edge.dest)) continue;
      if (!childrenMap.has(edge.src)) childrenMap.set(edge.src, []);
      childrenMap.get(edge.src)!.push(edge.dest);
      indegree.set(edge.dest, (indegree.get(edge.dest) ?? 0) + 1);
    }
    const root = parseTree.nodes.find(n => !hidden.has(n.id) && !indegree.has(n.id));
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

  // Serialize an s-expression node to text, matching to_sexpr's layout (two-space
  // indent, `(name` … `)` for internal nodes, bare name for leaves).
  function sexprToText(node: SexprNode, indent: number = 0): string {
    const pad = " ".repeat(indent);
    if (node.ref !== undefined) return `${pad}#${node.ref}#\n`;
    const prefix = node.shareLabel !== undefined ? `#${node.shareLabel}=` : "";
    if (node.children.length === 0) return `${pad}${prefix}${node.label}\n`;
    let s = `${pad}${prefix}(${node.label}\n`;
    for (const child of node.children) s += sexprToText(child, indent + 2);
    s += `${pad})\n`;
    return s;
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

<svelte:window onkeydown={handleParseViewKeydown} />

<div class="main-content">
  <!-- Left Panel -->
  <div class="left-panel" style="width: {leftPanelWidth}px">
    <!-- Header -->
    <div class="header">
      <div class="dropdown-wrapper">
        <span class="dropdown-label">Start:</span>
        <NonterminalPicker
          bind:value={startNonterminal}
          options={nonterminals}
          disabled={!parserDirectory || nonterminals.length === 0}
        />
      </div>
      <div class="parse-actions">
        <button class="parse-btn" onclick={parse} disabled={!parserDirectory || buildStatus !== "success" || !startNonterminal}>Parse</button>
        {#if onProfile}
          <button class="parse-btn" onclick={onProfile} disabled={!parserDirectory || buildStatus !== "success" || !startNonterminal || isProfiling}>
            {isProfiling ? "Profiling..." : "Profile"}
          </button>
        {/if}
      </div>
    </div>

    <!-- Input Area -->
    <div class="input-section">
      <InputEditor
        bind:value={inputText}
        error={parseErrorInfo}
        ambiguities={ambiguityWarnings}
        highlightSpan={parseTreeSelectedSpan}
        onclick={onParseInputClick}
        onchange={clearParseModeInputSelection}
        onescape={clearParseModeInputSelection}
        placeholder="Enter code to parse..."
      />
    </div>
  </div>

  <!-- Vertical Resize Handle -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="resize-handle-vertical" onmousedown={startVerticalDrag}></div>

  <!-- Right Panel -->
  <div class="right-panel">
    <div class="graph-section">
      <div class="tabs">
        <button class:active={activeTab === "tree"} onclick={() => activeTab = "tree"}>Tree</button>
        <button class:active={activeTab === "graph"} onclick={() => activeTab = "graph"}>Graph</button>
        <button class:active={activeTab === "sexpr"} onclick={() => activeTab = "sexpr"}>S-expr</button>
        {#if parseTree?.layout_name}
          <button
            class="layout-toggle"
            class:active={hideLayout}
            onclick={toggleHideLayout}
            title={hideLayout ? `Show ${parseTree.layout_name} nodes` : `Hide ${parseTree.layout_name} nodes`}
          >
            {#if hideLayout}
              <EyeOff size={14} />
            {:else}
              <Eye size={14} />
            {/if}
            <span>{parseTree.layout_name}</span>
          </button>
        {/if}
      </div>
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
              <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
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
                    <span class="tree-span">[{node.start}:{node.end}]</span>
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
              {#snippet sexprNode(node: SexprNode, indent: number)}
                {@const internal = node.ref === undefined && node.children.length > 0}
                {@const collapsed = sexprCollapsed.has(node.id)}
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
                    <span class="sexpr-token reference">#{node.ref}#</span>
                  {:else}
                    <span
                      class="sexpr-node"
                      class:selected={parseTreeSelectedNodeId === `n${node.id}`}
                      onmousedown={(e) => { if (e.detail > 1) e.preventDefault(); }}
                      onclick={() => clickSexprNode(node)}
                    >
                      {#if node.shareLabel !== undefined}<span class="sexpr-label">#{node.shareLabel}=</span>{/if}{#if internal}<span class="sexpr-paren">(</span>{/if}<span class="sexpr-token" class:amb={node.label === "Amb"}>{node.label}</span>{#if internal && collapsed}<span class="sexpr-ellipsis"> … </span><span class="sexpr-paren">)</span>{/if}
                    </span>
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
                {#if internal && !collapsed}
                  {#each node.children as child}
                    {@render sexprNode(child, indent + 1)}
                  {/each}
                  <div class="sexpr-line" style="padding-left: {indent * 9 + 8}px">
                    <span class="sexpr-toggle-placeholder"></span><span class="sexpr-paren">)</span>
                  </div>
                {/if}
              {/snippet}
              {@render sexprNode(sexprRoot, 0)}
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
          {#if parseTree}
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
              {#if onPopOut}
                <button onclick={onPopOut} title="Pop out">
                  <Fullscreen size={16} />
                </button>
              {/if}
            </div>
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
    min-height: 100px;
    user-select: none;  /* Contain selection - allow only in the input editor */
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

  /* Hide-layout toggle, right-aligned in the tab row */
  .layout-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    margin-right: 8px;
    align-self: center;
    padding: 3px 8px;
    background: #2d2d2d;
    color: #888;
    border: 1px solid #3c3c3c;
    border-radius: 3px;
    cursor: pointer;
    font-size: 11px;
  }

  .layout-toggle.active {
    background: #3c3c3c;
    color: #fff;
    border-color: #555;
  }

  .layout-toggle:hover:not(.active) {
    background: #3c3c3c;
    color: #d4d4d4;
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
    font-size: 11px;
    margin-left: auto;
    padding-left: 12px;
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
