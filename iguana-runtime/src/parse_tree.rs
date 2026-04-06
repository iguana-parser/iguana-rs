use std::fmt;

use crate::{
    parser::Parser,
    sppf::{NonterminalNode, SPPFNode, TerminalNode},
};

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
    node: &SPPFNode,
    parser: &P,
    builder: &impl ParseTreeBuilder<T>,
) -> OneOrMany<T> {
    match node {
        SPPFNode::Terminal(t) => {
            if t.terminal_id == P::epsilon() {
                OneOrMany::Zero
            } else {
                OneOrMany::One(builder.new_token(t))
            }
        }
        SPPFNode::Nonterminal(n) => {
            if n.ambiguous {
                println!("Ambiguous nonterminal: id={:?}, span={:?}", n.nonterminal_id, n.span);
                unimplemented!()
            }
            let child = parser.sppf_node(n.child);
            let children = visit_sppf(child, parser, builder);
            OneOrMany::One(builder.new_nonterminal_node(n, children))
        }
        SPPFNode::Intermediate(i) => {
            if i.ambiguous {
                println!("Ambiguous intermediate node: slot_id={:?}, span={:?}", i.slot_id, i.span);
                unimplemented!()
            }
            let (left_child, right_child) = i.child;
            let left_child = parser.sppf_node(left_child);
            let right_child = parser.sppf_node(right_child);
            visit_sppf(left_child, parser, builder).merge(visit_sppf(right_child, parser, builder))
        }
    }
}

pub trait ParseTreeBuilder<T: fmt::Debug> {
    fn new_token(&self, terminal_node: &TerminalNode) -> T;
    fn new_nonterminal_node(&self, nonterminal_node: &NonterminalNode, children: OneOrMany<T>)
    -> T;
}
