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
pub enum TokenKind {
    //Keyword
    T0,
    //Identifier
    T1,
    //String
    T2,
    //RangeChar
    T3,
    //Char
    T4,
    //Label
    T5,
    //WS
    T6,
    //"grammar"
    T7,
    //"layout"
    T8,
    //"="
    T9,
    //">"
    T10,
    //"@NoLayout"
    T11,
    //"@Layout"
    T12,
    //"("
    T13,
    //")"
    T14,
    //"regex"
    T15,
    //"{"
    T16,
    //"}"
    T17,
    //"|"
    T18,
    //"!<<"
    T19,
    //"\"
    T20,
    //"!>>"
    T21,
    //"left"
    T22,
    //"right"
    T23,
    //"none"
    T24,
    //"""
    T25,
    //"*"
    T26,
    //"+"
    T27,
    //"?"
    T28,
    //":"
    T29,
    //"'"
    T30,
    //"!"
    T31,
    //"["
    T32,
    //"]"
    T33,
    //"-"
    T34,
    //Layout
    T35,
}
impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::T0 => "Keyword",
            TokenKind::T1 => "Identifier",
            TokenKind::T2 => "String",
            TokenKind::T3 => "RangeChar",
            TokenKind::T4 => "Char",
            TokenKind::T5 => "Label",
            TokenKind::T6 => "WS",
            TokenKind::T7 => "\"grammar\"",
            TokenKind::T8 => "\"layout\"",
            TokenKind::T9 => "\"=\"",
            TokenKind::T10 => "\">\"",
            TokenKind::T11 => "\"@NoLayout\"",
            TokenKind::T12 => "\"@Layout\"",
            TokenKind::T13 => "\"(\"",
            TokenKind::T14 => "\")\"",
            TokenKind::T15 => "\"regex\"",
            TokenKind::T16 => "\"{\"",
            TokenKind::T17 => "\"}\"",
            TokenKind::T18 => "\"|\"",
            TokenKind::T19 => "\"!<<\"",
            TokenKind::T20 => "\"\\\"",
            TokenKind::T21 => "\"!>>\"",
            TokenKind::T22 => "\"left\"",
            TokenKind::T23 => "\"right\"",
            TokenKind::T24 => "\"none\"",
            TokenKind::T25 => "\"\"\"",
            TokenKind::T26 => "\"*\"",
            TokenKind::T27 => "\"+\"",
            TokenKind::T28 => "\"?\"",
            TokenKind::T29 => "\":\"",
            TokenKind::T30 => "\"'\"",
            TokenKind::T31 => "\"!\"",
            TokenKind::T32 => "\"[\"",
            TokenKind::T33 => "\"]\"",
            TokenKind::T34 => "\"-\"",
            TokenKind::T35 => "Layout",
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum ParseTree {
    Grammar(Grammar),
    LayoutDef(LayoutDef),
    SyntaxRule(SyntaxRule),
    Annotation(Annotation),
    RegexBlock(RegexBlock),
    RegexRule(RegexRule),
    PreCondition(PreCondition),
    PostCondition(PostCondition),
    PriorityLevel(PriorityLevel),
    Associativity(Associativity),
    Alternative(Alternative),
    Symbol(Symbol),
    Regex(Regex),
    CharClass(CharClass),
    RangeElement(RangeElement),
    Range(Range),
    //LayoutDef?
    GrammarOpt0(GrammarOpt0),
    //SyntaxRule+
    GrammarPlus0(GrammarPlus0),
    //SyntaxRule+?
    GrammarOpt1(GrammarOpt1),
    //SyntaxRule*
    GrammarStar0(GrammarStar0),
    //RegexBlock?
    GrammarOpt2(GrammarOpt2),
    //Identifier+
    LayoutDefPlus1(LayoutDefPlus1),
    //Identifier+?
    LayoutDefOpt3(LayoutDefOpt3),
    //Identifier*
    LayoutDefStar1(LayoutDefStar1),
    //Annotation?
    SyntaxRuleOpt4(SyntaxRuleOpt4),
    //{PriorityLevel ">"}+
    SyntaxRulePlus2(SyntaxRulePlus2),
    //{PriorityLevel ">"}+?
    SyntaxRuleOpt5(SyntaxRuleOpt5),
    //{PriorityLevel ">"}*
    SyntaxRuleStar2(SyntaxRuleStar2),
    //RegexRule+
    RegexBlockPlus3(RegexBlockPlus3),
    //RegexRule+?
    RegexBlockOpt6(RegexBlockOpt6),
    //RegexRule*
    RegexBlockStar3(RegexBlockStar3),
    //PreCondition?
    RegexRuleOpt7(RegexRuleOpt7),
    //Regex+
    RegexRulePlus5(RegexRulePlus5),
    //{Regex+ "|"}+
    RegexRulePlus4(RegexRulePlus4),
    //PostCondition+
    RegexRulePlus6(RegexRulePlus6),
    //PostCondition+?
    RegexRuleOpt8(RegexRuleOpt8),
    //PostCondition*
    RegexRuleStar4(RegexRuleStar4),
    //Associativity?
    PriorityLevelOpt9(PriorityLevelOpt9),
    //{Alternative "|"}+
    PriorityLevelPlus7(PriorityLevelPlus7),
    //{Alternative "|"}+?
    PriorityLevelOpt10(PriorityLevelOpt10),
    //{Alternative "|"}*
    PriorityLevelStar5(PriorityLevelStar5),
    //Symbol+
    AlternativePlus8(AlternativePlus8),
    //Symbol+?
    AlternativeOpt11(AlternativeOpt11),
    //Symbol*
    AlternativeStar6(AlternativeStar6),
    //Label?
    AlternativeOpt12(AlternativeOpt12),
    //("|" Symbol)
    SymbolGroup0(SymbolGroup0),
    //("|" Symbol)+
    SymbolPlus9(SymbolPlus9),
    //("|" Regex)
    RegexGroup1(RegexGroup1),
    //("|" Regex)+
    RegexPlus10(RegexPlus10),
    //"!"?
    CharClassOpt13(CharClassOpt13),
    //RangeElement+
    CharClassPlus11(CharClassPlus11),
    StartGrammar(StartGrammar),
    StartLayoutDef(StartLayoutDef),
    StartSyntaxRule(StartSyntaxRule),
    StartAnnotation(StartAnnotation),
    StartRegexBlock(StartRegexBlock),
    StartRegexRule(StartRegexRule),
    StartPreCondition(StartPreCondition),
    StartPostCondition(StartPostCondition),
    StartPriorityLevel(StartPriorityLevel),
    StartAssociativity(StartAssociativity),
    StartAlternative(StartAlternative),
    StartSymbol(StartSymbol),
    StartRegex(StartRegex),
    StartCharClass(StartCharClass),
    StartRangeElement(StartRangeElement),
    StartRange(StartRange),
    Token(Token),
}
impl ParseTree {
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        match self {
            ParseTree::Grammar(grammar) => grammar.as_parse_tree_ref(),
            ParseTree::LayoutDef(layout_def) => layout_def.as_parse_tree_ref(),
            ParseTree::SyntaxRule(syntax_rule) => syntax_rule.as_parse_tree_ref(),
            ParseTree::Annotation(annotation) => annotation.as_parse_tree_ref(),
            ParseTree::RegexBlock(regex_block) => regex_block.as_parse_tree_ref(),
            ParseTree::RegexRule(regex_rule) => regex_rule.as_parse_tree_ref(),
            ParseTree::PreCondition(pre_condition) => pre_condition.as_parse_tree_ref(),
            ParseTree::PostCondition(post_condition) => post_condition.as_parse_tree_ref(),
            ParseTree::PriorityLevel(priority_level) => priority_level.as_parse_tree_ref(),
            ParseTree::Associativity(associativity) => associativity.as_parse_tree_ref(),
            ParseTree::Alternative(alternative) => alternative.as_parse_tree_ref(),
            ParseTree::Symbol(symbol) => symbol.as_parse_tree_ref(),
            ParseTree::Regex(regex) => regex.as_parse_tree_ref(),
            ParseTree::CharClass(char_class) => char_class.as_parse_tree_ref(),
            ParseTree::RangeElement(range_element) => range_element.as_parse_tree_ref(),
            ParseTree::Range(range) => range.as_parse_tree_ref(),
            ParseTree::GrammarOpt0(grammar_opt_0) => grammar_opt_0.as_parse_tree_ref(),
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0.as_parse_tree_ref(),
            ParseTree::GrammarOpt1(grammar_opt_1) => grammar_opt_1.as_parse_tree_ref(),
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0.as_parse_tree_ref(),
            ParseTree::GrammarOpt2(grammar_opt_2) => grammar_opt_2.as_parse_tree_ref(),
            ParseTree::LayoutDefPlus1(layout_def_plus_1) => layout_def_plus_1.as_parse_tree_ref(),
            ParseTree::LayoutDefOpt3(layout_def_opt_3) => layout_def_opt_3.as_parse_tree_ref(),
            ParseTree::LayoutDefStar1(layout_def_star_1) => layout_def_star_1.as_parse_tree_ref(),
            ParseTree::SyntaxRuleOpt4(syntax_rule_opt_4) => syntax_rule_opt_4.as_parse_tree_ref(),
            ParseTree::SyntaxRulePlus2(syntax_rule_plus_2) => {
                syntax_rule_plus_2.as_parse_tree_ref()
            }
            ParseTree::SyntaxRuleOpt5(syntax_rule_opt_5) => syntax_rule_opt_5.as_parse_tree_ref(),
            ParseTree::SyntaxRuleStar2(syntax_rule_star_2) => {
                syntax_rule_star_2.as_parse_tree_ref()
            }
            ParseTree::RegexBlockPlus3(regex_block_plus_3) => {
                regex_block_plus_3.as_parse_tree_ref()
            }
            ParseTree::RegexBlockOpt6(regex_block_opt_6) => regex_block_opt_6.as_parse_tree_ref(),
            ParseTree::RegexBlockStar3(regex_block_star_3) => {
                regex_block_star_3.as_parse_tree_ref()
            }
            ParseTree::RegexRuleOpt7(regex_rule_opt_7) => regex_rule_opt_7.as_parse_tree_ref(),
            ParseTree::RegexRulePlus5(regex_rule_plus_5) => regex_rule_plus_5.as_parse_tree_ref(),
            ParseTree::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.as_parse_tree_ref(),
            ParseTree::RegexRulePlus6(regex_rule_plus_6) => regex_rule_plus_6.as_parse_tree_ref(),
            ParseTree::RegexRuleOpt8(regex_rule_opt_8) => regex_rule_opt_8.as_parse_tree_ref(),
            ParseTree::RegexRuleStar4(regex_rule_star_4) => regex_rule_star_4.as_parse_tree_ref(),
            ParseTree::PriorityLevelOpt9(priority_level_opt_9) => {
                priority_level_opt_9.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelPlus7(priority_level_plus_7) => {
                priority_level_plus_7.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelOpt10(priority_level_opt_10) => {
                priority_level_opt_10.as_parse_tree_ref()
            }
            ParseTree::PriorityLevelStar5(priority_level_star_5) => {
                priority_level_star_5.as_parse_tree_ref()
            }
            ParseTree::AlternativePlus8(alternative_plus_8) => {
                alternative_plus_8.as_parse_tree_ref()
            }
            ParseTree::AlternativeOpt11(alternative_opt_11) => {
                alternative_opt_11.as_parse_tree_ref()
            }
            ParseTree::AlternativeStar6(alternative_star_6) => {
                alternative_star_6.as_parse_tree_ref()
            }
            ParseTree::AlternativeOpt12(alternative_opt_12) => {
                alternative_opt_12.as_parse_tree_ref()
            }
            ParseTree::SymbolGroup0(symbol_group_0) => symbol_group_0.as_parse_tree_ref(),
            ParseTree::SymbolPlus9(symbol_plus_9) => symbol_plus_9.as_parse_tree_ref(),
            ParseTree::RegexGroup1(regex_group_1) => regex_group_1.as_parse_tree_ref(),
            ParseTree::RegexPlus10(regex_plus_10) => regex_plus_10.as_parse_tree_ref(),
            ParseTree::CharClassOpt13(char_class_opt_13) => char_class_opt_13.as_parse_tree_ref(),
            ParseTree::CharClassPlus11(char_class_plus_11) => {
                char_class_plus_11.as_parse_tree_ref()
            }
            ParseTree::StartGrammar(start_grammar) => start_grammar.as_parse_tree_ref(),
            ParseTree::StartLayoutDef(start_layout_def) => start_layout_def.as_parse_tree_ref(),
            ParseTree::StartSyntaxRule(start_syntax_rule) => start_syntax_rule.as_parse_tree_ref(),
            ParseTree::StartAnnotation(start_annotation) => start_annotation.as_parse_tree_ref(),
            ParseTree::StartRegexBlock(start_regex_block) => start_regex_block.as_parse_tree_ref(),
            ParseTree::StartRegexRule(start_regex_rule) => start_regex_rule.as_parse_tree_ref(),
            ParseTree::StartPreCondition(start_pre_condition) => {
                start_pre_condition.as_parse_tree_ref()
            }
            ParseTree::StartPostCondition(start_post_condition) => {
                start_post_condition.as_parse_tree_ref()
            }
            ParseTree::StartPriorityLevel(start_priority_level) => {
                start_priority_level.as_parse_tree_ref()
            }
            ParseTree::StartAssociativity(start_associativity) => {
                start_associativity.as_parse_tree_ref()
            }
            ParseTree::StartAlternative(start_alternative) => start_alternative.as_parse_tree_ref(),
            ParseTree::StartSymbol(start_symbol) => start_symbol.as_parse_tree_ref(),
            ParseTree::StartRegex(start_regex) => start_regex.as_parse_tree_ref(),
            ParseTree::StartCharClass(start_char_class) => start_char_class.as_parse_tree_ref(),
            ParseTree::StartRangeElement(start_range_element) => {
                start_range_element.as_parse_tree_ref()
            }
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
    fn unwrap_layout_def(self) -> LayoutDef {
        match self {
            ParseTree::LayoutDef(layout_def) => layout_def,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule(self) -> SyntaxRule {
        match self {
            ParseTree::SyntaxRule(syntax_rule) => syntax_rule,
            _ => panic!(),
        }
    }
    fn unwrap_annotation(self) -> Annotation {
        match self {
            ParseTree::Annotation(annotation) => annotation,
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
    fn unwrap_pre_condition(self) -> PreCondition {
        match self {
            ParseTree::PreCondition(pre_condition) => pre_condition,
            _ => panic!(),
        }
    }
    fn unwrap_post_condition(self) -> PostCondition {
        match self {
            ParseTree::PostCondition(post_condition) => post_condition,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level(self) -> PriorityLevel {
        match self {
            ParseTree::PriorityLevel(priority_level) => priority_level,
            _ => panic!(),
        }
    }
    fn unwrap_associativity(self) -> Associativity {
        match self {
            ParseTree::Associativity(associativity) => associativity,
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
    fn unwrap_range_element(self) -> RangeElement {
        match self {
            ParseTree::RangeElement(range_element) => range_element,
            _ => panic!(),
        }
    }
    fn unwrap_range(self) -> Range {
        match self {
            ParseTree::Range(range) => range,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_opt_0(self) -> GrammarOpt0 {
        match self {
            ParseTree::GrammarOpt0(grammar_opt_0) => grammar_opt_0,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_plus_0(self) -> GrammarPlus0 {
        match self {
            ParseTree::GrammarPlus0(grammar_plus_0) => grammar_plus_0,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_opt_1(self) -> GrammarOpt1 {
        match self {
            ParseTree::GrammarOpt1(grammar_opt_1) => grammar_opt_1,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_star_0(self) -> GrammarStar0 {
        match self {
            ParseTree::GrammarStar0(grammar_star_0) => grammar_star_0,
            _ => panic!(),
        }
    }
    fn unwrap_grammar_opt_2(self) -> GrammarOpt2 {
        match self {
            ParseTree::GrammarOpt2(grammar_opt_2) => grammar_opt_2,
            _ => panic!(),
        }
    }
    fn unwrap_layout_def_plus_1(self) -> LayoutDefPlus1 {
        match self {
            ParseTree::LayoutDefPlus1(layout_def_plus_1) => layout_def_plus_1,
            _ => panic!(),
        }
    }
    fn unwrap_layout_def_opt_3(self) -> LayoutDefOpt3 {
        match self {
            ParseTree::LayoutDefOpt3(layout_def_opt_3) => layout_def_opt_3,
            _ => panic!(),
        }
    }
    fn unwrap_layout_def_star_1(self) -> LayoutDefStar1 {
        match self {
            ParseTree::LayoutDefStar1(layout_def_star_1) => layout_def_star_1,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule_opt_4(self) -> SyntaxRuleOpt4 {
        match self {
            ParseTree::SyntaxRuleOpt4(syntax_rule_opt_4) => syntax_rule_opt_4,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule_plus_2(self) -> SyntaxRulePlus2 {
        match self {
            ParseTree::SyntaxRulePlus2(syntax_rule_plus_2) => syntax_rule_plus_2,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule_opt_5(self) -> SyntaxRuleOpt5 {
        match self {
            ParseTree::SyntaxRuleOpt5(syntax_rule_opt_5) => syntax_rule_opt_5,
            _ => panic!(),
        }
    }
    fn unwrap_syntax_rule_star_2(self) -> SyntaxRuleStar2 {
        match self {
            ParseTree::SyntaxRuleStar2(syntax_rule_star_2) => syntax_rule_star_2,
            _ => panic!(),
        }
    }
    fn unwrap_regex_block_plus_3(self) -> RegexBlockPlus3 {
        match self {
            ParseTree::RegexBlockPlus3(regex_block_plus_3) => regex_block_plus_3,
            _ => panic!(),
        }
    }
    fn unwrap_regex_block_opt_6(self) -> RegexBlockOpt6 {
        match self {
            ParseTree::RegexBlockOpt6(regex_block_opt_6) => regex_block_opt_6,
            _ => panic!(),
        }
    }
    fn unwrap_regex_block_star_3(self) -> RegexBlockStar3 {
        match self {
            ParseTree::RegexBlockStar3(regex_block_star_3) => regex_block_star_3,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_opt_7(self) -> RegexRuleOpt7 {
        match self {
            ParseTree::RegexRuleOpt7(regex_rule_opt_7) => regex_rule_opt_7,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_plus_5(self) -> RegexRulePlus5 {
        match self {
            ParseTree::RegexRulePlus5(regex_rule_plus_5) => regex_rule_plus_5,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_plus_4(self) -> RegexRulePlus4 {
        match self {
            ParseTree::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_plus_6(self) -> RegexRulePlus6 {
        match self {
            ParseTree::RegexRulePlus6(regex_rule_plus_6) => regex_rule_plus_6,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_opt_8(self) -> RegexRuleOpt8 {
        match self {
            ParseTree::RegexRuleOpt8(regex_rule_opt_8) => regex_rule_opt_8,
            _ => panic!(),
        }
    }
    fn unwrap_regex_rule_star_4(self) -> RegexRuleStar4 {
        match self {
            ParseTree::RegexRuleStar4(regex_rule_star_4) => regex_rule_star_4,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level_opt_9(self) -> PriorityLevelOpt9 {
        match self {
            ParseTree::PriorityLevelOpt9(priority_level_opt_9) => priority_level_opt_9,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level_plus_7(self) -> PriorityLevelPlus7 {
        match self {
            ParseTree::PriorityLevelPlus7(priority_level_plus_7) => priority_level_plus_7,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level_opt_10(self) -> PriorityLevelOpt10 {
        match self {
            ParseTree::PriorityLevelOpt10(priority_level_opt_10) => priority_level_opt_10,
            _ => panic!(),
        }
    }
    fn unwrap_priority_level_star_5(self) -> PriorityLevelStar5 {
        match self {
            ParseTree::PriorityLevelStar5(priority_level_star_5) => priority_level_star_5,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_plus_8(self) -> AlternativePlus8 {
        match self {
            ParseTree::AlternativePlus8(alternative_plus_8) => alternative_plus_8,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_opt_11(self) -> AlternativeOpt11 {
        match self {
            ParseTree::AlternativeOpt11(alternative_opt_11) => alternative_opt_11,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_star_6(self) -> AlternativeStar6 {
        match self {
            ParseTree::AlternativeStar6(alternative_star_6) => alternative_star_6,
            _ => panic!(),
        }
    }
    fn unwrap_alternative_opt_12(self) -> AlternativeOpt12 {
        match self {
            ParseTree::AlternativeOpt12(alternative_opt_12) => alternative_opt_12,
            _ => panic!(),
        }
    }
    fn unwrap_symbol_group_0(self) -> SymbolGroup0 {
        match self {
            ParseTree::SymbolGroup0(symbol_group_0) => symbol_group_0,
            _ => panic!(),
        }
    }
    fn unwrap_symbol_plus_9(self) -> SymbolPlus9 {
        match self {
            ParseTree::SymbolPlus9(symbol_plus_9) => symbol_plus_9,
            _ => panic!(),
        }
    }
    fn unwrap_regex_group_1(self) -> RegexGroup1 {
        match self {
            ParseTree::RegexGroup1(regex_group_1) => regex_group_1,
            _ => panic!(),
        }
    }
    fn unwrap_regex_plus_10(self) -> RegexPlus10 {
        match self {
            ParseTree::RegexPlus10(regex_plus_10) => regex_plus_10,
            _ => panic!(),
        }
    }
    fn unwrap_char_class_opt_13(self) -> CharClassOpt13 {
        match self {
            ParseTree::CharClassOpt13(char_class_opt_13) => char_class_opt_13,
            _ => panic!(),
        }
    }
    fn unwrap_char_class_plus_11(self) -> CharClassPlus11 {
        match self {
            ParseTree::CharClassPlus11(char_class_plus_11) => char_class_plus_11,
            _ => panic!(),
        }
    }
    fn unwrap_start_grammar(self) -> StartGrammar {
        match self {
            ParseTree::StartGrammar(start_grammar) => start_grammar,
            _ => panic!(),
        }
    }
    fn unwrap_start_layout_def(self) -> StartLayoutDef {
        match self {
            ParseTree::StartLayoutDef(start_layout_def) => start_layout_def,
            _ => panic!(),
        }
    }
    fn unwrap_start_syntax_rule(self) -> StartSyntaxRule {
        match self {
            ParseTree::StartSyntaxRule(start_syntax_rule) => start_syntax_rule,
            _ => panic!(),
        }
    }
    fn unwrap_start_annotation(self) -> StartAnnotation {
        match self {
            ParseTree::StartAnnotation(start_annotation) => start_annotation,
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
    fn unwrap_start_pre_condition(self) -> StartPreCondition {
        match self {
            ParseTree::StartPreCondition(start_pre_condition) => start_pre_condition,
            _ => panic!(),
        }
    }
    fn unwrap_start_post_condition(self) -> StartPostCondition {
        match self {
            ParseTree::StartPostCondition(start_post_condition) => start_post_condition,
            _ => panic!(),
        }
    }
    fn unwrap_start_priority_level(self) -> StartPriorityLevel {
        match self {
            ParseTree::StartPriorityLevel(start_priority_level) => start_priority_level,
            _ => panic!(),
        }
    }
    fn unwrap_start_associativity(self) -> StartAssociativity {
        match self {
            ParseTree::StartAssociativity(start_associativity) => start_associativity,
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
    fn unwrap_start_range_element(self) -> StartRangeElement {
        match self {
            ParseTree::StartRangeElement(start_range_element) => start_range_element,
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
    LayoutDef(&'a LayoutDef),
    SyntaxRule(&'a SyntaxRule),
    Annotation(&'a Annotation),
    RegexBlock(&'a RegexBlock),
    RegexRule(&'a RegexRule),
    PreCondition(&'a PreCondition),
    PostCondition(&'a PostCondition),
    PriorityLevel(&'a PriorityLevel),
    Associativity(&'a Associativity),
    Alternative(&'a Alternative),
    Symbol(&'a Symbol),
    Regex(&'a Regex),
    CharClass(&'a CharClass),
    RangeElement(&'a RangeElement),
    Range(&'a Range),
    GrammarOpt0(&'a GrammarOpt0),
    GrammarPlus0(&'a GrammarPlus0),
    GrammarOpt1(&'a GrammarOpt1),
    GrammarStar0(&'a GrammarStar0),
    GrammarOpt2(&'a GrammarOpt2),
    LayoutDefPlus1(&'a LayoutDefPlus1),
    LayoutDefOpt3(&'a LayoutDefOpt3),
    LayoutDefStar1(&'a LayoutDefStar1),
    SyntaxRuleOpt4(&'a SyntaxRuleOpt4),
    SyntaxRulePlus2(&'a SyntaxRulePlus2),
    SyntaxRuleOpt5(&'a SyntaxRuleOpt5),
    SyntaxRuleStar2(&'a SyntaxRuleStar2),
    RegexBlockPlus3(&'a RegexBlockPlus3),
    RegexBlockOpt6(&'a RegexBlockOpt6),
    RegexBlockStar3(&'a RegexBlockStar3),
    RegexRuleOpt7(&'a RegexRuleOpt7),
    RegexRulePlus5(&'a RegexRulePlus5),
    RegexRulePlus4(&'a RegexRulePlus4),
    RegexRulePlus6(&'a RegexRulePlus6),
    RegexRuleOpt8(&'a RegexRuleOpt8),
    RegexRuleStar4(&'a RegexRuleStar4),
    PriorityLevelOpt9(&'a PriorityLevelOpt9),
    PriorityLevelPlus7(&'a PriorityLevelPlus7),
    PriorityLevelOpt10(&'a PriorityLevelOpt10),
    PriorityLevelStar5(&'a PriorityLevelStar5),
    AlternativePlus8(&'a AlternativePlus8),
    AlternativeOpt11(&'a AlternativeOpt11),
    AlternativeStar6(&'a AlternativeStar6),
    AlternativeOpt12(&'a AlternativeOpt12),
    SymbolGroup0(&'a SymbolGroup0),
    SymbolPlus9(&'a SymbolPlus9),
    RegexGroup1(&'a RegexGroup1),
    RegexPlus10(&'a RegexPlus10),
    CharClassOpt13(&'a CharClassOpt13),
    CharClassPlus11(&'a CharClassPlus11),
    StartGrammar(&'a StartGrammar),
    StartLayoutDef(&'a StartLayoutDef),
    StartSyntaxRule(&'a StartSyntaxRule),
    StartAnnotation(&'a StartAnnotation),
    StartRegexBlock(&'a StartRegexBlock),
    StartRegexRule(&'a StartRegexRule),
    StartPreCondition(&'a StartPreCondition),
    StartPostCondition(&'a StartPostCondition),
    StartPriorityLevel(&'a StartPriorityLevel),
    StartAssociativity(&'a StartAssociativity),
    StartAlternative(&'a StartAlternative),
    StartSymbol(&'a StartSymbol),
    StartRegex(&'a StartRegex),
    StartCharClass(&'a StartCharClass),
    StartRangeElement(&'a StartRangeElement),
    StartRange(&'a StartRange),
    Token(&'a Token),
}
impl<'a> ParseTreeRef<'a> {
    pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
        match self {
            ParseTreeRef::Grammar(grammar) => (0..grammar.child_count())
                .filter_map(|i| grammar.child(i))
                .collect(),
            ParseTreeRef::LayoutDef(layout_def) => (0..layout_def.child_count())
                .filter_map(|i| layout_def.child(i))
                .collect(),
            ParseTreeRef::SyntaxRule(syntax_rule) => (0..syntax_rule.child_count())
                .filter_map(|i| syntax_rule.child(i))
                .collect(),
            ParseTreeRef::Annotation(annotation) => (0..annotation.child_count())
                .filter_map(|i| annotation.child(i))
                .collect(),
            ParseTreeRef::RegexBlock(regex_block) => (0..regex_block.child_count())
                .filter_map(|i| regex_block.child(i))
                .collect(),
            ParseTreeRef::RegexRule(regex_rule) => (0..regex_rule.child_count())
                .filter_map(|i| regex_rule.child(i))
                .collect(),
            ParseTreeRef::PreCondition(pre_condition) => (0..pre_condition.child_count())
                .filter_map(|i| pre_condition.child(i))
                .collect(),
            ParseTreeRef::PostCondition(post_condition) => (0..post_condition.child_count())
                .filter_map(|i| post_condition.child(i))
                .collect(),
            ParseTreeRef::PriorityLevel(priority_level) => (0..priority_level.child_count())
                .filter_map(|i| priority_level.child(i))
                .collect(),
            ParseTreeRef::Associativity(associativity) => (0..associativity.child_count())
                .filter_map(|i| associativity.child(i))
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
            ParseTreeRef::RangeElement(range_element) => (0..range_element.child_count())
                .filter_map(|i| range_element.child(i))
                .collect(),
            ParseTreeRef::Range(range) => (0..range.child_count())
                .filter_map(|i| range.child(i))
                .collect(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => (0..grammar_opt_0.child_count())
                .filter_map(|i| grammar_opt_0.child(i))
                .collect(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.iter().collect(),
            ParseTreeRef::GrammarOpt1(grammar_opt_1) => (0..grammar_opt_1.child_count())
                .filter_map(|i| grammar_opt_1.child(i))
                .collect(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.iter().collect(),
            ParseTreeRef::GrammarOpt2(grammar_opt_2) => (0..grammar_opt_2.child_count())
                .filter_map(|i| grammar_opt_2.child(i))
                .collect(),
            ParseTreeRef::LayoutDefPlus1(layout_def_plus_1) => layout_def_plus_1.iter().collect(),
            ParseTreeRef::LayoutDefOpt3(layout_def_opt_3) => (0..layout_def_opt_3.child_count())
                .filter_map(|i| layout_def_opt_3.child(i))
                .collect(),
            ParseTreeRef::LayoutDefStar1(layout_def_star_1) => layout_def_star_1.iter().collect(),
            ParseTreeRef::SyntaxRuleOpt4(syntax_rule_opt_4) => (0..syntax_rule_opt_4.child_count())
                .filter_map(|i| syntax_rule_opt_4.child(i))
                .collect(),
            ParseTreeRef::SyntaxRulePlus2(syntax_rule_plus_2) => {
                syntax_rule_plus_2.iter().collect()
            }
            ParseTreeRef::SyntaxRuleOpt5(syntax_rule_opt_5) => (0..syntax_rule_opt_5.child_count())
                .filter_map(|i| syntax_rule_opt_5.child(i))
                .collect(),
            ParseTreeRef::SyntaxRuleStar2(syntax_rule_star_2) => {
                syntax_rule_star_2.iter().collect()
            }
            ParseTreeRef::RegexBlockPlus3(regex_block_plus_3) => {
                regex_block_plus_3.iter().collect()
            }
            ParseTreeRef::RegexBlockOpt6(regex_block_opt_6) => (0..regex_block_opt_6.child_count())
                .filter_map(|i| regex_block_opt_6.child(i))
                .collect(),
            ParseTreeRef::RegexBlockStar3(regex_block_star_3) => {
                regex_block_star_3.iter().collect()
            }
            ParseTreeRef::RegexRuleOpt7(regex_rule_opt_7) => (0..regex_rule_opt_7.child_count())
                .filter_map(|i| regex_rule_opt_7.child(i))
                .collect(),
            ParseTreeRef::RegexRulePlus5(regex_rule_plus_5) => regex_rule_plus_5.iter().collect(),
            ParseTreeRef::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.iter().collect(),
            ParseTreeRef::RegexRulePlus6(regex_rule_plus_6) => regex_rule_plus_6.iter().collect(),
            ParseTreeRef::RegexRuleOpt8(regex_rule_opt_8) => (0..regex_rule_opt_8.child_count())
                .filter_map(|i| regex_rule_opt_8.child(i))
                .collect(),
            ParseTreeRef::RegexRuleStar4(regex_rule_star_4) => regex_rule_star_4.iter().collect(),
            ParseTreeRef::PriorityLevelOpt9(priority_level_opt_9) => (0..priority_level_opt_9
                .child_count())
                .filter_map(|i| priority_level_opt_9.child(i))
                .collect(),
            ParseTreeRef::PriorityLevelPlus7(priority_level_plus_7) => {
                priority_level_plus_7.iter().collect()
            }
            ParseTreeRef::PriorityLevelOpt10(priority_level_opt_10) => (0..priority_level_opt_10
                .child_count())
                .filter_map(|i| priority_level_opt_10.child(i))
                .collect(),
            ParseTreeRef::PriorityLevelStar5(priority_level_star_5) => {
                priority_level_star_5.iter().collect()
            }
            ParseTreeRef::AlternativePlus8(alternative_plus_8) => {
                alternative_plus_8.iter().collect()
            }
            ParseTreeRef::AlternativeOpt11(alternative_opt_11) => (0..alternative_opt_11
                .child_count())
                .filter_map(|i| alternative_opt_11.child(i))
                .collect(),
            ParseTreeRef::AlternativeStar6(alternative_star_6) => {
                alternative_star_6.iter().collect()
            }
            ParseTreeRef::AlternativeOpt12(alternative_opt_12) => (0..alternative_opt_12
                .child_count())
                .filter_map(|i| alternative_opt_12.child(i))
                .collect(),
            ParseTreeRef::SymbolGroup0(symbol_group_0) => (0..symbol_group_0.child_count())
                .filter_map(|i| symbol_group_0.child(i))
                .collect(),
            ParseTreeRef::SymbolPlus9(symbol_plus_9) => symbol_plus_9.iter().collect(),
            ParseTreeRef::RegexGroup1(regex_group_1) => (0..regex_group_1.child_count())
                .filter_map(|i| regex_group_1.child(i))
                .collect(),
            ParseTreeRef::RegexPlus10(regex_plus_10) => regex_plus_10.iter().collect(),
            ParseTreeRef::CharClassOpt13(char_class_opt_13) => (0..char_class_opt_13.child_count())
                .filter_map(|i| char_class_opt_13.child(i))
                .collect(),
            ParseTreeRef::CharClassPlus11(char_class_plus_11) => {
                char_class_plus_11.iter().collect()
            }
            ParseTreeRef::StartGrammar(start_grammar) => (0..start_grammar.child_count())
                .filter_map(|i| start_grammar.child(i))
                .collect(),
            ParseTreeRef::StartLayoutDef(start_layout_def) => (0..start_layout_def.child_count())
                .filter_map(|i| start_layout_def.child(i))
                .collect(),
            ParseTreeRef::StartSyntaxRule(start_syntax_rule) => (0..start_syntax_rule
                .child_count())
                .filter_map(|i| start_syntax_rule.child(i))
                .collect(),
            ParseTreeRef::StartAnnotation(start_annotation) => (0..start_annotation.child_count())
                .filter_map(|i| start_annotation.child(i))
                .collect(),
            ParseTreeRef::StartRegexBlock(start_regex_block) => (0..start_regex_block
                .child_count())
                .filter_map(|i| start_regex_block.child(i))
                .collect(),
            ParseTreeRef::StartRegexRule(start_regex_rule) => (0..start_regex_rule.child_count())
                .filter_map(|i| start_regex_rule.child(i))
                .collect(),
            ParseTreeRef::StartPreCondition(start_pre_condition) => (0..start_pre_condition
                .child_count())
                .filter_map(|i| start_pre_condition.child(i))
                .collect(),
            ParseTreeRef::StartPostCondition(start_post_condition) => (0..start_post_condition
                .child_count())
                .filter_map(|i| start_post_condition.child(i))
                .collect(),
            ParseTreeRef::StartPriorityLevel(start_priority_level) => (0..start_priority_level
                .child_count())
                .filter_map(|i| start_priority_level.child(i))
                .collect(),
            ParseTreeRef::StartAssociativity(start_associativity) => (0..start_associativity
                .child_count())
                .filter_map(|i| start_associativity.child(i))
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
            ParseTreeRef::StartRangeElement(start_range_element) => (0..start_range_element
                .child_count())
                .filter_map(|i| start_range_element.child(i))
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
            ParseTreeRef::LayoutDef(_) => "LayoutDef",
            ParseTreeRef::SyntaxRule(_) => "SyntaxRule",
            ParseTreeRef::Annotation(_) => "Annotation",
            ParseTreeRef::RegexBlock(_) => "RegexBlock",
            ParseTreeRef::RegexRule(_) => "RegexRule",
            ParseTreeRef::PreCondition(_) => "PreCondition",
            ParseTreeRef::PostCondition(_) => "PostCondition",
            ParseTreeRef::PriorityLevel(_) => "PriorityLevel",
            ParseTreeRef::Associativity(_) => "Associativity",
            ParseTreeRef::Alternative(_) => "Alternative",
            ParseTreeRef::Symbol(_) => "Symbol",
            ParseTreeRef::Regex(_) => "Regex",
            ParseTreeRef::CharClass(_) => "CharClass",
            ParseTreeRef::RangeElement(_) => "RangeElement",
            ParseTreeRef::Range(_) => "Range",
            ParseTreeRef::GrammarOpt0(_) => "LayoutDef?",
            ParseTreeRef::GrammarPlus0(_) => "SyntaxRule+",
            ParseTreeRef::GrammarOpt1(_) => "SyntaxRule+?",
            ParseTreeRef::GrammarStar0(_) => "SyntaxRule*",
            ParseTreeRef::GrammarOpt2(_) => "RegexBlock?",
            ParseTreeRef::LayoutDefPlus1(_) => "Identifier+",
            ParseTreeRef::LayoutDefOpt3(_) => "Identifier+?",
            ParseTreeRef::LayoutDefStar1(_) => "Identifier*",
            ParseTreeRef::SyntaxRuleOpt4(_) => "Annotation?",
            ParseTreeRef::SyntaxRulePlus2(_) => "{PriorityLevel \">\"}+",
            ParseTreeRef::SyntaxRuleOpt5(_) => "{PriorityLevel \">\"}+?",
            ParseTreeRef::SyntaxRuleStar2(_) => "{PriorityLevel \">\"}*",
            ParseTreeRef::RegexBlockPlus3(_) => "RegexRule+",
            ParseTreeRef::RegexBlockOpt6(_) => "RegexRule+?",
            ParseTreeRef::RegexBlockStar3(_) => "RegexRule*",
            ParseTreeRef::RegexRuleOpt7(_) => "PreCondition?",
            ParseTreeRef::RegexRulePlus5(_) => "Regex+",
            ParseTreeRef::RegexRulePlus4(_) => "{Regex+ \"|\"}+",
            ParseTreeRef::RegexRulePlus6(_) => "PostCondition+",
            ParseTreeRef::RegexRuleOpt8(_) => "PostCondition+?",
            ParseTreeRef::RegexRuleStar4(_) => "PostCondition*",
            ParseTreeRef::PriorityLevelOpt9(_) => "Associativity?",
            ParseTreeRef::PriorityLevelPlus7(_) => "{Alternative \"|\"}+",
            ParseTreeRef::PriorityLevelOpt10(_) => "{Alternative \"|\"}+?",
            ParseTreeRef::PriorityLevelStar5(_) => "{Alternative \"|\"}*",
            ParseTreeRef::AlternativePlus8(_) => "Symbol+",
            ParseTreeRef::AlternativeOpt11(_) => "Symbol+?",
            ParseTreeRef::AlternativeStar6(_) => "Symbol*",
            ParseTreeRef::AlternativeOpt12(_) => "Label?",
            ParseTreeRef::SymbolGroup0(_) => "(\"|\" Symbol)",
            ParseTreeRef::SymbolPlus9(_) => "(\"|\" Symbol)+",
            ParseTreeRef::RegexGroup1(_) => "(\"|\" Regex)",
            ParseTreeRef::RegexPlus10(_) => "(\"|\" Regex)+",
            ParseTreeRef::CharClassOpt13(_) => "\"!\"?",
            ParseTreeRef::CharClassPlus11(_) => "RangeElement+",
            ParseTreeRef::StartGrammar(_) => "StartGrammar",
            ParseTreeRef::StartLayoutDef(_) => "StartLayoutDef",
            ParseTreeRef::StartSyntaxRule(_) => "StartSyntaxRule",
            ParseTreeRef::StartAnnotation(_) => "StartAnnotation",
            ParseTreeRef::StartRegexBlock(_) => "StartRegexBlock",
            ParseTreeRef::StartRegexRule(_) => "StartRegexRule",
            ParseTreeRef::StartPreCondition(_) => "StartPreCondition",
            ParseTreeRef::StartPostCondition(_) => "StartPostCondition",
            ParseTreeRef::StartPriorityLevel(_) => "StartPriorityLevel",
            ParseTreeRef::StartAssociativity(_) => "StartAssociativity",
            ParseTreeRef::StartAlternative(_) => "StartAlternative",
            ParseTreeRef::StartSymbol(_) => "StartSymbol",
            ParseTreeRef::StartRegex(_) => "StartRegex",
            ParseTreeRef::StartCharClass(_) => "StartCharClass",
            ParseTreeRef::StartRangeElement(_) => "StartRangeElement",
            ParseTreeRef::StartRange(_) => "StartRange",
            ParseTreeRef::Token(token) => token.kind.name(),
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.child_count(),
            ParseTreeRef::LayoutDef(layout_def) => layout_def.child_count(),
            ParseTreeRef::SyntaxRule(syntax_rule) => syntax_rule.child_count(),
            ParseTreeRef::Annotation(annotation) => annotation.child_count(),
            ParseTreeRef::RegexBlock(regex_block) => regex_block.child_count(),
            ParseTreeRef::RegexRule(regex_rule) => regex_rule.child_count(),
            ParseTreeRef::PreCondition(pre_condition) => pre_condition.child_count(),
            ParseTreeRef::PostCondition(post_condition) => post_condition.child_count(),
            ParseTreeRef::PriorityLevel(priority_level) => priority_level.child_count(),
            ParseTreeRef::Associativity(associativity) => associativity.child_count(),
            ParseTreeRef::Alternative(alternative) => alternative.child_count(),
            ParseTreeRef::Symbol(symbol) => symbol.child_count(),
            ParseTreeRef::Regex(regex) => regex.child_count(),
            ParseTreeRef::CharClass(char_class) => char_class.child_count(),
            ParseTreeRef::RangeElement(range_element) => range_element.child_count(),
            ParseTreeRef::Range(range) => range.child_count(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.child_count(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.child_count(),
            ParseTreeRef::GrammarOpt1(grammar_opt_1) => grammar_opt_1.child_count(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.child_count(),
            ParseTreeRef::GrammarOpt2(grammar_opt_2) => grammar_opt_2.child_count(),
            ParseTreeRef::LayoutDefPlus1(layout_def_plus_1) => layout_def_plus_1.child_count(),
            ParseTreeRef::LayoutDefOpt3(layout_def_opt_3) => layout_def_opt_3.child_count(),
            ParseTreeRef::LayoutDefStar1(layout_def_star_1) => layout_def_star_1.child_count(),
            ParseTreeRef::SyntaxRuleOpt4(syntax_rule_opt_4) => syntax_rule_opt_4.child_count(),
            ParseTreeRef::SyntaxRulePlus2(syntax_rule_plus_2) => syntax_rule_plus_2.child_count(),
            ParseTreeRef::SyntaxRuleOpt5(syntax_rule_opt_5) => syntax_rule_opt_5.child_count(),
            ParseTreeRef::SyntaxRuleStar2(syntax_rule_star_2) => syntax_rule_star_2.child_count(),
            ParseTreeRef::RegexBlockPlus3(regex_block_plus_3) => regex_block_plus_3.child_count(),
            ParseTreeRef::RegexBlockOpt6(regex_block_opt_6) => regex_block_opt_6.child_count(),
            ParseTreeRef::RegexBlockStar3(regex_block_star_3) => regex_block_star_3.child_count(),
            ParseTreeRef::RegexRuleOpt7(regex_rule_opt_7) => regex_rule_opt_7.child_count(),
            ParseTreeRef::RegexRulePlus5(regex_rule_plus_5) => regex_rule_plus_5.child_count(),
            ParseTreeRef::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.child_count(),
            ParseTreeRef::RegexRulePlus6(regex_rule_plus_6) => regex_rule_plus_6.child_count(),
            ParseTreeRef::RegexRuleOpt8(regex_rule_opt_8) => regex_rule_opt_8.child_count(),
            ParseTreeRef::RegexRuleStar4(regex_rule_star_4) => regex_rule_star_4.child_count(),
            ParseTreeRef::PriorityLevelOpt9(priority_level_opt_9) => {
                priority_level_opt_9.child_count()
            }
            ParseTreeRef::PriorityLevelPlus7(priority_level_plus_7) => {
                priority_level_plus_7.child_count()
            }
            ParseTreeRef::PriorityLevelOpt10(priority_level_opt_10) => {
                priority_level_opt_10.child_count()
            }
            ParseTreeRef::PriorityLevelStar5(priority_level_star_5) => {
                priority_level_star_5.child_count()
            }
            ParseTreeRef::AlternativePlus8(alternative_plus_8) => alternative_plus_8.child_count(),
            ParseTreeRef::AlternativeOpt11(alternative_opt_11) => alternative_opt_11.child_count(),
            ParseTreeRef::AlternativeStar6(alternative_star_6) => alternative_star_6.child_count(),
            ParseTreeRef::AlternativeOpt12(alternative_opt_12) => alternative_opt_12.child_count(),
            ParseTreeRef::SymbolGroup0(symbol_group_0) => symbol_group_0.child_count(),
            ParseTreeRef::SymbolPlus9(symbol_plus_9) => symbol_plus_9.child_count(),
            ParseTreeRef::RegexGroup1(regex_group_1) => regex_group_1.child_count(),
            ParseTreeRef::RegexPlus10(regex_plus_10) => regex_plus_10.child_count(),
            ParseTreeRef::CharClassOpt13(char_class_opt_13) => char_class_opt_13.child_count(),
            ParseTreeRef::CharClassPlus11(char_class_plus_11) => char_class_plus_11.child_count(),
            ParseTreeRef::StartGrammar(start_grammar) => start_grammar.child_count(),
            ParseTreeRef::StartLayoutDef(start_layout_def) => start_layout_def.child_count(),
            ParseTreeRef::StartSyntaxRule(start_syntax_rule) => start_syntax_rule.child_count(),
            ParseTreeRef::StartAnnotation(start_annotation) => start_annotation.child_count(),
            ParseTreeRef::StartRegexBlock(start_regex_block) => start_regex_block.child_count(),
            ParseTreeRef::StartRegexRule(start_regex_rule) => start_regex_rule.child_count(),
            ParseTreeRef::StartPreCondition(start_pre_condition) => {
                start_pre_condition.child_count()
            }
            ParseTreeRef::StartPostCondition(start_post_condition) => {
                start_post_condition.child_count()
            }
            ParseTreeRef::StartPriorityLevel(start_priority_level) => {
                start_priority_level.child_count()
            }
            ParseTreeRef::StartAssociativity(start_associativity) => {
                start_associativity.child_count()
            }
            ParseTreeRef::StartAlternative(start_alternative) => start_alternative.child_count(),
            ParseTreeRef::StartSymbol(start_symbol) => start_symbol.child_count(),
            ParseTreeRef::StartRegex(start_regex) => start_regex.child_count(),
            ParseTreeRef::StartCharClass(start_char_class) => start_char_class.child_count(),
            ParseTreeRef::StartRangeElement(start_range_element) => {
                start_range_element.child_count()
            }
            ParseTreeRef::StartRange(start_range) => start_range.child_count(),
            ParseTreeRef::Token(_) => 0,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            ParseTreeRef::Grammar(grammar) => grammar.span(),
            ParseTreeRef::LayoutDef(layout_def) => layout_def.span(),
            ParseTreeRef::SyntaxRule(syntax_rule) => syntax_rule.span(),
            ParseTreeRef::Annotation(annotation) => annotation.span(),
            ParseTreeRef::RegexBlock(regex_block) => regex_block.span(),
            ParseTreeRef::RegexRule(regex_rule) => regex_rule.span(),
            ParseTreeRef::PreCondition(pre_condition) => pre_condition.span(),
            ParseTreeRef::PostCondition(post_condition) => post_condition.span(),
            ParseTreeRef::PriorityLevel(priority_level) => priority_level.span(),
            ParseTreeRef::Associativity(associativity) => associativity.span(),
            ParseTreeRef::Alternative(alternative) => alternative.span(),
            ParseTreeRef::Symbol(symbol) => symbol.span(),
            ParseTreeRef::Regex(regex) => regex.span(),
            ParseTreeRef::CharClass(char_class) => char_class.span(),
            ParseTreeRef::RangeElement(range_element) => range_element.span(),
            ParseTreeRef::Range(range) => range.span(),
            ParseTreeRef::GrammarOpt0(grammar_opt_0) => grammar_opt_0.span(),
            ParseTreeRef::GrammarPlus0(grammar_plus_0) => grammar_plus_0.span(),
            ParseTreeRef::GrammarOpt1(grammar_opt_1) => grammar_opt_1.span(),
            ParseTreeRef::GrammarStar0(grammar_star_0) => grammar_star_0.span(),
            ParseTreeRef::GrammarOpt2(grammar_opt_2) => grammar_opt_2.span(),
            ParseTreeRef::LayoutDefPlus1(layout_def_plus_1) => layout_def_plus_1.span(),
            ParseTreeRef::LayoutDefOpt3(layout_def_opt_3) => layout_def_opt_3.span(),
            ParseTreeRef::LayoutDefStar1(layout_def_star_1) => layout_def_star_1.span(),
            ParseTreeRef::SyntaxRuleOpt4(syntax_rule_opt_4) => syntax_rule_opt_4.span(),
            ParseTreeRef::SyntaxRulePlus2(syntax_rule_plus_2) => syntax_rule_plus_2.span(),
            ParseTreeRef::SyntaxRuleOpt5(syntax_rule_opt_5) => syntax_rule_opt_5.span(),
            ParseTreeRef::SyntaxRuleStar2(syntax_rule_star_2) => syntax_rule_star_2.span(),
            ParseTreeRef::RegexBlockPlus3(regex_block_plus_3) => regex_block_plus_3.span(),
            ParseTreeRef::RegexBlockOpt6(regex_block_opt_6) => regex_block_opt_6.span(),
            ParseTreeRef::RegexBlockStar3(regex_block_star_3) => regex_block_star_3.span(),
            ParseTreeRef::RegexRuleOpt7(regex_rule_opt_7) => regex_rule_opt_7.span(),
            ParseTreeRef::RegexRulePlus5(regex_rule_plus_5) => regex_rule_plus_5.span(),
            ParseTreeRef::RegexRulePlus4(regex_rule_plus_4) => regex_rule_plus_4.span(),
            ParseTreeRef::RegexRulePlus6(regex_rule_plus_6) => regex_rule_plus_6.span(),
            ParseTreeRef::RegexRuleOpt8(regex_rule_opt_8) => regex_rule_opt_8.span(),
            ParseTreeRef::RegexRuleStar4(regex_rule_star_4) => regex_rule_star_4.span(),
            ParseTreeRef::PriorityLevelOpt9(priority_level_opt_9) => priority_level_opt_9.span(),
            ParseTreeRef::PriorityLevelPlus7(priority_level_plus_7) => priority_level_plus_7.span(),
            ParseTreeRef::PriorityLevelOpt10(priority_level_opt_10) => priority_level_opt_10.span(),
            ParseTreeRef::PriorityLevelStar5(priority_level_star_5) => priority_level_star_5.span(),
            ParseTreeRef::AlternativePlus8(alternative_plus_8) => alternative_plus_8.span(),
            ParseTreeRef::AlternativeOpt11(alternative_opt_11) => alternative_opt_11.span(),
            ParseTreeRef::AlternativeStar6(alternative_star_6) => alternative_star_6.span(),
            ParseTreeRef::AlternativeOpt12(alternative_opt_12) => alternative_opt_12.span(),
            ParseTreeRef::SymbolGroup0(symbol_group_0) => symbol_group_0.span(),
            ParseTreeRef::SymbolPlus9(symbol_plus_9) => symbol_plus_9.span(),
            ParseTreeRef::RegexGroup1(regex_group_1) => regex_group_1.span(),
            ParseTreeRef::RegexPlus10(regex_plus_10) => regex_plus_10.span(),
            ParseTreeRef::CharClassOpt13(char_class_opt_13) => char_class_opt_13.span(),
            ParseTreeRef::CharClassPlus11(char_class_plus_11) => char_class_plus_11.span(),
            ParseTreeRef::StartGrammar(start_grammar) => start_grammar.span(),
            ParseTreeRef::StartLayoutDef(start_layout_def) => start_layout_def.span(),
            ParseTreeRef::StartSyntaxRule(start_syntax_rule) => start_syntax_rule.span(),
            ParseTreeRef::StartAnnotation(start_annotation) => start_annotation.span(),
            ParseTreeRef::StartRegexBlock(start_regex_block) => start_regex_block.span(),
            ParseTreeRef::StartRegexRule(start_regex_rule) => start_regex_rule.span(),
            ParseTreeRef::StartPreCondition(start_pre_condition) => start_pre_condition.span(),
            ParseTreeRef::StartPostCondition(start_post_condition) => start_post_condition.span(),
            ParseTreeRef::StartPriorityLevel(start_priority_level) => start_priority_level.span(),
            ParseTreeRef::StartAssociativity(start_associativity) => start_associativity.span(),
            ParseTreeRef::StartAlternative(start_alternative) => start_alternative.span(),
            ParseTreeRef::StartSymbol(start_symbol) => start_symbol.span(),
            ParseTreeRef::StartRegex(start_regex) => start_regex.span(),
            ParseTreeRef::StartCharClass(start_char_class) => start_char_class.span(),
            ParseTreeRef::StartRangeElement(start_range_element) => start_range_element.span(),
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
impl From<LayoutDef> for ParseTree {
    fn from(layout_def: LayoutDef) -> Self {
        ParseTree::LayoutDef(layout_def)
    }
}
impl From<SyntaxRule> for ParseTree {
    fn from(syntax_rule: SyntaxRule) -> Self {
        ParseTree::SyntaxRule(syntax_rule)
    }
}
impl From<Annotation> for ParseTree {
    fn from(annotation: Annotation) -> Self {
        ParseTree::Annotation(annotation)
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
impl From<PreCondition> for ParseTree {
    fn from(pre_condition: PreCondition) -> Self {
        ParseTree::PreCondition(pre_condition)
    }
}
impl From<PostCondition> for ParseTree {
    fn from(post_condition: PostCondition) -> Self {
        ParseTree::PostCondition(post_condition)
    }
}
impl From<PriorityLevel> for ParseTree {
    fn from(priority_level: PriorityLevel) -> Self {
        ParseTree::PriorityLevel(priority_level)
    }
}
impl From<Associativity> for ParseTree {
    fn from(associativity: Associativity) -> Self {
        ParseTree::Associativity(associativity)
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
impl From<RangeElement> for ParseTree {
    fn from(range_element: RangeElement) -> Self {
        ParseTree::RangeElement(range_element)
    }
}
impl From<Range> for ParseTree {
    fn from(range: Range) -> Self {
        ParseTree::Range(range)
    }
}
impl From<GrammarOpt0> for ParseTree {
    fn from(grammar_opt_0: GrammarOpt0) -> Self {
        ParseTree::GrammarOpt0(grammar_opt_0)
    }
}
impl From<GrammarPlus0> for ParseTree {
    fn from(grammar_plus_0: GrammarPlus0) -> Self {
        ParseTree::GrammarPlus0(grammar_plus_0)
    }
}
impl From<GrammarOpt1> for ParseTree {
    fn from(grammar_opt_1: GrammarOpt1) -> Self {
        ParseTree::GrammarOpt1(grammar_opt_1)
    }
}
impl From<GrammarStar0> for ParseTree {
    fn from(grammar_star_0: GrammarStar0) -> Self {
        ParseTree::GrammarStar0(grammar_star_0)
    }
}
impl From<GrammarOpt2> for ParseTree {
    fn from(grammar_opt_2: GrammarOpt2) -> Self {
        ParseTree::GrammarOpt2(grammar_opt_2)
    }
}
impl From<LayoutDefPlus1> for ParseTree {
    fn from(layout_def_plus_1: LayoutDefPlus1) -> Self {
        ParseTree::LayoutDefPlus1(layout_def_plus_1)
    }
}
impl From<LayoutDefOpt3> for ParseTree {
    fn from(layout_def_opt_3: LayoutDefOpt3) -> Self {
        ParseTree::LayoutDefOpt3(layout_def_opt_3)
    }
}
impl From<LayoutDefStar1> for ParseTree {
    fn from(layout_def_star_1: LayoutDefStar1) -> Self {
        ParseTree::LayoutDefStar1(layout_def_star_1)
    }
}
impl From<SyntaxRuleOpt4> for ParseTree {
    fn from(syntax_rule_opt_4: SyntaxRuleOpt4) -> Self {
        ParseTree::SyntaxRuleOpt4(syntax_rule_opt_4)
    }
}
impl From<SyntaxRulePlus2> for ParseTree {
    fn from(syntax_rule_plus_2: SyntaxRulePlus2) -> Self {
        ParseTree::SyntaxRulePlus2(syntax_rule_plus_2)
    }
}
impl From<SyntaxRuleOpt5> for ParseTree {
    fn from(syntax_rule_opt_5: SyntaxRuleOpt5) -> Self {
        ParseTree::SyntaxRuleOpt5(syntax_rule_opt_5)
    }
}
impl From<SyntaxRuleStar2> for ParseTree {
    fn from(syntax_rule_star_2: SyntaxRuleStar2) -> Self {
        ParseTree::SyntaxRuleStar2(syntax_rule_star_2)
    }
}
impl From<RegexBlockPlus3> for ParseTree {
    fn from(regex_block_plus_3: RegexBlockPlus3) -> Self {
        ParseTree::RegexBlockPlus3(regex_block_plus_3)
    }
}
impl From<RegexBlockOpt6> for ParseTree {
    fn from(regex_block_opt_6: RegexBlockOpt6) -> Self {
        ParseTree::RegexBlockOpt6(regex_block_opt_6)
    }
}
impl From<RegexBlockStar3> for ParseTree {
    fn from(regex_block_star_3: RegexBlockStar3) -> Self {
        ParseTree::RegexBlockStar3(regex_block_star_3)
    }
}
impl From<RegexRuleOpt7> for ParseTree {
    fn from(regex_rule_opt_7: RegexRuleOpt7) -> Self {
        ParseTree::RegexRuleOpt7(regex_rule_opt_7)
    }
}
impl From<RegexRulePlus5> for ParseTree {
    fn from(regex_rule_plus_5: RegexRulePlus5) -> Self {
        ParseTree::RegexRulePlus5(regex_rule_plus_5)
    }
}
impl From<RegexRulePlus4> for ParseTree {
    fn from(regex_rule_plus_4: RegexRulePlus4) -> Self {
        ParseTree::RegexRulePlus4(regex_rule_plus_4)
    }
}
impl From<RegexRulePlus6> for ParseTree {
    fn from(regex_rule_plus_6: RegexRulePlus6) -> Self {
        ParseTree::RegexRulePlus6(regex_rule_plus_6)
    }
}
impl From<RegexRuleOpt8> for ParseTree {
    fn from(regex_rule_opt_8: RegexRuleOpt8) -> Self {
        ParseTree::RegexRuleOpt8(regex_rule_opt_8)
    }
}
impl From<RegexRuleStar4> for ParseTree {
    fn from(regex_rule_star_4: RegexRuleStar4) -> Self {
        ParseTree::RegexRuleStar4(regex_rule_star_4)
    }
}
impl From<PriorityLevelOpt9> for ParseTree {
    fn from(priority_level_opt_9: PriorityLevelOpt9) -> Self {
        ParseTree::PriorityLevelOpt9(priority_level_opt_9)
    }
}
impl From<PriorityLevelPlus7> for ParseTree {
    fn from(priority_level_plus_7: PriorityLevelPlus7) -> Self {
        ParseTree::PriorityLevelPlus7(priority_level_plus_7)
    }
}
impl From<PriorityLevelOpt10> for ParseTree {
    fn from(priority_level_opt_10: PriorityLevelOpt10) -> Self {
        ParseTree::PriorityLevelOpt10(priority_level_opt_10)
    }
}
impl From<PriorityLevelStar5> for ParseTree {
    fn from(priority_level_star_5: PriorityLevelStar5) -> Self {
        ParseTree::PriorityLevelStar5(priority_level_star_5)
    }
}
impl From<AlternativePlus8> for ParseTree {
    fn from(alternative_plus_8: AlternativePlus8) -> Self {
        ParseTree::AlternativePlus8(alternative_plus_8)
    }
}
impl From<AlternativeOpt11> for ParseTree {
    fn from(alternative_opt_11: AlternativeOpt11) -> Self {
        ParseTree::AlternativeOpt11(alternative_opt_11)
    }
}
impl From<AlternativeStar6> for ParseTree {
    fn from(alternative_star_6: AlternativeStar6) -> Self {
        ParseTree::AlternativeStar6(alternative_star_6)
    }
}
impl From<AlternativeOpt12> for ParseTree {
    fn from(alternative_opt_12: AlternativeOpt12) -> Self {
        ParseTree::AlternativeOpt12(alternative_opt_12)
    }
}
impl From<SymbolGroup0> for ParseTree {
    fn from(symbol_group_0: SymbolGroup0) -> Self {
        ParseTree::SymbolGroup0(symbol_group_0)
    }
}
impl From<SymbolPlus9> for ParseTree {
    fn from(symbol_plus_9: SymbolPlus9) -> Self {
        ParseTree::SymbolPlus9(symbol_plus_9)
    }
}
impl From<RegexGroup1> for ParseTree {
    fn from(regex_group_1: RegexGroup1) -> Self {
        ParseTree::RegexGroup1(regex_group_1)
    }
}
impl From<RegexPlus10> for ParseTree {
    fn from(regex_plus_10: RegexPlus10) -> Self {
        ParseTree::RegexPlus10(regex_plus_10)
    }
}
impl From<CharClassOpt13> for ParseTree {
    fn from(char_class_opt_13: CharClassOpt13) -> Self {
        ParseTree::CharClassOpt13(char_class_opt_13)
    }
}
impl From<CharClassPlus11> for ParseTree {
    fn from(char_class_plus_11: CharClassPlus11) -> Self {
        ParseTree::CharClassPlus11(char_class_plus_11)
    }
}
impl From<StartGrammar> for ParseTree {
    fn from(start_grammar: StartGrammar) -> Self {
        ParseTree::StartGrammar(start_grammar)
    }
}
impl From<StartLayoutDef> for ParseTree {
    fn from(start_layout_def: StartLayoutDef) -> Self {
        ParseTree::StartLayoutDef(start_layout_def)
    }
}
impl From<StartSyntaxRule> for ParseTree {
    fn from(start_syntax_rule: StartSyntaxRule) -> Self {
        ParseTree::StartSyntaxRule(start_syntax_rule)
    }
}
impl From<StartAnnotation> for ParseTree {
    fn from(start_annotation: StartAnnotation) -> Self {
        ParseTree::StartAnnotation(start_annotation)
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
impl From<StartPreCondition> for ParseTree {
    fn from(start_pre_condition: StartPreCondition) -> Self {
        ParseTree::StartPreCondition(start_pre_condition)
    }
}
impl From<StartPostCondition> for ParseTree {
    fn from(start_post_condition: StartPostCondition) -> Self {
        ParseTree::StartPostCondition(start_post_condition)
    }
}
impl From<StartPriorityLevel> for ParseTree {
    fn from(start_priority_level: StartPriorityLevel) -> Self {
        ParseTree::StartPriorityLevel(start_priority_level)
    }
}
impl From<StartAssociativity> for ParseTree {
    fn from(start_associativity: StartAssociativity) -> Self {
        ParseTree::StartAssociativity(start_associativity)
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
impl From<StartRangeElement> for ParseTree {
    fn from(start_range_element: StartRangeElement) -> Self {
        ParseTree::StartRangeElement(start_range_element)
    }
}
impl From<StartRange> for ParseTree {
    fn from(start_range: StartRange) -> Self {
        ParseTree::StartRange(start_range)
    }
}
pub trait ListNode<'a> {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
}
pub trait OptNode {
    type Inner;
    fn value(&self) -> Option<&Self::Inner>;
}
//Grammar = "grammar" Layout name:Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?
#[derive(Debug)]
pub struct Grammar {
    pub lit_0: Token,
    pub layout_1: Token,
    pub name: Token,
    pub layout_3: Token,
    pub layout_def: GrammarOpt0,
    pub layout_5: Token,
    pub syntax_rules: GrammarStar0,
    pub layout_7: Token,
    pub regex_block: GrammarOpt2,
    pub span: Span,
}
//LayoutDef = "layout" Layout Identifier*
#[derive(Debug)]
pub struct LayoutDef {
    pub lit_0: Token,
    pub layout: Token,
    pub identifiers: LayoutDefStar1,
    pub span: Span,
}
//SyntaxRule = Annotation? Layout head:Identifier Layout "=" Layout {PriorityLevel ">"}*
#[derive(Debug)]
pub struct SyntaxRule {
    pub annotation: SyntaxRuleOpt4,
    pub layout_1: Token,
    pub head: Token,
    pub layout_3: Token,
    pub lit_4: Token,
    pub layout_5: Token,
    pub priority_levels: SyntaxRuleStar2,
    pub span: Span,
}
#[derive(Debug)]
pub enum Annotation {
    //"@NoLayout" #NoLayout
    NoLayout {
        lit_0: Token,
        span: Span,
    },
    //"@Layout" Layout "(" Layout Identifier Layout ")" #Layout
    Layout {
        lit_0: Token,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        identifier: Token,
        layout_5: Token,
        lit_6: Token,
        span: Span,
    },
}
//RegexBlock = "regex" Layout "{" Layout RegexRule* Layout "}"
#[derive(Debug)]
pub struct RegexBlock {
    pub lit_0: Token,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub regex_rules: RegexBlockStar3,
    pub layout_5: Token,
    pub lit_6: Token,
    pub span: Span,
}
//RegexRule = Identifier Layout "=" Layout PreCondition? Layout body:{Regex+ "|"}+ Layout PostCondition*
#[derive(Debug)]
pub struct RegexRule {
    pub identifier: Token,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub pre_condition: RegexRuleOpt7,
    pub layout_5: Token,
    pub body: RegexRulePlus4,
    pub layout_7: Token,
    pub post_conditions: RegexRuleStar4,
    pub span: Span,
}
//PreCondition = Identifier Layout "!<<" #PrecedeRestriction
#[derive(Debug)]
pub struct PreCondition {
    pub identifier: Token,
    pub layout: Token,
    pub lit_2: Token,
    pub span: Span,
}
#[derive(Debug)]
pub enum PostCondition {
    //"\" Layout Identifier #Except
    Except {
        lit_0: Token,
        layout: Token,
        identifier: Token,
        span: Span,
    },
    //"!>>" Layout Identifier #FollowRestriction
    FollowRestriction {
        lit_0: Token,
        layout: Token,
        identifier: Token,
        span: Span,
    },
}
//PriorityLevel = Associativity? Layout {Alternative "|"}*
#[derive(Debug)]
pub struct PriorityLevel {
    pub associativity: PriorityLevelOpt9,
    pub layout: Token,
    pub alternatives: PriorityLevelStar5,
    pub span: Span,
}
#[derive(Debug)]
pub enum Associativity {
    //"left"
    Alt0 { lit_0: Token, span: Span },
    //"right"
    Alt1 { lit_0: Token, span: Span },
    //"none"
    Alt2 { lit_0: Token, span: Span },
}
//Alternative = Symbol* Layout Label?
#[derive(Debug)]
pub struct Alternative {
    pub symbols: AlternativeStar6,
    pub layout: Token,
    pub label: AlternativeOpt12,
    pub span: Span,
}
#[derive(Debug)]
pub enum Symbol {
    //Identifier return 0 #Identifier
    Identifier {
        identifier: Token,
        span: Span,
    },
    //"(" Layout Symbol+ Layout ")" return 0 #Group
    Group {
        lit_0: Token,
        layout_1: Token,
        symbols: AlternativePlus8,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    //"(" Layout first:Symbol(0) Layout rest:("|" Symbol)+ Layout ")" return 0 #Alt
    Alt {
        lit_0: Token,
        layout_1: Token,
        first: Box<Symbol>,
        layout_3: Token,
        rest: SymbolPlus9,
        layout_5: Token,
        lit_6: Token,
        span: Span,
    },
    //""" Layout String Layout """ return 0 #Lit
    Lit {
        lit_0: Token,
        layout_1: Token,
        string: Token,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    //"{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0 #StarSep
    StarSep {
        lit_0: Token,
        layout_1: Token,
        symbol: Box<Symbol>,
        layout_3: Token,
        sep: Box<Symbol>,
        layout_5: Token,
        lit_6: Token,
        layout_7: Token,
        lit_8: Token,
        span: Span,
    },
    //"{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0 #PlusSep
    PlusSep {
        lit_0: Token,
        layout_1: Token,
        symbol: Box<Symbol>,
        layout_3: Token,
        sep: Box<Symbol>,
        layout_5: Token,
        lit_6: Token,
        layout_7: Token,
        lit_8: Token,
        span: Span,
    },
    //[3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0 #Star
    Star {
        symbol: Box<Symbol>,
        layout: Token,
        lit_2: Token,
        span: Span,
    },
    //[3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0 #Plus
    Plus {
        symbol: Box<Symbol>,
        layout: Token,
        lit_2: Token,
        span: Span,
    },
    //[3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0 #Opt
    Opt {
        symbol: Box<Symbol>,
        layout: Token,
        lit_2: Token,
        span: Span,
    },
    //[3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "\" Layout Identifier return 0 #Except
    Except {
        symbol: Box<Symbol>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        identifier: Token,
        span: Span,
    },
    //[3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0 #FollowRestriction
    FollowRestriction {
        symbol: Box<Symbol>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        identifier: Token,
        span: Span,
    },
    //Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2) #PrecedeRestriction
    PrecedeRestriction {
        identifier: Token,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        symbol: Box<Symbol>,
        span: Span,
    },
    //label:Identifier Layout ":" Layout Symbol(1) return 1 #Labeled
    Labeled {
        label: Token,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        symbol: Box<Symbol>,
        span: Span,
    },
}
#[derive(Debug)]
pub enum Regex {
    //Regex Layout "+" #Plus
    Plus {
        regex: Box<Regex>,
        layout: Token,
        lit_2: Token,
        span: Span,
    },
    //Regex Layout "*" #Star
    Star {
        regex: Box<Regex>,
        layout: Token,
        lit_2: Token,
        span: Span,
    },
    //Regex Layout "?" #Opt
    Opt {
        regex: Box<Regex>,
        layout: Token,
        lit_2: Token,
        span: Span,
    },
    //"(" Layout first:Regex Layout rest:("|" Regex)+ Layout ")" #Alt
    Alt {
        lit_0: Token,
        layout_1: Token,
        first: Box<Regex>,
        layout_3: Token,
        rest: RegexPlus10,
        layout_5: Token,
        lit_6: Token,
        span: Span,
    },
    //"(" Layout Regex+ Layout ")" #Group
    Group {
        lit_0: Token,
        layout_1: Token,
        regexes: RegexRulePlus5,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    //CharClass #CharClass
    CharClass {
        char_class: CharClass,
        span: Span,
    },
    //"'" Layout Char Layout "'" #Char
    Char {
        lit_0: Token,
        layout_1: Token,
        char: Token,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
    //""" Layout String Layout """ #String
    String {
        lit_0: Token,
        layout_1: Token,
        string: Token,
        layout_3: Token,
        lit_4: Token,
        span: Span,
    },
}
//CharClass = neg:"!"? Layout "[" Layout RangeElement+ Layout "]"
#[derive(Debug)]
pub struct CharClass {
    pub neg: CharClassOpt13,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub range_elements: CharClassPlus11,
    pub layout_5: Token,
    pub lit_6: Token,
    pub span: Span,
}
#[derive(Debug)]
pub enum RangeElement {
    //Range
    Alt0 { range: Range, span: Span },
    //RangeChar
    Alt1 { range_char: Token, span: Span },
}
//Range = start:RangeChar Layout "-" Layout end:RangeChar
#[derive(Debug)]
pub struct Range {
    pub start: Token,
    pub layout_1: Token,
    pub lit_2: Token,
    pub layout_3: Token,
    pub end: Token,
    pub span: Span,
}
//LayoutDef?
#[derive(Debug)]
pub enum GrammarOpt0 {
    //LayoutDef
    Alt0 {
        layout_def: Box<LayoutDef>,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//SyntaxRule+
#[derive(Debug)]
pub enum GrammarPlus0 {
    //SyntaxRule+ Layout SyntaxRule
    Alt0 {
        syntax_rules: Box<GrammarPlus0>,
        layout: Token,
        syntax_rule_2: Box<SyntaxRule>,
        span: Span,
    },
    //SyntaxRule
    Alt1 {
        syntax_rule: Box<SyntaxRule>,
        span: Span,
    },
}
//SyntaxRule+?
#[derive(Debug)]
pub enum GrammarOpt1 {
    //SyntaxRule+
    Alt0 {
        syntax_rules: GrammarPlus0,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//SyntaxRule*
#[derive(Debug)]
pub struct GrammarStar0 {
    pub grammar_opt_1: GrammarOpt1,
    pub span: Span,
}
//RegexBlock?
#[derive(Debug)]
pub enum GrammarOpt2 {
    //RegexBlock
    Alt0 {
        regex_block: Box<RegexBlock>,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//Identifier+
#[derive(Debug)]
pub enum LayoutDefPlus1 {
    //Identifier+ Layout Identifier
    Alt0 {
        identifiers: Box<LayoutDefPlus1>,
        layout: Token,
        identifier_2: Token,
        span: Span,
    },
    //Identifier
    Alt1 {
        identifier: Token,
        span: Span,
    },
}
//Identifier+?
#[derive(Debug)]
pub enum LayoutDefOpt3 {
    //Identifier+
    Alt0 {
        identifiers: LayoutDefPlus1,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//Identifier*
#[derive(Debug)]
pub struct LayoutDefStar1 {
    pub layout_def_opt_3: LayoutDefOpt3,
    pub span: Span,
}
//Annotation?
#[derive(Debug)]
pub enum SyntaxRuleOpt4 {
    //Annotation
    Alt0 {
        annotation: Box<Annotation>,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//{PriorityLevel ">"}+
#[derive(Debug)]
pub enum SyntaxRulePlus2 {
    //{PriorityLevel ">"}+ Layout ">" Layout PriorityLevel
    Alt0 {
        priority_levels: Box<SyntaxRulePlus2>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        priority_level_4: Box<PriorityLevel>,
        span: Span,
    },
    //PriorityLevel
    Alt1 {
        priority_level: Box<PriorityLevel>,
        span: Span,
    },
}
//{PriorityLevel ">"}+?
#[derive(Debug)]
pub enum SyntaxRuleOpt5 {
    //{PriorityLevel ">"}+
    Alt0 {
        priority_levels: SyntaxRulePlus2,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//{PriorityLevel ">"}*
#[derive(Debug)]
pub struct SyntaxRuleStar2 {
    pub syntax_rule_opt_5: SyntaxRuleOpt5,
    pub span: Span,
}
//RegexRule+
#[derive(Debug)]
pub enum RegexBlockPlus3 {
    //RegexRule+ Layout RegexRule
    Alt0 {
        regex_rules: Box<RegexBlockPlus3>,
        layout: Token,
        regex_rule_2: Box<RegexRule>,
        span: Span,
    },
    //RegexRule
    Alt1 {
        regex_rule: Box<RegexRule>,
        span: Span,
    },
}
//RegexRule+?
#[derive(Debug)]
pub enum RegexBlockOpt6 {
    //RegexRule+
    Alt0 {
        regex_rules: RegexBlockPlus3,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//RegexRule*
#[derive(Debug)]
pub struct RegexBlockStar3 {
    pub regex_block_opt_6: RegexBlockOpt6,
    pub span: Span,
}
//PreCondition?
#[derive(Debug)]
pub enum RegexRuleOpt7 {
    //PreCondition
    Alt0 {
        pre_condition: Box<PreCondition>,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//Regex+
#[derive(Debug)]
pub enum RegexRulePlus5 {
    //Regex+ Layout Regex
    Alt0 {
        regexes: Box<RegexRulePlus5>,
        layout: Token,
        regex_2: Box<Regex>,
        span: Span,
    },
    //Regex
    Alt1 {
        regex: Box<Regex>,
        span: Span,
    },
}
//{Regex+ "|"}+
#[derive(Debug)]
pub enum RegexRulePlus4 {
    //{Regex+ "|"}+ Layout "|" Layout Regex+
    Alt0 {
        regex_rule_plus_4: Box<RegexRulePlus4>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        regexes: RegexRulePlus5,
        span: Span,
    },
    //Regex+
    Alt1 {
        regexes: RegexRulePlus5,
        span: Span,
    },
}
//PostCondition+
#[derive(Debug)]
pub enum RegexRulePlus6 {
    //PostCondition+ Layout PostCondition
    Alt0 {
        post_conditions: Box<RegexRulePlus6>,
        layout: Token,
        post_condition_2: Box<PostCondition>,
        span: Span,
    },
    //PostCondition
    Alt1 {
        post_condition: Box<PostCondition>,
        span: Span,
    },
}
//PostCondition+?
#[derive(Debug)]
pub enum RegexRuleOpt8 {
    //PostCondition+
    Alt0 {
        post_conditions: RegexRulePlus6,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//PostCondition*
#[derive(Debug)]
pub struct RegexRuleStar4 {
    pub regex_rule_opt_8: RegexRuleOpt8,
    pub span: Span,
}
//Associativity?
#[derive(Debug)]
pub enum PriorityLevelOpt9 {
    //Associativity
    Alt0 {
        associativity: Box<Associativity>,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//{Alternative "|"}+
#[derive(Debug)]
pub enum PriorityLevelPlus7 {
    //{Alternative "|"}+ Layout "|" Layout Alternative
    Alt0 {
        alternatives: Box<PriorityLevelPlus7>,
        layout_1: Token,
        lit_2: Token,
        layout_3: Token,
        alternative_4: Box<Alternative>,
        span: Span,
    },
    //Alternative
    Alt1 {
        alternative: Box<Alternative>,
        span: Span,
    },
}
//{Alternative "|"}+?
#[derive(Debug)]
pub enum PriorityLevelOpt10 {
    //{Alternative "|"}+
    Alt0 {
        alternatives: PriorityLevelPlus7,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//{Alternative "|"}*
#[derive(Debug)]
pub struct PriorityLevelStar5 {
    pub priority_level_opt_10: PriorityLevelOpt10,
    pub span: Span,
}
//Symbol+
#[derive(Debug)]
pub enum AlternativePlus8 {
    //Symbol+ Layout Symbol(0)
    Alt0 {
        symbols: Box<AlternativePlus8>,
        layout: Token,
        symbol_2: Box<Symbol>,
        span: Span,
    },
    //Symbol(0)
    Alt1 {
        symbol: Box<Symbol>,
        span: Span,
    },
}
//Symbol+?
#[derive(Debug)]
pub enum AlternativeOpt11 {
    //Symbol+
    Alt0 {
        symbols: AlternativePlus8,
        span: Span,
    },
    //
    Alt1 {
        span: Span,
    },
}
//Symbol*
#[derive(Debug)]
pub struct AlternativeStar6 {
    pub alternative_opt_11: AlternativeOpt11,
    pub span: Span,
}
//Label?
#[derive(Debug)]
pub enum AlternativeOpt12 {
    //Label
    Alt0 { label: Token, span: Span },
    //
    Alt1 { span: Span },
}
//("|" Symbol)
#[derive(Debug)]
pub struct SymbolGroup0 {
    pub lit_0: Token,
    pub layout: Token,
    pub symbol: Box<Symbol>,
    pub span: Span,
}
//("|" Symbol)+
#[derive(Debug)]
pub enum SymbolPlus9 {
    //("|" Symbol)+ Layout ("|" Symbol)
    Alt0 {
        symbol_plus_9: Box<SymbolPlus9>,
        layout: Token,
        symbol_group_0: SymbolGroup0,
        span: Span,
    },
    //("|" Symbol)
    Alt1 {
        symbol_group_0: SymbolGroup0,
        span: Span,
    },
}
//("|" Regex)
#[derive(Debug)]
pub struct RegexGroup1 {
    pub lit_0: Token,
    pub layout: Token,
    pub regex: Box<Regex>,
    pub span: Span,
}
//("|" Regex)+
#[derive(Debug)]
pub enum RegexPlus10 {
    //("|" Regex)+ Layout ("|" Regex)
    Alt0 {
        regex_plus_10: Box<RegexPlus10>,
        layout: Token,
        regex_group_1: RegexGroup1,
        span: Span,
    },
    //("|" Regex)
    Alt1 {
        regex_group_1: RegexGroup1,
        span: Span,
    },
}
//"!"?
#[derive(Debug)]
pub enum CharClassOpt13 {
    //"!"
    Alt0 { lit_0: Token, span: Span },
    //
    Alt1 { span: Span },
}
//RangeElement+
#[derive(Debug)]
pub enum CharClassPlus11 {
    //RangeElement+ Layout RangeElement
    Alt0 {
        range_elements: Box<CharClassPlus11>,
        layout: Token,
        range_element_2: Box<RangeElement>,
        span: Span,
    },
    //RangeElement
    Alt1 {
        range_element: Box<RangeElement>,
        span: Span,
    },
}
//StartGrammar = Layout start:Grammar Layout
#[derive(Debug)]
pub struct StartGrammar {
    pub layout_0: Token,
    pub start: Grammar,
    pub layout_2: Token,
    pub span: Span,
}
//StartLayoutDef = Layout start:LayoutDef Layout
#[derive(Debug)]
pub struct StartLayoutDef {
    pub layout_0: Token,
    pub start: LayoutDef,
    pub layout_2: Token,
    pub span: Span,
}
//StartSyntaxRule = Layout start:SyntaxRule Layout
#[derive(Debug)]
pub struct StartSyntaxRule {
    pub layout_0: Token,
    pub start: SyntaxRule,
    pub layout_2: Token,
    pub span: Span,
}
//StartAnnotation = Layout start:Annotation Layout
#[derive(Debug)]
pub struct StartAnnotation {
    pub layout_0: Token,
    pub start: Annotation,
    pub layout_2: Token,
    pub span: Span,
}
//StartRegexBlock = Layout start:RegexBlock Layout
#[derive(Debug)]
pub struct StartRegexBlock {
    pub layout_0: Token,
    pub start: RegexBlock,
    pub layout_2: Token,
    pub span: Span,
}
//StartRegexRule = Layout start:RegexRule Layout
#[derive(Debug)]
pub struct StartRegexRule {
    pub layout_0: Token,
    pub start: RegexRule,
    pub layout_2: Token,
    pub span: Span,
}
//StartPreCondition = Layout start:PreCondition Layout
#[derive(Debug)]
pub struct StartPreCondition {
    pub layout_0: Token,
    pub start: PreCondition,
    pub layout_2: Token,
    pub span: Span,
}
//StartPostCondition = Layout start:PostCondition Layout
#[derive(Debug)]
pub struct StartPostCondition {
    pub layout_0: Token,
    pub start: PostCondition,
    pub layout_2: Token,
    pub span: Span,
}
//StartPriorityLevel = Layout start:PriorityLevel Layout
#[derive(Debug)]
pub struct StartPriorityLevel {
    pub layout_0: Token,
    pub start: PriorityLevel,
    pub layout_2: Token,
    pub span: Span,
}
//StartAssociativity = Layout start:Associativity Layout
#[derive(Debug)]
pub struct StartAssociativity {
    pub layout_0: Token,
    pub start: Associativity,
    pub layout_2: Token,
    pub span: Span,
}
//StartAlternative = Layout start:Alternative Layout
#[derive(Debug)]
pub struct StartAlternative {
    pub layout_0: Token,
    pub start: Alternative,
    pub layout_2: Token,
    pub span: Span,
}
//StartSymbol = Layout start:Symbol(0) Layout
#[derive(Debug)]
pub struct StartSymbol {
    pub layout_0: Token,
    pub start: Symbol,
    pub layout_2: Token,
    pub span: Span,
}
//StartRegex = Layout start:Regex Layout
#[derive(Debug)]
pub struct StartRegex {
    pub layout_0: Token,
    pub start: Regex,
    pub layout_2: Token,
    pub span: Span,
}
//StartCharClass = Layout start:CharClass Layout
#[derive(Debug)]
pub struct StartCharClass {
    pub layout_0: Token,
    pub start: CharClass,
    pub layout_2: Token,
    pub span: Span,
}
//StartRangeElement = Layout start:RangeElement Layout
#[derive(Debug)]
pub struct StartRangeElement {
    pub layout_0: Token,
    pub start: RangeElement,
    pub layout_2: Token,
    pub span: Span,
}
//StartRange = Layout start:Range Layout
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
            4 => Some(self.layout_def.as_parse_tree_ref()),
            5 => Some(self.layout_5.as_parse_tree_ref()),
            6 => Some(self.syntax_rules.as_parse_tree_ref()),
            7 => Some(self.layout_7.as_parse_tree_ref()),
            8 => Some(self.regex_block.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        9usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Grammar(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl LayoutDef {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.lit_0.as_parse_tree_ref()),
            1 => Some(self.layout.as_parse_tree_ref()),
            2 => Some(self.identifiers.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::LayoutDef(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl SyntaxRule {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.annotation.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.head.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.lit_4.as_parse_tree_ref()),
            5 => Some(self.layout_5.as_parse_tree_ref()),
            6 => Some(self.priority_levels.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        7usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRule(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl Annotation {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Annotation::NoLayout { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
            Annotation::Layout {
                lit_0,
                layout_1,
                lit_2,
                layout_3,
                identifier,
                layout_5,
                lit_6,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(identifier.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(lit_6.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Annotation::NoLayout { .. } => 1usize,
            Annotation::Layout { .. } => 7usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Annotation(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Annotation::NoLayout { span, .. } => *span,
            Annotation::Layout { span, .. } => *span,
        }
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
            0 => Some(self.identifier.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.pre_condition.as_parse_tree_ref()),
            5 => Some(self.layout_5.as_parse_tree_ref()),
            6 => Some(self.body.as_parse_tree_ref()),
            7 => Some(self.layout_7.as_parse_tree_ref()),
            8 => Some(self.post_conditions.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        9usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRule(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl PreCondition {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.identifier.as_parse_tree_ref()),
            1 => Some(self.layout.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PreCondition(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl PostCondition {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PostCondition::Except {
                lit_0,
                layout,
                identifier,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(identifier.as_parse_tree_ref()),
                _ => None,
            },
            PostCondition::FollowRestriction {
                lit_0,
                layout,
                identifier,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(identifier.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PostCondition::Except { .. } => 3usize,
            PostCondition::FollowRestriction { .. } => 3usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PostCondition(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PostCondition::Except { span, .. } => *span,
            PostCondition::FollowRestriction { span, .. } => *span,
        }
    }
}
impl PriorityLevel {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.associativity.as_parse_tree_ref()),
            1 => Some(self.layout.as_parse_tree_ref()),
            2 => Some(self.alternatives.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevel(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl Associativity {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Associativity::Alt0 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
            Associativity::Alt1 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
            Associativity::Alt2 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Associativity::Alt0 { .. } => 1usize,
            Associativity::Alt1 { .. } => 1usize,
            Associativity::Alt2 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Associativity(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Associativity::Alt0 { span, .. } => *span,
            Associativity::Alt1 { span, .. } => *span,
            Associativity::Alt2 { span, .. } => *span,
        }
    }
}
impl Alternative {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.symbols.as_parse_tree_ref()),
            1 => Some(self.layout.as_parse_tree_ref()),
            2 => Some(self.label.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
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
            Symbol::Identifier { identifier, .. } => match index {
                0 => Some(identifier.as_parse_tree_ref()),
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
            Symbol::Alt {
                lit_0,
                layout_1,
                first,
                layout_3,
                rest,
                layout_5,
                lit_6,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(first.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(rest.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(lit_6.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Lit {
                lit_0,
                layout_1,
                string,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(string.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::StarSep {
                lit_0,
                layout_1,
                symbol,
                layout_3,
                sep,
                layout_5,
                lit_6,
                layout_7,
                lit_8,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(symbol.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(sep.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(lit_6.as_parse_tree_ref()),
                7 => Some(layout_7.as_parse_tree_ref()),
                8 => Some(lit_8.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::PlusSep {
                lit_0,
                layout_1,
                symbol,
                layout_3,
                sep,
                layout_5,
                lit_6,
                layout_7,
                lit_8,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(symbol.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(sep.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(lit_6.as_parse_tree_ref()),
                7 => Some(layout_7.as_parse_tree_ref()),
                8 => Some(lit_8.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Star {
                symbol,
                layout,
                lit_2,
                ..
            } => match index {
                0 => Some(symbol.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Plus {
                symbol,
                layout,
                lit_2,
                ..
            } => match index {
                0 => Some(symbol.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Opt {
                symbol,
                layout,
                lit_2,
                ..
            } => match index {
                0 => Some(symbol.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Except {
                symbol,
                layout_1,
                lit_2,
                layout_3,
                identifier,
                ..
            } => match index {
                0 => Some(symbol.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(identifier.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::FollowRestriction {
                symbol,
                layout_1,
                lit_2,
                layout_3,
                identifier,
                ..
            } => match index {
                0 => Some(symbol.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(identifier.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::PrecedeRestriction {
                identifier,
                layout_1,
                lit_2,
                layout_3,
                symbol,
                ..
            } => match index {
                0 => Some(identifier.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(symbol.as_parse_tree_ref()),
                _ => None,
            },
            Symbol::Labeled {
                label,
                layout_1,
                lit_2,
                layout_3,
                symbol,
                ..
            } => match index {
                0 => Some(label.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(symbol.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            Symbol::Identifier { .. } => 1usize,
            Symbol::Group { .. } => 5usize,
            Symbol::Alt { .. } => 7usize,
            Symbol::Lit { .. } => 5usize,
            Symbol::StarSep { .. } => 9usize,
            Symbol::PlusSep { .. } => 9usize,
            Symbol::Star { .. } => 3usize,
            Symbol::Plus { .. } => 3usize,
            Symbol::Opt { .. } => 3usize,
            Symbol::Except { .. } => 5usize,
            Symbol::FollowRestriction { .. } => 5usize,
            Symbol::PrecedeRestriction { .. } => 5usize,
            Symbol::Labeled { .. } => 5usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::Symbol(self)
    }
    pub fn span(&self) -> Span {
        match self {
            Symbol::Identifier { span, .. } => *span,
            Symbol::Group { span, .. } => *span,
            Symbol::Alt { span, .. } => *span,
            Symbol::Lit { span, .. } => *span,
            Symbol::StarSep { span, .. } => *span,
            Symbol::PlusSep { span, .. } => *span,
            Symbol::Star { span, .. } => *span,
            Symbol::Plus { span, .. } => *span,
            Symbol::Opt { span, .. } => *span,
            Symbol::Except { span, .. } => *span,
            Symbol::FollowRestriction { span, .. } => *span,
            Symbol::PrecedeRestriction { span, .. } => *span,
            Symbol::Labeled { span, .. } => *span,
        }
    }
}
impl Regex {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            Regex::Plus {
                regex,
                layout,
                lit_2,
                ..
            } => match index {
                0 => Some(regex.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Star {
                regex,
                layout,
                lit_2,
                ..
            } => match index {
                0 => Some(regex.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Opt {
                regex,
                layout,
                lit_2,
                ..
            } => match index {
                0 => Some(regex.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Alt {
                lit_0,
                layout_1,
                first,
                layout_3,
                rest,
                layout_5,
                lit_6,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(first.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(rest.as_parse_tree_ref()),
                5 => Some(layout_5.as_parse_tree_ref()),
                6 => Some(lit_6.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Group {
                lit_0,
                layout_1,
                regexes,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(regexes.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            Regex::CharClass { char_class, .. } => match index {
                0 => Some(char_class.as_parse_tree_ref()),
                _ => None,
            },
            Regex::Char {
                lit_0,
                layout_1,
                char,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(char.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(lit_4.as_parse_tree_ref()),
                _ => None,
            },
            Regex::String {
                lit_0,
                layout_1,
                string,
                layout_3,
                lit_4,
                ..
            } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(string.as_parse_tree_ref()),
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
            Regex::Alt { .. } => 7usize,
            Regex::Group { .. } => 5usize,
            Regex::CharClass { .. } => 1usize,
            Regex::Char { .. } => 5usize,
            Regex::String { .. } => 5usize,
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
            Regex::Group { span, .. } => *span,
            Regex::CharClass { span, .. } => *span,
            Regex::Char { span, .. } => *span,
            Regex::String { span, .. } => *span,
        }
    }
}
impl CharClass {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.neg.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.range_elements.as_parse_tree_ref()),
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
impl RangeElement {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RangeElement::Alt0 { range, .. } => match index {
                0 => Some(range.as_parse_tree_ref()),
                _ => None,
            },
            RangeElement::Alt1 { range_char, .. } => match index {
                0 => Some(range_char.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RangeElement::Alt0 { .. } => 1usize,
            RangeElement::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RangeElement(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RangeElement::Alt0 { span, .. } => *span,
            RangeElement::Alt1 { span, .. } => *span,
        }
    }
}
impl Range {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.start.as_parse_tree_ref()),
            1 => Some(self.layout_1.as_parse_tree_ref()),
            2 => Some(self.lit_2.as_parse_tree_ref()),
            3 => Some(self.layout_3.as_parse_tree_ref()),
            4 => Some(self.end.as_parse_tree_ref()),
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
impl GrammarOpt0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarOpt0::Alt0 { layout_def, .. } => match index {
                0 => Some(layout_def.as_parse_tree_ref()),
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
impl GrammarPlus0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarPlus0::Alt0 {
                syntax_rules,
                layout,
                syntax_rule_2,
                ..
            } => match index {
                0 => Some(syntax_rules.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(syntax_rule_2.as_parse_tree_ref()),
                _ => None,
            },
            GrammarPlus0::Alt1 { syntax_rule, .. } => match index {
                0 => Some(syntax_rule.as_parse_tree_ref()),
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
impl GrammarOpt1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarOpt1::Alt0 { syntax_rules, .. } => match index {
                0 => Some(syntax_rules.as_parse_tree_ref()),
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
    pub fn syntax_rules(&self) -> impl Iterator<Item = &SyntaxRule> {
        self.value()
            .into_iter()
            .flat_map(|inner| inner.syntax_rules())
    }
}
impl GrammarStar0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.grammar_opt_1.as_parse_tree_ref()),
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
        self.grammar_opt_1.syntax_rules()
    }
}
impl GrammarOpt2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            GrammarOpt2::Alt0 { regex_block, .. } => match index {
                0 => Some(regex_block.as_parse_tree_ref()),
                _ => None,
            },
            GrammarOpt2::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            GrammarOpt2::Alt0 { .. } => 1usize,
            GrammarOpt2::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::GrammarOpt2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            GrammarOpt2::Alt0 { span, .. } => *span,
            GrammarOpt2::Alt1 { span, .. } => *span,
        }
    }
}
impl LayoutDefPlus1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            LayoutDefPlus1::Alt0 {
                identifiers,
                layout,
                identifier_2,
                ..
            } => match index {
                0 => Some(identifiers.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(identifier_2.as_parse_tree_ref()),
                _ => None,
            },
            LayoutDefPlus1::Alt1 { identifier, .. } => match index {
                0 => Some(identifier.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            LayoutDefPlus1::Alt0 { .. } => 3usize,
            LayoutDefPlus1::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::LayoutDefPlus1(self)
    }
    pub fn span(&self) -> Span {
        match self {
            LayoutDefPlus1::Alt0 { span, .. } => *span,
            LayoutDefPlus1::Alt1 { span, .. } => *span,
        }
    }
}
impl LayoutDefOpt3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            LayoutDefOpt3::Alt0 { identifiers, .. } => match index {
                0 => Some(identifiers.as_parse_tree_ref()),
                _ => None,
            },
            LayoutDefOpt3::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            LayoutDefOpt3::Alt0 { .. } => 1usize,
            LayoutDefOpt3::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::LayoutDefOpt3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            LayoutDefOpt3::Alt0 { span, .. } => *span,
            LayoutDefOpt3::Alt1 { span, .. } => *span,
        }
    }
}
impl LayoutDefStar1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.layout_def_opt_3.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::LayoutDefStar1(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl SyntaxRuleOpt4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SyntaxRuleOpt4::Alt0 { annotation, .. } => match index {
                0 => Some(annotation.as_parse_tree_ref()),
                _ => None,
            },
            SyntaxRuleOpt4::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SyntaxRuleOpt4::Alt0 { .. } => 1usize,
            SyntaxRuleOpt4::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRuleOpt4(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SyntaxRuleOpt4::Alt0 { span, .. } => *span,
            SyntaxRuleOpt4::Alt1 { span, .. } => *span,
        }
    }
}
impl SyntaxRulePlus2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SyntaxRulePlus2::Alt0 {
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
            SyntaxRulePlus2::Alt1 { priority_level, .. } => match index {
                0 => Some(priority_level.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SyntaxRulePlus2::Alt0 { .. } => 5usize,
            SyntaxRulePlus2::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRulePlus2(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SyntaxRulePlus2::Alt0 { span, .. } => *span,
            SyntaxRulePlus2::Alt1 { span, .. } => *span,
        }
    }
    pub fn priority_levels(&self) -> impl Iterator<Item = &PriorityLevel> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::PriorityLevel(r) => Some(r),
            _ => None,
        })
    }
}
impl SyntaxRuleOpt5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SyntaxRuleOpt5::Alt0 {
                priority_levels, ..
            } => match index {
                0 => Some(priority_levels.as_parse_tree_ref()),
                _ => None,
            },
            SyntaxRuleOpt5::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SyntaxRuleOpt5::Alt0 { .. } => 1usize,
            SyntaxRuleOpt5::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRuleOpt5(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SyntaxRuleOpt5::Alt0 { span, .. } => *span,
            SyntaxRuleOpt5::Alt1 { span, .. } => *span,
        }
    }
    pub fn priority_levels(&self) -> impl Iterator<Item = &PriorityLevel> {
        self.value()
            .into_iter()
            .flat_map(|inner| inner.priority_levels())
    }
}
impl SyntaxRuleStar2 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.syntax_rule_opt_5.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SyntaxRuleStar2(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn priority_levels(&self) -> impl Iterator<Item = &PriorityLevel> {
        self.syntax_rule_opt_5.priority_levels()
    }
}
impl RegexBlockPlus3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexBlockPlus3::Alt0 {
                regex_rules,
                layout,
                regex_rule_2,
                ..
            } => match index {
                0 => Some(regex_rules.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(regex_rule_2.as_parse_tree_ref()),
                _ => None,
            },
            RegexBlockPlus3::Alt1 { regex_rule, .. } => match index {
                0 => Some(regex_rule.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexBlockPlus3::Alt0 { .. } => 3usize,
            RegexBlockPlus3::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexBlockPlus3(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexBlockPlus3::Alt0 { span, .. } => *span,
            RegexBlockPlus3::Alt1 { span, .. } => *span,
        }
    }
    pub fn regex_rules(&self) -> impl Iterator<Item = &RegexRule> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::RegexRule(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexBlockOpt6 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexBlockOpt6::Alt0 { regex_rules, .. } => match index {
                0 => Some(regex_rules.as_parse_tree_ref()),
                _ => None,
            },
            RegexBlockOpt6::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexBlockOpt6::Alt0 { .. } => 1usize,
            RegexBlockOpt6::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexBlockOpt6(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexBlockOpt6::Alt0 { span, .. } => *span,
            RegexBlockOpt6::Alt1 { span, .. } => *span,
        }
    }
    pub fn regex_rules(&self) -> impl Iterator<Item = &RegexRule> {
        self.value()
            .into_iter()
            .flat_map(|inner| inner.regex_rules())
    }
}
impl RegexBlockStar3 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.regex_block_opt_6.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexBlockStar3(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn regex_rules(&self) -> impl Iterator<Item = &RegexRule> {
        self.regex_block_opt_6.regex_rules()
    }
}
impl RegexRuleOpt7 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexRuleOpt7::Alt0 { pre_condition, .. } => match index {
                0 => Some(pre_condition.as_parse_tree_ref()),
                _ => None,
            },
            RegexRuleOpt7::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexRuleOpt7::Alt0 { .. } => 1usize,
            RegexRuleOpt7::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRuleOpt7(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexRuleOpt7::Alt0 { span, .. } => *span,
            RegexRuleOpt7::Alt1 { span, .. } => *span,
        }
    }
}
impl RegexRulePlus5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexRulePlus5::Alt0 {
                regexes,
                layout,
                regex_2,
                ..
            } => match index {
                0 => Some(regexes.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(regex_2.as_parse_tree_ref()),
                _ => None,
            },
            RegexRulePlus5::Alt1 { regex, .. } => match index {
                0 => Some(regex.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexRulePlus5::Alt0 { .. } => 3usize,
            RegexRulePlus5::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRulePlus5(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexRulePlus5::Alt0 { span, .. } => *span,
            RegexRulePlus5::Alt1 { span, .. } => *span,
        }
    }
    pub fn regexes(&self) -> impl Iterator<Item = &Regex> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Regex(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexRulePlus4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexRulePlus4::Alt0 {
                regex_rule_plus_4,
                layout_1,
                lit_2,
                layout_3,
                regexes,
                ..
            } => match index {
                0 => Some(regex_rule_plus_4.as_parse_tree_ref()),
                1 => Some(layout_1.as_parse_tree_ref()),
                2 => Some(lit_2.as_parse_tree_ref()),
                3 => Some(layout_3.as_parse_tree_ref()),
                4 => Some(regexes.as_parse_tree_ref()),
                _ => None,
            },
            RegexRulePlus4::Alt1 { regexes, .. } => match index {
                0 => Some(regexes.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexRulePlus4::Alt0 { .. } => 5usize,
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
    pub fn regexes(&self) -> impl Iterator<Item = impl Iterator<Item = &Regex> + '_> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::RegexRulePlus5(r) => Some(r.regexes()),
            _ => None,
        })
    }
}
impl RegexRulePlus6 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexRulePlus6::Alt0 {
                post_conditions,
                layout,
                post_condition_2,
                ..
            } => match index {
                0 => Some(post_conditions.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(post_condition_2.as_parse_tree_ref()),
                _ => None,
            },
            RegexRulePlus6::Alt1 { post_condition, .. } => match index {
                0 => Some(post_condition.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexRulePlus6::Alt0 { .. } => 3usize,
            RegexRulePlus6::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRulePlus6(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexRulePlus6::Alt0 { span, .. } => *span,
            RegexRulePlus6::Alt1 { span, .. } => *span,
        }
    }
    pub fn post_conditions(&self) -> impl Iterator<Item = &PostCondition> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::PostCondition(r) => Some(r),
            _ => None,
        })
    }
}
impl RegexRuleOpt8 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexRuleOpt8::Alt0 {
                post_conditions, ..
            } => match index {
                0 => Some(post_conditions.as_parse_tree_ref()),
                _ => None,
            },
            RegexRuleOpt8::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexRuleOpt8::Alt0 { .. } => 1usize,
            RegexRuleOpt8::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRuleOpt8(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexRuleOpt8::Alt0 { span, .. } => *span,
            RegexRuleOpt8::Alt1 { span, .. } => *span,
        }
    }
    pub fn post_conditions(&self) -> impl Iterator<Item = &PostCondition> {
        self.value()
            .into_iter()
            .flat_map(|inner| inner.post_conditions())
    }
}
impl RegexRuleStar4 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.regex_rule_opt_8.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexRuleStar4(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn post_conditions(&self) -> impl Iterator<Item = &PostCondition> {
        self.regex_rule_opt_8.post_conditions()
    }
}
impl PriorityLevelOpt9 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelOpt9::Alt0 { associativity, .. } => match index {
                0 => Some(associativity.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelOpt9::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelOpt9::Alt0 { .. } => 1usize,
            PriorityLevelOpt9::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelOpt9(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelOpt9::Alt0 { span, .. } => *span,
            PriorityLevelOpt9::Alt1 { span, .. } => *span,
        }
    }
}
impl PriorityLevelPlus7 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelPlus7::Alt0 {
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
            PriorityLevelPlus7::Alt1 { alternative, .. } => match index {
                0 => Some(alternative.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelPlus7::Alt0 { .. } => 5usize,
            PriorityLevelPlus7::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelPlus7(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelPlus7::Alt0 { span, .. } => *span,
            PriorityLevelPlus7::Alt1 { span, .. } => *span,
        }
    }
    pub fn alternatives(&self) -> impl Iterator<Item = &Alternative> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Alternative(r) => Some(r),
            _ => None,
        })
    }
}
impl PriorityLevelOpt10 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            PriorityLevelOpt10::Alt0 { alternatives, .. } => match index {
                0 => Some(alternatives.as_parse_tree_ref()),
                _ => None,
            },
            PriorityLevelOpt10::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            PriorityLevelOpt10::Alt0 { .. } => 1usize,
            PriorityLevelOpt10::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelOpt10(self)
    }
    pub fn span(&self) -> Span {
        match self {
            PriorityLevelOpt10::Alt0 { span, .. } => *span,
            PriorityLevelOpt10::Alt1 { span, .. } => *span,
        }
    }
    pub fn alternatives(&self) -> impl Iterator<Item = &Alternative> {
        self.value()
            .into_iter()
            .flat_map(|inner| inner.alternatives())
    }
}
impl PriorityLevelStar5 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.priority_level_opt_10.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::PriorityLevelStar5(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn alternatives(&self) -> impl Iterator<Item = &Alternative> {
        self.priority_level_opt_10.alternatives()
    }
}
impl AlternativePlus8 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativePlus8::Alt0 {
                symbols,
                layout,
                symbol_2,
                ..
            } => match index {
                0 => Some(symbols.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(symbol_2.as_parse_tree_ref()),
                _ => None,
            },
            AlternativePlus8::Alt1 { symbol, .. } => match index {
                0 => Some(symbol.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativePlus8::Alt0 { .. } => 3usize,
            AlternativePlus8::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativePlus8(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativePlus8::Alt0 { span, .. } => *span,
            AlternativePlus8::Alt1 { span, .. } => *span,
        }
    }
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::Symbol(r) => Some(r),
            _ => None,
        })
    }
}
impl AlternativeOpt11 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativeOpt11::Alt0 { symbols, .. } => match index {
                0 => Some(symbols.as_parse_tree_ref()),
                _ => None,
            },
            AlternativeOpt11::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativeOpt11::Alt0 { .. } => 1usize,
            AlternativeOpt11::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativeOpt11(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativeOpt11::Alt0 { span, .. } => *span,
            AlternativeOpt11::Alt1 { span, .. } => *span,
        }
    }
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.value().into_iter().flat_map(|inner| inner.symbols())
    }
}
impl AlternativeStar6 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.alternative_opt_11.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        1usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativeStar6(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.alternative_opt_11.symbols()
    }
}
impl AlternativeOpt12 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            AlternativeOpt12::Alt0 { label, .. } => match index {
                0 => Some(label.as_parse_tree_ref()),
                _ => None,
            },
            AlternativeOpt12::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            AlternativeOpt12::Alt0 { .. } => 1usize,
            AlternativeOpt12::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::AlternativeOpt12(self)
    }
    pub fn span(&self) -> Span {
        match self {
            AlternativeOpt12::Alt0 { span, .. } => *span,
            AlternativeOpt12::Alt1 { span, .. } => *span,
        }
    }
}
impl SymbolGroup0 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.lit_0.as_parse_tree_ref()),
            1 => Some(self.layout.as_parse_tree_ref()),
            2 => Some(self.symbol.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SymbolGroup0(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn symbol(&self) -> Option<&Symbol> {
        self.iter().find_map(|node| match node {
            ParseTreeRef::Symbol(inner) => Some(inner),
            _ => None,
        })
    }
}
impl SymbolPlus9 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            SymbolPlus9::Alt0 {
                symbol_plus_9,
                layout,
                symbol_group_0,
                ..
            } => match index {
                0 => Some(symbol_plus_9.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(symbol_group_0.as_parse_tree_ref()),
                _ => None,
            },
            SymbolPlus9::Alt1 { symbol_group_0, .. } => match index {
                0 => Some(symbol_group_0.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            SymbolPlus9::Alt0 { .. } => 3usize,
            SymbolPlus9::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::SymbolPlus9(self)
    }
    pub fn span(&self) -> Span {
        match self {
            SymbolPlus9::Alt0 { span, .. } => *span,
            SymbolPlus9::Alt1 { span, .. } => *span,
        }
    }
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::SymbolGroup0(r) => Some(r.symbol.as_ref()),
            _ => None,
        })
    }
}
impl RegexGroup1 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match index {
            0 => Some(self.lit_0.as_parse_tree_ref()),
            1 => Some(self.layout.as_parse_tree_ref()),
            2 => Some(self.regex.as_parse_tree_ref()),
            _ => None,
        }
    }
    pub fn child_count(&self) -> usize {
        3usize
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexGroup1(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn regex(&self) -> Option<&Regex> {
        self.iter().find_map(|node| match node {
            ParseTreeRef::Regex(inner) => Some(inner),
            _ => None,
        })
    }
}
impl RegexPlus10 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            RegexPlus10::Alt0 {
                regex_plus_10,
                layout,
                regex_group_1,
                ..
            } => match index {
                0 => Some(regex_plus_10.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(regex_group_1.as_parse_tree_ref()),
                _ => None,
            },
            RegexPlus10::Alt1 { regex_group_1, .. } => match index {
                0 => Some(regex_group_1.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            RegexPlus10::Alt0 { .. } => 3usize,
            RegexPlus10::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::RegexPlus10(self)
    }
    pub fn span(&self) -> Span {
        match self {
            RegexPlus10::Alt0 { span, .. } => *span,
            RegexPlus10::Alt1 { span, .. } => *span,
        }
    }
    pub fn regexes(&self) -> impl Iterator<Item = &Regex> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::RegexGroup1(r) => Some(r.regex.as_ref()),
            _ => None,
        })
    }
}
impl CharClassOpt13 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassOpt13::Alt0 { lit_0, .. } => match index {
                0 => Some(lit_0.as_parse_tree_ref()),
                _ => None,
            },
            CharClassOpt13::Alt1 { .. } => match index {
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassOpt13::Alt0 { .. } => 1usize,
            CharClassOpt13::Alt1 { .. } => 0usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassOpt13(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassOpt13::Alt0 { span, .. } => *span,
            CharClassOpt13::Alt1 { span, .. } => *span,
        }
    }
}
impl CharClassPlus11 {
    pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
        match self {
            CharClassPlus11::Alt0 {
                range_elements,
                layout,
                range_element_2,
                ..
            } => match index {
                0 => Some(range_elements.as_parse_tree_ref()),
                1 => Some(layout.as_parse_tree_ref()),
                2 => Some(range_element_2.as_parse_tree_ref()),
                _ => None,
            },
            CharClassPlus11::Alt1 { range_element, .. } => match index {
                0 => Some(range_element.as_parse_tree_ref()),
                _ => None,
            },
        }
    }
    pub fn child_count(&self) -> usize {
        match self {
            CharClassPlus11::Alt0 { .. } => 3usize,
            CharClassPlus11::Alt1 { .. } => 1usize,
        }
    }
    pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
        ParseTreeRef::CharClassPlus11(self)
    }
    pub fn span(&self) -> Span {
        match self {
            CharClassPlus11::Alt0 { span, .. } => *span,
            CharClassPlus11::Alt1 { span, .. } => *span,
        }
    }
    pub fn range_elements(&self) -> impl Iterator<Item = &RangeElement> {
        self.iter().filter_map(|node| match node {
            ParseTreeRef::RangeElement(r) => Some(r),
            _ => None,
        })
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
impl StartLayoutDef {
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
        ParseTreeRef::StartLayoutDef(self)
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
impl StartAnnotation {
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
        ParseTreeRef::StartAnnotation(self)
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
impl StartPreCondition {
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
        ParseTreeRef::StartPreCondition(self)
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
impl StartPostCondition {
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
        ParseTreeRef::StartPostCondition(self)
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
impl StartAssociativity {
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
        ParseTreeRef::StartAssociativity(self)
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
impl StartRangeElement {
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
        ParseTreeRef::StartRangeElement(self)
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
                    layout: layout,
                    syntax_rule_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                GrammarPlus0::Alt1 {
                    syntax_rule: item, ..
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
impl<'a> ListNode<'a> for LayoutDefPlus1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                LayoutDefPlus1::Alt0 {
                    identifiers: rest,
                    layout: layout,
                    identifier_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                LayoutDefPlus1::Alt1 {
                    identifier: item, ..
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
impl<'a> ListNode<'a> for SyntaxRulePlus2 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                SyntaxRulePlus2::Alt0 {
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
                SyntaxRulePlus2::Alt1 {
                    priority_level: item,
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
impl<'a> ListNode<'a> for RegexBlockPlus3 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RegexBlockPlus3::Alt0 {
                    regex_rules: rest,
                    layout: layout,
                    regex_rule_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                RegexBlockPlus3::Alt1 {
                    regex_rule: item, ..
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
impl<'a> ListNode<'a> for RegexRulePlus5 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RegexRulePlus5::Alt0 {
                    regexes: rest,
                    layout: layout,
                    regex_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                RegexRulePlus5::Alt1 { regex: item, .. } => {
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
                    regex_rule_plus_4: rest,
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
                RegexRulePlus4::Alt1 { regexes: item, .. } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for RegexRulePlus6 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RegexRulePlus6::Alt0 {
                    post_conditions: rest,
                    layout: layout,
                    post_condition_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                RegexRulePlus6::Alt1 {
                    post_condition: item,
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
impl<'a> ListNode<'a> for PriorityLevelPlus7 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                PriorityLevelPlus7::Alt0 {
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
                PriorityLevelPlus7::Alt1 {
                    alternative: item, ..
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
impl<'a> ListNode<'a> for AlternativePlus8 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                AlternativePlus8::Alt0 {
                    symbols: rest,
                    layout: layout,
                    symbol_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                AlternativePlus8::Alt1 { symbol: item, .. } => {
                    items.push(item.as_parse_tree_ref());
                    break;
                }
            }
        }
        items.reverse();
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for SymbolPlus9 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                SymbolPlus9::Alt0 {
                    symbol_plus_9: rest,
                    layout: layout,
                    symbol_group_0: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                SymbolPlus9::Alt1 {
                    symbol_group_0: item,
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
impl<'a> ListNode<'a> for RegexPlus10 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                RegexPlus10::Alt0 {
                    regex_plus_10: rest,
                    layout: layout,
                    regex_group_1: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                RegexPlus10::Alt1 {
                    regex_group_1: item,
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
impl<'a> ListNode<'a> for CharClassPlus11 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        let mut current = self;
        loop {
            match current {
                CharClassPlus11::Alt0 {
                    range_elements: rest,
                    layout: layout,
                    range_element_2: item,
                    ..
                } => {
                    items.push(item.as_parse_tree_ref());
                    items.push(layout.as_parse_tree_ref());
                    current = rest;
                }
                CharClassPlus11::Alt1 {
                    range_element: item,
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
        match &self.grammar_opt_1 {
            GrammarOpt1::Alt0 {
                syntax_rules: grammar_opt_1,
                ..
            } => grammar_opt_1.iter(),
            GrammarOpt1::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for LayoutDefStar1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.layout_def_opt_3 {
            LayoutDefOpt3::Alt0 {
                identifiers: layout_def_opt_3,
                ..
            } => layout_def_opt_3.iter(),
            LayoutDefOpt3::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for SyntaxRuleStar2 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.syntax_rule_opt_5 {
            SyntaxRuleOpt5::Alt0 {
                priority_levels: syntax_rule_opt_5,
                ..
            } => syntax_rule_opt_5.iter(),
            SyntaxRuleOpt5::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for RegexBlockStar3 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.regex_block_opt_6 {
            RegexBlockOpt6::Alt0 {
                regex_rules: regex_block_opt_6,
                ..
            } => regex_block_opt_6.iter(),
            RegexBlockOpt6::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for RegexRuleStar4 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.regex_rule_opt_8 {
            RegexRuleOpt8::Alt0 {
                post_conditions: regex_rule_opt_8,
                ..
            } => regex_rule_opt_8.iter(),
            RegexRuleOpt8::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for PriorityLevelStar5 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.priority_level_opt_10 {
            PriorityLevelOpt10::Alt0 {
                alternatives: priority_level_opt_10,
                ..
            } => priority_level_opt_10.iter(),
            PriorityLevelOpt10::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for AlternativeStar6 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        match &self.alternative_opt_11 {
            AlternativeOpt11::Alt0 {
                symbols: alternative_opt_11,
                ..
            } => alternative_opt_11.iter(),
            AlternativeOpt11::Alt1 { .. } => vec![].into_iter(),
        }
    }
}
impl<'a> ListNode<'a> for SymbolGroup0 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        items.push(self.lit_0.as_parse_tree_ref());
        items.push(self.layout.as_parse_tree_ref());
        items.push(self.symbol.as_parse_tree_ref());
        items.into_iter()
    }
}
impl<'a> ListNode<'a> for RegexGroup1 {
    fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
        let mut items = vec![];
        items.push(self.lit_0.as_parse_tree_ref());
        items.push(self.layout.as_parse_tree_ref());
        items.push(self.regex.as_parse_tree_ref());
        items.into_iter()
    }
}
impl OptNode for GrammarOpt0 {
    type Inner = LayoutDef;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            GrammarOpt0::Alt0 { layout_def, .. } => Some(layout_def),
            GrammarOpt0::Alt1 { .. } => None,
        }
    }
}
impl OptNode for GrammarOpt1 {
    type Inner = GrammarPlus0;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            GrammarOpt1::Alt0 { syntax_rules, .. } => Some(syntax_rules),
            GrammarOpt1::Alt1 { .. } => None,
        }
    }
}
impl OptNode for GrammarOpt2 {
    type Inner = RegexBlock;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            GrammarOpt2::Alt0 { regex_block, .. } => Some(regex_block),
            GrammarOpt2::Alt1 { .. } => None,
        }
    }
}
impl OptNode for LayoutDefOpt3 {
    type Inner = LayoutDefPlus1;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            LayoutDefOpt3::Alt0 { identifiers, .. } => Some(identifiers),
            LayoutDefOpt3::Alt1 { .. } => None,
        }
    }
}
impl OptNode for SyntaxRuleOpt4 {
    type Inner = Annotation;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            SyntaxRuleOpt4::Alt0 { annotation, .. } => Some(annotation),
            SyntaxRuleOpt4::Alt1 { .. } => None,
        }
    }
}
impl OptNode for SyntaxRuleOpt5 {
    type Inner = SyntaxRulePlus2;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            SyntaxRuleOpt5::Alt0 {
                priority_levels, ..
            } => Some(priority_levels),
            SyntaxRuleOpt5::Alt1 { .. } => None,
        }
    }
}
impl OptNode for RegexBlockOpt6 {
    type Inner = RegexBlockPlus3;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            RegexBlockOpt6::Alt0 { regex_rules, .. } => Some(regex_rules),
            RegexBlockOpt6::Alt1 { .. } => None,
        }
    }
}
impl OptNode for RegexRuleOpt7 {
    type Inner = PreCondition;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            RegexRuleOpt7::Alt0 { pre_condition, .. } => Some(pre_condition),
            RegexRuleOpt7::Alt1 { .. } => None,
        }
    }
}
impl OptNode for RegexRuleOpt8 {
    type Inner = RegexRulePlus6;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            RegexRuleOpt8::Alt0 {
                post_conditions, ..
            } => Some(post_conditions),
            RegexRuleOpt8::Alt1 { .. } => None,
        }
    }
}
impl OptNode for PriorityLevelOpt9 {
    type Inner = Associativity;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            PriorityLevelOpt9::Alt0 { associativity, .. } => Some(associativity),
            PriorityLevelOpt9::Alt1 { .. } => None,
        }
    }
}
impl OptNode for PriorityLevelOpt10 {
    type Inner = PriorityLevelPlus7;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            PriorityLevelOpt10::Alt0 { alternatives, .. } => Some(alternatives),
            PriorityLevelOpt10::Alt1 { .. } => None,
        }
    }
}
impl OptNode for AlternativeOpt11 {
    type Inner = AlternativePlus8;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            AlternativeOpt11::Alt0 { symbols, .. } => Some(symbols),
            AlternativeOpt11::Alt1 { .. } => None,
        }
    }
}
impl OptNode for AlternativeOpt12 {
    type Inner = Token;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            AlternativeOpt12::Alt0 { label, .. } => Some(label),
            AlternativeOpt12::Alt1 { .. } => None,
        }
    }
}
impl OptNode for CharClassOpt13 {
    type Inner = Token;
    fn value(&self) -> Option<&Self::Inner> {
        match self {
            CharClassOpt13::Alt0 { lit_0, .. } => Some(lit_0),
            CharClassOpt13::Alt1 { .. } => None,
        }
    }
}
impl RangeElement {
    pub fn as_range(&self) -> Option<&Range> {
        match self {
            RangeElement::Alt0 { range, .. } => Some(range),
            _ => None,
        }
    }
    pub fn as_range_char(&self) -> Option<&Token> {
        match self {
            RangeElement::Alt1 { range_char, .. } => Some(range_char),
            _ => None,
        }
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
        //Keyword
        TerminalId(0) => TokenKind::T0,
        //Identifier
        TerminalId(1) => TokenKind::T1,
        //String
        TerminalId(2) => TokenKind::T2,
        //RangeChar
        TerminalId(3) => TokenKind::T3,
        //Char
        TerminalId(4) => TokenKind::T4,
        //Label
        TerminalId(5) => TokenKind::T5,
        //WS
        TerminalId(6) => TokenKind::T6,
        //"grammar"
        TerminalId(7) => TokenKind::T7,
        //"layout"
        TerminalId(8) => TokenKind::T8,
        //"="
        TerminalId(9) => TokenKind::T9,
        //">"
        TerminalId(10) => TokenKind::T10,
        //"@NoLayout"
        TerminalId(11) => TokenKind::T11,
        //"@Layout"
        TerminalId(12) => TokenKind::T12,
        //"("
        TerminalId(13) => TokenKind::T13,
        //")"
        TerminalId(14) => TokenKind::T14,
        //"regex"
        TerminalId(15) => TokenKind::T15,
        //"{"
        TerminalId(16) => TokenKind::T16,
        //"}"
        TerminalId(17) => TokenKind::T17,
        //"|"
        TerminalId(18) => TokenKind::T18,
        //"!<<"
        TerminalId(19) => TokenKind::T19,
        //"\"
        TerminalId(20) => TokenKind::T20,
        //"!>>"
        TerminalId(21) => TokenKind::T21,
        //"left"
        TerminalId(22) => TokenKind::T22,
        //"right"
        TerminalId(23) => TokenKind::T23,
        //"none"
        TerminalId(24) => TokenKind::T24,
        //"""
        TerminalId(25) => TokenKind::T25,
        //"*"
        TerminalId(26) => TokenKind::T26,
        //"+"
        TerminalId(27) => TokenKind::T27,
        //"?"
        TerminalId(28) => TokenKind::T28,
        //":"
        TerminalId(29) => TokenKind::T29,
        //"'"
        TerminalId(30) => TokenKind::T30,
        //"!"
        TerminalId(31) => TokenKind::T31,
        //"["
        TerminalId(32) => TokenKind::T32,
        //"]"
        TerminalId(33) => TokenKind::T33,
        //"-"
        TerminalId(34) => TokenKind::T34,
        //Layout
        TerminalId(35) => TokenKind::T35,
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
                    //Grammar : "grammar" Layout name:Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?.
                    SlotId(9) => {
                        let [
                            lit_0,
                            layout_1,
                            name,
                            layout_3,
                            layout_def,
                            layout_5,
                            syntax_rules,
                            layout_7,
                            regex_block,
                        ] = <[ParseTree; 9usize]>::try_from(children).unwrap();
                        Grammar {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            name: name.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            layout_def: layout_def.unwrap_grammar_opt_0(),
                            layout_5: layout_5.unwrap_token(),
                            syntax_rules: syntax_rules.unwrap_grammar_star_0(),
                            layout_7: layout_7.unwrap_token(),
                            regex_block: regex_block.unwrap_grammar_opt_2(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //LayoutDef
            NonterminalId(1) => {
                match nonterminal_node.return_slot {
                    //LayoutDef : "layout" Layout Identifier*.
                    SlotId(13) => {
                        let [lit_0, layout, identifiers] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        LayoutDef {
                            lit_0: lit_0.unwrap_token(),
                            layout: layout.unwrap_token(),
                            identifiers: identifiers.unwrap_layout_def_star_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule
            NonterminalId(2) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule : Annotation? Layout head:Identifier Layout "=" Layout {PriorityLevel ">"}*.
                    SlotId(21) => {
                        let [
                            annotation,
                            layout_1,
                            head,
                            layout_3,
                            lit_4,
                            layout_5,
                            priority_levels,
                        ] = <[ParseTree; 7usize]>::try_from(children).unwrap();
                        SyntaxRule {
                            annotation: annotation.unwrap_syntax_rule_opt_4(),
                            layout_1: layout_1.unwrap_token(),
                            head: head.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            layout_5: layout_5.unwrap_token(),
                            priority_levels: priority_levels.unwrap_syntax_rule_star_2(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Annotation
            NonterminalId(3) => {
                match nonterminal_node.return_slot {
                    //Annotation : "@NoLayout".
                    SlotId(23) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Annotation::NoLayout {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Annotation : "@Layout" Layout "(" Layout Identifier Layout ")".
                    SlotId(31) => {
                        let [
                            lit_0,
                            layout_1,
                            lit_2,
                            layout_3,
                            identifier,
                            layout_5,
                            lit_6,
                        ] = <[ParseTree; 7usize]>::try_from(children).unwrap();
                        Annotation::Layout {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            identifier: identifier.unwrap_token(),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock
            NonterminalId(4) => {
                match nonterminal_node.return_slot {
                    //RegexBlock : "regex" Layout "{" Layout RegexRule* Layout "}".
                    SlotId(39) => {
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
                            regex_rules: regex_rules.unwrap_regex_block_star_3(),
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
            NonterminalId(5) => {
                match nonterminal_node.return_slot {
                    //RegexRule : Identifier Layout "=" Layout PreCondition? Layout body:{Regex+ "|"}+ Layout PostCondition*.
                    SlotId(49) => {
                        let [
                            identifier,
                            layout_1,
                            lit_2,
                            layout_3,
                            pre_condition,
                            layout_5,
                            body,
                            layout_7,
                            post_conditions,
                        ] = <[ParseTree; 9usize]>::try_from(children).unwrap();
                        RegexRule {
                            identifier: identifier.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            pre_condition: pre_condition.unwrap_regex_rule_opt_7(),
                            layout_5: layout_5.unwrap_token(),
                            body: body.unwrap_regex_rule_plus_4(),
                            layout_7: layout_7.unwrap_token(),
                            post_conditions: post_conditions.unwrap_regex_rule_star_4(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PreCondition
            NonterminalId(6) => {
                match nonterminal_node.return_slot {
                    //PreCondition : Identifier Layout "!<<".
                    SlotId(53) => {
                        let [identifier, layout, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        PreCondition {
                            identifier: identifier.unwrap_token(),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PostCondition
            NonterminalId(7) => {
                match nonterminal_node.return_slot {
                    //PostCondition : "\" Layout Identifier.
                    SlotId(57) => {
                        let [lit_0, layout, identifier] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        PostCondition::Except {
                            lit_0: lit_0.unwrap_token(),
                            layout: layout.unwrap_token(),
                            identifier: identifier.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //PostCondition : "!>>" Layout Identifier.
                    SlotId(61) => {
                        let [lit_0, layout, identifier] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        PostCondition::FollowRestriction {
                            lit_0: lit_0.unwrap_token(),
                            layout: layout.unwrap_token(),
                            identifier: identifier.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel
            NonterminalId(8) => {
                match nonterminal_node.return_slot {
                    //PriorityLevel : Associativity? Layout {Alternative "|"}*.
                    SlotId(65) => {
                        let [associativity, layout, alternatives] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        PriorityLevel {
                            associativity: associativity.unwrap_priority_level_opt_9(),
                            layout: layout.unwrap_token(),
                            alternatives: alternatives.unwrap_priority_level_star_5(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Associativity
            NonterminalId(9) => {
                match nonterminal_node.return_slot {
                    //Associativity : "left".
                    SlotId(67) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Associativity::Alt0 {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Associativity : "right".
                    SlotId(69) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Associativity::Alt1 {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Associativity : "none".
                    SlotId(71) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Associativity::Alt2 {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative
            NonterminalId(10) => {
                match nonterminal_node.return_slot {
                    //Alternative : Symbol* Layout Label?.
                    SlotId(75) => {
                        let [symbols, layout, label] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Alternative {
                            symbols: symbols.unwrap_alternative_star_6(),
                            layout: layout.unwrap_token(),
                            label: label.unwrap_alternative_opt_12(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex
            NonterminalId(11) => {
                match nonterminal_node.return_slot {
                    //Regex : Regex Layout "+".
                    SlotId(180) => {
                        let [regex, layout, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Regex::Plus {
                            regex: Box::new(regex.unwrap_regex()),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : Regex Layout "*".
                    SlotId(184) => {
                        let [regex, layout, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Regex::Star {
                            regex: Box::new(regex.unwrap_regex()),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : Regex Layout "?".
                    SlotId(188) => {
                        let [regex, layout, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Regex::Opt {
                            regex: Box::new(regex.unwrap_regex()),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : "(" Layout first:Regex Layout rest:("|" Regex)+ Layout ")".
                    SlotId(196) => {
                        let [lit_0, layout_1, first, layout_3, rest, layout_5, lit_6] =
                            <[ParseTree; 7usize]>::try_from(children).unwrap();
                        Regex::Alt {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            first: Box::new(first.unwrap_regex()),
                            layout_3: layout_3.unwrap_token(),
                            rest: rest.unwrap_regex_plus_10(),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : "(" Layout Regex+ Layout ")".
                    SlotId(202) => {
                        let [lit_0, layout_1, regexes, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Regex::Group {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            regexes: regexes.unwrap_regex_rule_plus_5(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : CharClass.
                    SlotId(204) => {
                        let [char_class] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Regex::CharClass {
                            char_class: char_class.unwrap_char_class(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : "'" Layout Char Layout "'".
                    SlotId(210) => {
                        let [lit_0, layout_1, char, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Regex::Char {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            char: char.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex : """ Layout String Layout """.
                    SlotId(216) => {
                        let [lit_0, layout_1, string, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Regex::String {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            string: string.unwrap_token(),
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
            NonterminalId(12) => {
                match nonterminal_node.return_slot {
                    //CharClass : neg:"!"? Layout "[" Layout RangeElement+ Layout "]".
                    SlotId(224) => {
                        let [
                            neg,
                            layout_1,
                            lit_2,
                            layout_3,
                            range_elements,
                            layout_5,
                            lit_6,
                        ] = <[ParseTree; 7usize]>::try_from(children).unwrap();
                        CharClass {
                            neg: neg.unwrap_char_class_opt_13(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            range_elements: range_elements.unwrap_char_class_plus_11(),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RangeElement
            NonterminalId(13) => {
                match nonterminal_node.return_slot {
                    //RangeElement : Range.
                    SlotId(226) => {
                        let [range] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RangeElement::Alt0 {
                            range: range.unwrap_range(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RangeElement : RangeChar.
                    SlotId(228) => {
                        let [range_char] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RangeElement::Alt1 {
                            range_char: range_char.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Range
            NonterminalId(14) => {
                match nonterminal_node.return_slot {
                    //Range : start:RangeChar Layout "-" Layout end:RangeChar.
                    SlotId(234) => {
                        let [start, layout_1, lit_2, layout_3, end] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Range {
                            start: start.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            end: end.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Opt_0
            NonterminalId(15) => {
                match nonterminal_node.return_slot {
                    //LayoutDef? : LayoutDef.
                    SlotId(236) => {
                        let [layout_def] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt0 {
                            layout_def: Box::new(layout_def.unwrap_layout_def()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //LayoutDef? : .
                    SlotId(237) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarOpt0::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Plus_0
            NonterminalId(16) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule+ : SyntaxRule+ Layout SyntaxRule.
                    SlotId(241) => {
                        let [syntax_rules, layout, syntax_rule_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt0 {
                            syntax_rules: Box::new(syntax_rules.unwrap_grammar_plus_0()),
                            layout: layout.unwrap_token(),
                            syntax_rule_2: Box::new(syntax_rule_2.unwrap_syntax_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //SyntaxRule+ : SyntaxRule.
                    SlotId(243) => {
                        let [syntax_rule] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarPlus0::Alt1 {
                            syntax_rule: Box::new(syntax_rule.unwrap_syntax_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Opt_1
            NonterminalId(17) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule+? : SyntaxRule+.
                    SlotId(245) => {
                        let [syntax_rules] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarOpt1::Alt0 {
                            syntax_rules: syntax_rules.unwrap_grammar_plus_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //SyntaxRule+? : .
                    SlotId(246) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarOpt1::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Star_0
            NonterminalId(18) => {
                match nonterminal_node.return_slot {
                    //SyntaxRule* : SyntaxRule+?.
                    SlotId(248) => {
                        let [grammar_opt_1] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarStar0 {
                            grammar_opt_1: grammar_opt_1.unwrap_grammar_opt_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Grammar_Opt_2
            NonterminalId(19) => {
                match nonterminal_node.return_slot {
                    //RegexBlock? : RegexBlock.
                    SlotId(250) => {
                        let [regex_block] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        GrammarOpt2::Alt0 {
                            regex_block: Box::new(regex_block.unwrap_regex_block()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RegexBlock? : .
                    SlotId(251) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        GrammarOpt2::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //LayoutDef_Plus_1
            NonterminalId(20) => {
                match nonterminal_node.return_slot {
                    //Identifier+ : Identifier+ Layout Identifier.
                    SlotId(255) => {
                        let [identifiers, layout, identifier_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        LayoutDefPlus1::Alt0 {
                            identifiers: Box::new(identifiers.unwrap_layout_def_plus_1()),
                            layout: layout.unwrap_token(),
                            identifier_2: identifier_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Identifier+ : Identifier.
                    SlotId(257) => {
                        let [identifier] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        LayoutDefPlus1::Alt1 {
                            identifier: identifier.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //LayoutDef_Opt_3
            NonterminalId(21) => {
                match nonterminal_node.return_slot {
                    //Identifier+? : Identifier+.
                    SlotId(259) => {
                        let [identifiers] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        LayoutDefOpt3::Alt0 {
                            identifiers: identifiers.unwrap_layout_def_plus_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Identifier+? : .
                    SlotId(260) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        LayoutDefOpt3::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //LayoutDef_Star_1
            NonterminalId(22) => {
                match nonterminal_node.return_slot {
                    //Identifier* : Identifier+?.
                    SlotId(262) => {
                        let [layout_def_opt_3] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        LayoutDefStar1 {
                            layout_def_opt_3: layout_def_opt_3.unwrap_layout_def_opt_3(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule_Opt_4
            NonterminalId(23) => {
                match nonterminal_node.return_slot {
                    //Annotation? : Annotation.
                    SlotId(264) => {
                        let [annotation] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SyntaxRuleOpt4::Alt0 {
                            annotation: Box::new(annotation.unwrap_annotation()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Annotation? : .
                    SlotId(265) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        SyntaxRuleOpt4::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule_Plus_2
            NonterminalId(24) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}+ : {PriorityLevel ">"}+ Layout ">" Layout PriorityLevel.
                    SlotId(271) => {
                        let [priority_levels, layout_1, lit_2, layout_3, priority_level_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        SyntaxRulePlus2::Alt0 {
                            priority_levels: Box::new(priority_levels.unwrap_syntax_rule_plus_2()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            priority_level_4: Box::new(priority_level_4.unwrap_priority_level()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{PriorityLevel ">"}+ : PriorityLevel.
                    SlotId(273) => {
                        let [priority_level] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SyntaxRulePlus2::Alt1 {
                            priority_level: Box::new(priority_level.unwrap_priority_level()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule_Opt_5
            NonterminalId(25) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}+? : {PriorityLevel ">"}+.
                    SlotId(275) => {
                        let [priority_levels] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SyntaxRuleOpt5::Alt0 {
                            priority_levels: priority_levels.unwrap_syntax_rule_plus_2(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{PriorityLevel ">"}+? : .
                    SlotId(276) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        SyntaxRuleOpt5::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //SyntaxRule_Star_2
            NonterminalId(26) => {
                match nonterminal_node.return_slot {
                    //{PriorityLevel ">"}* : {PriorityLevel ">"}+?.
                    SlotId(278) => {
                        let [syntax_rule_opt_5] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SyntaxRuleStar2 {
                            syntax_rule_opt_5: syntax_rule_opt_5.unwrap_syntax_rule_opt_5(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock_Plus_3
            NonterminalId(27) => {
                match nonterminal_node.return_slot {
                    //RegexRule+ : RegexRule+ Layout RegexRule.
                    SlotId(282) => {
                        let [regex_rules, layout, regex_rule_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RegexBlockPlus3::Alt0 {
                            regex_rules: Box::new(regex_rules.unwrap_regex_block_plus_3()),
                            layout: layout.unwrap_token(),
                            regex_rule_2: Box::new(regex_rule_2.unwrap_regex_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RegexRule+ : RegexRule.
                    SlotId(284) => {
                        let [regex_rule] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexBlockPlus3::Alt1 {
                            regex_rule: Box::new(regex_rule.unwrap_regex_rule()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock_Opt_6
            NonterminalId(28) => {
                match nonterminal_node.return_slot {
                    //RegexRule+? : RegexRule+.
                    SlotId(286) => {
                        let [regex_rules] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexBlockOpt6::Alt0 {
                            regex_rules: regex_rules.unwrap_regex_block_plus_3(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RegexRule+? : .
                    SlotId(287) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RegexBlockOpt6::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexBlock_Star_3
            NonterminalId(29) => {
                match nonterminal_node.return_slot {
                    //RegexRule* : RegexRule+?.
                    SlotId(289) => {
                        let [regex_block_opt_6] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexBlockStar3 {
                            regex_block_opt_6: regex_block_opt_6.unwrap_regex_block_opt_6(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Opt_7
            NonterminalId(30) => {
                match nonterminal_node.return_slot {
                    //PreCondition? : PreCondition.
                    SlotId(291) => {
                        let [pre_condition] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRuleOpt7::Alt0 {
                            pre_condition: Box::new(pre_condition.unwrap_pre_condition()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //PreCondition? : .
                    SlotId(292) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RegexRuleOpt7::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Plus_5
            NonterminalId(31) => {
                match nonterminal_node.return_slot {
                    //Regex+ : Regex+ Layout Regex.
                    SlotId(296) => {
                        let [regexes, layout, regex_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RegexRulePlus5::Alt0 {
                            regexes: Box::new(regexes.unwrap_regex_rule_plus_5()),
                            layout: layout.unwrap_token(),
                            regex_2: Box::new(regex_2.unwrap_regex()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Regex+ : Regex.
                    SlotId(298) => {
                        let [regex] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRulePlus5::Alt1 {
                            regex: Box::new(regex.unwrap_regex()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Plus_4
            NonterminalId(32) => {
                match nonterminal_node.return_slot {
                    //{Regex+ "|"}+ : {Regex+ "|"}+ Layout "|" Layout Regex+.
                    SlotId(304) => {
                        let [regex_rule_plus_4, layout_1, lit_2, layout_3, regexes] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        RegexRulePlus4::Alt0 {
                            regex_rule_plus_4: Box::new(
                                regex_rule_plus_4.unwrap_regex_rule_plus_4(),
                            ),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            regexes: regexes.unwrap_regex_rule_plus_5(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{Regex+ "|"}+ : Regex+.
                    SlotId(306) => {
                        let [regexes] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRulePlus4::Alt1 {
                            regexes: regexes.unwrap_regex_rule_plus_5(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Plus_6
            NonterminalId(33) => {
                match nonterminal_node.return_slot {
                    //PostCondition+ : PostCondition+ Layout PostCondition.
                    SlotId(310) => {
                        let [post_conditions, layout, post_condition_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RegexRulePlus6::Alt0 {
                            post_conditions: Box::new(post_conditions.unwrap_regex_rule_plus_6()),
                            layout: layout.unwrap_token(),
                            post_condition_2: Box::new(post_condition_2.unwrap_post_condition()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //PostCondition+ : PostCondition.
                    SlotId(312) => {
                        let [post_condition] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRulePlus6::Alt1 {
                            post_condition: Box::new(post_condition.unwrap_post_condition()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Opt_8
            NonterminalId(34) => {
                match nonterminal_node.return_slot {
                    //PostCondition+? : PostCondition+.
                    SlotId(314) => {
                        let [post_conditions] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRuleOpt8::Alt0 {
                            post_conditions: post_conditions.unwrap_regex_rule_plus_6(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //PostCondition+? : .
                    SlotId(315) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        RegexRuleOpt8::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //RegexRule_Star_4
            NonterminalId(35) => {
                match nonterminal_node.return_slot {
                    //PostCondition* : PostCondition+?.
                    SlotId(317) => {
                        let [regex_rule_opt_8] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexRuleStar4 {
                            regex_rule_opt_8: regex_rule_opt_8.unwrap_regex_rule_opt_8(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Opt_9
            NonterminalId(36) => {
                match nonterminal_node.return_slot {
                    //Associativity? : Associativity.
                    SlotId(319) => {
                        let [associativity] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelOpt9::Alt0 {
                            associativity: Box::new(associativity.unwrap_associativity()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Associativity? : .
                    SlotId(320) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        PriorityLevelOpt9::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Plus_7
            NonterminalId(37) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}+ : {Alternative "|"}+ Layout "|" Layout Alternative.
                    SlotId(326) => {
                        let [alternatives, layout_1, lit_2, layout_3, alternative_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        PriorityLevelPlus7::Alt0 {
                            alternatives: Box::new(alternatives.unwrap_priority_level_plus_7()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            alternative_4: Box::new(alternative_4.unwrap_alternative()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{Alternative "|"}+ : Alternative.
                    SlotId(328) => {
                        let [alternative] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelPlus7::Alt1 {
                            alternative: Box::new(alternative.unwrap_alternative()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Opt_10
            NonterminalId(38) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}+? : {Alternative "|"}+.
                    SlotId(330) => {
                        let [alternatives] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelOpt10::Alt0 {
                            alternatives: alternatives.unwrap_priority_level_plus_7(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //{Alternative "|"}+? : .
                    SlotId(331) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        PriorityLevelOpt10::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //PriorityLevel_Star_5
            NonterminalId(39) => {
                match nonterminal_node.return_slot {
                    //{Alternative "|"}* : {Alternative "|"}+?.
                    SlotId(333) => {
                        let [priority_level_opt_10] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        PriorityLevelStar5 {
                            priority_level_opt_10: priority_level_opt_10
                                .unwrap_priority_level_opt_10(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Plus_8
            NonterminalId(40) => {
                match nonterminal_node.return_slot {
                    //Symbol+ : Symbol+ Layout Symbol(0).
                    SlotId(337) => {
                        let [symbols, layout, symbol_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        AlternativePlus8::Alt0 {
                            symbols: Box::new(symbols.unwrap_alternative_plus_8()),
                            layout: layout.unwrap_token(),
                            symbol_2: Box::new(symbol_2.unwrap_symbol()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol+ : Symbol(0).
                    SlotId(339) => {
                        let [symbol] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativePlus8::Alt1 {
                            symbol: Box::new(symbol.unwrap_symbol()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Opt_11
            NonterminalId(41) => {
                match nonterminal_node.return_slot {
                    //Symbol+? : Symbol+.
                    SlotId(341) => {
                        let [symbols] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeOpt11::Alt0 {
                            symbols: symbols.unwrap_alternative_plus_8(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol+? : .
                    SlotId(342) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        AlternativeOpt11::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Star_6
            NonterminalId(42) => {
                match nonterminal_node.return_slot {
                    //Symbol* : Symbol+?.
                    SlotId(344) => {
                        let [alternative_opt_11] =
                            <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeStar6 {
                            alternative_opt_11: alternative_opt_11.unwrap_alternative_opt_11(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Alternative_Opt_12
            NonterminalId(43) => {
                match nonterminal_node.return_slot {
                    //Label? : Label.
                    SlotId(346) => {
                        let [label] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        AlternativeOpt12::Alt0 {
                            label: label.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Label? : .
                    SlotId(347) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        AlternativeOpt12::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Symbol_Group_0
            NonterminalId(44) => {
                match nonterminal_node.return_slot {
                    //("|" Symbol) : "|" Layout Symbol(0).
                    SlotId(351) => {
                        let [lit_0, layout, symbol] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        SymbolGroup0 {
                            lit_0: lit_0.unwrap_token(),
                            layout: layout.unwrap_token(),
                            symbol: Box::new(symbol.unwrap_symbol()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Symbol_Plus_9
            NonterminalId(45) => {
                match nonterminal_node.return_slot {
                    //("|" Symbol)+ : ("|" Symbol)+ Layout ("|" Symbol).
                    SlotId(355) => {
                        let [symbol_plus_9, layout, symbol_group_0] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        SymbolPlus9::Alt0 {
                            symbol_plus_9: Box::new(symbol_plus_9.unwrap_symbol_plus_9()),
                            layout: layout.unwrap_token(),
                            symbol_group_0: symbol_group_0.unwrap_symbol_group_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //("|" Symbol)+ : ("|" Symbol).
                    SlotId(357) => {
                        let [symbol_group_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        SymbolPlus9::Alt1 {
                            symbol_group_0: symbol_group_0.unwrap_symbol_group_0(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex_Group_1
            NonterminalId(46) => {
                match nonterminal_node.return_slot {
                    //("|" Regex) : "|" Layout Regex.
                    SlotId(361) => {
                        let [lit_0, layout, regex] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RegexGroup1 {
                            lit_0: lit_0.unwrap_token(),
                            layout: layout.unwrap_token(),
                            regex: Box::new(regex.unwrap_regex()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //Regex_Plus_10
            NonterminalId(47) => {
                match nonterminal_node.return_slot {
                    //("|" Regex)+ : ("|" Regex)+ Layout ("|" Regex).
                    SlotId(365) => {
                        let [regex_plus_10, layout, regex_group_1] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        RegexPlus10::Alt0 {
                            regex_plus_10: Box::new(regex_plus_10.unwrap_regex_plus_10()),
                            layout: layout.unwrap_token(),
                            regex_group_1: regex_group_1.unwrap_regex_group_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //("|" Regex)+ : ("|" Regex).
                    SlotId(367) => {
                        let [regex_group_1] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        RegexPlus10::Alt1 {
                            regex_group_1: regex_group_1.unwrap_regex_group_1(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Opt_13
            NonterminalId(48) => {
                match nonterminal_node.return_slot {
                    //"!"? : "!".
                    SlotId(369) => {
                        let [lit_0] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassOpt13::Alt0 {
                            lit_0: lit_0.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //"!"? : .
                    SlotId(370) => {
                        let [] = <[ParseTree; 0usize]>::try_from(children).unwrap();
                        CharClassOpt13::Alt1 {
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //CharClass_Plus_11
            NonterminalId(49) => {
                match nonterminal_node.return_slot {
                    //RangeElement+ : RangeElement+ Layout RangeElement.
                    SlotId(374) => {
                        let [range_elements, layout, range_element_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        CharClassPlus11::Alt0 {
                            range_elements: Box::new(range_elements.unwrap_char_class_plus_11()),
                            layout: layout.unwrap_token(),
                            range_element_2: Box::new(range_element_2.unwrap_range_element()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //RangeElement+ : RangeElement.
                    SlotId(376) => {
                        let [range_element] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        CharClassPlus11::Alt1 {
                            range_element: Box::new(range_element.unwrap_range_element()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartGrammar
            NonterminalId(50) => {
                match nonterminal_node.return_slot {
                    //StartGrammar : Layout start:Grammar Layout.
                    SlotId(380) => {
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
            //StartLayoutDef
            NonterminalId(51) => {
                match nonterminal_node.return_slot {
                    //StartLayoutDef : Layout start:LayoutDef Layout.
                    SlotId(384) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartLayoutDef {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_layout_def(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartSyntaxRule
            NonterminalId(52) => {
                match nonterminal_node.return_slot {
                    //StartSyntaxRule : Layout start:SyntaxRule Layout.
                    SlotId(388) => {
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
            //StartAnnotation
            NonterminalId(53) => {
                match nonterminal_node.return_slot {
                    //StartAnnotation : Layout start:Annotation Layout.
                    SlotId(392) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartAnnotation {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_annotation(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartRegexBlock
            NonterminalId(54) => {
                match nonterminal_node.return_slot {
                    //StartRegexBlock : Layout start:RegexBlock Layout.
                    SlotId(396) => {
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
            NonterminalId(55) => {
                match nonterminal_node.return_slot {
                    //StartRegexRule : Layout start:RegexRule Layout.
                    SlotId(400) => {
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
            //StartPreCondition
            NonterminalId(56) => {
                match nonterminal_node.return_slot {
                    //StartPreCondition : Layout start:PreCondition Layout.
                    SlotId(404) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartPreCondition {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_pre_condition(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartPostCondition
            NonterminalId(57) => {
                match nonterminal_node.return_slot {
                    //StartPostCondition : Layout start:PostCondition Layout.
                    SlotId(408) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartPostCondition {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_post_condition(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartPriorityLevel
            NonterminalId(58) => {
                match nonterminal_node.return_slot {
                    //StartPriorityLevel : Layout start:PriorityLevel Layout.
                    SlotId(412) => {
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
            //StartAssociativity
            NonterminalId(59) => {
                match nonterminal_node.return_slot {
                    //StartAssociativity : Layout start:Associativity Layout.
                    SlotId(416) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartAssociativity {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_associativity(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartAlternative
            NonterminalId(60) => {
                match nonterminal_node.return_slot {
                    //StartAlternative : Layout start:Alternative Layout.
                    SlotId(420) => {
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
            NonterminalId(61) => {
                match nonterminal_node.return_slot {
                    //StartSymbol : Layout start:Symbol(0) Layout.
                    SlotId(424) => {
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
            NonterminalId(62) => {
                match nonterminal_node.return_slot {
                    //StartRegex : Layout start:Regex Layout.
                    SlotId(428) => {
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
            NonterminalId(63) => {
                match nonterminal_node.return_slot {
                    //StartCharClass : Layout start:CharClass Layout.
                    SlotId(432) => {
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
            //StartRangeElement
            NonterminalId(64) => {
                match nonterminal_node.return_slot {
                    //StartRangeElement : Layout start:RangeElement Layout.
                    SlotId(436) => {
                        let [layout_0, start, layout_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        StartRangeElement {
                            layout_0: layout_0.unwrap_token(),
                            start: start.unwrap_range_element(),
                            layout_2: layout_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    _ => unreachable!(),
                }
            }
            //StartRange
            NonterminalId(65) => {
                match nonterminal_node.return_slot {
                    //StartRange : Layout start:Range Layout.
                    SlotId(440) => {
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
            //Symbol
            NonterminalId(66) => {
                match nonterminal_node.return_slot {
                    //Symbol : Identifier return 0.
                    SlotId(78) => {
                        let [identifier] = <[ParseTree; 1usize]>::try_from(children).unwrap();
                        Symbol::Identifier {
                            identifier: identifier.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "(" Layout Symbol+ Layout ")" return 0.
                    SlotId(85) => {
                        let [lit_0, layout_1, symbols, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Group {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            symbols: symbols.unwrap_alternative_plus_8(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "(" Layout first:Symbol(0) Layout rest:("|" Symbol)+ Layout ")" return 0.
                    SlotId(94) => {
                        let [lit_0, layout_1, first, layout_3, rest, layout_5, lit_6] =
                            <[ParseTree; 7usize]>::try_from(children).unwrap();
                        Symbol::Alt {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            first: Box::new(first.unwrap_symbol()),
                            layout_3: layout_3.unwrap_token(),
                            rest: rest.unwrap_symbol_plus_9(),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : """ Layout String Layout """ return 0.
                    SlotId(101) => {
                        let [lit_0, layout_1, string, layout_3, lit_4] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Lit {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            string: string.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            lit_4: lit_4.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0.
                    SlotId(112) => {
                        let [
                            lit_0,
                            layout_1,
                            symbol,
                            layout_3,
                            sep,
                            layout_5,
                            lit_6,
                            layout_7,
                            lit_8,
                        ] = <[ParseTree; 9usize]>::try_from(children).unwrap();
                        Symbol::StarSep {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            symbol: Box::new(symbol.unwrap_symbol()),
                            layout_3: layout_3.unwrap_token(),
                            sep: Box::new(sep.unwrap_symbol()),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            layout_7: layout_7.unwrap_token(),
                            lit_8: lit_8.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0.
                    SlotId(123) => {
                        let [
                            lit_0,
                            layout_1,
                            symbol,
                            layout_3,
                            sep,
                            layout_5,
                            lit_6,
                            layout_7,
                            lit_8,
                        ] = <[ParseTree; 9usize]>::try_from(children).unwrap();
                        Symbol::PlusSep {
                            lit_0: lit_0.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            symbol: Box::new(symbol.unwrap_symbol()),
                            layout_3: layout_3.unwrap_token(),
                            sep: Box::new(sep.unwrap_symbol()),
                            layout_5: layout_5.unwrap_token(),
                            lit_6: lit_6.unwrap_token(),
                            layout_7: layout_7.unwrap_token(),
                            lit_8: lit_8.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0.
                    SlotId(130) => {
                        let [symbol, layout, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Star {
                            symbol: Box::new(symbol.unwrap_symbol()),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0.
                    SlotId(137) => {
                        let [symbol, layout, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Plus {
                            symbol: Box::new(symbol.unwrap_symbol()),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0.
                    SlotId(144) => {
                        let [symbol, layout, lit_2] =
                            <[ParseTree; 3usize]>::try_from(children).unwrap();
                        Symbol::Opt {
                            symbol: Box::new(symbol.unwrap_symbol()),
                            layout: layout.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "\" Layout Identifier return 0.
                    SlotId(153) => {
                        let [symbol, layout_1, lit_2, layout_3, identifier] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Except {
                            symbol: Box::new(symbol.unwrap_symbol()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            identifier: identifier.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0.
                    SlotId(162) => {
                        let [symbol, layout_1, lit_2, layout_3, identifier] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::FollowRestriction {
                            symbol: Box::new(symbol.unwrap_symbol()),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            identifier: identifier.unwrap_token(),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2).
                    SlotId(169) => {
                        let [identifier, layout_1, lit_2, layout_3, symbol] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::PrecedeRestriction {
                            identifier: identifier.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            symbol: Box::new(symbol.unwrap_symbol()),
                            span: nonterminal_node.span,
                        }
                        .into()
                    }
                    //Symbol : label:Identifier Layout ":" Layout Symbol(1) return 1.
                    SlotId(176) => {
                        let [label, layout_1, lit_2, layout_3, symbol] =
                            <[ParseTree; 5usize]>::try_from(children).unwrap();
                        Symbol::Labeled {
                            label: label.unwrap_token(),
                            layout_1: layout_1.unwrap_token(),
                            lit_2: lit_2.unwrap_token(),
                            layout_3: layout_3.unwrap_token(),
                            symbol: Box::new(symbol.unwrap_symbol()),
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
        "LayoutDef" => ParseTree::LayoutDef(create_parse_tree_layout_def(root_id, parser, builder)),
        "SyntaxRule" => {
            ParseTree::SyntaxRule(create_parse_tree_syntax_rule(root_id, parser, builder))
        }
        "Annotation" => {
            ParseTree::Annotation(create_parse_tree_annotation(root_id, parser, builder))
        }
        "RegexBlock" => {
            ParseTree::RegexBlock(create_parse_tree_regex_block(root_id, parser, builder))
        }
        "RegexRule" => ParseTree::RegexRule(create_parse_tree_regex_rule(root_id, parser, builder)),
        "PreCondition" => {
            ParseTree::PreCondition(create_parse_tree_pre_condition(root_id, parser, builder))
        }
        "PostCondition" => {
            ParseTree::PostCondition(create_parse_tree_post_condition(root_id, parser, builder))
        }
        "PriorityLevel" => {
            ParseTree::PriorityLevel(create_parse_tree_priority_level(root_id, parser, builder))
        }
        "Associativity" => {
            ParseTree::Associativity(create_parse_tree_associativity(root_id, parser, builder))
        }
        "Alternative" => {
            ParseTree::Alternative(create_parse_tree_alternative(root_id, parser, builder))
        }
        "Symbol" => ParseTree::Symbol(create_parse_tree_symbol(root_id, parser, builder)),
        "Regex" => ParseTree::Regex(create_parse_tree_regex(root_id, parser, builder)),
        "CharClass" => ParseTree::CharClass(create_parse_tree_char_class(root_id, parser, builder)),
        "RangeElement" => {
            ParseTree::RangeElement(create_parse_tree_range_element(root_id, parser, builder))
        }
        "Range" => ParseTree::Range(create_parse_tree_range(root_id, parser, builder)),
        "Grammar_Opt_0" => {
            ParseTree::GrammarOpt0(create_parse_tree_grammar_opt_0(root_id, parser, builder))
        }
        "Grammar_Plus_0" => {
            ParseTree::GrammarPlus0(create_parse_tree_grammar_plus_0(root_id, parser, builder))
        }
        "Grammar_Opt_1" => {
            ParseTree::GrammarOpt1(create_parse_tree_grammar_opt_1(root_id, parser, builder))
        }
        "Grammar_Star_0" => {
            ParseTree::GrammarStar0(create_parse_tree_grammar_star_0(root_id, parser, builder))
        }
        "Grammar_Opt_2" => {
            ParseTree::GrammarOpt2(create_parse_tree_grammar_opt_2(root_id, parser, builder))
        }
        "LayoutDef_Plus_1" => ParseTree::LayoutDefPlus1(create_parse_tree_layout_def_plus_1(
            root_id, parser, builder,
        )),
        "LayoutDef_Opt_3" => {
            ParseTree::LayoutDefOpt3(create_parse_tree_layout_def_opt_3(root_id, parser, builder))
        }
        "LayoutDef_Star_1" => ParseTree::LayoutDefStar1(create_parse_tree_layout_def_star_1(
            root_id, parser, builder,
        )),
        "SyntaxRule_Opt_4" => ParseTree::SyntaxRuleOpt4(create_parse_tree_syntax_rule_opt_4(
            root_id, parser, builder,
        )),
        "SyntaxRule_Plus_2" => ParseTree::SyntaxRulePlus2(create_parse_tree_syntax_rule_plus_2(
            root_id, parser, builder,
        )),
        "SyntaxRule_Opt_5" => ParseTree::SyntaxRuleOpt5(create_parse_tree_syntax_rule_opt_5(
            root_id, parser, builder,
        )),
        "SyntaxRule_Star_2" => ParseTree::SyntaxRuleStar2(create_parse_tree_syntax_rule_star_2(
            root_id, parser, builder,
        )),
        "RegexBlock_Plus_3" => ParseTree::RegexBlockPlus3(create_parse_tree_regex_block_plus_3(
            root_id, parser, builder,
        )),
        "RegexBlock_Opt_6" => ParseTree::RegexBlockOpt6(create_parse_tree_regex_block_opt_6(
            root_id, parser, builder,
        )),
        "RegexBlock_Star_3" => ParseTree::RegexBlockStar3(create_parse_tree_regex_block_star_3(
            root_id, parser, builder,
        )),
        "RegexRule_Opt_7" => {
            ParseTree::RegexRuleOpt7(create_parse_tree_regex_rule_opt_7(root_id, parser, builder))
        }
        "RegexRule_Plus_5" => ParseTree::RegexRulePlus5(create_parse_tree_regex_rule_plus_5(
            root_id, parser, builder,
        )),
        "RegexRule_Plus_4" => ParseTree::RegexRulePlus4(create_parse_tree_regex_rule_plus_4(
            root_id, parser, builder,
        )),
        "RegexRule_Plus_6" => ParseTree::RegexRulePlus6(create_parse_tree_regex_rule_plus_6(
            root_id, parser, builder,
        )),
        "RegexRule_Opt_8" => {
            ParseTree::RegexRuleOpt8(create_parse_tree_regex_rule_opt_8(root_id, parser, builder))
        }
        "RegexRule_Star_4" => ParseTree::RegexRuleStar4(create_parse_tree_regex_rule_star_4(
            root_id, parser, builder,
        )),
        "PriorityLevel_Opt_9" => ParseTree::PriorityLevelOpt9(
            create_parse_tree_priority_level_opt_9(root_id, parser, builder),
        ),
        "PriorityLevel_Plus_7" => ParseTree::PriorityLevelPlus7(
            create_parse_tree_priority_level_plus_7(root_id, parser, builder),
        ),
        "PriorityLevel_Opt_10" => ParseTree::PriorityLevelOpt10(
            create_parse_tree_priority_level_opt_10(root_id, parser, builder),
        ),
        "PriorityLevel_Star_5" => ParseTree::PriorityLevelStar5(
            create_parse_tree_priority_level_star_5(root_id, parser, builder),
        ),
        "Alternative_Plus_8" => ParseTree::AlternativePlus8(create_parse_tree_alternative_plus_8(
            root_id, parser, builder,
        )),
        "Alternative_Opt_11" => ParseTree::AlternativeOpt11(create_parse_tree_alternative_opt_11(
            root_id, parser, builder,
        )),
        "Alternative_Star_6" => ParseTree::AlternativeStar6(create_parse_tree_alternative_star_6(
            root_id, parser, builder,
        )),
        "Alternative_Opt_12" => ParseTree::AlternativeOpt12(create_parse_tree_alternative_opt_12(
            root_id, parser, builder,
        )),
        "Symbol_Group_0" => {
            ParseTree::SymbolGroup0(create_parse_tree_symbol_group_0(root_id, parser, builder))
        }
        "Symbol_Plus_9" => {
            ParseTree::SymbolPlus9(create_parse_tree_symbol_plus_9(root_id, parser, builder))
        }
        "Regex_Group_1" => {
            ParseTree::RegexGroup1(create_parse_tree_regex_group_1(root_id, parser, builder))
        }
        "Regex_Plus_10" => {
            ParseTree::RegexPlus10(create_parse_tree_regex_plus_10(root_id, parser, builder))
        }
        "CharClass_Opt_13" => ParseTree::CharClassOpt13(create_parse_tree_char_class_opt_13(
            root_id, parser, builder,
        )),
        "CharClass_Plus_11" => ParseTree::CharClassPlus11(create_parse_tree_char_class_plus_11(
            root_id, parser, builder,
        )),
        "StartGrammar" => {
            ParseTree::StartGrammar(create_parse_tree_start_grammar(root_id, parser, builder))
        }
        "StartLayoutDef" => {
            ParseTree::StartLayoutDef(create_parse_tree_start_layout_def(root_id, parser, builder))
        }
        "StartSyntaxRule" => ParseTree::StartSyntaxRule(create_parse_tree_start_syntax_rule(
            root_id, parser, builder,
        )),
        "StartAnnotation" => {
            ParseTree::StartAnnotation(create_parse_tree_start_annotation(root_id, parser, builder))
        }
        "StartRegexBlock" => ParseTree::StartRegexBlock(create_parse_tree_start_regex_block(
            root_id, parser, builder,
        )),
        "StartRegexRule" => {
            ParseTree::StartRegexRule(create_parse_tree_start_regex_rule(root_id, parser, builder))
        }
        "StartPreCondition" => ParseTree::StartPreCondition(create_parse_tree_start_pre_condition(
            root_id, parser, builder,
        )),
        "StartPostCondition" => ParseTree::StartPostCondition(
            create_parse_tree_start_post_condition(root_id, parser, builder),
        ),
        "StartPriorityLevel" => ParseTree::StartPriorityLevel(
            create_parse_tree_start_priority_level(root_id, parser, builder),
        ),
        "StartAssociativity" => ParseTree::StartAssociativity(
            create_parse_tree_start_associativity(root_id, parser, builder),
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
        "StartRangeElement" => ParseTree::StartRangeElement(create_parse_tree_start_range_element(
            root_id, parser, builder,
        )),
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
pub fn create_parse_tree_layout_def(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> LayoutDef {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_layout_def()
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
pub fn create_parse_tree_annotation(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Annotation {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_annotation()
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
pub fn create_parse_tree_pre_condition(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PreCondition {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_pre_condition()
}
pub fn create_parse_tree_post_condition(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PostCondition {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_post_condition()
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
pub fn create_parse_tree_associativity(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> Associativity {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_associativity()
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
pub fn create_parse_tree_range_element(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RangeElement {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_range_element()
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
pub fn create_parse_tree_grammar_opt_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> GrammarOpt2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_grammar_opt_2()
}
pub fn create_parse_tree_layout_def_plus_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> LayoutDefPlus1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_layout_def_plus_1()
}
pub fn create_parse_tree_layout_def_opt_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> LayoutDefOpt3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_layout_def_opt_3()
}
pub fn create_parse_tree_layout_def_star_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> LayoutDefStar1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_layout_def_star_1()
}
pub fn create_parse_tree_syntax_rule_opt_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRuleOpt4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule_opt_4()
}
pub fn create_parse_tree_syntax_rule_plus_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRulePlus2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule_plus_2()
}
pub fn create_parse_tree_syntax_rule_opt_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRuleOpt5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule_opt_5()
}
pub fn create_parse_tree_syntax_rule_star_2(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SyntaxRuleStar2 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_syntax_rule_star_2()
}
pub fn create_parse_tree_regex_block_plus_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexBlockPlus3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_block_plus_3()
}
pub fn create_parse_tree_regex_block_opt_6(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexBlockOpt6 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_block_opt_6()
}
pub fn create_parse_tree_regex_block_star_3(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexBlockStar3 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_block_star_3()
}
pub fn create_parse_tree_regex_rule_opt_7(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRuleOpt7 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule_opt_7()
}
pub fn create_parse_tree_regex_rule_plus_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRulePlus5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule_plus_5()
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
pub fn create_parse_tree_regex_rule_plus_6(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRulePlus6 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule_plus_6()
}
pub fn create_parse_tree_regex_rule_opt_8(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRuleOpt8 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule_opt_8()
}
pub fn create_parse_tree_regex_rule_star_4(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexRuleStar4 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_rule_star_4()
}
pub fn create_parse_tree_priority_level_opt_9(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelOpt9 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level_opt_9()
}
pub fn create_parse_tree_priority_level_plus_7(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelPlus7 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level_plus_7()
}
pub fn create_parse_tree_priority_level_opt_10(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelOpt10 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level_opt_10()
}
pub fn create_parse_tree_priority_level_star_5(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> PriorityLevelStar5 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_priority_level_star_5()
}
pub fn create_parse_tree_alternative_plus_8(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativePlus8 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_plus_8()
}
pub fn create_parse_tree_alternative_opt_11(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativeOpt11 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_opt_11()
}
pub fn create_parse_tree_alternative_star_6(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativeStar6 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_star_6()
}
pub fn create_parse_tree_alternative_opt_12(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> AlternativeOpt12 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_alternative_opt_12()
}
pub fn create_parse_tree_symbol_group_0(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SymbolGroup0 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_symbol_group_0()
}
pub fn create_parse_tree_symbol_plus_9(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> SymbolPlus9 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_symbol_plus_9()
}
pub fn create_parse_tree_regex_group_1(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexGroup1 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_group_1()
}
pub fn create_parse_tree_regex_plus_10(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> RegexPlus10 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_regex_plus_10()
}
pub fn create_parse_tree_char_class_opt_13(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassOpt13 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_char_class_opt_13()
}
pub fn create_parse_tree_char_class_plus_11(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> CharClassPlus11 {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_char_class_plus_11()
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
pub fn create_parse_tree_start_layout_def(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartLayoutDef {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_layout_def()
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
pub fn create_parse_tree_start_annotation(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartAnnotation {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_annotation()
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
pub fn create_parse_tree_start_pre_condition(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartPreCondition {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_pre_condition()
}
pub fn create_parse_tree_start_post_condition(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartPostCondition {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_post_condition()
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
pub fn create_parse_tree_start_associativity(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartAssociativity {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_associativity()
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
pub fn create_parse_tree_start_range_element(
    root_id: SPPFNodeId,
    parser: &IggyParser,
    builder: &IggyParseTreeBuilder,
) -> StartRangeElement {
    let node = parser.sppf_node(root_id);
    visit_sppf(node, parser, builder)
        .unwrap_one()
        .unwrap_start_range_element()
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

