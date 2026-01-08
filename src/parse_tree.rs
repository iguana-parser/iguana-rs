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
            (OneOrMany::Zero, other) => other,
            _ => unreachable!(),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(item) => vec![item],
            OneOrMany::Many(items) => items,
            OneOrMany::Zero => vec![],
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

pub fn visit_sppf<'i, T: fmt::Debug>(
    node: &SPPFNode,
    parser: &impl Parser<'i>,
    builder: &impl ParseTreeBuilder<T>,
) -> OneOrMany<T> {
    match node {
        SPPFNode::Terminal(t) => {
            if t.span.left_extent == t.span.right_extent {
                OneOrMany::Zero
            } else {
                OneOrMany::One(builder.new_token(t))
            }
        }
        SPPFNode::Nonterminal(n) => {
            if n.ambiguous {
                unimplemented!()
            }
            let child = parser.sppf_node(n.child);
            let children = visit_sppf(child, parser, builder);
            let nt = OneOrMany::One(builder.new_nonterminal_node(n, children));
            println!("nt: {:?}", nt);
            nt
        }
        SPPFNode::Intermediate(i) => {
            if i.ambiguous {
                unimplemented!()
            }
            let (left_child, right_child) = i.child;
            let left_child = parser.sppf_node(left_child);
            let right_child = parser.sppf_node(right_child);
            println!("{:?},{:?}", left_child, right_child);
            let res = visit_sppf(left_child, parser, builder).merge(visit_sppf(
                right_child,
                parser,
                builder,
            ));
            println!("res: {:?}", res);
            res
        }
    }
}

pub trait ParseTreeBuilder<T: fmt::Debug> {
    fn new_token(&self, terminal_node: &TerminalNode) -> T;
    fn new_nonterminal_node(&self, nonterminal_node: &NonterminalNode, children: OneOrMany<T>)
    -> T;
}
