use crate::parser::OptParser;
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
    //A?
    SOpt0(SOpt0),
    StartS(StartS),
    StartA(StartA),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::S(s) => s.as_parse_tree_ref(),
            ParseTree::A(a) => a.as_parse_tree_ref(),
            ParseTree::SOpt0(s_opt_0) => s_opt_0.as_parse_tree_ref(),
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
    fn unwrap_s_opt_0(self) -> SOpt0 {
        match self {
            ParseTree::SOpt0(s_opt_0) => s_opt_0,
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
    SOpt0(&'a SOpt0),
    StartS(&'a StartS),
    StartA(&'a StartA),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::S(s) => (0..s.child_count()).filter_map(|i| s.child(i)).collect(),
            ParseTreeRef::A(a) => (0..a.child_count()).filter_map(|i| a.child(i)).collect(),
            ParseTreeRef::SOpt0(s_opt_0) => (0..s_opt_0.child_count())
                .filter_map(|i| s_opt_0.child(i))
                .collect(),
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
            ParseTreeRef::SOpt0(_) => "A?",
            ParseTreeRef::StartS(_) => "StartS",
            ParseTreeRef::StartA(_) => "StartA",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::S(s) => s.child_count(),
            ParseTreeRef::A(a) => a.child_count(),
            ParseTreeRef::SOpt0(s_opt_0) => s_opt_0.child_count(),
            ParseTreeRef::StartS(start_s) => start_s.child_count(),
            ParseTreeRef::StartA(start_a) => start_a.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::S(s) => s.span(),
            ParseTreeRef::A(a) => a.span(),
            ParseTreeRef::SOpt0(s_opt_0) => s_opt_0.span(),
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
impl From<SOpt0> for ParseTree {
    fn from(s_opt_0: SOpt0) -> Self {
        ParseTree::SOpt0(s_opt_0)
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
//S = A?
#[derive(Debug)]
pub struct S {
    pub a: SOpt0,
    pub span: Span,
}
//A = "a"
#[derive(Debug)]
pub struct A {
    pub lit_0: Token,
    pub span: Span,
}
//A?
#[derive(Debug)]
pub enum SOpt0 {
    //A
    Alt0 { a: Box<A>, span: Span },
    //
    Alt1 { span: Span },
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
            0 => Some(self.a.as_parse_tree_ref()),
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
impl SOpt0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SOpt0::Alt0 { a, .. } => match index {
                0 => Some(a.as_parse_tree_ref()),
                _ => None,
            },
            SOpt0::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SOpt0::Alt0 { .. } => 1usize,
            SOpt0::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SOpt0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SOpt0::Alt0 { span, .. } => *span,
            SOpt0::Alt1 { span, .. } => *span,
        }
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
impl OptNode for SOpt0 {
    type Inner = A;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            SOpt0::Alt0 { a, .. } => Some(a),
            SOpt0::Alt1 { .. } => None,
        }
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
pub struct OptParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for OptParseTreeBuilder {
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
                    //S : A?.
                    SlotId(1) => {
                        let [a] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        S {
                            a: a.unwrap_s_opt_0(),
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
            //S_Opt_0
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //A? : A.
                    SlotId(5) => {
                        let [a] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SOpt0::Alt0 {
                            a: Box::new(a.unwrap_a()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //A? : .
                    SlotId(6) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        SOpt0::Alt1 {
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
                    //StartS : Layout S Layout.
                    SlotId(10) => {
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
                    //StartA : Layout A Layout.
                    SlotId(14) => {
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
    parser: &OptParser,
    builder: &OptParseTreeBuilder,
) -> ParseTree {
    match name {
        "S" => ParseTree::S(create_parse_tree_s(root_id, parser, builder)),
        "A" => ParseTree::A(create_parse_tree_a(root_id, parser, builder)),
        "S_Opt_0" => ParseTree::SOpt0(create_parse_tree_s_opt_0(root_id, parser, builder)),
        "StartS" => ParseTree::StartS(create_parse_tree_start_s(root_id, parser, builder)),
        "StartA" => ParseTree::StartA(create_parse_tree_start_a(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_s(
    root_id: SPPFNodeId,
    parser: &OptParser,
    builder: &OptParseTreeBuilder,
) -> S {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_s()
}
pub fn create_parse_tree_a(
    root_id: SPPFNodeId,
    parser: &OptParser,
    builder: &OptParseTreeBuilder,
) -> A {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_a()
}
pub fn create_parse_tree_s_opt_0(
    root_id: SPPFNodeId,
    parser: &OptParser,
    builder: &OptParseTreeBuilder,
) -> SOpt0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_s_opt_0()
}
pub fn create_parse_tree_start_s(
    root_id: SPPFNodeId,
    parser: &OptParser,
    builder: &OptParseTreeBuilder,
) -> StartS {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_s()
}
pub fn create_parse_tree_start_a(
    root_id: SPPFNodeId,
    parser: &OptParser,
    builder: &OptParseTreeBuilder,
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

