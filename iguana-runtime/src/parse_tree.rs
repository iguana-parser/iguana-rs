use std::fmt::{Debug, Write};

pub use bumpalo::Bump;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    ids::NonterminalId,
    parser::Parser,
    sppf::{NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
    utils::inline_vec::InlineVec,
};

/// Options for rendering a parse tree as an s-expression.
#[derive(Clone, Copy)]
pub struct SexprOptions {
    /// Include layout nodes (whitespace, comments) and their subtrees.
    pub show_layout: bool,
}

impl Default for SexprOptions {
    fn default() -> Self {
        SexprOptions { show_layout: true }
    }
}

pub struct ParseContext {
    bump: Bump,
}

impl ParseContext {
    pub fn new() -> Self {
        ParseContext { bump: Bump::new() }
    }

    pub fn bump(&self) -> &Bump {
        &self.bump
    }
}

impl Default for ParseContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum OneOrMany<T: Debug> {
    Zero,
    One(T),
    Many(Vec<T>),
    /// Multiple derivations under an ambiguous intermediate node, one child sequence
    /// per derivation.
    Multi(Vec<OneOrMany<T>>),
}

impl<T: Debug + Clone> OneOrMany<T> {
    pub fn merge(self, other: OneOrMany<T>) -> OneOrMany<T> {
        match (self, other) {
            (OneOrMany::One(l), OneOrMany::One(r)) => OneOrMany::Many(vec![l, r]),
            (OneOrMany::Many(mut l), OneOrMany::One(r)) => {
                l.push(r);
                OneOrMany::Many(l)
            }
            // Fan `r` across every alternative sequence. The right side of
            // an intermediate node is always a single grammar symbol, so
            // `r` is a `One` (never `Multi`) and the result stays flat.
            (OneOrMany::Multi(seqs), OneOrMany::One(r)) => OneOrMany::Multi(
                seqs.into_iter()
                    .map(|seq| seq.merge(OneOrMany::One(r.clone())))
                    .collect(),
            ),
            (OneOrMany::Zero, rhs) => rhs,
            (lhs, OneOrMany::Zero) => lhs,
            _ => unreachable!(""),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(item) => vec![item],
            OneOrMany::Many(items) => items,
            OneOrMany::Zero => vec![],
            OneOrMany::Multi(_) => unreachable!(),
        }
    }

    /// Destructure into a fixed-size array without allocating a `Vec` for the
    /// `Zero` and `One` cases. For `Many`, the existing `Vec` is consumed in place.
    ///
    /// The variant must match `N` (`Zero` ↔ 0, `One` ↔ 1, `Many` ↔ >1). The codegen
    /// guarantees this; any mismatch indicates a bug.
    pub fn into_array<const N: usize>(self) -> [T; N] {
        match self {
            OneOrMany::Zero => {
                if N != 0 {
                    unreachable!()
                }
                std::array::from_fn(|_| unreachable!())
            }
            OneOrMany::One(item) => {
                if N != 1 {
                    unreachable!()
                }
                let mut item = Some(item);
                std::array::from_fn(|_| item.take().unwrap())
            }
            OneOrMany::Many(items) => match <[T; N]>::try_from(items) {
                Ok(arr) => arr,
                Err(_) => unreachable!(),
            },
            OneOrMany::Multi(_) => unreachable!(),
        }
    }

    pub fn unwrap_one(self) -> T {
        match self {
            OneOrMany::One(item) => item,
            _ => panic!(),
        }
    }
}

