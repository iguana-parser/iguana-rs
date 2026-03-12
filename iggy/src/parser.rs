// grammar Iggy
//
// Grammar
//   = "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0
//
// LayoutDef
//   = "layout" Layout LayoutDef_Star_1
//
// Rule
//   = SyntaxRule #SyntaxRule
//   | RegexRule #RegexRule
//
// SyntaxRule
//   = SyntaxRule_Opt_3 Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2
//
// Annotation
//   = "@NoLayout" #NoLayout
//   | "@Layout" Layout "(" Layout Identifier Layout ")" #Layout
//
// RegexRule
//   = "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
//
// PreCondition
//   = Identifier Layout "!<<" #PrecedeRestriction
//
// PostCondition
//   = "\" Layout Identifier #Except
//   | "!>>" Layout Identifier #FollowRestriction
//
// PriorityLevel
//   = PriorityLevel_Opt_7 Layout PriorityLevel_Star_4
//
// Associativity
//   = "left"
//   | "right"
//   | "none"
//
// Alternative
//   = Alternative_Star_5 Layout Alternative_Opt_10
//
// Symbol(p: i32)
//   = Identifier return 0 #Identifier
//   | "(" Layout Alternative_Plus_7 Layout ")" return 0 #Group
//   | "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0 #Alt
//   | """ Layout String Layout """ return 0 #Lit
//   | "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0 #StarSep
//   | "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0 #PlusSep
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0 #Star
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0 #Plus
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0 #Opt
//   | [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout excepts:Symbol_Plus_9 return 0 #Except
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0 #FollowRestriction
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0 #Exclude
//   | Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2) #PrecedeRestriction
//   | label:Identifier Layout ":" Layout Symbol(1) return 1 #Labeled
//
// Regex
//   = Regex Layout "+" #Plus
//   | Regex Layout "*" #Star
//   | Regex Layout "?" #Opt
//   | "(" Layout first:Regex Layout rest:Regex_Plus_11 Layout ")" #Alt
//   | "(" Layout RegexRule_Plus_4 Layout ")" #Group
//   | CharClass #CharClass
//   | "'" Layout Char Layout "'" #Char
//   | """ Layout String Layout """ #String
//   | Identifier #Identifier
//
// CharClass
//   = neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 Layout "]"
//
// RangeElement
//   = Range
//   | RangeChar
//
// Range
//   = start:RangeChar Layout "-" Layout end:RangeChar
//
// Grammar_Opt_0
//   = LayoutDef
//   |
//
// Grammar_Plus_0
//   = Grammar_Plus_0 Layout Rule
//   | Rule
//
// Grammar_Opt_1
//   = Grammar_Plus_0
//   |
//
// Grammar_Star_0
//   = Grammar_Opt_1
//
// LayoutDef_Plus_1
//   = LayoutDef_Plus_1 Layout Identifier
//   | Identifier
//
// LayoutDef_Opt_2
//   = LayoutDef_Plus_1
//   |
//
// LayoutDef_Star_1
//   = LayoutDef_Opt_2
//
// SyntaxRule_Opt_3
//   = Annotation
//   |
//
// SyntaxRule_Plus_2
//   = SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel
//   | PriorityLevel
//
// SyntaxRule_Opt_4
//   = SyntaxRule_Plus_2
//   |
//
// SyntaxRule_Star_2
//   = SyntaxRule_Opt_4
//
// RegexRule_Opt_5
//   = PreCondition
//   |
//
// RegexRule_Plus_4
//   = RegexRule_Plus_4 Layout Regex
//   | Regex
//
// RegexRule_Plus_3
//   = RegexRule_Plus_3 Layout "|" Layout RegexRule_Plus_4
//   | RegexRule_Plus_4
//
// RegexRule_Plus_5
//   = RegexRule_Plus_5 Layout PostCondition
//   | PostCondition
//
// RegexRule_Opt_6
//   = RegexRule_Plus_5
//   |
//
// RegexRule_Star_3
//   = RegexRule_Opt_6
//
// PriorityLevel_Opt_7
//   = Associativity
//   |
//
// PriorityLevel_Plus_6
//   = PriorityLevel_Plus_6 Layout "|" Layout Alternative
//   | Alternative
//
// PriorityLevel_Opt_8
//   = PriorityLevel_Plus_6
//   |
//
// PriorityLevel_Star_4
//   = PriorityLevel_Opt_8
//
// Alternative_Plus_7
//   = Alternative_Plus_7 Layout Symbol(0)
//   | Symbol(0)
//
// Alternative_Opt_9
//   = Alternative_Plus_7
//   |
//
// Alternative_Star_5
//   = Alternative_Opt_9
//
// Alternative_Opt_10
//   = Label
//   |
//
// Symbol_Group_0
//   = "|" Layout Symbol(0)
//
// Symbol_Plus_8
//   = Symbol_Plus_8 Layout Symbol_Group_0
//   | Symbol_Group_0
//
// Symbol_Group_1
//   = "\" Layout Identifier
//
// Symbol_Plus_9
//   = Symbol_Plus_9 Layout Symbol_Group_1
//   | Symbol_Group_1
//
// Symbol_Group_2
//   = "!" Layout Identifier
//
// Symbol_Plus_10
//   = Symbol_Plus_10 Layout Symbol_Group_2
//   | Symbol_Group_2
//
// Regex_Group_3
//   = "|" Layout Regex
//
// Regex_Plus_11
//   = Regex_Plus_11 Layout Regex_Group_3
//   | Regex_Group_3
//
// CharClass_Opt_11
//   = "!"
//   |
//
// CharClass_Plus_12
//   = CharClass_Plus_12 Layout RangeElement
//   | RangeElement
//
// Symbol_except_Except(p: i32)
//   = Identifier return 0 #Identifier
//   | "(" Layout Alternative_Plus_7 Layout ")" return 0 #Group
//   | "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0 #Alt
//   | """ Layout String Layout """ return 0 #Lit
//   | "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0 #StarSep
//   | "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0 #PlusSep
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0 #Star
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0 #Plus
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0 #Opt
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0 #FollowRestriction
//   | [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0 #Exclude
//   | Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2) #PrecedeRestriction
//   | label:Identifier Layout ":" Layout Symbol(1) return 1 #Labeled
//
// StartGrammar
//   = Layout start:Grammar Layout
//
// StartLayoutDef
//   = Layout start:LayoutDef Layout
//
// StartRule
//   = Layout start:Rule Layout
//
// StartSyntaxRule
//   = Layout start:SyntaxRule Layout
//
// StartAnnotation
//   = Layout start:Annotation Layout
//
// StartRegexRule
//   = Layout start:RegexRule Layout
//
// StartPreCondition
//   = Layout start:PreCondition Layout
//
// StartPostCondition
//   = Layout start:PostCondition Layout
//
// StartPriorityLevel
//   = Layout start:PriorityLevel Layout
//
// StartAssociativity
//   = Layout start:Associativity Layout
//
// StartAlternative
//   = Layout start:Alternative Layout
//
// StartSymbol
//   = Layout start:Symbol(0) Layout
//
// StartRegex
//   = Layout start:Regex Layout
//
// StartCharClass
//   = Layout start:CharClass Layout
//
// StartRangeElement
//   = Layout start:RangeElement Layout
//
// StartRange
//   = Layout start:Range Layout
//
// Keyword = (grammar|layout|left|right|none)
// Identifier = ([a-z A-Z _][a-z A-Z _ 0-9]*) \ Keyword
// String = (((\\[\" \\ t f r n]|![\" \\]))*)
// RangeChar = (![\\ - [ ] \t \u{c} \r \n  ]|\\[\\ - [ ] t f r n  ])
// Char = (\\[\' \\ t f r n]|![\' \\])
// Label = (#[a-z A-Z _][a-z A-Z _ 0-9]*)
// WS = ([  \n \t]*)
// "grammar" = grammar
// "layout" = layout
// "=" = =
// ">" = >
// "@NoLayout" = @NoLayout
// "@Layout" = @Layout
// "(" = (
// ")" = )
// "@regex" = @regex
// "|" = |
// "!<<" = !<<
// "\" = \\
// "!>>" = !>>
// "left" = left
// "right" = right
// "none" = none
// """ = \"
// "{" = {
// "}" = }
// "*" = *
// "+" = +
// "?" = ?
// "!" = !
// ":" = :
// "'" = \'
// "[" = [
// "]" = ]
// "-" = -
// Layout = ([  \n \t]*)
use crate::{
    scanner::IggyScanner,
    types::{EbnfKind, Nonterminal, Slot, Terminal},
};
#[cfg(feature = "debug-trace")]
use iguana_runtime::trace::TraceEvent;
use iguana_runtime::{
    descriptor::Descriptor,
    env::{Env, EnvId},
    gss::{GSSNode, PoppedElement},
    ids::{GssNodeId, NonterminalId, SlotId, TerminalId},
    input::Input,
    parser::{Parser, Stats, init_logger},
    record,
    scanner::Scanner,
    sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
    utils::{inline_map::InlineMap, inline_vec::InlineVec},
};
use phf::phf_map;
use rustc_hash::FxHashMap;
use std::cell::OnceCell;
pub const NONTERMINALS: [Nonterminal; 68] = [
    Nonterminal {
        name: "Grammar",
        display: "Grammar",
        kind: None,
    },
    Nonterminal {
        name: "LayoutDef",
        display: "LayoutDef",
        kind: None,
    },
    Nonterminal {
        name: "Rule",
        display: "Rule",
        kind: None,
    },
    Nonterminal {
        name: "SyntaxRule",
        display: "SyntaxRule",
        kind: None,
    },
    Nonterminal {
        name: "Annotation",
        display: "Annotation",
        kind: None,
    },
    Nonterminal {
        name: "RegexRule",
        display: "RegexRule",
        kind: None,
    },
    Nonterminal {
        name: "PreCondition",
        display: "PreCondition",
        kind: None,
    },
    Nonterminal {
        name: "PostCondition",
        display: "PostCondition",
        kind: None,
    },
    Nonterminal {
        name: "PriorityLevel",
        display: "PriorityLevel",
        kind: None,
    },
    Nonterminal {
        name: "Associativity",
        display: "Associativity",
        kind: None,
    },
    Nonterminal {
        name: "Alternative",
        display: "Alternative",
        kind: None,
    },
    Nonterminal {
        name: "Regex",
        display: "Regex",
        kind: None,
    },
    Nonterminal {
        name: "CharClass",
        display: "CharClass",
        kind: None,
    },
    Nonterminal {
        name: "RangeElement",
        display: "RangeElement",
        kind: None,
    },
    Nonterminal {
        name: "Range",
        display: "Range",
        kind: None,
    },
    Nonterminal {
        name: "Grammar_Opt_0",
        display: "LayoutDef?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Grammar_Plus_0",
        display: "Rule+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Grammar_Opt_1",
        display: "Rule+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Grammar_Star_0",
        display: "Rule*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "LayoutDef_Plus_1",
        display: "Identifier+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "LayoutDef_Opt_2",
        display: "Identifier+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "LayoutDef_Star_1",
        display: "Identifier*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "SyntaxRule_Opt_3",
        display: "Annotation?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "SyntaxRule_Plus_2",
        display: "{PriorityLevel \">\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "SyntaxRule_Opt_4",
        display: "{PriorityLevel \">\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "SyntaxRule_Star_2",
        display: "{PriorityLevel \">\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "RegexRule_Opt_5",
        display: "PreCondition?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "RegexRule_Plus_4",
        display: "Regex+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "RegexRule_Plus_3",
        display: "{Regex+ \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "RegexRule_Plus_5",
        display: "PostCondition+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "RegexRule_Opt_6",
        display: "PostCondition+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "RegexRule_Star_3",
        display: "PostCondition*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_7",
        display: "Associativity?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "PriorityLevel_Plus_6",
        display: "{Alternative \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_8",
        display: "{Alternative \"|\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "PriorityLevel_Star_4",
        display: "{Alternative \"|\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Plus_7",
        display: "Symbol+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Alternative_Opt_9",
        display: "Symbol+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Alternative_Star_5",
        display: "Symbol*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Opt_10",
        display: "Label?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Symbol_Group_0",
        display: "(\"|\" Symbol)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Symbol_Plus_8",
        display: "(\"|\" Symbol)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Symbol_Group_1",
        display: "(\"\\\" Identifier)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Symbol_Plus_9",
        display: "(\"\\\" Identifier)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Symbol_Group_2",
        display: "(\"!\" Identifier)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Symbol_Plus_10",
        display: "(\"!\" Identifier)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Regex_Group_3",
        display: "(\"|\" Regex)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Regex_Plus_11",
        display: "(\"|\" Regex)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "CharClass_Opt_11",
        display: "\"!\"?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "CharClass_Plus_12",
        display: "RangeElement+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "StartGrammar",
        display: "StartGrammar",
        kind: None,
    },
    Nonterminal {
        name: "StartLayoutDef",
        display: "StartLayoutDef",
        kind: None,
    },
    Nonterminal {
        name: "StartRule",
        display: "StartRule",
        kind: None,
    },
    Nonterminal {
        name: "StartSyntaxRule",
        display: "StartSyntaxRule",
        kind: None,
    },
    Nonterminal {
        name: "StartAnnotation",
        display: "StartAnnotation",
        kind: None,
    },
    Nonterminal {
        name: "StartRegexRule",
        display: "StartRegexRule",
        kind: None,
    },
    Nonterminal {
        name: "StartPreCondition",
        display: "StartPreCondition",
        kind: None,
    },
    Nonterminal {
        name: "StartPostCondition",
        display: "StartPostCondition",
        kind: None,
    },
    Nonterminal {
        name: "StartPriorityLevel",
        display: "StartPriorityLevel",
        kind: None,
    },
    Nonterminal {
        name: "StartAssociativity",
        display: "StartAssociativity",
        kind: None,
    },
    Nonterminal {
        name: "StartAlternative",
        display: "StartAlternative",
        kind: None,
    },
    Nonterminal {
        name: "StartSymbol",
        display: "StartSymbol",
        kind: None,
    },
    Nonterminal {
        name: "StartRegex",
        display: "StartRegex",
        kind: None,
    },
    Nonterminal {
        name: "StartCharClass",
        display: "StartCharClass",
        kind: None,
    },
    Nonterminal {
        name: "StartRangeElement",
        display: "StartRangeElement",
        kind: None,
    },
    Nonterminal {
        name: "StartRange",
        display: "StartRange",
        kind: None,
    },
    Nonterminal {
        name: "Symbol",
        display: "Symbol",
        kind: None,
    },
    Nonterminal {
        name: "Symbol_except_Except",
        display: "Symbol !Except",
        kind: None,
    },
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Grammar" => NonterminalId(0), "LayoutDef" => NonterminalId(1), "Rule" =>
    NonterminalId(2), "SyntaxRule" => NonterminalId(3), "Annotation" => NonterminalId(4),
    "RegexRule" => NonterminalId(5), "PreCondition" => NonterminalId(6), "PostCondition"
    => NonterminalId(7), "PriorityLevel" => NonterminalId(8), "Associativity" =>
    NonterminalId(9), "Alternative" => NonterminalId(10), "Regex" => NonterminalId(11),
    "CharClass" => NonterminalId(12), "RangeElement" => NonterminalId(13), "Range" =>
    NonterminalId(14), "Grammar_Opt_0" => NonterminalId(15), "Grammar_Plus_0" =>
    NonterminalId(16), "Grammar_Opt_1" => NonterminalId(17), "Grammar_Star_0" =>
    NonterminalId(18), "LayoutDef_Plus_1" => NonterminalId(19), "LayoutDef_Opt_2" =>
    NonterminalId(20), "LayoutDef_Star_1" => NonterminalId(21), "SyntaxRule_Opt_3" =>
    NonterminalId(22), "SyntaxRule_Plus_2" => NonterminalId(23), "SyntaxRule_Opt_4" =>
    NonterminalId(24), "SyntaxRule_Star_2" => NonterminalId(25), "RegexRule_Opt_5" =>
    NonterminalId(26), "RegexRule_Plus_4" => NonterminalId(27), "RegexRule_Plus_3" =>
    NonterminalId(28), "RegexRule_Plus_5" => NonterminalId(29), "RegexRule_Opt_6" =>
    NonterminalId(30), "RegexRule_Star_3" => NonterminalId(31), "PriorityLevel_Opt_7" =>
    NonterminalId(32), "PriorityLevel_Plus_6" => NonterminalId(33), "PriorityLevel_Opt_8"
    => NonterminalId(34), "PriorityLevel_Star_4" => NonterminalId(35),
    "Alternative_Plus_7" => NonterminalId(36), "Alternative_Opt_9" => NonterminalId(37),
    "Alternative_Star_5" => NonterminalId(38), "Alternative_Opt_10" => NonterminalId(39),
    "Symbol_Group_0" => NonterminalId(40), "Symbol_Plus_8" => NonterminalId(41),
    "Symbol_Group_1" => NonterminalId(42), "Symbol_Plus_9" => NonterminalId(43),
    "Symbol_Group_2" => NonterminalId(44), "Symbol_Plus_10" => NonterminalId(45),
    "Regex_Group_3" => NonterminalId(46), "Regex_Plus_11" => NonterminalId(47),
    "CharClass_Opt_11" => NonterminalId(48), "CharClass_Plus_12" => NonterminalId(49),
    "StartGrammar" => NonterminalId(50), "StartLayoutDef" => NonterminalId(51),
    "StartRule" => NonterminalId(52), "StartSyntaxRule" => NonterminalId(53),
    "StartAnnotation" => NonterminalId(54), "StartRegexRule" => NonterminalId(55),
    "StartPreCondition" => NonterminalId(56), "StartPostCondition" => NonterminalId(57),
    "StartPriorityLevel" => NonterminalId(58), "StartAssociativity" => NonterminalId(59),
    "StartAlternative" => NonterminalId(60), "StartSymbol" => NonterminalId(61),
    "StartRegex" => NonterminalId(62), "StartCharClass" => NonterminalId(63),
    "StartRangeElement" => NonterminalId(64), "StartRange" => NonterminalId(65), "Symbol"
    => NonterminalId(66), "Symbol_except_Except" => NonterminalId(67)
};
pub const TERMINALS: [Terminal; 37] = [
    Terminal { name: "Keyword" },
    Terminal { name: "Identifier" },
    Terminal { name: "String" },
    Terminal { name: "RangeChar" },
    Terminal { name: "Char" },
    Terminal { name: "Label" },
    Terminal { name: "WS" },
    Terminal {
        name: "\"grammar\"",
    },
    Terminal { name: "\"layout\"" },
    Terminal { name: "\"=\"" },
    Terminal { name: "\">\"" },
    Terminal {
        name: "\"@NoLayout\"",
    },
    Terminal {
        name: "\"@Layout\"",
    },
    Terminal { name: "\"(\"" },
    Terminal { name: "\")\"" },
    Terminal { name: "\"@regex\"" },
    Terminal { name: "\"|\"" },
    Terminal { name: "\"!<<\"" },
    Terminal { name: "\"\\\"" },
    Terminal { name: "\"!>>\"" },
    Terminal { name: "\"left\"" },
    Terminal { name: "\"right\"" },
    Terminal { name: "\"none\"" },
    Terminal { name: "\"\"\"" },
    Terminal { name: "\"{\"" },
    Terminal { name: "\"}\"" },
    Terminal { name: "\"*\"" },
    Terminal { name: "\"+\"" },
    Terminal { name: "\"?\"" },
    Terminal { name: "\"!\"" },
    Terminal { name: "\":\"" },
    Terminal { name: "\"'\"" },
    Terminal { name: "\"[\"" },
    Terminal { name: "\"]\"" },
    Terminal { name: "\"-\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 549] = [
    Slot {
        display_name: "Grammar : . \"grammar\" Layout name:Identifier Layout LayoutDef? Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" . Layout name:Identifier Layout LayoutDef? Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout . name:Identifier Layout LayoutDef? Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier . Layout LayoutDef? Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout . LayoutDef? Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? . Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? Layout . Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? Layout Rule*.",
    },
    Slot {
        display_name: "LayoutDef : . \"layout\" Layout Identifier*",
    },
    Slot {
        display_name: "LayoutDef : \"layout\" . Layout Identifier*",
    },
    Slot {
        display_name: "LayoutDef : \"layout\" Layout . Identifier*",
    },
    Slot {
        display_name: "LayoutDef : \"layout\" Layout Identifier*.",
    },
    Slot {
        display_name: "Rule : . SyntaxRule",
    },
    Slot {
        display_name: "Rule : SyntaxRule.",
    },
    Slot {
        display_name: "Rule : . RegexRule",
    },
    Slot {
        display_name: "Rule : RegexRule.",
    },
    Slot {
        display_name: "SyntaxRule : . Annotation? Layout head:Identifier Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Annotation? . Layout head:Identifier Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Annotation? Layout . head:Identifier Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Annotation? Layout head:Identifier . Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Annotation? Layout head:Identifier Layout . \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Annotation? Layout head:Identifier Layout \"=\" . Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Annotation? Layout head:Identifier Layout \"=\" Layout . {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Annotation? Layout head:Identifier Layout \"=\" Layout {PriorityLevel \">\"}*.",
    },
    Slot {
        display_name: "Annotation : . \"@NoLayout\"",
    },
    Slot {
        display_name: "Annotation : \"@NoLayout\".",
    },
    Slot {
        display_name: "Annotation : . \"@Layout\" Layout \"(\" Layout Identifier Layout \")\"",
    },
    Slot {
        display_name: "Annotation : \"@Layout\" . Layout \"(\" Layout Identifier Layout \")\"",
    },
    Slot {
        display_name: "Annotation : \"@Layout\" Layout . \"(\" Layout Identifier Layout \")\"",
    },
    Slot {
        display_name: "Annotation : \"@Layout\" Layout \"(\" . Layout Identifier Layout \")\"",
    },
    Slot {
        display_name: "Annotation : \"@Layout\" Layout \"(\" Layout . Identifier Layout \")\"",
    },
    Slot {
        display_name: "Annotation : \"@Layout\" Layout \"(\" Layout Identifier . Layout \")\"",
    },
    Slot {
        display_name: "Annotation : \"@Layout\" Layout \"(\" Layout Identifier Layout . \")\"",
    },
    Slot {
        display_name: "Annotation : \"@Layout\" Layout \"(\" Layout Identifier Layout \")\".",
    },
    Slot {
        display_name: "RegexRule : . \"@regex\" Layout Identifier Layout \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" . Layout Identifier Layout \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout . Identifier Layout \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier . Layout \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout . \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout \"=\" . Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout \"=\" Layout . PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout \"=\" Layout PreCondition? . Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout \"=\" Layout PreCondition? Layout . body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ . Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout . PostCondition*",
    },
    Slot {
        display_name: "RegexRule : \"@regex\" Layout Identifier Layout \"=\" Layout PreCondition? Layout body:{Regex+ \"|\"}+ Layout PostCondition*.",
    },
    Slot {
        display_name: "PreCondition : . Identifier Layout \"!<<\"",
    },
    Slot {
        display_name: "PreCondition : Identifier . Layout \"!<<\"",
    },
    Slot {
        display_name: "PreCondition : Identifier Layout . \"!<<\"",
    },
    Slot {
        display_name: "PreCondition : Identifier Layout \"!<<\".",
    },
    Slot {
        display_name: "PostCondition : . \"\\\" Layout Identifier",
    },
    Slot {
        display_name: "PostCondition : \"\\\" . Layout Identifier",
    },
    Slot {
        display_name: "PostCondition : \"\\\" Layout . Identifier",
    },
    Slot {
        display_name: "PostCondition : \"\\\" Layout Identifier.",
    },
    Slot {
        display_name: "PostCondition : . \"!>>\" Layout Identifier",
    },
    Slot {
        display_name: "PostCondition : \"!>>\" . Layout Identifier",
    },
    Slot {
        display_name: "PostCondition : \"!>>\" Layout . Identifier",
    },
    Slot {
        display_name: "PostCondition : \"!>>\" Layout Identifier.",
    },
    Slot {
        display_name: "PriorityLevel : . Associativity? Layout {Alternative \"|\"}*",
    },
    Slot {
        display_name: "PriorityLevel : Associativity? . Layout {Alternative \"|\"}*",
    },
    Slot {
        display_name: "PriorityLevel : Associativity? Layout . {Alternative \"|\"}*",
    },
    Slot {
        display_name: "PriorityLevel : Associativity? Layout {Alternative \"|\"}*.",
    },
    Slot {
        display_name: "Associativity : . \"left\"",
    },
    Slot {
        display_name: "Associativity : \"left\".",
    },
    Slot {
        display_name: "Associativity : . \"right\"",
    },
    Slot {
        display_name: "Associativity : \"right\".",
    },
    Slot {
        display_name: "Associativity : . \"none\"",
    },
    Slot {
        display_name: "Associativity : \"none\".",
    },
    Slot {
        display_name: "Alternative : . Symbol* Layout Label?",
    },
    Slot {
        display_name: "Alternative : Symbol* . Layout Label?",
    },
    Slot {
        display_name: "Alternative : Symbol* Layout . Label?",
    },
    Slot {
        display_name: "Alternative : Symbol* Layout Label?.",
    },
    Slot {
        display_name: "Symbol : . Identifier return 0",
    },
    Slot {
        display_name: "Symbol : Identifier . return 0",
    },
    Slot {
        display_name: "Symbol : Identifier return 0.",
    },
    Slot {
        display_name: "Symbol : . \"(\" Layout Symbol+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" . Layout Symbol+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout . Symbol+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol+ . Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol+ Layout . \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol+ Layout \")\" . return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol+ Layout \")\" return 0.",
    },
    Slot {
        display_name: "Symbol : . \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" . Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout . first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout first:Symbol(0) . Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout first:Symbol(0) Layout . rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ . Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout . \")\" return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" . return 0",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0.",
    },
    Slot {
        display_name: "Symbol : . \"\"\" Layout String Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol : \"\"\" . Layout String Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout . String Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout String . Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout String Layout . \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout String Layout \"\"\" . return 0",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout String Layout \"\"\" return 0.",
    },
    Slot {
        display_name: "Symbol : . \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" . Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout . \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" . return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0.",
    },
    Slot {
        display_name: "Symbol : . \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" . Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout . \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" . return 0",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0.",
    },
    Slot {
        display_name: "Symbol : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" . return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" return 0.",
    },
    Slot {
        display_name: "Symbol : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" . return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" return 0.",
    },
    Slot {
        display_name: "Symbol : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" . return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" return 0.",
    },
    Slot {
        display_name: "Symbol : . [3 >= p] l=Symbol !Except(p) [l == 0 || l >= 3] Layout excepts:(\"\\\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] . l=Symbol !Except(p) [l == 0 || l >= 3] Layout excepts:(\"\\\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol !Except(p) . [l == 0 || l >= 3] Layout excepts:(\"\\\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol !Except(p) [l == 0 || l >= 3] . Layout excepts:(\"\\\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol !Except(p) [l == 0 || l >= 3] Layout . excepts:(\"\\\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol !Except(p) [l == 0 || l >= 3] Layout excepts:(\"\\\" Identifier)+ . return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol !Except(p) [l == 0 || l >= 3] Layout excepts:(\"\\\" Identifier)+ return 0.",
    },
    Slot {
        display_name: "Symbol : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" . Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout . Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier . return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0.",
    },
    Slot {
        display_name: "Symbol : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ . return 0",
    },
    Slot {
        display_name: "Symbol : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0.",
    },
    Slot {
        display_name: "Symbol : . Identifier Layout \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol : Identifier . Layout \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol : Identifier Layout . \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol : Identifier Layout \"!<<\" . Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol : Identifier Layout \"!<<\" Layout . r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol : Identifier Layout \"!<<\" Layout r=Symbol(2) . return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol : Identifier Layout \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2).",
    },
    Slot {
        display_name: "Symbol : . label:Identifier Layout \":\" Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol : label:Identifier . Layout \":\" Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol : label:Identifier Layout . \":\" Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol : label:Identifier Layout \":\" . Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol : label:Identifier Layout \":\" Layout . Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol : label:Identifier Layout \":\" Layout Symbol(1) . return 1",
    },
    Slot {
        display_name: "Symbol : label:Identifier Layout \":\" Layout Symbol(1) return 1.",
    },
    Slot {
        display_name: "Regex : . Regex Layout \"+\"",
    },
    Slot {
        display_name: "Regex : Regex . Layout \"+\"",
    },
    Slot {
        display_name: "Regex : Regex Layout . \"+\"",
    },
    Slot {
        display_name: "Regex : Regex Layout \"+\".",
    },
    Slot {
        display_name: "Regex : . Regex Layout \"*\"",
    },
    Slot {
        display_name: "Regex : Regex . Layout \"*\"",
    },
    Slot {
        display_name: "Regex : Regex Layout . \"*\"",
    },
    Slot {
        display_name: "Regex : Regex Layout \"*\".",
    },
    Slot {
        display_name: "Regex : . Regex Layout \"?\"",
    },
    Slot {
        display_name: "Regex : Regex . Layout \"?\"",
    },
    Slot {
        display_name: "Regex : Regex Layout . \"?\"",
    },
    Slot {
        display_name: "Regex : Regex Layout \"?\".",
    },
    Slot {
        display_name: "Regex : . \"(\" Layout first:Regex Layout rest:(\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" . Layout first:Regex Layout rest:(\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout . first:Regex Layout rest:(\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout first:Regex . Layout rest:(\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout first:Regex Layout . rest:(\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout first:Regex Layout rest:(\"|\" Regex)+ . Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout first:Regex Layout rest:(\"|\" Regex)+ Layout . \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout first:Regex Layout rest:(\"|\" Regex)+ Layout \")\".",
    },
    Slot {
        display_name: "Regex : . \"(\" Layout Regex+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" . Layout Regex+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout . Regex+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex+ . Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex+ Layout . \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex+ Layout \")\".",
    },
    Slot {
        display_name: "Regex : . CharClass",
    },
    Slot {
        display_name: "Regex : CharClass.",
    },
    Slot {
        display_name: "Regex : . \"'\" Layout Char Layout \"'\"",
    },
    Slot {
        display_name: "Regex : \"'\" . Layout Char Layout \"'\"",
    },
    Slot {
        display_name: "Regex : \"'\" Layout . Char Layout \"'\"",
    },
    Slot {
        display_name: "Regex : \"'\" Layout Char . Layout \"'\"",
    },
    Slot {
        display_name: "Regex : \"'\" Layout Char Layout . \"'\"",
    },
    Slot {
        display_name: "Regex : \"'\" Layout Char Layout \"'\".",
    },
    Slot {
        display_name: "Regex : . \"\"\" Layout String Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" . Layout String Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout . String Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout String . Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout String Layout . \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout String Layout \"\"\".",
    },
    Slot {
        display_name: "Regex : . Identifier",
    },
    Slot {
        display_name: "Regex : Identifier.",
    },
    Slot {
        display_name: "CharClass : . neg:\"!\"? Layout \"[\" Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : neg:\"!\"? . Layout \"[\" Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : neg:\"!\"? Layout . \"[\" Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : neg:\"!\"? Layout \"[\" . Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : neg:\"!\"? Layout \"[\" Layout . RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : neg:\"!\"? Layout \"[\" Layout RangeElement+ . Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : neg:\"!\"? Layout \"[\" Layout RangeElement+ Layout . \"]\"",
    },
    Slot {
        display_name: "CharClass : neg:\"!\"? Layout \"[\" Layout RangeElement+ Layout \"]\".",
    },
    Slot {
        display_name: "RangeElement : . Range",
    },
    Slot {
        display_name: "RangeElement : Range.",
    },
    Slot {
        display_name: "RangeElement : . RangeChar",
    },
    Slot {
        display_name: "RangeElement : RangeChar.",
    },
    Slot {
        display_name: "Range : . start:RangeChar Layout \"-\" Layout end:RangeChar",
    },
    Slot {
        display_name: "Range : start:RangeChar . Layout \"-\" Layout end:RangeChar",
    },
    Slot {
        display_name: "Range : start:RangeChar Layout . \"-\" Layout end:RangeChar",
    },
    Slot {
        display_name: "Range : start:RangeChar Layout \"-\" . Layout end:RangeChar",
    },
    Slot {
        display_name: "Range : start:RangeChar Layout \"-\" Layout . end:RangeChar",
    },
    Slot {
        display_name: "Range : start:RangeChar Layout \"-\" Layout end:RangeChar.",
    },
    Slot {
        display_name: "LayoutDef? : . LayoutDef",
    },
    Slot {
        display_name: "LayoutDef? : LayoutDef.",
    },
    Slot {
        display_name: "LayoutDef? : .",
    },
    Slot {
        display_name: "Rule+ : . Rule+ Layout Rule",
    },
    Slot {
        display_name: "Rule+ : Rule+ . Layout Rule",
    },
    Slot {
        display_name: "Rule+ : Rule+ Layout . Rule",
    },
    Slot {
        display_name: "Rule+ : Rule+ Layout Rule.",
    },
    Slot {
        display_name: "Rule+ : . Rule",
    },
    Slot {
        display_name: "Rule+ : Rule.",
    },
    Slot {
        display_name: "Rule+? : . Rule+",
    },
    Slot {
        display_name: "Rule+? : Rule+.",
    },
    Slot {
        display_name: "Rule+? : .",
    },
    Slot {
        display_name: "Rule* : . Rule+?",
    },
    Slot {
        display_name: "Rule* : Rule+?.",
    },
    Slot {
        display_name: "Identifier+ : . Identifier+ Layout Identifier",
    },
    Slot {
        display_name: "Identifier+ : Identifier+ . Layout Identifier",
    },
    Slot {
        display_name: "Identifier+ : Identifier+ Layout . Identifier",
    },
    Slot {
        display_name: "Identifier+ : Identifier+ Layout Identifier.",
    },
    Slot {
        display_name: "Identifier+ : . Identifier",
    },
    Slot {
        display_name: "Identifier+ : Identifier.",
    },
    Slot {
        display_name: "Identifier+? : . Identifier+",
    },
    Slot {
        display_name: "Identifier+? : Identifier+.",
    },
    Slot {
        display_name: "Identifier+? : .",
    },
    Slot {
        display_name: "Identifier* : . Identifier+?",
    },
    Slot {
        display_name: "Identifier* : Identifier+?.",
    },
    Slot {
        display_name: "Annotation? : . Annotation",
    },
    Slot {
        display_name: "Annotation? : Annotation.",
    },
    Slot {
        display_name: "Annotation? : .",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : . {PriorityLevel \">\"}+ Layout \">\" Layout PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ . Layout \">\" Layout PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ Layout . \">\" Layout PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ Layout \">\" . Layout PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ Layout \">\" Layout . PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ Layout \">\" Layout PriorityLevel.",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : . PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : PriorityLevel.",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+? : . {PriorityLevel \">\"}+",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+? : {PriorityLevel \">\"}+.",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+? : .",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}* : . {PriorityLevel \">\"}+?",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}* : {PriorityLevel \">\"}+?.",
    },
    Slot {
        display_name: "PreCondition? : . PreCondition",
    },
    Slot {
        display_name: "PreCondition? : PreCondition.",
    },
    Slot {
        display_name: "PreCondition? : .",
    },
    Slot {
        display_name: "Regex+ : . Regex+ Layout Regex",
    },
    Slot {
        display_name: "Regex+ : Regex+ . Layout Regex",
    },
    Slot {
        display_name: "Regex+ : Regex+ Layout . Regex",
    },
    Slot {
        display_name: "Regex+ : Regex+ Layout Regex.",
    },
    Slot {
        display_name: "Regex+ : . Regex",
    },
    Slot {
        display_name: "Regex+ : Regex.",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : . {Regex+ \"|\"}+ Layout \"|\" Layout Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ . Layout \"|\" Layout Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ Layout . \"|\" Layout Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ Layout \"|\" . Layout Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ Layout \"|\" Layout . Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ Layout \"|\" Layout Regex+.",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : . Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : Regex+.",
    },
    Slot {
        display_name: "PostCondition+ : . PostCondition+ Layout PostCondition",
    },
    Slot {
        display_name: "PostCondition+ : PostCondition+ . Layout PostCondition",
    },
    Slot {
        display_name: "PostCondition+ : PostCondition+ Layout . PostCondition",
    },
    Slot {
        display_name: "PostCondition+ : PostCondition+ Layout PostCondition.",
    },
    Slot {
        display_name: "PostCondition+ : . PostCondition",
    },
    Slot {
        display_name: "PostCondition+ : PostCondition.",
    },
    Slot {
        display_name: "PostCondition+? : . PostCondition+",
    },
    Slot {
        display_name: "PostCondition+? : PostCondition+.",
    },
    Slot {
        display_name: "PostCondition+? : .",
    },
    Slot {
        display_name: "PostCondition* : . PostCondition+?",
    },
    Slot {
        display_name: "PostCondition* : PostCondition+?.",
    },
    Slot {
        display_name: "Associativity? : . Associativity",
    },
    Slot {
        display_name: "Associativity? : Associativity.",
    },
    Slot {
        display_name: "Associativity? : .",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : . {Alternative \"|\"}+ Layout \"|\" Layout Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ . Layout \"|\" Layout Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ Layout . \"|\" Layout Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ Layout \"|\" . Layout Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ Layout \"|\" Layout . Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ Layout \"|\" Layout Alternative.",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : . Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : Alternative.",
    },
    Slot {
        display_name: "{Alternative \"|\"}+? : . {Alternative \"|\"}+",
    },
    Slot {
        display_name: "{Alternative \"|\"}+? : {Alternative \"|\"}+.",
    },
    Slot {
        display_name: "{Alternative \"|\"}+? : .",
    },
    Slot {
        display_name: "{Alternative \"|\"}* : . {Alternative \"|\"}+?",
    },
    Slot {
        display_name: "{Alternative \"|\"}* : {Alternative \"|\"}+?.",
    },
    Slot {
        display_name: "Symbol+ : . Symbol+ Layout Symbol(0)",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ . Layout Symbol(0)",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ Layout . Symbol(0)",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ Layout Symbol(0).",
    },
    Slot {
        display_name: "Symbol+ : . Symbol(0)",
    },
    Slot {
        display_name: "Symbol+ : Symbol(0).",
    },
    Slot {
        display_name: "Symbol+? : . Symbol+",
    },
    Slot {
        display_name: "Symbol+? : Symbol+.",
    },
    Slot {
        display_name: "Symbol+? : .",
    },
    Slot {
        display_name: "Symbol* : . Symbol+?",
    },
    Slot {
        display_name: "Symbol* : Symbol+?.",
    },
    Slot {
        display_name: "Label? : . Label",
    },
    Slot {
        display_name: "Label? : Label.",
    },
    Slot {
        display_name: "Label? : .",
    },
    Slot {
        display_name: "(\"|\" Symbol) : . \"|\" Layout Symbol(0)",
    },
    Slot {
        display_name: "(\"|\" Symbol) : \"|\" . Layout Symbol(0)",
    },
    Slot {
        display_name: "(\"|\" Symbol) : \"|\" Layout . Symbol(0)",
    },
    Slot {
        display_name: "(\"|\" Symbol) : \"|\" Layout Symbol(0).",
    },
    Slot {
        display_name: "(\"|\" Symbol)+ : . (\"|\" Symbol)+ Layout (\"|\" Symbol)",
    },
    Slot {
        display_name: "(\"|\" Symbol)+ : (\"|\" Symbol)+ . Layout (\"|\" Symbol)",
    },
    Slot {
        display_name: "(\"|\" Symbol)+ : (\"|\" Symbol)+ Layout . (\"|\" Symbol)",
    },
    Slot {
        display_name: "(\"|\" Symbol)+ : (\"|\" Symbol)+ Layout (\"|\" Symbol).",
    },
    Slot {
        display_name: "(\"|\" Symbol)+ : . (\"|\" Symbol)",
    },
    Slot {
        display_name: "(\"|\" Symbol)+ : (\"|\" Symbol).",
    },
    Slot {
        display_name: "(\"\\\" Identifier) : . \"\\\" Layout Identifier",
    },
    Slot {
        display_name: "(\"\\\" Identifier) : \"\\\" . Layout Identifier",
    },
    Slot {
        display_name: "(\"\\\" Identifier) : \"\\\" Layout . Identifier",
    },
    Slot {
        display_name: "(\"\\\" Identifier) : \"\\\" Layout Identifier.",
    },
    Slot {
        display_name: "(\"\\\" Identifier)+ : . (\"\\\" Identifier)+ Layout (\"\\\" Identifier)",
    },
    Slot {
        display_name: "(\"\\\" Identifier)+ : (\"\\\" Identifier)+ . Layout (\"\\\" Identifier)",
    },
    Slot {
        display_name: "(\"\\\" Identifier)+ : (\"\\\" Identifier)+ Layout . (\"\\\" Identifier)",
    },
    Slot {
        display_name: "(\"\\\" Identifier)+ : (\"\\\" Identifier)+ Layout (\"\\\" Identifier).",
    },
    Slot {
        display_name: "(\"\\\" Identifier)+ : . (\"\\\" Identifier)",
    },
    Slot {
        display_name: "(\"\\\" Identifier)+ : (\"\\\" Identifier).",
    },
    Slot {
        display_name: "(\"!\" Identifier) : . \"!\" Layout Identifier",
    },
    Slot {
        display_name: "(\"!\" Identifier) : \"!\" . Layout Identifier",
    },
    Slot {
        display_name: "(\"!\" Identifier) : \"!\" Layout . Identifier",
    },
    Slot {
        display_name: "(\"!\" Identifier) : \"!\" Layout Identifier.",
    },
    Slot {
        display_name: "(\"!\" Identifier)+ : . (\"!\" Identifier)+ Layout (\"!\" Identifier)",
    },
    Slot {
        display_name: "(\"!\" Identifier)+ : (\"!\" Identifier)+ . Layout (\"!\" Identifier)",
    },
    Slot {
        display_name: "(\"!\" Identifier)+ : (\"!\" Identifier)+ Layout . (\"!\" Identifier)",
    },
    Slot {
        display_name: "(\"!\" Identifier)+ : (\"!\" Identifier)+ Layout (\"!\" Identifier).",
    },
    Slot {
        display_name: "(\"!\" Identifier)+ : . (\"!\" Identifier)",
    },
    Slot {
        display_name: "(\"!\" Identifier)+ : (\"!\" Identifier).",
    },
    Slot {
        display_name: "(\"|\" Regex) : . \"|\" Layout Regex",
    },
    Slot {
        display_name: "(\"|\" Regex) : \"|\" . Layout Regex",
    },
    Slot {
        display_name: "(\"|\" Regex) : \"|\" Layout . Regex",
    },
    Slot {
        display_name: "(\"|\" Regex) : \"|\" Layout Regex.",
    },
    Slot {
        display_name: "(\"|\" Regex)+ : . (\"|\" Regex)+ Layout (\"|\" Regex)",
    },
    Slot {
        display_name: "(\"|\" Regex)+ : (\"|\" Regex)+ . Layout (\"|\" Regex)",
    },
    Slot {
        display_name: "(\"|\" Regex)+ : (\"|\" Regex)+ Layout . (\"|\" Regex)",
    },
    Slot {
        display_name: "(\"|\" Regex)+ : (\"|\" Regex)+ Layout (\"|\" Regex).",
    },
    Slot {
        display_name: "(\"|\" Regex)+ : . (\"|\" Regex)",
    },
    Slot {
        display_name: "(\"|\" Regex)+ : (\"|\" Regex).",
    },
    Slot {
        display_name: "\"!\"? : . \"!\"",
    },
    Slot {
        display_name: "\"!\"? : \"!\".",
    },
    Slot {
        display_name: "\"!\"? : .",
    },
    Slot {
        display_name: "RangeElement+ : . RangeElement+ Layout RangeElement",
    },
    Slot {
        display_name: "RangeElement+ : RangeElement+ . Layout RangeElement",
    },
    Slot {
        display_name: "RangeElement+ : RangeElement+ Layout . RangeElement",
    },
    Slot {
        display_name: "RangeElement+ : RangeElement+ Layout RangeElement.",
    },
    Slot {
        display_name: "RangeElement+ : . RangeElement",
    },
    Slot {
        display_name: "RangeElement+ : RangeElement.",
    },
    Slot {
        display_name: "Symbol !Except : . Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : Identifier . return 0",
    },
    Slot {
        display_name: "Symbol !Except : Identifier return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . \"(\" Layout Symbol+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" . Layout Symbol+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout . Symbol+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout Symbol+ . Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout Symbol+ Layout . \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout Symbol+ Layout \")\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout Symbol+ Layout \")\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" . Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout . first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout first:Symbol(0) . Layout rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout first:Symbol(0) Layout . rest:(\"|\" Symbol)+ Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ . Layout \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout . \")\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"(\" Layout first:Symbol(0) Layout rest:(\"|\" Symbol)+ Layout \")\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . \"\"\" Layout String Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"\"\" . Layout String Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"\"\" Layout . String Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"\"\" Layout String . Layout \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"\"\" Layout String Layout . \"\"\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"\"\" Layout String Layout \"\"\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"\"\" Layout String Layout \"\"\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . \"}\" Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" . Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout . \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"*\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . \"}\" Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" . Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout . \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : \"{\" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout \"}\" Layout \"+\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"*\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"*\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"+\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"+\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"?\" return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" . return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"?\" return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" . Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout . Identifier return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier . return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout \"!>>\" Layout Identifier return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . labels:(\"!\" Identifier)+ return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ . return 0",
    },
    Slot {
        display_name: "Symbol !Except : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:(\"!\" Identifier)+ return 0.",
    },
    Slot {
        display_name: "Symbol !Except : . Identifier Layout \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol !Except : Identifier . Layout \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol !Except : Identifier Layout . \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol !Except : Identifier Layout \"!<<\" . Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol !Except : Identifier Layout \"!<<\" Layout . r=Symbol(2) return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol !Except : Identifier Layout \"!<<\" Layout r=Symbol(2) . return r == 0 ? 2 : min(r, 2)",
    },
    Slot {
        display_name: "Symbol !Except : Identifier Layout \"!<<\" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2).",
    },
    Slot {
        display_name: "Symbol !Except : . label:Identifier Layout \":\" Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol !Except : label:Identifier . Layout \":\" Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol !Except : label:Identifier Layout . \":\" Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol !Except : label:Identifier Layout \":\" . Layout Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol !Except : label:Identifier Layout \":\" Layout . Symbol(1) return 1",
    },
    Slot {
        display_name: "Symbol !Except : label:Identifier Layout \":\" Layout Symbol(1) . return 1",
    },
    Slot {
        display_name: "Symbol !Except : label:Identifier Layout \":\" Layout Symbol(1) return 1.",
    },
    Slot {
        display_name: "StartGrammar : . Layout start:Grammar Layout",
    },
    Slot {
        display_name: "StartGrammar : Layout . start:Grammar Layout",
    },
    Slot {
        display_name: "StartGrammar : Layout start:Grammar . Layout",
    },
    Slot {
        display_name: "StartGrammar : Layout start:Grammar Layout.",
    },
    Slot {
        display_name: "StartLayoutDef : . Layout start:LayoutDef Layout",
    },
    Slot {
        display_name: "StartLayoutDef : Layout . start:LayoutDef Layout",
    },
    Slot {
        display_name: "StartLayoutDef : Layout start:LayoutDef . Layout",
    },
    Slot {
        display_name: "StartLayoutDef : Layout start:LayoutDef Layout.",
    },
    Slot {
        display_name: "StartRule : . Layout start:Rule Layout",
    },
    Slot {
        display_name: "StartRule : Layout . start:Rule Layout",
    },
    Slot {
        display_name: "StartRule : Layout start:Rule . Layout",
    },
    Slot {
        display_name: "StartRule : Layout start:Rule Layout.",
    },
    Slot {
        display_name: "StartSyntaxRule : . Layout start:SyntaxRule Layout",
    },
    Slot {
        display_name: "StartSyntaxRule : Layout . start:SyntaxRule Layout",
    },
    Slot {
        display_name: "StartSyntaxRule : Layout start:SyntaxRule . Layout",
    },
    Slot {
        display_name: "StartSyntaxRule : Layout start:SyntaxRule Layout.",
    },
    Slot {
        display_name: "StartAnnotation : . Layout start:Annotation Layout",
    },
    Slot {
        display_name: "StartAnnotation : Layout . start:Annotation Layout",
    },
    Slot {
        display_name: "StartAnnotation : Layout start:Annotation . Layout",
    },
    Slot {
        display_name: "StartAnnotation : Layout start:Annotation Layout.",
    },
    Slot {
        display_name: "StartRegexRule : . Layout start:RegexRule Layout",
    },
    Slot {
        display_name: "StartRegexRule : Layout . start:RegexRule Layout",
    },
    Slot {
        display_name: "StartRegexRule : Layout start:RegexRule . Layout",
    },
    Slot {
        display_name: "StartRegexRule : Layout start:RegexRule Layout.",
    },
    Slot {
        display_name: "StartPreCondition : . Layout start:PreCondition Layout",
    },
    Slot {
        display_name: "StartPreCondition : Layout . start:PreCondition Layout",
    },
    Slot {
        display_name: "StartPreCondition : Layout start:PreCondition . Layout",
    },
    Slot {
        display_name: "StartPreCondition : Layout start:PreCondition Layout.",
    },
    Slot {
        display_name: "StartPostCondition : . Layout start:PostCondition Layout",
    },
    Slot {
        display_name: "StartPostCondition : Layout . start:PostCondition Layout",
    },
    Slot {
        display_name: "StartPostCondition : Layout start:PostCondition . Layout",
    },
    Slot {
        display_name: "StartPostCondition : Layout start:PostCondition Layout.",
    },
    Slot {
        display_name: "StartPriorityLevel : . Layout start:PriorityLevel Layout",
    },
    Slot {
        display_name: "StartPriorityLevel : Layout . start:PriorityLevel Layout",
    },
    Slot {
        display_name: "StartPriorityLevel : Layout start:PriorityLevel . Layout",
    },
    Slot {
        display_name: "StartPriorityLevel : Layout start:PriorityLevel Layout.",
    },
    Slot {
        display_name: "StartAssociativity : . Layout start:Associativity Layout",
    },
    Slot {
        display_name: "StartAssociativity : Layout . start:Associativity Layout",
    },
    Slot {
        display_name: "StartAssociativity : Layout start:Associativity . Layout",
    },
    Slot {
        display_name: "StartAssociativity : Layout start:Associativity Layout.",
    },
    Slot {
        display_name: "StartAlternative : . Layout start:Alternative Layout",
    },
    Slot {
        display_name: "StartAlternative : Layout . start:Alternative Layout",
    },
    Slot {
        display_name: "StartAlternative : Layout start:Alternative . Layout",
    },
    Slot {
        display_name: "StartAlternative : Layout start:Alternative Layout.",
    },
    Slot {
        display_name: "StartSymbol : . Layout start:Symbol(0) Layout",
    },
    Slot {
        display_name: "StartSymbol : Layout . start:Symbol(0) Layout",
    },
    Slot {
        display_name: "StartSymbol : Layout start:Symbol(0) . Layout",
    },
    Slot {
        display_name: "StartSymbol : Layout start:Symbol(0) Layout.",
    },
    Slot {
        display_name: "StartRegex : . Layout start:Regex Layout",
    },
    Slot {
        display_name: "StartRegex : Layout . start:Regex Layout",
    },
    Slot {
        display_name: "StartRegex : Layout start:Regex . Layout",
    },
    Slot {
        display_name: "StartRegex : Layout start:Regex Layout.",
    },
    Slot {
        display_name: "StartCharClass : . Layout start:CharClass Layout",
    },
    Slot {
        display_name: "StartCharClass : Layout . start:CharClass Layout",
    },
    Slot {
        display_name: "StartCharClass : Layout start:CharClass . Layout",
    },
    Slot {
        display_name: "StartCharClass : Layout start:CharClass Layout.",
    },
    Slot {
        display_name: "StartRangeElement : . Layout start:RangeElement Layout",
    },
    Slot {
        display_name: "StartRangeElement : Layout . start:RangeElement Layout",
    },
    Slot {
        display_name: "StartRangeElement : Layout start:RangeElement . Layout",
    },
    Slot {
        display_name: "StartRangeElement : Layout start:RangeElement Layout.",
    },
    Slot {
        display_name: "StartRange : . Layout start:Range Layout",
    },
    Slot {
        display_name: "StartRange : Layout . start:Range Layout",
    },
    Slot {
        display_name: "StartRange : Layout start:Range . Layout",
    },
    Slot {
        display_name: "StartRange : Layout start:Range Layout.",
    },
];
impl<'i> Parser<'i> for IggyParser<'i> {
    fn nonterminal_display_name(nonterminal_id: NonterminalId) -> &'static str {
        NONTERMINALS[nonterminal_id.index()].display
    }
    fn nonterminal_id(name: &str) -> Option<NonterminalId> {
        NONTERMINAL_IDS.get(name).copied()
    }
    fn terminal_name(terminal_id: TerminalId) -> &'static str {
        TERMINALS[terminal_id.index()].name
    }
    fn slot_name(slot_id: SlotId) -> &'static str {
        SLOTS[slot_id.index()].display_name
    }
    fn epsilon() -> TerminalId {
        TerminalId((TERMINALS.len() - 1) as u16)
    }
    fn execute(
        &mut self,
        input_index: u32,
        slot_id: SlotId,
        result: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        env: Option<EnvId>,
    ) {
        record!(
            self,
            ProcessingDescriptor,
            input_index,
            slot_id,
            result,
            gss_node_id
        );
        match slot_id {
            //Grammar : . "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0
            SlotId(0) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(7), i, j);
                        //Grammar : "grammar" . Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0
                        let next_slot_id = SlotId(1);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"grammar\"",
                            i,
                            SlotId(0),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" . Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0
            SlotId(1) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Grammar : "grammar" Layout . name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0
                        let next_slot_id = SlotId(2);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(1),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" Layout . name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0
            SlotId(2) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Grammar : "grammar" Layout name:Identifier . Layout Grammar_Opt_0 Layout Grammar_Star_0
                        let next_slot_id = SlotId(3);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(2),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" Layout name:Identifier . Layout Grammar_Opt_0 Layout Grammar_Star_0
            SlotId(3) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Grammar : "grammar" Layout name:Identifier Layout . Grammar_Opt_0 Layout Grammar_Star_0
                        let next_slot_id = SlotId(4);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(3),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" Layout name:Identifier Layout . Grammar_Opt_0 Layout Grammar_Star_0
            SlotId(4) => {
                self.create_grammar_opt_0(result, gss_node_id, SlotId(5));
            }
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 . Layout Grammar_Star_0
            SlotId(5) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout . Grammar_Star_0
                        let next_slot_id = SlotId(6);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(5),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout . Grammar_Star_0
            SlotId(6) => {
                self.create_grammar_star_0(result, gss_node_id, SlotId(7));
            }
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0.
            SlotId(7) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
                let end_slot_id = SlotId(7);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //LayoutDef : . "layout" Layout LayoutDef_Star_1
            SlotId(8) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"layout\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"layout\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //LayoutDef : "layout" . Layout LayoutDef_Star_1
                        let next_slot_id = SlotId(9);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"layout\"",
                            i,
                            SlotId(8),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef : "layout" . Layout LayoutDef_Star_1
            SlotId(9) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //LayoutDef : "layout" Layout . LayoutDef_Star_1
                        let next_slot_id = SlotId(10);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(9),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef : "layout" Layout . LayoutDef_Star_1
            SlotId(10) => {
                self.create_layout_def_star_1(result, gss_node_id, SlotId(11));
            }
            //LayoutDef : "layout" Layout LayoutDef_Star_1.
            SlotId(11) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
                let end_slot_id = SlotId(11);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Rule : . SyntaxRule
            SlotId(12) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(13));
            }
            //Rule : SyntaxRule.
            SlotId(13) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let end_slot_id = SlotId(13);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Rule : . RegexRule
            SlotId(14) => {
                self.create_regex_rule(result, gss_node_id, SlotId(15));
            }
            //Rule : RegexRule.
            SlotId(15) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let end_slot_id = SlotId(15);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule : . SyntaxRule_Opt_3 Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(16) => {
                self.create_syntax_rule_opt_3(result, gss_node_id, SlotId(17));
            }
            //SyntaxRule : SyntaxRule_Opt_3 . Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(17) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //SyntaxRule : SyntaxRule_Opt_3 Layout . head:Identifier Layout "=" Layout SyntaxRule_Star_2
                        let next_slot_id = SlotId(18);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(17),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : SyntaxRule_Opt_3 Layout . head:Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(18) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier . Layout "=" Layout SyntaxRule_Star_2
                        let next_slot_id = SlotId(19);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(18),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier . Layout "=" Layout SyntaxRule_Star_2
            SlotId(19) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier Layout . "=" Layout SyntaxRule_Star_2
                        let next_slot_id = SlotId(20);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(19),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier Layout . "=" Layout SyntaxRule_Star_2
            SlotId(20) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier Layout "=" . Layout SyntaxRule_Star_2
                        let next_slot_id = SlotId(21);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"=\"",
                            i,
                            SlotId(20),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier Layout "=" . Layout SyntaxRule_Star_2
            SlotId(21) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier Layout "=" Layout . SyntaxRule_Star_2
                        let next_slot_id = SlotId(22);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(21),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier Layout "=" Layout . SyntaxRule_Star_2
            SlotId(22) => {
                self.create_syntax_rule_star_2(result, gss_node_id, SlotId(23));
            }
            //SyntaxRule : SyntaxRule_Opt_3 Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2.
            SlotId(23) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let end_slot_id = SlotId(23);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Annotation : . "@NoLayout"
            SlotId(24) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"@NoLayout\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"@NoLayout\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Annotation : "@NoLayout".
                        let next_slot_id = SlotId(25);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"@NoLayout\"",
                            i,
                            SlotId(24),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@NoLayout".
            SlotId(25) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(25);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Annotation : . "@Layout" Layout "(" Layout Identifier Layout ")"
            SlotId(26) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"@Layout\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"@Layout\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Annotation : "@Layout" . Layout "(" Layout Identifier Layout ")"
                        let next_slot_id = SlotId(27);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"@Layout\"",
                            i,
                            SlotId(26),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" . Layout "(" Layout Identifier Layout ")"
            SlotId(27) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Annotation : "@Layout" Layout . "(" Layout Identifier Layout ")"
                        let next_slot_id = SlotId(28);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(27),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout . "(" Layout Identifier Layout ")"
            SlotId(28) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Annotation : "@Layout" Layout "(" . Layout Identifier Layout ")"
                        let next_slot_id = SlotId(29);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(28),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" . Layout Identifier Layout ")"
            SlotId(29) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Annotation : "@Layout" Layout "(" Layout . Identifier Layout ")"
                        let next_slot_id = SlotId(30);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(29),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" Layout . Identifier Layout ")"
            SlotId(30) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Annotation : "@Layout" Layout "(" Layout Identifier . Layout ")"
                        let next_slot_id = SlotId(31);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(30),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" Layout Identifier . Layout ")"
            SlotId(31) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Annotation : "@Layout" Layout "(" Layout Identifier Layout . ")"
                        let next_slot_id = SlotId(32);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(31),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" Layout Identifier Layout . ")"
            SlotId(32) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Annotation : "@Layout" Layout "(" Layout Identifier Layout ")".
                        let next_slot_id = SlotId(33);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(32),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" Layout Identifier Layout ")".
            SlotId(33) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(33);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule : . "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(34) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"@regex\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"@regex\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //RegexRule : "@regex" . Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
                        let next_slot_id = SlotId(35);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"@regex\"",
                            i,
                            SlotId(34),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" . Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(35) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule : "@regex" Layout . Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
                        let next_slot_id = SlotId(36);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(35),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" Layout . Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(36) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //RegexRule : "@regex" Layout Identifier . Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
                        let next_slot_id = SlotId(37);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(36),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" Layout Identifier . Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(37) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule : "@regex" Layout Identifier Layout . "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
                        let next_slot_id = SlotId(38);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(37),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" Layout Identifier Layout . "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(38) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //RegexRule : "@regex" Layout Identifier Layout "=" . Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
                        let next_slot_id = SlotId(39);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"=\"",
                            i,
                            SlotId(38),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" Layout Identifier Layout "=" . Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(39) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule : "@regex" Layout Identifier Layout "=" Layout . RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
                        let next_slot_id = SlotId(40);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(39),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" Layout Identifier Layout "=" Layout . RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(40) => {
                self.create_regex_rule_opt_5(result, gss_node_id, SlotId(41));
            }
            //RegexRule : "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 . Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(41) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule : "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout . body:RegexRule_Plus_3 Layout RegexRule_Star_3
                        let next_slot_id = SlotId(42);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(41),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout . body:RegexRule_Plus_3 Layout RegexRule_Star_3
            SlotId(42) => {
                self.create_regex_rule_plus_3(result, gss_node_id, SlotId(43));
            }
            //RegexRule : "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 . Layout RegexRule_Star_3
            SlotId(43) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule : "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout . RegexRule_Star_3
                        let next_slot_id = SlotId(44);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(43),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout . RegexRule_Star_3
            SlotId(44) => {
                self.create_regex_rule_star_3(result, gss_node_id, SlotId(45));
            }
            //RegexRule : "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3.
            SlotId(45) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(45);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PreCondition : . Identifier Layout "!<<"
            SlotId(46) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //PreCondition : Identifier . Layout "!<<"
                        let next_slot_id = SlotId(47);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(46),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PreCondition : Identifier . Layout "!<<"
            SlotId(47) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //PreCondition : Identifier Layout . "!<<"
                        let next_slot_id = SlotId(48);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(47),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PreCondition : Identifier Layout . "!<<"
            SlotId(48) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!<<\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!<<\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //PreCondition : Identifier Layout "!<<".
                        let next_slot_id = SlotId(49);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!<<\"",
                            i,
                            SlotId(48),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PreCondition : Identifier Layout "!<<".
            SlotId(49) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(49);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PostCondition : . "\" Layout Identifier
            SlotId(50) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\\\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\\\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //PostCondition : "\" . Layout Identifier
                        let next_slot_id = SlotId(51);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\\\"",
                            i,
                            SlotId(50),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "\" . Layout Identifier
            SlotId(51) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //PostCondition : "\" Layout . Identifier
                        let next_slot_id = SlotId(52);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(51),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "\" Layout . Identifier
            SlotId(52) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //PostCondition : "\" Layout Identifier.
                        let next_slot_id = SlotId(53);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(52),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "\" Layout Identifier.
            SlotId(53) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(53);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PostCondition : . "!>>" Layout Identifier
            SlotId(54) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!>>\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!>>\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //PostCondition : "!>>" . Layout Identifier
                        let next_slot_id = SlotId(55);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!>>\"",
                            i,
                            SlotId(54),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "!>>" . Layout Identifier
            SlotId(55) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //PostCondition : "!>>" Layout . Identifier
                        let next_slot_id = SlotId(56);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(55),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "!>>" Layout . Identifier
            SlotId(56) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //PostCondition : "!>>" Layout Identifier.
                        let next_slot_id = SlotId(57);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(56),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "!>>" Layout Identifier.
            SlotId(57) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(57);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel : . PriorityLevel_Opt_7 Layout PriorityLevel_Star_4
            SlotId(58) => {
                self.create_priority_level_opt_7(result, gss_node_id, SlotId(59));
            }
            //PriorityLevel : PriorityLevel_Opt_7 . Layout PriorityLevel_Star_4
            SlotId(59) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //PriorityLevel : PriorityLevel_Opt_7 Layout . PriorityLevel_Star_4
                        let next_slot_id = SlotId(60);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(59),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel : PriorityLevel_Opt_7 Layout . PriorityLevel_Star_4
            SlotId(60) => {
                self.create_priority_level_star_4(result, gss_node_id, SlotId(61));
            }
            //PriorityLevel : PriorityLevel_Opt_7 Layout PriorityLevel_Star_4.
            SlotId(61) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(61);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Associativity : . "left"
            SlotId(62) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"left\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"left\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //Associativity : "left".
                        let next_slot_id = SlotId(63);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"left\"",
                            i,
                            SlotId(62),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "left".
            SlotId(63) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(63);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Associativity : . "right"
            SlotId(64) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"right\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"right\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Associativity : "right".
                        let next_slot_id = SlotId(65);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"right\"",
                            i,
                            SlotId(64),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "right".
            SlotId(65) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(65);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Associativity : . "none"
            SlotId(66) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"none\"", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"none\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Associativity : "none".
                        let next_slot_id = SlotId(67);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"none\"",
                            i,
                            SlotId(66),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "none".
            SlotId(67) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(67);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative : . Alternative_Star_5 Layout Alternative_Opt_10
            SlotId(68) => {
                self.create_alternative_star_5(result, gss_node_id, SlotId(69));
            }
            //Alternative : Alternative_Star_5 . Layout Alternative_Opt_10
            SlotId(69) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Alternative : Alternative_Star_5 Layout . Alternative_Opt_10
                        let next_slot_id = SlotId(70);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(69),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative : Alternative_Star_5 Layout . Alternative_Opt_10
            SlotId(70) => {
                self.create_alternative_opt_10(result, gss_node_id, SlotId(71));
            }
            //Alternative : Alternative_Star_5 Layout Alternative_Opt_10.
            SlotId(71) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(71);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . Identifier return 0
            SlotId(72) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : Identifier . return 0
                        let next_slot_id = SlotId(73);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(72),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : Identifier . return 0
            SlotId(73) => {
                self.execute(input_index, SlotId(74), result, gss_node_id, env);
            }
            //Symbol(p: i32) : Identifier return 0.
            SlotId(74) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(74);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . "(" Layout Alternative_Plus_7 Layout ")" return 0
            SlotId(75) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol(p: i32) : "(" . Layout Alternative_Plus_7 Layout ")" return 0
                        let next_slot_id = SlotId(76);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(75),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" . Layout Alternative_Plus_7 Layout ")" return 0
            SlotId(76) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "(" Layout . Alternative_Plus_7 Layout ")" return 0
                        let next_slot_id = SlotId(77);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(76),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout . Alternative_Plus_7 Layout ")" return 0
            SlotId(77) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(78));
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 . Layout ")" return 0
            SlotId(78) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout . ")" return 0
                        let next_slot_id = SlotId(79);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(78),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout . ")" return 0
            SlotId(79) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" . return 0
                        let next_slot_id = SlotId(80);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(79),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" . return 0
            SlotId(80) => {
                self.execute(input_index, SlotId(81), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" return 0.
            SlotId(81) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(81);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(82) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(83);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(82),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(83) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(84);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(83),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(84) => {
                self.create_symbol(result, gss_node_id, SlotId(85), env, None, 0);
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) . Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(85) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(86);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(85),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(86) => {
                self.create_symbol_plus_8(result, gss_node_id, SlotId(87));
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 . Layout ")" return 0
            SlotId(87) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout . ")" return 0
                        let next_slot_id = SlotId(88);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(87),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout . ")" return 0
            SlotId(88) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" . return 0
                        let next_slot_id = SlotId(89);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(88),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" . return 0
            SlotId(89) => {
                self.execute(input_index, SlotId(90), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0.
            SlotId(90) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(90);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . """ Layout String Layout """ return 0
            SlotId(91) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Symbol(p: i32) : """ . Layout String Layout """ return 0
                        let next_slot_id = SlotId(92);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(91),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ . Layout String Layout """ return 0
            SlotId(92) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : """ Layout . String Layout """ return 0
                        let next_slot_id = SlotId(93);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(92),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout . String Layout """ return 0
            SlotId(93) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Symbol(p: i32) : """ Layout String . Layout """ return 0
                        let next_slot_id = SlotId(94);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "String",
                            i,
                            SlotId(93),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout String . Layout """ return 0
            SlotId(94) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : """ Layout String Layout . """ return 0
                        let next_slot_id = SlotId(95);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(94),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout String Layout . """ return 0
            SlotId(95) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Symbol(p: i32) : """ Layout String Layout """ . return 0
                        let next_slot_id = SlotId(96);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(95),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout String Layout """ . return 0
            SlotId(96) => {
                self.execute(input_index, SlotId(97), result, gss_node_id, env);
            }
            //Symbol(p: i32) : """ Layout String Layout """ return 0.
            SlotId(97) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(97);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(98) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(99);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(98),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(99) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(100);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(99),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(100) => {
                self.create_symbol(result, gss_node_id, SlotId(101), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(101) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(102);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(101),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(102) => {
                self.create_symbol(result, gss_node_id, SlotId(103), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "*" return 0
            SlotId(103) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
                        let next_slot_id = SlotId(104);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(103),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
            SlotId(104) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
                        let next_slot_id = SlotId(105);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(104),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
            SlotId(105) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
                        let next_slot_id = SlotId(106);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(105),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
            SlotId(106) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
                        let next_slot_id = SlotId(107);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(106),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
            SlotId(107) => {
                self.execute(input_index, SlotId(108), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0.
            SlotId(108) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(108);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(109) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(110);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(109),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(110) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(111);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(110),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(111) => {
                self.create_symbol(result, gss_node_id, SlotId(112), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(112) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(113);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(112),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(113) => {
                self.create_symbol(result, gss_node_id, SlotId(114), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "+" return 0
            SlotId(114) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
                        let next_slot_id = SlotId(115);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(114),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
            SlotId(115) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
                        let next_slot_id = SlotId(116);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(115),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
            SlotId(116) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
                        let next_slot_id = SlotId(117);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(116),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
            SlotId(117) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
                        let next_slot_id = SlotId(118);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(117),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
            SlotId(118) => {
                self.execute(input_index, SlotId(119), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0.
            SlotId(119) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(119);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0
            SlotId(120) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(121), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0
            SlotId(121) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(122),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "*" return 0
            SlotId(122) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(123), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "*" return 0
            SlotId(123) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "*" return 0
                        let next_slot_id = SlotId(124);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(123),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "*" return 0
            SlotId(124) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" . return 0
                        let next_slot_id = SlotId(125);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(124),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" . return 0
            SlotId(125) => {
                self.execute(input_index, SlotId(126), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0.
            SlotId(126) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(126);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0
            SlotId(127) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(128), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0
            SlotId(128) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(129),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "+" return 0
            SlotId(129) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(130), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "+" return 0
            SlotId(130) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "+" return 0
                        let next_slot_id = SlotId(131);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(130),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "+" return 0
            SlotId(131) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" . return 0
                        let next_slot_id = SlotId(132);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(131),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" . return 0
            SlotId(132) => {
                self.execute(input_index, SlotId(133), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0.
            SlotId(133) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(133);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0
            SlotId(134) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(135), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0
            SlotId(135) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(136),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "?" return 0
            SlotId(136) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(137), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "?" return 0
            SlotId(137) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "?" return 0
                        let next_slot_id = SlotId(138);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(137),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "?" return 0
            SlotId(138) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(28), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(28), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" . return 0
                        let next_slot_id = SlotId(139);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"?\"",
                            i,
                            SlotId(138),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" . return 0
            SlotId(139) => {
                self.execute(input_index, SlotId(140), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0.
            SlotId(140) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(140);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout excepts:Symbol_Plus_9 return 0
            SlotId(141) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(142), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] . l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout excepts:Symbol_Plus_9 return 0
            SlotId(142) => {
                self.create_symbol_except_except(
                    result,
                    gss_node_id,
                    SlotId(143),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol_except_Except(p) . [l == 0 || l >= 3] Layout excepts:Symbol_Plus_9 return 0
            SlotId(143) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(144), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] . Layout excepts:Symbol_Plus_9 return 0
            SlotId(144) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout . excepts:Symbol_Plus_9 return 0
                        let next_slot_id = SlotId(145);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(144),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout . excepts:Symbol_Plus_9 return 0
            SlotId(145) => {
                self.create_symbol_plus_9(result, gss_node_id, SlotId(146));
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout excepts:Symbol_Plus_9 . return 0
            SlotId(146) => {
                self.execute(input_index, SlotId(147), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout excepts:Symbol_Plus_9 return 0.
            SlotId(147) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(147);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
            SlotId(148) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(149), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
            SlotId(149) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(150),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
            SlotId(150) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(151), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "!>>" Layout Identifier return 0
            SlotId(151) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "!>>" Layout Identifier return 0
                        let next_slot_id = SlotId(152);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(151),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "!>>" Layout Identifier return 0
            SlotId(152) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!>>\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!>>\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" . Layout Identifier return 0
                        let next_slot_id = SlotId(153);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!>>\"",
                            i,
                            SlotId(152),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" . Layout Identifier return 0
            SlotId(153) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout . Identifier return 0
                        let next_slot_id = SlotId(154);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(153),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout . Identifier return 0
            SlotId(154) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier . return 0
                        let next_slot_id = SlotId(155);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(154),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier . return 0
            SlotId(155) => {
                self.execute(input_index, SlotId(156), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0.
            SlotId(156) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(156);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
            SlotId(157) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(158), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
            SlotId(158) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(159),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
            SlotId(159) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(160), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout labels:Symbol_Plus_10 return 0
            SlotId(160) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . labels:Symbol_Plus_10 return 0
                        let next_slot_id = SlotId(161);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(160),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . labels:Symbol_Plus_10 return 0
            SlotId(161) => {
                self.create_symbol_plus_10(result, gss_node_id, SlotId(162));
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 . return 0
            SlotId(162) => {
                self.execute(input_index, SlotId(163), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0.
            SlotId(163) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(163);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(164) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : Identifier . Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(165);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(164),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : Identifier . Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(165) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : Identifier Layout . "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(166);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(165),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : Identifier Layout . "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(166) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!<<\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!<<\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol(p: i32) : Identifier Layout "!<<" . Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(167);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!<<\"",
                            i,
                            SlotId(166),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : Identifier Layout "!<<" . Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(167) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : Identifier Layout "!<<" Layout . r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(168);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(167),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : Identifier Layout "!<<" Layout . r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(168) => {
                self.create_symbol(result, gss_node_id, SlotId(169), env, Some("r"), 2);
            }
            //Symbol(p: i32) : Identifier Layout "!<<" Layout r=Symbol(2) . return r == 0 ? 2 : min(r, 2)
            SlotId(169) => {
                self.execute(input_index, SlotId(170), result, gss_node_id, env);
            }
            //Symbol(p: i32) : Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2).
            SlotId(170) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(170);
                let return_value = if self.lookup("r", env.unwrap()) == 0 {
                    2
                } else {
                    std::cmp::min(self.lookup("r", env.unwrap()), 2)
                };
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
            SlotId(171) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
                        let next_slot_id = SlotId(172);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(171),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
            SlotId(172) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
                        let next_slot_id = SlotId(173);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(172),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
            SlotId(173) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\":\"", i);
                match self.scanner.match_token(TerminalId(30), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\":\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(30), i, j);
                        //Symbol(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
                        let next_slot_id = SlotId(174);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\":\"",
                            i,
                            SlotId(173),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
            SlotId(174) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
                        let next_slot_id = SlotId(175);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(174),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
            SlotId(175) => {
                self.create_symbol(result, gss_node_id, SlotId(176), env, None, 1);
            }
            //Symbol(p: i32) : label:Identifier Layout ":" Layout Symbol(1) . return 1
            SlotId(176) => {
                self.execute(input_index, SlotId(177), result, gss_node_id, env);
            }
            //Symbol(p: i32) : label:Identifier Layout ":" Layout Symbol(1) return 1.
            SlotId(177) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(66);
                let end_slot_id = SlotId(177);
                let return_value = 1;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . Regex Layout "+"
            SlotId(178) => {
                self.create_regex(result, gss_node_id, SlotId(179));
            }
            //Regex : Regex . Layout "+"
            SlotId(179) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : Regex Layout . "+"
                        let next_slot_id = SlotId(180);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(179),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout . "+"
            SlotId(180) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Regex : Regex Layout "+".
                        let next_slot_id = SlotId(181);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(180),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "+".
            SlotId(181) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(181);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . Regex Layout "*"
            SlotId(182) => {
                self.create_regex(result, gss_node_id, SlotId(183));
            }
            //Regex : Regex . Layout "*"
            SlotId(183) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : Regex Layout . "*"
                        let next_slot_id = SlotId(184);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(183),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout . "*"
            SlotId(184) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Regex : Regex Layout "*".
                        let next_slot_id = SlotId(185);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(184),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "*".
            SlotId(185) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(185);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . Regex Layout "?"
            SlotId(186) => {
                self.create_regex(result, gss_node_id, SlotId(187));
            }
            //Regex : Regex . Layout "?"
            SlotId(187) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : Regex Layout . "?"
                        let next_slot_id = SlotId(188);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(187),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout . "?"
            SlotId(188) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(28), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(28), i, j);
                        //Regex : Regex Layout "?".
                        let next_slot_id = SlotId(189);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"?\"",
                            i,
                            SlotId(188),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "?".
            SlotId(189) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(189);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . "(" Layout first:Regex Layout rest:Regex_Plus_11 Layout ")"
            SlotId(190) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Regex : "(" . Layout first:Regex Layout rest:Regex_Plus_11 Layout ")"
                        let next_slot_id = SlotId(191);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(190),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout first:Regex Layout rest:Regex_Plus_11 Layout ")"
            SlotId(191) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : "(" Layout . first:Regex Layout rest:Regex_Plus_11 Layout ")"
                        let next_slot_id = SlotId(192);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(191),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout . first:Regex Layout rest:Regex_Plus_11 Layout ")"
            SlotId(192) => {
                self.create_regex(result, gss_node_id, SlotId(193));
            }
            //Regex : "(" Layout first:Regex . Layout rest:Regex_Plus_11 Layout ")"
            SlotId(193) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : "(" Layout first:Regex Layout . rest:Regex_Plus_11 Layout ")"
                        let next_slot_id = SlotId(194);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(193),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout first:Regex Layout . rest:Regex_Plus_11 Layout ")"
            SlotId(194) => {
                self.create_regex_plus_11(result, gss_node_id, SlotId(195));
            }
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_11 . Layout ")"
            SlotId(195) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_11 Layout . ")"
                        let next_slot_id = SlotId(196);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(195),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_11 Layout . ")"
            SlotId(196) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_11 Layout ")".
                        let next_slot_id = SlotId(197);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(196),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_11 Layout ")".
            SlotId(197) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(197);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . "(" Layout RegexRule_Plus_4 Layout ")"
            SlotId(198) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Regex : "(" . Layout RegexRule_Plus_4 Layout ")"
                        let next_slot_id = SlotId(199);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(198),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout RegexRule_Plus_4 Layout ")"
            SlotId(199) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : "(" Layout . RegexRule_Plus_4 Layout ")"
                        let next_slot_id = SlotId(200);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(199),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout . RegexRule_Plus_4 Layout ")"
            SlotId(200) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(201));
            }
            //Regex : "(" Layout RegexRule_Plus_4 . Layout ")"
            SlotId(201) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : "(" Layout RegexRule_Plus_4 Layout . ")"
                        let next_slot_id = SlotId(202);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(201),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_4 Layout . ")"
            SlotId(202) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Regex : "(" Layout RegexRule_Plus_4 Layout ")".
                        let next_slot_id = SlotId(203);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(202),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_4 Layout ")".
            SlotId(203) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(203);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . CharClass
            SlotId(204) => {
                self.create_char_class(result, gss_node_id, SlotId(205));
            }
            //Regex : CharClass.
            SlotId(205) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(205);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . "'" Layout Char Layout "'"
            SlotId(206) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"'\"", i);
                match self.scanner.match_token(TerminalId(31), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"'\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(31), i, j);
                        //Regex : "'" . Layout Char Layout "'"
                        let next_slot_id = SlotId(207);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"'\"",
                            i,
                            SlotId(206),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" . Layout Char Layout "'"
            SlotId(207) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : "'" Layout . Char Layout "'"
                        let next_slot_id = SlotId(208);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(207),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout . Char Layout "'"
            SlotId(208) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Regex : "'" Layout Char . Layout "'"
                        let next_slot_id = SlotId(209);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(208),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout Char . Layout "'"
            SlotId(209) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : "'" Layout Char Layout . "'"
                        let next_slot_id = SlotId(210);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(209),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout Char Layout . "'"
            SlotId(210) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"'\"", i);
                match self.scanner.match_token(TerminalId(31), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"'\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(31), i, j);
                        //Regex : "'" Layout Char Layout "'".
                        let next_slot_id = SlotId(211);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"'\"",
                            i,
                            SlotId(210),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout Char Layout "'".
            SlotId(211) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(211);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . """ Layout String Layout """
            SlotId(212) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Regex : """ . Layout String Layout """
                        let next_slot_id = SlotId(213);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(212),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ . Layout String Layout """
            SlotId(213) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : """ Layout . String Layout """
                        let next_slot_id = SlotId(214);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(213),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout . String Layout """
            SlotId(214) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Regex : """ Layout String . Layout """
                        let next_slot_id = SlotId(215);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "String",
                            i,
                            SlotId(214),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String . Layout """
            SlotId(215) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex : """ Layout String Layout . """
                        let next_slot_id = SlotId(216);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(215),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String Layout . """
            SlotId(216) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Regex : """ Layout String Layout """.
                        let next_slot_id = SlotId(217);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(216),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String Layout """.
            SlotId(217) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(217);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex : . Identifier
            SlotId(218) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Regex : Identifier.
                        let next_slot_id = SlotId(219);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(218),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Identifier.
            SlotId(219) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(219);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //CharClass : . neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 Layout "]"
            SlotId(220) => {
                self.create_char_class_opt_11(result, gss_node_id, SlotId(221));
            }
            //CharClass : neg:CharClass_Opt_11 . Layout "[" Layout CharClass_Plus_12 Layout "]"
            SlotId(221) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout . "[" Layout CharClass_Plus_12 Layout "]"
                        let next_slot_id = SlotId(222);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(221),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_11 Layout . "[" Layout CharClass_Plus_12 Layout "]"
            SlotId(222) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"[\"", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"[\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" . Layout CharClass_Plus_12 Layout "]"
                        let next_slot_id = SlotId(223);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"[\"",
                            i,
                            SlotId(222),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" . Layout CharClass_Plus_12 Layout "]"
            SlotId(223) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" Layout . CharClass_Plus_12 Layout "]"
                        let next_slot_id = SlotId(224);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(223),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout . CharClass_Plus_12 Layout "]"
            SlotId(224) => {
                self.create_char_class_plus_12(result, gss_node_id, SlotId(225));
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 . Layout "]"
            SlotId(225) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 Layout . "]"
                        let next_slot_id = SlotId(226);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(225),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 Layout . "]"
            SlotId(226) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"]\"", i);
                match self.scanner.match_token(TerminalId(33), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"]\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(33), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 Layout "]".
                        let next_slot_id = SlotId(227);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"]\"",
                            i,
                            SlotId(226),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 Layout "]".
            SlotId(227) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
                let end_slot_id = SlotId(227);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RangeElement : . Range
            SlotId(228) => {
                self.create_range(result, gss_node_id, SlotId(229));
            }
            //RangeElement : Range.
            SlotId(229) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
                let end_slot_id = SlotId(229);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RangeElement : . RangeChar
            SlotId(230) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //RangeElement : RangeChar.
                        let next_slot_id = SlotId(231);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(230),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RangeElement : RangeChar.
            SlotId(231) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
                let end_slot_id = SlotId(231);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Range : . start:RangeChar Layout "-" Layout end:RangeChar
            SlotId(232) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Range : start:RangeChar . Layout "-" Layout end:RangeChar
                        let next_slot_id = SlotId(233);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(232),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar . Layout "-" Layout end:RangeChar
            SlotId(233) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Range : start:RangeChar Layout . "-" Layout end:RangeChar
                        let next_slot_id = SlotId(234);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(233),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout . "-" Layout end:RangeChar
            SlotId(234) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"-\"", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"-\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Range : start:RangeChar Layout "-" . Layout end:RangeChar
                        let next_slot_id = SlotId(235);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"-\"",
                            i,
                            SlotId(234),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" . Layout end:RangeChar
            SlotId(235) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Range : start:RangeChar Layout "-" Layout . end:RangeChar
                        let next_slot_id = SlotId(236);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(235),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" Layout . end:RangeChar
            SlotId(236) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Range : start:RangeChar Layout "-" Layout end:RangeChar.
                        let next_slot_id = SlotId(237);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(236),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" Layout end:RangeChar.
            SlotId(237) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(237);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Grammar_Opt_0 : . LayoutDef
            SlotId(238) => {
                self.create_layout_def(result, gss_node_id, SlotId(239));
            }
            //Grammar_Opt_0 : LayoutDef.
            SlotId(239) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
                let end_slot_id = SlotId(239);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Grammar_Opt_0 : .
            SlotId(240) => {
                let end_slot_id = SlotId(240);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(15);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Grammar_Plus_0 : . Grammar_Plus_0 Layout Rule
            SlotId(241) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(242));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 . Layout Rule
            SlotId(242) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Grammar_Plus_0 : Grammar_Plus_0 Layout . Rule
                        let next_slot_id = SlotId(243);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(242),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout . Rule
            SlotId(243) => {
                self.create_rule(result, gss_node_id, SlotId(244));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout Rule.
            SlotId(244) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
                let end_slot_id = SlotId(244);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Grammar_Plus_0 : . Rule
            SlotId(245) => {
                self.create_rule(result, gss_node_id, SlotId(246));
            }
            //Grammar_Plus_0 : Rule.
            SlotId(246) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
                let end_slot_id = SlotId(246);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Grammar_Opt_1 : . Grammar_Plus_0
            SlotId(247) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(248));
            }
            //Grammar_Opt_1 : Grammar_Plus_0.
            SlotId(248) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(248);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Grammar_Opt_1 : .
            SlotId(249) => {
                let end_slot_id = SlotId(249);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(17);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Grammar_Star_0 : . Grammar_Opt_1
            SlotId(250) => {
                self.create_grammar_opt_1(result, gss_node_id, SlotId(251));
            }
            //Grammar_Star_0 : Grammar_Opt_1.
            SlotId(251) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
                let end_slot_id = SlotId(251);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //LayoutDef_Plus_1 : . LayoutDef_Plus_1 Layout Identifier
            SlotId(252) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(253));
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 . Layout Identifier
            SlotId(253) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
                        let next_slot_id = SlotId(254);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(253),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
            SlotId(254) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
                        let next_slot_id = SlotId(255);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(254),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
            SlotId(255) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(255);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //LayoutDef_Plus_1 : . Identifier
            SlotId(256) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //LayoutDef_Plus_1 : Identifier.
                        let next_slot_id = SlotId(257);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(256),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : Identifier.
            SlotId(257) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(257);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //LayoutDef_Opt_2 : . LayoutDef_Plus_1
            SlotId(258) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(259));
            }
            //LayoutDef_Opt_2 : LayoutDef_Plus_1.
            SlotId(259) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
                let end_slot_id = SlotId(259);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //LayoutDef_Opt_2 : .
            SlotId(260) => {
                let end_slot_id = SlotId(260);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(20);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //LayoutDef_Star_1 : . LayoutDef_Opt_2
            SlotId(261) => {
                self.create_layout_def_opt_2(result, gss_node_id, SlotId(262));
            }
            //LayoutDef_Star_1 : LayoutDef_Opt_2.
            SlotId(262) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
                let end_slot_id = SlotId(262);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule_Opt_3 : . Annotation
            SlotId(263) => {
                self.create_annotation(result, gss_node_id, SlotId(264));
            }
            //SyntaxRule_Opt_3 : Annotation.
            SlotId(264) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
                let end_slot_id = SlotId(264);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule_Opt_3 : .
            SlotId(265) => {
                let end_slot_id = SlotId(265);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(22);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule_Plus_2 : . SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel
            SlotId(266) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(267));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 . Layout ">" Layout PriorityLevel
            SlotId(267) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
                        let next_slot_id = SlotId(268);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(267),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
            SlotId(268) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
                        let next_slot_id = SlotId(269);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\">\"",
                            i,
                            SlotId(268),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
            SlotId(269) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
                        let next_slot_id = SlotId(270);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(269),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
            SlotId(270) => {
                self.create_priority_level(result, gss_node_id, SlotId(271));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel.
            SlotId(271) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(271);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule_Plus_2 : . PriorityLevel
            SlotId(272) => {
                self.create_priority_level(result, gss_node_id, SlotId(273));
            }
            //SyntaxRule_Plus_2 : PriorityLevel.
            SlotId(273) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(273);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule_Opt_4 : . SyntaxRule_Plus_2
            SlotId(274) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(275));
            }
            //SyntaxRule_Opt_4 : SyntaxRule_Plus_2.
            SlotId(275) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
                let end_slot_id = SlotId(275);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule_Opt_4 : .
            SlotId(276) => {
                let end_slot_id = SlotId(276);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(24);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule_Star_2 : . SyntaxRule_Opt_4
            SlotId(277) => {
                self.create_syntax_rule_opt_4(result, gss_node_id, SlotId(278));
            }
            //SyntaxRule_Star_2 : SyntaxRule_Opt_4.
            SlotId(278) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(278);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Opt_5 : . PreCondition
            SlotId(279) => {
                self.create_pre_condition(result, gss_node_id, SlotId(280));
            }
            //RegexRule_Opt_5 : PreCondition.
            SlotId(280) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
                let end_slot_id = SlotId(280);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Opt_5 : .
            SlotId(281) => {
                let end_slot_id = SlotId(281);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(26);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout Regex
            SlotId(282) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(283));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 . Layout Regex
            SlotId(283) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . Regex
                        let next_slot_id = SlotId(284);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(283),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . Regex
            SlotId(284) => {
                self.create_regex(result, gss_node_id, SlotId(285));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout Regex.
            SlotId(285) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
                let end_slot_id = SlotId(285);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Plus_4 : . Regex
            SlotId(286) => {
                self.create_regex(result, gss_node_id, SlotId(287));
            }
            //RegexRule_Plus_4 : Regex.
            SlotId(287) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
                let end_slot_id = SlotId(287);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Plus_3 : . RegexRule_Plus_3 Layout "|" Layout RegexRule_Plus_4
            SlotId(288) => {
                self.create_regex_rule_plus_3(result, gss_node_id, SlotId(289));
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 . Layout "|" Layout RegexRule_Plus_4
            SlotId(289) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule_Plus_3 : RegexRule_Plus_3 Layout . "|" Layout RegexRule_Plus_4
                        let next_slot_id = SlotId(290);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(289),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout . "|" Layout RegexRule_Plus_4
            SlotId(290) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" . Layout RegexRule_Plus_4
                        let next_slot_id = SlotId(291);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(290),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" . Layout RegexRule_Plus_4
            SlotId(291) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" Layout . RegexRule_Plus_4
                        let next_slot_id = SlotId(292);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(291),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" Layout . RegexRule_Plus_4
            SlotId(292) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(293));
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" Layout RegexRule_Plus_4.
            SlotId(293) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
                let end_slot_id = SlotId(293);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Plus_3 : . RegexRule_Plus_4
            SlotId(294) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(295));
            }
            //RegexRule_Plus_3 : RegexRule_Plus_4.
            SlotId(295) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
                let end_slot_id = SlotId(295);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout PostCondition
            SlotId(296) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(297));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 . Layout PostCondition
            SlotId(297) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . PostCondition
                        let next_slot_id = SlotId(298);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(297),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . PostCondition
            SlotId(298) => {
                self.create_post_condition(result, gss_node_id, SlotId(299));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout PostCondition.
            SlotId(299) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
                let end_slot_id = SlotId(299);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Plus_5 : . PostCondition
            SlotId(300) => {
                self.create_post_condition(result, gss_node_id, SlotId(301));
            }
            //RegexRule_Plus_5 : PostCondition.
            SlotId(301) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
                let end_slot_id = SlotId(301);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Opt_6 : . RegexRule_Plus_5
            SlotId(302) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(303));
            }
            //RegexRule_Opt_6 : RegexRule_Plus_5.
            SlotId(303) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(30);
                let end_slot_id = SlotId(303);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Opt_6 : .
            SlotId(304) => {
                let end_slot_id = SlotId(304);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(30);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule_Star_3 : . RegexRule_Opt_6
            SlotId(305) => {
                self.create_regex_rule_opt_6(result, gss_node_id, SlotId(306));
            }
            //RegexRule_Star_3 : RegexRule_Opt_6.
            SlotId(306) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
                let end_slot_id = SlotId(306);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel_Opt_7 : . Associativity
            SlotId(307) => {
                self.create_associativity(result, gss_node_id, SlotId(308));
            }
            //PriorityLevel_Opt_7 : Associativity.
            SlotId(308) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(32);
                let end_slot_id = SlotId(308);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel_Opt_7 : .
            SlotId(309) => {
                let end_slot_id = SlotId(309);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(32);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel_Plus_6 : . PriorityLevel_Plus_6 Layout "|" Layout Alternative
            SlotId(310) => {
                self.create_priority_level_plus_6(result, gss_node_id, SlotId(311));
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 . Layout "|" Layout Alternative
            SlotId(311) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout . "|" Layout Alternative
                        let next_slot_id = SlotId(312);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(311),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout . "|" Layout Alternative
            SlotId(312) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" . Layout Alternative
                        let next_slot_id = SlotId(313);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(312),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" . Layout Alternative
            SlotId(313) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout . Alternative
                        let next_slot_id = SlotId(314);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(313),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout . Alternative
            SlotId(314) => {
                self.create_alternative(result, gss_node_id, SlotId(315));
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout Alternative.
            SlotId(315) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
                let end_slot_id = SlotId(315);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel_Plus_6 : . Alternative
            SlotId(316) => {
                self.create_alternative(result, gss_node_id, SlotId(317));
            }
            //PriorityLevel_Plus_6 : Alternative.
            SlotId(317) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
                let end_slot_id = SlotId(317);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel_Opt_8 : . PriorityLevel_Plus_6
            SlotId(318) => {
                self.create_priority_level_plus_6(result, gss_node_id, SlotId(319));
            }
            //PriorityLevel_Opt_8 : PriorityLevel_Plus_6.
            SlotId(319) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
                let end_slot_id = SlotId(319);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel_Opt_8 : .
            SlotId(320) => {
                let end_slot_id = SlotId(320);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(34);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PriorityLevel_Star_4 : . PriorityLevel_Opt_8
            SlotId(321) => {
                self.create_priority_level_opt_8(result, gss_node_id, SlotId(322));
            }
            //PriorityLevel_Star_4 : PriorityLevel_Opt_8.
            SlotId(322) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(35);
                let end_slot_id = SlotId(322);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative_Plus_7 : . Alternative_Plus_7 Layout Symbol(0)
            SlotId(323) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(324));
            }
            //Alternative_Plus_7 : Alternative_Plus_7 . Layout Symbol(0)
            SlotId(324) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Alternative_Plus_7 : Alternative_Plus_7 Layout . Symbol(0)
                        let next_slot_id = SlotId(325);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(324),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Plus_7 : Alternative_Plus_7 Layout . Symbol(0)
            SlotId(325) => {
                self.create_symbol(result, gss_node_id, SlotId(326), env, None, 0);
            }
            //Alternative_Plus_7 : Alternative_Plus_7 Layout Symbol(0).
            SlotId(326) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(36);
                let end_slot_id = SlotId(326);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative_Plus_7 : . Symbol(0)
            SlotId(327) => {
                self.create_symbol(result, gss_node_id, SlotId(328), env, None, 0);
            }
            //Alternative_Plus_7 : Symbol(0).
            SlotId(328) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(36);
                let end_slot_id = SlotId(328);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative_Opt_9 : . Alternative_Plus_7
            SlotId(329) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(330));
            }
            //Alternative_Opt_9 : Alternative_Plus_7.
            SlotId(330) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(37);
                let end_slot_id = SlotId(330);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative_Opt_9 : .
            SlotId(331) => {
                let end_slot_id = SlotId(331);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(37);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative_Star_5 : . Alternative_Opt_9
            SlotId(332) => {
                self.create_alternative_opt_9(result, gss_node_id, SlotId(333));
            }
            //Alternative_Star_5 : Alternative_Opt_9.
            SlotId(333) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(38);
                let end_slot_id = SlotId(333);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative_Opt_10 : . Label
            SlotId(334) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Label", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Label", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(5), i, j);
                        //Alternative_Opt_10 : Label.
                        let next_slot_id = SlotId(335);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Label",
                            i,
                            SlotId(334),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Opt_10 : Label.
            SlotId(335) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(39);
                let end_slot_id = SlotId(335);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Alternative_Opt_10 : .
            SlotId(336) => {
                let end_slot_id = SlotId(336);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(39);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Group_0 : . "|" Layout Symbol(0)
            SlotId(337) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Symbol_Group_0 : "|" . Layout Symbol(0)
                        let next_slot_id = SlotId(338);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(337),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" . Layout Symbol(0)
            SlotId(338) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_Group_0 : "|" Layout . Symbol(0)
                        let next_slot_id = SlotId(339);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(338),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" Layout . Symbol(0)
            SlotId(339) => {
                self.create_symbol(result, gss_node_id, SlotId(340), env, None, 0);
            }
            //Symbol_Group_0 : "|" Layout Symbol(0).
            SlotId(340) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(40);
                let end_slot_id = SlotId(340);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Plus_8 : . Symbol_Plus_8 Layout Symbol_Group_0
            SlotId(341) => {
                self.create_symbol_plus_8(result, gss_node_id, SlotId(342));
            }
            //Symbol_Plus_8 : Symbol_Plus_8 . Layout Symbol_Group_0
            SlotId(342) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_Plus_8 : Symbol_Plus_8 Layout . Symbol_Group_0
                        let next_slot_id = SlotId(343);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(342),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Plus_8 : Symbol_Plus_8 Layout . Symbol_Group_0
            SlotId(343) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(344));
            }
            //Symbol_Plus_8 : Symbol_Plus_8 Layout Symbol_Group_0.
            SlotId(344) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(41);
                let end_slot_id = SlotId(344);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Plus_8 : . Symbol_Group_0
            SlotId(345) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(346));
            }
            //Symbol_Plus_8 : Symbol_Group_0.
            SlotId(346) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(41);
                let end_slot_id = SlotId(346);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Group_1 : . "\" Layout Identifier
            SlotId(347) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\\\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\\\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Symbol_Group_1 : "\" . Layout Identifier
                        let next_slot_id = SlotId(348);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\\\"",
                            i,
                            SlotId(347),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_1 : "\" . Layout Identifier
            SlotId(348) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_Group_1 : "\" Layout . Identifier
                        let next_slot_id = SlotId(349);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(348),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_1 : "\" Layout . Identifier
            SlotId(349) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol_Group_1 : "\" Layout Identifier.
                        let next_slot_id = SlotId(350);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(349),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_1 : "\" Layout Identifier.
            SlotId(350) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(42);
                let end_slot_id = SlotId(350);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Plus_9 : . Symbol_Plus_9 Layout Symbol_Group_1
            SlotId(351) => {
                self.create_symbol_plus_9(result, gss_node_id, SlotId(352));
            }
            //Symbol_Plus_9 : Symbol_Plus_9 . Layout Symbol_Group_1
            SlotId(352) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_Plus_9 : Symbol_Plus_9 Layout . Symbol_Group_1
                        let next_slot_id = SlotId(353);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(352),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Plus_9 : Symbol_Plus_9 Layout . Symbol_Group_1
            SlotId(353) => {
                self.create_symbol_group_1(result, gss_node_id, SlotId(354));
            }
            //Symbol_Plus_9 : Symbol_Plus_9 Layout Symbol_Group_1.
            SlotId(354) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
                let end_slot_id = SlotId(354);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Plus_9 : . Symbol_Group_1
            SlotId(355) => {
                self.create_symbol_group_1(result, gss_node_id, SlotId(356));
            }
            //Symbol_Plus_9 : Symbol_Group_1.
            SlotId(356) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
                let end_slot_id = SlotId(356);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Group_2 : . "!" Layout Identifier
            SlotId(357) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(29), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(29), i, j);
                        //Symbol_Group_2 : "!" . Layout Identifier
                        let next_slot_id = SlotId(358);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(357),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_2 : "!" . Layout Identifier
            SlotId(358) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_Group_2 : "!" Layout . Identifier
                        let next_slot_id = SlotId(359);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(358),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_2 : "!" Layout . Identifier
            SlotId(359) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol_Group_2 : "!" Layout Identifier.
                        let next_slot_id = SlotId(360);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(359),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_2 : "!" Layout Identifier.
            SlotId(360) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(44);
                let end_slot_id = SlotId(360);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Plus_10 : . Symbol_Plus_10 Layout Symbol_Group_2
            SlotId(361) => {
                self.create_symbol_plus_10(result, gss_node_id, SlotId(362));
            }
            //Symbol_Plus_10 : Symbol_Plus_10 . Layout Symbol_Group_2
            SlotId(362) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_Plus_10 : Symbol_Plus_10 Layout . Symbol_Group_2
                        let next_slot_id = SlotId(363);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(362),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Plus_10 : Symbol_Plus_10 Layout . Symbol_Group_2
            SlotId(363) => {
                self.create_symbol_group_2(result, gss_node_id, SlotId(364));
            }
            //Symbol_Plus_10 : Symbol_Plus_10 Layout Symbol_Group_2.
            SlotId(364) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(45);
                let end_slot_id = SlotId(364);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_Plus_10 : . Symbol_Group_2
            SlotId(365) => {
                self.create_symbol_group_2(result, gss_node_id, SlotId(366));
            }
            //Symbol_Plus_10 : Symbol_Group_2.
            SlotId(366) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(45);
                let end_slot_id = SlotId(366);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex_Group_3 : . "|" Layout Regex
            SlotId(367) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Regex_Group_3 : "|" . Layout Regex
                        let next_slot_id = SlotId(368);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(367),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_3 : "|" . Layout Regex
            SlotId(368) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex_Group_3 : "|" Layout . Regex
                        let next_slot_id = SlotId(369);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(368),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_3 : "|" Layout . Regex
            SlotId(369) => {
                self.create_regex(result, gss_node_id, SlotId(370));
            }
            //Regex_Group_3 : "|" Layout Regex.
            SlotId(370) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(46);
                let end_slot_id = SlotId(370);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex_Plus_11 : . Regex_Plus_11 Layout Regex_Group_3
            SlotId(371) => {
                self.create_regex_plus_11(result, gss_node_id, SlotId(372));
            }
            //Regex_Plus_11 : Regex_Plus_11 . Layout Regex_Group_3
            SlotId(372) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Regex_Plus_11 : Regex_Plus_11 Layout . Regex_Group_3
                        let next_slot_id = SlotId(373);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(372),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Plus_11 : Regex_Plus_11 Layout . Regex_Group_3
            SlotId(373) => {
                self.create_regex_group_3(result, gss_node_id, SlotId(374));
            }
            //Regex_Plus_11 : Regex_Plus_11 Layout Regex_Group_3.
            SlotId(374) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(47);
                let end_slot_id = SlotId(374);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Regex_Plus_11 : . Regex_Group_3
            SlotId(375) => {
                self.create_regex_group_3(result, gss_node_id, SlotId(376));
            }
            //Regex_Plus_11 : Regex_Group_3.
            SlotId(376) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(47);
                let end_slot_id = SlotId(376);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //CharClass_Opt_11 : . "!"
            SlotId(377) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(29), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(29), i, j);
                        //CharClass_Opt_11 : "!".
                        let next_slot_id = SlotId(378);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(377),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Opt_11 : "!".
            SlotId(378) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(48);
                let end_slot_id = SlotId(378);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //CharClass_Opt_11 : .
            SlotId(379) => {
                let end_slot_id = SlotId(379);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(36), input_index, input_index);
                let nonterminal_id = NonterminalId(48);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //CharClass_Plus_12 : . CharClass_Plus_12 Layout RangeElement
            SlotId(380) => {
                self.create_char_class_plus_12(result, gss_node_id, SlotId(381));
            }
            //CharClass_Plus_12 : CharClass_Plus_12 . Layout RangeElement
            SlotId(381) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //CharClass_Plus_12 : CharClass_Plus_12 Layout . RangeElement
                        let next_slot_id = SlotId(382);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(381),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Plus_12 : CharClass_Plus_12 Layout . RangeElement
            SlotId(382) => {
                self.create_range_element(result, gss_node_id, SlotId(383));
            }
            //CharClass_Plus_12 : CharClass_Plus_12 Layout RangeElement.
            SlotId(383) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(49);
                let end_slot_id = SlotId(383);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //CharClass_Plus_12 : . RangeElement
            SlotId(384) => {
                self.create_range_element(result, gss_node_id, SlotId(385));
            }
            //CharClass_Plus_12 : RangeElement.
            SlotId(385) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(49);
                let end_slot_id = SlotId(385);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . Identifier return 0
            SlotId(386) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol_except_Except(p: i32) : Identifier . return 0
                        let next_slot_id = SlotId(387);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(386),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : Identifier . return 0
            SlotId(387) => {
                self.execute(input_index, SlotId(388), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : Identifier return 0.
            SlotId(388) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(388);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . "(" Layout Alternative_Plus_7 Layout ")" return 0
            SlotId(389) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol_except_Except(p: i32) : "(" . Layout Alternative_Plus_7 Layout ")" return 0
                        let next_slot_id = SlotId(390);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(389),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" . Layout Alternative_Plus_7 Layout ")" return 0
            SlotId(390) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "(" Layout . Alternative_Plus_7 Layout ")" return 0
                        let next_slot_id = SlotId(391);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(390),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" Layout . Alternative_Plus_7 Layout ")" return 0
            SlotId(391) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(392));
            }
            //Symbol_except_Except(p: i32) : "(" Layout Alternative_Plus_7 . Layout ")" return 0
            SlotId(392) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "(" Layout Alternative_Plus_7 Layout . ")" return 0
                        let next_slot_id = SlotId(393);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(392),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" Layout Alternative_Plus_7 Layout . ")" return 0
            SlotId(393) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol_except_Except(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" . return 0
                        let next_slot_id = SlotId(394);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(393),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" . return 0
            SlotId(394) => {
                self.execute(input_index, SlotId(395), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" return 0.
            SlotId(395) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(395);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(396) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol_except_Except(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(397);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(396),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(397) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(398);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(397),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(398) => {
                self.create_symbol(result, gss_node_id, SlotId(399), env, None, 0);
            }
            //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) . Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(399) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(400);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(399),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(400) => {
                self.create_symbol_plus_8(result, gss_node_id, SlotId(401));
            }
            //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 . Layout ")" return 0
            SlotId(401) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout . ")" return 0
                        let next_slot_id = SlotId(402);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(401),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout . ")" return 0
            SlotId(402) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" . return 0
                        let next_slot_id = SlotId(403);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(402),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" . return 0
            SlotId(403) => {
                self.execute(input_index, SlotId(404), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0.
            SlotId(404) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(404);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . """ Layout String Layout """ return 0
            SlotId(405) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Symbol_except_Except(p: i32) : """ . Layout String Layout """ return 0
                        let next_slot_id = SlotId(406);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(405),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : """ . Layout String Layout """ return 0
            SlotId(406) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : """ Layout . String Layout """ return 0
                        let next_slot_id = SlotId(407);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(406),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : """ Layout . String Layout """ return 0
            SlotId(407) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Symbol_except_Except(p: i32) : """ Layout String . Layout """ return 0
                        let next_slot_id = SlotId(408);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "String",
                            i,
                            SlotId(407),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : """ Layout String . Layout """ return 0
            SlotId(408) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : """ Layout String Layout . """ return 0
                        let next_slot_id = SlotId(409);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(408),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : """ Layout String Layout . """ return 0
            SlotId(409) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Symbol_except_Except(p: i32) : """ Layout String Layout """ . return 0
                        let next_slot_id = SlotId(410);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(409),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : """ Layout String Layout """ . return 0
            SlotId(410) => {
                self.execute(input_index, SlotId(411), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : """ Layout String Layout """ return 0.
            SlotId(411) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(411);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(412) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Symbol_except_Except(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(413);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(412),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(413) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(414);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(413),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(414) => {
                self.create_symbol(result, gss_node_id, SlotId(415), env, None, 0);
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(415) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(416);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(415),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(416) => {
                self.create_symbol(result, gss_node_id, SlotId(417), env, None, 0);
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "*" return 0
            SlotId(417) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
                        let next_slot_id = SlotId(418);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(417),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
            SlotId(418) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
                        let next_slot_id = SlotId(419);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(418),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
            SlotId(419) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
                        let next_slot_id = SlotId(420);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(419),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
            SlotId(420) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
                        let next_slot_id = SlotId(421);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(420),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
            SlotId(421) => {
                self.execute(input_index, SlotId(422), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0.
            SlotId(422) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(422);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(423) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Symbol_except_Except(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(424);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(423),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(424) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(425);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(424),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(425) => {
                self.create_symbol(result, gss_node_id, SlotId(426), env, None, 0);
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(426) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(427);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(426),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(427) => {
                self.create_symbol(result, gss_node_id, SlotId(428), env, None, 0);
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "+" return 0
            SlotId(428) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
                        let next_slot_id = SlotId(429);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(428),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
            SlotId(429) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
                        let next_slot_id = SlotId(430);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(429),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
            SlotId(430) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
                        let next_slot_id = SlotId(431);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(430),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
            SlotId(431) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
                        let next_slot_id = SlotId(432);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(431),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
            SlotId(432) => {
                self.execute(input_index, SlotId(433), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0.
            SlotId(433) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(433);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0
            SlotId(434) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(435), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0
            SlotId(435) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(436),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "*" return 0
            SlotId(436) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(437), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "*" return 0
            SlotId(437) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "*" return 0
                        let next_slot_id = SlotId(438);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(437),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "*" return 0
            SlotId(438) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" . return 0
                        let next_slot_id = SlotId(439);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(438),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" . return 0
            SlotId(439) => {
                self.execute(input_index, SlotId(440), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0.
            SlotId(440) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(440);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0
            SlotId(441) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(442), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0
            SlotId(442) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(443),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "+" return 0
            SlotId(443) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(444), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "+" return 0
            SlotId(444) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "+" return 0
                        let next_slot_id = SlotId(445);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(444),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "+" return 0
            SlotId(445) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" . return 0
                        let next_slot_id = SlotId(446);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(445),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" . return 0
            SlotId(446) => {
                self.execute(input_index, SlotId(447), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0.
            SlotId(447) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(447);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0
            SlotId(448) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(449), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0
            SlotId(449) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(450),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "?" return 0
            SlotId(450) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(451), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "?" return 0
            SlotId(451) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "?" return 0
                        let next_slot_id = SlotId(452);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(451),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "?" return 0
            SlotId(452) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(28), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(28), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" . return 0
                        let next_slot_id = SlotId(453);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"?\"",
                            i,
                            SlotId(452),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" . return 0
            SlotId(453) => {
                self.execute(input_index, SlotId(454), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0.
            SlotId(454) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(454);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
            SlotId(455) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(456), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
            SlotId(456) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(457),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
            SlotId(457) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(458), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout "!>>" Layout Identifier return 0
            SlotId(458) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "!>>" Layout Identifier return 0
                        let next_slot_id = SlotId(459);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(458),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . "!>>" Layout Identifier return 0
            SlotId(459) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!>>\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!>>\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" . Layout Identifier return 0
                        let next_slot_id = SlotId(460);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!>>\"",
                            i,
                            SlotId(459),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" . Layout Identifier return 0
            SlotId(460) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout . Identifier return 0
                        let next_slot_id = SlotId(461);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(460),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout . Identifier return 0
            SlotId(461) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier . return 0
                        let next_slot_id = SlotId(462);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(461),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier . return 0
            SlotId(462) => {
                self.execute(input_index, SlotId(463), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0.
            SlotId(463) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(463);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
            SlotId(464) => {
                if 3 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(465), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] . l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
            SlotId(465) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(466),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) . [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
            SlotId(466) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 3) {
                    self.execute(input_index, SlotId(467), result, gss_node_id, env);
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] . Layout labels:Symbol_Plus_10 return 0
            SlotId(467) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . labels:Symbol_Plus_10 return 0
                        let next_slot_id = SlotId(468);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(467),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout . labels:Symbol_Plus_10 return 0
            SlotId(468) => {
                self.create_symbol_plus_10(result, gss_node_id, SlotId(469));
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 . return 0
            SlotId(469) => {
                self.execute(input_index, SlotId(470), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0.
            SlotId(470) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(470);
                let return_value = 0;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(471) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol_except_Except(p: i32) : Identifier . Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(472);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(471),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : Identifier . Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(472) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : Identifier Layout . "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(473);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(472),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : Identifier Layout . "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(473) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!<<\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!<<\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol_except_Except(p: i32) : Identifier Layout "!<<" . Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(474);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!<<\"",
                            i,
                            SlotId(473),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : Identifier Layout "!<<" . Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(474) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : Identifier Layout "!<<" Layout . r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                        let next_slot_id = SlotId(475);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(474),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : Identifier Layout "!<<" Layout . r=Symbol(2) return r == 0 ? 2 : min(r, 2)
            SlotId(475) => {
                self.create_symbol(result, gss_node_id, SlotId(476), env, Some("r"), 2);
            }
            //Symbol_except_Except(p: i32) : Identifier Layout "!<<" Layout r=Symbol(2) . return r == 0 ? 2 : min(r, 2)
            SlotId(476) => {
                self.execute(input_index, SlotId(477), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2).
            SlotId(477) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(477);
                let return_value = if self.lookup("r", env.unwrap()) == 0 {
                    2
                } else {
                    std::cmp::min(self.lookup("r", env.unwrap()), 2)
                };
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Symbol_except_Except(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
            SlotId(478) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol_except_Except(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
                        let next_slot_id = SlotId(479);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(478),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
            SlotId(479) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
                        let next_slot_id = SlotId(480);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(479),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
            SlotId(480) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\":\"", i);
                match self.scanner.match_token(TerminalId(30), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\":\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(30), i, j);
                        //Symbol_except_Except(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
                        let next_slot_id = SlotId(481);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\":\"",
                            i,
                            SlotId(480),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
            SlotId(481) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //Symbol_except_Except(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
                        let next_slot_id = SlotId(482);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(481),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_except_Except(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
            SlotId(482) => {
                self.create_symbol(result, gss_node_id, SlotId(483), env, None, 1);
            }
            //Symbol_except_Except(p: i32) : label:Identifier Layout ":" Layout Symbol(1) . return 1
            SlotId(483) => {
                self.execute(input_index, SlotId(484), result, gss_node_id, env);
            }
            //Symbol_except_Except(p: i32) : label:Identifier Layout ":" Layout Symbol(1) return 1.
            SlotId(484) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(67);
                let end_slot_id = SlotId(484);
                let return_value = 1;
                if let Some(nonterminal_node_id) = self
                    .create_nonterminal_node_or_attach_children_symbol_except_except(
                        nonterminal_id,
                        end_slot_id,
                        left_extent,
                        right_extent,
                        result,
                        return_value,
                    )
                {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: Some(return_value),
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartGrammar : . Layout start:Grammar Layout
            SlotId(485) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartGrammar : Layout . start:Grammar Layout
                        let next_slot_id = SlotId(486);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(485),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout . start:Grammar Layout
            SlotId(486) => {
                self.create_grammar(result, gss_node_id, SlotId(487));
            }
            //StartGrammar : Layout start:Grammar . Layout
            SlotId(487) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartGrammar : Layout start:Grammar Layout.
                        let next_slot_id = SlotId(488);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(487),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout start:Grammar Layout.
            SlotId(488) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(50);
                let end_slot_id = SlotId(488);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartLayoutDef : . Layout start:LayoutDef Layout
            SlotId(489) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartLayoutDef : Layout . start:LayoutDef Layout
                        let next_slot_id = SlotId(490);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(489),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartLayoutDef : Layout . start:LayoutDef Layout
            SlotId(490) => {
                self.create_layout_def(result, gss_node_id, SlotId(491));
            }
            //StartLayoutDef : Layout start:LayoutDef . Layout
            SlotId(491) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartLayoutDef : Layout start:LayoutDef Layout.
                        let next_slot_id = SlotId(492);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(491),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartLayoutDef : Layout start:LayoutDef Layout.
            SlotId(492) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(51);
                let end_slot_id = SlotId(492);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartRule : . Layout start:Rule Layout
            SlotId(493) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRule : Layout . start:Rule Layout
                        let next_slot_id = SlotId(494);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(493),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRule : Layout . start:Rule Layout
            SlotId(494) => {
                self.create_rule(result, gss_node_id, SlotId(495));
            }
            //StartRule : Layout start:Rule . Layout
            SlotId(495) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRule : Layout start:Rule Layout.
                        let next_slot_id = SlotId(496);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(495),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRule : Layout start:Rule Layout.
            SlotId(496) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(52);
                let end_slot_id = SlotId(496);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartSyntaxRule : . Layout start:SyntaxRule Layout
            SlotId(497) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartSyntaxRule : Layout . start:SyntaxRule Layout
                        let next_slot_id = SlotId(498);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(497),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout . start:SyntaxRule Layout
            SlotId(498) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(499));
            }
            //StartSyntaxRule : Layout start:SyntaxRule . Layout
            SlotId(499) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartSyntaxRule : Layout start:SyntaxRule Layout.
                        let next_slot_id = SlotId(500);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(499),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout start:SyntaxRule Layout.
            SlotId(500) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(53);
                let end_slot_id = SlotId(500);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartAnnotation : . Layout start:Annotation Layout
            SlotId(501) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartAnnotation : Layout . start:Annotation Layout
                        let next_slot_id = SlotId(502);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(501),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAnnotation : Layout . start:Annotation Layout
            SlotId(502) => {
                self.create_annotation(result, gss_node_id, SlotId(503));
            }
            //StartAnnotation : Layout start:Annotation . Layout
            SlotId(503) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartAnnotation : Layout start:Annotation Layout.
                        let next_slot_id = SlotId(504);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(503),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAnnotation : Layout start:Annotation Layout.
            SlotId(504) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(54);
                let end_slot_id = SlotId(504);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartRegexRule : . Layout start:RegexRule Layout
            SlotId(505) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRegexRule : Layout . start:RegexRule Layout
                        let next_slot_id = SlotId(506);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(505),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout . start:RegexRule Layout
            SlotId(506) => {
                self.create_regex_rule(result, gss_node_id, SlotId(507));
            }
            //StartRegexRule : Layout start:RegexRule . Layout
            SlotId(507) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRegexRule : Layout start:RegexRule Layout.
                        let next_slot_id = SlotId(508);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(507),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout start:RegexRule Layout.
            SlotId(508) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(55);
                let end_slot_id = SlotId(508);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartPreCondition : . Layout start:PreCondition Layout
            SlotId(509) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartPreCondition : Layout . start:PreCondition Layout
                        let next_slot_id = SlotId(510);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(509),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPreCondition : Layout . start:PreCondition Layout
            SlotId(510) => {
                self.create_pre_condition(result, gss_node_id, SlotId(511));
            }
            //StartPreCondition : Layout start:PreCondition . Layout
            SlotId(511) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartPreCondition : Layout start:PreCondition Layout.
                        let next_slot_id = SlotId(512);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(511),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPreCondition : Layout start:PreCondition Layout.
            SlotId(512) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(56);
                let end_slot_id = SlotId(512);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartPostCondition : . Layout start:PostCondition Layout
            SlotId(513) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartPostCondition : Layout . start:PostCondition Layout
                        let next_slot_id = SlotId(514);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(513),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPostCondition : Layout . start:PostCondition Layout
            SlotId(514) => {
                self.create_post_condition(result, gss_node_id, SlotId(515));
            }
            //StartPostCondition : Layout start:PostCondition . Layout
            SlotId(515) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartPostCondition : Layout start:PostCondition Layout.
                        let next_slot_id = SlotId(516);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(515),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPostCondition : Layout start:PostCondition Layout.
            SlotId(516) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(57);
                let end_slot_id = SlotId(516);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartPriorityLevel : . Layout start:PriorityLevel Layout
            SlotId(517) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartPriorityLevel : Layout . start:PriorityLevel Layout
                        let next_slot_id = SlotId(518);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(517),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout . start:PriorityLevel Layout
            SlotId(518) => {
                self.create_priority_level(result, gss_node_id, SlotId(519));
            }
            //StartPriorityLevel : Layout start:PriorityLevel . Layout
            SlotId(519) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartPriorityLevel : Layout start:PriorityLevel Layout.
                        let next_slot_id = SlotId(520);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(519),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout start:PriorityLevel Layout.
            SlotId(520) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(520);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartAssociativity : . Layout start:Associativity Layout
            SlotId(521) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartAssociativity : Layout . start:Associativity Layout
                        let next_slot_id = SlotId(522);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(521),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAssociativity : Layout . start:Associativity Layout
            SlotId(522) => {
                self.create_associativity(result, gss_node_id, SlotId(523));
            }
            //StartAssociativity : Layout start:Associativity . Layout
            SlotId(523) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartAssociativity : Layout start:Associativity Layout.
                        let next_slot_id = SlotId(524);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(523),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAssociativity : Layout start:Associativity Layout.
            SlotId(524) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(59);
                let end_slot_id = SlotId(524);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartAlternative : . Layout start:Alternative Layout
            SlotId(525) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartAlternative : Layout . start:Alternative Layout
                        let next_slot_id = SlotId(526);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(525),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout . start:Alternative Layout
            SlotId(526) => {
                self.create_alternative(result, gss_node_id, SlotId(527));
            }
            //StartAlternative : Layout start:Alternative . Layout
            SlotId(527) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartAlternative : Layout start:Alternative Layout.
                        let next_slot_id = SlotId(528);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(527),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout start:Alternative Layout.
            SlotId(528) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(60);
                let end_slot_id = SlotId(528);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartSymbol : . Layout start:Symbol(0) Layout
            SlotId(529) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartSymbol : Layout . start:Symbol(0) Layout
                        let next_slot_id = SlotId(530);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(529),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSymbol : Layout . start:Symbol(0) Layout
            SlotId(530) => {
                self.create_symbol(result, gss_node_id, SlotId(531), env, None, 0);
            }
            //StartSymbol : Layout start:Symbol(0) . Layout
            SlotId(531) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartSymbol : Layout start:Symbol(0) Layout.
                        let next_slot_id = SlotId(532);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(531),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSymbol : Layout start:Symbol(0) Layout.
            SlotId(532) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(61);
                let end_slot_id = SlotId(532);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartRegex : . Layout start:Regex Layout
            SlotId(533) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRegex : Layout . start:Regex Layout
                        let next_slot_id = SlotId(534);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(533),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout . start:Regex Layout
            SlotId(534) => {
                self.create_regex(result, gss_node_id, SlotId(535));
            }
            //StartRegex : Layout start:Regex . Layout
            SlotId(535) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRegex : Layout start:Regex Layout.
                        let next_slot_id = SlotId(536);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(535),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout start:Regex Layout.
            SlotId(536) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(62);
                let end_slot_id = SlotId(536);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartCharClass : . Layout start:CharClass Layout
            SlotId(537) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartCharClass : Layout . start:CharClass Layout
                        let next_slot_id = SlotId(538);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(537),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout . start:CharClass Layout
            SlotId(538) => {
                self.create_char_class(result, gss_node_id, SlotId(539));
            }
            //StartCharClass : Layout start:CharClass . Layout
            SlotId(539) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartCharClass : Layout start:CharClass Layout.
                        let next_slot_id = SlotId(540);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(539),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout start:CharClass Layout.
            SlotId(540) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(540);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartRangeElement : . Layout start:RangeElement Layout
            SlotId(541) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRangeElement : Layout . start:RangeElement Layout
                        let next_slot_id = SlotId(542);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(541),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRangeElement : Layout . start:RangeElement Layout
            SlotId(542) => {
                self.create_range_element(result, gss_node_id, SlotId(543));
            }
            //StartRangeElement : Layout start:RangeElement . Layout
            SlotId(543) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRangeElement : Layout start:RangeElement Layout.
                        let next_slot_id = SlotId(544);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(543),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRangeElement : Layout start:RangeElement Layout.
            SlotId(544) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(64);
                let end_slot_id = SlotId(544);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartRange : . Layout start:Range Layout
            SlotId(545) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRange : Layout . start:Range Layout
                        let next_slot_id = SlotId(546);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(545),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout . start:Range Layout
            SlotId(546) => {
                self.create_range(result, gss_node_id, SlotId(547));
            }
            //StartRange : Layout start:Range . Layout
            SlotId(547) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(35), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(35), i, j);
                        //StartRange : Layout start:Range Layout.
                        let next_slot_id = SlotId(548);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
                            right_child_id,
                        ) {
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(547),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout start:Range Layout.
            SlotId(548) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(65);
                let end_slot_id = SlotId(548);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            _ => {
                panic!("Unknown grammar slot id: {slot_id}");
            }
        }
    }
    fn add_first_descriptors(
        &mut self,
        nonterminal_id: NonterminalId,
        input_index: u32,
        gss_node_id: GssNodeId,
        env: Option<EnvId>,
    ) {
        match nonterminal_id {
            //Grammar
            NonterminalId(0) => {
                //Grammar : . "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef
            NonterminalId(1) => {
                //LayoutDef : . "layout" Layout LayoutDef_Star_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(8),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Rule
            NonterminalId(2) => {
                //Rule : . SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(12),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Rule : . RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule
            NonterminalId(3) => {
                //SyntaxRule : . SyntaxRule_Opt_3 Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(16),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Annotation
            NonterminalId(4) => {
                //Annotation : . "@NoLayout"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(24),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Annotation : . "@Layout" Layout "(" Layout Identifier Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(26),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule
            NonterminalId(5) => {
                //RegexRule : . "@regex" Layout Identifier Layout "=" Layout RegexRule_Opt_5 Layout body:RegexRule_Plus_3 Layout RegexRule_Star_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(34),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PreCondition
            NonterminalId(6) => {
                //PreCondition : . Identifier Layout "!<<"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(46),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PostCondition
            NonterminalId(7) => {
                //PostCondition : . "\" Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(50),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PostCondition : . "!>>" Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(54),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel
            NonterminalId(8) => {
                //PriorityLevel : . PriorityLevel_Opt_7 Layout PriorityLevel_Star_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(58),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Associativity
            NonterminalId(9) => {
                //Associativity : . "left"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(62),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Associativity : . "right"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(64),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Associativity : . "none"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(66),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative
            NonterminalId(10) => {
                //Alternative : . Alternative_Star_5 Layout Alternative_Opt_10
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(68),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol
            NonterminalId(66) => {
                //Symbol(p: i32) : . Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(72),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "(" Layout Alternative_Plus_7 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(75),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(82),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . """ Layout String Layout """ return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(91),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(98),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(109),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(120),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(127),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(134),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [3 >= p] l=Symbol_except_Except(p) [l == 0 || l >= 3] Layout excepts:Symbol_Plus_9 return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(141),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(148),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(157),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(164),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(171),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex
            NonterminalId(11) => {
                //Regex : . Regex Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(178),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . Regex Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(182),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . Regex Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(186),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "(" Layout first:Regex Layout rest:Regex_Plus_11 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(190),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "(" Layout RegexRule_Plus_4 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(198),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . CharClass
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(204),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "'" Layout Char Layout "'"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(206),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . """ Layout String Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(212),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(218),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass
            NonterminalId(12) => {
                //CharClass : . neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_12 Layout "]"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(220),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RangeElement
            NonterminalId(13) => {
                //RangeElement : . Range
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(228),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RangeElement : . RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(230),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Range
            NonterminalId(14) => {
                //Range : . start:RangeChar Layout "-" Layout end:RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(232),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_0
            NonterminalId(15) => {
                //Grammar_Opt_0 : . LayoutDef
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(238),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(240),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Plus_0
            NonterminalId(16) => {
                //Grammar_Plus_0 : . Grammar_Plus_0 Layout Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(241),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Plus_0 : . Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(245),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_1
            NonterminalId(17) => {
                //Grammar_Opt_1 : . Grammar_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(247),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(249),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Star_0
            NonterminalId(18) => {
                //Grammar_Star_0 : . Grammar_Opt_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(250),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Plus_1
            NonterminalId(19) => {
                //LayoutDef_Plus_1 : . LayoutDef_Plus_1 Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(252),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //LayoutDef_Plus_1 : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(256),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Opt_2
            NonterminalId(20) => {
                //LayoutDef_Opt_2 : . LayoutDef_Plus_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(258),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //LayoutDef_Opt_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(260),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Star_1
            NonterminalId(21) => {
                //LayoutDef_Star_1 : . LayoutDef_Opt_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(261),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Opt_3
            NonterminalId(22) => {
                //SyntaxRule_Opt_3 : . Annotation
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(263),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Opt_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(265),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Plus_2
            NonterminalId(23) => {
                //SyntaxRule_Plus_2 : . SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(266),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Plus_2 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(272),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Opt_4
            NonterminalId(24) => {
                //SyntaxRule_Opt_4 : . SyntaxRule_Plus_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(274),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Opt_4 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(276),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Star_2
            NonterminalId(25) => {
                //SyntaxRule_Star_2 : . SyntaxRule_Opt_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(277),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Opt_5
            NonterminalId(26) => {
                //RegexRule_Opt_5 : . PreCondition
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(279),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Opt_5 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(281),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_4
            NonterminalId(27) => {
                //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(282),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_4 : . Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(286),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_3
            NonterminalId(28) => {
                //RegexRule_Plus_3 : . RegexRule_Plus_3 Layout "|" Layout RegexRule_Plus_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(288),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_3 : . RegexRule_Plus_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(294),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_5
            NonterminalId(29) => {
                //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout PostCondition
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(296),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_5 : . PostCondition
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(300),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Opt_6
            NonterminalId(30) => {
                //RegexRule_Opt_6 : . RegexRule_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(302),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Opt_6 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(304),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Star_3
            NonterminalId(31) => {
                //RegexRule_Star_3 : . RegexRule_Opt_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(305),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Opt_7
            NonterminalId(32) => {
                //PriorityLevel_Opt_7 : . Associativity
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(307),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Opt_7 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(309),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Plus_6
            NonterminalId(33) => {
                //PriorityLevel_Plus_6 : . PriorityLevel_Plus_6 Layout "|" Layout Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(310),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Plus_6 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(316),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Opt_8
            NonterminalId(34) => {
                //PriorityLevel_Opt_8 : . PriorityLevel_Plus_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(318),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Opt_8 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(320),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Star_4
            NonterminalId(35) => {
                //PriorityLevel_Star_4 : . PriorityLevel_Opt_8
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(321),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Plus_7
            NonterminalId(36) => {
                //Alternative_Plus_7 : . Alternative_Plus_7 Layout Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(323),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Plus_7 : . Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(327),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Opt_9
            NonterminalId(37) => {
                //Alternative_Opt_9 : . Alternative_Plus_7
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(329),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Opt_9 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(331),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Star_5
            NonterminalId(38) => {
                //Alternative_Star_5 : . Alternative_Opt_9
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(332),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Opt_10
            NonterminalId(39) => {
                //Alternative_Opt_10 : . Label
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(334),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Opt_10 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(336),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Group_0
            NonterminalId(40) => {
                //Symbol_Group_0 : . "|" Layout Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(337),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Plus_8
            NonterminalId(41) => {
                //Symbol_Plus_8 : . Symbol_Plus_8 Layout Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(341),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_Plus_8 : . Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(345),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Group_1
            NonterminalId(42) => {
                //Symbol_Group_1 : . "\" Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(347),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Plus_9
            NonterminalId(43) => {
                //Symbol_Plus_9 : . Symbol_Plus_9 Layout Symbol_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(351),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_Plus_9 : . Symbol_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(355),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Group_2
            NonterminalId(44) => {
                //Symbol_Group_2 : . "!" Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(357),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Plus_10
            NonterminalId(45) => {
                //Symbol_Plus_10 : . Symbol_Plus_10 Layout Symbol_Group_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(361),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_Plus_10 : . Symbol_Group_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(365),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex_Group_3
            NonterminalId(46) => {
                //Regex_Group_3 : . "|" Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(367),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex_Plus_11
            NonterminalId(47) => {
                //Regex_Plus_11 : . Regex_Plus_11 Layout Regex_Group_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(371),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex_Plus_11 : . Regex_Group_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(375),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass_Opt_11
            NonterminalId(48) => {
                //CharClass_Opt_11 : . "!"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(377),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //CharClass_Opt_11 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(379),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass_Plus_12
            NonterminalId(49) => {
                //CharClass_Plus_12 : . CharClass_Plus_12 Layout RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(380),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //CharClass_Plus_12 : . RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(384),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_except_Except
            NonterminalId(67) => {
                //Symbol_except_Except(p: i32) : . Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(386),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . "(" Layout Alternative_Plus_7 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(389),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(396),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . """ Layout String Layout """ return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(405),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(412),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(423),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(434),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(441),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "?" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(448),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout "!>>" Layout Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(455),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . [3 >= p] l=Symbol(p) [l == 0 || l >= 3] Layout labels:Symbol_Plus_10 return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(464),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . Identifier Layout "!<<" Layout r=Symbol(2) return r == 0 ? 2 : min(r, 2)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(471),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_except_Except(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(478),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartGrammar
            NonterminalId(50) => {
                //StartGrammar : . Layout start:Grammar Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(485),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartLayoutDef
            NonterminalId(51) => {
                //StartLayoutDef : . Layout start:LayoutDef Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(489),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRule
            NonterminalId(52) => {
                //StartRule : . Layout start:Rule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(493),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartSyntaxRule
            NonterminalId(53) => {
                //StartSyntaxRule : . Layout start:SyntaxRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(497),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAnnotation
            NonterminalId(54) => {
                //StartAnnotation : . Layout start:Annotation Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(501),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegexRule
            NonterminalId(55) => {
                //StartRegexRule : . Layout start:RegexRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(505),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartPreCondition
            NonterminalId(56) => {
                //StartPreCondition : . Layout start:PreCondition Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(509),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartPostCondition
            NonterminalId(57) => {
                //StartPostCondition : . Layout start:PostCondition Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(513),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartPriorityLevel
            NonterminalId(58) => {
                //StartPriorityLevel : . Layout start:PriorityLevel Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(517),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAssociativity
            NonterminalId(59) => {
                //StartAssociativity : . Layout start:Associativity Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(521),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAlternative
            NonterminalId(60) => {
                //StartAlternative : . Layout start:Alternative Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(525),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartSymbol
            NonterminalId(61) => {
                //StartSymbol : . Layout start:Symbol(0) Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(529),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegex
            NonterminalId(62) => {
                //StartRegex : . Layout start:Regex Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(533),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartCharClass
            NonterminalId(63) => {
                //StartCharClass : . Layout start:CharClass Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(537),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRangeElement
            NonterminalId(64) => {
                //StartRangeElement : . Layout start:RangeElement Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(541),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRange
            NonterminalId(65) => {
                //StartRange : . Layout start:Range Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(545),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            _ => {
                panic!("Unknown nonterminal id: {nonterminal_id}");
            }
        }
    }
    fn get_gss_node(&self, nonterminal_id: NonterminalId, input_index: u32) -> Option<GssNodeId> {
        let gss_nodes = &self.gss_nodes_index[nonterminal_id.index()];
        gss_nodes
            .iter()
            .find(|(k, _)| *k == input_index)
            .map(|x| x.1)
    }
    fn add_gss_node(
        &mut self,
        nonterminal_id: NonterminalId,
        input_index: u32,
        gss_node_id: GssNodeId,
    ) {
        let gss_nodes = &mut self.gss_nodes_index[nonterminal_id.index()];
        gss_nodes.push((input_index, gss_node_id));
    }
    fn new_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> GssNodeId {
        let gss_node_id = GssNodeId(self.gss_nodes.len() as u32);
        let gss_node = GSSNode::new(gss_node_id, nonterminal_id, input_index);
        record!(self, GSSNodeCreated, nonterminal_id, input_index);
        self.gss_nodes.push(gss_node);
        self.stats.gss_nodes_count += 1;
        gss_node_id
    }
    fn gss_node(&self, id: GssNodeId) -> &GSSNode {
        &self.gss_nodes[id.index()]
    }
    fn gss_node_mut(&mut self, id: GssNodeId) -> &mut GSSNode {
        self.gss_nodes
            .get_mut(id.index())
            .expect("GSS node id should be valid")
    }
    fn sppf_node(&self, id: SPPFNodeId) -> &SPPFNode {
        &self.sppf_nodes[id.index()]
    }
    fn sppf_node_mut(&mut self, id: SPPFNodeId) -> &mut SPPFNode {
        &mut self.sppf_nodes[id.index()]
    }
    fn add_descriptor(&mut self, descriptor: Descriptor) {
        record!(
            self,
            DescriptorAdded,
            descriptor.input_index,
            descriptor.slot_id,
            descriptor.sppf_node_id,
            descriptor.gss_node_id
        );
        self.stats_mut().descriptors_count += 1;
        self.descriptors.push(descriptor);
    }
    fn next_descriptor(&mut self) -> Option<Descriptor> {
        self.descriptors.pop()
    }
    fn add_terminal_node(&mut self, terminal_node: TerminalNode) -> SPPFNodeId {
        let terminal_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
        self.stats.terminal_nodes_count += 1;
        self.terminal_nodes_index[terminal_node.terminal_id.index()]
            .insert(terminal_node.span, terminal_node_id);
        record!(
            self,
            TerminalNodeCreated,
            terminal_node.terminal_id,
            terminal_node.span
        );
        self.sppf_nodes.push(SPPFNode::Terminal(terminal_node));
        terminal_node_id
    }
    fn add_nonterminal_node(&mut self, nonterminal_node: NonterminalNode) -> SPPFNodeId {
        let nonterminal_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
        self.stats.nonterminal_nodes_count += 1;
        self.nonterminal_nodes_index[nonterminal_node.nonterminal_id.index()]
            .insert(nonterminal_node.span, nonterminal_node_id);
        record!(
            self,
            NonterminalNodeCreated,
            nonterminal_node.nonterminal_id,
            nonterminal_node.span,
            nonterminal_node.child
        );
        self.sppf_nodes
            .push(SPPFNode::Nonterminal(nonterminal_node));
        nonterminal_node_id
    }
    fn add_intermediate_node(&mut self, intermediate_node: IntermediateNode) -> SPPFNodeId {
        let intermediate_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
        self.stats.intermediate_nodes_count += 1;
        self.intermediate_nodes_index[intermediate_node.slot_id.index()]
            .insert(intermediate_node.span, intermediate_node_id);
        record!(
            self,
            IntermediateNodeCreated,
            intermediate_node.slot_id,
            intermediate_node.span,
            intermediate_node.child.0,
            intermediate_node.child.1
        );
        self.sppf_nodes
            .push(SPPFNode::Intermediate(intermediate_node));
        intermediate_node_id
    }
    fn input(&self) -> &'i Input {
        self.scanner.input
    }
    fn stats(&self) -> &Stats {
        &self.stats
    }
    fn stats_mut(&mut self) -> &mut Stats {
        &mut self.stats
    }
    fn lookup_nonterminal_node(
        &self,
        nonterminal_id: NonterminalId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<SPPFNodeId> {
        let map = &self.nonterminal_nodes_index[nonterminal_id.index()];
        map.get(&Span::new(left_extent, right_extent)).copied()
    }
    fn lookup_intermediate_node(
        &self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<SPPFNodeId> {
        let map = &self.intermediate_nodes_index[slot_id.index()];
        map.get(&Span::new(left_extent, right_extent)).copied()
    }
    fn lookup_terminal_node(
        &self,
        terminal_id: TerminalId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<SPPFNodeId> {
        let map = &self.terminal_nodes_index[terminal_id.index()];
        map.get(&Span::new(left_extent, right_extent)).copied()
    }
    fn gss_nodes(&self) -> impl Iterator<Item = &GSSNode> {
        self.gss_nodes.iter()
    }
    fn add_intermediate_node_child(
        &mut self,
        node: SPPFNodeId,
        child1: SPPFNodeId,
        child2: SPPFNodeId,
    ) {
        self.intermediate_nodes_children
            .push((node, (child1, child2)));
    }
    fn add_nonterminal_node_child(&mut self, node: SPPFNodeId, child: SPPFNodeId) {
        self.nonterminal_nodes_children.push((node, child));
    }
    fn intermediate_nodes_children_map(
        &self,
    ) -> &FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>> {
        self.intermediate_nodes_children_map.get_or_init(|| {
            let mut map: FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>> =
                FxHashMap::default();
            for (k, v) in &self.intermediate_nodes_children {
                map.entry(*k).or_default().push(*v);
            }
            map
        })
    }
    fn nonterminal_nodes_children_map(&self) -> &FxHashMap<SPPFNodeId, Vec<SPPFNodeId>> {
        self.nonterminal_nodes_children_map.get_or_init(|| {
            let mut map: FxHashMap<SPPFNodeId, Vec<SPPFNodeId>> = FxHashMap::default();
            for (k, v) in &self.nonterminal_nodes_children {
                map.entry(*k).or_default().push(*v);
            }
            map
        })
    }
    #[cfg(feature = "debug-trace")]
    fn add_trace_event(&mut self, event: TraceEvent) {
        if let Some(trace_events) = &mut self.trace_events {
            trace_events.push(event);
        }
    }
    fn start_nonterminal(&self) -> NonterminalId {
        self.start_nonterminal
    }
    fn new_env(&mut self) -> (EnvId, &mut Env) {
        let id = EnvId(self.envs.len() as u32);
        self.envs.push(Env::default());
        (id, &mut self.envs[id.index()])
    }
    fn lookup(&self, name: &str, env_id: EnvId) -> i32 {
        let env = &self.envs[env_id.index()];
        env.get(name)
    }
    fn clone_env(&mut self, source: EnvId) -> (EnvId, &mut Env) {
        let bindings = self.envs[source.0 as usize].bindings.clone();
        let (new_id, new_env) = self.new_env();
        new_env.bindings = bindings;
        (new_id, new_env)
    }
    fn post_conditions(&self, slot: SlotId, left_extent: u32, right_extent: u32) -> bool {
        match slot {
            _ => true,
        }
    }
}
pub struct IggyParser<'i> {
    start_nonterminal: NonterminalId,
    scanner: IggyScanner<'i>,
    descriptors: Vec<Descriptor>,
    gss_nodes: Vec<GSSNode>,
    //A vector from nonterminal_ids to a tuple (input_index, gss_node_id)
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 68],
    //GSS index for nonterminal Symbol
    gss_nodes_index_symbol: Vec<(u32, i32, GssNodeId)>,
    //GSS index for nonterminal Symbol_except_Except
    gss_nodes_index_symbol_except_except: Vec<(u32, i32, GssNodeId)>,
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 68],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 549],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 37],
    intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
    intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
    nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
    nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
    nonterminal_nodes_index_symbol: FxHashMap<Span, InlineVec<(i32, SPPFNodeId)>>,
    nonterminal_nodes_index_symbol_except_except: FxHashMap<Span, InlineVec<(i32, SPPFNodeId)>>,
    envs: Vec<Env>,
    #[cfg(feature = "debug-trace")]
    pub trace_events: Option<Vec<TraceEvent>>,
}
impl<'i> IggyParser<'i> {
    pub fn new(input: &'i Input, start_nonterminal: NonterminalId) -> Self {
        init_logger();
        Self {
            start_nonterminal,
            scanner: IggyScanner::new(input),
            gss_nodes_index: [const { vec![] }; 68],
            gss_nodes_index_symbol: vec![],
            gss_nodes_index_symbol_except_except: vec![],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 68],
            intermediate_nodes_index: [const { InlineMap::Empty }; 549],
            terminal_nodes_index: [const { InlineMap::Empty }; 37],
            stats: Stats::default(),
            intermediate_nodes_children: vec![],
            intermediate_nodes_children_map: OnceCell::new(),
            nonterminal_nodes_children: vec![],
            nonterminal_nodes_children_map: OnceCell::new(),
            nonterminal_nodes_index_symbol: FxHashMap::default(),
            nonterminal_nodes_index_symbol_except_except: FxHashMap::default(),
            envs: vec![],
            #[cfg(feature = "debug-trace")]
            trace_events: None,
        }
    }
    fn create_grammar(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(0), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(1), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(2), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(3), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_annotation(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(4), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(5), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_pre_condition(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(6), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_post_condition(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(7), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(8), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_associativity(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(9), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(10), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(11), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(12), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(13), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(14), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(15), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_plus_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(16), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(17), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_star_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(18), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_plus_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(19), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_opt_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(20), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_star_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(21), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_opt_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(22), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_plus_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(23), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_opt_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(24), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_star_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(25), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_opt_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(26), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_plus_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(27), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_plus_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(28), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_plus_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(29), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_opt_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(30), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_star_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(31), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_opt_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(32), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_plus_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(33), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_opt_8(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(34), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_star_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(35), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_plus_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(36), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_opt_9(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(37), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_star_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(38), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_opt_10(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(39), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_group_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(40), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_plus_8(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(41), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_group_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(42), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_plus_9(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(43), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_group_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(44), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_plus_10(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(45), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_group_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(46), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_plus_11(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(47), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_opt_11(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(48), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_plus_12(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(49), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_grammar(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(50), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_layout_def(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(51), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(52), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_syntax_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(53), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_annotation(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(54), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(55), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_pre_condition(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(56), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_post_condition(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(57), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(58), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_associativity(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(59), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(60), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_symbol(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(61), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(62), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(63), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(64), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(65), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
        env: Option<EnvId>,
        binding: Option<&'static str>,
        p: i32,
    ) {
        record!(self, Call, sppf_node_id, gss_node_id, return_slot);
        let left_child = sppf_node_id.map(|id| {
            let node = self.sppf_node(id);
            (id, node.left_extent())
        });
        let gss_node = self.gss_node(gss_node_id);
        let i = match left_child {
            Some((id, _)) => self.sppf_node(id).right_extent(),
            None => gss_node.index,
        };
        //If there is already a GSS node for this call, add an edge.
        if let Some(existing_gss_node_id) = self.get_gss_node_symbol(i, p) {
            record!(self, GSSNodeFound, NonterminalId(66), i);
            self.add_edge_to_existing_gss_node(
                existing_gss_node_id,
                gss_node_id,
                left_child,
                return_slot,
                env,
                binding,
            );
        } else {
            record!(self, GSSNodeNotFound, NonterminalId(66), i);
            let new_gss_node_id = self.new_gss_node(NonterminalId(66), i);
            self.add_gss_edge(
                new_gss_node_id,
                gss_node_id,
                sppf_node_id,
                return_slot,
                env,
                binding,
            );
            let (env_id, env) = self.new_env();
            env.bind("p", p);
            self.add_first_descriptors(NonterminalId(66), i, new_gss_node_id, Some(env_id));
            self.add_gss_node_symbol(i, p, new_gss_node_id);
        }
    }
    fn create_symbol_except_except(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
        env: Option<EnvId>,
        binding: Option<&'static str>,
        p: i32,
    ) {
        record!(self, Call, sppf_node_id, gss_node_id, return_slot);
        let left_child = sppf_node_id.map(|id| {
            let node = self.sppf_node(id);
            (id, node.left_extent())
        });
        let gss_node = self.gss_node(gss_node_id);
        let i = match left_child {
            Some((id, _)) => self.sppf_node(id).right_extent(),
            None => gss_node.index,
        };
        //If there is already a GSS node for this call, add an edge.
        if let Some(existing_gss_node_id) = self.get_gss_node_symbol_except_except(i, p) {
            record!(self, GSSNodeFound, NonterminalId(67), i);
            self.add_edge_to_existing_gss_node(
                existing_gss_node_id,
                gss_node_id,
                left_child,
                return_slot,
                env,
                binding,
            );
        } else {
            record!(self, GSSNodeNotFound, NonterminalId(67), i);
            let new_gss_node_id = self.new_gss_node(NonterminalId(67), i);
            self.add_gss_edge(
                new_gss_node_id,
                gss_node_id,
                sppf_node_id,
                return_slot,
                env,
                binding,
            );
            let (env_id, env) = self.new_env();
            env.bind("p", p);
            self.add_first_descriptors(NonterminalId(67), i, new_gss_node_id, Some(env_id));
            self.add_gss_node_symbol_except_except(i, p, new_gss_node_id);
        }
    }
    fn get_gss_node_symbol(&self, input_index: u32, p: i32) -> Option<GssNodeId> {
        self.gss_nodes_index_symbol
            .iter()
            .find(|(i, a0, _)| *i == input_index && *a0 == p)
            .map(|x| x.2)
    }
    fn get_gss_node_symbol_except_except(&self, input_index: u32, p: i32) -> Option<GssNodeId> {
        self.gss_nodes_index_symbol_except_except
            .iter()
            .find(|(i, a0, _)| *i == input_index && *a0 == p)
            .map(|x| x.2)
    }
    fn add_gss_node_symbol(&mut self, input_index: u32, p: i32, gss_node_id: GssNodeId) {
        self.gss_nodes_index_symbol
            .push((input_index, p, gss_node_id));
    }
    fn add_gss_node_symbol_except_except(
        &mut self,
        input_index: u32,
        p: i32,
        gss_node_id: GssNodeId,
    ) {
        self.gss_nodes_index_symbol_except_except
            .push((input_index, p, gss_node_id));
    }
    fn lookup_nonterminal_node_symbol(
        &self,
        left_extent: u32,
        right_extent: u32,
        return_value: i32,
    ) -> Option<SPPFNodeId> {
        let span = Span::new(left_extent, right_extent);
        self.nonterminal_nodes_index_symbol
            .get(&span)
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|(rv, _)| *rv == return_value)
                    .map(|(_, id)| *id)
            })
    }
    fn lookup_nonterminal_node_symbol_except_except(
        &self,
        left_extent: u32,
        right_extent: u32,
        return_value: i32,
    ) -> Option<SPPFNodeId> {
        let span = Span::new(left_extent, right_extent);
        self.nonterminal_nodes_index_symbol_except_except
            .get(&span)
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|(rv, _)| *rv == return_value)
                    .map(|(_, id)| *id)
            })
    }
    fn add_nonterminal_node_symbol(
        &mut self,
        nonterminal_node: NonterminalNode,
        return_value: i32,
    ) -> SPPFNodeId {
        let nonterminal_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
        self.stats.nonterminal_nodes_count += 1;
        self.nonterminal_nodes_index_symbol
            .entry(nonterminal_node.span)
            .or_default()
            .push((return_value, nonterminal_node_id));
        record!(
            self,
            NonterminalNodeCreated,
            nonterminal_node.nonterminal_id,
            nonterminal_node.span,
            nonterminal_node.child
        );
        self.sppf_nodes
            .push(SPPFNode::Nonterminal(nonterminal_node));
        nonterminal_node_id
    }
    fn add_nonterminal_node_symbol_except_except(
        &mut self,
        nonterminal_node: NonterminalNode,
        return_value: i32,
    ) -> SPPFNodeId {
        let nonterminal_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
        self.stats.nonterminal_nodes_count += 1;
        self.nonterminal_nodes_index_symbol_except_except
            .entry(nonterminal_node.span)
            .or_default()
            .push((return_value, nonterminal_node_id));
        record!(
            self,
            NonterminalNodeCreated,
            nonterminal_node.nonterminal_id,
            nonterminal_node.span,
            nonterminal_node.child
        );
        self.sppf_nodes
            .push(SPPFNode::Nonterminal(nonterminal_node));
        nonterminal_node_id
    }
    fn create_nonterminal_node_or_attach_children_symbol(
        &mut self,
        nonterminal_id: NonterminalId,
        return_slot: SlotId,
        left_extent: u32,
        right_extent: u32,
        child: SPPFNodeId,
        return_value: i32,
    ) -> Option<SPPFNodeId> {
        if let Some(existing_node_id) =
            self.lookup_nonterminal_node_symbol(left_extent, right_extent, return_value)
        {
            record!(self, NonterminalNodeFound, existing_node_id);
            let node = self.sppf_node_mut(existing_node_id);
            let SPPFNode::Nonterminal(node) = node else {
                unreachable!("Expects a nonterminal node");
            };
            if !node.ambiguous {
                node.ambiguous = true;
                self.stats_mut().ambiguous_nodes += 1;
            }
            self.add_nonterminal_node_child(existing_node_id, child);
            return None;
        }
        let nonterminal_node = NonterminalNode {
            nonterminal_id,
            return_slot,
            span: Span {
                left_extent,
                right_extent,
            },
            child,
            ambiguous: false,
        };
        Some(self.add_nonterminal_node_symbol(nonterminal_node, return_value))
    }
    fn create_nonterminal_node_or_attach_children_symbol_except_except(
        &mut self,
        nonterminal_id: NonterminalId,
        return_slot: SlotId,
        left_extent: u32,
        right_extent: u32,
        child: SPPFNodeId,
        return_value: i32,
    ) -> Option<SPPFNodeId> {
        if let Some(existing_node_id) = self.lookup_nonterminal_node_symbol_except_except(
            left_extent,
            right_extent,
            return_value,
        ) {
            record!(self, NonterminalNodeFound, existing_node_id);
            let node = self.sppf_node_mut(existing_node_id);
            let SPPFNode::Nonterminal(node) = node else {
                unreachable!("Expects a nonterminal node");
            };
            if !node.ambiguous {
                node.ambiguous = true;
                self.stats_mut().ambiguous_nodes += 1;
            }
            self.add_nonterminal_node_child(existing_node_id, child);
            return None;
        }
        let nonterminal_node = NonterminalNode {
            nonterminal_id,
            return_slot,
            span: Span {
                left_extent,
                right_extent,
            },
            child,
            ambiguous: false,
        };
        Some(self.add_nonterminal_node_symbol_except_except(nonterminal_node, return_value))
    }
}

