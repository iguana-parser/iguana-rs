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
    //">"
    T5,
    //"|"
    T6,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Identifier",
            TokenKind::T1 => "WS",
            TokenKind::T2 => "\"grammar\"",
            TokenKind::T3 => "\";\"",
            TokenKind::T4 => "\":\"",
            TokenKind::T5 => "\">\"",
            TokenKind::T6 => "\"|\"",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    //Grammar
    Grammar(Grammar),
    //Rule
    Rule(Rule),
    //PriorityLevel
    PriorityLevel(PriorityLevel),
    //Alternative
    Alternative(Alternative),
    //Symbol
    Symbol(Symbol),
    //Rule*
    GrammarStar0(GrammarStar0),
    //PriorityLevel?
    RuleOpt0(RuleOpt0),
    //(">" PriorityLevel)
    RuleGroup0(RuleGroup0),
    //(">" PriorityLevel)*
    RuleStar1(RuleStar1),
    //Alternative?
    PriorityLevelOpt1(PriorityLevelOpt1),
    //("|" Alternative)
    PriorityLevelGroup1(PriorityLevelGroup1),
    //("|" Alternative)*
    PriorityLevelStar2(PriorityLevelStar2),
    //Symbol*
    AlternativeStar3(AlternativeStar3),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::Grammar(grammar) => grammar.as_parse_tree_ref(),
            ParseTree::Rule(rule) => rule.as_parse_tree_ref(),
            ParseTree::PriorityLevel(prioritylevel) => prioritylevel.as_parse_tree_ref(),
            ParseTree::Alternative(alternative) => alternative.as_parse_tree_ref(),
            ParseTree::Symbol(symbol) => symbol.as_parse_tree_ref(),
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0.as_parse_tree_ref(),
            ParseTree::RuleOpt0(rule_opt_0) => rule_opt_0.as_parse_tree_ref(),
            ParseTree::RuleGroup0(rule_group_0) => rule_group_0.as_parse_tree_ref(),
            ParseTree::RuleStar1(rule_star_1) => rule_star_1.as_parse_tree_ref(),
            ParseTree::PriorityLevelOpt1(prioritylevel_opt_1) => {
                prioritylevel_opt_1.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelGroup1(prioritylevel_group_1) => {
                prioritylevel_group_1.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.as_parse_tree_ref()
            }
            ParseTree::AlternativeStar3(alternative_star_3) => {
                alternative_star_3.as_parse_tree_ref()
            }
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
    fn unwrap_prioritylevel(self) -> PriorityLevel {
        match self {
            ParseTree::PriorityLevel(prioritylevel) => prioritylevel,
            _ => panic!(),
        }
    }
    fn unwrap_alternative(self) -> Alternative {
        match self {
            ParseTree::Alternative(alternative) => alternative,
            _ => panic!(),
        }
    }
    fn unwrap_symbol(self) -> Symbol {
        match self {
            ParseTree::Symbol(symbol) => symbol,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_star_0(self) -> GrammarStar0 {
        match self {
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0,
            _ => panic!(),
        }
    }
    fn unwrap_rule_opt_0(self) -> RuleOpt0 {
        match self {
            ParseTree::RuleOpt0(rule_opt_0) => rule_opt_0,
            _ => panic!(),
        }
    }
    fn unwrap_rule_group_0(self) -> RuleGroup0 {
        match self {
            ParseTree::RuleGroup0(rule_group_0) => rule_group_0,
            _ => panic!(),
        }
    }
    fn unwrap_rule_star_1(self) -> RuleStar1 {
        match self {
            ParseTree::RuleStar1(rule_star_1) => rule_star_1,
            _ => panic!(),
        }
    }
    fn unwrap_prioritylevel_opt_1(self) -> PriorityLevelOpt1 {
        match self {
            ParseTree::PriorityLevelOpt1(prioritylevel_opt_1) => prioritylevel_opt_1,
            _ => panic!(),
        }
    }
    fn unwrap_prioritylevel_group_1(self) -> PriorityLevelGroup1 {
        match self {
            ParseTree::PriorityLevelGroup1(prioritylevel_group_1) => prioritylevel_group_1,
            _ => panic!(),
        }
    }
    fn unwrap_prioritylevel_star_2(self) -> PriorityLevelStar2 {
        match self {
            ParseTree::PriorityLevelStar2(prioritylevel_star_2) => prioritylevel_star_2,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_star_3(self) -> AlternativeStar3 {
        match self {
            ParseTree::AlternativeStar3(alternative_star_3) => alternative_star_3,
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
    PriorityLevel(&'a PriorityLevel),
    Alternative(&'a Alternative),
    Symbol(&'a Symbol),
    GrammarStar0(&'a GrammarStar0),
    RuleOpt0(&'a RuleOpt0),
    RuleGroup0(&'a RuleGroup0),
    RuleStar1(&'a RuleStar1),
    PriorityLevelOpt1(&'a PriorityLevelOpt1),
    PriorityLevelGroup1(&'a PriorityLevelGroup1),
    PriorityLevelStar2(&'a PriorityLevelStar2),
    AlternativeStar3(&'a AlternativeStar3),
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
            ParseTreeRef::PriorityLevel(prioritylevel) => (0..prioritylevel.child_count())
                .filter_map(|i| prioritylevel.child(i))
                .collect(),
            ParseTreeRef::Alternative(alternative) => (0..alternative.child_count())
                .filter_map(|i| alternative.child(i))
                .collect(),
            ParseTreeRef::Symbol(symbol) => (0..symbol.child_count())
                .filter_map(|i| symbol.child(i))
                .collect(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0
                .iter()
                .map(|a| a.as_parse_tree_ref())
                .collect(),
            ParseTreeRef::RuleOpt0(rule_opt_0) => (0..rule_opt_0.child_count())
                .filter_map(|i| rule_opt_0.child(i))
                .collect(),
            ParseTreeRef::RuleGroup0(rule_group_0) => (0..rule_group_0.child_count())
                .filter_map(|i| rule_group_0.child(i))
                .collect(),
            ParseTreeRef::RuleStar1(rule_star_1) => {
                rule_star_1.iter().map(|a| a.as_parse_tree_ref()).collect()
            }
            ParseTreeRef::PriorityLevelOpt1(prioritylevel_opt_1) => (0..prioritylevel_opt_1
                .child_count())
                .filter_map(|i| prioritylevel_opt_1.child(i))
                .collect(),
            ParseTreeRef::PriorityLevelGroup1(prioritylevel_group_1) => (0..prioritylevel_group_1
                .child_count())
                .filter_map(|i| prioritylevel_group_1.child(i))
                .collect(),
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => prioritylevel_star_2
                .iter()
                .map(|a| a.as_parse_tree_ref())
                .collect(),
            ParseTreeRef::AlternativeStar3(alternative_star_3) => alternative_star_3
                .iter()
                .map(|a| a.as_parse_tree_ref())
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::Grammar(_) => "Grammar",
            ParseTreeRef::Rule(_) => "Rule",
            ParseTreeRef::PriorityLevel(_) => "PriorityLevel",
            ParseTreeRef::Alternative(_) => "Alternative",
            ParseTreeRef::Symbol(_) => "Symbol",
            ParseTreeRef::GrammarStar0(_) => "Rule*",
            ParseTreeRef::RuleOpt0(_) => "PriorityLevel?",
            ParseTreeRef::RuleGroup0(_) => "(\">\" PriorityLevel)",
            ParseTreeRef::RuleStar1(_) => "(\">\" PriorityLevel)*",
            ParseTreeRef::PriorityLevelOpt1(_) => "Alternative?",
            ParseTreeRef::PriorityLevelGroup1(_) => "(\"|\" Alternative)",
            ParseTreeRef::PriorityLevelStar2(_) => "(\"|\" Alternative)*",
            ParseTreeRef::AlternativeStar3(_) => "Symbol*",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.child_count(),
            ParseTreeRef::Rule(rule) => rule.child_count(),
            ParseTreeRef::PriorityLevel(prioritylevel) => prioritylevel.child_count(),
            ParseTreeRef::Alternative(alternative) => alternative.child_count(),
            ParseTreeRef::Symbol(symbol) => symbol.child_count(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.child_count(),
            ParseTreeRef::RuleOpt0(rule_opt_0) => rule_opt_0.child_count(),
            ParseTreeRef::RuleGroup0(rule_group_0) => rule_group_0.child_count(),
            ParseTreeRef::RuleStar1(rule_star_1) => rule_star_1.child_count(),
            ParseTreeRef::PriorityLevelOpt1(prioritylevel_opt_1) => {
                prioritylevel_opt_1.child_count()
            }
            ParseTreeRef::PriorityLevelGroup1(prioritylevel_group_1) => {
                prioritylevel_group_1.child_count()
            }
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.child_count()
            }
            ParseTreeRef::AlternativeStar3(alternative_star_3) => alternative_star_3.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.span(),
            ParseTreeRef::Rule(rule) => rule.span(),
            ParseTreeRef::PriorityLevel(prioritylevel) => prioritylevel.span(),
            ParseTreeRef::Alternative(alternative) => alternative.span(),
            ParseTreeRef::Symbol(symbol) => symbol.span(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.span(),
            ParseTreeRef::RuleOpt0(rule_opt_0) => rule_opt_0.span(),
            ParseTreeRef::RuleGroup0(rule_group_0) => rule_group_0.span(),
            ParseTreeRef::RuleStar1(rule_star_1) => rule_star_1.span(),
            ParseTreeRef::PriorityLevelOpt1(prioritylevel_opt_1) => prioritylevel_opt_1.span(),
            ParseTreeRef::PriorityLevelGroup1(prioritylevel_group_1) => {
                prioritylevel_group_1.span()
            }
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => prioritylevel_star_2.span(),
            ParseTreeRef::AlternativeStar3(alternative_star_3) => alternative_star_3.span(),
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
impl From<PriorityLevel> for ParseTree {
    fn from(prioritylevel: PriorityLevel) -> Self {
        ParseTree::PriorityLevel(prioritylevel)
    }
}
impl From<Alternative> for ParseTree {
    fn from(alternative: Alternative) -> Self {
        ParseTree::Alternative(alternative)
    }
}
impl From<Symbol> for ParseTree {
    fn from(symbol: Symbol) -> Self {
        ParseTree::Symbol(symbol)
    }
}
impl From<GrammarStar0> for ParseTree {
    fn from(grammar_star_0: GrammarStar0) -> Self {
        ParseTree::GrammarStar0(grammar_star_0)
    }
}
impl From<RuleOpt0> for ParseTree {
    fn from(rule_opt_0: RuleOpt0) -> Self {
        ParseTree::RuleOpt0(rule_opt_0)
    }
}
impl From<RuleGroup0> for ParseTree {
    fn from(rule_group_0: RuleGroup0) -> Self {
        ParseTree::RuleGroup0(rule_group_0)
    }
}
impl From<RuleStar1> for ParseTree {
    fn from(rule_star_1: RuleStar1) -> Self {
        ParseTree::RuleStar1(rule_star_1)
    }
}
impl From<PriorityLevelOpt1> for ParseTree {
    fn from(prioritylevel_opt_1: PriorityLevelOpt1) -> Self {
        ParseTree::PriorityLevelOpt1(prioritylevel_opt_1)
    }
}
impl From<PriorityLevelGroup1> for ParseTree {
    fn from(prioritylevel_group_1: PriorityLevelGroup1) -> Self {
        ParseTree::PriorityLevelGroup1(prioritylevel_group_1)
    }
}
impl From<PriorityLevelStar2> for ParseTree {
    fn from(prioritylevel_star_2: PriorityLevelStar2) -> Self {
        ParseTree::PriorityLevelStar2(prioritylevel_star_2)
    }
}
impl From<AlternativeStar3> for ParseTree {
    fn from(alternative_star_3: AlternativeStar3) -> Self {
        ParseTree::AlternativeStar3(alternative_star_3)
    }
}
trait ListNode {
    type Item;
    fn iter(&self) -> impl Iterator<Item = &Self::Item>;
}
#[derive(Debug)]
pub struct Grammar(Token, Token, Token, GrammarStar0, Span);
#[derive(Debug)]
pub struct Rule(Token, Token, RuleOpt0, RuleStar1, Token, Span);
#[derive(Debug)]
pub struct PriorityLevel(PriorityLevelOpt1, PriorityLevelStar2, Span);
#[derive(Debug)]
pub struct Alternative(AlternativeStar3, Span);
#[derive(Debug)]
pub struct Symbol(Token, Span);
#[derive(Debug)]
pub enum GrammarStar0 {
    Alt0(Box<GrammarStar0>, Rule, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub enum RuleOpt0 {
    Alt0(PriorityLevel, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub struct RuleGroup0(Token, PriorityLevel, Span);
#[derive(Debug)]
pub enum RuleStar1 {
    Alt0(Box<RuleStar1>, RuleGroup0, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub enum PriorityLevelOpt1 {
    Alt0(Alternative, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub struct PriorityLevelGroup1(Token, Alternative, Span);
#[derive(Debug)]
pub enum PriorityLevelStar2 {
    Alt0(Box<PriorityLevelStar2>, PriorityLevelGroup1, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub enum AlternativeStar3 {
    Alt0(Box<AlternativeStar3>, Symbol, Span),
    Alt1(Span),
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
            4 => Some(self.4.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        5usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Rule(self)
    }
    pub fn span(&self) -> Span {
        self.5
    }
}
impl PriorityLevel {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            1 => Some(self.1.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        2usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevel(self)
    }
    pub fn span(&self) -> Span {
        self.2
    }
}
impl Alternative {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Alternative(self)
    }
    pub fn span(&self) -> Span {
        self.1
    }
}
impl Symbol {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Symbol(self)
    }
    pub fn span(&self) -> Span {
        self.1
    }
}
impl GrammarStar0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarStar0::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            GrammarStar0::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            GrammarStar0::Alt0(..) => 2usize,
            GrammarStar0::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::GrammarStar0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            GrammarStar0::Alt0(.., span) => *span,
            GrammarStar0::Alt1(.., span) => *span,
        }
    }
}
impl RuleOpt0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RuleOpt0::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            RuleOpt0::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RuleOpt0::Alt0(..) => 1usize,
            RuleOpt0::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RuleOpt0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RuleOpt0::Alt0(.., span) => *span,
            RuleOpt0::Alt1(.., span) => *span,
        }
    }
}
impl RuleGroup0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            1 => Some(self.1.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        2usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RuleGroup0(self)
    }
    pub fn span(&self) -> Span {
        self.2
    }
}
impl RuleStar1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RuleStar1::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            RuleStar1::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RuleStar1::Alt0(..) => 2usize,
            RuleStar1::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RuleStar1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RuleStar1::Alt0(.., span) => *span,
            RuleStar1::Alt1(.., span) => *span,
        }
    }
}
impl PriorityLevelOpt1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelOpt1::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelOpt1::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelOpt1::Alt0(..) => 1usize,
            PriorityLevelOpt1::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelOpt1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelOpt1::Alt0(.., span) => *span,
            PriorityLevelOpt1::Alt1(.., span) => *span,
        }
    }
}
impl PriorityLevelGroup1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.0.as_parse_tree_ref()),
            1 => Some(self.1.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        2usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelGroup1(self)
    }
    pub fn span(&self) -> Span {
        self.2
    }
}
impl PriorityLevelStar2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelStar2::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelStar2::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelStar2::Alt0(..) => 2usize,
            PriorityLevelStar2::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelStar2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelStar2::Alt0(.., span) => *span,
            PriorityLevelStar2::Alt1(.., span) => *span,
        }
    }
}
impl AlternativeStar3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativeStar3::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            AlternativeStar3::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativeStar3::Alt0(..) => 2usize,
            AlternativeStar3::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativeStar3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativeStar3::Alt0(.., span) => *span,
            AlternativeStar3::Alt1(.., span) => *span,
        }
    }
}
impl ListNode for GrammarStar0 {
    type Item = Rule;
    fn iter(&self) -> impl Iterator<Item = &Rule> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                GrammarStar0::Alt0(rest, item, _) => {
                    items.push(item);
                    current = rest;
                }
                GrammarStar0::Alt1(_) => {
                    break;
                }
            }
        }
        items.into_iter().rev()
    }
}
impl ListNode for RuleStar1 {
    type Item = RuleGroup0;
    fn iter(&self) -> impl Iterator<Item = &RuleGroup0> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RuleStar1::Alt0(rest, item, _) => {
                    items.push(item);
                    current = rest;
                }
                RuleStar1::Alt1(_) => {
                    break;
                }
            }
        }
        items.into_iter().rev()
    }
}
impl ListNode for PriorityLevelStar2 {
    type Item = PriorityLevelGroup1;
    fn iter(&self) -> impl Iterator<Item = &PriorityLevelGroup1> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                PriorityLevelStar2::Alt0(rest, item, _) => {
                    items.push(item);
                    current = rest;
                }
                PriorityLevelStar2::Alt1(_) => {
                    break;
                }
            }
        }
        items.into_iter().rev()
    }
}
impl ListNode for AlternativeStar3 {
    type Item = Symbol;
    fn iter(&self) -> impl Iterator<Item = &Symbol> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                AlternativeStar3::Alt0(rest, item, _) => {
                    items.push(item);
                    current = rest;
                }
                AlternativeStar3::Alt1(_) => {
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
        //">"
        TerminalId(5) => TokenKind::T5,
        //"|"
        TerminalId(6) => TokenKind::T6,
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
                    //Grammar : "grammar" Identifier ";" Grammar_Star_0.
                    SlotId(4) => {
                        let [c0, c1, c2, c3] = <[ParseTree; 4usize]>::try_from(children).unwrap();
                        Grammar(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_token(),
                            c3.unwrap_grammar_star_0(),
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
                    //Rule : Identifier ":" Rule_Opt_0 Rule_Star_1 ";".
                    SlotId(10) => {
                        let [c0, c1, c2, c3, c4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Rule(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_rule_opt_0(),
                            c3.unwrap_rule_star_1(),
                            c4.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel : PriorityLevel_Opt_1 PriorityLevel_Star_2.
                    SlotId(13) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        PriorityLevel(
                            c0.unwrap_prioritylevel_opt_1(),
                            c1.unwrap_prioritylevel_star_2(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //Alternative : Alternative_Star_3.
                    SlotId(15) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Alternative(c0.unwrap_alternative_star_3(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Symbol
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //Symbol : Identifier.
                    SlotId(17) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Symbol(c0.unwrap_token(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Star_0
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //Grammar_Star_0 : Grammar_Star_0 Rule.
                    SlotId(20) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        GrammarStar0::Alt0(
                            Box::new(c0.unwrap_grammar_star_0()),
                            c1.unwrap_rule(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Grammar_Star_0 : .
                    SlotId(21) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarStar0::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Opt_0
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //Rule_Opt_0 : PriorityLevel.
                    SlotId(23) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RuleOpt0::Alt0(c0.unwrap_prioritylevel(), nonterminal_node.span).into()
                    }
                    //Rule_Opt_0 : .
                    SlotId(24) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RuleOpt0::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Group_0
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //Rule_Group_0 : ">" PriorityLevel.
                    SlotId(27) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        RuleGroup0(
                            c0.unwrap_token(),
                            c1.unwrap_prioritylevel(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Star_1
            NonterminalId(8) => {
                match nonterminal_node.return_slot {
                    //Rule_Star_1 : Rule_Star_1 Rule_Group_0.
                    SlotId(30) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        RuleStar1::Alt0(
                            Box::new(c0.unwrap_rule_star_1()),
                            c1.unwrap_rule_group_0(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Rule_Star_1 : .
                    SlotId(31) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RuleStar1::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Opt_1
            NonterminalId(9) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel_Opt_1 : Alternative.
                    SlotId(33) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelOpt1::Alt0(c0.unwrap_alternative(), nonterminal_node.span)
                            .into()
                    }
                    //PriorityLevel_Opt_1 : .
                    SlotId(34) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        PriorityLevelOpt1::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Group_1
            NonterminalId(10) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel_Group_1 : "|" Alternative.
                    SlotId(37) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        PriorityLevelGroup1(
                            c0.unwrap_token(),
                            c1.unwrap_alternative(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Star_2
            NonterminalId(11) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel_Star_2 : PriorityLevel_Star_2 PriorityLevel_Group_1.
                    SlotId(40) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        PriorityLevelStar2::Alt0(
                            Box::new(c0.unwrap_prioritylevel_star_2()),
                            c1.unwrap_prioritylevel_group_1(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //PriorityLevel_Star_2 : .
                    SlotId(41) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        PriorityLevelStar2::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Star_3
            NonterminalId(12) => {
                match nonterminal_node.return_slot {
                    //Alternative_Star_3 : Alternative_Star_3 Symbol.
                    SlotId(44) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        AlternativeStar3::Alt0(
                            Box::new(c0.unwrap_alternative_star_3()),
                            c1.unwrap_symbol(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Alternative_Star_3 : .
                    SlotId(45) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        AlternativeStar3::Alt1(nonterminal_node.span).into()
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
        "PriorityLevel" => {
            ParseTree::PriorityLevel(create_parse_tree_prioritylevel(root_id, parser, builder))
        }
        "Alternative" => {
            ParseTree::Alternative(create_parse_tree_alternative(root_id, parser, builder))
        }
        "Symbol" => ParseTree::Symbol(create_parse_tree_symbol(root_id, parser, builder)),
        "Grammar_Star_0" => {
            ParseTree::GrammarStar0(create_parse_tree_grammar_star_0(root_id, parser, builder))
        }
        "Rule_Opt_0" => ParseTree::RuleOpt0(create_parse_tree_rule_opt_0(root_id, parser, builder)),
        "Rule_Group_0" => {
            ParseTree::RuleGroup0(create_parse_tree_rule_group_0(root_id, parser, builder))
        }
        "Rule_Star_1" => {
            ParseTree::RuleStar1(create_parse_tree_rule_star_1(root_id, parser, builder))
        }
        "PriorityLevel_Opt_1" => ParseTree::PriorityLevelOpt1(
            create_parse_tree_prioritylevel_opt_1(root_id, parser, builder),
        ),
        "PriorityLevel_Group_1" => ParseTree::PriorityLevelGroup1(
            create_parse_tree_prioritylevel_group_1(root_id, parser, builder),
        ),
        "PriorityLevel_Star_2" => ParseTree::PriorityLevelStar2(
            create_parse_tree_prioritylevel_star_2(root_id, parser, builder),
        ),
        "Alternative_Star_3" => ParseTree::AlternativeStar3(create_parse_tree_alternative_star_3(
            root_id, parser, builder,
        )),
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
pub fn create_parse_tree_prioritylevel(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevel {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_prioritylevel()
}
pub fn create_parse_tree_alternative(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Alternative {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative()
}
pub fn create_parse_tree_symbol(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Symbol {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_symbol()
}
pub fn create_parse_tree_grammar_star_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> GrammarStar0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_grammar_star_0()
}
pub fn create_parse_tree_rule_opt_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RuleOpt0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_rule_opt_0()
}
pub fn create_parse_tree_rule_group_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RuleGroup0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_rule_group_0()
}
pub fn create_parse_tree_rule_star_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RuleStar1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_rule_star_1()
}
pub fn create_parse_tree_prioritylevel_opt_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelOpt1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_prioritylevel_opt_1()
}
pub fn create_parse_tree_prioritylevel_group_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelGroup1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_prioritylevel_group_1()
}
pub fn create_parse_tree_prioritylevel_star_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelStar2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_prioritylevel_star_2()
}
pub fn create_parse_tree_alternative_star_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativeStar3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_star_3()
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