/// True iff the SPPF rooted at `root_id` contains at least one reachable
/// ambiguous node. Entries in the parser's side maps record GLL machinery
/// state and can include ambiguous nodes that the accepted parse never
/// reaches, so an `is_empty` check on the maps is too coarse: it must be
/// followed by an actual DFS from the root. The empty-maps case is a fast
/// out covering most parses.
pub fn is_ambiguous<'i, P: Parser<'i>>(parser: &P, root_id: SPPFNodeId) -> bool {
    if parser.nonterminal_nodes_children_map().is_empty()
        && parser.intermediate_nodes_children_map().is_empty()
    {
        return false;
    }
    let mut visited = FxHashSet::default();
    let mut stack = vec![root_id];
    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        let node = parser.sppf_node(id);
        if node.is_ambiguous() {
            return true;
        }
        match node {
            SPPFNode::Nonterminal(n) => stack.push(n.child),
            SPPFNode::Intermediate(i) => {
                stack.push(i.child.0);
                stack.push(i.child.1);
            }
            SPPFNode::Terminal(_) => {}
        }
    }
    false
}

pub fn visit_sppf<'i, T: Debug + Clone, P: Parser<'i>>(
    node_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
) -> OneOrMany<T> {
    // Memoize across the SPPF only when ambiguity is reachable; otherwise
    // each node is visited at most once anyway, and the empty map plus its
    // per-node check would be pure overhead.
    let mut memo = if is_ambiguous(parser, node_id) {
        Some(FxHashMap::default())
    } else {
        None
    };
    visit_sppf_impl(node_id, parser, builder, &mut memo)
}

/// A frame in the explicit stack that replaces recursion in `visit_sppf_impl`.
struct Frame<T: Debug> {
    node_id: SPPFNodeId,
    children: InlineVec<SPPFNodeId>,
    next: usize, // index of the next child to visit
    results: InlineVec<OneOrMany<T>>,
}

impl<T: Debug> Frame<T> {
    fn new<'i>(node_id: SPPFNodeId, parser: &impl Parser<'i>) -> Self {
        Frame {
            node_id,
            children: parser.sppf_children(node_id),
            next: 0,
            results: InlineVec::Empty,
        }
    }
}

fn visit_sppf_impl<'i, T: Debug + Clone, P: Parser<'i>>(
    root_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
    memo: &mut Option<FxHashMap<SPPFNodeId, OneOrMany<T>>>,
) -> OneOrMany<T> {
    // A frame is fully built (and memoized) before its parent advances to the
    // next child, so a shared node's first visit completes before any later one
    // and children land on a frame's `results` in source order.
    let mut stack = vec![Frame::new(root_id, parser)];
    loop {
        let top = stack.len() - 1;
        match stack[top].children.get(stack[top].next).copied() {
            Some(child) => {
                stack[top].next += 1;
                // Reuse a child already built elsewhere (only possible once the
                // memo is active, i.e. when the parse is ambiguous).
                match memo.as_ref().and_then(|m| m.get(&child).cloned()) {
                    Some(cached) => stack[top].results.push(cached),
                    None => stack.push(Frame::new(child, parser)),
                }
            }
            None => {
                let frame = stack.pop().unwrap();
                let result = build_node(parser, builder, frame.node_id, frame.results);
                if let Some(m) = memo.as_mut() {
                    m.insert(frame.node_id, result.clone());
                }
                match stack.last_mut() {
                    Some(parent) => parent.results.push(result),
                    None => return result,
                }
            }
        }
    }
}

