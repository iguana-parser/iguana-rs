use std::fmt::{Debug, Write};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    ids::NonterminalId,
    input::Span,
    parser::Parser,
    sppf::{NonterminalNode, SPPFNode, SPPFNodeId, TerminalNode},
    utils::inline_vec::InlineVec,
};

/// Which constructs a rendered parse tree keeps. Each field is an independent
/// presentation toggle: `true` shows the construct as it sits in the real parse
/// tree, `false` simplifies it away. The `default` is every toggle on (the
/// truthful, lossless tree), so `to_sexpr` and the golden files stay faithful.
/// The CLI, REPL, and viewers start from `simplified` instead.
#[derive(Clone, Copy)]
pub struct DisplayOptions {
    /// Show layout nodes (whitespace, comments) and their subtrees.
    pub show_layout: bool,
    /// Show empty optionals and repetitions (`X?`, `X*` that matched nothing)
    /// rather than dropping them.
    pub show_empty: bool,
    /// Show wrapper nodes (the start wrapper, optionals, anonymous groups,
    /// and alternations) rather than splicing their children into the parent.
    pub show_wrappers: bool,
}

impl Default for DisplayOptions {
    /// The truthful tree: every construct shown as it really is.
    fn default() -> Self {
        DisplayOptions {
            show_layout: true,
            show_empty: true,
            show_wrappers: true,
        }
    }
}

