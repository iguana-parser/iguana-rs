import type { Core, NodeSingular, EdgeSingular, ElementDefinition } from "cytoscape";
import { truncateLabel, LABEL_MAX_LENGTH } from "./graph-styles";

/**
 * Manages collapse/expand functionality for parse-tree and SPPF graph nodes.
 */
export class GraphCollapseManager {
  private cy: Core | null = null;
  private collapsedNodes = new Set<string>();
  private focusedNodeId: string | null = null;

  setCy(cy: Core | null) {
    this.cy = cy;
  }

  reset() {
    this.collapsedNodes = new Set();
    this.focusedNodeId = null;
  }

  isFocused(): boolean {
    return this.focusedNodeId !== null;
  }

  getFocusedNodeId(): string | null {
    return this.focusedNodeId;
  }

  isCollapsed(nodeId: string): boolean {
    return this.collapsedNodes.has(nodeId);
  }

  private findRoot(): string | null {
    if (!this.cy) return null;
    const roots = this.cy.nodes().filter((node: NodeSingular) =>
      node.incomers('edge').length === 0
    );
    return roots.length > 0 ? roots.first().id() : null;
  }

  /**
   * Get all nodes reachable from a starting node, respecting collapsed nodes.
   * `expandStart` makes the start node act as if uncollapsed, which is used
   * by focus mode so that focusing on a collapsed node still shows its
   * subtree. The graph-root BFS passes `false` so the root can be collapsed
   * like any other node.
   */
  private getReachableFromNode(startNodeId: string, expandStart: boolean): Set<string> {
    if (!this.cy) return new Set();
    const reachable = new Set<string>();

    const queue = [startNodeId];
    while (queue.length > 0) {
      const nodeId = queue.shift()!;
      if (reachable.has(nodeId)) continue;
      reachable.add(nodeId);

      // Don't traverse children of collapsed nodes; the focused start node is
      // the only exception (see `expandStart` above).
      if (
        this.collapsedNodes.has(nodeId) &&
        !(expandStart && nodeId === startNodeId)
      ) continue;

      const node = this.cy.getElementById(nodeId);
      node.outgoers('node').forEach((child: NodeSingular) => {
        if (!reachable.has(child.id())) {
          queue.push(child.id());
        }
      });
    }

    return reachable;
  }

  private getReachableNodes(): Set<string> {
    if (!this.cy) return new Set();

    if (this.focusedNodeId !== null) {
      return this.getReachableFromNode(this.focusedNodeId, true);
    }
    const root = this.findRoot();
    if (!root) return new Set();
    return this.getReachableFromNode(root, false);
  }

  updateVisibility() {
    if (!this.cy) return;

    const reachable = this.getReachableNodes();

    this.cy.nodes().forEach((node: NodeSingular) => {
      if (reachable.has(node.id())) {
        node.style('display', 'element');
      } else {
        node.style('display', 'none');
      }
    });

    this.cy.edges().forEach((edge: EdgeSingular) => {
      const sourceId = edge.source().id();
      const targetId = edge.target().id();
      if (reachable.has(sourceId) && reachable.has(targetId) && !this.collapsedNodes.has(sourceId)) {
        edge.style('display', 'element');
      } else {
        edge.style('display', 'none');
      }
    });
  }

  toggleCollapse(nodeId: string) {
    if (!this.cy) return;

    const node = this.cy.getElementById(nodeId);
    if (node.outgoers('node').length === 0) return;

    const isCollapsed = this.collapsedNodes.has(nodeId);

    if (isCollapsed) {
      this.collapsedNodes.delete(nodeId);
      node.removeClass('collapsed');
    } else {
      this.collapsedNodes.add(nodeId);
      node.addClass('collapsed');
    }

    this.updateVisibility();
  }

  expandAll() {
    if (!this.cy) return;

    this.collapsedNodes.forEach((nodeId) => {
      const node = this.cy!.getElementById(nodeId);
      node.removeClass('collapsed');
    });

    this.collapsedNodes = new Set();
    this.updateVisibility();
  }

  /**
   * Uncollapse every ancestor of the given node so it becomes visible.
   */
  expandAncestors(nodeId: string) {
    if (!this.cy) return;

    let changed = false;
    let currentId: string | null = nodeId;
    while (currentId !== null) {
      const current = this.cy.getElementById(currentId);
      if (current.length === 0) break;
      const parents = current.incomers('node');
      if (parents.length === 0) break;
      const parent = parents.first();
      const parentId = parent.id();
      if (this.collapsedNodes.has(parentId)) {
        this.collapsedNodes.delete(parentId);
        parent.removeClass('collapsed');
        changed = true;
      }
      currentId = parentId;
    }

    if (changed) this.updateVisibility();
  }

  /**
   * Focus on a subtree rooted at the given node.
   * Only the node and its descendants will be visible.
   */
  focusOnSubtree(nodeId: string) {
    if (!this.cy) return;

    this.focusedNodeId = nodeId;
    this.updateVisibility();

    // Fit the view to the visible subtree
    const visibleNodes = this.cy.nodes().filter((n: NodeSingular) => n.style('display') !== 'none');
    if (visibleNodes.length > 0) {
      this.cy.fit(visibleNodes, 50);
    }
  }

  /**
   * Clear focus and show all nodes (respecting collapsed state).
   */
  clearFocus() {
    if (!this.cy) return;

    this.focusedNodeId = null;
    this.updateVisibility();

    // Fit the view to all visible nodes
    const visibleNodes = this.cy.nodes().filter((n: NodeSingular) => n.style('display') !== 'none');
    if (visibleNodes.length > 0) {
      this.cy.fit(visibleNodes, 50);
    }
  }
}

