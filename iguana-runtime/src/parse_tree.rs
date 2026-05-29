use std::fmt;

pub use bumpalo::Bump;

use crate::{
    ids::{NonterminalId, SlotId},
    parser::Parser,
    sppf::{NonterminalNode, SPPFNode, SPPFNodeId, TerminalNode},
};

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

#[derive(Debug)]
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

pub fn visit_sppf<'i, T: fmt::Debug + Clone, P: Parser<'i>>(
    node_id: SPPFNodeId,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
) -> OneOrMany<T> {
    let node = parser.sppf_node(node_id);
    match node {
        SPPFNode::Terminal(t) => {
            if t.terminal_id == P::epsilon() {
                OneOrMany::Zero
            } else {
                OneOrMany::One(builder.new_token(t))
            }
        }
        SPPFNode::Nonterminal(n) => {
            let children = visit_sppf(n.child, parser, builder);
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
                        let extra_children = visit_sppf(child_id, parser, builder);
                        alternatives.extend(create_nonterminal_nodes(
                            extra_children,
                            &synthetic,
                            builder,
                        ));
                    }
                }
                return OneOrMany::One(builder.new_ambiguity_node(n.nonterminal_id, alternatives));
            }
            OneOrMany::One(builder.new_nonterminal_node(n, children))
        }
        SPPFNode::Intermediate(i) => {
            if !i.ambiguous {
                let (left, right) = i.child;
                return visit_sppf(left, parser, builder).merge(visit_sppf(right, parser, builder));
            }
            let mut pairs = vec![i.child];
            if let Some(extras) = parser.intermediate_nodes_children_map().get(&node_id) {
                pairs.extend(extras.iter().copied());
            }
            let mut derivations: Vec<OneOrMany<T>> = Vec::with_capacity(pairs.len());
            for (left, right) in pairs {
                let merged =
                    visit_sppf(left, parser, builder).merge(visit_sppf(right, parser, builder));
                // A deeper ambiguity may have already produced a `Multi`; flatten
                // so the outer `Multi` stays one level deep.
                match merged {
                    OneOrMany::Multi(inner) => derivations.extend(inner),
                    flat => derivations.push(flat),
                }
            }
            OneOrMany::Multi(derivations)
        }
    }
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