/// Builds a node's result from its children's results.
fn build_node<'i, T: Debug + Clone, P: Parser<'i>>(
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
    node_id: SPPFNodeId,
    results: InlineVec<OneOrMany<T>>,
) -> OneOrMany<T> {
    match parser.sppf_node(node_id) {
        SPPFNode::Terminal(t) => {
            if t.terminal_id == P::epsilon() {
                OneOrMany::Zero
            } else {
                OneOrMany::One(builder.new_token(t))
            }
        }
        SPPFNode::Nonterminal(n) => {
            let mut results = results.into_iter();
            let children = results.next().unwrap();
            if n.ambiguous {
                // Each remaining result is a separate derivation of this
                // nonterminal; collect one alternative per derivation.
                let mut alternatives = create_nonterminal_nodes(children, n, builder);
                let extras = parser
                    .nonterminal_nodes_children_map()
                    .get(&node_id)
                    .unwrap();
                for ((child, return_slot), derivation) in extras.iter().zip(results) {
                    let synthetic = NonterminalNode {
                        nonterminal_id: n.nonterminal_id,
                        return_slot: *return_slot,
                        span: n.span,
                        child: *child,
                        ambiguous: false,
                    };
                    alternatives.extend(create_nonterminal_nodes(derivation, &synthetic, builder));
                }
                OneOrMany::One(builder.new_ambiguity_node(n.nonterminal_id, alternatives))
            } else if matches!(children, OneOrMany::Multi(_)) {
                // `Multi` means derivations from an ambiguous intermediate node
                // below bubbled up and need the same `Amb` wrapping.
                let alternatives = create_nonterminal_nodes(children, n, builder);
                OneOrMany::One(builder.new_ambiguity_node(n.nonterminal_id, alternatives))
            } else {
                OneOrMany::One(builder.new_nonterminal_node(n, children))
            }
        }
        SPPFNode::Intermediate(i) => {
            if !i.ambiguous {
                let mut results = results.into_iter();
                let left = results.next().unwrap();
                let right = results.next().unwrap();
                left.merge(right)
            } else {
                // The flattened results re-pair into one derivation per
                // `(left, right)`. A deeper ambiguity may already have produced a
                // `Multi`, so flatten to keep the outer `Multi` one level deep.
                let mut derivations: Vec<OneOrMany<T>> = Vec::with_capacity(results.len() / 2);
                let mut results = results.into_iter();
                while let (Some(left), Some(right)) = (results.next(), results.next()) {
                    match left.merge(right) {
                        OneOrMany::Multi(inner) => derivations.extend(inner),
                        flat => derivations.push(flat),
                    }
                }
                OneOrMany::Multi(derivations)
            }
        }
    }
}

/// Builds one parse-tree node per derivation. When the children arrive
/// as a `Multi` from an ambiguous intermediate node, there are several
/// derivations of the same alternative and the resulting nodes go into
/// an `Amb`.
fn create_nonterminal_nodes<T: Debug>(
    children: OneOrMany<T>,
    node: &NonterminalNode,
    builder: &impl ParseTreeBuilder<T>,
) -> Vec<T> {
    match children {
        OneOrMany::Multi(derivations) => derivations
            .into_iter()
            .map(|d| builder.new_nonterminal_node(node, d))
            .collect(),
        single => vec![builder.new_nonterminal_node(node, single)],
    }
}

