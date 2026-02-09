use crate::parser::PlusParser;
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
    S(S),
    A(A),
    //A+
    SPlus0(SPlus0),
    StartS(StartS),
    StartA(StartA),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::S(s) => s.as_parse_tree_ref(),
            ParseTree::A(a) => a.as_parse_tree_ref(),
            ParseTree::SPlus0(s_plus_0) => s_plus_0.as_parse_tree_ref(),
            ParseTree::StartS(start_s) => start_s.as_parse_tree_ref(),
            ParseTree::StartA(start_a) => start_a.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
        }
    }
    fn unwrap_s(self) -> S {
        match self {
            ParseTree::S(s) => s,
            _ => panic!(),
        }
    }
    fn unwrap_a(self) -> A {
        match self {
            ParseTree::A(a) => a,
            _ => panic!(),
        }
    }
    fn unwrap_s_plus_0(self) -> SPlus0 {
        match self {
            ParseTree::SPlus0(s_plus_0) => s_plus_0,
            _ => panic!(),
        }
    }
    fn unwrap_start_s(self) -> StartS {
        match self {
            ParseTree::StartS(start_s) => start_s,
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
    S(&'a S),
    A(&'a A),
    SPlus0(&'a SPlus0),
    StartS(&'a StartS),
    StartA(&'a StartA),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::S(s) => (0..s.child_count()).filter_map(|i| s.child(i)).collect(),
            ParseTreeRef::A(a) => (0..a.child_count()).filter_map(|i| a.child(i)).collect(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.iter().collect(),
            ParseTreeRef::StartS(start_s) => (0..start_s.child_count())
                .filter_map(|i| start_s.child(i))
                .collect(),
            ParseTreeRef::StartA(start_a) => (0..start_a.child_count())
                .filter_map(|i| start_a.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::S(_) => "S",
            ParseTreeRef::A(_) => "A",
            ParseTreeRef::SPlus0(_) => "A+",
            ParseTreeRef::StartS(_) => "StartS",
            ParseTreeRef::StartA(_) => "StartA",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::S(s) => s.child_count(),
            ParseTreeRef::A(a) => a.child_count(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.child_count(),
            ParseTreeRef::StartS(start_s) => start_s.child_count(),
            ParseTreeRef::StartA(start_a) => start_a.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::S(s) => s.span(),
            ParseTreeRef::A(a) => a.span(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.span(),
            ParseTreeRef::StartS(start_s) => start_s.span(),
            ParseTreeRef::StartA(start_a) => start_a.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
    }
}
impl From<S> for ParseTree {
    fn from(s: S) -> Self {
        ParseTree::S(s)
    }
}
impl From<A> for ParseTree {
    fn from(a: A) -> Self {
        ParseTree::A(a)
    }
}
impl From<SPlus0> for ParseTree {
    fn from(s_plus_0: SPlus0) -> Self {
        ParseTree::SPlus0(s_plus_0)
    }
}
impl From<StartS> for ParseTree {
    fn from(start_s: StartS) -> Self {
        ParseTree::StartS(start_s)
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
//S = A+
#[derive(Debug)]
pub struct S {
    pub r#as: SPlus0,
    pub span: Span,
}
//A = "a"
#[derive(Debug)]
pub struct A {
    pub lit_0: Token,
    pub span: Span,
}
//A+
#[derive(Debug)]
pub enum SPlus0 {
    //A+ Layout A
    Alt0 {
        r#as: Box<SPlus0>,
        layout: Token,
        a_2: Box<A>,
        span: Span,
    },
    //A
    Alt1 {
        a: Box<A>,
        span: Span,
    },
}
//StartS = Layout start:S Layout
#[derive(Debug)]
pub struct StartS {
    pub layout_0: Token,
    pub start: S,
    pub layout_2: Token,
    pub span: Span,
}
//StartA = Layout start:A Layout
#[derive(Debug)]
pub struct StartA {
    pub layout_0: Token,
    pub start: A,
    pub layout_2: Token,
    pub span: Span,
}
impl S {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.r#as.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::S(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl A {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.lit_0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::A(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl SPlus0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SPlus0::Alt0 {
                r#as, layout, a_2, ..
            } => match index {
                0 => Some(r#as.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(a_2.as_parse_tree_ref()),
                _ => None,
            },
            SPlus0::Alt1 { a, .. } => match index {
                0 => Some(a.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SPlus0::Alt0 { .. } => 3usize,
            SPlus0::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SPlus0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SPlus0::Alt0 { span, .. } => *span,
            SPlus0::Alt1 { span, .. } => *span,
        }
    }
    pub fn r#as(&self) -> impl Iterator<Item = &A> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::A(r) => Some(r),
            _ => None,
        })
    }
}
impl StartS {
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
        ParseTreeRef::StartS(self)
    }
    pub fn span(&self) -> Span {
        self.span
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
impl<'a> ListNode<'a> for SPlus0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                SPlus0::Alt0 {
                    r#as: rest,
                    layout: layout,
                    a_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                SPlus0::Alt1 { a: item, .. } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
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
pub struct PlusParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for PlusParseTreeBuilder {
    fn new_nonterminal_node(
        &self,
        nonterminal_node: &NonterminalNode,
        children: OneOrMany<ParseTree>,
    ) -> ParseTree {
        let children = children.into_vec();
        match nonterminal_node.nonterminal_id {
            //S
            NonterminalId(0) => {
                match nonterminal_node.return_slot {
                    //S : A+.
                    SlotId(1) => {
                        let [r#as] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        S {
                            r#as: r#as.unwrap_s_plus_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //A
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //A : "a".
                    SlotId(3) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        A {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //S_Plus_0
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //A+ : A+ Layout A.
                    SlotId(7) => {
                        let [r#as, layout, a_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        SPlus0::Alt0 {
                            r#as: Box::new(r#as.unwrap_s_plus_0()),
                            layout: layout.unwrap_token(),
                            a_2: Box::new(a_2.unwrap_a()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //A+ : A.
                    SlotId(9) => {
                        let [a] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SPlus0::Alt1 {
                            a: Box::new(a.unwrap_a()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartS
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //StartS : Layout start:S Layout.
                    SlotId(13) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartS {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_s(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartA
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //StartA : Layout start:A Layout.
                    SlotId(17) => {
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
    parser: &PlusParser,
    builder: &PlusParseTreeBuilder,
) -> ParseTree {
    match name {
        "S" => ParseTree::S(create_parse_tree_s(root_id, parser, builder)),
        "A" => ParseTree::A(create_parse_tree_a(root_id, parser, builder)),
        "S_Plus_0" => ParseTree::SPlus0(create_parse_tree_s_plus_0(root_id, parser, builder)),
        "StartS" => ParseTree::StartS(create_parse_tree_start_s(root_id, parser, builder)),
        "StartA" => ParseTree::StartA(create_parse_tree_start_a(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_s(
    root_id: SPPFNodeId,
    parser: &PlusParser,
    builder: &PlusParseTreeBuilder,
) -> S {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_s()
}
pub fn create_parse_tree_a(
    root_id: SPPFNodeId,
    parser: &PlusParser,
    builder: &PlusParseTreeBuilder,
) -> A {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_a()
}
pub fn create_parse_tree_s_plus_0(
    root_id: SPPFNodeId,
    parser: &PlusParser,
    builder: &PlusParseTreeBuilder,
) -> SPlus0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_s_plus_0()
}
pub fn create_parse_tree_start_s(
    root_id: SPPFNodeId,
    parser: &PlusParser,
    builder: &PlusParseTreeBuilder,
) -> StartS {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_s()
}
pub fn create_parse_tree_start_a(
    root_id: SPPFNodeId,
    parser: &PlusParser,
    builder: &PlusParseTreeBuilder,
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

