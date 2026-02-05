use crate::parser::PlusGroupParser;
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
    //"b"
    T1,
    //"c"
    T2,
    //Layout
    T3,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "\"a\"",
            TokenKind::T1 => "\"b\"",
            TokenKind::T2 => "\"c\"",
            TokenKind::T3 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    S(S),
    A(A),
    B(B),
    C(C),
    //(A B C)
    SGroup0(SGroup0),
    //(A B C)+
    SPlus0(SPlus0),
    StartS(StartS),
    StartA(StartA),
    StartB(StartB),
    StartC(StartC),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::S(s) => s.as_parse_tree_ref(),
            ParseTree::A(a) => a.as_parse_tree_ref(),
            ParseTree::B(b) => b.as_parse_tree_ref(),
            ParseTree::C(c) => c.as_parse_tree_ref(),
            ParseTree::SGroup0(s_group_0) => s_group_0.as_parse_tree_ref(),
            ParseTree::SPlus0(s_plus_0) => s_plus_0.as_parse_tree_ref(),
            ParseTree::StartS(start_s) => start_s.as_parse_tree_ref(),
            ParseTree::StartA(start_a) => start_a.as_parse_tree_ref(),
            ParseTree::StartB(start_b) => start_b.as_parse_tree_ref(),
            ParseTree::StartC(start_c) => start_c.as_parse_tree_ref(),
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
    fn unwrap_b(self) -> B {
        match self {
            ParseTree::B(b) => b,
            _ => panic!(),
        }
    }
    fn unwrap_c(self) -> C {
        match self {
            ParseTree::C(c) => c,
            _ => panic!(),
        }
    }
    fn unwrap_s_group_0(self) -> SGroup0 {
        match self {
            ParseTree::SGroup0(s_group_0) => s_group_0,
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
    fn unwrap_start_b(self) -> StartB {
        match self {
            ParseTree::StartB(start_b) => start_b,
            _ => panic!(),
        }
    }
    fn unwrap_start_c(self) -> StartC {
        match self {
            ParseTree::StartC(start_c) => start_c,
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
    B(&'a B),
    C(&'a C),
    SGroup0(&'a SGroup0),
    SPlus0(&'a SPlus0),
    StartS(&'a StartS),
    StartA(&'a StartA),
    StartB(&'a StartB),
    StartC(&'a StartC),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::S(s) => (0..s.child_count()).filter_map(|i| s.child(i)).collect(),
            ParseTreeRef::A(a) => (0..a.child_count()).filter_map(|i| a.child(i)).collect(),
            ParseTreeRef::B(b) => (0..b.child_count()).filter_map(|i| b.child(i)).collect(),
            ParseTreeRef::C(c) => (0..c.child_count()).filter_map(|i| c.child(i)).collect(),
            ParseTreeRef::SGroup0(s_group_0) => (0..s_group_0.child_count())
                .filter_map(|i| s_group_0.child(i))
                .collect(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.iter().collect(),
            ParseTreeRef::StartS(start_s) => (0..start_s.child_count())
                .filter_map(|i| start_s.child(i))
                .collect(),
            ParseTreeRef::StartA(start_a) => (0..start_a.child_count())
                .filter_map(|i| start_a.child(i))
                .collect(),
            ParseTreeRef::StartB(start_b) => (0..start_b.child_count())
                .filter_map(|i| start_b.child(i))
                .collect(),
            ParseTreeRef::StartC(start_c) => (0..start_c.child_count())
                .filter_map(|i| start_c.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::S(_) => "S",
            ParseTreeRef::A(_) => "A",
            ParseTreeRef::B(_) => "B",
            ParseTreeRef::C(_) => "C",
            ParseTreeRef::SGroup0(_) => "(A B C)",
            ParseTreeRef::SPlus0(_) => "(A B C)+",
            ParseTreeRef::StartS(_) => "StartS",
            ParseTreeRef::StartA(_) => "StartA",
            ParseTreeRef::StartB(_) => "StartB",
            ParseTreeRef::StartC(_) => "StartC",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::S(s) => s.child_count(),
            ParseTreeRef::A(a) => a.child_count(),
            ParseTreeRef::B(b) => b.child_count(),
            ParseTreeRef::C(c) => c.child_count(),
            ParseTreeRef::SGroup0(s_group_0) => s_group_0.child_count(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.child_count(),
            ParseTreeRef::StartS(start_s) => start_s.child_count(),
            ParseTreeRef::StartA(start_a) => start_a.child_count(),
            ParseTreeRef::StartB(start_b) => start_b.child_count(),
            ParseTreeRef::StartC(start_c) => start_c.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::S(s) => s.span(),
            ParseTreeRef::A(a) => a.span(),
            ParseTreeRef::B(b) => b.span(),
            ParseTreeRef::C(c) => c.span(),
            ParseTreeRef::SGroup0(s_group_0) => s_group_0.span(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.span(),
            ParseTreeRef::StartS(start_s) => start_s.span(),
            ParseTreeRef::StartA(start_a) => start_a.span(),
            ParseTreeRef::StartB(start_b) => start_b.span(),
            ParseTreeRef::StartC(start_c) => start_c.span(),
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
impl From<B> for ParseTree {
    fn from(b: B) -> Self {
        ParseTree::B(b)
    }
}
impl From<C> for ParseTree {
    fn from(c: C) -> Self {
        ParseTree::C(c)
    }
}
impl From<SGroup0> for ParseTree {
    fn from(s_group_0: SGroup0) -> Self {
        ParseTree::SGroup0(s_group_0)
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
impl From<StartB> for ParseTree {
    fn from(start_b: StartB) -> Self {
        ParseTree::StartB(start_b)
    }
}
impl From<StartC> for ParseTree {
    fn from(start_c: StartC) -> Self {
        ParseTree::StartC(start_c)
    }
}
pub trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
pub trait OptNode {
    type Inner;
    fn value(&self) -> Option<&Self::Inner>;
}
//S = (A B C)+
#[derive(Debug)]
pub struct S {
    pub s_plus_0: SPlus0,
    pub span: Span,
}
//A = "a"
#[derive(Debug)]
pub struct A {
    pub lit_0: Token,
    pub span: Span,
}
//B = "b"
#[derive(Debug)]
pub struct B {
    pub lit_0: Token,
    pub span: Span,
}
//C = "c"
#[derive(Debug)]
pub struct C {
    pub lit_0: Token,
    pub span: Span,
}
//(A B C)
#[derive(Debug)]
pub struct SGroup0 {
    pub a: Box<A>,
    pub layout_1: Token,
    pub b: Box<B>,
    pub layout_3: Token,
    pub c: Box<C>,
    pub span: Span,
}
//(A B C)+
#[derive(Debug)]
pub enum SPlus0 {
    //(A B C)+ Layout (A B C)
    Alt0 {
        s_plus_0: Box<SPlus0>,
        layout: Token,
        s_group_0: SGroup0,
        span: Span,
    },
    //(A B C)
    Alt1 {
        s_group_0: SGroup0,
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
//StartB = Layout start:B Layout
#[derive(Debug)]
pub struct StartB {
    pub layout_0: Token,
    pub start: B,
    pub layout_2: Token,
    pub span: Span,
}
//StartC = Layout start:C Layout
#[derive(Debug)]
pub struct StartC {
    pub layout_0: Token,
    pub start: C,
    pub layout_2: Token,
    pub span: Span,
}
impl S {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.s_plus_0.as_parse_tree_ref()),
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
impl B {
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
        ParseTreeRef::B(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl C {
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
        ParseTreeRef::C(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl SGroup0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.a.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.b.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.c.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        5usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SGroup0(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl SPlus0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SPlus0::Alt0 {
                s_plus_0,
                layout,
                s_group_0,
                ..
            } => match index {
                0 => Some(s_plus_0.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(s_group_0.as_parse_tree_ref()),
                _ => None,
            },
            SPlus0::Alt1 { s_group_0, .. } => match index {
                0 => Some(s_group_0.as_parse_tree_ref()),
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
impl StartB {
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
        ParseTreeRef::StartB(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartC {
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
        ParseTreeRef::StartC(self)
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
                    s_plus_0: rest,
                    layout: layout,
                    s_group_0: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                SPlus0::Alt1 {
                    s_group_0: item, ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for SGroup0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        items.push(self.a.as_parse_tree_ref());
        items.push(self.layout_1.as_parse_tree_ref());
        items.push(self.b.as_parse_tree_ref());
        items.push(self.layout_3.as_parse_tree_ref());
        items.push(self.c.as_parse_tree_ref());
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
        //"b"
        TerminalId(1) => TokenKind::T1,
        //"c"
        TerminalId(2) => TokenKind::T2,
        //Layout
        TerminalId(3) => TokenKind::T3,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct PlusGroupParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for PlusGroupParseTreeBuilder {
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
                    //S : (A B C)+.
                    SlotId(1) => {
                        let [s_plus_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        S {
                            s_plus_0: s_plus_0.unwrap_s_plus_0(),
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
            //B
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //B : "b".
                    SlotId(5) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        B {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //C
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //C : "c".
                    SlotId(7) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        C {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //S_Group_0
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //(A B C) : A Layout B Layout C.
                    SlotId(13) => {
                        let [a, layout_1, b, layout_3, c] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        SGroup0 {
                            a: Box::new(a.unwrap_a()),
                            layout_1: layout_1.unwrap_token(),
                            b: Box::new(b.unwrap_b()),
                            layout_3: layout_3.unwrap_token(),
                            c: Box::new(c.unwrap_c()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //S_Plus_0
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //(A B C)+ : (A B C)+ Layout (A B C).
                    SlotId(17) => {
                        let [s_plus_0, layout, s_group_0] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        SPlus0::Alt0 {
                            s_plus_0: Box::new(s_plus_0.unwrap_s_plus_0()),
                            layout: layout.unwrap_token(),
                            s_group_0: s_group_0.unwrap_s_group_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //(A B C)+ : (A B C).
                    SlotId(19) => {
                        let [s_group_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SPlus0::Alt1 {
                            s_group_0: s_group_0.unwrap_s_group_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartS
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //StartS : Layout S Layout.
                    SlotId(23) => {
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
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //StartA : Layout A Layout.
                    SlotId(27) => {
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
            //StartB
            NonterminalId(8) => {
                match nonterminal_node.return_slot {
                    //StartB : Layout B Layout.
                    SlotId(31) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartB {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_b(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartC
            NonterminalId(9) => {
                match nonterminal_node.return_slot {
                    //StartC : Layout C Layout.
                    SlotId(35) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartC {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_c(),
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
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> ParseTree {
    match name {
        "S" => ParseTree::S(create_parse_tree_s(root_id, parser, builder)),
        "A" => ParseTree::A(create_parse_tree_a(root_id, parser, builder)),
        "B" => ParseTree::B(create_parse_tree_b(root_id, parser, builder)),
        "C" => ParseTree::C(create_parse_tree_c(root_id, parser, builder)),
        "S_Group_0" => ParseTree::SGroup0(create_parse_tree_s_group_0(root_id, parser, builder)),
        "S_Plus_0" => ParseTree::SPlus0(create_parse_tree_s_plus_0(root_id, parser, builder)),
        "StartS" => ParseTree::StartS(create_parse_tree_start_s(root_id, parser, builder)),
        "StartA" => ParseTree::StartA(create_parse_tree_start_a(root_id, parser, builder)),
        "StartB" => ParseTree::StartB(create_parse_tree_start_b(root_id, parser, builder)),
        "StartC" => ParseTree::StartC(create_parse_tree_start_c(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_s(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> S {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_s()
}
pub fn create_parse_tree_a(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> A {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_a()
}
pub fn create_parse_tree_b(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> B {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_b()
}
pub fn create_parse_tree_c(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> C {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_c()
}
pub fn create_parse_tree_s_group_0(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> SGroup0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_s_group_0()
}
pub fn create_parse_tree_s_plus_0(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> SPlus0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_s_plus_0()
}
pub fn create_parse_tree_start_s(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> StartS {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_s()
}
pub fn create_parse_tree_start_a(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> StartA {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_a()
}
pub fn create_parse_tree_start_b(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> StartB {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_b()
}
pub fn create_parse_tree_start_c(
    root_id: SPPFNodeId,
    parser: &PlusGroupParser,
    builder: &PlusGroupParseTreeBuilder,
) -> StartC {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_c()
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

