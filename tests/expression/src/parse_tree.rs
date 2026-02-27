use crate::parser::ExpressionParser;
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
    //"*"
    T0,
    //"+"
    T1,
    //"a"
    T2,
    //Layout
    T3,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "\"*\"",
            TokenKind::T1 => "\"+\"",
            TokenKind::T2 => "\"a\"",
            TokenKind::T3 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    E(E),
    StartE(StartE),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::E(e) => e.as_parse_tree_ref(),
            ParseTree::StartE(start_e) => start_e.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
        }
    }
    fn unwrap_e(self) -> E {
        match self {
            ParseTree::E(e) => e,
            _ => panic!(),
        }
    }
    fn unwrap_start_e(self) -> StartE {
        match self {
            ParseTree::StartE(start_e) => start_e,
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
    E(&'a E),
    StartE(&'a StartE),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::E(e) => (0..e.child_count()).filter_map(|i| e.child(i)).collect(),
            ParseTreeRef::StartE(start_e) => (0..start_e.child_count())
                .filter_map(|i| start_e.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::E(_) => "E",
            ParseTreeRef::StartE(_) => "StartE",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::E(e) => e.child_count(),
            ParseTreeRef::StartE(start_e) => start_e.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::E(e) => e.span(),
            ParseTreeRef::StartE(start_e) => start_e.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
    }
}
impl From<E> for ParseTree {
    fn from(e: E) -> Self {
        ParseTree::E(e)
    }
}
impl From<StartE> for ParseTree {
    fn from(start_e: StartE) -> Self {
        ParseTree::StartE(start_e)
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
pub enum E {
    //E Layout "*" Layout E #Mul
    Mul {
        e_0: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        e_4: Box<E>,
        span: Span,
    },
    //E Layout "+" Layout E #Add
    Add {
        e_0: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        e_4: Box<E>,
        span: Span,
    },
    //"a" #Lit
    Lit {
        lit_0: Token,
        span: Span,
    },
}
//StartE = Layout start:E Layout
#[derive(Debug)]
pub struct StartE {
    pub layout_0: Token,
    pub start: E,
    pub layout_2: Token,
    pub span: Span,
}
impl E {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            E::Mul {
                e_0,
                layout_1,
                lit_2,
                layout_3,
                e_4,
                ..
            } => match index {
                0 => Some(e_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(e_4.as_parse_tree_ref()),
                _ => None,
            },
            E::Add {
                e_0,
                layout_1,
                lit_2,
                layout_3,
                e_4,
                ..
            } => match index {
                0 => Some(e_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(e_4.as_parse_tree_ref()),
                _ => None,
            },
            E::Lit { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            E::Mul { .. } => 5usize,
            E::Add { .. } => 5usize,
            E::Lit { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::E(self)
    }
    pub fn span(&self) -> Span {
        match self {
            E::Mul { span, .. } => *span,
            E::Add { span, .. } => *span,
            E::Lit { span, .. } => *span,
        }
    }
}
impl StartE {
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
        ParseTreeRef::StartE(self)
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
        //"*"
        TerminalId(0) => TokenKind::T0,
        //"+"
        TerminalId(1) => TokenKind::T1,
        //"a"
        TerminalId(2) => TokenKind::T2,
        //Layout
        TerminalId(3) => TokenKind::T3,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct ExpressionParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for ExpressionParseTreeBuilder {
    fn new_nonterminal_node(
        &self,
        nonterminal_node: &NonterminalNode,
        children: OneOrMany<ParseTree>,
    ) -> ParseTree {
        let children = children.into_vec();
        match nonterminal_node.nonterminal_id {
            //E
            NonterminalId(0) => {
                match nonterminal_node.return_slot {
                    //E : E Layout "*" Layout E.
                    SlotId(5) => {
                        let [e_0, layout_1, lit_2, layout_3, e_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Mul {
                            e_0: Box::new(e_0.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            e_4: Box::new(e_4.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : E Layout "+" Layout E.
                    SlotId(11) => {
                        let [e_0, layout_1, lit_2, layout_3, e_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Add {
                            e_0: Box::new(e_0.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            e_4: Box::new(e_4.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : "a".
                    SlotId(13) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        E::Lit {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartE
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //StartE : Layout start:E Layout.
                    SlotId(17) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartE {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_e(),
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
    parser: &ExpressionParser,
    builder: &ExpressionParseTreeBuilder,
) -> ParseTree {
    match name {
        "E" => ParseTree::E(create_parse_tree_e(root_id, parser, builder)),
        "StartE" => ParseTree::StartE(create_parse_tree_start_e(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_e(
    root_id: SPPFNodeId,
    parser: &ExpressionParser,
    builder: &ExpressionParseTreeBuilder,
) -> E {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_e()
}
pub fn create_parse_tree_start_e(
    root_id: SPPFNodeId,
    parser: &ExpressionParser,
    builder: &ExpressionParseTreeBuilder,
) -> StartE {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_e()
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

