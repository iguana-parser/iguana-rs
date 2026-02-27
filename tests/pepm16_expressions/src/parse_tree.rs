use crate::parser::Pepm16ExpressionsParser;
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
    //WS
    T0,
    //"."
    T1,
    //"f"
    T2,
    //"*"
    T3,
    //"+"
    T4,
    //"-"
    T5,
    //"if"
    T6,
    //"then"
    T7,
    //"else"
    T8,
    //";"
    T9,
    //"("
    T10,
    //")"
    T11,
    //"a"
    T12,
    //Layout
    T13,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "WS",
            TokenKind::T1 => "\".\"",
            TokenKind::T2 => "\"f\"",
            TokenKind::T3 => "\"*\"",
            TokenKind::T4 => "\"+\"",
            TokenKind::T5 => "\"-\"",
            TokenKind::T6 => "\"if\"",
            TokenKind::T7 => "\"then\"",
            TokenKind::T8 => "\"else\"",
            TokenKind::T9 => "\";\"",
            TokenKind::T10 => "\"(\"",
            TokenKind::T11 => "\")\"",
            TokenKind::T12 => "\"a\"",
            TokenKind::T13 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    S(S),
    E(E),
    StartS(StartS),
    StartE(StartE),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::S(s) => s.as_parse_tree_ref(),
            ParseTree::E(e) => e.as_parse_tree_ref(),
            ParseTree::StartS(start_s) => start_s.as_parse_tree_ref(),
            ParseTree::StartE(start_e) => start_e.as_parse_tree_ref(),
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
    StartS(&'a StartS),
    StartE(&'a StartE),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::S(s) => (0..s.child_count()).filter_map(|i| s.child(i)).collect(),
            ParseTreeRef::E(e) => (0..e.child_count()).filter_map(|i| e.child(i)).collect(),
            ParseTreeRef::StartS(start_s) => (0..start_s.child_count())
                .filter_map(|i| start_s.child(i))
                .collect(),
            ParseTreeRef::StartE(start_e) => (0..start_e.child_count())
                .filter_map(|i| start_e.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::S(_) => "S",
            ParseTreeRef::E(_) => "E",
            ParseTreeRef::StartS(_) => "StartS",
            ParseTreeRef::StartE(_) => "StartE",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::S(s) => s.child_count(),
            ParseTreeRef::E(e) => e.child_count(),
            ParseTreeRef::StartS(start_s) => start_s.child_count(),
            ParseTreeRef::StartE(start_e) => start_e.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::S(s) => s.span(),
            ParseTreeRef::E(e) => e.span(),
            ParseTreeRef::StartS(start_s) => start_s.span(),
            ParseTreeRef::StartE(start_e) => start_e.span(),
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
    //[6 >= p] l=E(p) [l == 0 || l >= 6] Layout "." Layout "f" return 0
    Alt0 {
        e: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    //[6 >= p] l=E(p) [l == 0 || l >= 6] Layout r=E(6) return r == 0 ? 6 : min(r, 6)
    Alt1 {
        e_0: Box<E>,
        layout: Token,
        e_2: Box<E>,
        span: Span,
    },
    //[5 >= p] l=E(p) [l == 0 || l >= 5] Layout "*" Layout r=E(6) return r == 0 ? 5 : min(r, 5)
    Alt2 {
        e_0: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        e_4: Box<E>,
        span: Span,
    },
    //[4 >= p] l=E(p) [l == 0 || l >= 4] Layout "+" Layout r=E(5) return r == 0 ? 4 : min(r, 4)
    Alt3 {
        e_0: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        e_4: Box<E>,
        span: Span,
    },
    //[4 >= p] l=E(p) [l == 0 || l >= 4] Layout "-" Layout r=E(5) return r == 0 ? 4 : min(r, 4)
    Alt4 {
        e_0: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        e_4: Box<E>,
        span: Span,
    },
    //"-" Layout r=E(3) return r == 0 ? 3 : min(r, 3)
    Alt5 {
        lit_0: Token,
        layout: Token,
        e: Box<E>,
        span: Span,
    },
    //"if" Layout E(0) Layout "then" Layout E(0) Layout "else" Layout E(2) return 2
    Alt6 {
        lit_0: Token,
        layout_1: Token,
        e_2: Box<E>,
        layout_3: Token,
        lit_4: Token,
        layout_5: Token,
        e_6: Box<E>,
        layout_7: Token,
        lit_8: Token,
        layout_9: Token,
        e_10: Box<E>,
        span: Span,
    },
    //[1 >= p] l=E(p) [l == 0 || l >= 2] Layout ";" Layout E(1) return 1
    Alt7 {
        e_0: Box<E>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        e_4: Box<E>,
        span: Span,
    },
    //"(" Layout E(0) Layout ")" return 0
    Alt8 {
        lit_0: Token,
        layout_1: Token,
        e: Box<E>,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    //"a" return 0
    Alt9 {
        lit_0: Token,
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
//StartE = Layout start:E(0) Layout
#[derive(Debug)]
pub struct StartE {
    pub layout_0: Token,
    pub start: E,
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
                e,
                layout_1,
                lit_2,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(e.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            E::Alt1 {
                e_0, layout, e_2, ..
            } => match index {
                0 => Some(e_0.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(e_2.as_parse_tree_ref()),
                _ => None,
            },
            E::Alt2 {
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
            E::Alt3 {
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
            E::Alt4 {
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
            E::Alt5 {
                lit_0, layout, e, ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(e.as_parse_tree_ref()),
                _ => None,
            },
            E::Alt6 {
                lit_0,
                layout_1,
                e_2,
                layout_3,
                lit_4,
                layout_5,
                e_6,
                layout_7,
                lit_8,
                layout_9,
                e_10,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(e_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(e_6.as_parse_tree_ref()),
                7 => Some(layout_7.as_parse_tree_ref()),
                8 => Some(lit_8.as_parse_tree_ref()),
                9 => Some(layout_9.as_parse_tree_ref()),
                10 => Some(e_10.as_parse_tree_ref()),
                _ => None,
            },
            E::Alt7 {
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
            E::Alt8 {
                lit_0,
                layout_1,
                e,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(e.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            E::Alt9 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            E::Alt0 { .. } => 5usize,
            E::Alt1 { .. } => 3usize,
            E::Alt2 { .. } => 5usize,
            E::Alt3 { .. } => 5usize,
            E::Alt4 { .. } => 5usize,
            E::Alt5 { .. } => 3usize,
            E::Alt6 { .. } => 11usize,
            E::Alt7 { .. } => 5usize,
            E::Alt8 { .. } => 5usize,
            E::Alt9 { .. } => 1usize,
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
            E::Alt3 { span, .. } => *span,
            E::Alt4 { span, .. } => *span,
            E::Alt5 { span, .. } => *span,
            E::Alt6 { span, .. } => *span,
            E::Alt7 { span, .. } => *span,
            E::Alt8 { span, .. } => *span,
            E::Alt9 { span, .. } => *span,
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
        //WS
        TerminalId(0) => TokenKind::T0,
        //"."
        TerminalId(1) => TokenKind::T1,
        //"f"
        TerminalId(2) => TokenKind::T2,
        //"*"
        TerminalId(3) => TokenKind::T3,
        //"+"
        TerminalId(4) => TokenKind::T4,
        //"-"
        TerminalId(5) => TokenKind::T5,
        //"if"
        TerminalId(6) => TokenKind::T6,
        //"then"
        TerminalId(7) => TokenKind::T7,
        //"else"
        TerminalId(8) => TokenKind::T8,
        //";"
        TerminalId(9) => TokenKind::T9,
        //"("
        TerminalId(10) => TokenKind::T10,
        //")"
        TerminalId(11) => TokenKind::T11,
        //"a"
        TerminalId(12) => TokenKind::T12,
        //Layout
        TerminalId(13) => TokenKind::T13,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct Pepm16ExpressionsParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for Pepm16ExpressionsParseTreeBuilder {
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
            //StartS
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //StartS : Layout start:S Layout.
                    SlotId(85) => {
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
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //StartE : Layout start:E(0) Layout.
                    SlotId(89) => {
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
            //E
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //E : [6 >= p] l=E(p) [l == 0 || l >= 6] Layout "." Layout "f" return 0.
                    SlotId(10) => {
                        let [e, layout_1, lit_2, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Alt0 {
                            e: Box::new(e.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : [6 >= p] l=E(p) [l == 0 || l >= 6] Layout r=E(6) return r == 0 ? 6 : min(r, 6).
                    SlotId(17) => {
                        let [e_0, layout, e_2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        E::Alt1 {
                            e_0: Box::new(e_0.unwrap_e()),
                            layout: layout.unwrap_token(),
                            e_2: Box::new(e_2.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : [5 >= p] l=E(p) [l == 0 || l >= 5] Layout "*" Layout r=E(6) return r == 0 ? 5 : min(r, 5).
                    SlotId(26) => {
                        let [e_0, layout_1, lit_2, layout_3, e_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Alt2 {
                            e_0: Box::new(e_0.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            e_4: Box::new(e_4.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : [4 >= p] l=E(p) [l == 0 || l >= 4] Layout "+" Layout r=E(5) return r == 0 ? 4 : min(r, 4).
                    SlotId(35) => {
                        let [e_0, layout_1, lit_2, layout_3, e_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Alt3 {
                            e_0: Box::new(e_0.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            e_4: Box::new(e_4.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : [4 >= p] l=E(p) [l == 0 || l >= 4] Layout "-" Layout r=E(5) return r == 0 ? 4 : min(r, 4).
                    SlotId(44) => {
                        let [e_0, layout_1, lit_2, layout_3, e_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Alt4 {
                            e_0: Box::new(e_0.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            e_4: Box::new(e_4.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : "-" Layout r=E(3) return r == 0 ? 3 : min(r, 3).
                    SlotId(49) => {
                        let [lit_0, layout, e] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        E::Alt5 {
                            lit_0: lit_0.unwrap_token(),
                            layout: layout.unwrap_token(),
                            e: Box::new(e.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : "if" Layout E(0) Layout "then" Layout E(0) Layout "else" Layout E(2) return 2.
                    SlotId(62) => {
                        let [
                            lit_0,
                            layout_1,
                            e_2,
                            layout_3,
                            lit_4,
                            layout_5,
                            e_6,
                            layout_7,
                            lit_8,
                            layout_9,
                            e_10,
                        ] = <[ParseTree; 11usize]>::try_from(children).unwrap();
                        E::Alt6 {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            e_2: Box::new(e_2.unwrap_e()),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            layout_5: layout_5.unwrap_token(),
                            e_6: Box::new(e_6.unwrap_e()),
                            layout_7: layout_7.unwrap_token(),
                            lit_8: lit_8.unwrap_token(),
                            layout_9: layout_9.unwrap_token(),
                            e_10: Box::new(e_10.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : [1 >= p] l=E(p) [l == 0 || l >= 2] Layout ";" Layout E(1) return 1.
                    SlotId(71) => {
                        let [e_0, layout_1, lit_2, layout_3, e_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Alt7 {
                            e_0: Box::new(e_0.unwrap_e()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            e_4: Box::new(e_4.unwrap_e()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : "(" Layout E(0) Layout ")" return 0.
                    SlotId(78) => {
                        let [lit_0, layout_1, e, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        E::Alt8 {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            e: Box::new(e.unwrap_e()),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //E : "a" return 0.
                    SlotId(81) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        E::Alt9 {
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
    parser: &Pepm16ExpressionsParser,
    builder: &Pepm16ExpressionsParseTreeBuilder,
) -> ParseTree {
    match name {
        "S" => ParseTree::S(create_parse_tree_s(root_id, parser, builder)),
        "E" => ParseTree::E(create_parse_tree_e(root_id, parser, builder)),
        "StartS" => ParseTree::StartS(create_parse_tree_start_s(root_id, parser, builder)),
        "StartE" => ParseTree::StartE(create_parse_tree_start_e(root_id, parser, builder)),
        _ => panic!(),
    }
}
pub fn create_parse_tree_s(
    root_id: SPPFNodeId,
    parser: &Pepm16ExpressionsParser,
    builder: &Pepm16ExpressionsParseTreeBuilder,
) -> S {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_s()
}
pub fn create_parse_tree_e(
    root_id: SPPFNodeId,
    parser: &Pepm16ExpressionsParser,
    builder: &Pepm16ExpressionsParseTreeBuilder,
) -> E {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_e()
}
pub fn create_parse_tree_start_s(
    root_id: SPPFNodeId,
    parser: &Pepm16ExpressionsParser,
    builder: &Pepm16ExpressionsParseTreeBuilder,
) -> StartS {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_s()
}
pub fn create_parse_tree_start_e(
    root_id: SPPFNodeId,
    parser: &Pepm16ExpressionsParser,
    builder: &Pepm16ExpressionsParseTreeBuilder,
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

