import type { Core, NodeSingular, EdgeSingular, ElementDefinition } from "cytoscape";
import type { DebugSPPFNode, DebugGSSNode, DebugGSSEdge, SPPF, GSS, NodeKind } from "../bindings";
import { truncateLabel, LABEL_MAX_LENGTH, INTERMEDIATE_MAX_LENGTH } from "./graph-styles";

/**
 * Extracts the kind name and ambiguous flag from a NodeKind.
 */
export function parseNodeKind(kind: NodeKind): { name: string; ambiguous: boolean } {
  if (typeof kind === "string") {
    // Terminal or Packed
    return { name: kind, ambiguous: false };
  } else if ("Nonterminal" in kind) {
    return { name: "Nonterminal", ambiguous: kind.Nonterminal.ambiguous };
  } else if ("Intermediate" in kind) {
    return { name: "Intermediate", ambiguous: kind.Intermediate.ambiguous };
  }
  return { name: "Unknown", ambiguous: false };
}
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";

/**
 * Exports a Cytoscape graph as a PNG file using native save dialog.
 * @param graph - The Cytoscape instance to export
 * @param defaultName - Default filename (without extension)
 */
export async function exportGraphPng(graph: Core | null, defaultName: string): Promise<void> {
  if (!graph) return;

  const path = await save({
    defaultPath: `${defaultName}.png`,
    filters: [{ name: "PNG Image", extensions: ["png"] }],
  });

  if (!path) return; // User cancelled

  const blob = await graph.png({ output: "blob", bg: "#1e1e1e", scale: 2 });
  const buffer = await blob.arrayBuffer();
  await writeFile(path, new Uint8Array(buffer));
}

/**
 * Manages collapse/expand functionality for SPPF graph nodes.
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

/**
 * Builds Cytoscape elements for debug SPPF visualization.
 * Filters to show only the subtree reachable from currentNodeId.
 * Returns null if there are no reachable nodes.
 * @param showSpans - Whether to include span information in node labels (default: true for backward compatibility)
 */
export function buildDebugSppfElements(
  nodes: DebugSPPFNode[],
  currentNodeId: number | null,
  showSpans: boolean = true
): ElementDefinition[] | null {
  // Build a map for quick lookup
  const nodeMap = new Map<number, DebugSPPFNode>();
  for (const node of nodes) {
    nodeMap.set(node.id, node);
  }

  // Find all nodes reachable from current node (the subtree to show)
  const reachableIds = new Set<number>();
  if (currentNodeId !== null && nodeMap.has(currentNodeId)) {
    const queue = [currentNodeId];
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

  // If no reachable nodes, return null
  if (reachableIds.size === 0) {
    return null;
  }

  // Count terminal nodes per span to detect shared spans (only count reachable)
  const spanCounts = new Map<string, number>();
  for (const node of nodes) {
    if (!reachableIds.has(node.id)) continue;
    if (node.kind === "Terminal") {
      const spanKey = `${node.left_extent},${node.right_extent}`;
      spanCounts.set(spanKey, (spanCounts.get(spanKey) || 0) + 1);
    }
  }

  const elements: ElementDefinition[] = [];

  // Add only nodes in the current subtree
  for (const node of nodes) {
    if (!reachableIds.has(node.id)) continue;

    // Intermediate nodes get longer max length since they show grammar slots
    const maxLen = node.kind === "Intermediate" ? INTERMEDIATE_MAX_LENGTH : LABEL_MAX_LENGTH;
    // Optionally show span on second line
    const span = `(${node.left_extent}, ${node.right_extent})`;
    const displayLabel = showSpans
      ? `${truncateLabel(node.label, maxLen)}\n${span}`
      : truncateLabel(node.label, maxLen);
    const fullLabel = showSpans
      ? `${node.label}\n${span}`
      : node.label;

    // Check if this terminal node shares its span with other terminals
    const spanKey = `${node.left_extent},${node.right_extent}`;
    const hasSharedSpan = node.kind === "Terminal" && (spanCounts.get(spanKey) || 0) > 1;
    let classes = node.kind.toLowerCase();
    if (hasSharedSpan) classes += ' shared-span';

    elements.push({
      data: {
        id: `n${node.id}`,
        label: displayLabel,
        fullLabel: fullLabel,
        kind: node.kind,
        leftExtent: node.left_extent,
        rightExtent: node.right_extent,
      },
      classes: classes,
    });
  }

  // Add edges only within the subtree
  for (const node of nodes) {
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
}

/**
 * Builds Cytoscape elements for SPPF visualization (parse mode).
 */
export function buildSppfElements(sppf: SPPF): ElementDefinition[] {
  // Build a map of node IDs to their ambiguous status
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

  const nodes = sppf.nodes.map((node) => {
    const { name: kindName, ambiguous } = parseNodeKind(node.kind);
    nodeAmbiguousMap.set(node.id, ambiguous);
    const fullLabel = node.label || "";
    const maxLen = kindName === "Intermediate" ? INTERMEDIATE_MAX_LENGTH : LABEL_MAX_LENGTH;
    // Check if this terminal node shares its span with other terminals
    const spanKey = `${node.left_extent},${node.right_extent}`;
    const hasSharedSpan = kindName === "Terminal" && (spanCounts.get(spanKey) || 0) > 1;
    let classes = kindName.toLowerCase();
    if (ambiguous) classes += ' ambiguous';
    if (hasSharedSpan) classes += ' shared-span';
    return {
      data: {
        id: `n${node.id}`,
        label: truncateLabel(fullLabel, maxLen),
        fullLabel: fullLabel,
        kind: kindName,
        ambiguous: ambiguous,
        leftExtent: node.left_extent,
        rightExtent: node.right_extent,
      },
      classes: classes,
    };
  });

  const edges = sppf.edges.map((edge, i) => {
    const sourceAmbiguous = nodeAmbiguousMap.get(edge.src) || false;
    return {
      data: {
        id: `e${i}`,
        source: `n${edge.src}`,
        target: `n${edge.dest}`,
      },
      classes: sourceAmbiguous ? "edge-ambiguous" : "",
    };
  });

  return [...nodes, ...edges];
}

/**
 * Builds Cytoscape elements for GSS visualization (parse mode).
 */
export function buildGssElements(gss: GSS): ElementDefinition[] {
  const nodes = gss.nodes.map((node) => ({
    data: {
      id: `n${node.id}`,
      label: node.label,
    },
  }));

  const edges = gss.edges.map((edge, i) => ({
    data: {
      id: `e${i}`,
      source: `n${edge.src}`,
      target: `n${edge.dest}`,
      label: edge.label,
    },
  }));

  return [...nodes, ...edges];
}

/**
 * Builds Cytoscape elements for debug GSS visualization.
 */
export function buildDebugGssElements(
  nodes: DebugGSSNode[],
  edges: DebugGSSEdge[],
  currentNodeId: number | null
): ElementDefinition[] {
  const nodeElements = nodes.map((node) => ({
    data: {
      id: `n${node.id}`,
      label: node.label,
    },
    classes: currentNodeId === node.id ? "current" : "",
  }));

  const edgeElements = edges.map((edge, i) => ({
    data: {
      id: `e${i}`,
      source: `n${edge.src}`,
      target: `n${edge.dest}`,
      label: edge.label,
    },
  }));

  return [...nodeElements, ...edgeElements];
}
