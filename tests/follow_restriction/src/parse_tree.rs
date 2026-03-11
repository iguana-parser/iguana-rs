use crate::parser::FollowRestrictionParser;
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
    //Char
    T0,
    //WS
    T1,
    //Layout
    T2,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Char",
            TokenKind::T1 => "WS",
            TokenKind::T2 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    S(S),
    T(T),
    Id(Id),
    //Id+
    SPlus0(SPlus0),
    //Char+
    IdPlus1(IdPlus1),
    StartS(StartS),
    StartT(StartT),
    StartId(StartId),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::S(s) => s.as_parse_tree_ref(),
            ParseTree::T(t) => t.as_parse_tree_ref(),
            ParseTree::Id(id) => id.as_parse_tree_ref(),
            ParseTree::SPlus0(s_plus_0) => s_plus_0.as_parse_tree_ref(),
            ParseTree::IdPlus1(id_plus_1) => id_plus_1.as_parse_tree_ref(),
            ParseTree::StartS(start_s) => start_s.as_parse_tree_ref(),
            ParseTree::StartT(start_t) => start_t.as_parse_tree_ref(),
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
    fn unwrap_t(self) -> T {
        match self {
            ParseTree::T(t) => t,
            _ => panic!(),
        }
    }
    fn unwrap_id(self) -> Id {
        match self {
            ParseTree::Id(id) => id,
            _ => panic!(),
        }
    }
    fn unwrap_s_plus_0(self) -> SPlus0 {
        match self {
            ParseTree::SPlus0(s_plus_0) => s_plus_0,
            _ => panic!(),
        }
    }
    fn unwrap_id_plus_1(self) -> IdPlus1 {
        match self {
            ParseTree::IdPlus1(id_plus_1) => id_plus_1,
            _ => panic!(),
        }
    }
    fn unwrap_start_s(self) -> StartS {
        match self {
            ParseTree::StartS(start_s) => start_s,
            _ => panic!(),
        }
    }
    fn unwrap_start_t(self) -> StartT {
        match self {
            ParseTree::StartT(start_t) => start_t,
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
    T(&'a T),
    Id(&'a Id),
    SPlus0(&'a SPlus0),
    IdPlus1(&'a IdPlus1),
    StartS(&'a StartS),
    StartT(&'a StartT),
    StartId(&'a StartId),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::S(s) => (0..s.child_count()).filter_map(|i| s.child(i)).collect(),
            ParseTreeRef::T(t) => (0..t.child_count()).filter_map(|i| t.child(i)).collect(),
            ParseTreeRef::Id(id) => (0..id.child_count()).filter_map(|i| id.child(i)).collect(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.iter().collect(),
            ParseTreeRef::IdPlus1(id_plus_1) => id_plus_1.iter().collect(),
            ParseTreeRef::StartS(start_s) => (0..start_s.child_count())
                .filter_map(|i| start_s.child(i))
                .collect(),
            ParseTreeRef::StartT(start_t) => (0..start_t.child_count())
                .filter_map(|i| start_t.child(i))
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
            ParseTreeRef::T(_) => "T",
            ParseTreeRef::Id(_) => "Id",
            ParseTreeRef::SPlus0(_) => "Id+",
            ParseTreeRef::IdPlus1(_) => "Char+",
            ParseTreeRef::StartS(_) => "StartS",
            ParseTreeRef::StartT(_) => "StartT",
            ParseTreeRef::StartId(_) => "StartId",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::S(s) => s.child_count(),
            ParseTreeRef::T(t) => t.child_count(),
            ParseTreeRef::Id(id) => id.child_count(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.child_count(),
            ParseTreeRef::IdPlus1(id_plus_1) => id_plus_1.child_count(),
            ParseTreeRef::StartS(start_s) => start_s.child_count(),
            ParseTreeRef::StartT(start_t) => start_t.child_count(),
            ParseTreeRef::StartId(start_id) => start_id.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::S(s) => s.span(),
            ParseTreeRef::T(t) => t.span(),
            ParseTreeRef::Id(id) => id.span(),
            ParseTreeRef::SPlus0(s_plus_0) => s_plus_0.span(),
            ParseTreeRef::IdPlus1(id_plus_1) => id_plus_1.span(),
            ParseTreeRef::StartS(start_s) => start_s.span(),
            ParseTreeRef::StartT(start_t) => start_t.span(),
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
impl From<T> for ParseTree {
    fn from(t: T) -> Self {
        ParseTree::T(t)
    }
}
impl From<Id> for ParseTree {
    fn from(id: Id) -> Self {
        ParseTree::Id(id)
    }
}
impl From<SPlus0> for ParseTree {
    fn from(s_plus_0: SPlus0) -> Self {
        ParseTree::SPlus0(s_plus_0)
    }
}
impl From<IdPlus1> for ParseTree {
    fn from(id_plus_1: IdPlus1) -> Self {
        ParseTree::IdPlus1(id_plus_1)
    }
}
impl From<StartS> for ParseTree {
    fn from(start_s: StartS) -> Self {
        ParseTree::StartS(start_s)
    }
}
impl From<StartT> for ParseTree {
    fn from(start_t: StartT) -> Self {
        ParseTree::StartT(start_t)
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
//S = Id+
#[derive(Debug)]
pub struct S {
    pub ids: SPlus0,
    pub span: Span,
}
//T = Char !>> Char
#[derive(Debug)]
pub struct T {
    pub char: Token,
    pub span: Span,
}
//Id = Char+ !>> Char
#[derive(Debug)]
pub struct Id {
    pub chars: IdPlus1,
    pub span: Span,
}
//Id+
#[derive(Debug)]
pub enum SPlus0 {
    //Id+ Layout Id
    Alt0 {
        ids: Box<SPlus0>,
        layout: Token,
        id_2: Box<Id>,
        span: Span,
    },
    //Id
    Alt1 {
        id: Box<Id>,
        span: Span,
    },
}
//Char+
#[derive(Debug)]
pub enum IdPlus1 {
    //Char+ Char
    Alt0 {
        chars: Box<IdPlus1>,
        char_1: Token,
        span: Span,
    },
    //Char
    Alt1 {
        char: Token,
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
//StartT = Layout start:T Layout
#[derive(Debug)]
pub struct StartT {
    pub layout_0: Token,
    pub start: T,
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
            0 => Some(self.ids.as_parse_tree_ref()),
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
impl T {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.char.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::T(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl Id {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.chars.as_parse_tree_ref()),
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
impl SPlus0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SPlus0::Alt0 {
                ids, layout, id_2, ..
            } => match index {
                0 => Some(ids.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(id_2.as_parse_tree_ref()),
                _ => None,
            },
            SPlus0::Alt1 { id, .. } => match index {
                0 => Some(id.as_parse_tree_ref()),
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
    pub fn ids(&self) -> impl Iterator<Item = &Id> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Id(r) => Some(r),
            _ => None,
        })
    }
}
impl IdPlus1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            IdPlus1::Alt0 { chars, char_1, .. } => match index {
                0 => Some(chars.as_parse_tree_ref()),
                1 => Some(char_1.as_parse_tree_ref()),
                _ => None,
            },
            IdPlus1::Alt1 { char, .. } => match index {
                0 => Some(char.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            IdPlus1::Alt0 { .. } => 2usize,
            IdPlus1::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::IdPlus1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            IdPlus1::Alt0 { span, .. } => *span,
            IdPlus1::Alt1 { span, .. } => *span,
        }
    }
    pub fn chars(&self) -> impl Iterator<Item = &Token> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Token(r) => Some(r),
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
impl StartT {
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
        ParseTreeRef::StartT(self)
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
impl<'a> ListNode<'a> for SPlus0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                SPlus0::Alt0 {
                    ids: rest,
                    layout: layout,
                    id_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                SPlus0::Alt1 { id: item, .. } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for IdPlus1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                IdPlus1::Alt0 {
                    chars: rest,
                    char_1: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    current = rest;
                }
                IdPlus1::Alt1 { char: item, .. } => {
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
        //Char
        TerminalId(0) => TokenKind::T0,
        //WS
        TerminalId(1) => TokenKind::T1,
        //Layout
        TerminalId(2) => TokenKind::T2,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct FollowRestrictionParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for FollowRestrictionParseTreeBuilder {
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
                    //S : Id+.
                    SlotId(1) => {
                        let [ids] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        S {
                            ids: ids.unwrap_s_plus_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //T
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //T : Char !>> Char.
                    SlotId(3) => {
                        let [char] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        T {
                            char: char.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Id
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //Id : Char+ !>> Char.
                    SlotId(5) => {
                        let [chars] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Id {
                            chars: chars.unwrap_id_plus_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //S_Plus_0
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //Id+ : Id+ Layout Id.
                    SlotId(9) => {
                        let [ids, layout, id_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        SPlus0::Alt0 {
                            ids: Box::new(ids.unwrap_s_plus_0()),
                            layout: layout.unwrap_token(),
                            id_2: Box::new(id_2.unwrap_id()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Id+ : Id.
                    SlotId(11) => {
                        let [id] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SPlus0::Alt1 {
                            id: Box::new(id.unwrap_id()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Id_Plus_1
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //Char+ : Char+ Char.
                    SlotId(14) => {
                        let [chars, char_1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        IdPlus1::Alt0 {
                            chars: Box::new(chars.unwrap_id_plus_1()),
                            char_1: char_1.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Char+ : Char.
                    SlotId(16) => {
                        let [char] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        IdPlus1::Alt1 {
                            char: char.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartS
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //StartS : Layout start:S Layout.
                    SlotId(20) => {
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
            //StartT
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //StartT : Layout start:T Layout.
                    SlotId(24) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartT {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_t(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartId
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //StartId : Layout start:Id Layout.
                    SlotId(28) => {
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
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> ParseTree {
    match name {
        "S" => ParseTree::S(create_parse_tree_s(root_id, parser, builder)),
        "T" => ParseTree::T(create_parse_tree_t(root_id, parser, builder)),
        "Id" => ParseTree::Id(create_parse_tree_id(root_id, parser, builder)),
        "S_Plus_0" => ParseTree::SPlus0(create_parse_tree_s_plus_0(root_id, parser, builder)),
        "Id_Plus_1" => ParseTree::IdPlus1(create_parse_tree_id_plus_1(root_id, parser, builder)),
        "StartS" => ParseTree::StartS(create_parse_tree_start_s(root_id, parser, builder)),
        "StartT" => ParseTree::StartT(create_parse_tree_start_t(root_id, parser, builder)),
        "StartId" => ParseTree::StartId(create_parse_tree_start_id(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_s(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> S {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_s()
}
pub fn create_parse_tree_t(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> T {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_t()
}
pub fn create_parse_tree_id(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> Id {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_id()
}
pub fn create_parse_tree_s_plus_0(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> SPlus0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_s_plus_0()
}
pub fn create_parse_tree_id_plus_1(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> IdPlus1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_id_plus_1()
}
pub fn create_parse_tree_start_s(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> StartS {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_s()
}
pub fn create_parse_tree_start_t(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
) -> StartT {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_t()
}
pub fn create_parse_tree_start_id(
    root_id: SPPFNodeId,
    parser: &FollowRestrictionParser,
    builder: &FollowRestrictionParseTreeBuilder,
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

