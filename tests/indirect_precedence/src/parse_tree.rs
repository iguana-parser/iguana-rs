use crate::parser::IndirectPrecedenceParser;
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
    //"-"
    T0,
    //"*"
    T1,
    //"a"
    T2,
    //"/"
    T3,
    //Layout
    T4,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "\"-\"",
            TokenKind::T1 => "\"*\"",
            TokenKind::T2 => "\"a\"",
            TokenKind::T3 => "\"/\"",
            TokenKind::T4 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    S(S),
    E(E),
    F(F),
    K(K),
    StartS(StartS),
    StartE(StartE),
    StartF(StartF),
    StartK(StartK),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::S(s) => s.as_parse_tree_ref(),
            ParseTree::E(e) => e.as_parse_tree_ref(),
            ParseTree::F(f) => f.as_parse_tree_ref(),
            ParseTree::K(k) => k.as_parse_tree_ref(),
            ParseTree::StartS(start_s) => start_s.as_parse_tree_ref(),
            ParseTree::StartE(start_e) => start_e.as_parse_tree_ref(),
            ParseTree::StartF(start_f) => start_f.as_parse_tree_ref(),
            ParseTree::StartK(start_k) => start_k.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
        }
    }
    fn unwrap_s(self) -> S {
        match self {
            ParseTree::S(s) => s,
            _ => panic!(),
        }
    }
    fn unwrap_e(self) -> E {
        match self {
            ParseTree::E(e) => e,
            _ => panic!(),
        }
    }
    fn unwrap_f(self) -> F {
        match self {
            ParseTree::F(f) => f,
            _ => panic!(),
        }
    }
    fn unwrap_k(self) -> K {
        match self {
            ParseTree::K(k) => k,
            _ => panic!(),
        }
    }
    fn unwrap_start_s(self) -> StartS {
        match self {
            ParseTree::StartS(start_s) => start_s,
            _ => panic!(),
        }
    }
    fn unwrap_start_e(self) -> StartE {
        match self {
            ParseTree::StartE(start_e) => start_e,
            _ => panic!(),
        }
    }
    fn unwrap_start_f(self) -> StartF {
        match self {
            ParseTree::StartF(start_f) => start_f,
            _ => panic!(),
        }
    }
    fn unwrap_start_k(self) -> StartK {
        match self {
            ParseTree::StartK(start_k) => start_k,
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
    E(&'a E),
    F(&'a F),
    K(&'a K),
    StartS(&'a StartS),
    StartE(&'a StartE),
    StartF(&'a StartF),
    StartK(&'a StartK),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::S(s) => (0..s.child_count()).filter_map(|i| s.child(i)).collect(),
            ParseTreeRef::E(e) => (0..e.child_count()).filter_map(|i| e.child(i)).collect(),
            ParseTreeRef::F(f) => (0..f.child_count()).filter_map(|i| f.child(i)).collect(),
            ParseTreeRef::K(k) => (0..k.child_count()).filter_map(|i| k.child(i)).collect(),
            ParseTreeRef::StartS(start_s) => (0..start_s.child_count())
                .filter_map(|i| start_s.child(i))
                .collect(),
            ParseTreeRef::StartE(start_e) => (0..start_e.child_count())
                .filter_map(|i| start_e.child(i))
                .collect(),
            ParseTreeRef::StartF(start_f) => (0..start_f.child_count())
                .filter_map(|i| start_f.child(i))
                .collect(),
            ParseTreeRef::StartK(start_k) => (0..start_k.child_count())
                .filter_map(|i| start_k.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::S(_) => "S",
            ParseTreeRef::E(_) => "E",
            ParseTreeRef::F(_) => "F",
            ParseTreeRef::K(_) => "K",
            ParseTreeRef::StartS(_) => "StartS",
            ParseTreeRef::StartE(_) => "StartE",
            ParseTreeRef::StartF(_) => "StartF",
            ParseTreeRef::StartK(_) => "StartK",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::S(s) => s.child_count(),
            ParseTreeRef::E(e) => e.child_count(),
            ParseTreeRef::F(f) => f.child_count(),
            ParseTreeRef::K(k) => k.child_count(),
            ParseTreeRef::StartS(start_s) => start_s.child_count(),
            ParseTreeRef::StartE(start_e) => start_e.child_count(),
            ParseTreeRef::StartF(start_f) => start_f.child_count(),
            ParseTreeRef::StartK(start_k) => start_k.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::S(s) => s.span(),
            ParseTreeRef::E(e) => e.span(),
            ParseTreeRef::F(f) => f.span(),
            ParseTreeRef::K(k) => k.span(),
            ParseTreeRef::StartS(start_s) => start_s.span(),
            ParseTreeRef::StartE(start_e) => start_e.span(),
            ParseTreeRef::StartF(start_f) => start_f.span(),
            ParseTreeRef::StartK(start_k) => start_k.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
    }
}
impl From<S> for ParseTree {
    fn from(s: S) -> Self {
        ParseTree::S(s)
    }
}
impl From<E> for ParseTree {
    fn from(e: E) -> Self {
        ParseTree::E(e)
    }
}
impl From<F> for ParseTree {
    fn from(f: F) -> Self {
        ParseTree::F(f)
    }
}
impl From<K> for ParseTree {
    fn from(k: K) -> Self {
        ParseTree::K(k)
    }
}
impl From<StartS> for ParseTree {
    fn from(start_s: StartS) -> Self {
        ParseTree::StartS(start_s)
    }
}
impl From<StartE> for ParseTree {
    fn from(start_e: StartE) -> Self {
        ParseTree::StartE(start_e)
    }
}
impl From<StartF> for ParseTree {
    fn from(start_f: StartF) -> Self {
        ParseTree::StartF(start_f)
    }
}
impl From<StartK> for ParseTree {
    fn from(start_k: StartK) -> Self {
        ParseTree::StartK(start_k)
    }
}
pub trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
pub trait OptNode {
    type Inner;
    fn value(&self) -> Option<&Self::Inner>;
}
//S = E(0)
#[derive(Debug)]
pub struct S {
    pub e: E,
    pub span: Span,
}
#[derive(Debug)]
pub enum E {
    //"-" Layout E(2) return 2
    Alt0 {
        lit_0: Token,
        layout: Token,
        e: Box<E>,
        span: Span,
    },
    //[1 >= p] l=E(p) [l == 0 || l >= 1] Layout "*" Layout F return 0
    Alt1 {
        e: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        f: Box<F>,
        span: Span,
    },
    //"a" return 0
    Alt2 {
        lit_0: Token,
        span: Span,
    },
}
//F = E(0) Layout "/" Layout K
#[derive(Debug)]
pub struct F {
    pub e: Box<E>,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub k: Box<K>,
    pub span: Span,
}
//K = E(0)
#[derive(Debug)]
pub struct K {
    pub e: Box<E>,
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
//StartE = Layout start:E(0) Layout
#[derive(Debug)]
pub struct StartE {
    pub layout_0: Token,
    pub start: E,
    pub layout_2: Token,
    pub span: Span,
}
//StartF = Layout start:F Layout
#[derive(Debug)]
pub struct StartF {
    pub layout_0: Token,
    pub start: F,
    pub layout_2: Token,
    pub span: Span,
}
//StartK = Layout start:K Layout
#[derive(Debug)]
pub struct StartK {
    pub layout_0: Token,
    pub start: K,
    pub layout_2: Token,
    pub span: Span,
}
impl S {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.e.as_parse_tree_ref()),
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
impl E {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            E::Alt0 {
                lit_0, layout, e, ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(e.as_parse_tree_ref()),
                _ => None,
            },
            E::Alt1 {
                e,
                layout_1,
                lit_2,
                layout_3,
                f,
                ..
            } => match index {
                0 => Some(e.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(f.as_parse_tree_ref()),
                _ => None,
            },
            E::Alt2 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            E::Alt0 { .. } => 3usize,
            E::Alt1 { .. } => 5usize,
            E::Alt2 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::E(self)
    }
    pub fn span(&self) -> Span {
        match self {
            E::Alt0 { span, .. } => *span,
            E::Alt1 { span, .. } => *span,
            E::Alt2 { span, .. } => *span,
        }
    }
}
impl F {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.e.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.k.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        5usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::F(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl K {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.e.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::K(self)
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
impl StartF {
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
        ParseTreeRef::StartF(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartK {
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
        ParseTreeRef::StartK(self)
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
        //"-"
        TerminalId(0) => TokenKind::T0,
        //"*"
        TerminalId(1) => TokenKind::T1,
        //"a"
        TerminalId(2) => TokenKind::T2,
        //"/"
        TerminalId(3) => TokenKind::T3,
        //Layout
        TerminalId(4) => TokenKind::T4,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct IndirectPrecedenceParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for IndirectPrecedenceParseTreeBuilder {
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
                    //S : E(0).
                    SlotId(1) => {
                        let [e] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        S {
                            e: e.unwrap_e(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //F
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //F : E(0) Layout "/" Layout K.
                    SlotId(24) => {
                        let [e, layout_1, lit_2, layout_3, k] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        F {
                            e: Box::new(e.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            k: Box::new(k.unwrap_k()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //K
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //K : E(0).
                    SlotId(26) => {
                        let [e] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        K {
                            e: Box::new(e.unwrap_e()),
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
                    SlotId(30) => {
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
            //StartE
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //StartE : Layout start:E(0) Layout.
                    SlotId(34) => {
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
            //StartF
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //StartF : Layout start:F Layout.
                    SlotId(38) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartF {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_f(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartK
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //StartK : Layout start:K Layout.
                    SlotId(42) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartK {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_k(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //E
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //E : "-" Layout E(2) return 2.
                    SlotId(6) => {
                        let [lit_0, layout, e] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        E::Alt0 {
                            lit_0: lit_0.unwrap_token(),
                            layout: layout.unwrap_token(),
                            e: Box::new(e.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : [1 >= p] l=E(p) [l == 0 || l >= 1] Layout "*" Layout F return 0.
                    SlotId(15) => {
                        let [e, layout_1, lit_2, layout_3, f] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Alt1 {
                            e: Box::new(e.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            f: Box::new(f.unwrap_f()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : "a" return 0.
                    SlotId(18) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        E::Alt2 {
                            lit_0: lit_0.unwrap_token(),
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
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> ParseTree {
    match name {
        "S" => ParseTree::S(create_parse_tree_s(root_id, parser, builder)),
        "E" => ParseTree::E(create_parse_tree_e(root_id, parser, builder)),
        "F" => ParseTree::F(create_parse_tree_f(root_id, parser, builder)),
        "K" => ParseTree::K(create_parse_tree_k(root_id, parser, builder)),
        "StartS" => ParseTree::StartS(create_parse_tree_start_s(root_id, parser, builder)),
        "StartE" => ParseTree::StartE(create_parse_tree_start_e(root_id, parser, builder)),
        "StartF" => ParseTree::StartF(create_parse_tree_start_f(root_id, parser, builder)),
        "StartK" => ParseTree::StartK(create_parse_tree_start_k(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_s(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> S {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_s()
}
pub fn create_parse_tree_e(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> E {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_e()
}
pub fn create_parse_tree_f(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> F {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_f()
}
pub fn create_parse_tree_k(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> K {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_k()
}
pub fn create_parse_tree_start_s(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> StartS {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_s()
}
pub fn create_parse_tree_start_e(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> StartE {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_e()
}
pub fn create_parse_tree_start_f(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> StartF {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_f()
}
pub fn create_parse_tree_start_k(
    root_id: SPPFNodeId,
    parser: &IndirectPrecedenceParser,
    builder: &IndirectPrecedenceParseTreeBuilder,
) -> StartK {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_k()
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

