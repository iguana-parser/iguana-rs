use crate::parser::IggyParser;
use core::fmt;
use iguana_runtime::{
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
    //RangeChar
    T2,
    //Char
    T3,
    //WS
    T4,
    //"grammar"
    T5,
    //"="
    T6,
    //">"
    T7,
    //"regex"
    T8,
    //"{"
    T9,
    //"}"
    T10,
    //"|"
    T11,
    //"*"
    T12,
    //"+"
    T13,
    //"?"
    T14,
    //"("
    T15,
    //")"
    T16,
    //"""
    T17,
    //"!"
    T18,
    //"["
    T19,
    //"]"
    T20,
    //"-"
    T21,
    //Layout
    T22,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Identifier",
            TokenKind::T1 => "String",
            TokenKind::T2 => "RangeChar",
            TokenKind::T3 => "Char",
            TokenKind::T4 => "WS",
            TokenKind::T5 => "\"grammar\"",
            TokenKind::T6 => "\"=\"",
            TokenKind::T7 => "\">\"",
            TokenKind::T8 => "\"regex\"",
            TokenKind::T9 => "\"{\"",
            TokenKind::T10 => "\"}\"",
            TokenKind::T11 => "\"|\"",
            TokenKind::T12 => "\"*\"",
            TokenKind::T13 => "\"+\"",
            TokenKind::T14 => "\"?\"",
            TokenKind::T15 => "\"(\"",
            TokenKind::T16 => "\")\"",
            TokenKind::T17 => "\"\"\"",
            TokenKind::T18 => "\"!\"",
            TokenKind::T19 => "\"[\"",
            TokenKind::T20 => "\"]\"",
            TokenKind::T21 => "\"-\"",
            TokenKind::T22 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    //Grammar
    Grammar(Grammar),
    //SyntaxRule
    SyntaxRule(SyntaxRule),
    //RegexBlock
    RegexBlock(RegexBlock),
    //RegexRule
    RegexRule(RegexRule),
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
    //Range
    Range(Range),
    //SyntaxRule+
    GrammarPlus0(GrammarPlus0),
    //SyntaxRule+?
    GrammarOpt0(GrammarOpt0),
    //SyntaxRule*
    GrammarStar0(GrammarStar0),
    //RegexBlock?
    GrammarOpt1(GrammarOpt1),
    //{PriorityLevel ">"}+
    SyntaxRulePlus1(SyntaxRulePlus1),
    //{PriorityLevel ">"}+?
    SyntaxRuleOpt2(SyntaxRuleOpt2),
    //{PriorityLevel ">"}*
    SyntaxRuleStar1(SyntaxRuleStar1),
    //RegexRule+
    RegexBlockPlus2(RegexBlockPlus2),
    //RegexRule+?
    RegexBlockOpt3(RegexBlockOpt3),
    //RegexRule*
    RegexBlockStar2(RegexBlockStar2),
    //Regex+
    RegexRulePlus4(RegexRulePlus4),
    //{Regex+ "|"}+
    RegexRulePlus3(RegexRulePlus3),
    //{Alternative "|"}+
    PriorityLevelPlus5(PriorityLevelPlus5),
    //{Alternative "|"}+?
    PriorityLevelOpt4(PriorityLevelOpt4),
    //{Alternative "|"}*
    PriorityLevelStar3(PriorityLevelStar3),
    //Symbol+
    AlternativePlus6(AlternativePlus6),
    //Symbol+?
    AlternativeOpt5(AlternativeOpt5),
    //Symbol*
    AlternativeStar4(AlternativeStar4),
    //{Regex+ "|"}+?
    RegexOpt6(RegexOpt6),
    //{Regex+ "|"}*
    RegexStar5(RegexStar5),
    //"!"?
    CharClassOpt7(CharClassOpt7),
    //(Range | RangeChar)
    CharClassAlt0(CharClassAlt0),
    //(Range | RangeChar)+
    CharClassPlus7(CharClassPlus7),
    //StartGrammar
    StartGrammar(StartGrammar),
    //StartSyntaxRule
    StartSyntaxRule(StartSyntaxRule),
    //StartRegexBlock
    StartRegexBlock(StartRegexBlock),
    //StartRegexRule
    StartRegexRule(StartRegexRule),
    //StartPriorityLevel
    StartPriorityLevel(StartPriorityLevel),
    //StartAlternative
    StartAlternative(StartAlternative),
    //StartSymbol
    StartSymbol(StartSymbol),
    //StartRegex
    StartRegex(StartRegex),
    //StartCharClass
    StartCharClass(StartCharClass),
    //StartRange
    StartRange(StartRange),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::Grammar(grammar) => grammar.as_parse_tree_ref(),
            ParseTree::SyntaxRule(syntax_rule) => syntax_rule.as_parse_tree_ref(),
            ParseTree::RegexBlock(regex_block) => regex_block.as_parse_tree_ref(),
            ParseTree::RegexRule(regex_rule) => regex_rule.as_parse_tree_ref(),
            ParseTree::PriorityLevel(priority_level) => priority_level.as_parse_tree_ref(),
            ParseTree::Alternative(alternative) => alternative.as_parse_tree_ref(),
            ParseTree::Symbol(symbol) => symbol.as_parse_tree_ref(),
            ParseTree::Regex(regex) => regex.as_parse_tree_ref(),
            ParseTree::CharClass(char_class) => char_class.as_parse_tree_ref(),
            ParseTree::Range(range) => range.as_parse_tree_ref(),
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0.as_parse_tree_ref(),
            ParseTree::GrammarOpt0(grammar_opt_0) => grammar_opt_0.as_parse_tree_ref(),
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0.as_parse_tree_ref(),
            ParseTree::GrammarOpt1(grammar_opt_1) => grammar_opt_1.as_parse_tree_ref(),
            ParseTree::SyntaxRulePlus1(syntax_rule_plus_1) => {
                syntax_rule_plus_1.as_parse_tree_ref()
            }
            ParseTree::SyntaxRuleOpt2(syntax_rule_opt_2) => syntax_rule_opt_2.as_parse_tree_ref(),
            ParseTree::SyntaxRuleStar1(syntax_rule_star_1) => {
                syntax_rule_star_1.as_parse_tree_ref()
            }
            ParseTree::RegexBlockPlus2(regex_block_plus_2) => {
                regex_block_plus_2.as_parse_tree_ref()
            }
            ParseTree::RegexBlockOpt3(regex_block_opt_3) => regex_block_opt_3.as_parse_tree_ref(),
            ParseTree::RegexBlockStar2(regex_block_star_2) => {
                regex_block_star_2.as_parse_tree_ref()
            }
            ParseTree::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.as_parse_tree_ref(),
            ParseTree::RegexRulePlus3(regex_rule_plus_3) => regex_rule_plus_3.as_parse_tree_ref(),
            ParseTree::PriorityLevelPlus5(priority_level_plus_5) => {
                priority_level_plus_5.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelOpt4(priority_level_opt_4) => {
                priority_level_opt_4.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelStar3(priority_level_star_3) => {
                priority_level_star_3.as_parse_tree_ref()
            }
            ParseTree::AlternativePlus6(alternative_plus_6) => {
                alternative_plus_6.as_parse_tree_ref()
            }
            ParseTree::AlternativeOpt5(alternative_opt_5) => alternative_opt_5.as_parse_tree_ref(),
            ParseTree::AlternativeStar4(alternative_star_4) => {
                alternative_star_4.as_parse_tree_ref()
            }
            ParseTree::RegexOpt6(regex_opt_6) => regex_opt_6.as_parse_tree_ref(),
            ParseTree::RegexStar5(regex_star_5) => regex_star_5.as_parse_tree_ref(),
            ParseTree::CharClassOpt7(char_class_opt_7) => char_class_opt_7.as_parse_tree_ref(),
            ParseTree::CharClassAlt0(char_class_alt_0) => char_class_alt_0.as_parse_tree_ref(),
            ParseTree::CharClassPlus7(char_class_plus_7) => char_class_plus_7.as_parse_tree_ref(),
            ParseTree::StartGrammar(start_grammar) => start_grammar.as_parse_tree_ref(),
            ParseTree::StartSyntaxRule(start_syntax_rule) => start_syntax_rule.as_parse_tree_ref(),
            ParseTree::StartRegexBlock(start_regex_block) => start_regex_block.as_parse_tree_ref(),
            ParseTree::StartRegexRule(start_regex_rule) => start_regex_rule.as_parse_tree_ref(),
            ParseTree::StartPriorityLevel(start_priority_level) => {
                start_priority_level.as_parse_tree_ref()
            }
            ParseTree::StartAlternative(start_alternative) => start_alternative.as_parse_tree_ref(),
            ParseTree::StartSymbol(start_symbol) => start_symbol.as_parse_tree_ref(),
            ParseTree::StartRegex(start_regex) => start_regex.as_parse_tree_ref(),
            ParseTree::StartCharClass(start_char_class) => start_char_class.as_parse_tree_ref(),
            ParseTree::StartRange(start_range) => start_range.as_parse_tree_ref(),
            ParseTree::Token(token) => token.as_parse_tree_ref(),
        }
    }
    fn unwrap_grammar(self) -> Grammar {
        match self {
            ParseTree::Grammar(grammar) => grammar,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule(self) -> SyntaxRule {
        match self {
            ParseTree::SyntaxRule(syntax_rule) => syntax_rule,
            _ => panic!(),
        }
    }
    fn unwrap_regex_block(self) -> RegexBlock {
        match self {
            ParseTree::RegexBlock(regex_block) => regex_block,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule(self) -> RegexRule {
        match self {
            ParseTree::RegexRule(regex_rule) => regex_rule,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level(self) -> PriorityLevel {
        match self {
            ParseTree::PriorityLevel(priority_level) => priority_level,
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
    fn unwrap_char_class(self) -> CharClass {
        match self {
            ParseTree::CharClass(char_class) => char_class,
            _ => panic!(),
        }
    }
    fn unwrap_range(self) -> Range {
        match self {
            ParseTree::Range(range) => range,
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
    fn unwrap_grammar_opt_1(self) -> GrammarOpt1 {
        match self {
            ParseTree::GrammarOpt1(grammar_opt_1) => grammar_opt_1,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule_plus_1(self) -> SyntaxRulePlus1 {
        match self {
            ParseTree::SyntaxRulePlus1(syntax_rule_plus_1) => syntax_rule_plus_1,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule_opt_2(self) -> SyntaxRuleOpt2 {
        match self {
            ParseTree::SyntaxRuleOpt2(syntax_rule_opt_2) => syntax_rule_opt_2,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule_star_1(self) -> SyntaxRuleStar1 {
        match self {
            ParseTree::SyntaxRuleStar1(syntax_rule_star_1) => syntax_rule_star_1,
            _ => panic!(),
        }
    }
    fn unwrap_regex_block_plus_2(self) -> RegexBlockPlus2 {
        match self {
            ParseTree::RegexBlockPlus2(regex_block_plus_2) => regex_block_plus_2,
            _ => panic!(),
        }
    }
    fn unwrap_regex_block_opt_3(self) -> RegexBlockOpt3 {
        match self {
            ParseTree::RegexBlockOpt3(regex_block_opt_3) => regex_block_opt_3,
            _ => panic!(),
        }
    }
    fn unwrap_regex_block_star_2(self) -> RegexBlockStar2 {
        match self {
            ParseTree::RegexBlockStar2(regex_block_star_2) => regex_block_star_2,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_plus_4(self) -> RegexRulePlus4 {
        match self {
            ParseTree::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_plus_3(self) -> RegexRulePlus3 {
        match self {
            ParseTree::RegexRulePlus3(regex_rule_plus_3) => regex_rule_plus_3,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level_plus_5(self) -> PriorityLevelPlus5 {
        match self {
            ParseTree::PriorityLevelPlus5(priority_level_plus_5) => priority_level_plus_5,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level_opt_4(self) -> PriorityLevelOpt4 {
        match self {
            ParseTree::PriorityLevelOpt4(priority_level_opt_4) => priority_level_opt_4,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level_star_3(self) -> PriorityLevelStar3 {
        match self {
            ParseTree::PriorityLevelStar3(priority_level_star_3) => priority_level_star_3,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_plus_6(self) -> AlternativePlus6 {
        match self {
            ParseTree::AlternativePlus6(alternative_plus_6) => alternative_plus_6,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_opt_5(self) -> AlternativeOpt5 {
        match self {
            ParseTree::AlternativeOpt5(alternative_opt_5) => alternative_opt_5,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_star_4(self) -> AlternativeStar4 {
        match self {
            ParseTree::AlternativeStar4(alternative_star_4) => alternative_star_4,
            _ => panic!(),
        }
    }
    fn unwrap_regex_opt_6(self) -> RegexOpt6 {
        match self {
            ParseTree::RegexOpt6(regex_opt_6) => regex_opt_6,
            _ => panic!(),
        }
    }
    fn unwrap_regex_star_5(self) -> RegexStar5 {
        match self {
            ParseTree::RegexStar5(regex_star_5) => regex_star_5,
            _ => panic!(),
        }
    }
    fn unwrap_char_class_opt_7(self) -> CharClassOpt7 {
        match self {
            ParseTree::CharClassOpt7(char_class_opt_7) => char_class_opt_7,
            _ => panic!(),
        }
    }
    fn unwrap_char_class_alt_0(self) -> CharClassAlt0 {
        match self {
            ParseTree::CharClassAlt0(char_class_alt_0) => char_class_alt_0,
            _ => panic!(),
        }
    }
    fn unwrap_char_class_plus_7(self) -> CharClassPlus7 {
        match self {
            ParseTree::CharClassPlus7(char_class_plus_7) => char_class_plus_7,
            _ => panic!(),
        }
    }
    fn unwrap_start_grammar(self) -> StartGrammar {
        match self {
            ParseTree::StartGrammar(start_grammar) => start_grammar,
            _ => panic!(),
        }
    }
    fn unwrap_start_syntax_rule(self) -> StartSyntaxRule {
        match self {
            ParseTree::StartSyntaxRule(start_syntax_rule) => start_syntax_rule,
            _ => panic!(),
        }
    }
    fn unwrap_start_regex_block(self) -> StartRegexBlock {
        match self {
            ParseTree::StartRegexBlock(start_regex_block) => start_regex_block,
            _ => panic!(),
        }
    }
    fn unwrap_start_regex_rule(self) -> StartRegexRule {
        match self {
            ParseTree::StartRegexRule(start_regex_rule) => start_regex_rule,
            _ => panic!(),
        }
    }
    fn unwrap_start_priority_level(self) -> StartPriorityLevel {
        match self {
            ParseTree::StartPriorityLevel(start_priority_level) => start_priority_level,
            _ => panic!(),
        }
    }
    fn unwrap_start_alternative(self) -> StartAlternative {
        match self {
            ParseTree::StartAlternative(start_alternative) => start_alternative,
            _ => panic!(),
        }
    }
    fn unwrap_start_symbol(self) -> StartSymbol {
        match self {
            ParseTree::StartSymbol(start_symbol) => start_symbol,
            _ => panic!(),
        }
    }
    fn unwrap_start_regex(self) -> StartRegex {
        match self {
            ParseTree::StartRegex(start_regex) => start_regex,
            _ => panic!(),
        }
    }
    fn unwrap_start_char_class(self) -> StartCharClass {
        match self {
            ParseTree::StartCharClass(start_char_class) => start_char_class,
            _ => panic!(),
        }
    }
    fn unwrap_start_range(self) -> StartRange {
        match self {
            ParseTree::StartRange(start_range) => start_range,
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
    SyntaxRule(&'a SyntaxRule),
    RegexBlock(&'a RegexBlock),
    RegexRule(&'a RegexRule),
    PriorityLevel(&'a PriorityLevel),
    Alternative(&'a Alternative),
    Symbol(&'a Symbol),
    Regex(&'a Regex),
    CharClass(&'a CharClass),
    Range(&'a Range),
    GrammarPlus0(&'a GrammarPlus0),
    GrammarOpt0(&'a GrammarOpt0),
    GrammarStar0(&'a GrammarStar0),
    GrammarOpt1(&'a GrammarOpt1),
    SyntaxRulePlus1(&'a SyntaxRulePlus1),
    SyntaxRuleOpt2(&'a SyntaxRuleOpt2),
    SyntaxRuleStar1(&'a SyntaxRuleStar1),
    RegexBlockPlus2(&'a RegexBlockPlus2),
    RegexBlockOpt3(&'a RegexBlockOpt3),
    RegexBlockStar2(&'a RegexBlockStar2),
    RegexRulePlus4(&'a RegexRulePlus4),
    RegexRulePlus3(&'a RegexRulePlus3),
    PriorityLevelPlus5(&'a PriorityLevelPlus5),
    PriorityLevelOpt4(&'a PriorityLevelOpt4),
    PriorityLevelStar3(&'a PriorityLevelStar3),
    AlternativePlus6(&'a AlternativePlus6),
    AlternativeOpt5(&'a AlternativeOpt5),
    AlternativeStar4(&'a AlternativeStar4),
    RegexOpt6(&'a RegexOpt6),
    RegexStar5(&'a RegexStar5),
    CharClassOpt7(&'a CharClassOpt7),
    CharClassAlt0(&'a CharClassAlt0),
    CharClassPlus7(&'a CharClassPlus7),
    StartGrammar(&'a StartGrammar),
    StartSyntaxRule(&'a StartSyntaxRule),
    StartRegexBlock(&'a StartRegexBlock),
    StartRegexRule(&'a StartRegexRule),
    StartPriorityLevel(&'a StartPriorityLevel),
    StartAlternative(&'a StartAlternative),
    StartSymbol(&'a StartSymbol),
    StartRegex(&'a StartRegex),
    StartCharClass(&'a StartCharClass),
    StartRange(&'a StartRange),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::Grammar(grammar) => (0..grammar.child_count())
                .filter_map(|i| grammar.child(i))
                .collect(),
            ParseTreeRef::SyntaxRule(syntax_rule) => (0..syntax_rule.child_count())
                .filter_map(|i| syntax_rule.child(i))
                .collect(),
            ParseTreeRef::RegexBlock(regex_block) => (0..regex_block.child_count())
                .filter_map(|i| regex_block.child(i))
                .collect(),
            ParseTreeRef::RegexRule(regex_rule) => (0..regex_rule.child_count())
                .filter_map(|i| regex_rule.child(i))
                .collect(),
            ParseTreeRef::PriorityLevel(priority_level) => (0..priority_level.child_count())
                .filter_map(|i| priority_level.child(i))
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
            ParseTreeRef::CharClass(char_class) => (0..char_class.child_count())
                .filter_map(|i| char_class.child(i))
                .collect(),
            ParseTreeRef::Range(range) => (0..range.child_count())
                .filter_map(|i| range.child(i))
                .collect(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.iter().collect(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => (0..grammar_opt_0.child_count())
                .filter_map(|i| grammar_opt_0.child(i))
                .collect(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.iter().collect(),
            ParseTreeRef::GrammarOpt1(grammar_opt_1) => (0..grammar_opt_1.child_count())
                .filter_map(|i| grammar_opt_1.child(i))
                .collect(),
            ParseTreeRef::SyntaxRulePlus1(syntax_rule_plus_1) => {
                syntax_rule_plus_1.iter().collect()
            }
            ParseTreeRef::SyntaxRuleOpt2(syntax_rule_opt_2) => (0..syntax_rule_opt_2.child_count())
                .filter_map(|i| syntax_rule_opt_2.child(i))
                .collect(),
            ParseTreeRef::SyntaxRuleStar1(syntax_rule_star_1) => {
                syntax_rule_star_1.iter().collect()
            }
            ParseTreeRef::RegexBlockPlus2(regex_block_plus_2) => {
                regex_block_plus_2.iter().collect()
            }
            ParseTreeRef::RegexBlockOpt3(regex_block_opt_3) => (0..regex_block_opt_3.child_count())
                .filter_map(|i| regex_block_opt_3.child(i))
                .collect(),
            ParseTreeRef::RegexBlockStar2(regex_block_star_2) => {
                regex_block_star_2.iter().collect()
            }
            ParseTreeRef::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.iter().collect(),
            ParseTreeRef::RegexRulePlus3(regex_rule_plus_3) => regex_rule_plus_3.iter().collect(),
            ParseTreeRef::PriorityLevelPlus5(priority_level_plus_5) => {
                priority_level_plus_5.iter().collect()
            }
            ParseTreeRef::PriorityLevelOpt4(priority_level_opt_4) => (0..priority_level_opt_4
                .child_count())
                .filter_map(|i| priority_level_opt_4.child(i))
                .collect(),
            ParseTreeRef::PriorityLevelStar3(priority_level_star_3) => {
                priority_level_star_3.iter().collect()
            }
            ParseTreeRef::AlternativePlus6(alternative_plus_6) => {
                alternative_plus_6.iter().collect()
            }
            ParseTreeRef::AlternativeOpt5(alternative_opt_5) => (0..alternative_opt_5
                .child_count())
                .filter_map(|i| alternative_opt_5.child(i))
                .collect(),
            ParseTreeRef::AlternativeStar4(alternative_star_4) => {
                alternative_star_4.iter().collect()
            }
            ParseTreeRef::RegexOpt6(regex_opt_6) => (0..regex_opt_6.child_count())
                .filter_map(|i| regex_opt_6.child(i))
                .collect(),
            ParseTreeRef::RegexStar5(regex_star_5) => regex_star_5.iter().collect(),
            ParseTreeRef::CharClassOpt7(char_class_opt_7) => (0..char_class_opt_7.child_count())
                .filter_map(|i| char_class_opt_7.child(i))
                .collect(),
            ParseTreeRef::CharClassAlt0(char_class_alt_0) => (0..char_class_alt_0.child_count())
                .filter_map(|i| char_class_alt_0.child(i))
                .collect(),
            ParseTreeRef::CharClassPlus7(char_class_plus_7) => char_class_plus_7.iter().collect(),
            ParseTreeRef::StartGrammar(start_grammar) => (0..start_grammar.child_count())
                .filter_map(|i| start_grammar.child(i))
                .collect(),
            ParseTreeRef::StartSyntaxRule(start_syntax_rule) => (0..start_syntax_rule
                .child_count())
                .filter_map(|i| start_syntax_rule.child(i))
                .collect(),
            ParseTreeRef::StartRegexBlock(start_regex_block) => (0..start_regex_block
                .child_count())
                .filter_map(|i| start_regex_block.child(i))
                .collect(),
            ParseTreeRef::StartRegexRule(start_regex_rule) => (0..start_regex_rule.child_count())
                .filter_map(|i| start_regex_rule.child(i))
                .collect(),
            ParseTreeRef::StartPriorityLevel(start_priority_level) => (0..start_priority_level
                .child_count())
                .filter_map(|i| start_priority_level.child(i))
                .collect(),
            ParseTreeRef::StartAlternative(start_alternative) => (0..start_alternative
                .child_count())
                .filter_map(|i| start_alternative.child(i))
                .collect(),
            ParseTreeRef::StartSymbol(start_symbol) => (0..start_symbol.child_count())
                .filter_map(|i| start_symbol.child(i))
                .collect(),
            ParseTreeRef::StartRegex(start_regex) => (0..start_regex.child_count())
                .filter_map(|i| start_regex.child(i))
                .collect(),
            ParseTreeRef::StartCharClass(start_char_class) => (0..start_char_class.child_count())
                .filter_map(|i| start_char_class.child(i))
                .collect(),
            ParseTreeRef::StartRange(start_range) => (0..start_range.child_count())
                .filter_map(|i| start_range.child(i))
                .collect(),
            ParseTreeRef::Token(_) => vec![],
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            ParseTreeRef::Grammar(_) => "Grammar",
            ParseTreeRef::SyntaxRule(_) => "SyntaxRule",
            ParseTreeRef::RegexBlock(_) => "RegexBlock",
            ParseTreeRef::RegexRule(_) => "RegexRule",
            ParseTreeRef::PriorityLevel(_) => "PriorityLevel",
            ParseTreeRef::Alternative(_) => "Alternative",
            ParseTreeRef::Symbol(_) => "Symbol",
            ParseTreeRef::Regex(_) => "Regex",
            ParseTreeRef::CharClass(_) => "CharClass",
            ParseTreeRef::Range(_) => "Range",
            ParseTreeRef::GrammarPlus0(_) => "SyntaxRule+",
            ParseTreeRef::GrammarOpt0(_) => "SyntaxRule+?",
            ParseTreeRef::GrammarStar0(_) => "SyntaxRule*",
            ParseTreeRef::GrammarOpt1(_) => "RegexBlock?",
            ParseTreeRef::SyntaxRulePlus1(_) => "{PriorityLevel \">\"}+",
            ParseTreeRef::SyntaxRuleOpt2(_) => "{PriorityLevel \">\"}+?",
            ParseTreeRef::SyntaxRuleStar1(_) => "{PriorityLevel \">\"}*",
            ParseTreeRef::RegexBlockPlus2(_) => "RegexRule+",
            ParseTreeRef::RegexBlockOpt3(_) => "RegexRule+?",
            ParseTreeRef::RegexBlockStar2(_) => "RegexRule*",
            ParseTreeRef::RegexRulePlus4(_) => "Regex+",
            ParseTreeRef::RegexRulePlus3(_) => "{Regex+ \"|\"}+",
            ParseTreeRef::PriorityLevelPlus5(_) => "{Alternative \"|\"}+",
            ParseTreeRef::PriorityLevelOpt4(_) => "{Alternative \"|\"}+?",
            ParseTreeRef::PriorityLevelStar3(_) => "{Alternative \"|\"}*",
            ParseTreeRef::AlternativePlus6(_) => "Symbol+",
            ParseTreeRef::AlternativeOpt5(_) => "Symbol+?",
            ParseTreeRef::AlternativeStar4(_) => "Symbol*",
            ParseTreeRef::RegexOpt6(_) => "{Regex+ \"|\"}+?",
            ParseTreeRef::RegexStar5(_) => "{Regex+ \"|\"}*",
            ParseTreeRef::CharClassOpt7(_) => "\"!\"?",
            ParseTreeRef::CharClassAlt0(_) => "(Range | RangeChar)",
            ParseTreeRef::CharClassPlus7(_) => "(Range | RangeChar)+",
            ParseTreeRef::StartGrammar(_) => "StartGrammar",
            ParseTreeRef::StartSyntaxRule(_) => "StartSyntaxRule",
            ParseTreeRef::StartRegexBlock(_) => "StartRegexBlock",
            ParseTreeRef::StartRegexRule(_) => "StartRegexRule",
            ParseTreeRef::StartPriorityLevel(_) => "StartPriorityLevel",
            ParseTreeRef::StartAlternative(_) => "StartAlternative",
            ParseTreeRef::StartSymbol(_) => "StartSymbol",
            ParseTreeRef::StartRegex(_) => "StartRegex",
            ParseTreeRef::StartCharClass(_) => "StartCharClass",
            ParseTreeRef::StartRange(_) => "StartRange",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.child_count(),
            ParseTreeRef::SyntaxRule(syntax_rule) => syntax_rule.child_count(),
            ParseTreeRef::RegexBlock(regex_block) => regex_block.child_count(),
            ParseTreeRef::RegexRule(regex_rule) => regex_rule.child_count(),
            ParseTreeRef::PriorityLevel(priority_level) => priority_level.child_count(),
            ParseTreeRef::Alternative(alternative) => alternative.child_count(),
            ParseTreeRef::Symbol(symbol) => symbol.child_count(),
            ParseTreeRef::Regex(regex) => regex.child_count(),
            ParseTreeRef::CharClass(char_class) => char_class.child_count(),
            ParseTreeRef::Range(range) => range.child_count(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.child_count(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.child_count(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.child_count(),
            ParseTreeRef::GrammarOpt1(grammar_opt_1) => grammar_opt_1.child_count(),
            ParseTreeRef::SyntaxRulePlus1(syntax_rule_plus_1) => syntax_rule_plus_1.child_count(),
            ParseTreeRef::SyntaxRuleOpt2(syntax_rule_opt_2) => syntax_rule_opt_2.child_count(),
            ParseTreeRef::SyntaxRuleStar1(syntax_rule_star_1) => syntax_rule_star_1.child_count(),
            ParseTreeRef::RegexBlockPlus2(regex_block_plus_2) => regex_block_plus_2.child_count(),
            ParseTreeRef::RegexBlockOpt3(regex_block_opt_3) => regex_block_opt_3.child_count(),
            ParseTreeRef::RegexBlockStar2(regex_block_star_2) => regex_block_star_2.child_count(),
            ParseTreeRef::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.child_count(),
            ParseTreeRef::RegexRulePlus3(regex_rule_plus_3) => regex_rule_plus_3.child_count(),
            ParseTreeRef::PriorityLevelPlus5(priority_level_plus_5) => {
                priority_level_plus_5.child_count()
            }
            ParseTreeRef::PriorityLevelOpt4(priority_level_opt_4) => {
                priority_level_opt_4.child_count()
            }
            ParseTreeRef::PriorityLevelStar3(priority_level_star_3) => {
                priority_level_star_3.child_count()
            }
            ParseTreeRef::AlternativePlus6(alternative_plus_6) => alternative_plus_6.child_count(),
            ParseTreeRef::AlternativeOpt5(alternative_opt_5) => alternative_opt_5.child_count(),
            ParseTreeRef::AlternativeStar4(alternative_star_4) => alternative_star_4.child_count(),
            ParseTreeRef::RegexOpt6(regex_opt_6) => regex_opt_6.child_count(),
            ParseTreeRef::RegexStar5(regex_star_5) => regex_star_5.child_count(),
            ParseTreeRef::CharClassOpt7(char_class_opt_7) => char_class_opt_7.child_count(),
            ParseTreeRef::CharClassAlt0(char_class_alt_0) => char_class_alt_0.child_count(),
            ParseTreeRef::CharClassPlus7(char_class_plus_7) => char_class_plus_7.child_count(),
            ParseTreeRef::StartGrammar(start_grammar) => start_grammar.child_count(),
            ParseTreeRef::StartSyntaxRule(start_syntax_rule) => start_syntax_rule.child_count(),
            ParseTreeRef::StartRegexBlock(start_regex_block) => start_regex_block.child_count(),
            ParseTreeRef::StartRegexRule(start_regex_rule) => start_regex_rule.child_count(),
            ParseTreeRef::StartPriorityLevel(start_priority_level) => {
                start_priority_level.child_count()
            }
            ParseTreeRef::StartAlternative(start_alternative) => start_alternative.child_count(),
            ParseTreeRef::StartSymbol(start_symbol) => start_symbol.child_count(),
            ParseTreeRef::StartRegex(start_regex) => start_regex.child_count(),
            ParseTreeRef::StartCharClass(start_char_class) => start_char_class.child_count(),
            ParseTreeRef::StartRange(start_range) => start_range.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.span(),
            ParseTreeRef::SyntaxRule(syntax_rule) => syntax_rule.span(),
            ParseTreeRef::RegexBlock(regex_block) => regex_block.span(),
            ParseTreeRef::RegexRule(regex_rule) => regex_rule.span(),
            ParseTreeRef::PriorityLevel(priority_level) => priority_level.span(),
            ParseTreeRef::Alternative(alternative) => alternative.span(),
            ParseTreeRef::Symbol(symbol) => symbol.span(),
            ParseTreeRef::Regex(regex) => regex.span(),
            ParseTreeRef::CharClass(char_class) => char_class.span(),
            ParseTreeRef::Range(range) => range.span(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.span(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.span(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.span(),
            ParseTreeRef::GrammarOpt1(grammar_opt_1) => grammar_opt_1.span(),
            ParseTreeRef::SyntaxRulePlus1(syntax_rule_plus_1) => syntax_rule_plus_1.span(),
            ParseTreeRef::SyntaxRuleOpt2(syntax_rule_opt_2) => syntax_rule_opt_2.span(),
            ParseTreeRef::SyntaxRuleStar1(syntax_rule_star_1) => syntax_rule_star_1.span(),
            ParseTreeRef::RegexBlockPlus2(regex_block_plus_2) => regex_block_plus_2.span(),
            ParseTreeRef::RegexBlockOpt3(regex_block_opt_3) => regex_block_opt_3.span(),
            ParseTreeRef::RegexBlockStar2(regex_block_star_2) => regex_block_star_2.span(),
            ParseTreeRef::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.span(),
            ParseTreeRef::RegexRulePlus3(regex_rule_plus_3) => regex_rule_plus_3.span(),
            ParseTreeRef::PriorityLevelPlus5(priority_level_plus_5) => priority_level_plus_5.span(),
            ParseTreeRef::PriorityLevelOpt4(priority_level_opt_4) => priority_level_opt_4.span(),
            ParseTreeRef::PriorityLevelStar3(priority_level_star_3) => priority_level_star_3.span(),
            ParseTreeRef::AlternativePlus6(alternative_plus_6) => alternative_plus_6.span(),
            ParseTreeRef::AlternativeOpt5(alternative_opt_5) => alternative_opt_5.span(),
            ParseTreeRef::AlternativeStar4(alternative_star_4) => alternative_star_4.span(),
            ParseTreeRef::RegexOpt6(regex_opt_6) => regex_opt_6.span(),
            ParseTreeRef::RegexStar5(regex_star_5) => regex_star_5.span(),
            ParseTreeRef::CharClassOpt7(char_class_opt_7) => char_class_opt_7.span(),
            ParseTreeRef::CharClassAlt0(char_class_alt_0) => char_class_alt_0.span(),
            ParseTreeRef::CharClassPlus7(char_class_plus_7) => char_class_plus_7.span(),
            ParseTreeRef::StartGrammar(start_grammar) => start_grammar.span(),
            ParseTreeRef::StartSyntaxRule(start_syntax_rule) => start_syntax_rule.span(),
            ParseTreeRef::StartRegexBlock(start_regex_block) => start_regex_block.span(),
            ParseTreeRef::StartRegexRule(start_regex_rule) => start_regex_rule.span(),
            ParseTreeRef::StartPriorityLevel(start_priority_level) => start_priority_level.span(),
            ParseTreeRef::StartAlternative(start_alternative) => start_alternative.span(),
            ParseTreeRef::StartSymbol(start_symbol) => start_symbol.span(),
            ParseTreeRef::StartRegex(start_regex) => start_regex.span(),
            ParseTreeRef::StartCharClass(start_char_class) => start_char_class.span(),
            ParseTreeRef::StartRange(start_range) => start_range.span(),
            ParseTreeRef::Token(token) => token.span(),
        }
    }
}
impl From<Grammar> for ParseTree {
    fn from(grammar: Grammar) -> Self {
        ParseTree::Grammar(grammar)
    }
}
impl From<SyntaxRule> for ParseTree {
    fn from(syntax_rule: SyntaxRule) -> Self {
        ParseTree::SyntaxRule(syntax_rule)
    }
}
impl From<RegexBlock> for ParseTree {
    fn from(regex_block: RegexBlock) -> Self {
        ParseTree::RegexBlock(regex_block)
    }
}
impl From<RegexRule> for ParseTree {
    fn from(regex_rule: RegexRule) -> Self {
        ParseTree::RegexRule(regex_rule)
    }
}
impl From<PriorityLevel> for ParseTree {
    fn from(priority_level: PriorityLevel) -> Self {
        ParseTree::PriorityLevel(priority_level)
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
    fn from(char_class: CharClass) -> Self {
        ParseTree::CharClass(char_class)
    }
}
impl From<Range> for ParseTree {
    fn from(range: Range) -> Self {
        ParseTree::Range(range)
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
impl From<GrammarOpt1> for ParseTree {
    fn from(grammar_opt_1: GrammarOpt1) -> Self {
        ParseTree::GrammarOpt1(grammar_opt_1)
    }
}
impl From<SyntaxRulePlus1> for ParseTree {
    fn from(syntax_rule_plus_1: SyntaxRulePlus1) -> Self {
        ParseTree::SyntaxRulePlus1(syntax_rule_plus_1)
    }
}
impl From<SyntaxRuleOpt2> for ParseTree {
    fn from(syntax_rule_opt_2: SyntaxRuleOpt2) -> Self {
        ParseTree::SyntaxRuleOpt2(syntax_rule_opt_2)
    }
}
impl From<SyntaxRuleStar1> for ParseTree {
    fn from(syntax_rule_star_1: SyntaxRuleStar1) -> Self {
        ParseTree::SyntaxRuleStar1(syntax_rule_star_1)
    }
}
impl From<RegexBlockPlus2> for ParseTree {
    fn from(regex_block_plus_2: RegexBlockPlus2) -> Self {
        ParseTree::RegexBlockPlus2(regex_block_plus_2)
    }
}
impl From<RegexBlockOpt3> for ParseTree {
    fn from(regex_block_opt_3: RegexBlockOpt3) -> Self {
        ParseTree::RegexBlockOpt3(regex_block_opt_3)
    }
}
impl From<RegexBlockStar2> for ParseTree {
    fn from(regex_block_star_2: RegexBlockStar2) -> Self {
        ParseTree::RegexBlockStar2(regex_block_star_2)
    }
}
impl From<RegexRulePlus4> for ParseTree {
    fn from(regex_rule_plus_4: RegexRulePlus4) -> Self {
        ParseTree::RegexRulePlus4(regex_rule_plus_4)
    }
}
impl From<RegexRulePlus3> for ParseTree {
    fn from(regex_rule_plus_3: RegexRulePlus3) -> Self {
        ParseTree::RegexRulePlus3(regex_rule_plus_3)
    }
}
impl From<PriorityLevelPlus5> for ParseTree {
    fn from(priority_level_plus_5: PriorityLevelPlus5) -> Self {
        ParseTree::PriorityLevelPlus5(priority_level_plus_5)
    }
}
impl From<PriorityLevelOpt4> for ParseTree {
    fn from(priority_level_opt_4: PriorityLevelOpt4) -> Self {
        ParseTree::PriorityLevelOpt4(priority_level_opt_4)
    }
}
impl From<PriorityLevelStar3> for ParseTree {
    fn from(priority_level_star_3: PriorityLevelStar3) -> Self {
        ParseTree::PriorityLevelStar3(priority_level_star_3)
    }
}
impl From<AlternativePlus6> for ParseTree {
    fn from(alternative_plus_6: AlternativePlus6) -> Self {
        ParseTree::AlternativePlus6(alternative_plus_6)
    }
}
impl From<AlternativeOpt5> for ParseTree {
    fn from(alternative_opt_5: AlternativeOpt5) -> Self {
        ParseTree::AlternativeOpt5(alternative_opt_5)
    }
}
impl From<AlternativeStar4> for ParseTree {
    fn from(alternative_star_4: AlternativeStar4) -> Self {
        ParseTree::AlternativeStar4(alternative_star_4)
    }
}
impl From<RegexOpt6> for ParseTree {
    fn from(regex_opt_6: RegexOpt6) -> Self {
        ParseTree::RegexOpt6(regex_opt_6)
    }
}
impl From<RegexStar5> for ParseTree {
    fn from(regex_star_5: RegexStar5) -> Self {
        ParseTree::RegexStar5(regex_star_5)
    }
}
impl From<CharClassOpt7> for ParseTree {
    fn from(char_class_opt_7: CharClassOpt7) -> Self {
        ParseTree::CharClassOpt7(char_class_opt_7)
    }
}
impl From<CharClassAlt0> for ParseTree {
    fn from(char_class_alt_0: CharClassAlt0) -> Self {
        ParseTree::CharClassAlt0(char_class_alt_0)
    }
}
impl From<CharClassPlus7> for ParseTree {
    fn from(char_class_plus_7: CharClassPlus7) -> Self {
        ParseTree::CharClassPlus7(char_class_plus_7)
    }
}
impl From<StartGrammar> for ParseTree {
    fn from(start_grammar: StartGrammar) -> Self {
        ParseTree::StartGrammar(start_grammar)
    }
}
impl From<StartSyntaxRule> for ParseTree {
    fn from(start_syntax_rule: StartSyntaxRule) -> Self {
        ParseTree::StartSyntaxRule(start_syntax_rule)
    }
}
impl From<StartRegexBlock> for ParseTree {
    fn from(start_regex_block: StartRegexBlock) -> Self {
        ParseTree::StartRegexBlock(start_regex_block)
    }
}
impl From<StartRegexRule> for ParseTree {
    fn from(start_regex_rule: StartRegexRule) -> Self {
        ParseTree::StartRegexRule(start_regex_rule)
    }
}
impl From<StartPriorityLevel> for ParseTree {
    fn from(start_priority_level: StartPriorityLevel) -> Self {
        ParseTree::StartPriorityLevel(start_priority_level)
    }
}
impl From<StartAlternative> for ParseTree {
    fn from(start_alternative: StartAlternative) -> Self {
        ParseTree::StartAlternative(start_alternative)
    }
}
impl From<StartSymbol> for ParseTree {
    fn from(start_symbol: StartSymbol) -> Self {
        ParseTree::StartSymbol(start_symbol)
    }
}
impl From<StartRegex> for ParseTree {
    fn from(start_regex: StartRegex) -> Self {
        ParseTree::StartRegex(start_regex)
    }
}
impl From<StartCharClass> for ParseTree {
    fn from(start_char_class: StartCharClass) -> Self {
        ParseTree::StartCharClass(start_char_class)
    }
}
impl From<StartRange> for ParseTree {
    fn from(start_range: StartRange) -> Self {
        ParseTree::StartRange(start_range)
    }
}
trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
pub trait OptNode {
    type Inner;
    fn value(&self) -> Option<&Self::Inner>;
}
#[derive(Debug)]
pub struct Grammar {
    pub lit_0: Token,
    pub layout_1: Token,
    pub name: Token,
    pub layout_3: Token,
    pub syntax_rules: GrammarStar0,
    pub layout_5: Token,
    pub regex_block: GrammarOpt1,
    pub span: Span,
}
#[derive(Debug)]
pub struct SyntaxRule {
    pub head: Token,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub priority_levels: SyntaxRuleStar1,
    pub span: Span,
}
#[derive(Debug)]
pub struct RegexBlock {
    pub lit_0: Token,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub regex_rules: RegexBlockStar2,
    pub layout_5: Token,
    pub lit_6: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct RegexRule {
    pub identifier_0: Token,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub regex_rule_plus_3_4: RegexRulePlus3,
    pub span: Span,
}
#[derive(Debug)]
pub struct PriorityLevel {
    pub alternatives: PriorityLevelStar3,
    pub span: Span,
}
#[derive(Debug)]
pub struct Alternative {
    pub symbols: AlternativeStar4,
    pub span: Span,
}
#[derive(Debug)]
pub enum Symbol {
    Star {
        symbol_0: Box<Symbol>,
        layout_1: Token,
        lit_2: Token,
        span: Span,
    },
    Plus {
        symbol_0: Box<Symbol>,
        layout_1: Token,
        lit_2: Token,
        span: Span,
    },
    Opt {
        symbol_0: Box<Symbol>,
        layout_1: Token,
        lit_2: Token,
        span: Span,
    },
    Alt {
        lit_0: Token,
        layout_1: Token,
        symbol_2: Box<Symbol>,
        layout_3: Token,
        lit_4: Token,
        layout_5: Token,
        symbol_6: Box<Symbol>,
        layout_7: Token,
        lit_8: Token,
        span: Span,
    },
    Lit {
        lit_0: Token,
        layout_1: Token,
        string_2: Token,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    StarSep {
        lit_0: Token,
        layout_1: Token,
        symbol_2: Box<Symbol>,
        layout_3: Token,
        symbol_4: Box<Symbol>,
        layout_5: Token,
        lit_6: Token,
        layout_7: Token,
        lit_8: Token,
        span: Span,
    },
    PlusSep {
        lit_0: Token,
        layout_1: Token,
        symbol_2: Box<Symbol>,
        layout_3: Token,
        symbol_4: Box<Symbol>,
        layout_5: Token,
        lit_6: Token,
        layout_7: Token,
        lit_8: Token,
        span: Span,
    },
    Group {
        lit_0: Token,
        layout_1: Token,
        symbols: AlternativePlus6,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    Identifier {
        identifier_0: Token,
        span: Span,
    },
}
#[derive(Debug)]
pub enum Regex {
    Plus {
        regex_0: Box<Regex>,
        layout_1: Token,
        lit_2: Token,
        span: Span,
    },
    Star {
        regex_0: Box<Regex>,
        layout_1: Token,
        lit_2: Token,
        span: Span,
    },
    Opt {
        regex_0: Box<Regex>,
        layout_1: Token,
        lit_2: Token,
        span: Span,
    },
    Alt {
        lit_0: Token,
        layout_1: Token,
        regex_star_5_2: RegexStar5,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    CharClass {
        char_class_0: CharClass,
        span: Span,
    },
    Char {
        lit_0: Token,
        layout_1: Token,
        char_2: Token,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
}
#[derive(Debug)]
pub struct CharClass {
    pub char_class_opt_7_0: CharClassOpt7,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub char_class_plus_7_4: CharClassPlus7,
    pub layout_5: Token,
    pub lit_6: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct Range {
    pub range_char_0: Token,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub range_char_4: Token,
    pub span: Span,
}
//SyntaxRule+
#[derive(Debug)]
pub enum GrammarPlus0 {
    Alt0 {
        syntax_rules: Box<GrammarPlus0>,
        layout_1: Token,
        syntax_rule_2: Box<SyntaxRule>,
        span: Span,
    },
    Alt1 {
        syntax_rule_0: Box<SyntaxRule>,
        span: Span,
    },
}
//SyntaxRule+?
#[derive(Debug)]
pub enum GrammarOpt0 {
    Alt0 {
        syntax_rules: GrammarPlus0,
        span: Span,
    },
    Alt1 {
        span: Span,
    },
}
//SyntaxRule*
#[derive(Debug)]
pub struct GrammarStar0 {
    pub grammar_opt_0_0: GrammarOpt0,
    pub span: Span,
}
//RegexBlock?
#[derive(Debug)]
pub enum GrammarOpt1 {
    Alt0 {
        regex_block_0: RegexBlock,
        span: Span,
    },
    Alt1 {
        span: Span,
    },
}
//{PriorityLevel ">"}+
#[derive(Debug)]
pub enum SyntaxRulePlus1 {
    Alt0 {
        priority_levels: Box<SyntaxRulePlus1>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        priority_level_4: Box<PriorityLevel>,
        span: Span,
    },
    Alt1 {
        priority_level_0: Box<PriorityLevel>,
        span: Span,
    },
}
//{PriorityLevel ">"}+?
#[derive(Debug)]
pub enum SyntaxRuleOpt2 {
    Alt0 {
        priority_levels: SyntaxRulePlus1,
        span: Span,
    },
    Alt1 {
        span: Span,
    },
}
//{PriorityLevel ">"}*
#[derive(Debug)]
pub struct SyntaxRuleStar1 {
    pub syntax_rule_opt_2_0: SyntaxRuleOpt2,
    pub span: Span,
}
//RegexRule+
#[derive(Debug)]
pub enum RegexBlockPlus2 {
    Alt0 {
        regex_rules: Box<RegexBlockPlus2>,
        layout_1: Token,
        regex_rule_2: Box<RegexRule>,
        span: Span,
    },
    Alt1 {
        regex_rule_0: Box<RegexRule>,
        span: Span,
    },
}
//RegexRule+?
#[derive(Debug)]
pub enum RegexBlockOpt3 {
    Alt0 {
        regex_rules: RegexBlockPlus2,
        span: Span,
    },
    Alt1 {
        span: Span,
    },
}
//RegexRule*
#[derive(Debug)]
pub struct RegexBlockStar2 {
    pub regex_block_opt_3_0: RegexBlockOpt3,
    pub span: Span,
}
//Regex+
#[derive(Debug)]
pub enum RegexRulePlus4 {
    Alt0 {
        regexes: Box<RegexRulePlus4>,
        layout_1: Token,
        regex_2: Box<Regex>,
        span: Span,
    },
    Alt1 {
        regex_0: Box<Regex>,
        span: Span,
    },
}
//{Regex+ "|"}+
#[derive(Debug)]
pub enum RegexRulePlus3 {
    Alt0 {
        regex_rule_plus_3_0: Box<RegexRulePlus3>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        regexes: RegexRulePlus4,
        span: Span,
    },
    Alt1 {
        regexes: RegexRulePlus4,
        span: Span,
    },
}
//{Alternative "|"}+
#[derive(Debug)]
pub enum PriorityLevelPlus5 {
    Alt0 {
        alternatives: Box<PriorityLevelPlus5>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        alternative_4: Box<Alternative>,
        span: Span,
    },
    Alt1 {
        alternative_0: Box<Alternative>,
        span: Span,
    },
}
//{Alternative "|"}+?
#[derive(Debug)]
pub enum PriorityLevelOpt4 {
    Alt0 {
        alternatives: PriorityLevelPlus5,
        span: Span,
    },
    Alt1 {
        span: Span,
    },
}
//{Alternative "|"}*
#[derive(Debug)]
pub struct PriorityLevelStar3 {
    pub priority_level_opt_4_0: PriorityLevelOpt4,
    pub span: Span,
}
//Symbol+
#[derive(Debug)]
pub enum AlternativePlus6 {
    Alt0 {
        symbols: Box<AlternativePlus6>,
        layout_1: Token,
        symbol_2: Box<Symbol>,
        span: Span,
    },
    Alt1 {
        symbol_0: Box<Symbol>,
        span: Span,
    },
}
//Symbol+?
#[derive(Debug)]
pub enum AlternativeOpt5 {
    Alt0 {
        symbols: AlternativePlus6,
        span: Span,
    },
    Alt1 {
        span: Span,
    },
}
//Symbol*
#[derive(Debug)]
pub struct AlternativeStar4 {
    pub alternative_opt_5_0: AlternativeOpt5,
    pub span: Span,
}
//{Regex+ "|"}+?
#[derive(Debug)]
pub enum RegexOpt6 {
    Alt0 {
        regex_rule_plus_3_0: RegexRulePlus3,
        span: Span,
    },
    Alt1 {
        span: Span,
    },
}
//{Regex+ "|"}*
#[derive(Debug)]
pub struct RegexStar5 {
    pub regex_opt_6_0: RegexOpt6,
    pub span: Span,
}
//"!"?
#[derive(Debug)]
pub enum CharClassOpt7 {
    Alt0 { lit_0: Token, span: Span },
    Alt1 { span: Span },
}
//(Range | RangeChar)
#[derive(Debug)]
pub enum CharClassAlt0 {
    Alt0 { range_0: Range, span: Span },
    Alt1 { range_char_0: Token, span: Span },
}
//(Range | RangeChar)+
#[derive(Debug)]
pub enum CharClassPlus7 {
    Alt0 {
        char_class_plus_7_0: Box<CharClassPlus7>,
        layout_1: Token,
        char_class_alt_0_2: CharClassAlt0,
        span: Span,
    },
    Alt1 {
        char_class_alt_0_0: CharClassAlt0,
        span: Span,
    },
}
#[derive(Debug)]
pub struct StartGrammar {
    pub layout_0: Token,
    pub start: Grammar,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartSyntaxRule {
    pub layout_0: Token,
    pub start: SyntaxRule,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartRegexBlock {
    pub layout_0: Token,
    pub start: RegexBlock,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartRegexRule {
    pub layout_0: Token,
    pub start: RegexRule,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartPriorityLevel {
    pub layout_0: Token,
    pub start: PriorityLevel,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartAlternative {
    pub layout_0: Token,
    pub start: Alternative,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartSymbol {
    pub layout_0: Token,
    pub start: Symbol,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartRegex {
    pub layout_0: Token,
    pub start: Regex,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartCharClass {
    pub layout_0: Token,
    pub start: CharClass,
    pub layout_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub struct StartRange {
    pub layout_0: Token,
    pub start: Range,
    pub layout_2: Token,
    pub span: Span,
}
impl Grammar {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.lit_0.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.name.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.syntax_rules.as_parse_tree_ref()),
            5 => Some(self.layout_5.as_parse_tree_ref()),
            6 => Some(self.regex_block.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        7usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Grammar(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl SyntaxRule {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.head.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.priority_levels.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        5usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRule(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl RegexBlock {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.lit_0.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.regex_rules.as_parse_tree_ref()),
            5 => Some(self.layout_5.as_parse_tree_ref()),
            6 => Some(self.lit_6.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        7usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexBlock(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl RegexRule {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.identifier_0.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.regex_rule_plus_3_4.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        5usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRule(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl PriorityLevel {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.alternatives.as_parse_tree_ref()),
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
        self.span
    }
}
impl Alternative {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.symbols.as_parse_tree_ref()),
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
        self.span
    }
}
impl Symbol {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Symbol::Star {
                symbol_0,
                layout_1,
                lit_2,
                ..
            } => match index {
                0 => Some(symbol_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Plus {
                symbol_0,
                layout_1,
                lit_2,
                ..
            } => match index {
                0 => Some(symbol_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Opt {
                symbol_0,
                layout_1,
                lit_2,
                ..
            } => match index {
                0 => Some(symbol_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Alt {
                lit_0,
                layout_1,
                symbol_2,
                layout_3,
                lit_4,
                layout_5,
                symbol_6,
                layout_7,
                lit_8,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(symbol_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(symbol_6.as_parse_tree_ref()),
                7 => Some(layout_7.as_parse_tree_ref()),
                8 => Some(lit_8.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Lit {
                lit_0,
                layout_1,
                string_2,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(string_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::StarSep {
                lit_0,
                layout_1,
                symbol_2,
                layout_3,
                symbol_4,
                layout_5,
                lit_6,
                layout_7,
                lit_8,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(symbol_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(symbol_4.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(lit_6.as_parse_tree_ref()),
                7 => Some(layout_7.as_parse_tree_ref()),
                8 => Some(lit_8.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::PlusSep {
                lit_0,
                layout_1,
                symbol_2,
                layout_3,
                symbol_4,
                layout_5,
                lit_6,
                layout_7,
                lit_8,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(symbol_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(symbol_4.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(lit_6.as_parse_tree_ref()),
                7 => Some(layout_7.as_parse_tree_ref()),
                8 => Some(lit_8.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Group {
                lit_0,
                layout_1,
                symbols,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(symbols.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Identifier { identifier_0, .. } => match index {
                0 => Some(identifier_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Symbol::Star { .. } => 3usize,
            Symbol::Plus { .. } => 3usize,
            Symbol::Opt { .. } => 3usize,
            Symbol::Alt { .. } => 9usize,
            Symbol::Lit { .. } => 5usize,
            Symbol::StarSep { .. } => 9usize,
            Symbol::PlusSep { .. } => 9usize,
            Symbol::Group { .. } => 5usize,
            Symbol::Identifier { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Symbol(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Symbol::Star { span, .. } => *span,
            Symbol::Plus { span, .. } => *span,
            Symbol::Opt { span, .. } => *span,
            Symbol::Alt { span, .. } => *span,
            Symbol::Lit { span, .. } => *span,
            Symbol::StarSep { span, .. } => *span,
            Symbol::PlusSep { span, .. } => *span,
            Symbol::Group { span, .. } => *span,
            Symbol::Identifier { span, .. } => *span,
        }
    }
}
impl Regex {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Regex::Plus {
                regex_0,
                layout_1,
                lit_2,
                ..
            } => match index {
                0 => Some(regex_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Star {
                regex_0,
                layout_1,
                lit_2,
                ..
            } => match index {
                0 => Some(regex_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Opt {
                regex_0,
                layout_1,
                lit_2,
                ..
            } => match index {
                0 => Some(regex_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Alt {
                lit_0,
                layout_1,
                regex_star_5_2,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(regex_star_5_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            Regex::CharClass { char_class_0, .. } => match index {
                0 => Some(char_class_0.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Char {
                lit_0,
                layout_1,
                char_2,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(char_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Regex::Plus { .. } => 3usize,
            Regex::Star { .. } => 3usize,
            Regex::Opt { .. } => 3usize,
            Regex::Alt { .. } => 5usize,
            Regex::CharClass { .. } => 1usize,
            Regex::Char { .. } => 5usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Regex(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Regex::Plus { span, .. } => *span,
            Regex::Star { span, .. } => *span,
            Regex::Opt { span, .. } => *span,
            Regex::Alt { span, .. } => *span,
            Regex::CharClass { span, .. } => *span,
            Regex::Char { span, .. } => *span,
        }
    }
}
impl CharClass {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.char_class_opt_7_0.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.char_class_plus_7_4.as_parse_tree_ref()),
            5 => Some(self.layout_5.as_parse_tree_ref()),
            6 => Some(self.lit_6.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        7usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClass(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl Range {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.range_char_0.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.range_char_4.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        5usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Range(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl GrammarPlus0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarPlus0::Alt0 {
                syntax_rules,
                layout_1,
                syntax_rule_2,
                ..
            } => match index {
                0 => Some(syntax_rules.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(syntax_rule_2.as_parse_tree_ref()),
                _ => None,
            },
            GrammarPlus0::Alt1 { syntax_rule_0, .. } => match index {
                0 => Some(syntax_rule_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            GrammarPlus0::Alt0 { .. } => 3usize,
            GrammarPlus0::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::GrammarPlus0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            GrammarPlus0::Alt0 { span, .. } => *span,
            GrammarPlus0::Alt1 { span, .. } => *span,
        }
    }
    pub fn syntax_rules(&self) -> impl Iterator<Item = &SyntaxRule> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::SyntaxRule(r) => Some(r),
            _ => None,
        })
    }
}
impl GrammarOpt0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarOpt0::Alt0 { syntax_rules, .. } => match index {
                0 => Some(syntax_rules.as_parse_tree_ref()),
                _ => None,
            },
            GrammarOpt0::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            GrammarOpt0::Alt0 { .. } => 1usize,
            GrammarOpt0::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::GrammarOpt0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            GrammarOpt0::Alt0 { span, .. } => *span,
            GrammarOpt0::Alt1 { span, .. } => *span,
        }
    }
}
impl GrammarStar0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.grammar_opt_0_0.as_parse_tree_ref()),
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
        self.span
    }
    pub fn syntax_rules(&self) -> impl Iterator<Item = &SyntaxRule> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::SyntaxRule(r) => Some(r),
            _ => None,
        })
    }
}
impl GrammarOpt1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarOpt1::Alt0 { regex_block_0, .. } => match index {
                0 => Some(regex_block_0.as_parse_tree_ref()),
                _ => None,
            },
            GrammarOpt1::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            GrammarOpt1::Alt0 { .. } => 1usize,
            GrammarOpt1::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::GrammarOpt1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            GrammarOpt1::Alt0 { span, .. } => *span,
            GrammarOpt1::Alt1 { span, .. } => *span,
        }
    }
}
impl SyntaxRulePlus1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SyntaxRulePlus1::Alt0 {
                priority_levels,
                layout_1,
                lit_2,
                layout_3,
                priority_level_4,
                ..
            } => match index {
                0 => Some(priority_levels.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(priority_level_4.as_parse_tree_ref()),
                _ => None,
            },
            SyntaxRulePlus1::Alt1 {
                priority_level_0, ..
            } => match index {
                0 => Some(priority_level_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SyntaxRulePlus1::Alt0 { .. } => 5usize,
            SyntaxRulePlus1::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRulePlus1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SyntaxRulePlus1::Alt0 { span, .. } => *span,
            SyntaxRulePlus1::Alt1 { span, .. } => *span,
        }
    }
    pub fn priority_levels(&self) -> impl Iterator<Item = &PriorityLevel> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::PriorityLevel(r) => Some(r),
            _ => None,
        })
    }
}
impl SyntaxRuleOpt2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SyntaxRuleOpt2::Alt0 {
                priority_levels, ..
            } => match index {
                0 => Some(priority_levels.as_parse_tree_ref()),
                _ => None,
            },
            SyntaxRuleOpt2::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SyntaxRuleOpt2::Alt0 { .. } => 1usize,
            SyntaxRuleOpt2::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRuleOpt2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SyntaxRuleOpt2::Alt0 { span, .. } => *span,
            SyntaxRuleOpt2::Alt1 { span, .. } => *span,
        }
    }
}
impl SyntaxRuleStar1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.syntax_rule_opt_2_0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRuleStar1(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn priority_levels(&self) -> impl Iterator<Item = &PriorityLevel> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::PriorityLevel(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexBlockPlus2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexBlockPlus2::Alt0 {
                regex_rules,
                layout_1,
                regex_rule_2,
                ..
            } => match index {
                0 => Some(regex_rules.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(regex_rule_2.as_parse_tree_ref()),
                _ => None,
            },
            RegexBlockPlus2::Alt1 { regex_rule_0, .. } => match index {
                0 => Some(regex_rule_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexBlockPlus2::Alt0 { .. } => 3usize,
            RegexBlockPlus2::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexBlockPlus2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexBlockPlus2::Alt0 { span, .. } => *span,
            RegexBlockPlus2::Alt1 { span, .. } => *span,
        }
    }
    pub fn regex_rules(&self) -> impl Iterator<Item = &RegexRule> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::RegexRule(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexBlockOpt3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexBlockOpt3::Alt0 { regex_rules, .. } => match index {
                0 => Some(regex_rules.as_parse_tree_ref()),
                _ => None,
            },
            RegexBlockOpt3::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexBlockOpt3::Alt0 { .. } => 1usize,
            RegexBlockOpt3::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexBlockOpt3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexBlockOpt3::Alt0 { span, .. } => *span,
            RegexBlockOpt3::Alt1 { span, .. } => *span,
        }
    }
}
impl RegexBlockStar2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.regex_block_opt_3_0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexBlockStar2(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn regex_rules(&self) -> impl Iterator<Item = &RegexRule> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::RegexRule(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexRulePlus4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexRulePlus4::Alt0 {
                regexes,
                layout_1,
                regex_2,
                ..
            } => match index {
                0 => Some(regexes.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(regex_2.as_parse_tree_ref()),
                _ => None,
            },
            RegexRulePlus4::Alt1 { regex_0, .. } => match index {
                0 => Some(regex_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexRulePlus4::Alt0 { .. } => 3usize,
            RegexRulePlus4::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRulePlus4(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexRulePlus4::Alt0 { span, .. } => *span,
            RegexRulePlus4::Alt1 { span, .. } => *span,
        }
    }
    pub fn regexes(&self) -> impl Iterator<Item = &Regex> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Regex(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexRulePlus3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexRulePlus3::Alt0 {
                regex_rule_plus_3_0,
                layout_1,
                lit_2,
                layout_3,
                regexes,
                ..
            } => match index {
                0 => Some(regex_rule_plus_3_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(regexes.as_parse_tree_ref()),
                _ => None,
            },
            RegexRulePlus3::Alt1 { regexes, .. } => match index {
                0 => Some(regexes.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexRulePlus3::Alt0 { .. } => 5usize,
            RegexRulePlus3::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRulePlus3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexRulePlus3::Alt0 { span, .. } => *span,
            RegexRulePlus3::Alt1 { span, .. } => *span,
        }
    }
}
impl PriorityLevelPlus5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelPlus5::Alt0 {
                alternatives,
                layout_1,
                lit_2,
                layout_3,
                alternative_4,
                ..
            } => match index {
                0 => Some(alternatives.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(alternative_4.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelPlus5::Alt1 { alternative_0, .. } => match index {
                0 => Some(alternative_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelPlus5::Alt0 { .. } => 5usize,
            PriorityLevelPlus5::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelPlus5(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelPlus5::Alt0 { span, .. } => *span,
            PriorityLevelPlus5::Alt1 { span, .. } => *span,
        }
    }
    pub fn alternatives(&self) -> impl Iterator<Item = &Alternative> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Alternative(r) => Some(r),
            _ => None,
        })
    }
}
impl PriorityLevelOpt4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelOpt4::Alt0 { alternatives, .. } => match index {
                0 => Some(alternatives.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelOpt4::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelOpt4::Alt0 { .. } => 1usize,
            PriorityLevelOpt4::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelOpt4(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelOpt4::Alt0 { span, .. } => *span,
            PriorityLevelOpt4::Alt1 { span, .. } => *span,
        }
    }
}
impl PriorityLevelStar3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.priority_level_opt_4_0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelStar3(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn alternatives(&self) -> impl Iterator<Item = &Alternative> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Alternative(r) => Some(r),
            _ => None,
        })
    }
}
impl AlternativePlus6 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativePlus6::Alt0 {
                symbols,
                layout_1,
                symbol_2,
                ..
            } => match index {
                0 => Some(symbols.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(symbol_2.as_parse_tree_ref()),
                _ => None,
            },
            AlternativePlus6::Alt1 { symbol_0, .. } => match index {
                0 => Some(symbol_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativePlus6::Alt0 { .. } => 3usize,
            AlternativePlus6::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativePlus6(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativePlus6::Alt0 { span, .. } => *span,
            AlternativePlus6::Alt1 { span, .. } => *span,
        }
    }
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Symbol(r) => Some(r),
            _ => None,
        })
    }
}
impl AlternativeOpt5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativeOpt5::Alt0 { symbols, .. } => match index {
                0 => Some(symbols.as_parse_tree_ref()),
                _ => None,
            },
            AlternativeOpt5::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativeOpt5::Alt0 { .. } => 1usize,
            AlternativeOpt5::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativeOpt5(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativeOpt5::Alt0 { span, .. } => *span,
            AlternativeOpt5::Alt1 { span, .. } => *span,
        }
    }
}
impl AlternativeStar4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.alternative_opt_5_0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativeStar4(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Symbol(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexOpt6 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexOpt6::Alt0 {
                regex_rule_plus_3_0,
                ..
            } => match index {
                0 => Some(regex_rule_plus_3_0.as_parse_tree_ref()),
                _ => None,
            },
            RegexOpt6::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexOpt6::Alt0 { .. } => 1usize,
            RegexOpt6::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexOpt6(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexOpt6::Alt0 { span, .. } => *span,
            RegexOpt6::Alt1 { span, .. } => *span,
        }
    }
}
impl RegexStar5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.regex_opt_6_0.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexStar5(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl CharClassOpt7 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassOpt7::Alt0 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
            CharClassOpt7::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassOpt7::Alt0 { .. } => 1usize,
            CharClassOpt7::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassOpt7(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassOpt7::Alt0 { span, .. } => *span,
            CharClassOpt7::Alt1 { span, .. } => *span,
        }
    }
}
impl CharClassAlt0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassAlt0::Alt0 { range_0, .. } => match index {
                0 => Some(range_0.as_parse_tree_ref()),
                _ => None,
            },
            CharClassAlt0::Alt1 { range_char_0, .. } => match index {
                0 => Some(range_char_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassAlt0::Alt0 { .. } => 1usize,
            CharClassAlt0::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassAlt0(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassAlt0::Alt0 { span, .. } => *span,
            CharClassAlt0::Alt1 { span, .. } => *span,
        }
    }
}
impl CharClassPlus7 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassPlus7::Alt0 {
                char_class_plus_7_0,
                layout_1,
                char_class_alt_0_2,
                ..
            } => match index {
                0 => Some(char_class_plus_7_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(char_class_alt_0_2.as_parse_tree_ref()),
                _ => None,
            },
            CharClassPlus7::Alt1 {
                char_class_alt_0_0, ..
            } => match index {
                0 => Some(char_class_alt_0_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassPlus7::Alt0 { .. } => 3usize,
            CharClassPlus7::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassPlus7(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassPlus7::Alt0 { span, .. } => *span,
            CharClassPlus7::Alt1 { span, .. } => *span,
        }
    }
}
impl StartGrammar {
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
        ParseTreeRef::StartGrammar(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartSyntaxRule {
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
        ParseTreeRef::StartSyntaxRule(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartRegexBlock {
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
        ParseTreeRef::StartRegexBlock(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartRegexRule {
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
        ParseTreeRef::StartRegexRule(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartPriorityLevel {
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
        ParseTreeRef::StartPriorityLevel(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartAlternative {
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
        ParseTreeRef::StartAlternative(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartSymbol {
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
        ParseTreeRef::StartSymbol(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartRegex {
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
        ParseTreeRef::StartRegex(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartCharClass {
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
        ParseTreeRef::StartCharClass(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartRange {
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
        ParseTreeRef::StartRange(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl<'a> ListNode<'a> for GrammarPlus0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                GrammarPlus0::Alt0 {
                    syntax_rules: rest,
                    layout_1: layout,
                    syntax_rule_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                GrammarPlus0::Alt1 {
                    syntax_rule_0: item,
                    ..
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
impl<'a> ListNode<'a> for SyntaxRulePlus1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                SyntaxRulePlus1::Alt0 {
                    priority_levels: rest,
                    layout_1: layout1,
                    lit_2: sep,
                    layout_3: layout2,
                    priority_level_4: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout2.as_parse_tree_ref());
                    items.push(sep.as_parse_tree_ref());
                    items.push(layout1.as_parse_tree_ref());
                    current = rest;
                }
                SyntaxRulePlus1::Alt1 {
                    priority_level_0: item,
                    ..
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
impl<'a> ListNode<'a> for RegexBlockPlus2 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RegexBlockPlus2::Alt0 {
                    regex_rules: rest,
                    layout_1: layout,
                    regex_rule_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                RegexBlockPlus2::Alt1 {
                    regex_rule_0: item, ..
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
impl<'a> ListNode<'a> for RegexRulePlus4 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RegexRulePlus4::Alt0 {
                    regexes: rest,
                    layout_1: layout,
                    regex_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                RegexRulePlus4::Alt1 { regex_0: item, .. } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for RegexRulePlus3 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RegexRulePlus3::Alt0 {
                    regex_rule_plus_3_0: rest,
                    layout_1: layout1,
                    lit_2: sep,
                    layout_3: layout2,
                    regexes: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout2.as_parse_tree_ref());
                    items.push(sep.as_parse_tree_ref());
                    items.push(layout1.as_parse_tree_ref());
                    current = rest;
                }
                RegexRulePlus3::Alt1 { regexes: item, .. } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for PriorityLevelPlus5 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                PriorityLevelPlus5::Alt0 {
                    alternatives: rest,
                    layout_1: layout1,
                    lit_2: sep,
                    layout_3: layout2,
                    alternative_4: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout2.as_parse_tree_ref());
                    items.push(sep.as_parse_tree_ref());
                    items.push(layout1.as_parse_tree_ref());
                    current = rest;
                }
                PriorityLevelPlus5::Alt1 {
                    alternative_0: item,
                    ..
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
impl<'a> ListNode<'a> for AlternativePlus6 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                AlternativePlus6::Alt0 {
                    symbols: rest,
                    layout_1: layout,
                    symbol_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                AlternativePlus6::Alt1 { symbol_0: item, .. } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for CharClassPlus7 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                CharClassPlus7::Alt0 {
                    char_class_plus_7_0: rest,
                    layout_1: layout,
                    char_class_alt_0_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                CharClassPlus7::Alt1 {
                    char_class_alt_0_0: item,
                    ..
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
impl<'a> ListNode<'a> for GrammarStar0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.grammar_opt_0_0 {
            GrammarOpt0::Alt0 {
                syntax_rules: grammar_opt_0,
                ..
            } => grammar_opt_0.iter(),
            GrammarOpt0::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for SyntaxRuleStar1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.syntax_rule_opt_2_0 {
            SyntaxRuleOpt2::Alt0 {
                priority_levels: syntax_rule_opt_2,
                ..
            } => syntax_rule_opt_2.iter(),
            SyntaxRuleOpt2::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for RegexBlockStar2 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.regex_block_opt_3_0 {
            RegexBlockOpt3::Alt0 {
                regex_rules: regex_block_opt_3,
                ..
            } => regex_block_opt_3.iter(),
            RegexBlockOpt3::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for PriorityLevelStar3 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.priority_level_opt_4_0 {
            PriorityLevelOpt4::Alt0 {
                alternatives: priority_level_opt_4,
                ..
            } => priority_level_opt_4.iter(),
            PriorityLevelOpt4::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for AlternativeStar4 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.alternative_opt_5_0 {
            AlternativeOpt5::Alt0 {
                symbols: alternative_opt_5,
                ..
            } => alternative_opt_5.iter(),
            AlternativeOpt5::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for RegexStar5 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.regex_opt_6_0 {
            RegexOpt6::Alt0 {
                regex_rule_plus_3_0: regex_opt_6,
                ..
            } => regex_opt_6.iter(),
            RegexOpt6::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl OptNode for GrammarOpt0 {
    type Inner = GrammarPlus0;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            GrammarOpt0::Alt0 { syntax_rules, .. } => Some(syntax_rules),
            GrammarOpt0::Alt1 { .. } => None,
        }
    }
}
impl OptNode for GrammarOpt1 {
    type Inner = RegexBlock;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            GrammarOpt1::Alt0 { regex_block_0, .. } => Some(regex_block_0),
            GrammarOpt1::Alt1 { .. } => None,
        }
    }
}
impl OptNode for SyntaxRuleOpt2 {
    type Inner = SyntaxRulePlus1;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            SyntaxRuleOpt2::Alt0 {
                priority_levels, ..
            } => Some(priority_levels),
            SyntaxRuleOpt2::Alt1 { .. } => None,
        }
    }
}
impl OptNode for RegexBlockOpt3 {
    type Inner = RegexBlockPlus2;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            RegexBlockOpt3::Alt0 { regex_rules, .. } => Some(regex_rules),
            RegexBlockOpt3::Alt1 { .. } => None,
        }
    }
}
impl OptNode for PriorityLevelOpt4 {
    type Inner = PriorityLevelPlus5;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            PriorityLevelOpt4::Alt0 { alternatives, .. } => Some(alternatives),
            PriorityLevelOpt4::Alt1 { .. } => None,
        }
    }
}
impl OptNode for AlternativeOpt5 {
    type Inner = AlternativePlus6;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            AlternativeOpt5::Alt0 { symbols, .. } => Some(symbols),
            AlternativeOpt5::Alt1 { .. } => None,
        }
    }
}
impl OptNode for RegexOpt6 {
    type Inner = RegexRulePlus3;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            RegexOpt6::Alt0 {
                regex_rule_plus_3_0,
                ..
            } => Some(regex_rule_plus_3_0),
            RegexOpt6::Alt1 { .. } => None,
        }
    }
}
impl OptNode for CharClassOpt7 {
    type Inner = Token;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            CharClassOpt7::Alt0 { lit_0, .. } => Some(lit_0),
            CharClassOpt7::Alt1 { .. } => None,
        }
    }
}
impl CharClassAlt0 {
    pub fn as_range(&self) -> Option<&Range> {
        match self {
            CharClassAlt0::Alt0 { range_0, .. } => Some(range_0),
            _ => None,
        }
    }
    pub fn as_range_char(&self) -> Option<&Token> {
        match self {
            CharClassAlt0::Alt1 { range_char_0, .. } => Some(range_char_0),
            _ => None,
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
        //RangeChar
        TerminalId(2) => TokenKind::T2,
        //Char
        TerminalId(3) => TokenKind::T3,
        //WS
        TerminalId(4) => TokenKind::T4,
        //"grammar"
        TerminalId(5) => TokenKind::T5,
        //"="
        TerminalId(6) => TokenKind::T6,
        //">"
        TerminalId(7) => TokenKind::T7,
        //"regex"
        TerminalId(8) => TokenKind::T8,
        //"{"
        TerminalId(9) => TokenKind::T9,
        //"}"
        TerminalId(10) => TokenKind::T10,
        //"|"
        TerminalId(11) => TokenKind::T11,
        //"*"
        TerminalId(12) => TokenKind::T12,
        //"+"
        TerminalId(13) => TokenKind::T13,
        //"?"
        TerminalId(14) => TokenKind::T14,
        //"("
        TerminalId(15) => TokenKind::T15,
        //")"
        TerminalId(16) => TokenKind::T16,
        //"""
        TerminalId(17) => TokenKind::T17,
        //"!"
        TerminalId(18) => TokenKind::T18,
        //"["
        TerminalId(19) => TokenKind::T19,
        //"]"
        TerminalId(20) => TokenKind::T20,
        //"-"
        TerminalId(21) => TokenKind::T21,
        //Layout
        TerminalId(22) => TokenKind::T22,
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
                    //Grammar : "grammar" Layout Identifier Layout SyntaxRule* Layout RegexBlock?.
                    SlotId(7) => {
                        let [
                            lit_0,
                            layout_1,
                            name,
                            layout_3,
                            syntax_rules,
                            layout_5,
                            regex_block,
                        ] = <[ParseTree; 7usize]>::try_from(children).unwrap();
                        Grammar {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            name: name.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            syntax_rules: syntax_rules.unwrap_grammar_star_0(),
                            layout_5: layout_5.unwrap_token(),
                            regex_block: regex_block.unwrap_grammar_opt_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule : Identifier Layout "=" Layout {PriorityLevel ">"}*.
                    SlotId(13) => {
                        let [head, layout_1, lit_2, layout_3, priority_levels] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        SyntaxRule {
                            head: head.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            priority_levels: priority_levels.unwrap_syntax_rule_star_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //RegexBlock : "regex" Layout "{" Layout RegexRule* Layout "}".
                    SlotId(21) => {
                        let [
                            lit_0,
                            layout_1,
                            lit_2,
                            layout_3,
                            regex_rules,
                            layout_5,
                            lit_6,
                        ] = <[ParseTree; 7usize]>::try_from(children).unwrap();
                        RegexBlock {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            regex_rules: regex_rules.unwrap_regex_block_star_2(),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //RegexRule : Identifier Layout "=" Layout {Regex+ "|"}+.
                    SlotId(27) => {
                        let [identifier_0, layout_1, lit_2, layout_3, regex_rule_plus_3_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        RegexRule {
                            identifier_0: identifier_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            regex_rule_plus_3_4: regex_rule_plus_3_4.unwrap_regex_rule_plus_3(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel : {Alternative "|"}*.
                    SlotId(29) => {
                        let [alternatives] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevel {
                            alternatives: alternatives.unwrap_priority_level_star_3(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //Alternative : Symbol*.
                    SlotId(31) => {
                        let [symbols] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Alternative {
                            symbols: symbols.unwrap_alternative_star_4(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Symbol
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //Symbol : Symbol Layout "*".
                    SlotId(35) => {
                        let [symbol_0, layout_1, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Star {
                            symbol_0: Box::new(symbol_0.unwrap_symbol()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : Symbol Layout "+".
                    SlotId(39) => {
                        let [symbol_0, layout_1, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Plus {
                            symbol_0: Box::new(symbol_0.unwrap_symbol()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : Symbol Layout "?".
                    SlotId(43) => {
                        let [symbol_0, layout_1, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Opt {
                            symbol_0: Box::new(symbol_0.unwrap_symbol()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "(" Layout Symbol Layout "|" Layout Symbol Layout ")".
                    SlotId(53) => {
                        let [
                            lit_0,
                            layout_1,
                            symbol_2,
                            layout_3,
                            lit_4,
                            layout_5,
                            symbol_6,
                            layout_7,
                            lit_8,
                        ] = <[ParseTree; 9usize]>::try_from(children).unwrap();
                        Symbol::Alt {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            symbol_2: Box::new(symbol_2.unwrap_symbol()),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            layout_5: layout_5.unwrap_token(),
                            symbol_6: Box::new(symbol_6.unwrap_symbol()),
                            layout_7: layout_7.unwrap_token(),
                            lit_8: lit_8.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : """ Layout String Layout """.
                    SlotId(59) => {
                        let [lit_0, layout_1, string_2, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Lit {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            string_2: string_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "*".
                    SlotId(69) => {
                        let [
                            lit_0,
                            layout_1,
                            symbol_2,
                            layout_3,
                            symbol_4,
                            layout_5,
                            lit_6,
                            layout_7,
                            lit_8,
                        ] = <[ParseTree; 9usize]>::try_from(children).unwrap();
                        Symbol::StarSep {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            symbol_2: Box::new(symbol_2.unwrap_symbol()),
                            layout_3: layout_3.unwrap_token(),
                            symbol_4: Box::new(symbol_4.unwrap_symbol()),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            layout_7: layout_7.unwrap_token(),
                            lit_8: lit_8.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "+".
                    SlotId(79) => {
                        let [
                            lit_0,
                            layout_1,
                            symbol_2,
                            layout_3,
                            symbol_4,
                            layout_5,
                            lit_6,
                            layout_7,
                            lit_8,
                        ] = <[ParseTree; 9usize]>::try_from(children).unwrap();
                        Symbol::PlusSep {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            symbol_2: Box::new(symbol_2.unwrap_symbol()),
                            layout_3: layout_3.unwrap_token(),
                            symbol_4: Box::new(symbol_4.unwrap_symbol()),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            layout_7: layout_7.unwrap_token(),
                            lit_8: lit_8.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "(" Layout Symbol+ Layout ")".
                    SlotId(85) => {
                        let [lit_0, layout_1, symbols, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Group {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            symbols: symbols.unwrap_alternative_plus_6(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : Identifier.
                    SlotId(87) => {
                        let [identifier_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Symbol::Identifier {
                            identifier_0: identifier_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //Regex : Regex Layout "+".
                    SlotId(91) => {
                        let [regex_0, layout_1, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Regex::Plus {
                            regex_0: Box::new(regex_0.unwrap_regex()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : Regex Layout "*".
                    SlotId(95) => {
                        let [regex_0, layout_1, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Regex::Star {
                            regex_0: Box::new(regex_0.unwrap_regex()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : Regex Layout "?".
                    SlotId(99) => {
                        let [regex_0, layout_1, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Regex::Opt {
                            regex_0: Box::new(regex_0.unwrap_regex()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : "(" Layout {Regex+ "|"}* Layout ")".
                    SlotId(105) => {
                        let [lit_0, layout_1, regex_star_5_2, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Regex::Alt {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            regex_star_5_2: regex_star_5_2.unwrap_regex_star_5(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : CharClass.
                    SlotId(107) => {
                        let [char_class_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Regex::CharClass {
                            char_class_0: char_class_0.unwrap_char_class(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : """ Layout Char Layout """.
                    SlotId(113) => {
                        let [lit_0, layout_1, char_2, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Regex::Char {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            char_2: char_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass
            NonterminalId(8) => {
                match nonterminal_node.return_slot {
                    //CharClass : "!"? Layout "[" Layout (Range | RangeChar)+ Layout "]".
                    SlotId(121) => {
                        let [
                            char_class_opt_7_0,
                            layout_1,
                            lit_2,
                            layout_3,
                            char_class_plus_7_4,
                            layout_5,
                            lit_6,
                        ] = <[ParseTree; 7usize]>::try_from(children).unwrap();
                        CharClass {
                            char_class_opt_7_0: char_class_opt_7_0.unwrap_char_class_opt_7(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            char_class_plus_7_4: char_class_plus_7_4.unwrap_char_class_plus_7(),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Range
            NonterminalId(9) => {
                match nonterminal_node.return_slot {
                    //Range : RangeChar Layout "-" Layout RangeChar.
                    SlotId(127) => {
                        let [range_char_0, layout_1, lit_2, layout_3, range_char_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Range {
                            range_char_0: range_char_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            range_char_4: range_char_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Plus_0
            NonterminalId(10) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule+ : SyntaxRule+ Layout SyntaxRule.
                    SlotId(131) => {
                        let [syntax_rules, layout_1, syntax_rule_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt0 {
                            syntax_rules: Box::new(syntax_rules.unwrap_grammar_plus_0()),
                            layout_1: layout_1.unwrap_token(),
                            syntax_rule_2: Box::new(syntax_rule_2.unwrap_syntax_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //SyntaxRule+ : SyntaxRule.
                    SlotId(133) => {
                        let [syntax_rule_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt1 {
                            syntax_rule_0: Box::new(syntax_rule_0.unwrap_syntax_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Opt_0
            NonterminalId(11) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule+? : SyntaxRule+.
                    SlotId(135) => {
                        let [syntax_rules] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt0 {
                            syntax_rules: syntax_rules.unwrap_grammar_plus_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //SyntaxRule+? : .
                    SlotId(136) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Star_0
            NonterminalId(12) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule* : SyntaxRule+?.
                    SlotId(138) => {
                        let [grammar_opt_0_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarStar0 {
                            grammar_opt_0_0: grammar_opt_0_0.unwrap_grammar_opt_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Opt_1
            NonterminalId(13) => {
                match nonterminal_node.return_slot {
                    //RegexBlock? : RegexBlock.
                    SlotId(140) => {
                        let [regex_block_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarOpt1::Alt0 {
                            regex_block_0: regex_block_0.unwrap_regex_block(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RegexBlock? : .
                    SlotId(141) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarOpt1::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule_Plus_1
            NonterminalId(14) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}+ : {PriorityLevel ">"}+ Layout ">" Layout PriorityLevel.
                    SlotId(147) => {
                        let [priority_levels, layout_1, lit_2, layout_3, priority_level_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        SyntaxRulePlus1::Alt0 {
                            priority_levels: Box::new(priority_levels.unwrap_syntax_rule_plus_1()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            priority_level_4: Box::new(priority_level_4.unwrap_priority_level()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{PriorityLevel ">"}+ : PriorityLevel.
                    SlotId(149) => {
                        let [priority_level_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SyntaxRulePlus1::Alt1 {
                            priority_level_0: Box::new(priority_level_0.unwrap_priority_level()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule_Opt_2
            NonterminalId(15) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}+? : {PriorityLevel ">"}+.
                    SlotId(151) => {
                        let [priority_levels] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SyntaxRuleOpt2::Alt0 {
                            priority_levels: priority_levels.unwrap_syntax_rule_plus_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{PriorityLevel ">"}+? : .
                    SlotId(152) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        SyntaxRuleOpt2::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule_Star_1
            NonterminalId(16) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}* : {PriorityLevel ">"}+?.
                    SlotId(154) => {
                        let [syntax_rule_opt_2_0] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SyntaxRuleStar1 {
                            syntax_rule_opt_2_0: syntax_rule_opt_2_0.unwrap_syntax_rule_opt_2(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock_Plus_2
            NonterminalId(17) => {
                match nonterminal_node.return_slot {
                    //RegexRule+ : RegexRule+ Layout RegexRule.
                    SlotId(158) => {
                        let [regex_rules, layout_1, regex_rule_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RegexBlockPlus2::Alt0 {
                            regex_rules: Box::new(regex_rules.unwrap_regex_block_plus_2()),
                            layout_1: layout_1.unwrap_token(),
                            regex_rule_2: Box::new(regex_rule_2.unwrap_regex_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RegexRule+ : RegexRule.
                    SlotId(160) => {
                        let [regex_rule_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexBlockPlus2::Alt1 {
                            regex_rule_0: Box::new(regex_rule_0.unwrap_regex_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock_Opt_3
            NonterminalId(18) => {
                match nonterminal_node.return_slot {
                    //RegexRule+? : RegexRule+.
                    SlotId(162) => {
                        let [regex_rules] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexBlockOpt3::Alt0 {
                            regex_rules: regex_rules.unwrap_regex_block_plus_2(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RegexRule+? : .
                    SlotId(163) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RegexBlockOpt3::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock_Star_2
            NonterminalId(19) => {
                match nonterminal_node.return_slot {
                    //RegexRule* : RegexRule+?.
                    SlotId(165) => {
                        let [regex_block_opt_3_0] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexBlockStar2 {
                            regex_block_opt_3_0: regex_block_opt_3_0.unwrap_regex_block_opt_3(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Plus_4
            NonterminalId(20) => {
                match nonterminal_node.return_slot {
                    //Regex+ : Regex+ Layout Regex.
                    SlotId(169) => {
                        let [regexes, layout_1, regex_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RegexRulePlus4::Alt0 {
                            regexes: Box::new(regexes.unwrap_regex_rule_plus_4()),
                            layout_1: layout_1.unwrap_token(),
                            regex_2: Box::new(regex_2.unwrap_regex()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex+ : Regex.
                    SlotId(171) => {
                        let [regex_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRulePlus4::Alt1 {
                            regex_0: Box::new(regex_0.unwrap_regex()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Plus_3
            NonterminalId(21) => {
                match nonterminal_node.return_slot {
                    //{Regex+ "|"}+ : {Regex+ "|"}+ Layout "|" Layout Regex+.
                    SlotId(177) => {
                        let [regex_rule_plus_3_0, layout_1, lit_2, layout_3, regexes] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        RegexRulePlus3::Alt0 {
                            regex_rule_plus_3_0: Box::new(
                                regex_rule_plus_3_0.unwrap_regex_rule_plus_3(),
                            ),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            regexes: regexes.unwrap_regex_rule_plus_4(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{Regex+ "|"}+ : Regex+.
                    SlotId(179) => {
                        let [regexes] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRulePlus3::Alt1 {
                            regexes: regexes.unwrap_regex_rule_plus_4(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Plus_5
            NonterminalId(22) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}+ : {Alternative "|"}+ Layout "|" Layout Alternative.
                    SlotId(185) => {
                        let [alternatives, layout_1, lit_2, layout_3, alternative_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        PriorityLevelPlus5::Alt0 {
                            alternatives: Box::new(alternatives.unwrap_priority_level_plus_5()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            alternative_4: Box::new(alternative_4.unwrap_alternative()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{Alternative "|"}+ : Alternative.
                    SlotId(187) => {
                        let [alternative_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelPlus5::Alt1 {
                            alternative_0: Box::new(alternative_0.unwrap_alternative()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Opt_4
            NonterminalId(23) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}+? : {Alternative "|"}+.
                    SlotId(189) => {
                        let [alternatives] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelOpt4::Alt0 {
                            alternatives: alternatives.unwrap_priority_level_plus_5(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{Alternative "|"}+? : .
                    SlotId(190) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        PriorityLevelOpt4::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Star_3
            NonterminalId(24) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}* : {Alternative "|"}+?.
                    SlotId(192) => {
                        let [priority_level_opt_4_0] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelStar3 {
                            priority_level_opt_4_0: priority_level_opt_4_0
                                .unwrap_priority_level_opt_4(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Plus_6
            NonterminalId(25) => {
                match nonterminal_node.return_slot {
                    //Symbol+ : Symbol+ Layout Symbol.
                    SlotId(196) => {
                        let [symbols, layout_1, symbol_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        AlternativePlus6::Alt0 {
                            symbols: Box::new(symbols.unwrap_alternative_plus_6()),
                            layout_1: layout_1.unwrap_token(),
                            symbol_2: Box::new(symbol_2.unwrap_symbol()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol+ : Symbol.
                    SlotId(198) => {
                        let [symbol_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativePlus6::Alt1 {
                            symbol_0: Box::new(symbol_0.unwrap_symbol()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Opt_5
            NonterminalId(26) => {
                match nonterminal_node.return_slot {
                    //Symbol+? : Symbol+.
                    SlotId(200) => {
                        let [symbols] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeOpt5::Alt0 {
                            symbols: symbols.unwrap_alternative_plus_6(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol+? : .
                    SlotId(201) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        AlternativeOpt5::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Star_4
            NonterminalId(27) => {
                match nonterminal_node.return_slot {
                    //Symbol* : Symbol+?.
                    SlotId(203) => {
                        let [alternative_opt_5_0] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeStar4 {
                            alternative_opt_5_0: alternative_opt_5_0.unwrap_alternative_opt_5(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex_Opt_6
            NonterminalId(28) => {
                match nonterminal_node.return_slot {
                    //{Regex+ "|"}+? : {Regex+ "|"}+.
                    SlotId(205) => {
                        let [regex_rule_plus_3_0] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexOpt6::Alt0 {
                            regex_rule_plus_3_0: regex_rule_plus_3_0.unwrap_regex_rule_plus_3(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{Regex+ "|"}+? : .
                    SlotId(206) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RegexOpt6::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex_Star_5
            NonterminalId(29) => {
                match nonterminal_node.return_slot {
                    //{Regex+ "|"}* : {Regex+ "|"}+?.
                    SlotId(208) => {
                        let [regex_opt_6_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexStar5 {
                            regex_opt_6_0: regex_opt_6_0.unwrap_regex_opt_6(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Opt_7
            NonterminalId(30) => {
                match nonterminal_node.return_slot {
                    //"!"? : "!".
                    SlotId(210) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassOpt7::Alt0 {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //"!"? : .
                    SlotId(211) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        CharClassOpt7::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Alt_0
            NonterminalId(31) => {
                match nonterminal_node.return_slot {
                    //(Range | RangeChar) : Range.
                    SlotId(213) => {
                        let [range_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassAlt0::Alt0 {
                            range_0: range_0.unwrap_range(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //(Range | RangeChar) : RangeChar.
                    SlotId(215) => {
                        let [range_char_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassAlt0::Alt1 {
                            range_char_0: range_char_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Plus_7
            NonterminalId(32) => {
                match nonterminal_node.return_slot {
                    //(Range | RangeChar)+ : (Range | RangeChar)+ Layout (Range | RangeChar).
                    SlotId(219) => {
                        let [char_class_plus_7_0, layout_1, char_class_alt_0_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        CharClassPlus7::Alt0 {
                            char_class_plus_7_0: Box::new(
                                char_class_plus_7_0.unwrap_char_class_plus_7(),
                            ),
                            layout_1: layout_1.unwrap_token(),
                            char_class_alt_0_2: char_class_alt_0_2.unwrap_char_class_alt_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //(Range | RangeChar)+ : (Range | RangeChar).
                    SlotId(221) => {
                        let [char_class_alt_0_0] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassPlus7::Alt1 {
                            char_class_alt_0_0: char_class_alt_0_0.unwrap_char_class_alt_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartGrammar
            NonterminalId(33) => {
                match nonterminal_node.return_slot {
                    //StartGrammar : Layout Grammar Layout.
                    SlotId(225) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartGrammar {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_grammar(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartSyntaxRule
            NonterminalId(34) => {
                match nonterminal_node.return_slot {
                    //StartSyntaxRule : Layout SyntaxRule Layout.
                    SlotId(229) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartSyntaxRule {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_syntax_rule(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartRegexBlock
            NonterminalId(35) => {
                match nonterminal_node.return_slot {
                    //StartRegexBlock : Layout RegexBlock Layout.
                    SlotId(233) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartRegexBlock {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_regex_block(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartRegexRule
            NonterminalId(36) => {
                match nonterminal_node.return_slot {
                    //StartRegexRule : Layout RegexRule Layout.
                    SlotId(237) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartRegexRule {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_regex_rule(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartPriorityLevel
            NonterminalId(37) => {
                match nonterminal_node.return_slot {
                    //StartPriorityLevel : Layout PriorityLevel Layout.
                    SlotId(241) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartPriorityLevel {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_priority_level(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartAlternative
            NonterminalId(38) => {
                match nonterminal_node.return_slot {
                    //StartAlternative : Layout Alternative Layout.
                    SlotId(245) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartAlternative {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_alternative(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartSymbol
            NonterminalId(39) => {
                match nonterminal_node.return_slot {
                    //StartSymbol : Layout Symbol Layout.
                    SlotId(249) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartSymbol {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_symbol(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartRegex
            NonterminalId(40) => {
                match nonterminal_node.return_slot {
                    //StartRegex : Layout Regex Layout.
                    SlotId(253) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartRegex {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_regex(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartCharClass
            NonterminalId(41) => {
                match nonterminal_node.return_slot {
                    //StartCharClass : Layout CharClass Layout.
                    SlotId(257) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartCharClass {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_char_class(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartRange
            NonterminalId(42) => {
                match nonterminal_node.return_slot {
                    //StartRange : Layout Range Layout.
                    SlotId(261) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartRange {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_range(),
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
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> ParseTree {
    match name {
        "Grammar" => ParseTree::Grammar(create_parse_tree_grammar(root_id, parser, builder)),
        "SyntaxRule" => {
            ParseTree::SyntaxRule(create_parse_tree_syntax_rule(root_id, parser, builder))
        }
        "RegexBlock" => {
            ParseTree::RegexBlock(create_parse_tree_regex_block(root_id, parser, builder))
        }
        "RegexRule" => ParseTree::RegexRule(create_parse_tree_regex_rule(root_id, parser, builder)),
        "PriorityLevel" => {
            ParseTree::PriorityLevel(create_parse_tree_priority_level(root_id, parser, builder))
        }
        "Alternative" => {
            ParseTree::Alternative(create_parse_tree_alternative(root_id, parser, builder))
        }
        "Symbol" => ParseTree::Symbol(create_parse_tree_symbol(root_id, parser, builder)),
        "Regex" => ParseTree::Regex(create_parse_tree_regex(root_id, parser, builder)),
        "CharClass" => ParseTree::CharClass(create_parse_tree_char_class(root_id, parser, builder)),
        "Range" => ParseTree::Range(create_parse_tree_range(root_id, parser, builder)),
        "Grammar_Plus_0" => {
            ParseTree::GrammarPlus0(create_parse_tree_grammar_plus_0(root_id, parser, builder))
        }
        "Grammar_Opt_0" => {
            ParseTree::GrammarOpt0(create_parse_tree_grammar_opt_0(root_id, parser, builder))
        }
        "Grammar_Star_0" => {
            ParseTree::GrammarStar0(create_parse_tree_grammar_star_0(root_id, parser, builder))
        }
        "Grammar_Opt_1" => {
            ParseTree::GrammarOpt1(create_parse_tree_grammar_opt_1(root_id, parser, builder))
        }
        "SyntaxRule_Plus_1" => ParseTree::SyntaxRulePlus1(create_parse_tree_syntax_rule_plus_1(
            root_id, parser, builder,
        )),
        "SyntaxRule_Opt_2" => ParseTree::SyntaxRuleOpt2(create_parse_tree_syntax_rule_opt_2(
            root_id, parser, builder,
        )),
        "SyntaxRule_Star_1" => ParseTree::SyntaxRuleStar1(create_parse_tree_syntax_rule_star_1(
            root_id, parser, builder,
        )),
        "RegexBlock_Plus_2" => ParseTree::RegexBlockPlus2(create_parse_tree_regex_block_plus_2(
            root_id, parser, builder,
        )),
        "RegexBlock_Opt_3" => ParseTree::RegexBlockOpt3(create_parse_tree_regex_block_opt_3(
            root_id, parser, builder,
        )),
        "RegexBlock_Star_2" => ParseTree::RegexBlockStar2(create_parse_tree_regex_block_star_2(
            root_id, parser, builder,
        )),
        "RegexRule_Plus_4" => ParseTree::RegexRulePlus4(create_parse_tree_regex_rule_plus_4(
            root_id, parser, builder,
        )),
        "RegexRule_Plus_3" => ParseTree::RegexRulePlus3(create_parse_tree_regex_rule_plus_3(
            root_id, parser, builder,
        )),
        "PriorityLevel_Plus_5" => ParseTree::PriorityLevelPlus5(
            create_parse_tree_priority_level_plus_5(root_id, parser, builder),
        ),
        "PriorityLevel_Opt_4" => ParseTree::PriorityLevelOpt4(
            create_parse_tree_priority_level_opt_4(root_id, parser, builder),
        ),
        "PriorityLevel_Star_3" => ParseTree::PriorityLevelStar3(
            create_parse_tree_priority_level_star_3(root_id, parser, builder),
        ),
        "Alternative_Plus_6" => ParseTree::AlternativePlus6(create_parse_tree_alternative_plus_6(
            root_id, parser, builder,
        )),
        "Alternative_Opt_5" => ParseTree::AlternativeOpt5(create_parse_tree_alternative_opt_5(
            root_id, parser, builder,
        )),
        "Alternative_Star_4" => ParseTree::AlternativeStar4(create_parse_tree_alternative_star_4(
            root_id, parser, builder,
        )),
        "Regex_Opt_6" => {
            ParseTree::RegexOpt6(create_parse_tree_regex_opt_6(root_id, parser, builder))
        }
        "Regex_Star_5" => {
            ParseTree::RegexStar5(create_parse_tree_regex_star_5(root_id, parser, builder))
        }
        "CharClass_Opt_7" => {
            ParseTree::CharClassOpt7(create_parse_tree_char_class_opt_7(root_id, parser, builder))
        }
        "CharClass_Alt_0" => {
            ParseTree::CharClassAlt0(create_parse_tree_char_class_alt_0(root_id, parser, builder))
        }
        "CharClass_Plus_7" => ParseTree::CharClassPlus7(create_parse_tree_char_class_plus_7(
            root_id, parser, builder,
        )),
        "StartGrammar" => {
            ParseTree::StartGrammar(create_parse_tree_start_grammar(root_id, parser, builder))
        }
        "StartSyntaxRule" => ParseTree::StartSyntaxRule(create_parse_tree_start_syntax_rule(
            root_id, parser, builder,
        )),
        "StartRegexBlock" => ParseTree::StartRegexBlock(create_parse_tree_start_regex_block(
            root_id, parser, builder,
        )),
        "StartRegexRule" => {
            ParseTree::StartRegexRule(create_parse_tree_start_regex_rule(root_id, parser, builder))
        }
        "StartPriorityLevel" => ParseTree::StartPriorityLevel(
            create_parse_tree_start_priority_level(root_id, parser, builder),
        ),
        "StartAlternative" => ParseTree::StartAlternative(create_parse_tree_start_alternative(
            root_id, parser, builder,
        )),
        "StartSymbol" => {
            ParseTree::StartSymbol(create_parse_tree_start_symbol(root_id, parser, builder))
        }
        "StartRegex" => {
            ParseTree::StartRegex(create_parse_tree_start_regex(root_id, parser, builder))
        }
        "StartCharClass" => {
            ParseTree::StartCharClass(create_parse_tree_start_char_class(root_id, parser, builder))
        }
        "StartRange" => {
            ParseTree::StartRange(create_parse_tree_start_range(root_id, parser, builder))
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
pub fn create_parse_tree_syntax_rule(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRule {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule()
}
pub fn create_parse_tree_regex_block(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexBlock {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_block()
}
pub fn create_parse_tree_regex_rule(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRule {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule()
}
pub fn create_parse_tree_priority_level(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevel {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level()
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
pub fn create_parse_tree_char_class(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClass {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_char_class()
}
pub fn create_parse_tree_range(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Range {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_range()
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
pub fn create_parse_tree_grammar_opt_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> GrammarOpt1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_grammar_opt_1()
}
pub fn create_parse_tree_syntax_rule_plus_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRulePlus1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule_plus_1()
}
pub fn create_parse_tree_syntax_rule_opt_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRuleOpt2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule_opt_2()
}
pub fn create_parse_tree_syntax_rule_star_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRuleStar1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule_star_1()
}
pub fn create_parse_tree_regex_block_plus_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexBlockPlus2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_block_plus_2()
}
pub fn create_parse_tree_regex_block_opt_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexBlockOpt3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_block_opt_3()
}
pub fn create_parse_tree_regex_block_star_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexBlockStar2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_block_star_2()
}
pub fn create_parse_tree_regex_rule_plus_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRulePlus4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule_plus_4()
}
pub fn create_parse_tree_regex_rule_plus_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRulePlus3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule_plus_3()
}
pub fn create_parse_tree_priority_level_plus_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelPlus5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level_plus_5()
}
pub fn create_parse_tree_priority_level_opt_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelOpt4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level_opt_4()
}
pub fn create_parse_tree_priority_level_star_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelStar3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level_star_3()
}
pub fn create_parse_tree_alternative_plus_6(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativePlus6 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_plus_6()
}
pub fn create_parse_tree_alternative_opt_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativeOpt5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_opt_5()
}
pub fn create_parse_tree_alternative_star_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativeStar4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_star_4()
}
pub fn create_parse_tree_regex_opt_6(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexOpt6 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_opt_6()
}
pub fn create_parse_tree_regex_star_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexStar5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_star_5()
}
pub fn create_parse_tree_char_class_opt_7(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassOpt7 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_char_class_opt_7()
}
pub fn create_parse_tree_char_class_alt_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassAlt0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_char_class_alt_0()
}
pub fn create_parse_tree_char_class_plus_7(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassPlus7 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_char_class_plus_7()
}
pub fn create_parse_tree_start_grammar(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartGrammar {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_grammar()
}
pub fn create_parse_tree_start_syntax_rule(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartSyntaxRule {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_syntax_rule()
}
pub fn create_parse_tree_start_regex_block(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartRegexBlock {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_regex_block()
}
pub fn create_parse_tree_start_regex_rule(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartRegexRule {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_regex_rule()
}
pub fn create_parse_tree_start_priority_level(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartPriorityLevel {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_priority_level()
}
pub fn create_parse_tree_start_alternative(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartAlternative {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_alternative()
}
pub fn create_parse_tree_start_symbol(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartSymbol {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_symbol()
}
pub fn create_parse_tree_start_regex(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartRegex {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_regex()
}
pub fn create_parse_tree_start_char_class(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartCharClass {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_char_class()
}
pub fn create_parse_tree_start_range(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartRange {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_range()
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

