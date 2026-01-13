use crate::parser::IggyParser;
use core::fmt;
use iguana::{
    ids::{NonterminalId, SlotId, TerminalId},
    parse_tree::{OneOrMany, ParseTreeBuilder, visit_sppf},
    parser::Parser,
    sppf::{NonterminalNode, SPPFNodeId, Span, TerminalNode},
};
use std::{fmt::Write, vec::IntoIter};
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
    //Rule+
    GrammarPlus0(GrammarPlus0),
    //Rule+?
    GrammarOpt0(GrammarOpt0),
    //Rule*
    GrammarStar0(GrammarStar0),
    //{PriorityLevel ">"}+
    RulePlus1(RulePlus1),
    //{PriorityLevel ">"}+?
    RuleOpt1(RuleOpt1),
    //{PriorityLevel ">"}*
    RuleStar1(RuleStar1),
    //{Alternative "|"}+
    PriorityLevelPlus2(PriorityLevelPlus2),
    //{Alternative "|"}+?
    PriorityLevelOpt2(PriorityLevelOpt2),
    //{Alternative "|"}*
    PriorityLevelStar2(PriorityLevelStar2),
    //Symbol+
    AlternativePlus3(AlternativePlus3),
    //Symbol+?
    AlternativeOpt3(AlternativeOpt3),
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
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0.as_parse_tree_ref(),
            ParseTree::GrammarOpt0(grammar_opt_0) => grammar_opt_0.as_parse_tree_ref(),
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0.as_parse_tree_ref(),
            ParseTree::RulePlus1(rule_plus_1) => rule_plus_1.as_parse_tree_ref(),
            ParseTree::RuleOpt1(rule_opt_1) => rule_opt_1.as_parse_tree_ref(),
            ParseTree::RuleStar1(rule_star_1) => rule_star_1.as_parse_tree_ref(),
            ParseTree::PriorityLevelPlus2(prioritylevel_plus_2) => {
                prioritylevel_plus_2.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelOpt2(prioritylevel_opt_2) => {
                prioritylevel_opt_2.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.as_parse_tree_ref()
            }
            ParseTree::AlternativePlus3(alternative_plus_3) => {
                alternative_plus_3.as_parse_tree_ref()
            }
            ParseTree::AlternativeOpt3(alternative_opt_3) => alternative_opt_3.as_parse_tree_ref(),
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
    fn unwrap_grammar_plus_0(self) -> GrammarPlus0 {
        match self {
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_opt_0(self) -> GrammarOpt0 {
        match self {
            ParseTree::GrammarOpt0(grammar_opt_0) => grammar_opt_0,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_star_0(self) -> GrammarStar0 {
        match self {
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0,
            _ => panic!(),
        }
    }
    fn unwrap_rule_plus_1(self) -> RulePlus1 {
        match self {
            ParseTree::RulePlus1(rule_plus_1) => rule_plus_1,
            _ => panic!(),
        }
    }
    fn unwrap_rule_opt_1(self) -> RuleOpt1 {
        match self {
            ParseTree::RuleOpt1(rule_opt_1) => rule_opt_1,
            _ => panic!(),
        }
    }
    fn unwrap_rule_star_1(self) -> RuleStar1 {
        match self {
            ParseTree::RuleStar1(rule_star_1) => rule_star_1,
            _ => panic!(),
        }
    }
    fn unwrap_prioritylevel_plus_2(self) -> PriorityLevelPlus2 {
        match self {
            ParseTree::PriorityLevelPlus2(prioritylevel_plus_2) => prioritylevel_plus_2,
            _ => panic!(),
        }
    }
    fn unwrap_prioritylevel_opt_2(self) -> PriorityLevelOpt2 {
        match self {
            ParseTree::PriorityLevelOpt2(prioritylevel_opt_2) => prioritylevel_opt_2,
            _ => panic!(),
        }
    }
    fn unwrap_prioritylevel_star_2(self) -> PriorityLevelStar2 {
        match self {
            ParseTree::PriorityLevelStar2(prioritylevel_star_2) => prioritylevel_star_2,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_plus_3(self) -> AlternativePlus3 {
        match self {
            ParseTree::AlternativePlus3(alternative_plus_3) => alternative_plus_3,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_opt_3(self) -> AlternativeOpt3 {
        match self {
            ParseTree::AlternativeOpt3(alternative_opt_3) => alternative_opt_3,
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
    GrammarPlus0(&'a GrammarPlus0),
    GrammarOpt0(&'a GrammarOpt0),
    GrammarStar0(&'a GrammarStar0),
    RulePlus1(&'a RulePlus1),
    RuleOpt1(&'a RuleOpt1),
    RuleStar1(&'a RuleStar1),
    PriorityLevelPlus2(&'a PriorityLevelPlus2),
    PriorityLevelOpt2(&'a PriorityLevelOpt2),
    PriorityLevelStar2(&'a PriorityLevelStar2),
    AlternativePlus3(&'a AlternativePlus3),
    AlternativeOpt3(&'a AlternativeOpt3),
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
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.iter().collect(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => (0..grammar_opt_0.child_count())
                .filter_map(|i| grammar_opt_0.child(i))
                .collect(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.iter().collect(),
            ParseTreeRef::RulePlus1(rule_plus_1) => rule_plus_1.iter().collect(),
            ParseTreeRef::RuleOpt1(rule_opt_1) => (0..rule_opt_1.child_count())
                .filter_map(|i| rule_opt_1.child(i))
                .collect(),
            ParseTreeRef::RuleStar1(rule_star_1) => rule_star_1.iter().collect(),
            ParseTreeRef::PriorityLevelPlus2(prioritylevel_plus_2) => {
                prioritylevel_plus_2.iter().collect()
            }
            ParseTreeRef::PriorityLevelOpt2(prioritylevel_opt_2) => (0..prioritylevel_opt_2
                .child_count())
                .filter_map(|i| prioritylevel_opt_2.child(i))
                .collect(),
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.iter().collect()
            }
            ParseTreeRef::AlternativePlus3(alternative_plus_3) => {
                alternative_plus_3.iter().collect()
            }
            ParseTreeRef::AlternativeOpt3(alternative_opt_3) => (0..alternative_opt_3
                .child_count())
                .filter_map(|i| alternative_opt_3.child(i))
                .collect(),
            ParseTreeRef::AlternativeStar3(alternative_star_3) => {
                alternative_star_3.iter().collect()
            }
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
            ParseTreeRef::GrammarPlus0(_) => "Rule+",
            ParseTreeRef::GrammarOpt0(_) => "Rule+?",
            ParseTreeRef::GrammarStar0(_) => "Rule*",
            ParseTreeRef::RulePlus1(_) => "{PriorityLevel \">\"}+",
            ParseTreeRef::RuleOpt1(_) => "{PriorityLevel \">\"}+?",
            ParseTreeRef::RuleStar1(_) => "{PriorityLevel \">\"}*",
            ParseTreeRef::PriorityLevelPlus2(_) => "{Alternative \"|\"}+",
            ParseTreeRef::PriorityLevelOpt2(_) => "{Alternative \"|\"}+?",
            ParseTreeRef::PriorityLevelStar2(_) => "{Alternative \"|\"}*",
            ParseTreeRef::AlternativePlus3(_) => "Symbol+",
            ParseTreeRef::AlternativeOpt3(_) => "Symbol+?",
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
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.child_count(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.child_count(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.child_count(),
            ParseTreeRef::RulePlus1(rule_plus_1) => rule_plus_1.child_count(),
            ParseTreeRef::RuleOpt1(rule_opt_1) => rule_opt_1.child_count(),
            ParseTreeRef::RuleStar1(rule_star_1) => rule_star_1.child_count(),
            ParseTreeRef::PriorityLevelPlus2(prioritylevel_plus_2) => {
                prioritylevel_plus_2.child_count()
            }
            ParseTreeRef::PriorityLevelOpt2(prioritylevel_opt_2) => {
                prioritylevel_opt_2.child_count()
            }
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.child_count()
            }
            ParseTreeRef::AlternativePlus3(alternative_plus_3) => alternative_plus_3.child_count(),
            ParseTreeRef::AlternativeOpt3(alternative_opt_3) => alternative_opt_3.child_count(),
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
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.span(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.span(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.span(),
            ParseTreeRef::RulePlus1(rule_plus_1) => rule_plus_1.span(),
            ParseTreeRef::RuleOpt1(rule_opt_1) => rule_opt_1.span(),
            ParseTreeRef::RuleStar1(rule_star_1) => rule_star_1.span(),
            ParseTreeRef::PriorityLevelPlus2(prioritylevel_plus_2) => prioritylevel_plus_2.span(),
            ParseTreeRef::PriorityLevelOpt2(prioritylevel_opt_2) => prioritylevel_opt_2.span(),
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => prioritylevel_star_2.span(),
            ParseTreeRef::AlternativePlus3(alternative_plus_3) => alternative_plus_3.span(),
            ParseTreeRef::AlternativeOpt3(alternative_opt_3) => alternative_opt_3.span(),
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
impl From<GrammarPlus0> for ParseTree {
    fn from(grammar_plus_0: GrammarPlus0) -> Self {
        ParseTree::GrammarPlus0(grammar_plus_0)
    }
}
impl From<GrammarOpt0> for ParseTree {
    fn from(grammar_opt_0: GrammarOpt0) -> Self {
        ParseTree::GrammarOpt0(grammar_opt_0)
    }
}
impl From<GrammarStar0> for ParseTree {
    fn from(grammar_star_0: GrammarStar0) -> Self {
        ParseTree::GrammarStar0(grammar_star_0)
    }
}
impl From<RulePlus1> for ParseTree {
    fn from(rule_plus_1: RulePlus1) -> Self {
        ParseTree::RulePlus1(rule_plus_1)
    }
}
impl From<RuleOpt1> for ParseTree {
    fn from(rule_opt_1: RuleOpt1) -> Self {
        ParseTree::RuleOpt1(rule_opt_1)
    }
}
impl From<RuleStar1> for ParseTree {
    fn from(rule_star_1: RuleStar1) -> Self {
        ParseTree::RuleStar1(rule_star_1)
    }
}
impl From<PriorityLevelPlus2> for ParseTree {
    fn from(prioritylevel_plus_2: PriorityLevelPlus2) -> Self {
        ParseTree::PriorityLevelPlus2(prioritylevel_plus_2)
    }
}
impl From<PriorityLevelOpt2> for ParseTree {
    fn from(prioritylevel_opt_2: PriorityLevelOpt2) -> Self {
        ParseTree::PriorityLevelOpt2(prioritylevel_opt_2)
    }
}
impl From<PriorityLevelStar2> for ParseTree {
    fn from(prioritylevel_star_2: PriorityLevelStar2) -> Self {
        ParseTree::PriorityLevelStar2(prioritylevel_star_2)
    }
}
impl From<AlternativePlus3> for ParseTree {
    fn from(alternative_plus_3: AlternativePlus3) -> Self {
        ParseTree::AlternativePlus3(alternative_plus_3)
    }
}
impl From<AlternativeOpt3> for ParseTree {
    fn from(alternative_opt_3: AlternativeOpt3) -> Self {
        ParseTree::AlternativeOpt3(alternative_opt_3)
    }
}
impl From<AlternativeStar3> for ParseTree {
    fn from(alternative_star_3: AlternativeStar3) -> Self {
        ParseTree::AlternativeStar3(alternative_star_3)
    }
}
trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
#[derive(Debug)]
pub struct Grammar(Token, Token, Token, GrammarStar0, Span);
#[derive(Debug)]
pub struct Rule(Token, Token, RuleStar1, Token, Span);
#[derive(Debug)]
pub struct PriorityLevel(PriorityLevelStar2, Span);
#[derive(Debug)]
pub struct Alternative(AlternativeStar3, Span);
#[derive(Debug)]
pub struct Symbol(Token, Span);
#[derive(Debug)]
pub enum GrammarPlus0 {
    Alt0(Box<GrammarPlus0>, Rule, Span),
    Alt1(Rule, Span),
}
#[derive(Debug)]
pub enum GrammarOpt0 {
    Alt0(GrammarPlus0, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub struct GrammarStar0(GrammarOpt0, Span);
#[derive(Debug)]
pub enum RulePlus1 {
    Alt0(Box<RulePlus1>, Token, PriorityLevel, Span),
    Alt1(PriorityLevel, Span),
}
#[derive(Debug)]
pub enum RuleOpt1 {
    Alt0(RulePlus1, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub struct RuleStar1(RuleOpt1, Span);
#[derive(Debug)]
pub enum PriorityLevelPlus2 {
    Alt0(Box<PriorityLevelPlus2>, Token, Alternative, Span),
    Alt1(Alternative, Span),
}
#[derive(Debug)]
pub enum PriorityLevelOpt2 {
    Alt0(PriorityLevelPlus2, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub struct PriorityLevelStar2(PriorityLevelOpt2, Span);
#[derive(Debug)]
pub enum AlternativePlus3 {
    Alt0(Box<AlternativePlus3>, Symbol, Span),
    Alt1(Symbol, Span),
}
#[derive(Debug)]
pub enum AlternativeOpt3 {
    Alt0(AlternativePlus3, Span),
    Alt1(Span),
}
#[derive(Debug)]
pub struct AlternativeStar3(AlternativeOpt3, Span);
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
impl PriorityLevel {
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
        ParseTreeRef::PriorityLevel(self)
    }
    pub fn span(&self) -> Span {
        self.1
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
impl GrammarOpt0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarOpt0::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            GrammarOpt0::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            GrammarOpt0::Alt0(..) => 1usize,
            GrammarOpt0::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::GrammarOpt0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            GrammarOpt0::Alt0(.., span) => *span,
            GrammarOpt0::Alt1(.., span) => *span,
        }
    }
}
impl GrammarStar0 {
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
        ParseTreeRef::GrammarStar0(self)
    }
    pub fn span(&self) -> Span {
        self.1
    }
}
impl RulePlus1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RulePlus1::Alt0(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
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
            RulePlus1::Alt0(..) => 3usize,
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
impl RuleOpt1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RuleOpt1::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            RuleOpt1::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RuleOpt1::Alt0(..) => 1usize,
            RuleOpt1::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RuleOpt1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RuleOpt1::Alt0(.., span) => *span,
            RuleOpt1::Alt1(.., span) => *span,
        }
    }
}
impl RuleStar1 {
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
        ParseTreeRef::RuleStar1(self)
    }
    pub fn span(&self) -> Span {
        self.1
    }
}
impl PriorityLevelPlus2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelPlus2::Alt0(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelPlus2::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelPlus2::Alt0(..) => 3usize,
            PriorityLevelPlus2::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelPlus2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelPlus2::Alt0(.., span) => *span,
            PriorityLevelPlus2::Alt1(.., span) => *span,
        }
    }
}
impl PriorityLevelOpt2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelOpt2::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelOpt2::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelOpt2::Alt0(..) => 1usize,
            PriorityLevelOpt2::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelOpt2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelOpt2::Alt0(.., span) => *span,
            PriorityLevelOpt2::Alt1(.., span) => *span,
        }
    }
}
impl PriorityLevelStar2 {
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
        ParseTreeRef::PriorityLevelStar2(self)
    }
    pub fn span(&self) -> Span {
        self.1
    }
}
impl AlternativePlus3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativePlus3::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            AlternativePlus3::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativePlus3::Alt0(..) => 2usize,
            AlternativePlus3::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativePlus3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativePlus3::Alt0(.., span) => *span,
            AlternativePlus3::Alt1(.., span) => *span,
        }
    }
}
impl AlternativeOpt3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativeOpt3::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            AlternativeOpt3::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativeOpt3::Alt0(..) => 1usize,
            AlternativeOpt3::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativeOpt3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativeOpt3::Alt0(.., span) => *span,
            AlternativeOpt3::Alt1(.., span) => *span,
        }
    }
}
impl AlternativeStar3 {
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
        ParseTreeRef::AlternativeStar3(self)
    }
    pub fn span(&self) -> Span {
        self.1
    }
}
impl<'a> ListNode<'a> for GrammarPlus0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                GrammarPlus0::Alt0(rest, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    current = rest;
                }
                GrammarPlus0::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for RulePlus1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RulePlus1::Alt0(rest, sep, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    items.push(sep.as_parse_tree_ref());
                    current = rest;
                }
                RulePlus1::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for PriorityLevelPlus2 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                PriorityLevelPlus2::Alt0(rest, sep, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    items.push(sep.as_parse_tree_ref());
                    current = rest;
                }
                PriorityLevelPlus2::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for AlternativePlus3 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                AlternativePlus3::Alt0(rest, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    current = rest;
                }
                AlternativePlus3::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for GrammarStar0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.0 {
            GrammarOpt0::Alt0(grammar_opt_0, _) => grammar_opt_0.iter(),
            GrammarOpt0::Alt1(_) => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for RuleStar1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.0 {
            RuleOpt1::Alt0(rule_opt_1, _) => rule_opt_1.iter(),
            RuleOpt1::Alt1(_) => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for PriorityLevelStar2 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.0 {
            PriorityLevelOpt2::Alt0(prioritylevel_opt_2, _) => prioritylevel_opt_2.iter(),
            PriorityLevelOpt2::Alt1(_) => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for AlternativeStar3 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.0 {
            AlternativeOpt3::Alt0(alternative_opt_3, _) => alternative_opt_3.iter(),
            AlternativeOpt3::Alt1(_) => vec![].into_iter(),
        }
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
                    //Rule : Identifier ":" Rule_Star_1 ";".
                    SlotId(9) => {
                        let [c0, c1, c2, c3] = <[ParseTree; 4usize]>::try_from(children).unwrap();
                        Rule(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_rule_star_1(),
                            c3.unwrap_token(),
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
                    //PriorityLevel : PriorityLevel_Star_2.
                    SlotId(11) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevel(c0.unwrap_prioritylevel_star_2(), nonterminal_node.span)
                            .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //Alternative : Alternative_Star_3.
                    SlotId(13) => {
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
                    SlotId(15) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Symbol(c0.unwrap_token(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Plus_0
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //Grammar_Plus_0 : Grammar_Plus_0 Rule.
                    SlotId(18) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt0(
                            Box::new(c0.unwrap_grammar_plus_0()),
                            c1.unwrap_rule(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Grammar_Plus_0 : Rule.
                    SlotId(20) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt1(c0.unwrap_rule(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Opt_0
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //Grammar_Opt_0 : Grammar_Plus_0.
                    SlotId(22) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt0(c0.unwrap_grammar_plus_0(), nonterminal_node.span).into()
                    }
                    //Grammar_Opt_0 : .
                    SlotId(23) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Star_0
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //Grammar_Star_0 : Grammar_Opt_0.
                    SlotId(25) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarStar0(c0.unwrap_grammar_opt_0(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Plus_1
            NonterminalId(8) => {
                match nonterminal_node.return_slot {
                    //Rule_Plus_1 : Rule_Plus_1 ">" PriorityLevel.
                    SlotId(29) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RulePlus1::Alt0(
                            Box::new(c0.unwrap_rule_plus_1()),
                            c1.unwrap_token(),
                            c2.unwrap_prioritylevel(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Rule_Plus_1 : PriorityLevel.
                    SlotId(31) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RulePlus1::Alt1(c0.unwrap_prioritylevel(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Opt_1
            NonterminalId(9) => {
                match nonterminal_node.return_slot {
                    //Rule_Opt_1 : Rule_Plus_1.
                    SlotId(33) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RuleOpt1::Alt0(c0.unwrap_rule_plus_1(), nonterminal_node.span).into()
                    }
                    //Rule_Opt_1 : .
                    SlotId(34) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RuleOpt1::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Star_1
            NonterminalId(10) => {
                match nonterminal_node.return_slot {
                    //Rule_Star_1 : Rule_Opt_1.
                    SlotId(36) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RuleStar1(c0.unwrap_rule_opt_1(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Plus_2
            NonterminalId(11) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel_Plus_2 : PriorityLevel_Plus_2 "|" Alternative.
                    SlotId(40) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        PriorityLevelPlus2::Alt0(
                            Box::new(c0.unwrap_prioritylevel_plus_2()),
                            c1.unwrap_token(),
                            c2.unwrap_alternative(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //PriorityLevel_Plus_2 : Alternative.
                    SlotId(42) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelPlus2::Alt1(c0.unwrap_alternative(), nonterminal_node.span)
                            .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Opt_2
            NonterminalId(12) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel_Opt_2 : PriorityLevel_Plus_2.
                    SlotId(44) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelOpt2::Alt0(
                            c0.unwrap_prioritylevel_plus_2(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //PriorityLevel_Opt_2 : .
                    SlotId(45) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        PriorityLevelOpt2::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Star_2
            NonterminalId(13) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel_Star_2 : PriorityLevel_Opt_2.
                    SlotId(47) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelStar2(c0.unwrap_prioritylevel_opt_2(), nonterminal_node.span)
                            .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Plus_3
            NonterminalId(14) => {
                match nonterminal_node.return_slot {
                    //Alternative_Plus_3 : Alternative_Plus_3 Symbol.
                    SlotId(50) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        AlternativePlus3::Alt0(
                            Box::new(c0.unwrap_alternative_plus_3()),
                            c1.unwrap_symbol(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Alternative_Plus_3 : Symbol.
                    SlotId(52) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativePlus3::Alt1(c0.unwrap_symbol(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Opt_3
            NonterminalId(15) => {
                match nonterminal_node.return_slot {
                    //Alternative_Opt_3 : Alternative_Plus_3.
                    SlotId(54) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeOpt3::Alt0(c0.unwrap_alternative_plus_3(), nonterminal_node.span)
                            .into()
                    }
                    //Alternative_Opt_3 : .
                    SlotId(55) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        AlternativeOpt3::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Star_3
            NonterminalId(16) => {
                match nonterminal_node.return_slot {
                    //Alternative_Star_3 : Alternative_Opt_3.
                    SlotId(57) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeStar3(c0.unwrap_alternative_opt_3(), nonterminal_node.span)
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
        "Grammar_Plus_0" => {
            ParseTree::GrammarPlus0(create_parse_tree_grammar_plus_0(root_id, parser, builder))
        }
        "Grammar_Opt_0" => {
            ParseTree::GrammarOpt0(create_parse_tree_grammar_opt_0(root_id, parser, builder))
        }
        "Grammar_Star_0" => {
            ParseTree::GrammarStar0(create_parse_tree_grammar_star_0(root_id, parser, builder))
        }
        "Rule_Plus_1" => {
            ParseTree::RulePlus1(create_parse_tree_rule_plus_1(root_id, parser, builder))
        }
        "Rule_Opt_1" => ParseTree::RuleOpt1(create_parse_tree_rule_opt_1(root_id, parser, builder)),
        "Rule_Star_1" => {
            ParseTree::RuleStar1(create_parse_tree_rule_star_1(root_id, parser, builder))
        }
        "PriorityLevel_Plus_2" => ParseTree::PriorityLevelPlus2(
            create_parse_tree_prioritylevel_plus_2(root_id, parser, builder),
        ),
        "PriorityLevel_Opt_2" => ParseTree::PriorityLevelOpt2(
            create_parse_tree_prioritylevel_opt_2(root_id, parser, builder),
        ),
        "PriorityLevel_Star_2" => ParseTree::PriorityLevelStar2(
            create_parse_tree_prioritylevel_star_2(root_id, parser, builder),
        ),
        "Alternative_Plus_3" => ParseTree::AlternativePlus3(create_parse_tree_alternative_plus_3(
            root_id, parser, builder,
        )),
        "Alternative_Opt_3" => ParseTree::AlternativeOpt3(create_parse_tree_alternative_opt_3(
            root_id, parser, builder,
        )),
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
pub fn create_parse_tree_grammar_opt_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> GrammarOpt0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_grammar_opt_0()
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
pub fn create_parse_tree_rule_opt_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RuleOpt1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_rule_opt_1()
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
pub fn create_parse_tree_prioritylevel_plus_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelPlus2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_prioritylevel_plus_2()
}
pub fn create_parse_tree_prioritylevel_opt_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelOpt2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_prioritylevel_opt_2()
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
pub fn create_parse_tree_alternative_plus_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativePlus3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_plus_3()
}
pub fn create_parse_tree_alternative_opt_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativeOpt3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_opt_3()
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

