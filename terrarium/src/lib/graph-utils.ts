import type { Core, NodeSingular, EdgeSingular } from "cytoscape";

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
