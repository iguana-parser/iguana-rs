use std::{
    fmt::{self, Debug, Write},
    sync::OnceLock,
};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    grammar::Grammar,
    ids::{NonterminalId, SPPFNodeId},
    input::Span,
    parser::Parser,
    sppf::{NonterminalNode, SPPFNode, TerminalNode},
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

/// The children of a node under construction in the general walker.
#[derive(Debug, Clone)]
enum OneOrMany<T: Debug> {
    Zero,
    One(T),
    Many(Vec<T>),
    /// Multiple derivations under an ambiguous intermediate node, one child sequence
    /// per derivation.
    Multi(Vec<OneOrMany<T>>),
}

impl<T: Debug + Clone> OneOrMany<T> {
    fn merge(self, other: OneOrMany<T>) -> OneOrMany<T> {
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

    /// The children of one derivation, in order.
    fn as_slice(&self) -> &[T] {
        match self {
            OneOrMany::Zero => &[],
            OneOrMany::One(item) => std::slice::from_ref(item),
            OneOrMany::Many(items) => items,
            OneOrMany::Multi(_) => unreachable!("a derivation is a flat child sequence"),
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
    // The first-parse mode records no ambiguity, so the answer is known at
    // compile time and the walk below is dead code.
    if P::FIRST_PARSE {
        return false;
    }
    if parser.nonterminal_nodes_children_map().is_empty()
        && parser.intermediate_nodes_children_map().is_empty()
    {
        return false;
    }
    // Before the walk reaches an ambiguous node, only empty nodes can be
    // shared, so only they are recorded as visited. Every SPPF cycle contains
    // an ambiguous node, where the walk stops.
    let mut visited_empty = FxHashSet::default();
    let mut stack = vec![root_id];
    while let Some(id) = stack.pop() {
        let node = parser.sppf_node(id);
        if node.left_extent() == node.right_extent() && !visited_empty.insert(id) {
            continue;
        }
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
) -> T {
    // If the SPPF is not ambiguous, that is, no ambiguous node is reachable
    // from the root, every node has one derivation, and we use the
    // unambiguous walker, which does not use a memo. The first-parse mode
    // never creates an ambiguous node, so it skips the check.
    // Every SPPF cycle includes an ambiguous node: closing a cycle adds an
    // edge to an existing node. So a cyclic SPPF always takes the general
    // walker, which relies on its memo to terminate.
    if P::FIRST_PARSE || !is_ambiguous(parser, node_id) {
        return visit_sppf_unambiguous(node_id, parser, builder);
    }
    match visit_sppf_impl(node_id, parser, builder) {
        OneOrMany::One(tree) => tree,
        _ => unreachable!("the root is a single nonterminal value"),
    }
}

/// A frame in the unambiguous walker: `Pre` visits a node, `Post` builds a
/// nonterminal node from its children. A nonterminal node's visit pushes the
/// node's `Post` and continues with the child, so the `Post` runs once the
/// whole subtree is done. An intermediate node's visit pushes the right
/// child's `Pre` and continues with the left child. Tokens are built at their
/// visit and need no `Post`.
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
/// contributes no value of its own (it only pushes a `Pre` frame for its right
/// child), and a nonterminal node records the value-stack depth on entry
/// (the `Post` frame's `mark`), so on completion the values above the mark
/// are exactly its children, handed to the builder as a slice.
fn visit_sppf_unambiguous<'i, 'arena, T: Debug + Clone, P: Parser<'i, 'arena>>(
    root_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
) -> T {
    let mut work: Vec<UnambiguousFrame> = Vec::new();
    let mut values: Vec<T> = Vec::new();
    // The next node to visit
    let mut next = Some(root_id);
    loop {
        while let Some(node_id) = next {
            next = match parser.sppf_node(node_id) {
                SPPFNode::Terminal(t) => {
                    if t.terminal_id != P::Grammar::epsilon() {
                        values.push(builder.new_token(t));
                    }
                    None
                }
                SPPFNode::Nonterminal(n) => {
                    work.push(UnambiguousFrame::Post {
                        node: n,
                        mark: values.len(),
                    });
                    Some(n.child)
                }
                SPPFNode::Intermediate(i) => {
                    // The right child waits on `work` while the left child is
                    // visited, so the values land in source order.
                    work.push(UnambiguousFrame::Pre(i.child.1));
                    Some(i.child.0)
                }
            };
        }
        match work.pop() {
            None => break,
            Some(UnambiguousFrame::Pre(node_id)) => next = Some(node_id),
            Some(UnambiguousFrame::Post { node, mark }) => {
                let result = builder.new_nonterminal_node(node, &values[mark..]);
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

/// A nonterminal under construction or a completed SPPF node's result.
enum MemoEntry<T: Debug> {
    /// The nonterminal's frame is still on the stack. The Cycle node is
    /// allocated on the first back-edge and shared by later references.
    Active(Option<T>),
    /// The finished result, available for reuse.
    Done(OneOrMany<T>),
}

impl<'arena, T: Debug> Frame<'arena, T> {
    fn enter<'i>(
        node_id: SPPFNodeId,
        parser: &impl Parser<'i, 'arena>,
        memo: &mut FxHashMap<SPPFNodeId, MemoEntry<T>>,
    ) -> Self {
        // Only nonterminals are marked active. An intermediate node produces
        // a child sequence rather than a typed value, so a re-entered
        // intermediate node is expanded again until the walk reaches an
        // active nonterminal.
        if matches!(parser.sppf_node(node_id), SPPFNode::Nonterminal(_)) {
            memo.insert(node_id, MemoEntry::Active(None));
        }
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
) -> OneOrMany<T> {
    // A child frame is fully built before its parent visits the next child,
    // so results arrive in source order. Completed nodes are reused;
    // re-entering an active nonterminal produces a Cycle reference.
    let arena = parser.vec_arena();
    let mut memo = FxHashMap::default();
    let mut stack = vec![Frame::enter(root_id, parser, &mut memo)];
    loop {
        let top = stack.len() - 1;
        match stack[top].children.get(stack[top].next).copied() {
            Some(child) => {
                stack[top].next += 1;
                match memo.get_mut(&child) {
                    Some(MemoEntry::Done(cached)) => stack[top].results.push(cached.clone(), arena),
                    Some(MemoEntry::Active(cycle)) => {
                        let cycle = cycle.get_or_insert_with(|| {
                            let SPPFNode::Nonterminal(node) = parser.sppf_node(child) else {
                                unreachable!("only nonterminals are marked active")
                            };
                            builder.new_cycle_node(node)
                        });
                        stack[top]
                            .results
                            .push(OneOrMany::One(cycle.clone()), arena);
                    }
                    None => stack.push(Frame::enter(child, parser, &mut memo)),
                }
            }
            None => {
                let frame = stack.pop().unwrap();
                let result = build_node(parser, builder, frame.node_id, frame.results);
                if let Some(MemoEntry::Active(Some(cycle))) =
                    memo.insert(frame.node_id, MemoEntry::Done(result.clone()))
                {
                    let OneOrMany::One(target) = &result else {
                        unreachable!("a cycle target must be a single nonterminal value")
                    };
                    builder.set_cycle_target(&cycle, target);
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
            if t.terminal_id == P::Grammar::epsilon() {
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
                OneOrMany::One(builder.new_nonterminal_node(n, children.as_slice()))
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
fn create_nonterminal_nodes<T: Debug + Clone>(
    children: OneOrMany<T>,
    node: &NonterminalNode,
    builder: &impl ParseTreeBuilder<T>,
) -> Vec<T> {
    match children {
        OneOrMany::Multi(derivations) => derivations
            .iter()
            .map(|d| builder.new_nonterminal_node(node, d.as_slice()))
            .collect(),
        single => vec![builder.new_nonterminal_node(node, single.as_slice())],
    }
}

pub trait ParseTreeBuilder<T: Debug> {
    fn new_token(&self, terminal_node: &TerminalNode) -> T;
    /// Builds a nonterminal node from the children of one derivation, in order.
    fn new_nonterminal_node(&self, nonterminal_node: &NonterminalNode, children: &[T]) -> T;
    fn new_ambiguity_node(&self, parent: NonterminalId, alternatives: Vec<T>) -> T {
        let _ = (parent, alternatives);
        unimplemented!("ambiguity handling not yet implemented for this builder")
    }

    /// Creates an uninitialized cycle reference. Clones of the returned value
    /// must share the target later initialized by `set_cycle_target`.
    fn new_cycle_node(&self, node: &NonterminalNode) -> T {
        let _ = node;
        unimplemented!("cycle handling requires regenerating this parser")
    }

    /// Initializes the cycle reference and all its clones with the completed
    /// nonterminal result, including its `Amb` node when present.
    fn set_cycle_target(&self, cycle: &T, target: &T) {
        let _ = (cycle, target);
        unimplemented!("cycle handling requires regenerating this parser")
    }
}

/// A typed cycle reference initialized when its target finishes construction.
/// Debug prints the target's address without following the reference.
pub struct CycleTarget<'a, T>(OnceLock<&'a T>);

impl<'a, T> CycleTarget<'a, T> {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    /// The target, which is initialized before the parser returns the tree.
    /// Panics if construction has not initialized it yet.
    pub fn get(&self) -> &'a T {
        self.0
            .get()
            .copied()
            .expect("cycle target has not been initialized")
    }

    /// Initializes the reference. Panics if it was already initialized.
    pub fn set(&self, target: &'a T) {
        assert!(self.0.set(target).is_ok(), "cycle target initialized twice");
    }
}

impl<T> Default for CycleTarget<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for CycleTarget<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CycleTarget")
            .field(&self.0.get().map(|node| *node as *const T))
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Token,
    Nonterminal,
    Amb,
    Cycle,
}

/// The grammar construct a nonterminal node was derived from. Drives the
/// presentation transforms (hiding empty repetitions, splicing wrappers) and
/// the logical children of lists in `ParseTreeNode::children`.
/// `None` for a user-declared nonterminal or a token.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The synthetic start wrapper, the start symbol between leading and
    /// trailing layout.
    Start,
    /// An optional, `X?`.
    Opt,
    /// A possibly empty repetition, `X*` or `{X sep}*`, defined as
    /// `X* = X+ | ()` and `{X sep}* = {X sep}+ | ()`.
    Star,
    /// A non-empty repetition, `X+` or `{X sep}+`, defined as
    /// `X+ = X+ X | X` and `{X sep}+ = {X sep}+ sep X | X`.
    Plus,
    /// An anonymous sequence group, `(A B C)`.
    Group,
    /// An anonymous alternation, `(A | B)`.
    Alt,
}

/// The uniform interface for generic parse-tree traversals, such as conversion
/// to an s-expression or JSON. Generated parsers implement it for their
/// `ParseTree` type.
pub trait ParseTreeNode: Copy {
    /// The child at `index`. Returns `None` when the index is past the last
    /// child. The child is the node's actual child in the parse tree. To
    /// iterate over the children of a list, use `children()`. It skips the
    /// `Plus` nodes of the list's left recursion and returns only the logical
    /// children (see `list_children`). The children of an Amb node are its
    /// alternatives, and a Cycle node doesn't have any children.
    #[doc(hidden)]
    fn child(&self, index: usize) -> Option<Self>;
    /// The number of children `child` returns.
    #[doc(hidden)]
    fn child_count(&self) -> usize;

    /// The node's logical children. A list's children are the ones
    /// `list_children` returns.
    fn children(&self) -> Vec<Self> {
        match self.origin() {
            Some(Origin::Star | Origin::Plus) if self.kind() == NodeKind::Nonterminal => {
                list_children(*self)
            }
            _ => (0..self.child_count())
                .filter_map(|i| self.child(i))
                .collect(),
        }
    }

    fn display_name(&self) -> &'static str;
    fn span(&self) -> Span;
    fn kind(&self) -> NodeKind;
    /// A stable identity for the node, or `None` for nodes that are never
    /// shared. Used to detect sharing in an ambiguity DAG.
    fn node_id(&self) -> Option<usize>;
    /// The grammar construct this node was derived from, or `None` for a
    /// user-declared nonterminal or a token.
    fn origin(&self) -> Option<Origin> {
        None
    }

    /// The target of a Cycle node. It is not part of `children()`.
    fn cycle_target(&self) -> Option<Self> {
        None
    }

    /// Whether any reachable node is ambiguous, including cycle targets.
    fn contains_ambiguity(&self) -> bool {
        let mut visited = FxHashSet::default();
        let mut stack = vec![*self];
        while let Some(node) = stack.pop() {
            if node.kind() == NodeKind::Amb {
                return true;
            }
            if let Some(id) = node.node_id()
                && !visited.insert(id)
            {
                continue;
            }
            stack.extend(node.cycle_target());
            stack.extend(node.children());
        }
        false
    }

    /// Whether any node reachable through `children()` is a Cycle node.
    fn contains_cycle(&self) -> bool {
        let mut visited = FxHashSet::default();
        let mut stack = vec![*self];
        while let Some(node) = stack.pop() {
            if node.kind() == NodeKind::Cycle {
                return true;
            }
            if let Some(id) = node.node_id()
                && !visited.insert(id)
            {
                continue;
            }
            stack.extend(node.children());
        }
        false
    }
}

/// The logical children of a `Star` or `Plus` node: its elements, with the
/// separators and layout between them, without the `Plus` nodes of the list's
/// left recursion. If the list or one of those `Plus` nodes is an Amb or Cycle
/// node, that node is returned as one child.
pub fn list_children<N: ParseTreeNode>(list: N) -> Vec<N> {
    let mut plus = list;
    if list.origin() == Some(Origin::Star) && list.kind() == NodeKind::Nonterminal {
        match list.child(0) {
            Some(child) => plus = child,
            None => return Vec::new(),
        }
    }
    // The walk descends the left recursion from the last element, so it
    // collects the children in reverse.
    let mut children = Vec::new();
    loop {
        if plus.kind() != NodeKind::Nonterminal {
            children.push(plus);
            break;
        }
        let count = plus.child_count();
        for i in (1..count).rev() {
            children.extend(plus.child(i));
        }
        match plus.child(0) {
            Some(rest) if count > 1 => plus = rest,
            first => {
                children.extend(first);
                break;
            }
        }
    }
    children.reverse();
    children
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
    /// The children `walk` should descend into. Defaults to the node's logical
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
/// to the existing node instead of a re-expanded subtree. Cycle references are
/// edges to their targets, without separate nodes.
/// The root is always the first entry in `nodes`.
pub fn to_json<N: ParseTreeNode>(root: N, layout_name: Option<&str>) -> String {
    let mut builder = JsonBuilder::default();
    walk(resolve_cycle(root), &mut builder);
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
        Origin::Star | Origin::Plus => "List",
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
            NodeKind::Cycle => unreachable!("cycle references are resolved before rendering"),
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

    fn children(&self, node: N) -> Vec<N> {
        node.children().into_iter().map(resolve_cycle).collect()
    }
}

/// The node a renderer shows: a Cycle's target, or the node itself.
fn resolve_cycle<N: ParseTreeNode>(node: N) -> N {
    node.cycle_target().unwrap_or(node)
}

/// Renders a parse tree as an s-expression with default options.
pub fn to_sexpr<N: ParseTreeNode>(root: N, layout_name: Option<&str>) -> String {
    to_sexpr_with(root, layout_name, DisplayOptions::default())
}

/// Renders a parse tree as an s-expression. A first pass counts how many parents
/// reach each node; a node reached by more than one is written once with a `#N=`
/// label and referenced as `#N#` elsewhere, so a shared forest stays bounded.
/// A Cycle node is written as the `#N#` reference to its target.
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

/// An empty optional or list: an `X?`, `X*`, or `{X sep}*` that matched the
/// empty string. The renderers show these nodes only when `show_empty` is set.
fn is_empty_opt_or_list<N: ParseTreeNode>(node: N) -> bool {
    matches!(node.origin(), Some(Origin::Opt | Origin::Star)) && node.child_count() == 0
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
        let child = resolve_cycle(child);
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
    let mut node = resolve_cycle(root);
    while !options.show_wrappers && is_wrapper(node.origin()) {
        match display_children(node, layout_name, options).as_slice() {
            [only] => node = *only,
            _ => break,
        }
    }
    node
}

/// The 64-bit FNV offset basis, the value FNV-1a starts from.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// The 64-bit FNV prime, 2^40 + 2^8 + 0xb3.
const FNV_PRIME: u64 = 0x100_0000_01b3;

/// Returns a digest of the parse tree. The digest is used for comparison of
/// two parse trees, without generating the s-expression.
///
/// The walk over the tree for digest calculation is exactly the same as for
/// rendering s-expressions. We pass `DisplayOptions::default()` so that all
/// the toggles are on, to traverse the tree with all the nodes.
pub fn tree_digest<N: ParseTreeNode>(root: N) -> u64 {
    let root = display_root(root, None, DisplayOptions::default());
    let mut digest = TreeDigest {
        hash: FNV_OFFSET_BASIS,
        node_ids: FxHashMap::default(),
        next_id: 0,
    };
    walk(root, &mut digest);
    digest.hash
}

/// Folds a tree into a digest.
struct TreeDigest {
    /// The hash value: FNV-1a over the name and the span of every node in
    /// the tree. Because the constants in FNV-1a are fixed and the numbers
    /// are written as little-endian bytes, the same tree produces the same
    /// hash in every build. This makes it possible to compare parse trees
    /// across any machine.
    hash: u64,
    /// A map that associates a node's id (`node_id()`, a pointer value in a
    /// generated parser) to an incremented id to uniquely identify the node.
    /// This map is also used to avoid visiting a node for a second time.
    node_ids: FxHashMap<usize, u32>,
    /// The calculated id for the next node. This is used instead of the `node_id`
    /// so that the digests are repeatable between runs.
    next_id: u32,
}

impl TreeDigest {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= *byte as u64;
            self.hash = self.hash.wrapping_mul(FNV_PRIME);
        }
    }
}

impl<N: ParseTreeNode> TreeVisitor<N> for TreeDigest {
    fn enter(&mut self, node: N) -> Visit {
        if let Some(key) = node.node_id() {
            if let Some(id) = self.node_ids.get(&key).copied() {
                // A second parent of a shared node, or a cycle back to it.
                self.write(b"#");
                self.write(&id.to_le_bytes());
                return Visit::Skip;
            }
            self.node_ids.insert(key, self.next_id);
        }
        // Counted for every node, not only for the ones with an identity.
        // This allows creating distinguishable digests when there are
        // ambiguities or cycles, and nodes are shared in the tree.
        // Numbering only the nodes with an identity gives the same id to the
        // first sharable node of two trees that share different nodes, which
        // then cannot be told apart. See the test case
        // `digest_separates_trees_that_share_different_nodes`.
        self.next_id += 1;
        // The write format is as follows:
        // ( name_length name_bytes left_extent right_extent children )
        // The length allows the name to hold any characters, including '('
        // and ')'. The parens maintain the shape of the tree. Without them,
        // we cannot distinguish between a child and a sibling node, and as a
        // result, two different trees can give the same digest.
        self.write(b"(");
        let name = node.display_name().as_bytes();
        self.write(&(name.len() as u32).to_le_bytes());
        self.write(name);
        let span = node.span();
        self.write(&span.left_extent.to_le_bytes());
        self.write(&span.right_extent.to_le_bytes());
        Visit::Children
    }

    // Without a marker for the end of a node, a child and a sibling would fold
    // in the same bytes: (A (B (C))) would match (A (B) (C)).
    fn exit(&mut self, _node: N) {
        self.write(b")");
    }

    fn children(&self, node: N) -> Vec<N> {
        display_children(node, None, DisplayOptions::default())
    }
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
        fn child(&self, index: usize) -> Option<Self> {
            self.children.get(index).copied()
        }
        fn child_count(&self) -> usize {
            self.children.len()
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
            // usize::MAX stands for a node that is never shared, as a token
            // is in a generated parser.
            (self.id != usize::MAX).then_some(self.id)
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

    /// A node with no identity, which a renderer never labels and always
    /// re-expands, as it does a token.
    fn anonymous<'a>(name: &'static str, children: &'a [Node<'a>]) -> Node<'a> {
        Node {
            name,
            id: usize::MAX,
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
        let plus = [derived("A+", 4, Origin::Plus, &items)];
        let list = derived("A*", 2, Origin::Star, &plus);
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

    /// The digest exists to stand in for the rendering, so it has to agree
    /// with it: same s-expression, same digest, and a different one whenever
    /// the rendering differs.
    #[test]
    fn digest_agrees_with_the_rendering() {
        let leaf_b = [nonterminal("B", 2, &[])];
        let leaf_c = [nonterminal("C", 3, &[])];
        let nested = [nonterminal("B", 2, &leaf_c)];
        let siblings = [nonterminal("B", 2, &[]), nonterminal("C", 3, &[])];

        let one = nonterminal("A", 1, &leaf_b);
        let same = nonterminal("A", 1, &leaf_b);
        let deeper = nonterminal("A", 1, &nested);
        let flat = nonterminal("A", 1, &siblings);

        let options = DisplayOptions::default();
        let digest = |node| tree_digest(node);
        let render = |node| to_sexpr_with(node, None, options);

        assert_eq!(render(one), render(same));
        assert_eq!(digest(one), digest(same));

        // A child and a sibling are different trees, and the digest has to
        // separate them even though the same names appear in both.
        assert_ne!(render(deeper), render(flat));
        assert_ne!(digest(deeper), digest(flat));
    }

    /// Two trees that share different nodes must not fold into the same
    /// bytes. Both of these expand the same two subtrees in the same order
    /// and then write one back-reference, so the reference has to say which
    /// node it points at.
    #[test]
    fn digest_separates_trees_that_share_different_nodes() {
        let shared = nonterminal("A", 1, &[]);
        let plain = nonterminal("B", 2, &[]);

        // A is shared; B is a node that is never labelled.
        let shares_first = [shared, anonymous("B", &[]), shared];
        // B is shared; A is the one with no identity.
        let shares_second = [anonymous("A", &[]), plain, plain];

        assert_ne!(
            tree_digest(nonterminal("R", 7, &shares_first)),
            tree_digest(nonterminal("R", 8, &shares_second))
        );
    }

    /// A node reached twice is expanded once and referenced after that, in
    /// the digest as in the rendering, so a shared forest stays bounded.
    #[test]
    fn digest_handles_a_shared_node() {
        let shared = [nonterminal("S", 9, &[])];
        let twice = [shared[0], shared[0]];
        let once = [shared[0]];

        assert_ne!(
            tree_digest(nonterminal("A", 1, &twice)),
            tree_digest(nonterminal("A", 1, &once))
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
        let empty_list = derived("C*", 4, Origin::Star, &[]);
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
        let empty_list = derived("C*", 4, Origin::Star, &[]);
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
        // B* -> B+ -> [B+ -> b1, b2], the left-recursive shape of a list.
        let b1 = [leaf("b1", 3)];
        let first = derived("B+", 11, Origin::Plus, &b1);
        let bitems = [first, leaf("b2", 4)];
        let plus = [derived("B+", 12, Origin::Plus, &bitems)];
        let list = derived("B*", 5, Origin::Star, &plus);
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
        let a1 = [leaf("a", 1)];
        let a3 = [leaf("a", 3)];
        let items1 = [derived("A+", 8, Origin::Plus, &a1), leaf("a", 2)];
        let items2 = [derived("A+", 9, Origin::Plus, &a3), leaf("a", 4)];
        let alt1 = derived("A+", 5, Origin::Plus, &items1);
        let alt2 = derived("A+", 6, Origin::Plus, &items2);
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
