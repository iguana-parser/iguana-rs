import type { ParseTreeData, ParseTreeNodeData } from "./parse-tree-graph";

// Parses a printed parse-tree s-expression back into the ParseTreeData DAG the
// graph views consume: the inverse of the generated to_sexpr (mirrored by the
// parse view's sexprToText). The parser exists for hosts that only have the
// printed tree: the docs site renders its verified .sexpr snippets as graphs
// without shipping a parser wasm per page.
//
// The printed form is not whitespace-tokenizable: a wrapper node's label can
// embed spaces, quotes, and brackets ({Num ","}*, (A B)?), so atoms are scanned
// with balanced-bracket and string awareness instead of split on whitespace.
// Sharing follows the printer: #N= labels a node at its first occurrence and
// #N# refers back to it, which here becomes an extra edge to the same node.
//
// Spans are not part of the printed form, so every node gets start = end = 0;
// callers that render from this data must not show spans.

export function parseSexprTree(text: string): ParseTreeData {
  const nodes: ParseTreeNodeData[] = [];
  const edges: { src: number; dest: number }[] = [];
  const shared = new Map<number, number>(); // share label N -> node id
  let pos = 0;
  let nextId = 0;

  function fail(message: string): never {
    throw new Error(`s-expression parse error at offset ${pos}: ${message}`);
  }

  function skipWhitespace() {
    while (pos < text.length && /\s/.test(text[pos])) pos++;
  }

  // Consumes a double-quoted string (backslash escapes included), returning it
  // with its quotes, which is also how the token should read in a graph label.
  function scanString(): string {
    const start = pos;
    pos++; // opening quote
    while (pos < text.length) {
      const c = text[pos];
      if (c === "\\") pos += 2;
      else if (c === '"') { pos++; return text.slice(start, pos); }
      else pos++;
    }
    fail("unterminated string");
  }

  // Consumes a balanced bracket group, string-aware, so a quote inside the
  // group cannot unbalance it and a bracket inside a string does not count.
  function scanBalanced(open: string, close: string): string {
    const start = pos;
    let depth = 0;
    while (pos < text.length) {
      const c = text[pos];
      if (c === '"') { scanString(); continue; }
      if (c === open) depth++;
      else if (c === close) {
        depth--;
        if (depth === 0) { pos++; return text.slice(start, pos); }
      }
      pos++;
    }
    fail(`unbalanced ${open}${close} group`);
  }

  // Consumes one label atom: a run of segments up to top-level whitespace or a
  // closing paren. Brace groups and strings are consumed whole, so labels like
  // {Num ","}* stay one atom. `allowParenGroup` admits a leading balanced
  // (...) segment, used where a group label is known to sit (a node's head, or
  // a leaf already identified by its ?/*/+ suffix).
  function scanAtom(allowParenGroup: boolean): string {
    const start = pos;
    if (allowParenGroup && text[pos] === "(") scanBalanced("(", ")");
    while (pos < text.length) {
      const c = text[pos];
      if (/\s/.test(c) || c === ")") break;
      if (c === "(") break;
      if (c === "{") { scanBalanced("{", "}"); continue; }
      if (c === '"') { scanString(); continue; }
      pos++;
    }
    if (pos === start) fail("expected an atom");
    return text.slice(start, pos);
  }

  function addNode(label: string, kind: ParseTreeNodeData["kind"]): number {
    const id = nextId++;
    nodes.push({ id, kind, label, start: 0, end: 0 });
    return id;
  }

  // Reads a `#N=` share label or `#N#` reference if one starts here. Returns
  // the numeric label plus which form it was, or null when the `#` starts an
  // ordinary atom (a label may begin with `#` in principle).
  function scanShareMark(): { n: number; kind: "def" | "ref" } | null {
    const m = /^#(\d+)([=#])/.exec(text.slice(pos));
    if (!m) return null;
    pos += m[0].length;
    return { n: Number(m[1]), kind: m[2] === "=" ? "def" : "ref" };
  }

  // Whether the balanced (...) group starting here is immediately followed by
  // a ?/*/+ suffix, which makes it a leaf's group label (an empty (A B)?
  // prints bare in child position) rather than a subtree.
  function parenGroupIsLeafLabel(): boolean {
    const saved = pos;
    scanBalanced("(", ")");
    const suffixed = pos < text.length && /[?*+]/.test(text[pos]);
    pos = saved;
    return suffixed;
  }

  // Parses one unit (a subtree, a leaf, or a reference) and returns the id
  // of the node it denotes.
  function parseUnit(): number {
    skipWhitespace();
    if (pos >= text.length) fail("unexpected end of input");

    const mark = scanShareMark();
    if (mark?.kind === "ref") {
      const id = shared.get(mark.n);
      if (id === undefined) fail(`reference #${mark.n}# before its definition`);
      return id;
    }

    let id: number;
    if (text[pos] === "(" && !parenGroupIsLeafLabel()) {
      pos++; // opening paren
      skipWhitespace();
      const label = scanAtom(true);
      id = addNode(label, label === "Amb" ? "Amb" : "Nonterminal");
      if (mark) shared.set(mark.n, id);
      skipWhitespace();
      while (pos < text.length && text[pos] !== ")") {
        const child = parseUnit();
        edges.push({ src: id, dest: child });
        skipWhitespace();
      }
      if (text[pos] !== ")") fail("expected )");
      pos++;
    } else if (text[pos] === '"') {
      id = addNode(scanString(), "Token");
      if (mark) shared.set(mark.n, id);
    } else {
      id = addNode(scanAtom(true), "Nonterminal");
      if (mark) shared.set(mark.n, id);
    }
    return id;
  }

  parseUnit();
  skipWhitespace();
  if (pos < text.length) fail("trailing content after the tree");
  return { layout_name: null, nodes, edges };
}
