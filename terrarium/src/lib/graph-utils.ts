import type { Core, NodeSingular, EdgeSingular, ElementDefinition } from "cytoscape";
import type { DebugSPPFNode } from "../bindings";
import { truncateLabel, LABEL_MAX_LENGTH, INTERMEDIATE_MAX_LENGTH } from "./graph-styles";

/**
 * Manages collapse/expand functionality for SPPF graph nodes.
 */
export class GraphCollapseManager {
  private cy: Core | null = null;
  private collapsedNodes = new Set<string>();

  setCy(cy: Core | null) {
    this.cy = cy;
  }

  reset() {
    this.collapsedNodes = new Set();
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

  private getReachableNodes(): Set<string> {
    if (!this.cy) return new Set();
    const reachable = new Set<string>();
    const root = this.findRoot();
    if (!root) return reachable;

    const queue = [root];
    while (queue.length > 0) {
      const nodeId = queue.shift()!;
      if (reachable.has(nodeId)) continue;
      reachable.add(nodeId);

      if (this.collapsedNodes.has(nodeId)) continue;

      const node = this.cy.getElementById(nodeId);
      node.outgoers('node').forEach((child: NodeSingular) => {
        if (!reachable.has(child.id())) {
          queue.push(child.id());
        }
      });
    }

    return reachable;
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
}

/**
 * Builds Cytoscape elements for debug SPPF visualization.
 * Filters to show only the subtree reachable from currentNodeId.
 * Returns null if there are no reachable nodes.
 */
export function buildDebugSppfElements(
  nodes: DebugSPPFNode[],
  currentNodeId: number | null
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

  const elements: ElementDefinition[] = [];

  // Add only nodes in the current subtree
  for (const node of nodes) {
    if (!reachableIds.has(node.id)) continue;

    // Line 1: grammar slot (truncated if needed), Line 2: span
    // Intermediate nodes get longer max length since they show grammar slots
    const maxLen = node.kind === "Intermediate" ? INTERMEDIATE_MAX_LENGTH : LABEL_MAX_LENGTH;
    const span = `(${node.left_extent}, ${node.right_extent})`;
    const displayLabel = `${truncateLabel(node.label, maxLen)}\n${span}`;
    const fullLabel = `${node.label}\n${span}`;

    elements.push({
      data: {
        id: `n${node.id}`,
        label: displayLabel,
        fullLabel: fullLabel,
        kind: node.kind,
        leftExtent: node.left_extent,
        rightExtent: node.right_extent,
      },
      classes: node.kind.toLowerCase(),
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
