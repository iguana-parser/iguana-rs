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
    //String
    T1,
    //Char
    T2,
    //WS
    T3,
    //"grammar"
    T4,
    //"="
    T5,
    //">"
    T6,
    //"/"
    T7,
    //"|"
    T8,
    //"*"
    T9,
    //"+"
    T10,
    //"("
    T11,
    //")"
    T12,
    //"""
    T13,
    //"{"
    T14,
    //"}"
    T15,
    //"?"
    T16,
    //"!"
    T17,
    //"["
    T18,
    //"]"
    T19,
    //"-"
    T20,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Identifier",
            TokenKind::T1 => "String",
            TokenKind::T2 => "Char",
            TokenKind::T3 => "WS",
            TokenKind::T4 => "\"grammar\"",
            TokenKind::T5 => "\"=\"",
            TokenKind::T6 => "\">\"",
            TokenKind::T7 => "\"/\"",
            TokenKind::T8 => "\"|\"",
            TokenKind::T9 => "\"*\"",
            TokenKind::T10 => "\"+\"",
            TokenKind::T11 => "\"(\"",
            TokenKind::T12 => "\")\"",
            TokenKind::T13 => "\"\"\"",
            TokenKind::T14 => "\"{\"",
            TokenKind::T15 => "\"}\"",
            TokenKind::T16 => "\"?\"",
            TokenKind::T17 => "\"!\"",
            TokenKind::T18 => "\"[\"",
            TokenKind::T19 => "\"]\"",
            TokenKind::T20 => "\"-\"",
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
    //Regex
    Regex(Regex),
    //CharClass
    CharClass(CharClass),
    //CharRange
    CharRange(CharRange),
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
    //Regex+
    RulePlus3(RulePlus3),
    //{Regex+ "|"}+
    RulePlus2(RulePlus2),
    //{Alternative "|"}+
    PriorityLevelPlus4(PriorityLevelPlus4),
    //{Alternative "|"}+?
    PriorityLevelOpt2(PriorityLevelOpt2),
    //{Alternative "|"}*
    PriorityLevelStar2(PriorityLevelStar2),
    //Symbol+
    AlternativePlus5(AlternativePlus5),
    //Symbol+?
    AlternativeOpt3(AlternativeOpt3),
    //Symbol*
    AlternativeStar3(AlternativeStar3),
    //{Regex+ "|"}+?
    RegexOpt4(RegexOpt4),
    //{Regex+ "|"}*
    RegexStar4(RegexStar4),
    //"!"?
    CharClassOpt5(CharClassOpt5),
    //(CharRange | Char)
    CharClassAlt0(CharClassAlt0),
    //(CharRange | Char)+
    CharClassPlus6(CharClassPlus6),
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
            ParseTree::Regex(regex) => regex.as_parse_tree_ref(),
            ParseTree::CharClass(charclass) => charclass.as_parse_tree_ref(),
            ParseTree::CharRange(charrange) => charrange.as_parse_tree_ref(),
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0.as_parse_tree_ref(),
            ParseTree::GrammarOpt0(grammar_opt_0) => grammar_opt_0.as_parse_tree_ref(),
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0.as_parse_tree_ref(),
            ParseTree::RulePlus1(rule_plus_1) => rule_plus_1.as_parse_tree_ref(),
            ParseTree::RuleOpt1(rule_opt_1) => rule_opt_1.as_parse_tree_ref(),
            ParseTree::RuleStar1(rule_star_1) => rule_star_1.as_parse_tree_ref(),
            ParseTree::RulePlus3(rule_plus_3) => rule_plus_3.as_parse_tree_ref(),
            ParseTree::RulePlus2(rule_plus_2) => rule_plus_2.as_parse_tree_ref(),
            ParseTree::PriorityLevelPlus4(prioritylevel_plus_4) => {
                prioritylevel_plus_4.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelOpt2(prioritylevel_opt_2) => {
                prioritylevel_opt_2.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.as_parse_tree_ref()
            }
            ParseTree::AlternativePlus5(alternative_plus_5) => {
                alternative_plus_5.as_parse_tree_ref()
            }
            ParseTree::AlternativeOpt3(alternative_opt_3) => alternative_opt_3.as_parse_tree_ref(),
            ParseTree::AlternativeStar3(alternative_star_3) => {
                alternative_star_3.as_parse_tree_ref()
            }
            ParseTree::RegexOpt4(regex_opt_4) => regex_opt_4.as_parse_tree_ref(),
            ParseTree::RegexStar4(regex_star_4) => regex_star_4.as_parse_tree_ref(),
            ParseTree::CharClassOpt5(charclass_opt_5) => charclass_opt_5.as_parse_tree_ref(),
            ParseTree::CharClassAlt0(charclass_alt_0) => charclass_alt_0.as_parse_tree_ref(),
            ParseTree::CharClassPlus6(charclass_plus_6) => charclass_plus_6.as_parse_tree_ref(),
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
    fn unwrap_regex(self) -> Regex {
        match self {
            ParseTree::Regex(regex) => regex,
            _ => panic!(),
        }
    }
    fn unwrap_charclass(self) -> CharClass {
        match self {
            ParseTree::CharClass(charclass) => charclass,
            _ => panic!(),
        }
    }
    fn unwrap_charrange(self) -> CharRange {
        match self {
            ParseTree::CharRange(charrange) => charrange,
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
    fn unwrap_rule_plus_3(self) -> RulePlus3 {
        match self {
            ParseTree::RulePlus3(rule_plus_3) => rule_plus_3,
            _ => panic!(),
        }
    }
    fn unwrap_rule_plus_2(self) -> RulePlus2 {
        match self {
            ParseTree::RulePlus2(rule_plus_2) => rule_plus_2,
            _ => panic!(),
        }
    }
    fn unwrap_prioritylevel_plus_4(self) -> PriorityLevelPlus4 {
        match self {
            ParseTree::PriorityLevelPlus4(prioritylevel_plus_4) => prioritylevel_plus_4,
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
    fn unwrap_alternative_plus_5(self) -> AlternativePlus5 {
        match self {
            ParseTree::AlternativePlus5(alternative_plus_5) => alternative_plus_5,
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
    fn unwrap_regex_opt_4(self) -> RegexOpt4 {
        match self {
            ParseTree::RegexOpt4(regex_opt_4) => regex_opt_4,
            _ => panic!(),
        }
    }
    fn unwrap_regex_star_4(self) -> RegexStar4 {
        match self {
            ParseTree::RegexStar4(regex_star_4) => regex_star_4,
            _ => panic!(),
        }
    }
    fn unwrap_charclass_opt_5(self) -> CharClassOpt5 {
        match self {
            ParseTree::CharClassOpt5(charclass_opt_5) => charclass_opt_5,
            _ => panic!(),
        }
    }
    fn unwrap_charclass_alt_0(self) -> CharClassAlt0 {
        match self {
            ParseTree::CharClassAlt0(charclass_alt_0) => charclass_alt_0,
            _ => panic!(),
        }
    }
    fn unwrap_charclass_plus_6(self) -> CharClassPlus6 {
        match self {
            ParseTree::CharClassPlus6(charclass_plus_6) => charclass_plus_6,
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
    Regex(&'a Regex),
    CharClass(&'a CharClass),
    CharRange(&'a CharRange),
    GrammarPlus0(&'a GrammarPlus0),
    GrammarOpt0(&'a GrammarOpt0),
    GrammarStar0(&'a GrammarStar0),
    RulePlus1(&'a RulePlus1),
    RuleOpt1(&'a RuleOpt1),
    RuleStar1(&'a RuleStar1),
    RulePlus3(&'a RulePlus3),
    RulePlus2(&'a RulePlus2),
    PriorityLevelPlus4(&'a PriorityLevelPlus4),
    PriorityLevelOpt2(&'a PriorityLevelOpt2),
    PriorityLevelStar2(&'a PriorityLevelStar2),
    AlternativePlus5(&'a AlternativePlus5),
    AlternativeOpt3(&'a AlternativeOpt3),
    AlternativeStar3(&'a AlternativeStar3),
    RegexOpt4(&'a RegexOpt4),
    RegexStar4(&'a RegexStar4),
    CharClassOpt5(&'a CharClassOpt5),
    CharClassAlt0(&'a CharClassAlt0),
    CharClassPlus6(&'a CharClassPlus6),
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
            ParseTreeRef::Regex(regex) => (0..regex.child_count())
                .filter_map(|i| regex.child(i))
                .collect(),
            ParseTreeRef::CharClass(charclass) => (0..charclass.child_count())
                .filter_map(|i| charclass.child(i))
                .collect(),
            ParseTreeRef::CharRange(charrange) => (0..charrange.child_count())
                .filter_map(|i| charrange.child(i))
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
            ParseTreeRef::RulePlus3(rule_plus_3) => rule_plus_3.iter().collect(),
            ParseTreeRef::RulePlus2(rule_plus_2) => rule_plus_2.iter().collect(),
            ParseTreeRef::PriorityLevelPlus4(prioritylevel_plus_4) => {
                prioritylevel_plus_4.iter().collect()
            }
            ParseTreeRef::PriorityLevelOpt2(prioritylevel_opt_2) => (0..prioritylevel_opt_2
                .child_count())
                .filter_map(|i| prioritylevel_opt_2.child(i))
                .collect(),
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.iter().collect()
            }
            ParseTreeRef::AlternativePlus5(alternative_plus_5) => {
                alternative_plus_5.iter().collect()
            }
            ParseTreeRef::AlternativeOpt3(alternative_opt_3) => (0..alternative_opt_3
                .child_count())
                .filter_map(|i| alternative_opt_3.child(i))
                .collect(),
            ParseTreeRef::AlternativeStar3(alternative_star_3) => {
                alternative_star_3.iter().collect()
            }
            ParseTreeRef::RegexOpt4(regex_opt_4) => (0..regex_opt_4.child_count())
                .filter_map(|i| regex_opt_4.child(i))
                .collect(),
            ParseTreeRef::RegexStar4(regex_star_4) => regex_star_4.iter().collect(),
            ParseTreeRef::CharClassOpt5(charclass_opt_5) => (0..charclass_opt_5.child_count())
                .filter_map(|i| charclass_opt_5.child(i))
                .collect(),
            ParseTreeRef::CharClassAlt0(charclass_alt_0) => (0..charclass_alt_0.child_count())
                .filter_map(|i| charclass_alt_0.child(i))
                .collect(),
            ParseTreeRef::CharClassPlus6(charclass_plus_6) => charclass_plus_6.iter().collect(),
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
            ParseTreeRef::Regex(_) => "Regex",
            ParseTreeRef::CharClass(_) => "CharClass",
            ParseTreeRef::CharRange(_) => "CharRange",
            ParseTreeRef::GrammarPlus0(_) => "Rule+",
            ParseTreeRef::GrammarOpt0(_) => "Rule+?",
            ParseTreeRef::GrammarStar0(_) => "Rule*",
            ParseTreeRef::RulePlus1(_) => "{PriorityLevel \">\"}+",
            ParseTreeRef::RuleOpt1(_) => "{PriorityLevel \">\"}+?",
            ParseTreeRef::RuleStar1(_) => "{PriorityLevel \">\"}*",
            ParseTreeRef::RulePlus3(_) => "Regex+",
            ParseTreeRef::RulePlus2(_) => "{Regex+ \"|\"}+",
            ParseTreeRef::PriorityLevelPlus4(_) => "{Alternative \"|\"}+",
            ParseTreeRef::PriorityLevelOpt2(_) => "{Alternative \"|\"}+?",
            ParseTreeRef::PriorityLevelStar2(_) => "{Alternative \"|\"}*",
            ParseTreeRef::AlternativePlus5(_) => "Symbol+",
            ParseTreeRef::AlternativeOpt3(_) => "Symbol+?",
            ParseTreeRef::AlternativeStar3(_) => "Symbol*",
            ParseTreeRef::RegexOpt4(_) => "{Regex+ \"|\"}+?",
            ParseTreeRef::RegexStar4(_) => "{Regex+ \"|\"}*",
            ParseTreeRef::CharClassOpt5(_) => "\"!\"?",
            ParseTreeRef::CharClassAlt0(_) => "(CharRange | Char)",
            ParseTreeRef::CharClassPlus6(_) => "(CharRange | Char)+",
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
            ParseTreeRef::Regex(regex) => regex.child_count(),
            ParseTreeRef::CharClass(charclass) => charclass.child_count(),
            ParseTreeRef::CharRange(charrange) => charrange.child_count(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.child_count(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.child_count(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.child_count(),
            ParseTreeRef::RulePlus1(rule_plus_1) => rule_plus_1.child_count(),
            ParseTreeRef::RuleOpt1(rule_opt_1) => rule_opt_1.child_count(),
            ParseTreeRef::RuleStar1(rule_star_1) => rule_star_1.child_count(),
            ParseTreeRef::RulePlus3(rule_plus_3) => rule_plus_3.child_count(),
            ParseTreeRef::RulePlus2(rule_plus_2) => rule_plus_2.child_count(),
            ParseTreeRef::PriorityLevelPlus4(prioritylevel_plus_4) => {
                prioritylevel_plus_4.child_count()
            }
            ParseTreeRef::PriorityLevelOpt2(prioritylevel_opt_2) => {
                prioritylevel_opt_2.child_count()
            }
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => {
                prioritylevel_star_2.child_count()
            }
            ParseTreeRef::AlternativePlus5(alternative_plus_5) => alternative_plus_5.child_count(),
            ParseTreeRef::AlternativeOpt3(alternative_opt_3) => alternative_opt_3.child_count(),
            ParseTreeRef::AlternativeStar3(alternative_star_3) => alternative_star_3.child_count(),
            ParseTreeRef::RegexOpt4(regex_opt_4) => regex_opt_4.child_count(),
            ParseTreeRef::RegexStar4(regex_star_4) => regex_star_4.child_count(),
            ParseTreeRef::CharClassOpt5(charclass_opt_5) => charclass_opt_5.child_count(),
            ParseTreeRef::CharClassAlt0(charclass_alt_0) => charclass_alt_0.child_count(),
            ParseTreeRef::CharClassPlus6(charclass_plus_6) => charclass_plus_6.child_count(),
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
            ParseTreeRef::Regex(regex) => regex.span(),
            ParseTreeRef::CharClass(charclass) => charclass.span(),
            ParseTreeRef::CharRange(charrange) => charrange.span(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.span(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.span(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.span(),
            ParseTreeRef::RulePlus1(rule_plus_1) => rule_plus_1.span(),
            ParseTreeRef::RuleOpt1(rule_opt_1) => rule_opt_1.span(),
            ParseTreeRef::RuleStar1(rule_star_1) => rule_star_1.span(),
            ParseTreeRef::RulePlus3(rule_plus_3) => rule_plus_3.span(),
            ParseTreeRef::RulePlus2(rule_plus_2) => rule_plus_2.span(),
            ParseTreeRef::PriorityLevelPlus4(prioritylevel_plus_4) => prioritylevel_plus_4.span(),
            ParseTreeRef::PriorityLevelOpt2(prioritylevel_opt_2) => prioritylevel_opt_2.span(),
            ParseTreeRef::PriorityLevelStar2(prioritylevel_star_2) => prioritylevel_star_2.span(),
            ParseTreeRef::AlternativePlus5(alternative_plus_5) => alternative_plus_5.span(),
            ParseTreeRef::AlternativeOpt3(alternative_opt_3) => alternative_opt_3.span(),
            ParseTreeRef::AlternativeStar3(alternative_star_3) => alternative_star_3.span(),
            ParseTreeRef::RegexOpt4(regex_opt_4) => regex_opt_4.span(),
            ParseTreeRef::RegexStar4(regex_star_4) => regex_star_4.span(),
            ParseTreeRef::CharClassOpt5(charclass_opt_5) => charclass_opt_5.span(),
            ParseTreeRef::CharClassAlt0(charclass_alt_0) => charclass_alt_0.span(),
            ParseTreeRef::CharClassPlus6(charclass_plus_6) => charclass_plus_6.span(),
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
impl From<Regex> for ParseTree {
    fn from(regex: Regex) -> Self {
        ParseTree::Regex(regex)
    }
}
impl From<CharClass> for ParseTree {
    fn from(charclass: CharClass) -> Self {
        ParseTree::CharClass(charclass)
    }
}
impl From<CharRange> for ParseTree {
    fn from(charrange: CharRange) -> Self {
        ParseTree::CharRange(charrange)
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
impl From<RulePlus3> for ParseTree {
    fn from(rule_plus_3: RulePlus3) -> Self {
        ParseTree::RulePlus3(rule_plus_3)
    }
}
impl From<RulePlus2> for ParseTree {
    fn from(rule_plus_2: RulePlus2) -> Self {
        ParseTree::RulePlus2(rule_plus_2)
    }
}
impl From<PriorityLevelPlus4> for ParseTree {
    fn from(prioritylevel_plus_4: PriorityLevelPlus4) -> Self {
        ParseTree::PriorityLevelPlus4(prioritylevel_plus_4)
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
impl From<AlternativePlus5> for ParseTree {
    fn from(alternative_plus_5: AlternativePlus5) -> Self {
        ParseTree::AlternativePlus5(alternative_plus_5)
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
impl From<RegexOpt4> for ParseTree {
    fn from(regex_opt_4: RegexOpt4) -> Self {
        ParseTree::RegexOpt4(regex_opt_4)
    }
}
impl From<RegexStar4> for ParseTree {
    fn from(regex_star_4: RegexStar4) -> Self {
        ParseTree::RegexStar4(regex_star_4)
    }
}
impl From<CharClassOpt5> for ParseTree {
    fn from(charclass_opt_5: CharClassOpt5) -> Self {
        ParseTree::CharClassOpt5(charclass_opt_5)
    }
}
impl From<CharClassAlt0> for ParseTree {
    fn from(charclass_alt_0: CharClassAlt0) -> Self {
        ParseTree::CharClassAlt0(charclass_alt_0)
    }
}
impl From<CharClassPlus6> for ParseTree {
    fn from(charclass_plus_6: CharClassPlus6) -> Self {
        ParseTree::CharClassPlus6(charclass_plus_6)
    }
}
trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
#[derive(Debug)]
pub struct Grammar(Token, Token, GrammarStar0, Span);
#[derive(Debug)]
pub enum Rule {
    Alt0(Token, Token, RuleStar1, Span),
    Alt1(Token, Token, Token, RulePlus2, Token, Span),
}
#[derive(Debug)]
pub struct PriorityLevel(PriorityLevelStar2, Span);
#[derive(Debug)]
pub struct Alternative(AlternativeStar3, Span);
#[derive(Debug)]
pub enum Symbol {
    Alt0(Box<Symbol>, Token, Span),
    Alt1(Box<Symbol>, Token, Span),
    Alt2(Token, Box<Symbol>, Token, Box<Symbol>, Token, Span),
    Alt3(Token, Token, Token, Span),
    Alt4(Token, Box<Symbol>, Box<Symbol>, Token, Token, Span),
    Alt5(Token, Box<Symbol>, Box<Symbol>, Token, Token, Span),
    Alt6(Token, AlternativeStar3, Token, Span),
    Alt7(Token, Span),
}
#[derive(Debug)]
pub enum Regex {
    Alt0(Box<Regex>, Token, Span),
    Alt1(Box<Regex>, Token, Span),
    Alt2(Box<Regex>, Token, Span),
    Alt3(Token, RegexStar4, Token, Span),
    Alt4(CharClass, Span),
    Alt5(Token, Span),
}
#[derive(Debug)]
pub struct CharClass(CharClassOpt5, Token, CharClassPlus6, Token, Span);
#[derive(Debug)]
pub struct CharRange(Token, Token, Token, Span);
//Rule+
#[derive(Debug)]
pub enum GrammarPlus0 {
    Alt0(Box<GrammarPlus0>, Rule, Span),
    Alt1(Rule, Span),
}
//Rule+?
#[derive(Debug)]
pub enum GrammarOpt0 {
    Alt0(GrammarPlus0, Span),
    Alt1(Span),
}
//Rule*
#[derive(Debug)]
pub struct GrammarStar0(GrammarOpt0, Span);
//{PriorityLevel ">"}+
#[derive(Debug)]
pub enum RulePlus1 {
    Alt0(Box<RulePlus1>, Token, PriorityLevel, Span),
    Alt1(PriorityLevel, Span),
}
//{PriorityLevel ">"}+?
#[derive(Debug)]
pub enum RuleOpt1 {
    Alt0(RulePlus1, Span),
    Alt1(Span),
}
//{PriorityLevel ">"}*
#[derive(Debug)]
pub struct RuleStar1(RuleOpt1, Span);
//Regex+
#[derive(Debug)]
pub enum RulePlus3 {
    Alt0(Box<RulePlus3>, Box<Regex>, Span),
    Alt1(Box<Regex>, Span),
}
//{Regex+ "|"}+
#[derive(Debug)]
pub enum RulePlus2 {
    Alt0(Box<RulePlus2>, Token, RulePlus3, Span),
    Alt1(RulePlus3, Span),
}
//{Alternative "|"}+
#[derive(Debug)]
pub enum PriorityLevelPlus4 {
    Alt0(Box<PriorityLevelPlus4>, Token, Alternative, Span),
    Alt1(Alternative, Span),
}
//{Alternative "|"}+?
#[derive(Debug)]
pub enum PriorityLevelOpt2 {
    Alt0(PriorityLevelPlus4, Span),
    Alt1(Span),
}
//{Alternative "|"}*
#[derive(Debug)]
pub struct PriorityLevelStar2(PriorityLevelOpt2, Span);
//Symbol+
#[derive(Debug)]
pub enum AlternativePlus5 {
    Alt0(Box<AlternativePlus5>, Box<Symbol>, Span),
    Alt1(Box<Symbol>, Span),
}
//Symbol+?
#[derive(Debug)]
pub enum AlternativeOpt3 {
    Alt0(AlternativePlus5, Span),
    Alt1(Span),
}
//Symbol*
#[derive(Debug)]
pub struct AlternativeStar3(AlternativeOpt3, Span);
//{Regex+ "|"}+?
#[derive(Debug)]
pub enum RegexOpt4 {
    Alt0(RulePlus2, Span),
    Alt1(Span),
}
//{Regex+ "|"}*
#[derive(Debug)]
pub struct RegexStar4(RegexOpt4, Span);
//"!"?
#[derive(Debug)]
pub enum CharClassOpt5 {
    Alt0(Token, Span),
    Alt1(Span),
}
//(CharRange | Char)
#[derive(Debug)]
pub enum CharClassAlt0 {
    Alt0(CharRange, Span),
    Alt1(Token, Span),
}
//(CharRange | Char)+
#[derive(Debug)]
pub enum CharClassPlus6 {
    Alt0(Box<CharClassPlus6>, CharClassAlt0, Span),
    Alt1(CharClassAlt0, Span),
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
    pub fn span(&self) -> Span {
        self.3
    }
}
impl Rule {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Rule::Alt0(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                _ => None,
            },
            Rule::Alt1(c0, c1, c2, c3, c4, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                3 => Some(c3.as_parse_tree_ref()),
                4 => Some(c4.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Rule::Alt0(..) => 3usize,
            Rule::Alt1(..) => 5usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Rule(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Rule::Alt0(.., span) => *span,
            Rule::Alt1(.., span) => *span,
        }
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
        match self {
            Symbol::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt1(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt2(c0, c1, c2, c3, c4, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                3 => Some(c3.as_parse_tree_ref()),
                4 => Some(c4.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt3(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt4(c0, c1, c2, c3, c4, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                3 => Some(c3.as_parse_tree_ref()),
                4 => Some(c4.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt5(c0, c1, c2, c3, c4, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                3 => Some(c3.as_parse_tree_ref()),
                4 => Some(c4.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt6(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt7(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Symbol::Alt0(..) => 2usize,
            Symbol::Alt1(..) => 2usize,
            Symbol::Alt2(..) => 5usize,
            Symbol::Alt3(..) => 3usize,
            Symbol::Alt4(..) => 5usize,
            Symbol::Alt5(..) => 5usize,
            Symbol::Alt6(..) => 3usize,
            Symbol::Alt7(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Symbol(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Symbol::Alt0(.., span) => *span,
            Symbol::Alt1(.., span) => *span,
            Symbol::Alt2(.., span) => *span,
            Symbol::Alt3(.., span) => *span,
            Symbol::Alt4(.., span) => *span,
            Symbol::Alt5(.., span) => *span,
            Symbol::Alt6(.., span) => *span,
            Symbol::Alt7(.., span) => *span,
        }
    }
}
impl Regex {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Regex::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Alt1(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Alt2(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Alt3(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Alt4(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Alt5(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Regex::Alt0(..) => 2usize,
            Regex::Alt1(..) => 2usize,
            Regex::Alt2(..) => 2usize,
            Regex::Alt3(..) => 3usize,
            Regex::Alt4(..) => 1usize,
            Regex::Alt5(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Regex(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Regex::Alt0(.., span) => *span,
            Regex::Alt1(.., span) => *span,
            Regex::Alt2(.., span) => *span,
            Regex::Alt3(.., span) => *span,
            Regex::Alt4(.., span) => *span,
            Regex::Alt5(.., span) => *span,
        }
    }
}
impl CharClass {
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
        ParseTreeRef::CharClass(self)
    }
    pub fn span(&self) -> Span {
        self.4
    }
}
impl CharRange {
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
        ParseTreeRef::CharRange(self)
    }
    pub fn span(&self) -> Span {
        self.3
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
impl RulePlus3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RulePlus3::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            RulePlus3::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RulePlus3::Alt0(..) => 2usize,
            RulePlus3::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RulePlus3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RulePlus3::Alt0(.., span) => *span,
            RulePlus3::Alt1(.., span) => *span,
        }
    }
}
impl RulePlus2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RulePlus2::Alt0(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                _ => None,
            },
            RulePlus2::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RulePlus2::Alt0(..) => 3usize,
            RulePlus2::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RulePlus2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RulePlus2::Alt0(.., span) => *span,
            RulePlus2::Alt1(.., span) => *span,
        }
    }
}
impl PriorityLevelPlus4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelPlus4::Alt0(c0, c1, c2, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                2 => Some(c2.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelPlus4::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelPlus4::Alt0(..) => 3usize,
            PriorityLevelPlus4::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelPlus4(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelPlus4::Alt0(.., span) => *span,
            PriorityLevelPlus4::Alt1(.., span) => *span,
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
impl AlternativePlus5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativePlus5::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            AlternativePlus5::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativePlus5::Alt0(..) => 2usize,
            AlternativePlus5::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativePlus5(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativePlus5::Alt0(.., span) => *span,
            AlternativePlus5::Alt1(.., span) => *span,
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
impl RegexOpt4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexOpt4::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            RegexOpt4::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexOpt4::Alt0(..) => 1usize,
            RegexOpt4::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexOpt4(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexOpt4::Alt0(.., span) => *span,
            RegexOpt4::Alt1(.., span) => *span,
        }
    }
}
impl RegexStar4 {
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
        ParseTreeRef::RegexStar4(self)
    }
    pub fn span(&self) -> Span {
        self.1
    }
}
impl CharClassOpt5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassOpt5::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            CharClassOpt5::Alt1(_) => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassOpt5::Alt0(..) => 1usize,
            CharClassOpt5::Alt1(..) => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassOpt5(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassOpt5::Alt0(.., span) => *span,
            CharClassOpt5::Alt1(.., span) => *span,
        }
    }
}
impl CharClassAlt0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassAlt0::Alt0(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
            CharClassAlt0::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassAlt0::Alt0(..) => 1usize,
            CharClassAlt0::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassAlt0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassAlt0::Alt0(.., span) => *span,
            CharClassAlt0::Alt1(.., span) => *span,
        }
    }
}
impl CharClassPlus6 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassPlus6::Alt0(c0, c1, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                1 => Some(c1.as_parse_tree_ref()),
                _ => None,
            },
            CharClassPlus6::Alt1(c0, _) => match index {
                0 => Some(c0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassPlus6::Alt0(..) => 2usize,
            CharClassPlus6::Alt1(..) => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassPlus6(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassPlus6::Alt0(.., span) => *span,
            CharClassPlus6::Alt1(.., span) => *span,
        }
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
        items.reverse();
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
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for RulePlus3 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RulePlus3::Alt0(rest, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    current = rest;
                }
                RulePlus3::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for RulePlus2 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RulePlus2::Alt0(rest, sep, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    items.push(sep.as_parse_tree_ref());
                    current = rest;
                }
                RulePlus2::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for PriorityLevelPlus4 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                PriorityLevelPlus4::Alt0(rest, sep, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    items.push(sep.as_parse_tree_ref());
                    current = rest;
                }
                PriorityLevelPlus4::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for AlternativePlus5 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                AlternativePlus5::Alt0(rest, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    current = rest;
                }
                AlternativePlus5::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for CharClassPlus6 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                CharClassPlus6::Alt0(rest, item, _) => {
                    items.push(item.as_parse_tree_ref());
                    current = rest;
                }
                CharClassPlus6::Alt1(item, _) => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
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
impl<'a> ListNode<'a> for RegexStar4 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.0 {
            RegexOpt4::Alt0(regex_opt_4, _) => regex_opt_4.iter(),
            RegexOpt4::Alt1(_) => vec![].into_iter(),
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
        //String
        TerminalId(1) => TokenKind::T1,
        //Char
        TerminalId(2) => TokenKind::T2,
        //WS
        TerminalId(3) => TokenKind::T3,
        //"grammar"
        TerminalId(4) => TokenKind::T4,
        //"="
        TerminalId(5) => TokenKind::T5,
        //">"
        TerminalId(6) => TokenKind::T6,
        //"/"
        TerminalId(7) => TokenKind::T7,
        //"|"
        TerminalId(8) => TokenKind::T8,
        //"*"
        TerminalId(9) => TokenKind::T9,
        //"+"
        TerminalId(10) => TokenKind::T10,
        //"("
        TerminalId(11) => TokenKind::T11,
        //")"
        TerminalId(12) => TokenKind::T12,
        //"""
        TerminalId(13) => TokenKind::T13,
        //"{"
        TerminalId(14) => TokenKind::T14,
        //"}"
        TerminalId(15) => TokenKind::T15,
        //"?"
        TerminalId(16) => TokenKind::T16,
        //"!"
        TerminalId(17) => TokenKind::T17,
        //"["
        TerminalId(18) => TokenKind::T18,
        //"]"
        TerminalId(19) => TokenKind::T19,
        //"-"
        TerminalId(20) => TokenKind::T20,
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
                    //Grammar : "grammar" Identifier Rule*.
                    SlotId(3) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Grammar(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_grammar_star_0(),
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
                    //Rule : Identifier "=" {PriorityLevel ">"}*.
                    SlotId(7) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Rule::Alt0(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_rule_star_1(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Rule : Identifier "=" "/" {Regex+ "|"}+ "/".
                    SlotId(13) => {
                        let [c0, c1, c2, c3, c4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Rule::Alt1(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_token(),
                            c3.unwrap_rule_plus_2(),
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
                    //PriorityLevel : {Alternative "|"}*.
                    SlotId(15) => {
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
                    //Alternative : Symbol*.
                    SlotId(17) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Alternative(c0.unwrap_alternative_star_3(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Symbol
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //Symbol : Symbol "*".
                    SlotId(20) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        Symbol::Alt0(
                            Box::new(c0.unwrap_symbol()),
                            c1.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol : Symbol "+".
                    SlotId(23) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        Symbol::Alt1(
                            Box::new(c0.unwrap_symbol()),
                            c1.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol : "(" Symbol "|" Symbol ")".
                    SlotId(29) => {
                        let [c0, c1, c2, c3, c4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Alt2(
                            c0.unwrap_token(),
                            Box::new(c1.unwrap_symbol()),
                            c2.unwrap_token(),
                            Box::new(c3.unwrap_symbol()),
                            c4.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol : """ String """.
                    SlotId(33) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Alt3(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol : "{" Symbol Symbol "}" "*".
                    SlotId(39) => {
                        let [c0, c1, c2, c3, c4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Alt4(
                            c0.unwrap_token(),
                            Box::new(c1.unwrap_symbol()),
                            Box::new(c2.unwrap_symbol()),
                            c3.unwrap_token(),
                            c4.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol : "{" Symbol Symbol "}" "+".
                    SlotId(45) => {
                        let [c0, c1, c2, c3, c4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Alt5(
                            c0.unwrap_token(),
                            Box::new(c1.unwrap_symbol()),
                            Box::new(c2.unwrap_symbol()),
                            c3.unwrap_token(),
                            c4.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol : "(" Symbol* ")".
                    SlotId(49) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Alt6(
                            c0.unwrap_token(),
                            c1.unwrap_alternative_star_3(),
                            c2.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol : Identifier.
                    SlotId(51) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Symbol::Alt7(c0.unwrap_token(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //Regex : Regex "+".
                    SlotId(54) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        Regex::Alt0(
                            Box::new(c0.unwrap_regex()),
                            c1.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Regex : Regex "*".
                    SlotId(57) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        Regex::Alt1(
                            Box::new(c0.unwrap_regex()),
                            c1.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Regex : Regex "?".
                    SlotId(60) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        Regex::Alt2(
                            Box::new(c0.unwrap_regex()),
                            c1.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Regex : "(" {Regex+ "|"}* ")".
                    SlotId(64) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Regex::Alt3(
                            c0.unwrap_token(),
                            c1.unwrap_regex_star_4(),
                            c2.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Regex : CharClass.
                    SlotId(66) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Regex::Alt4(c0.unwrap_charclass(), nonterminal_node.span).into()
                    }
                    //Regex : Char.
                    SlotId(68) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Regex::Alt5(c0.unwrap_token(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //CharClass : "!"? "[" (CharRange | Char)+ "]".
                    SlotId(73) => {
                        let [c0, c1, c2, c3] = <[ParseTree; 4usize]>::try_from(children).unwrap();
                        CharClass(
                            c0.unwrap_charclass_opt_5(),
                            c1.unwrap_token(),
                            c2.unwrap_charclass_plus_6(),
                            c3.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharRange
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //CharRange : Char "-" Char.
                    SlotId(77) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        CharRange(
                            c0.unwrap_token(),
                            c1.unwrap_token(),
                            c2.unwrap_token(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Plus_0
            NonterminalId(8) => {
                match nonterminal_node.return_slot {
                    //Rule+ : Rule+ Rule.
                    SlotId(80) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt0(
                            Box::new(c0.unwrap_grammar_plus_0()),
                            c1.unwrap_rule(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Rule+ : Rule.
                    SlotId(82) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt1(c0.unwrap_rule(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Opt_0
            NonterminalId(9) => {
                match nonterminal_node.return_slot {
                    //Rule+? : Rule+.
                    SlotId(84) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt0(c0.unwrap_grammar_plus_0(), nonterminal_node.span).into()
                    }
                    //Rule+? : .
                    SlotId(85) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Star_0
            NonterminalId(10) => {
                match nonterminal_node.return_slot {
                    //Rule* : Rule+?.
                    SlotId(87) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarStar0(c0.unwrap_grammar_opt_0(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Plus_1
            NonterminalId(11) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}+ : {PriorityLevel ">"}+ ">" PriorityLevel.
                    SlotId(91) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RulePlus1::Alt0(
                            Box::new(c0.unwrap_rule_plus_1()),
                            c1.unwrap_token(),
                            c2.unwrap_prioritylevel(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //{PriorityLevel ">"}+ : PriorityLevel.
                    SlotId(93) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RulePlus1::Alt1(c0.unwrap_prioritylevel(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Opt_1
            NonterminalId(12) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}+? : {PriorityLevel ">"}+.
                    SlotId(95) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RuleOpt1::Alt0(c0.unwrap_rule_plus_1(), nonterminal_node.span).into()
                    }
                    //{PriorityLevel ">"}+? : .
                    SlotId(96) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RuleOpt1::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Star_1
            NonterminalId(13) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}* : {PriorityLevel ">"}+?.
                    SlotId(98) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RuleStar1(c0.unwrap_rule_opt_1(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Plus_3
            NonterminalId(14) => {
                match nonterminal_node.return_slot {
                    //Regex+ : Regex+ Regex.
                    SlotId(101) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        RulePlus3::Alt0(
                            Box::new(c0.unwrap_rule_plus_3()),
                            Box::new(c1.unwrap_regex()),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Regex+ : Regex.
                    SlotId(103) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RulePlus3::Alt1(Box::new(c0.unwrap_regex()), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Rule_Plus_2
            NonterminalId(15) => {
                match nonterminal_node.return_slot {
                    //{Regex+ "|"}+ : {Regex+ "|"}+ "|" Regex+.
                    SlotId(107) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RulePlus2::Alt0(
                            Box::new(c0.unwrap_rule_plus_2()),
                            c1.unwrap_token(),
                            c2.unwrap_rule_plus_3(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //{Regex+ "|"}+ : Regex+.
                    SlotId(109) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RulePlus2::Alt1(c0.unwrap_rule_plus_3(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Plus_4
            NonterminalId(16) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}+ : {Alternative "|"}+ "|" Alternative.
                    SlotId(113) => {
                        let [c0, c1, c2] = <[ParseTree; 3usize]>::try_from(children).unwrap();
                        PriorityLevelPlus4::Alt0(
                            Box::new(c0.unwrap_prioritylevel_plus_4()),
                            c1.unwrap_token(),
                            c2.unwrap_alternative(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //{Alternative "|"}+ : Alternative.
                    SlotId(115) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelPlus4::Alt1(c0.unwrap_alternative(), nonterminal_node.span)
                            .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Opt_2
            NonterminalId(17) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}+? : {Alternative "|"}+.
                    SlotId(117) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelOpt2::Alt0(
                            c0.unwrap_prioritylevel_plus_4(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //{Alternative "|"}+? : .
                    SlotId(118) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        PriorityLevelOpt2::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Star_2
            NonterminalId(18) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}* : {Alternative "|"}+?.
                    SlotId(120) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelStar2(c0.unwrap_prioritylevel_opt_2(), nonterminal_node.span)
                            .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Plus_5
            NonterminalId(19) => {
                match nonterminal_node.return_slot {
                    //Symbol+ : Symbol+ Symbol.
                    SlotId(123) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        AlternativePlus5::Alt0(
                            Box::new(c0.unwrap_alternative_plus_5()),
                            Box::new(c1.unwrap_symbol()),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //Symbol+ : Symbol.
                    SlotId(125) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativePlus5::Alt1(Box::new(c0.unwrap_symbol()), nonterminal_node.span)
                            .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Opt_3
            NonterminalId(20) => {
                match nonterminal_node.return_slot {
                    //Symbol+? : Symbol+.
                    SlotId(127) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeOpt3::Alt0(c0.unwrap_alternative_plus_5(), nonterminal_node.span)
                            .into()
                    }
                    //Symbol+? : .
                    SlotId(128) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        AlternativeOpt3::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Star_3
            NonterminalId(21) => {
                match nonterminal_node.return_slot {
                    //Symbol* : Symbol+?.
                    SlotId(130) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeStar3(c0.unwrap_alternative_opt_3(), nonterminal_node.span)
                            .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex_Opt_4
            NonterminalId(22) => {
                match nonterminal_node.return_slot {
                    //{Regex+ "|"}+? : {Regex+ "|"}+.
                    SlotId(132) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexOpt4::Alt0(c0.unwrap_rule_plus_2(), nonterminal_node.span).into()
                    }
                    //{Regex+ "|"}+? : .
                    SlotId(133) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RegexOpt4::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex_Star_4
            NonterminalId(23) => {
                match nonterminal_node.return_slot {
                    //{Regex+ "|"}* : {Regex+ "|"}+?.
                    SlotId(135) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexStar4(c0.unwrap_regex_opt_4(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Opt_5
            NonterminalId(24) => {
                match nonterminal_node.return_slot {
                    //"!"? : "!".
                    SlotId(137) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassOpt5::Alt0(c0.unwrap_token(), nonterminal_node.span).into()
                    }
                    //"!"? : .
                    SlotId(138) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        CharClassOpt5::Alt1(nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Alt_0
            NonterminalId(25) => {
                match nonterminal_node.return_slot {
                    //(CharRange | Char) : CharRange.
                    SlotId(140) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassAlt0::Alt0(c0.unwrap_charrange(), nonterminal_node.span).into()
                    }
                    //(CharRange | Char) : Char.
                    SlotId(142) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassAlt0::Alt1(c0.unwrap_token(), nonterminal_node.span).into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Plus_6
            NonterminalId(26) => {
                match nonterminal_node.return_slot {
                    //(CharRange | Char)+ : (CharRange | Char)+ (CharRange | Char).
                    SlotId(145) => {
                        let [c0, c1] = <[ParseTree; 2usize]>::try_from(children).unwrap();
                        CharClassPlus6::Alt0(
                            Box::new(c0.unwrap_charclass_plus_6()),
                            c1.unwrap_charclass_alt_0(),
                            nonterminal_node.span,
                        )
                        .into()
                    }
                    //(CharRange | Char)+ : (CharRange | Char).
                    SlotId(147) => {
                        let [c0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassPlus6::Alt1(c0.unwrap_charclass_alt_0(), nonterminal_node.span)
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
        "Regex" => ParseTree::Regex(create_parse_tree_regex(root_id, parser, builder)),
        "CharClass" => ParseTree::CharClass(create_parse_tree_charclass(root_id, parser, builder)),
        "CharRange" => ParseTree::CharRange(create_parse_tree_charrange(root_id, parser, builder)),
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
        "Rule_Plus_3" => {
            ParseTree::RulePlus3(create_parse_tree_rule_plus_3(root_id, parser, builder))
        }
        "Rule_Plus_2" => {
            ParseTree::RulePlus2(create_parse_tree_rule_plus_2(root_id, parser, builder))
        }
        "PriorityLevel_Plus_4" => ParseTree::PriorityLevelPlus4(
            create_parse_tree_prioritylevel_plus_4(root_id, parser, builder),
        ),
        "PriorityLevel_Opt_2" => ParseTree::PriorityLevelOpt2(
            create_parse_tree_prioritylevel_opt_2(root_id, parser, builder),
        ),
        "PriorityLevel_Star_2" => ParseTree::PriorityLevelStar2(
            create_parse_tree_prioritylevel_star_2(root_id, parser, builder),
        ),
        "Alternative_Plus_5" => ParseTree::AlternativePlus5(create_parse_tree_alternative_plus_5(
            root_id, parser, builder,
        )),
        "Alternative_Opt_3" => ParseTree::AlternativeOpt3(create_parse_tree_alternative_opt_3(
            root_id, parser, builder,
        )),
        "Alternative_Star_3" => ParseTree::AlternativeStar3(create_parse_tree_alternative_star_3(
            root_id, parser, builder,
        )),
        "Regex_Opt_4" => {
            ParseTree::RegexOpt4(create_parse_tree_regex_opt_4(root_id, parser, builder))
        }
        "Regex_Star_4" => {
            ParseTree::RegexStar4(create_parse_tree_regex_star_4(root_id, parser, builder))
        }
        "CharClass_Opt_5" => {
            ParseTree::CharClassOpt5(create_parse_tree_charclass_opt_5(root_id, parser, builder))
        }
        "CharClass_Alt_0" => {
            ParseTree::CharClassAlt0(create_parse_tree_charclass_alt_0(root_id, parser, builder))
        }
        "CharClass_Plus_6" => {
            ParseTree::CharClassPlus6(create_parse_tree_charclass_plus_6(root_id, parser, builder))
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
pub fn create_parse_tree_regex(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Regex {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex()
}
pub fn create_parse_tree_charclass(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClass {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_charclass()
}
pub fn create_parse_tree_charrange(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharRange {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_charrange()
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
pub fn create_parse_tree_rule_plus_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RulePlus3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_rule_plus_3()
}
pub fn create_parse_tree_rule_plus_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RulePlus2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_rule_plus_2()
}
pub fn create_parse_tree_prioritylevel_plus_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelPlus4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_prioritylevel_plus_4()
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
pub fn create_parse_tree_alternative_plus_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativePlus5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_plus_5()
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
pub fn create_parse_tree_regex_opt_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexOpt4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_opt_4()
}
pub fn create_parse_tree_regex_star_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexStar4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_star_4()
}
pub fn create_parse_tree_charclass_opt_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassOpt5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_charclass_opt_5()
}
pub fn create_parse_tree_charclass_alt_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassAlt0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_charclass_alt_0()
}
pub fn create_parse_tree_charclass_plus_6(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassPlus6 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_charclass_plus_6()
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