pub trait ParseTreeBuilder<T: Debug> {
    fn new_token(&self, terminal_node: &TerminalNode) -> T;
    fn new_nonterminal_node(&self, nonterminal_node: &NonterminalNode, children: OneOrMany<T>)
    -> T;
    fn new_ambiguity_node(&self, parent: NonterminalId, alternatives: Vec<T>) -> T {
        let _ = (parent, alternatives);
        unimplemented!("ambiguity handling not yet implemented for this builder")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Token,
    Nonterminal,
    Amb,
}

/// The uniform interface for generic parse-tree traversals, such as conversion
/// to an s-expression or JSON. Generated parsers implement it for their
/// `ParseTree` type.
pub trait ParseTreeNode: Copy {
    fn children(&self) -> Vec<Self>;
    fn display_name(&self) -> &'static str;
    fn span(&self) -> Span;
    fn kind(&self) -> NodeKind;
    /// A stable identity for the node, or `None` for nodes that are never
    /// shared. Used to detect sharing in an ambiguity DAG.
    fn node_id(&self) -> Option<usize>;
}

/// Whether `walk` should descend into a node's children.
pub enum Visit {
    Children,
    /// Skip the children: already handled inline, hidden, or seen before.
    Skip,
}

/// Handles the two events `walk` raises for each node. `enter` runs before a
/// node's children and decides whether to descend; `exit` runs after the
/// children of a descended node.
pub trait TreeVisitor<N: ParseTreeNode> {
    fn enter(&mut self, node: N) -> Visit;
    fn exit(&mut self, node: N) {
        let _ = node;
    }
}

/// Walks a parse tree depth-first, driving `visitor`. The s-expression and JSON
/// renderings are built on this.
pub fn walk<N: ParseTreeNode, V: TreeVisitor<N>>(root: N, visitor: &mut V) {
    enum WalkEvent<N> {
        Enter(N),
        Exit(N),
    }
    let mut stack = vec![WalkEvent::Enter(root)];
    while let Some(event) = stack.pop() {
        match event {
            WalkEvent::Enter(node) => {
                if let Visit::Children = visitor.enter(node) {
                    // Exit goes under the children so it runs after them; the
                    // children are reversed so they pop in source order.
                    stack.push(WalkEvent::Exit(node));
                    for child in node.children().into_iter().rev() {
                        stack.push(WalkEvent::Enter(child));
                    }
                }
            }
            WalkEvent::Exit(node) => visitor.exit(node),
        }
    }
}

/// Renders a parse tree as a JSON graph of nodes and edges, for visualization.
/// A node shared in an ambiguity DAG is emitted once; later parents get an edge
/// to the existing node instead of a re-expanded subtree.
pub fn to_json<N: ParseTreeNode>(root: N, layout_name: Option<&str>) -> String {
    let mut builder = JsonBuilder::default();
    walk(root, &mut builder);
    serde_json::json!({
        "layout_name": layout_name,
        "nodes": builder.nodes,
        "edges": builder.edges,
    })
    .to_string()
}

#[derive(Default)]
struct JsonBuilder {
    nodes: Vec<serde_json::Value>,
    edges: Vec<serde_json::Value>,
    ids: FxHashMap<usize, u32>,
    next_id: u32,
    ancestors: Vec<u32>,
}

impl JsonBuilder {
    fn edge_from_parent(&mut self, child: u32) {
        if let Some(&parent) = self.ancestors.last() {
            self.edges
                .push(serde_json::json!({ "src": parent, "dest": child }));
        }
    }
}

impl<N: ParseTreeNode> TreeVisitor<N> for JsonBuilder {
    fn enter(&mut self, node: N) -> Visit {
        // A node already seen (shared in an ambiguity DAG) reuses its id and is
        // linked but not re-expanded.
        if let Some(&id) = node.node_id().and_then(|key| self.ids.get(&key)) {
            self.edge_from_parent(id);
            return Visit::Skip;
        }
        let id = self.next_id;
        self.next_id += 1;
        if let Some(key) = node.node_id() {
            self.ids.insert(key, id);
        }
        let span = node.span();
        let kind = match node.kind() {
            NodeKind::Token => "Token",
            NodeKind::Nonterminal => "Nonterminal",
            NodeKind::Amb => "Amb",
        };
        self.nodes.push(serde_json::json!({
            "id": id,
            "kind": kind,
            "label": node.display_name(),
            "start": span.left_extent,
            "end": span.right_extent,
        }));
        self.edge_from_parent(id);
        self.ancestors.push(id);
        Visit::Children
    }

    fn exit(&mut self, _node: N) {
        self.ancestors.pop();
    }
}

/// Renders a parse tree as an s-expression with default options.
pub fn to_sexpr<N: ParseTreeNode>(root: N, layout_name: Option<&str>) -> String {
    to_sexpr_with(root, layout_name, SexprOptions::default())
}

/// Renders a parse tree as an s-expression. A first pass counts how many parents
/// reach each node; a node reached by more than one is written once with a `#N=`
/// label and referenced as `#N#` elsewhere, so a shared forest stays bounded.
pub fn to_sexpr_with<N: ParseTreeNode>(
    root: N,
    layout_name: Option<&str>,
    options: SexprOptions,
) -> String {
    let mut counter = IndegreeCounter {
        layout_name,
        options,
        indegree: FxHashMap::default(),
        visited: FxHashSet::default(),
    };
    walk(root, &mut counter);

    let mut printer = SexprPrinter {
        layout_name,
        options,
        indegree: counter.indegree,
        labels: FxHashMap::default(),
        next_label: 1,
        out: String::new(),
        indent: 0,
        wrote_root: false,
    };
    walk(root, &mut printer);
    printer.out.push('\n');
    printer.out
}

/// A layout node (whitespace, comments) is hidden, with its subtree, unless the
/// caller asks for it.
fn sexpr_hidden<N: ParseTreeNode>(
    node: N,
    layout_name: Option<&str>,
    options: SexprOptions,
) -> bool {
    !options.show_layout && layout_name == Some(node.display_name())
}

fn visible_children<N: ParseTreeNode>(
    node: N,
    layout_name: Option<&str>,
    options: SexprOptions,
) -> Vec<N> {
    node.children()
        .into_iter()
        .filter(|c| !sexpr_hidden(*c, layout_name, options))
        .collect()
}

struct IndegreeCounter<'n> {
    layout_name: Option<&'n str>,
    options: SexprOptions,
    indegree: FxHashMap<usize, u32>,
    visited: FxHashSet<usize>,
}