impl DisplayOptions {
    /// The clean view the CLI, REPL, and viewers default to: layout, empty
    /// optionals and repetitions, and wrapper nodes hidden.
    pub fn simplified() -> Self {
        DisplayOptions {
            show_layout: false,
            show_empty: false,
            show_wrappers: false,
        }
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
pub fn is_ambiguous<'i, 'arena, P: Parser<'i, 'arena>>(parser: &P, root_id: SPPFNodeId) -> bool {
    // The unsafe mode records no ambiguity, so the answer is known at
    // compile time and the walk below is dead code.
    if P::UNSAFE {
        return false;
    }
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

pub fn visit_sppf<'i, 'arena, T: Debug + Clone, P: Parser<'i, 'arena>>(
    node_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
) -> OneOrMany<T> {
    // The unsafe mode guarantees one derivation per node, so the specialized
    // walker skips the memo and the OneOrMany machinery entirely.
    if P::UNSAFE {
        return OneOrMany::One(visit_sppf_unambiguous(node_id, parser, builder));
    }
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

/// A frame in the unambiguous walker: `Pre` visits a node, `Post` builds a
/// nonterminal node from its children. A nonterminal node's `Pre` pushes the
/// node's `Post` first and the child's `Pre` on top of it, so the `Post` runs
/// once the whole subtree is done. Tokens are built at their `Pre` visit and
/// need no `Post`.
enum UnambiguousFrame<'p> {
    Pre(SPPFNodeId),
    /// `mark` is `values.len()` at the moment a `Post` frame is pushed. Everything
    /// the subtree pushes is placed above it, so at build time `values[mark..]` is
    /// exactly the node's children, in order.
    Post {
        node: &'p NonterminalNode,
        mark: usize,
    },
}

/// Builds the parse tree for an unambiguous SPPF, where every node has
/// exactly one derivation. The walk keeps two explicit stacks, work frames
/// and finished values. A token pushes one value, an intermediate node
/// contributes no value of its own (it only pushes `Pre` frames for its two
/// children), and a nonterminal node records the value-stack depth on entry
/// (the `Post` frame's `mark`), so on completion the values above the mark
/// are exactly its children, handed to the builder as a slice.
fn visit_sppf_unambiguous<'i, 'arena, T: Debug + Clone, P: Parser<'i, 'arena>>(
    root_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
) -> T {
    let mut work = vec![UnambiguousFrame::Pre(root_id)];
    let mut values: Vec<T> = Vec::new();
    while let Some(frame) = work.pop() {
        match frame {
            UnambiguousFrame::Pre(node_id) => match parser.sppf_node(node_id) {
                SPPFNode::Terminal(t) => {
                    if t.terminal_id != P::epsilon() {
                        values.push(builder.new_token(t));
                    }
                }
                SPPFNode::Nonterminal(n) => {
                    work.push(UnambiguousFrame::Post {
                        node: n,
                        mark: values.len(),
                    });
                    work.push(UnambiguousFrame::Pre(n.child));
                }
                SPPFNode::Intermediate(i) => {
                    // The right child is pushed first so the left child pops
                    // first and the values land in source order.
                    work.push(UnambiguousFrame::Pre(i.child.1));
                    work.push(UnambiguousFrame::Pre(i.child.0));
                }
            },
            UnambiguousFrame::Post { node, mark } => {
                let result = builder.new_unambiguous_nonterminal_node(node, &values[mark..]);
                values.truncate(mark);
                values.push(result);
            }
        }
    }
    assert_eq!(values.len(), 1);
    values.pop().unwrap()
}

/// A frame in the explicit stack that replaces recursion in `visit_sppf_impl`.
struct Frame<'arena, T: Debug> {
    node_id: SPPFNodeId,
    children: InlineVec<'arena, SPPFNodeId>,
    next: usize, // index of the next child to visit
    results: InlineVec<'arena, OneOrMany<T>>,
}

impl<'arena, T: Debug> Frame<'arena, T> {
    fn new<'i>(node_id: SPPFNodeId, parser: &impl Parser<'i, 'arena>) -> Self {
        Frame {
            node_id,
            children: parser.sppf_children(node_id),
            next: 0,
            results: InlineVec::Empty,
        }
    }
}

fn visit_sppf_impl<'i, 'arena, T: Debug + Clone, P: Parser<'i, 'arena>>(
    root_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
    memo: &mut Option<FxHashMap<SPPFNodeId, OneOrMany<T>>>,
) -> OneOrMany<T> {
    // A frame is fully built (and memoized) before its parent advances to the
    // next child, so a shared node's first visit completes before any later one
    // and children land on a frame's `results` in source order.
    let arena = parser.vec_arena();
    let mut stack = vec![Frame::new(root_id, parser)];
    loop {
        let top = stack.len() - 1;
        match stack[top].children.get(stack[top].next).copied() {
            Some(child) => {
                stack[top].next += 1;
                // Reuse a child already built elsewhere (only possible once the
                // memo is active, i.e. when the parse is ambiguous).
                match memo.as_ref().and_then(|m| m.get(&child).cloned()) {
                    Some(cached) => stack[top].results.push(cached, arena),
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
                    Some(parent) => parent.results.push(result, arena),
                    None => return result,
                }
            }
        }
    }
}

/// Builds a node's result from its children's results.
fn build_node<'i, 'arena, T: Debug + Clone, P: Parser<'i, 'arena>>(
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
    node_id: SPPFNodeId,
    results: InlineVec<'arena, OneOrMany<T>>,
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
            // The unsafe mode never marks a node ambiguous and never produces
            // a `Multi`, so the const guards compile both Amb arms out.
            if !P::UNSAFE && n.ambiguous {
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
            } else if !P::UNSAFE && matches!(children, OneOrMany::Multi(_)) {
                // `Multi` means derivations from an ambiguous intermediate node
                // below bubbled up and need the same `Amb` wrapping.
                let alternatives = create_nonterminal_nodes(children, n, builder);
                OneOrMany::One(builder.new_ambiguity_node(n.nonterminal_id, alternatives))
            } else {
                OneOrMany::One(builder.new_nonterminal_node(n, children))
            }
        }
        SPPFNode::Intermediate(i) => {
            if P::UNSAFE || !i.ambiguous {
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
    /// Builds a nonterminal node from its children, wrapped in `OneOrMany`
    /// because an ambiguous parse can deliver several derivations at once.
    /// Only default-mode builders implement it; the unsafe mode's walker
    /// never calls it, so there the default covers the signature.
    fn new_nonterminal_node(
        &self,
        nonterminal_node: &NonterminalNode,
        children: OneOrMany<T>,
    ) -> T {
        let _ = (nonterminal_node, children);
        unimplemented!("only default-mode builders implement this")
    }
    /// Builds a nonterminal node from its children, which arrive as a plain
    /// slice: with one derivation per node there are no `OneOrMany` variants
    /// to distinguish. Only unsafe-mode builders implement it; the default
    /// mode's walker never calls it, so there the default covers the
    /// signature.
    fn new_unambiguous_nonterminal_node(
        &self,
        nonterminal_node: &NonterminalNode,
        children: &[T],
    ) -> T {
        let _ = (nonterminal_node, children);
        unimplemented!(
            "only unsafe-mode builders implement this; a panic here means the parser \
             was generated before the unsafe-mode walker and must be regenerated"
        )
    }
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

/// The grammar construct a nonterminal node was derived from. Drives the
/// presentation transforms (hiding empty repetitions, splicing wrappers).
/// `None` for a user-declared nonterminal or a token.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The synthetic start wrapper, the start symbol between leading and
    /// trailing layout.
    Start,
    /// An optional, `X?`.
    Opt,
    /// A repetition, `X*` / `X+` / `{X sep}+`.
    List,
    /// An anonymous sequence group, `(A B C)`.
    Group,
    /// An anonymous alternation, `(A | B)`.
    Alt,
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
    /// The grammar construct this node was derived from, or `None` for a
    /// user-declared nonterminal or a token. The default keeps parsers
    /// generated before this method was added compiling. Those parsers report
    /// `None`, so the presentation transforms are no-ops until regeneration.
    fn origin(&self) -> Option<Origin> {
        None
    }

    /// Whether any node in this subtree is ambiguous.
    fn contains_ambiguity(&self) -> bool {
        let mut stack = vec![*self];
        while let Some(node) = stack.pop() {
            if node.kind() == NodeKind::Amb {
                return true;
            }
            stack.extend(node.children());
        }
        false
    }
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
    /// The children `walk` should descend into. Defaults to the node's real
    /// children; the s-expression visitors override it to descend into the
    /// transformed child list (layout hidden, empties dropped, wrappers
    /// spliced).
    fn children(&self, node: N) -> Vec<N> {
        node.children()
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
                    for child in visitor.children(node).into_iter().rev() {
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

fn origin_name(origin: Option<Origin>) -> Option<&'static str> {
    origin.map(|o| match o {
        Origin::Start => "Start",
        Origin::Opt => "Opt",
        Origin::List => "List",
        Origin::Group => "Group",
        Origin::Alt => "Alt",
    })
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
            "origin": origin_name(node.origin()),
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
    to_sexpr_with(root, layout_name, DisplayOptions::default())
}

/// Renders a parse tree as an s-expression. A first pass counts how many parents
/// reach each node; a node reached by more than one is written once with a `#N=`
/// label and referenced as `#N#` elsewhere, so a shared forest stays bounded.
pub fn to_sexpr_with<N: ParseTreeNode>(
    root: N,
    layout_name: Option<&str>,
    options: DisplayOptions,
) -> String {
    let root = display_root(root, layout_name, options);

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

/// A layout node (whitespace, comments) that `show_layout = false` hides, with
/// its subtree.
fn is_hidden_layout<N: ParseTreeNode>(
    node: N,
    layout_name: Option<&str>,
    options: DisplayOptions,
) -> bool {
    !options.show_layout && layout_name == Some(node.display_name())
}

/// A wrapper node carries no information of its own: the start-wrapper
/// scaffolding, an optional, or an anonymous group or alternation.
/// `show_wrappers = false` splices its children into the parent.
fn is_wrapper(origin: Option<Origin>) -> bool {
    matches!(
        origin,
        Some(Origin::Start | Origin::Opt | Origin::Group | Origin::Alt)
    )
}

/// An empty optional or list: an `X?`, `X*`, or `{X sep}+` that matched
/// nothing. The `show_empty` toggle decides whether these nodes appear.
fn is_empty_opt_or_list<N: ParseTreeNode>(node: N) -> bool {
    matches!(node.origin(), Some(Origin::Opt | Origin::List)) && node.children().is_empty()
}

/// The children of `node` that remain after filtering:
///
/// - `show_layout = false` drops layout nodes, with their subtrees.
/// - `show_empty = false` drops empty optionals and repetitions.
/// - `show_wrappers = false` replaces a wrapper node with its own display
///   children. A chain of wrappers collapses in one pass.
fn display_children<N: ParseTreeNode>(
    node: N,
    layout_name: Option<&str>,
    options: DisplayOptions,
) -> Vec<N> {
    let mut result = Vec::new();
    for child in node.children() {
        if is_hidden_layout(child, layout_name, options) {
            continue;
        }
        // An empty `X?` is both empty and a wrapper, so `show_empty` decides it
        // here, before the wrapper splice below can reach it. Splicing replaces
        // a node with its children, and an empty node has none, so the splice
        // would delete an empty `X?` that `show_empty = true` keeps.
        if is_empty_opt_or_list(child) {
            if options.show_empty {
                result.push(child);
            }
            continue;
        }
        if !options.show_wrappers && is_wrapper(child.origin()) {
            result.extend(display_children(child, layout_name, options));
        } else {
            result.push(child);
        }
    }
    result
}

/// The root of the filtered tree. When wrappers are spliced and the real root
/// is one (typically the start-wrapper node), descend to the single node it
/// wraps so the output is not headed by scaffolding.
fn display_root<N: ParseTreeNode>(
    root: N,
    layout_name: Option<&str>,
    options: DisplayOptions,
) -> N {
    let mut node = root;
    while !options.show_wrappers && is_wrapper(node.origin()) {
        match display_children(node, layout_name, options).as_slice() {
            [only] => node = *only,
            _ => break,
        }
    }
    node
}

struct IndegreeCounter<'n> {
    layout_name: Option<&'n str>,
    options: DisplayOptions,
    indegree: FxHashMap<usize, u32>,
    visited: FxHashSet<usize>,
}

impl<N: ParseTreeNode> TreeVisitor<N> for IndegreeCounter<'_> {
    fn enter(&mut self, node: N) -> Visit {
        if let Some(id) = node.node_id() {
            *self.indegree.entry(id).or_insert(0) += 1;
            // Count every parent's reference, but expand the subtree only once.
            if !self.visited.insert(id) {
                return Visit::Skip;
            }
        }
        Visit::Children
    }

    fn children(&self, node: N) -> Vec<N> {
        display_children(node, self.layout_name, self.options)
    }
}

struct SexprPrinter<'n> {
    layout_name: Option<&'n str>,
    options: DisplayOptions,
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
        let children = display_children(node, self.layout_name, self.options);
        if children.is_empty() {
            let _ = write!(self.out, "{}", node.display_name());
            Visit::Skip
        } else if children
            .iter()
            .all(|c| display_children(*c, self.layout_name, self.options).is_empty())
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

    fn children(&self, node: N) -> Vec<N> {
        display_children(node, self.layout_name, self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::Arena;

    /// A minimal parse tree for exercising the generic traversals. A node
    /// borrows its children so a tree can be built without the real builder.
    #[derive(Clone, Copy)]
    struct Node<'a> {
        name: &'static str,
        id: usize,
        kind: NodeKind,
        origin: Option<Origin>,
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
        fn origin(&self) -> Option<Origin> {
            self.origin
        }
    }

    /// A user-declared nonterminal (no origin) with the given children.
    fn nonterminal<'a>(name: &'static str, id: usize, children: &'a [Node<'a>]) -> Node<'a> {
        Node {
            name,
            id,
            kind: NodeKind::Nonterminal,
            origin: None,
            children,
        }
    }

