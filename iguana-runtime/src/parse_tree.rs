use std::fmt;

pub use bumpalo::Bump;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    ids::{NonterminalId, SlotId},
    parser::Parser,
    sppf::{NonterminalNode, SPPFNode, SPPFNodeId, TerminalNode},
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
pub enum OneOrMany<T: fmt::Debug> {
    Zero,
    One(T),
    Many(Vec<T>),
    /// Multiple derivations under an ambiguous intermediate node, one child sequence
    /// per derivation.
    Multi(Vec<OneOrMany<T>>),
}

impl<T: fmt::Debug + Clone> OneOrMany<T> {
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

pub fn visit_sppf<'i, T: fmt::Debug + Clone, P: Parser<'i>>(
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

fn visit_sppf_impl<'i, T: fmt::Debug + Clone, P: Parser<'i>>(
    node_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
    memo: &mut Option<FxHashMap<SPPFNodeId, OneOrMany<T>>>,
) -> OneOrMany<T> {
    if let Some(m) = memo.as_ref() {
        if let Some(cached) = m.get(&node_id) {
            return cached.clone();
        }
    }
    let node = parser.sppf_node(node_id);
    let result = match node {
        SPPFNode::Terminal(t) => {
            if t.terminal_id == P::epsilon() {
                OneOrMany::Zero
            } else {
                OneOrMany::One(builder.new_token(t))
            }
        }
        SPPFNode::Nonterminal(n) => {
            let children = visit_sppf_impl(n.child, parser, builder, memo);
            // `Multi` here means derivations from an ambiguous intermediate
            // node below have bubbled up and need wrapping in `Amb` just
            // like nonterminal-level ambiguity.
            if n.ambiguous || matches!(children, OneOrMany::Multi(_)) {
                let mut alternatives = create_nonterminal_nodes(children, n, builder);
                if n.ambiguous {
                    let extras: Vec<(SPPFNodeId, SlotId)> = parser
                        .nonterminal_nodes_children_map()
                        .get(&node_id)
                        .cloned()
                        .unwrap_or_default();
                    for (child_id, return_slot) in extras {
                        let synthetic = NonterminalNode {
                            nonterminal_id: n.nonterminal_id,
                            return_slot,
                            span: n.span,
                            child: child_id,
                            ambiguous: false,
                        };
                        let extra_children = visit_sppf_impl(child_id, parser, builder, memo);
                        alternatives.extend(create_nonterminal_nodes(
                            extra_children,
                            &synthetic,
                            builder,
                        ));
                    }
                }
                OneOrMany::One(builder.new_ambiguity_node(n.nonterminal_id, alternatives))
            } else {
                OneOrMany::One(builder.new_nonterminal_node(n, children))
            }
        }
        SPPFNode::Intermediate(i) => {
            if !i.ambiguous {
                let (left, right) = i.child;
                visit_sppf_impl(left, parser, builder, memo)
                    .merge(visit_sppf_impl(right, parser, builder, memo))
            } else {
                let mut pairs = vec![i.child];
                if let Some(extras) = parser.intermediate_nodes_children_map().get(&node_id) {
                    pairs.extend(extras.iter().copied());
                }
                let mut derivations: Vec<OneOrMany<T>> = Vec::with_capacity(pairs.len());
                for (left, right) in pairs {
                    let merged = visit_sppf_impl(left, parser, builder, memo)
                        .merge(visit_sppf_impl(right, parser, builder, memo));
                    // A deeper ambiguity may have already produced a `Multi`;
                    // flatten so the outer `Multi` stays one level deep.
                    match merged {
                        OneOrMany::Multi(inner) => derivations.extend(inner),
                        flat => derivations.push(flat),
                    }
                }
                OneOrMany::Multi(derivations)
            }
        }
    };
    if let Some(m) = memo.as_mut() {
        m.insert(node_id, result.clone());
    }
    result
}

/// Builds one parse-tree node per derivation. When the children arrive
/// as a `Multi` from an ambiguous intermediate node, there are several
/// derivations of the same alternative and the resulting nodes go into
/// an `Amb`.
fn create_nonterminal_nodes<T: fmt::Debug>(
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

pub trait ParseTreeBuilder<T: fmt::Debug> {
    fn new_token(&self, terminal_node: &TerminalNode) -> T;
    fn new_nonterminal_node(&self, nonterminal_node: &NonterminalNode, children: OneOrMany<T>)
    -> T;
    fn new_ambiguity_node(&self, parent: NonterminalId, alternatives: Vec<T>) -> T {
        let _ = (parent, alternatives);
        unimplemented!("ambiguity handling not yet implemented for this builder")
    }
}