impl<N: ParseTreeNode> TreeVisitor<N> for IndegreeCounter<'_> {
    fn enter(&mut self, node: N) -> Visit {
        if sexpr_hidden(node, self.layout_name, self.options) {
            return Visit::Skip;
        }
        if let Some(id) = node.node_id() {
            *self.indegree.entry(id).or_insert(0) += 1;
            // Count every parent's reference, but expand the subtree only once.
            if !self.visited.insert(id) {
                return Visit::Skip;
            }
        }
        Visit::Children
    }
}

struct SexprPrinter<'n> {
    layout_name: Option<&'n str>,
    options: SexprOptions,
    indegree: FxHashMap<usize, u32>,
    labels: FxHashMap<usize, u32>,
    next_label: u32,
    out: String,
    indent: usize,
    wrote_root: bool,
}

impl SexprPrinter<'_> {
    /// Writes a `#N#` back-reference (returning `true`) for a shared node already
    /// seen, or a `#N=` label prefix (returning `false`) the first time a shared
    /// node is written. An unshared node writes nothing and returns `false`.
    fn write_label<N: ParseTreeNode>(&mut self, node: N) -> bool {
        if let Some(id) = node.node_id() {
            if self.indegree.get(&id).copied().unwrap_or(0) > 1 {
                if let Some(&label) = self.labels.get(&id) {
                    let _ = write!(self.out, "#{label}#");
                    return true;
                }
                let label = self.next_label;
                self.next_label += 1;
                self.labels.insert(id, label);
                let _ = write!(self.out, "#{label}=");
            }
        }
        false
    }
}