    /// A derived nonterminal carrying a presentation `origin`.
    fn derived<'a>(
        name: &'static str,
        id: usize,
        origin: Origin,
        children: &'a [Node<'a>],
    ) -> Node<'a> {
        Node {
            name,
            id,
            kind: NodeKind::Nonterminal,
            origin: Some(origin),
            children,
        }
    }

    /// An ambiguity cluster: kind `Amb`, no presentation origin, with the
    /// alternative derivations as children.
    fn amb<'a>(id: usize, children: &'a [Node<'a>]) -> Node<'a> {
        Node {
            name: "Amb",
            id,
            kind: NodeKind::Amb,
            origin: None,
            children,
        }
    }

    fn leaf(name: &'static str, id: usize) -> Node<'static> {
        Node {
            name,
            id,
            kind: NodeKind::Token,
            origin: None,
            children: &[],
        }
    }

    #[test]
    fn to_json_emits_nodes_and_edges() {
        let items = [leaf("a", 1)];
        let list = derived("A*", 2, Origin::List, &items);
        let kids = [list, leaf("b", 3)];
        let root = nonterminal("S", 0, &kids);
        let json = to_json(root, Some("Layout"));
        assert!(json.contains("\"label\":\"S\""));
        assert!(json.contains("\"label\":\"a\""));
        assert!(json.contains("\"kind\":\"Token\""));
        assert!(json.contains("\"layout_name\":\"Layout\""));
        // A user nonterminal and a token report no origin; a derived repetition
        // carries its own, so the frontend can simplify per node.
        assert!(json.contains("\"origin\":null"));
        assert!(json.contains("\"origin\":\"List\""));
    }

    #[test]
    fn to_sexpr_formats_nested_tree() {
        // S -> [A -> [x, y], z]: S is multi-line, A folds onto one line.
        let a_kids = [leaf("x", 1), leaf("y", 2)];
        let a = nonterminal("A", 3, &a_kids);
        let top = [a, leaf("z", 4)];
        let s = nonterminal("S", 0, &top);
        assert_eq!(to_sexpr(s, None), "(S\n  (A x y)\n  z)\n");
    }

    #[test]
    fn to_sexpr_hides_layout_subtree() {
        // S is multi-line, with a hidden Layout child between two visible ones;
        // hiding it must not leave a blank line where the node would have been.
        let x = [leaf("x", 1)];
        let a = nonterminal("A", 2, &x);
        let ws = [leaf("ws", 5)];
        let layout = nonterminal("Layout", 6, &ws);
        let kids = [a, layout, leaf("b", 3)];
        let s = nonterminal("S", 0, &kids);

        let hidden = DisplayOptions {
            show_layout: false,
            ..Default::default()
        };
        assert_eq!(
            to_sexpr_with(s, Some("Layout"), hidden),
            "(S\n  (A x)\n  b)\n"
        );
        assert_eq!(
            to_sexpr_with(s, Some("Layout"), DisplayOptions::default()),
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

        let arena = Arena::new();
        let mut node = leaf("leaf", 0);
        for i in 1..200_000 {
            let children: &[Node] = arena.alloc_slice([node]);
            node = nonterminal("N", i, children);
        }

        let mut counter = Counter(0);
        walk(node, &mut counter);
        assert_eq!(counter.0, 200_000);
    }

    #[test]
    fn show_empty_drops_empty_repetitions() {
        // S -> [A? (present), B? (empty), C* (empty), d].
        let a = [leaf("a", 1)];
        let present = derived("A?", 2, Origin::Opt, &a);
        let empty_opt = derived("B?", 3, Origin::Opt, &[]);
        let empty_list = derived("C*", 4, Origin::List, &[]);
        let kids = [present, empty_opt, empty_list, leaf("d", 5)];
        let s = nonterminal("S", 0, &kids);

        // Faithful: the empty optional and list show as leaves.
        assert_eq!(to_sexpr(s, None), "(S\n  (A? a)\n  B?\n  C*\n  d)\n");

        // show_empty off drops them; the present optional stays.
        let opts = DisplayOptions {
            show_empty: false,
            ..Default::default()
        };
        assert_eq!(to_sexpr_with(s, None, opts), "(S\n  (A? a)\n  d)\n");
    }

    #[test]
    fn show_empty_keeps_empty_optionals_under_spliced_wrappers() {
        // S -> [A? (present), B? (empty), C* (empty), d].
        let a = [leaf("a", 1)];
        let present = derived("A?", 2, Origin::Opt, &a);
        let empty_opt = derived("B?", 3, Origin::Opt, &[]);
        let empty_list = derived("C*", 4, Origin::List, &[]);
        let kids = [present, empty_opt, empty_list, leaf("d", 5)];
        let s = nonterminal("S", 0, &kids);

        // An empty optional is a wrapper too, so splicing wrappers would drop it.
        // show_empty keeps it, alongside the empty list.
        let opts = DisplayOptions {
            show_wrappers: false,
            ..Default::default()
        };
        assert_eq!(to_sexpr_with(s, None, opts), "(S a B? C* d)\n");

        // With both off, the present optional still splices and the empty ones go.
        let opts = DisplayOptions {
            show_empty: false,
            show_wrappers: false,
            ..Default::default()
        };
        assert_eq!(to_sexpr_with(s, None, opts), "(S a d)\n");
    }

    #[test]
    fn show_wrappers_splices_optionals_and_groups() {
        // S -> [A? -> a, (B C) -> b c].
        let a = [leaf("a", 1)];
        let opt = derived("A?", 2, Origin::Opt, &a);
        let group_kids = [leaf("b", 3), leaf("c", 4)];
        let group = derived("(B C)", 5, Origin::Group, &group_kids);
        let kids = [opt, group];
        let s = nonterminal("S", 0, &kids);

        assert_eq!(to_sexpr(s, None), "(S\n  (A? a)\n  ((B C) b c))\n");

        let opts = DisplayOptions {
            show_wrappers: false,
            ..Default::default()
        };
        assert_eq!(to_sexpr_with(s, None, opts), "(S a b c)\n");
    }

    #[test]
    fn simplified_unwraps_start_and_combines() {
        // StartGrammar -> [Layout, Grammar -> [A? -> a, B* -> b1 b2, C? empty], Layout].
        let a = [leaf("a", 1)];
        let opt = derived("A?", 2, Origin::Opt, &a);
        let bitems = [leaf("b1", 3), leaf("b2", 4)];
        let list = derived("B*", 5, Origin::List, &bitems);
        let empty_opt = derived("C?", 6, Origin::Opt, &[]);
        let grammar_kids = [opt, list, empty_opt];
        let grammar = nonterminal("Grammar", 10, &grammar_kids);

        let ws_before = [leaf("ws", 61)];
        let ws_after = [leaf("ws", 71)];
        let before = nonterminal("Layout", 7, &ws_before);
        let after = nonterminal("Layout", 8, &ws_after);
        // The start wrapper displays as `Start`, like the real one; the inner
        // child names the actual nonterminal.
        let start_kids = [before, grammar, after];
        let start = derived("Start", 0, Origin::Start, &start_kids);

        // Faithful: the start wrapper and its layout doubling are all there.
        assert_eq!(
            to_sexpr(start, Some("Layout")),
            "(Start\n  (Layout ws)\n  (Grammar\n    (A? a)\n    (B* b1 b2)\n    C?)\n  (Layout ws))\n"
        );

        // Simplified: start wrapper unwrapped, layout/empties/wrappers gone.
        assert_eq!(
            to_sexpr_with(start, Some("Layout"), DisplayOptions::simplified()),
            "(Grammar\n  a\n  (B* b1 b2))\n"
        );
    }

    #[test]
    fn simplified_keeps_ambiguity_cluster_intact() {
        // An ambiguous `A+`: an Amb cluster over two list derivations. The
        // cluster reports no origin, so simplified mode leaves it intact as
        // `(Amb …)` rather than splicing or dropping it; each alternative shows
        // as its own `(A+ …)` node.
        let items1 = [leaf("a", 1), leaf("a", 2)];
        let items2 = [leaf("a", 3), leaf("a", 4)];
        let alt1 = derived("A+", 5, Origin::List, &items1);
        let alt2 = derived("A+", 6, Origin::List, &items2);
        let alts = [alt1, alt2];
        let cluster = amb(7, &alts);
        let kids = [cluster];
        let s = nonterminal("S", 0, &kids);

        assert_eq!(
            to_sexpr_with(s, None, DisplayOptions::simplified()),
            "(S\n  (Amb\n    (A+ a a)\n    (A+ a a)))\n"
        );
    }
}
