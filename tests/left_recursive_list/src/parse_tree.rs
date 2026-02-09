use crate::parser::LeftRecursiveListParser;
use core::fmt;
use iguana_runtime::{
    ids::{NonterminalId, SlotId, TerminalId},
    parse_tree::{OneOrMany, ParseTreeBuilder, visit_sppf},
    parser::Parser,
    sppf::{NonterminalNode, SPPFNodeId, Span, TerminalNode},
};
use std::{fmt::Write, vec::IntoIter};
#[derive(Debug)]
pub enum TokenKind {
    //"a"
    T0,
    //Layout
    T1,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "\"a\"",
            TokenKind::T1 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    A(A),
    StartA(StartA),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::A(a) => a.as_parse_tree_ref(),
            ParseTree::StartA(start_a) => start_a.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
        }
    }
    fn unwrap_a(self) -> A {
        match self {
            ParseTree::A(a) => a,
            _ => panic!(),
        }
    }
    fn unwrap_start_a(self) -> StartA {
        match self {
            ParseTree::StartA(start_a) => start_a,
            _ => panic!(),
        }
    }
    fn unwrap_token(self) -> Token {
        match self {
            ParseTree::Token(t) => t,
            _ => panic!(),
        }
    }
}
#[derive(Clone, Copy)]
pub enum ParseTreeRef<'a> {
    A(&'a A),
    StartA(&'a StartA),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::A(a) => (0..a.child_count()).filter_map(|i| a.child(i)).collect(),
            ParseTreeRef::StartA(start_a) => (0..start_a.child_count())
                .filter_map(|i| start_a.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::A(_) => "A",
            ParseTreeRef::StartA(_) => "StartA",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::A(a) => a.child_count(),
            ParseTreeRef::StartA(start_a) => start_a.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::A(a) => a.span(),
            ParseTreeRef::StartA(start_a) => start_a.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
    }
}
impl From<A> for ParseTree {
    fn from(a: A) -> Self {
        ParseTree::A(a)
    }
}
impl From<StartA> for ParseTree {
    fn from(start_a: StartA) -> Self {
        ParseTree::StartA(start_a)
    }
}
pub trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
pub trait OptNode {
    type Inner;
    fn value(&self) -> Option<&Self::Inner>;
}
#[derive(Debug)]
pub enum A {
    //A Layout "a"
    Alt0 {
        a: Box<A>,
        layout: Token,
        lit_2: Token,
        span: Span,
    },
    //"a"
    Alt1 {
        lit_0: Token,
        span: Span,
    },
}
//StartA = Layout start:A Layout
#[derive(Debug)]
pub struct StartA {
    pub layout_0: Token,
    pub start: A,
    pub layout_2: Token,
    pub span: Span,
}
impl A {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            A::Alt0 {
                a, layout, lit_2, ..
            } => match index {
                0 => Some(a.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            A::Alt1 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            A::Alt0 { .. } => 3usize,
            A::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::A(self)
    }
    pub fn span(&self) -> Span {
        match self {
            A::Alt0 { span, .. } => *span,
            A::Alt1 { span, .. } => *span,
        }
    }
}
impl StartA {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.layout_0.as_parse_tree_ref()),
            1 => Some(self.start.as_parse_tree_ref()),
            2 => Some(self.layout_2.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::StartA(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    span: Span,
}
impl Token {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Token(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
fn token_kind(terminal_id: TerminalId) -> TokenKind {
    match terminal_id {
        //"a"
        TerminalId(0) => TokenKind::T0,
        //Layout
        TerminalId(1) => TokenKind::T1,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct LeftRecursiveListParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for LeftRecursiveListParseTreeBuilder {
    fn new_nonterminal_node(
        &self,
        nonterminal_node: &NonterminalNode,
        children: OneOrMany<ParseTree>,
    ) -> ParseTree {
        let children = children.into_vec();
        match nonterminal_node.nonterminal_id {
            //A
            NonterminalId(0) => {
                match nonterminal_node.return_slot {
                    //A : A Layout "a".
                    SlotId(3) => {
                        let [a, layout, lit_2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        A::Alt0 {
                            a: Box::new(a.unwrap_a()),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //A : "a".
                    SlotId(5) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        A::Alt1 {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartA
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //StartA : Layout start:A Layout.
                    SlotId(9) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartA {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_a(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
    fn new_token(&self, terminal_node: &TerminalNode) -> ParseTree {
        ParseTree::Token(Token {
            kind: token_kind(terminal_node.terminal_id),
            span: terminal_node.span,
        })
    }
}
pub fn create_parse_tree(
    root_id: SPPFNodeId,
    name: &str,
    parser: &LeftRecursiveListParser,
    builder: &LeftRecursiveListParseTreeBuilder,
) -> ParseTree {
    match name {
        "A" => ParseTree::A(create_parse_tree_a(root_id, parser, builder)),
        "StartA" => ParseTree::StartA(create_parse_tree_start_a(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_a(
    root_id: SPPFNodeId,
    parser: &LeftRecursiveListParser,
    builder: &LeftRecursiveListParseTreeBuilder,
) -> A {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_a()
}
pub fn create_parse_tree_start_a(
    root_id: SPPFNodeId,
    parser: &LeftRecursiveListParser,
    builder: &LeftRecursiveListParseTreeBuilder,
) -> StartA {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_a()
}
pub fn to_sexpr(node: ParseTreeRef<'_>) -> String {
    let mut s = String::new();
    node_to_sexpr(node, 0, &mut s).expect("error");
    s
}
fn node_to_sexpr(node: ParseTreeRef<'_>, indent: usize, w: &mut impl Write) -> fmt::Result {
    let children = node.children();
    if children.is_empty() {
        writeln!(w, "{:indent$}{}", "", node.display_name())
    } else {
        writeln!(w, "{:indent$}({}", "", node.display_name())?;
        for child in children {
            node_to_sexpr(child, indent + 2, w)?;
        }
        writeln!(w, "{:indent$})", "")
    }
}
/// Converts a parse tree to JSON format for visualization.
/// Returns a JSON string with nodes and edges arrays.
pub fn to_json(node: ParseTreeRef<'_>) -> String {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut next_id = 0u32;
    build_json_graph(node, &mut nodes, &mut edges, &mut next_id);
    let result = serde_json::json!({ "nodes" : nodes, "edges" : edges });
    result.to_string()
}
fn build_json_graph(
    node: ParseTreeRef<'_>,
    nodes: &mut Vec<serde_json::Value>,
    edges: &mut Vec<serde_json::Value>,
    next_id: &mut u32,
) -> u32 {
    let my_id = *next_id;
    *next_id += 1;
    let span = node.span();
    let kind = match node {
        ParseTreeRef::Token(_) => "Token",
        _ => "Nonterminal",
    };
    nodes.push(serde_json::json!(
        { "id" : my_id, "kind" : kind, "label" : node.display_name(), "start" :
        span.left_extent, "end" : span.right_extent }
    ));
    for child in node.children() {
        let child_id = build_json_graph(child, nodes, edges, next_id);
        edges.push(serde_json::json!({ "src" : my_id, "dest" : child_id }));
    }
    my_id
}