impl<N: ParseTreeNode> TreeVisitor<N> for SexprPrinter<'_> {
    fn enter(&mut self, node: N) -> Visit {
        if sexpr_hidden(node, self.layout_name, self.options) {
            return Visit::Skip;
        }
        // Every node but the root sits on its own indented line. Leaf and
        // one-line nodes are written by their parent, so a node reaches here
        // only as the root or a block child.
        if self.wrote_root {
            let _ = write!(self.out, "\n{:indent$}", "", indent = self.indent);
        } else {
            self.wrote_root = true;
        }
        if self.write_label(node) {
            return Visit::Skip;
        }
        let children = visible_children(node, self.layout_name, self.options);
        if children.is_empty() {
            let _ = write!(self.out, "{}", node.display_name());
            Visit::Skip
        } else if children
            .iter()
            .all(|c| visible_children(*c, self.layout_name, self.options).is_empty())
        {
            // Every child is a leaf, so the node fits on one line.
            let _ = write!(self.out, "({}", node.display_name());
            for child in children {
                let _ = write!(self.out, " ");
                if !self.write_label(child) {
                    let _ = write!(self.out, "{}", child.display_name());
                }
            }
            let _ = write!(self.out, ")");
            Visit::Skip
        } else {
            let _ = write!(self.out, "({}", node.display_name());
            self.indent += 2;
            Visit::Children
        }
    }

    fn exit(&mut self, _node: N) {
        self.indent -= 2;
        let _ = write!(self.out, ")");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal parse tree for exercising the generic traversals. A node
    /// borrows its children so a tree can be built without the real builder.
    #[derive(Clone, Copy)]
    struct Node<'a> {
        name: &'static str,
        id: usize,
        kind: NodeKind,
        children: &'a [Node<'a>],
    }

    impl<'a> ParseTreeNode for Node<'a> {
        fn children(&self) -> Vec<Self> {
            self.children.to_vec()
        }
        fn display_name(&self) -> &'static str {
            self.name
        }
        fn span(&self) -> Span {
            Span::new(0, 0)
        }
        fn kind(&self) -> NodeKind {
            self.kind
        }
        fn node_id(&self) -> Option<usize> {
            Some(self.id)
        }
    }

    fn leaf(name: &'static str, id: usize) -> Node<'static> {
        Node {
            name,
            id,
            kind: NodeKind::Token,
            children: &[],
        }
    }

    #[test]
    fn to_json_emits_nodes_and_edges() {
        let kids = [leaf("a", 1), leaf("b", 2)];
        let root = Node {
            name: "S",
            id: 0,
            kind: NodeKind::Nonterminal,
            children: &kids,
        };
        let json = to_json(root, Some("Layout"));
        assert!(json.contains("\"label\":\"S\""));
        assert!(json.contains("\"label\":\"a\""));
        assert!(json.contains("\"kind\":\"Token\""));
        assert!(json.contains("\"layout_name\":\"Layout\""));
    }

    #[test]
    fn to_sexpr_formats_nested_tree() {
        // S -> [A -> [x, y], z]: S is multi-line, A folds onto one line.
        let a_kids = [leaf("x", 1), leaf("y", 2)];
        let a = Node {
            name: "A",
            id: 3,
            kind: NodeKind::Nonterminal,
            children: &a_kids,
        };
        let top = [a, leaf("z", 4)];
        let s = Node {
            name: "S",
            id: 0,
            kind: NodeKind::Nonterminal,
            children: &top,
        };
        assert_eq!(to_sexpr(s, None), "(S\n  (A x y)\n  z)\n");
    }

    #[test]
    fn to_sexpr_hides_layout_subtree() {
        // S is multi-line, with a hidden Layout child between two visible ones;
        // hiding it must not leave a blank line where the node would have been.
        let x = [leaf("x", 1)];
        let a = Node {
            name: "A",
            id: 2,
            kind: NodeKind::Nonterminal,
            children: &x,
        };
        let ws = [leaf("ws", 5)];
        let layout = Node {
            name: "Layout",
            id: 6,
            kind: NodeKind::Nonterminal,
            children: &ws,
        };
        let kids = [a, layout, leaf("b", 3)];
        let s = Node {
            name: "S",
            id: 0,
            kind: NodeKind::Nonterminal,
            children: &kids,
        };

        let hidden = SexprOptions { show_layout: false };
        assert_eq!(
            to_sexpr_with(s, Some("Layout"), hidden),
            "(S\n  (A x)\n  b)\n"
        );
        let shown = SexprOptions { show_layout: true };
        assert_eq!(
            to_sexpr_with(s, Some("Layout"), shown),
            "(S\n  (A x)\n  (Layout ws)\n  b)\n"
        );
    }

    #[test]
    fn walk_handles_deeply_nested_tree() {
        // A left-deep chain this deep would overflow a recursive walk.
        struct Counter(usize);
        impl<'a> TreeVisitor<Node<'a>> for Counter {
            fn enter(&mut self, _node: Node<'a>) -> Visit {
                self.0 += 1;
                Visit::Children
            }
        }

        let bump = Bump::new();
        let mut node = leaf("leaf", 0);
        for i in 1..200_000 {
            let children: &[Node] = bump.alloc_slice_copy(&[node]);
            node = Node {
                name: "N",
                id: i,
                kind: NodeKind::Nonterminal,
                children,
            };
        }

        let mut counter = Counter(0);
        walk(node, &mut counter);
        assert_eq!(counter.0, 200_000);
    }
}