/** The grammar construct a node was derived from, as `to_json` emits it. */
export type Origin = "Start" | "Opt" | "List" | "Group" | "Alt";

/** A node of the parse-tree DAG `to_json` emits. `origin` drives the display
 * transform (hiding empties, splicing wrappers). */
export interface ParseTreeNodeData {
  id: number;
  kind: "Nonterminal" | "Token" | "Amb";
  label: string;
  start: number;
  end: number;
  origin?: Origin | null;
}

/** The parse-tree shape `to_json` emits, as consumed by the parse view. */
export interface ParseTreeData {
  layout_name?: string | null;
  nodes: ParseTreeNodeData[];
  edges: { src: number; dest: number }[];
}

/** The presentation toggles the parse view offers, matching the s-expression
 * printer's options. All true is the faithful tree; all false is the simplified
 * view. Each is independent. */
export interface DisplayOptions {
  showLayout: boolean;
  showEmpty: boolean;
  showWrappers: boolean;
}

const WRAPPER_ORIGINS = new Set<Origin>(["Start", "Opt", "Group", "Alt"]);

/**
 * Produces the parse-tree DAG as the parse view shows it under `options`, the
 * frontend counterpart of the runtime's s-expression display transform (the two
 * stay in sync). Layout nonterminals are filtered out, empty repetitions dropped,
 * wrapper nodes (Start/Opt/Group/Alt) spliced into their parent, and a wrapper
 * root (typically `@Start`) unwrapped to the symbol it wraps. Node ids are
 * preserved so cross-view selection and ambiguity sharing still key on them; a
 * node shared in the ambiguity DAG is emitted once with an extra edge per
 * additional parent.
 */
export function buildDisplayGraph(raw: ParseTreeData, options: DisplayOptions): ParseTreeData {
  const layoutName = raw.layout_name ?? null;
  const nodeMap = new Map(raw.nodes.map((n) => [n.id, n]));
  const childrenMap = new Map<number, number[]>();
  const hasParent = new Set<number>();
  for (const edge of raw.edges) {
    if (!childrenMap.has(edge.src)) childrenMap.set(edge.src, []);
    childrenMap.get(edge.src)!.push(edge.dest);
    hasParent.add(edge.dest);
  }

  const isWrapper = (n: ParseTreeNodeData) => n.origin != null && WRAPPER_ORIGINS.has(n.origin);
  const isEmptyOptOrList = (n: ParseTreeNodeData) =>
    (n.origin === "Opt" || n.origin === "List") && (childrenMap.get(n.id)?.length ?? 0) === 0;
  const isHiddenLayout = (n: ParseTreeNodeData) =>
    !options.showLayout && layoutName !== null && n.kind === "Nonterminal" && n.label === layoutName;

  // The display children of a node: layout filtered out, empty repetitions
  // dropped, and wrappers spliced into place (recursively, so a chain of
  // wrappers collapses in one pass).
  function displayChildren(id: number): number[] {
    const result: number[] = [];
    for (const childId of childrenMap.get(id) ?? []) {
      const child = nodeMap.get(childId)!;
      if (isHiddenLayout(child)) continue;
      if (!options.showEmpty && isEmptyOptOrList(child)) continue;
      if (!options.showWrappers && isWrapper(child)) {
        result.push(...displayChildren(childId));
      } else {
        result.push(childId);
      }
    }
    return result;
  }

  const nodes: ParseTreeNodeData[] = [];
  const edges: { src: number; dest: number }[] = [];
  const seen = new Set<number>();

  // Emit a node once (a shared node reached again just gets another edge), then
  // descend into its display children.
  function emit(id: number, parentId: number | null) {
    if (seen.has(id)) {
      if (parentId !== null) edges.push({ src: parentId, dest: id });
      return;
    }
    seen.add(id);
    const node = nodeMap.get(id)!;
    nodes.push({ ...node });
    if (parentId !== null) edges.push({ src: parentId, dest: id });
    for (const childId of displayChildren(id)) emit(childId, id);
  }

  let rootId = raw.nodes.find((n) => !hasParent.has(n.id))?.id;
  if (rootId === undefined) return { layout_name: raw.layout_name, nodes, edges };
  // With wrappers spliced, descend through a wrapper root to the single node it
  // wraps, so the display is not headed by scaffolding.
  while (!options.showWrappers && isWrapper(nodeMap.get(rootId)!)) {
    const children = displayChildren(rootId);
    if (children.length === 1) rootId = children[0];
    else break;
  }

  emit(rootId, null);
  return { layout_name: raw.layout_name, nodes, edges };
}

/**
 * Builds Cytoscape elements for the parse-tree graph (parse view and its pop-out).
 * The tree is already transformed by `buildDisplayGraph`, so there is no filtering
 * here. With `showSpans`, the label gets a "(start, end)" second line.
 */
export function buildParseTreeElements(
  parseTree: ParseTreeData,
  showSpans: boolean,
): ElementDefinition[] {
  const nodes = parseTree.nodes.map((node) => {
    const span = `(${node.start}, ${node.end})`;
    const displayLabel = showSpans
      ? `${truncateLabel(node.label, LABEL_MAX_LENGTH)}\n${span}`
      : truncateLabel(node.label, LABEL_MAX_LENGTH);
    const fullLabel = showSpans ? `${node.label}\n${span}` : node.label;
    return {
      data: { id: `n${node.id}`, label: displayLabel, fullLabel, start: node.start, end: node.end },
      classes: node.kind.toLowerCase(),
    };
  });

  const edges = parseTree.edges.map((edge, i) => ({
    data: { id: `e${i}`, source: `n${edge.src}`, target: `n${edge.dest}` },
  }));

  return [...nodes, ...edges];
}
