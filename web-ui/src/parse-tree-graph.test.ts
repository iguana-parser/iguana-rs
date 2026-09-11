import cytoscape from "cytoscape";
import { describe, expect, it } from "vitest";
import { buildDisplayGraph, buildParseTreeElements, GraphCollapseManager, type ParseTreeData } from "./parse-tree-graph";

describe("cyclic parse trees", () => {
  const graph: ParseTreeData = {
    nodes: [
      { id: 0, kind: "Nonterminal", label: "Start", start: 0, end: 1, origin: "Start" },
      { id: 1, kind: "Amb", label: "Amb", start: 0, end: 1 },
      { id: 2, kind: "Nonterminal", label: "C", start: 0, end: 1 },
      { id: 4, kind: "Nonterminal", label: "C", start: 0, end: 1 },
      { id: 5, kind: "Token", label: "b", start: 0, end: 1 },
    ],
    edges: [
      { src: 0, dest: 1 }, { src: 1, dest: 2 }, { src: 2, dest: 1 },
      { src: 1, dest: 4 }, { src: 4, dest: 5 },
    ],
  };

  it("preserves a back-reference while removing the start wrapper", () => {
    const display = buildDisplayGraph(graph, { showLayout: false, showEmpty: false, showWrappers: false });
    expect(display.nodes.map(n => n.id)).toEqual([1, 2, 4, 5]);
    expect(display.edges).toEqual(graph.edges.slice(1));
  });

  it("renders a cyclic root even when every node has an incoming edge", () => {
    const inner = { nodes: graph.nodes.slice(1), edges: graph.edges.slice(1) };
    const display = buildDisplayGraph(inner, { showLayout: true, showEmpty: true, showWrappers: true });
    expect(display.nodes).toEqual(inner.nodes);
    expect(display.edges).toEqual(inner.edges);
  });

  it("draws the back-edge directly to its target", () => {
    const display = buildDisplayGraph(graph, { showLayout: false, showEmpty: false, showWrappers: false });
    const elements = buildParseTreeElements(display, false);
    const nodes = elements.filter(e => !e.data.source);
    const edges = elements.filter(e => e.data.source);
    expect(nodes.map(n => n.data.id)).toEqual(["n1", "n2", "n4", "n5"]);
    expect(edges.map(e => [e.data.source, e.data.target])).toEqual([
      ["n1", "n2"], ["n2", "n1"], ["n1", "n4"], ["n4", "n5"],
    ]);
  });

  it("reveals a node whose ancestor chain cycles back to the root", () => {
    const display = buildDisplayGraph(graph, { showLayout: false, showEmpty: false, showWrappers: false });
    const cy = cytoscape({
      headless: true,
      styleEnabled: true,
      elements: buildParseTreeElements(display, false),
    });
    try {
      const manager = new GraphCollapseManager();
      manager.setCy(cy);
      manager.toggleCollapse("n1");
      manager.toggleCollapse("n2");
      expect(cy.getElementById("n5").style("display")).toBe("none");

      manager.expandAncestors("n5");

      expect(manager.isCollapsed("n1")).toBe(false);
      expect(manager.isCollapsed("n2")).toBe(false);
      expect(cy.getElementById("n5").style("display")).toBe("element");
    } finally {
      cy.destroy();
    }
  });

  it("preserves multiple references to the same target", () => {
    const shared = { nodes: graph.nodes, edges: [...graph.edges, { src: 4, dest: 1 }] };
    const elements = buildParseTreeElements(shared, false);
    const edges = elements.filter(e => e.data.target === "n1");
    expect(edges.map(e => e.data.source)).toEqual(["n0", "n2", "n4"]);
    expect(elements.some(e => e.data.id === "n3" || e.data.target === "n3")).toBe(false);
  });

  it("splices wrappers on a cycle while preserving its ambiguity", () => {
    const wrappedTarget: ParseTreeData = {
      nodes: [
        { id: 0, kind: "Nonterminal", label: "C?", origin: "Opt", start: 0, end: 0 },
        { id: 1, kind: "Amb", label: "Amb", start: 0, end: 0 },
        { id: 2, kind: "Nonterminal", label: "C?", origin: "Opt", start: 0, end: 0 },
        { id: 3, kind: "Nonterminal", label: "Empty", start: 0, end: 0 },
      ],
      edges: [{ src: 0, dest: 1 }, { src: 1, dest: 2 }, { src: 2, dest: 0 }, { src: 1, dest: 3 }],
    };
    const display = buildDisplayGraph(wrappedTarget, { showLayout: false, showEmpty: false, showWrappers: false });
    expect(display.nodes.map(n => n.id)).toEqual([1, 3]);
    expect(display.edges).toEqual([{ src: 1, dest: 1 }, { src: 1, dest: 3 }]);
  });
});
