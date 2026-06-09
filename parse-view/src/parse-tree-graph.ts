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

/** The parse-tree shape `to_json` emits, as consumed by the parse view. */
export interface ParseTreeData {
  layout_name?: string | null;
  nodes: { id: number; kind: "Nonterminal" | "Token" | "Amb"; label: string; start: number; end: number }[];
  edges: { src: number; dest: number }[];
}

/**
 * Builds Cytoscape elements for the parse-tree graph (parse view and its pop-out).
 * Skips any node id in `hidden` and edges touching one. With `showSpans`, the
 * label gets a "(start, end)" second line.
 */
export function buildParseTreeElements(
  parseTree: ParseTreeData,
  showSpans: boolean,
  hidden: Set<number> = new Set(),
): ElementDefinition[] {
  const nodes = parseTree.nodes
    .filter((node) => !hidden.has(node.id))
    .map((node) => {
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

  const edges = parseTree.edges
    .filter((edge) => !hidden.has(edge.src) && !hidden.has(edge.dest))
    .map((edge, i) => ({
      data: { id: `e${i}`, source: `n${edge.src}`, target: `n${edge.dest}` },
    }));

  return [...nodes, ...edges];
}
