use crate::parser::GroupParser;
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
    //"b"
    T0,
    //"c"
    T1,
    //"d"
    T2,
    //Layout
    T3,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "\"b\"",
            TokenKind::T1 => "\"c\"",
            TokenKind::T2 => "\"d\"",
            TokenKind::T3 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    A(A),
    B(B),
    C(C),
    D(D),
    //(B C D)
    AGroup0(AGroup0),
    StartA(StartA),
    StartB(StartB),
    StartC(StartC),
    StartD(StartD),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::A(a) => a.as_parse_tree_ref(),
            ParseTree::B(b) => b.as_parse_tree_ref(),
            ParseTree::C(c) => c.as_parse_tree_ref(),
            ParseTree::D(d) => d.as_parse_tree_ref(),
            ParseTree::AGroup0(a_group_0) => a_group_0.as_parse_tree_ref(),
            ParseTree::StartA(start_a) => start_a.as_parse_tree_ref(),
            ParseTree::StartB(start_b) => start_b.as_parse_tree_ref(),
            ParseTree::StartC(start_c) => start_c.as_parse_tree_ref(),
            ParseTree::StartD(start_d) => start_d.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
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
    fn unwrap_d(self) -> D {
        match self {
            ParseTree::D(d) => d,
            _ => panic!(),
        }
    }
    fn unwrap_a_group_0(self) -> AGroup0 {
        match self {
            ParseTree::AGroup0(a_group_0) => a_group_0,
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
    fn unwrap_start_d(self) -> StartD {
        match self {
            ParseTree::StartD(start_d) => start_d,
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
    B(&'a B),
    C(&'a C),
    D(&'a D),
    AGroup0(&'a AGroup0),
    StartA(&'a StartA),
    StartB(&'a StartB),
    StartC(&'a StartC),
    StartD(&'a StartD),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::A(a) => (0..a.child_count()).filter_map(|i| a.child(i)).collect(),
            ParseTreeRef::B(b) => (0..b.child_count()).filter_map(|i| b.child(i)).collect(),
            ParseTreeRef::C(c) => (0..c.child_count()).filter_map(|i| c.child(i)).collect(),
            ParseTreeRef::D(d) => (0..d.child_count()).filter_map(|i| d.child(i)).collect(),
            ParseTreeRef::AGroup0(a_group_0) => (0..a_group_0.child_count())
                .filter_map(|i| a_group_0.child(i))
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
            ParseTreeRef::StartD(start_d) => (0..start_d.child_count())
                .filter_map(|i| start_d.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::A(_) => "A",
            ParseTreeRef::B(_) => "B",
            ParseTreeRef::C(_) => "C",
            ParseTreeRef::D(_) => "D",
            ParseTreeRef::AGroup0(_) => "(B C D)",
            ParseTreeRef::StartA(_) => "StartA",
            ParseTreeRef::StartB(_) => "StartB",
            ParseTreeRef::StartC(_) => "StartC",
            ParseTreeRef::StartD(_) => "StartD",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::A(a) => a.child_count(),
            ParseTreeRef::B(b) => b.child_count(),
            ParseTreeRef::C(c) => c.child_count(),
            ParseTreeRef::D(d) => d.child_count(),
            ParseTreeRef::AGroup0(a_group_0) => a_group_0.child_count(),
            ParseTreeRef::StartA(start_a) => start_a.child_count(),
            ParseTreeRef::StartB(start_b) => start_b.child_count(),
            ParseTreeRef::StartC(start_c) => start_c.child_count(),
            ParseTreeRef::StartD(start_d) => start_d.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::A(a) => a.span(),
            ParseTreeRef::B(b) => b.span(),
            ParseTreeRef::C(c) => c.span(),
            ParseTreeRef::D(d) => d.span(),
            ParseTreeRef::AGroup0(a_group_0) => a_group_0.span(),
            ParseTreeRef::StartA(start_a) => start_a.span(),
            ParseTreeRef::StartB(start_b) => start_b.span(),
            ParseTreeRef::StartC(start_c) => start_c.span(),
            ParseTreeRef::StartD(start_d) => start_d.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
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
impl From<D> for ParseTree {
    fn from(d: D) -> Self {
        ParseTree::D(d)
    }
}
impl From<AGroup0> for ParseTree {
    fn from(a_group_0: AGroup0) -> Self {
        ParseTree::AGroup0(a_group_0)
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
impl From<StartD> for ParseTree {
    fn from(start_d: StartD) -> Self {
        ParseTree::StartD(start_d)
    }
}
pub trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
pub trait OptNode {
    type Inner;
    fn value(&self) -> Option<&Self::Inner>;
}
//A = (B C D)
#[derive(Debug)]
pub struct A {
    pub a_group_0: AGroup0,
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
//D = "d"
#[derive(Debug)]
pub struct D {
    pub lit_0: Token,
    pub span: Span,
}
//(B C D)
#[derive(Debug)]
pub struct AGroup0 {
    pub b: Box<B>,
    pub layout_1: Token,
    pub c: Box<C>,
    pub layout_3: Token,
    pub d: Box<D>,
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
//StartD = Layout start:D Layout
#[derive(Debug)]
pub struct StartD {
    pub layout_0: Token,
    pub start: D,
    pub layout_2: Token,
    pub span: Span,
}
impl A {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.a_group_0.as_parse_tree_ref()),
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
impl D {
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
        ParseTreeRef::D(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl AGroup0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.b.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.c.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.d.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        5usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AGroup0(self)
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
impl StartD {
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
        ParseTreeRef::StartD(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl<'a> ListNode<'a> for AGroup0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        items.push(self.b.as_parse_tree_ref());
        items.push(self.layout_1.as_parse_tree_ref());
        items.push(self.c.as_parse_tree_ref());
        items.push(self.layout_3.as_parse_tree_ref());
        items.push(self.d.as_parse_tree_ref());
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
        //"b"
        TerminalId(0) => TokenKind::T0,
        //"c"
        TerminalId(1) => TokenKind::T1,
        //"d"
        TerminalId(2) => TokenKind::T2,
        //Layout
        TerminalId(3) => TokenKind::T3,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct GroupParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for GroupParseTreeBuilder {
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
                    //A : (B C D).
                    SlotId(1) => {
                        let [a_group_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        A {
                            a_group_0: a_group_0.unwrap_a_group_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //B
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //B : "b".
                    SlotId(3) => {
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
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //C : "c".
                    SlotId(5) => {
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
            //D
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //D : "d".
                    SlotId(7) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        D {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //A_Group_0
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //(B C D) : B Layout C Layout D.
                    SlotId(13) => {
                        let [b, layout_1, c, layout_3, d] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        AGroup0 {
                            b: Box::new(b.unwrap_b()),
                            layout_1: layout_1.unwrap_token(),
                            c: Box::new(c.unwrap_c()),
                            layout_3: layout_3.unwrap_token(),
                            d: Box::new(d.unwrap_d()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartA
            NonterminalId(5) => {
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
            //StartB
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //StartB : Layout start:B Layout.
                    SlotId(21) => {
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
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //StartC : Layout start:C Layout.
                    SlotId(25) => {
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
            //StartD
            NonterminalId(8) => {
                match nonterminal_node.return_slot {
                    //StartD : Layout start:D Layout.
                    SlotId(29) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartD {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_d(),
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
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> ParseTree {
    match name {
        "A" => ParseTree::A(create_parse_tree_a(root_id, parser, builder)),
        "B" => ParseTree::B(create_parse_tree_b(root_id, parser, builder)),
        "C" => ParseTree::C(create_parse_tree_c(root_id, parser, builder)),
        "D" => ParseTree::D(create_parse_tree_d(root_id, parser, builder)),
        "A_Group_0" => ParseTree::AGroup0(create_parse_tree_a_group_0(root_id, parser, builder)),
        "StartA" => ParseTree::StartA(create_parse_tree_start_a(root_id, parser, builder)),
        "StartB" => ParseTree::StartB(create_parse_tree_start_b(root_id, parser, builder)),
        "StartC" => ParseTree::StartC(create_parse_tree_start_c(root_id, parser, builder)),
        "StartD" => ParseTree::StartD(create_parse_tree_start_d(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_a(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> A {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_a()
}
pub fn create_parse_tree_b(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> B {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_b()
}
pub fn create_parse_tree_c(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> C {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_c()
}
pub fn create_parse_tree_d(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> D {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_d()
}
pub fn create_parse_tree_a_group_0(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> AGroup0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_a_group_0()
}
pub fn create_parse_tree_start_a(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> StartA {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_a()
}
pub fn create_parse_tree_start_b(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> StartB {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_b()
}
pub fn create_parse_tree_start_c(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> StartC {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_c()
}
pub fn create_parse_tree_start_d(
    root_id: SPPFNodeId,
    parser: &GroupParser,
    builder: &GroupParseTreeBuilder,
) -> StartD {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_d()
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

