use crate::parser::IggyParser;
use core::fmt;
use iguana::trace::TraceEvent;
use iguana::{
    parse_tree::{OneOrMany, ParseTreeBuilder, visit_sppf},
    parser::{NonterminalId, Parser, SlotId, TerminalId},
    sppf::{NonterminalNode, SPPFNodeId},
};
use std::fmt::Write;
#[derive(Debug)]
enum TokenKind {
    //Identifier
    T0,
    //WS
    T1,
    //"grammar"
    T2,
    //":"
    T3,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Identifier",
            TokenKind::T1 => "WS",
            TokenKind::T2 => "\"grammar\"",
            TokenKind::T3 => "\":\"",
        }
    }
}
#[derive(Debug)]
enum ParseTree {
    Grammar(Grammar),
    Rule(Rule),
    Grammar_Plus0(Grammar_Plus0),
    Rule_Plus1(Rule_Plus1),
    Token(Token),
}
impl ParseTree {
    fn unwrap_grammar(self) -> Grammar {
        match self {
            ParseTree::Grammar(grammar) => grammar,
            _ => panic!(),
        }
    }
    fn unwrap_rule(self) -> Rule {
        match self {
            ParseTree::Rule(rule) => rule,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_Plus0(self) -> Grammar_Plus0 {
        match self {
            ParseTree::Grammar_Plus0(grammar_Plus0) => grammar_Plus0,
            _ => panic!(),
        }
    }
    fn unwrap_rule_Plus1(self) -> Rule_Plus1 {
        match self {
            ParseTree::Rule_Plus1(rule_Plus1) => rule_Plus1,
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
    Grammar(&'a Grammar),
    Rule(&'a Rule),
    Grammar_Plus0(&'a Grammar_Plus0),
    Rule_Plus1(&'a Rule_Plus1),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> ChildIter<'a> {
        ChildIter {
            node: *self,
            index: 0,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            ParseTreeRef::Grammar(_) => "Grammar",
            ParseTreeRef::Rule(_) => "Rule",
            ParseTreeRef::Grammar_Plus0(_) => "Grammar_Plus0",
            ParseTreeRef::Rule_Plus1(_) => "Rule_Plus1",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.child_count(),
            ParseTreeRef::Rule(rule) => rule.child_count(),
            ParseTreeRef::Grammar_Plus0(grammar_Plus0) => grammar_Plus0.child_count(),
            ParseTreeRef::Rule_Plus1(rule_Plus1) => rule_Plus1.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
}
pub struct ChildIter<'a> {
    node: ParseTreeRef<'a>,
    index: usize,
}
impl<'a> Iterator for ChildIter<'a> {
    type Item = ParseTreeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let child = match self.node {
            ParseTreeRef::Grammar(grammar) => grammar.child(self.index),
            ParseTreeRef::Rule(rule) => rule.child(self.index),
            ParseTreeRef::Grammar_Plus0(grammar_Plus0) => grammar_Plus0.child(self.index),
            ParseTreeRef::Rule_Plus1(rule_Plus1) => rule_Plus1.child(self.index),
            ParseTreeRef::Token(_) => None,
        };
        self.index += 1;
        child
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.node.child_count().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}
impl<'a> ExactSizeIterator for ChildIter<'a> {}
impl From<Grammar> for ParseTree {
    fn from(grammar: Grammar) -> Self {
        ParseTree::Grammar(grammar)
    }
}
impl From<Rule> for ParseTree {
    fn from(rule: Rule) -> Self {
        ParseTree::Rule(rule)
    }
}
impl From<Grammar_Plus0> for ParseTree {
    fn from(grammar_Plus0: Grammar_Plus0) -> Self {
        ParseTree::Grammar_Plus0(grammar_Plus0)
    }
}
impl From<Rule_Plus1> for ParseTree {
    fn from(rule_Plus1: Rule_Plus1) -> Self {
        ParseTree::Rule_Plus1(rule_Plus1)
    }
}
#[derive(Debug)]
pub struct Grammar(Token, Token, Grammar_Plus0);
#[derive(Debug)]
pub struct Rule(Token, Token, Rule_Plus1);
#[derive(Debug)]
pub enum Grammar_Plus0 {
    Alt0(Box<Grammar_Plus0>, Rule),
    Alt1(Rule),
}
#[derive(Debug)]
pub enum Rule_Plus1 {
    Alt0(Box<Rule_Plus1>, Token),
    Alt1(Token),
}
impl Grammar {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            1 => Some(self.1.as_parse_tree_ref()),
            2 => Some(self.2.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Grammar(self)
    }
}
impl Rule {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            1 => Some(self.1.as_parse_tree_ref()),
            2 => Some(self.2.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Rule(self)
    }
}
impl Grammar_Plus0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Grammar_Plus0::Alt0(c0, c1) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            Grammar_Plus0::Alt1(c0) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Grammar_Plus0::Alt0(..) => 2usize,
            Grammar_Plus0::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Grammar_Plus0(self)
    }
}
impl Rule_Plus1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Rule_Plus1::Alt0(c0, c1) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            Rule_Plus1::Alt1(c0) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Rule_Plus1::Alt0(..) => 2usize,
            Rule_Plus1::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Rule_Plus1(self)
    }
}
#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
}
impl Token {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Token(self)
    }
}
fn token_kind(terminal_id: TerminalId) -> TokenKind {
    match terminal_id {
        //Identifier
        TerminalId(0) => TokenKind::T0,
        //WS
        TerminalId(1) => TokenKind::T1,
        //"grammar"
        TerminalId(2) => TokenKind::T2,
        //":"
        TerminalId(3) => TokenKind::T3,
        _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
    }
}
pub struct IggyParseTreeBuilder;
impl ParseTreeBuilder<ParseTree> for IggyParseTreeBuilder {
    fn new_nonterminal_node(
        &self,
        nonterminal_node: &NonterminalNode,
        children: OneOrMany<ParseTree>,
    ) -> ParseTree {
        let children = children.into_vec();
        match nonterminal_node.nonterminal_id {
            //Grammar
            NonterminalId(0) => {
                match nonterminal_node.return_slot {
                    //Grammar : "grammar" Identifier Grammar_Plus0.
                    SlotId(3) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Grammar(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_grammar_Plus0(),
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //Rule : Identifier ":" Rule_Plus1.
                    SlotId(7) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Rule(c0.unwrap_token(), c1.unwrap_token(), c2.unwrap_rule_Plus1()).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Plus0
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //Grammar_Plus0 : Grammar_Plus0 Rule.
                    SlotId(10) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        Grammar_Plus0::Alt0(Box::new(c0.unwrap_grammar_Plus0()), c1.unwrap_rule())
                            .into()
                    }
                    //Grammar_Plus0 : Rule.
                    SlotId(12) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Grammar_Plus0::Alt1(c0.unwrap_rule()).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Plus1
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //Rule_Plus1 : Rule_Plus1 Identifier.
                    SlotId(15) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        Rule_Plus1::Alt0(Box::new(c0.unwrap_rule_Plus1()), c1.unwrap_token()).into()
                    }
                    //Rule_Plus1 : Identifier.
                    SlotId(17) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Rule_Plus1::Alt1(c0.unwrap_token()).into()
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
    fn new_token(&self, terminal_id: TerminalId) -> ParseTree {
        ParseTree::Token(Token {
            kind: token_kind(terminal_id),
        })
    }
}
pub fn create_parse_tree(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Grammar {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_grammar()
}
pub fn to_sexpr(node: ParseTreeRef<'_>) -> String {
    let mut s = String::new();
    node_to_sexpr(node, 0, &mut s).expect("error");
    s
}
fn node_to_sexpr(node: ParseTreeRef<'_>, indent: usize, w: &mut impl Write) -> fmt::Result {
    let children: Vec<_> = node.children().collect();
    if children.is_empty() {
        writeln!(w, "{:indent$}{}", "", node.name())
    } else {
        writeln!(w, "{:indent$}({}", "", node.name())?;
        for child in children {
            node_to_sexpr(child, indent + 2, w)?;
        }
        writeln!(w, "{:indent$})", "")
    }
}

