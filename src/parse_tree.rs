use crate::{
    parser::Parser,
    sppf::{NonterminalNode, SPPFNode, TerminalNode},
};

pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn merge(self, other: OneOrMany<T>) -> OneOrMany<T> {
        match (self, other) {
            (OneOrMany::One(l), OneOrMany::One(r)) => OneOrMany::Many(vec![l, r]),
            (OneOrMany::Many(mut l), OneOrMany::One(r)) => {
                l.push(r);
                OneOrMany::Many(l)
            }
            _ => unreachable!(),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(item) => vec![item],
            OneOrMany::Many(items) => items,
        }
    }

    pub fn unwrap_one(self) -> T {
        match self {
            OneOrMany::One(item) => item,
            OneOrMany::Many(_) => panic!(),
        }
    }
}

pub fn visit_sppf<'i, T>(
    node: &SPPFNode,
    parser: &impl Parser<'i>,
    builder: &impl ParseTreeBuilder<T>,
) -> OneOrMany<T> {
    match node {
        SPPFNode::Terminal(t) => OneOrMany::One(builder.new_token(t)),
        SPPFNode::Nonterminal(n) => {
            if n.ambiguous {
                unimplemented!()
            }
            let child = parser.sppf_node(n.child);
            let children = visit_sppf(child, parser, builder);
            OneOrMany::One(builder.new_nonterminal_node(n, children))
        }
        SPPFNode::Intermediate(i) => {
            if i.ambiguous {
                unimplemented!()
            }
            let (left_child, right_child) = i.child;
            let left_child = parser.sppf_node(left_child);
            let right_child = parser.sppf_node(right_child);
            visit_sppf(left_child, parser, builder).merge(visit_sppf(right_child, parser, builder))
        }
    }
}

pub trait ParseTreeBuilder<T> {
    fn new_token(&self, terminal_node: &TerminalNode) -> T;
    fn new_nonterminal_node(&self, nonterminal_node: &NonterminalNode, children: OneOrMany<T>)
    -> T;
}
