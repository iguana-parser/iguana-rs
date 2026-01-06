use crate::parser::IggyParser;
use core::fmt;
use iguana::{
    ids::{NonterminalId, SlotId, TerminalId},
    parse_tree::{OneOrMany, ParseTreeBuilder, visit_sppf},
    parser::Parser,
    sppf::{NonterminalNode, SPPFNodeId, Span, TerminalNode},
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
    //";"
    T3,
    //":"
    T4,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Identifier",
            TokenKind::T1 => "WS",
            TokenKind::T2 => "\"grammar\"",
            TokenKind::T3 => "\";\"",
            TokenKind::T4 => "\":\"",
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    Grammar(Grammar),
    Rule(Rule),
    GrammarPlus0(GrammarPlus0),
    RulePlus1(RulePlus1),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::Grammar(grammar) => grammar.as_parse_tree_ref(),
            ParseTree::Rule(rule) => rule.as_parse_tree_ref(),
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0.as_parse_tree_ref(),
            ParseTree::RulePlus1(rule_plus_1) => rule_plus_1.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
        }
    }
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
    fn unwrap_grammar_plus_0(self) -> GrammarPlus0 {
        match self {
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0,
            _ => panic!(),
        }
    }
    fn unwrap_rule_plus_1(self) -> RulePlus1 {
        match self {
            ParseTree::RulePlus1(rule_plus_1) => rule_plus_1,
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
    GrammarPlus0(&'a GrammarPlus0),
    RulePlus1(&'a RulePlus1),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::Grammar(grammar) => (0..grammar.child_count())
                .filter_map(|i| grammar.child(i))
                .collect(),
            ParseTreeRef::Rule(rule) => (0..rule.child_count())
                .filter_map(|i| rule.child(i))
                .collect(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0
                .iter()
                .map(|a| a.as_parse_tree_ref())
                .collect(),
            ParseTreeRef::RulePlus1(rule_plus_1) => {
                rule_plus_1.iter().map(|a| a.as_parse_tree_ref()).collect()
            }
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            ParseTreeRef::Grammar(_) => "Grammar",
            ParseTreeRef::Rule(_) => "Rule",
            ParseTreeRef::GrammarPlus0(_) => "Grammar_Plus_0",
            ParseTreeRef::RulePlus1(_) => "Rule_Plus_1",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.child_count(),
            ParseTreeRef::Rule(rule) => rule.child_count(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.child_count(),
            ParseTreeRef::RulePlus1(rule_plus_1) => rule_plus_1.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.span(),
            ParseTreeRef::Rule(rule) => rule.span(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.span(),
            ParseTreeRef::RulePlus1(rule_plus_1) => rule_plus_1.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
    }
}
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
impl From<GrammarPlus0> for ParseTree {
    fn from(grammar_plus_0: GrammarPlus0) -> Self {
        ParseTree::GrammarPlus0(grammar_plus_0)
    }
}
impl From<RulePlus1> for ParseTree {
    fn from(rule_plus_1: RulePlus1) -> Self {
        ParseTree::RulePlus1(rule_plus_1)
    }
}
trait ListNode {
    type Item;
    fn iter(&self) -> impl Iterator<Item = &Self::Item>;
}
#[derive(Debug)]
pub struct Grammar(Token, Token, Token, GrammarPlus0, Span);
#[derive(Debug)]
pub struct Rule(Token, Token, RulePlus1, Token, Span);
#[derive(Debug)]
pub enum GrammarPlus0 {
    Alt0(Box<GrammarPlus0>, Rule, Span),
    Alt1(Rule, Span),
}
#[derive(Debug)]
pub enum RulePlus1 {
    Alt0(Box<RulePlus1>, Token, Span),
    Alt1(Token, Span),
}
impl Grammar {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            1 => Some(self.1.as_parse_tree_ref()),
            2 => Some(self.2.as_parse_tree_ref()),
            3 => Some(self.3.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        4usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Grammar(self)
    }
    pub fn span(&self) -> Span {
        self.4
    }
}
impl Rule {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            1 => Some(self.1.as_parse_tree_ref()),
            2 => Some(self.2.as_parse_tree_ref()),
            3 => Some(self.3.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        4usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Rule(self)
    }
    pub fn span(&self) -> Span {
        self.4
    }
}
impl GrammarPlus0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarPlus0::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            GrammarPlus0::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            GrammarPlus0::Alt0(..) => 2usize,
            GrammarPlus0::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::GrammarPlus0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            GrammarPlus0::Alt0(.., span) => *span,
            GrammarPlus0::Alt1(.., span) => *span,
        }
    }
}
impl RulePlus1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RulePlus1::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            RulePlus1::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RulePlus1::Alt0(..) => 2usize,
            RulePlus1::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RulePlus1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RulePlus1::Alt0(.., span) => *span,
            RulePlus1::Alt1(.., span) => *span,
        }
    }
}
impl ListNode for GrammarPlus0 {
    type Item = Rule;
    fn iter(&self) -> impl Iterator<Item = &Rule> {
        let mut items = Vec::new();
        let mut current = self;
        loop {
            match current {
                GrammarPlus0::Alt0(rest, item, _) => {
                    items.push(item);
                    current = rest;
                }
                GrammarPlus0::Alt1(item, _) => {
                    items.push(item);
                    break;
                }
            }
        }
        items.into_iter().rev()
    }
}
impl ListNode for RulePlus1 {
    type Item = Token;
    fn iter(&self) -> impl Iterator<Item = &Token> {
        let mut items = Vec::new();
        let mut current = self;
        loop {
            match current {
                RulePlus1::Alt0(rest, item, _) => {
                    items.push(item);
                    current = rest;
                }
                RulePlus1::Alt1(item, _) => {
                    items.push(item);
                    break;
                }
            }
        }
        items.into_iter().rev()
    }
}
#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
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
        //WS
        TerminalId(1) => TokenKind::T1,
        //"grammar"
        TerminalId(2) => TokenKind::T2,
        //";"
        TerminalId(3) => TokenKind::T3,
        //":"
        TerminalId(4) => TokenKind::T4,
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
                    //Grammar : "grammar" Identifier ";" Rule+.
                    SlotId(4) => {
                        let [c0, c1, c2, c3] = <[ParseTree; 4usize]>::try_from(children).unwrap();
                        Grammar(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_token(),
                            c3.unwrap_grammar_plus_0(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //Rule : Identifier ":" Identifier+ ";".
                    SlotId(9) => {
                        let [c0, c1, c2, c3] = <[ParseTree; 4usize]>::try_from(children).unwrap();
                        Rule(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_rule_plus_1(),
                            c3.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Plus_0
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //Rule+ : Rule+ Rule.
                    SlotId(12) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt0(
                            Box::new(c0.unwrap_grammar_plus_0()),
                            c1.unwrap_rule(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Rule+ : Rule.
                    SlotId(14) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt1(c0.unwrap_rule(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Plus_1
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //Identifier+ : Identifier+ Identifier.
                    SlotId(17) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        RulePlus1::Alt0(
                            Box::new(c0.unwrap_rule_plus_1()),
                            c1.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Identifier+ : Identifier.
                    SlotId(19) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RulePlus1::Alt1(c0.unwrap_token(), nonterminal_node.span).into()
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
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> ParseTree {
    match name {
        "Grammar" => ParseTree::Grammar(create_parse_tree_grammar(root_id, parser, builder)),
        "Rule" => ParseTree::Rule(create_parse_tree_rule(root_id, parser, builder)),
        "Grammar_Plus_0" => {
            ParseTree::GrammarPlus0(create_parse_tree_grammar_plus_0(root_id, parser, builder))
        }
        "Rule_Plus_1" => {
            ParseTree::RulePlus1(create_parse_tree_rule_plus_1(root_id, parser, builder))
        }
        _ => panic!(),
    }
}
pub fn create_parse_tree_grammar(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Grammar {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_grammar()
}
pub fn create_parse_tree_rule(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Rule {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder).unwrap_one().unwrap_rule()
}
pub fn create_parse_tree_grammar_plus_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> GrammarPlus0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_grammar_plus_0()
}
pub fn create_parse_tree_rule_plus_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RulePlus1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_rule_plus_1()
}
pub fn to_sexpr(node: ParseTreeRef<'_>) -> String {
    let mut s = String::new();
    node_to_sexpr(node, 0, &mut s).expect("error");
    s
}
fn node_to_sexpr(node: ParseTreeRef<'_>, indent: usize, w: &mut impl Write) -> fmt::Result {
    let children = node.children();
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
        { "id" : my_id, "kind" : kind, "label" : node.name(), "start" : span
        .left_extent, "end" : span.right_extent }
    ));
    for child in node.children() {
        let child_id = build_json_graph(child, nodes, edges, next_id);
        edges.push(serde_json::json!({ "src" : my_id, "dest" : child_id }));
    }
    my_id
}

