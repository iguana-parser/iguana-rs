use crate::parser::ExceptTerminalParser;
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
    //Identifier
    T0,
    //Keyword
    T1,
    //Layout
    T2,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Identifier",
            TokenKind::T1 => "Keyword",
            TokenKind::T2 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    S(S),
    Id(Id),
    StartS(StartS),
    StartId(StartId),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::S(s) => s.as_parse_tree_ref(),
            ParseTree::Id(id) => id.as_parse_tree_ref(),
            ParseTree::StartS(start_s) => start_s.as_parse_tree_ref(),
            ParseTree::StartId(start_id) => start_id.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
        }
    }
    fn unwrap_s(self) -> S {
        match self {
            ParseTree::S(s) => s,
            _ => panic!(),
        }
    }
    fn unwrap_id(self) -> Id {
        match self {
            ParseTree::Id(id) => id,
            _ => panic!(),
        }
    }
    fn unwrap_start_s(self) -> StartS {
        match self {
            ParseTree::StartS(start_s) => start_s,
            _ => panic!(),
        }
    }
    fn unwrap_start_id(self) -> StartId {
        match self {
            ParseTree::StartId(start_id) => start_id,
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
    Id(&'a Id),
    StartS(&'a StartS),
    StartId(&'a StartId),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::S(s) => (0..s.child_count()).filter_map(|i| s.child(i)).collect(),
            ParseTreeRef::Id(id) => (0..id.child_count()).filter_map(|i| id.child(i)).collect(),
            ParseTreeRef::StartS(start_s) => (0..start_s.child_count())
                .filter_map(|i| start_s.child(i))
                .collect(),
            ParseTreeRef::StartId(start_id) => (0..start_id.child_count())
                .filter_map(|i| start_id.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::S(_) => "S",
            ParseTreeRef::Id(_) => "Id",
            ParseTreeRef::StartS(_) => "StartS",
            ParseTreeRef::StartId(_) => "StartId",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::S(s) => s.child_count(),
            ParseTreeRef::Id(id) => id.child_count(),
            ParseTreeRef::StartS(start_s) => start_s.child_count(),
            ParseTreeRef::StartId(start_id) => start_id.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::S(s) => s.span(),
            ParseTreeRef::Id(id) => id.span(),
            ParseTreeRef::StartS(start_s) => start_s.span(),
            ParseTreeRef::StartId(start_id) => start_id.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
    }
}
impl From<S> for ParseTree {
    fn from(s: S) -> Self {
        ParseTree::S(s)
    }
}
impl From<Id> for ParseTree {
    fn from(id: Id) -> Self {
        ParseTree::Id(id)
    }
}
impl From<StartS> for ParseTree {
    fn from(start_s: StartS) -> Self {
        ParseTree::StartS(start_s)
    }
}
impl From<StartId> for ParseTree {
    fn from(start_id: StartId) -> Self {
        ParseTree::StartId(start_id)
    }
}
pub trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
pub trait OptNode {
    type Inner;
    fn value(&self) -> Option<&Self::Inner>;
}
//S = Id
#[derive(Debug)]
pub struct S {
    pub id: Id,
    pub span: Span,
}
//Id = Identifier \ Keyword
#[derive(Debug)]
pub struct Id {
    pub identifier: Token,
    pub span: Span,
}
//StartS = Layout start:S Layout
#[derive(Debug)]
pub struct StartS {
    pub layout_0: Token,
    pub start: S,
    pub layout_2: Token,
    pub span: Span,
}
//StartId = Layout start:Id Layout
#[derive(Debug)]
pub struct StartId {
    pub layout_0: Token,
    pub start: Id,
    pub layout_2: Token,
    pub span: Span,
}
impl S {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.id.as_parse_tree_ref()),
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
impl Id {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.identifier.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Id(self)
    }
    pub fn span(&self) -> Span {
        self.span
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
impl StartId {
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
        ParseTreeRef::StartId(self)
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
        //Identifier
        TerminalId(0) => TokenKind::T0,
        //Keyword
        TerminalId(1) => TokenKind::T1,
        //Layout
        TerminalId(2) => TokenKind::T2,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct ExceptTerminalParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for ExceptTerminalParseTreeBuilder {
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
                    //S : Id.
                    SlotId(1) => {
                        let [id] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        S {
                            id: id.unwrap_id(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Id
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //Id : Identifier \ Keyword.
                    SlotId(3) => {
                        let [identifier] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Id {
                            identifier: identifier.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartS
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //StartS : Layout start:S Layout.
                    SlotId(7) => {
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
            //StartId
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //StartId : Layout start:Id Layout.
                    SlotId(11) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartId {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_id(),
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
    parser: &ExceptTerminalParser,
    builder: &ExceptTerminalParseTreeBuilder,
) -> ParseTree {
    match name {
        "S" => ParseTree::S(create_parse_tree_s(root_id, parser, builder)),
        "Id" => ParseTree::Id(create_parse_tree_id(root_id, parser, builder)),
        "StartS" => ParseTree::StartS(create_parse_tree_start_s(root_id, parser, builder)),
        "StartId" => ParseTree::StartId(create_parse_tree_start_id(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_s(
    root_id: SPPFNodeId,
    parser: &ExceptTerminalParser,
    builder: &ExceptTerminalParseTreeBuilder,
) -> S {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_s()
}
pub fn create_parse_tree_id(
    root_id: SPPFNodeId,
    parser: &ExceptTerminalParser,
    builder: &ExceptTerminalParseTreeBuilder,
) -> Id {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_id()
}
pub fn create_parse_tree_start_s(
    root_id: SPPFNodeId,
    parser: &ExceptTerminalParser,
    builder: &ExceptTerminalParseTreeBuilder,
) -> StartS {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_s()
}
pub fn create_parse_tree_start_id(
    root_id: SPPFNodeId,
    parser: &ExceptTerminalParser,
    builder: &ExceptTerminalParseTreeBuilder,
) -> StartId {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_id()
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

