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
}

impl<T: fmt::Debug> OneOrMany<T> {
    pub fn merge(self, other: OneOrMany<T>) -> OneOrMany<T> {
        match (self, other) {
            (OneOrMany::One(l), OneOrMany::One(r)) => OneOrMany::Many(vec![l, r]),
            (OneOrMany::Many(mut l), OneOrMany::One(r)) => {
                l.push(r);
                OneOrMany::Many(l)
            }
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
        }
    }

    pub fn unwrap_one(self) -> T {
        match self {
            OneOrMany::One(item) => item,
            OneOrMany::Many(_) => panic!(),
            OneOrMany::Zero => panic!(),
        }
    }
}

pub fn visit_sppf<'i, T: fmt::Debug, P: Parser<'i>>(
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
            if !n.ambiguous {
                let children = visit_sppf(n.child, parser, builder);
                return OneOrMany::One(builder.new_nonterminal_node(n, children));
            }
            // Ambiguous nonterminal: each child is a complete alternative
            // derivation. Build one parse-tree node per child using its own
            // return slot and wrap the results in an Amb.
            let extras: Vec<(SPPFNodeId, SlotId)> = parser
                .nonterminal_nodes_children_map()
                .get(&node_id)
                .cloned()
                .unwrap_or_default();
            let nonterminal_id = n.nonterminal_id;
            let span = n.span;
            let mut alternatives: Vec<T> = Vec::with_capacity(1 + extras.len());
            let primary_children = visit_sppf(n.child, parser, builder);
            alternatives.push(builder.new_nonterminal_node(n, primary_children));
            for (child_id, return_slot) in extras {
                let synthetic = NonterminalNode {
                    nonterminal_id,
                    return_slot,
                    span,
                    child: child_id,
                    ambiguous: false,
                };
                let children = visit_sppf(child_id, parser, builder);
                alternatives.push(builder.new_nonterminal_node(&synthetic, children));
            }
            OneOrMany::One(builder.new_ambiguity_node(nonterminal_id, alternatives))
        }
        SPPFNode::Intermediate(i) => {
            if i.ambiguous {
                todo!(
                    "ambiguous intermediate extraction: slot_id={:?}, span={:?}",
                    i.slot_id,
                    i.span
                )
            }
            let (left_child, right_child) = i.child;
            visit_sppf(left_child, parser, builder).merge(visit_sppf(right_child, parser, builder))
        }
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
