use crate::{
    scanner::IggyScanner,
    types::{EbnfKind, Nonterminal, Slot, Terminal},
};
#[cfg(feature = "debug-trace")]
use iguana_runtime::trace::TraceEvent;
use iguana_runtime::{
    descriptor::Descriptor,
    gss::GSSNode,
    ids::{GssNodeId, NonterminalId, SlotId, TerminalId},
    input::Input,
    parser::{Parser, Stats, init_logger},
    record,
    scanner::Scanner,
    sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
    utils::inline_map::InlineMap,
};
use phf::phf_map;
use rustc_hash::FxHashMap;
use std::cell::OnceCell;
pub const NONTERMINALS: [Nonterminal; 45] = [
    Nonterminal {
        name: "Grammar",
        display: "Grammar",
        kind: None,
    },
    Nonterminal {
        name: "SyntaxRule",
        display: "SyntaxRule",
        kind: None,
    },
    Nonterminal {
        name: "RegexBlock",
        display: "RegexBlock",
        kind: None,
    },
    Nonterminal {
        name: "RegexRule",
        display: "RegexRule",
        kind: None,
    },
    Nonterminal {
        name: "PriorityLevel",
        display: "PriorityLevel",
        kind: None,
    },
    Nonterminal {
        name: "Alternative",
        display: "Alternative",
        kind: None,
    },
    Nonterminal {
        name: "Symbol",
        display: "Symbol",
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
        name: "Range",
        display: "Range",
        kind: None,
    },
    Nonterminal {
        name: "Grammar_Plus_0",
        display: "SyntaxRule+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Grammar_Opt_0",
        display: "SyntaxRule+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Grammar_Star_0",
        display: "SyntaxRule*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Grammar_Opt_1",
        display: "RegexBlock?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "SyntaxRule_Plus_1",
        display: "{PriorityLevel \">\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "SyntaxRule_Opt_2",
        display: "{PriorityLevel \">\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "SyntaxRule_Star_1",
        display: "{PriorityLevel \">\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "RegexBlock_Plus_2",
        display: "RegexRule+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "RegexBlock_Opt_3",
        display: "RegexRule+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "RegexBlock_Star_2",
        display: "RegexRule*",
        kind: Some(EbnfKind::Star),
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
        name: "PriorityLevel_Plus_5",
        display: "{Alternative \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_4",
        display: "{Alternative \"|\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "PriorityLevel_Star_3",
        display: "{Alternative \"|\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Plus_6",
        display: "Symbol+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Alternative_Opt_5",
        display: "Symbol+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Alternative_Star_4",
        display: "Symbol*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Symbol_Group_0",
        display: "(\"|\" Symbol)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Symbol_Plus_7",
        display: "(\"|\" Symbol)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Regex_Group_1",
        display: "(\"|\" Regex)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Regex_Plus_8",
        display: "(\"|\" Regex)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "CharClass_Opt_6",
        display: "\"!\"?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "CharClass_Alt_0",
        display: "(Range | RangeChar)",
        kind: Some(EbnfKind::Alt),
    },
    Nonterminal {
        name: "CharClass_Plus_9",
        display: "(Range | RangeChar)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "StartGrammar",
        display: "StartGrammar",
        kind: None,
    },
    Nonterminal {
        name: "StartSyntaxRule",
        display: "StartSyntaxRule",
        kind: None,
    },
    Nonterminal {
        name: "StartRegexBlock",
        display: "StartRegexBlock",
        kind: None,
    },
    Nonterminal {
        name: "StartRegexRule",
        display: "StartRegexRule",
        kind: None,
    },
    Nonterminal {
        name: "StartPriorityLevel",
        display: "StartPriorityLevel",
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
        name: "StartRange",
        display: "StartRange",
        kind: None,
    },
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Grammar" => NonterminalId(0), "SyntaxRule" => NonterminalId(1), "RegexBlock" =>
    NonterminalId(2), "RegexRule" => NonterminalId(3), "PriorityLevel" =>
    NonterminalId(4), "Alternative" => NonterminalId(5), "Symbol" => NonterminalId(6),
    "Regex" => NonterminalId(7), "CharClass" => NonterminalId(8), "Range" =>
    NonterminalId(9), "Grammar_Plus_0" => NonterminalId(10), "Grammar_Opt_0" =>
    NonterminalId(11), "Grammar_Star_0" => NonterminalId(12), "Grammar_Opt_1" =>
    NonterminalId(13), "SyntaxRule_Plus_1" => NonterminalId(14), "SyntaxRule_Opt_2" =>
    NonterminalId(15), "SyntaxRule_Star_1" => NonterminalId(16), "RegexBlock_Plus_2" =>
    NonterminalId(17), "RegexBlock_Opt_3" => NonterminalId(18), "RegexBlock_Star_2" =>
    NonterminalId(19), "RegexRule_Plus_4" => NonterminalId(20), "RegexRule_Plus_3" =>
    NonterminalId(21), "PriorityLevel_Plus_5" => NonterminalId(22), "PriorityLevel_Opt_4"
    => NonterminalId(23), "PriorityLevel_Star_3" => NonterminalId(24),
    "Alternative_Plus_6" => NonterminalId(25), "Alternative_Opt_5" => NonterminalId(26),
    "Alternative_Star_4" => NonterminalId(27), "Symbol_Group_0" => NonterminalId(28),
    "Symbol_Plus_7" => NonterminalId(29), "Regex_Group_1" => NonterminalId(30),
    "Regex_Plus_8" => NonterminalId(31), "CharClass_Opt_6" => NonterminalId(32),
    "CharClass_Alt_0" => NonterminalId(33), "CharClass_Plus_9" => NonterminalId(34),
    "StartGrammar" => NonterminalId(35), "StartSyntaxRule" => NonterminalId(36),
    "StartRegexBlock" => NonterminalId(37), "StartRegexRule" => NonterminalId(38),
    "StartPriorityLevel" => NonterminalId(39), "StartAlternative" => NonterminalId(40),
    "StartSymbol" => NonterminalId(41), "StartRegex" => NonterminalId(42),
    "StartCharClass" => NonterminalId(43), "StartRange" => NonterminalId(44)
};
pub const TERMINALS: [Terminal; 24] = [
    Terminal { name: "Identifier" },
    Terminal { name: "String" },
    Terminal { name: "RangeChar" },
    Terminal { name: "Char" },
    Terminal { name: "WS" },
    Terminal {
        name: "\"grammar\"",
    },
    Terminal { name: "\"=\"" },
    Terminal { name: "\">\"" },
    Terminal { name: "\"regex\"" },
    Terminal { name: "\"{\"" },
    Terminal { name: "\"}\"" },
    Terminal { name: "\"|\"" },
    Terminal { name: "\"*\"" },
    Terminal { name: "\"+\"" },
    Terminal { name: "\"?\"" },
    Terminal { name: "\"(\"" },
    Terminal { name: "\")\"" },
    Terminal { name: "\"\"\"" },
    Terminal { name: "\"!\"" },
    Terminal { name: "\"[\"" },
    Terminal { name: "\"]\"" },
    Terminal { name: "\"-\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 283] = [
    Slot {
        display_name: "Grammar : . \"grammar\" Layout Identifier Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" . Layout Identifier Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout . Identifier Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier . Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout . SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout SyntaxRule* . Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout SyntaxRule* Layout . RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout SyntaxRule* Layout RegexBlock?.",
    },
    Slot {
        display_name: "SyntaxRule : . Identifier Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Identifier . Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Identifier Layout . \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Identifier Layout \"=\" . Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Identifier Layout \"=\" Layout . {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : Identifier Layout \"=\" Layout {PriorityLevel \">\"}*.",
    },
    Slot {
        display_name: "RegexBlock : . \"regex\" Layout \"{\" Layout RegexRule* Layout \"}\"",
    },
    Slot {
        display_name: "RegexBlock : \"regex\" . Layout \"{\" Layout RegexRule* Layout \"}\"",
    },
    Slot {
        display_name: "RegexBlock : \"regex\" Layout . \"{\" Layout RegexRule* Layout \"}\"",
    },
    Slot {
        display_name: "RegexBlock : \"regex\" Layout \"{\" . Layout RegexRule* Layout \"}\"",
    },
    Slot {
        display_name: "RegexBlock : \"regex\" Layout \"{\" Layout . RegexRule* Layout \"}\"",
    },
    Slot {
        display_name: "RegexBlock : \"regex\" Layout \"{\" Layout RegexRule* . Layout \"}\"",
    },
    Slot {
        display_name: "RegexBlock : \"regex\" Layout \"{\" Layout RegexRule* Layout . \"}\"",
    },
    Slot {
        display_name: "RegexBlock : \"regex\" Layout \"{\" Layout RegexRule* Layout \"}\".",
    },
    Slot {
        display_name: "RegexRule : . Identifier Layout \"=\" Layout {Regex+ \"|\"}+",
    },
    Slot {
        display_name: "RegexRule : Identifier . Layout \"=\" Layout {Regex+ \"|\"}+",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout . \"=\" Layout {Regex+ \"|\"}+",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" . Layout {Regex+ \"|\"}+",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout . {Regex+ \"|\"}+",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout {Regex+ \"|\"}+.",
    },
    Slot {
        display_name: "PriorityLevel : . {Alternative \"|\"}*",
    },
    Slot {
        display_name: "PriorityLevel : {Alternative \"|\"}*.",
    },
    Slot {
        display_name: "Alternative : . Symbol*",
    },
    Slot {
        display_name: "Alternative : Symbol*.",
    },
    Slot {
        display_name: "Symbol : . Symbol Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : Symbol . Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : Symbol Layout . \"*\"",
    },
    Slot {
        display_name: "Symbol : Symbol Layout \"*\".",
    },
    Slot {
        display_name: "Symbol : . Symbol Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : Symbol . Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : Symbol Layout . \"+\"",
    },
    Slot {
        display_name: "Symbol : Symbol Layout \"+\".",
    },
    Slot {
        display_name: "Symbol : . Symbol Layout \"?\"",
    },
    Slot {
        display_name: "Symbol : Symbol . Layout \"?\"",
    },
    Slot {
        display_name: "Symbol : Symbol Layout . \"?\"",
    },
    Slot {
        display_name: "Symbol : Symbol Layout \"?\".",
    },
    Slot {
        display_name: "Symbol : . \"(\" Layout Symbol Layout (\"|\" Symbol)+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" . Layout Symbol Layout (\"|\" Symbol)+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout . Symbol Layout (\"|\" Symbol)+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol . Layout (\"|\" Symbol)+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout . (\"|\" Symbol)+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout (\"|\" Symbol)+ . Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout (\"|\" Symbol)+ Layout . \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout (\"|\" Symbol)+ Layout \")\".",
    },
    Slot {
        display_name: "Symbol : . \"\"\" Layout String Layout \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" . Layout String Layout \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout . String Layout \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout String . Layout \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout String Layout . \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" Layout String Layout \"\"\".",
    },
    Slot {
        display_name: "Symbol : . \"{\" Layout Symbol Layout Symbol Layout \"}\" Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" . Layout Symbol Layout Symbol Layout \"}\" Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout . Symbol Layout Symbol Layout \"}\" Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol . Layout Symbol Layout \"}\" Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout . Symbol Layout \"}\" Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol . Layout \"}\" Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout . \"}\" Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout \"}\" . Layout \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout \"}\" Layout . \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout \"}\" Layout \"*\".",
    },
    Slot {
        display_name: "Symbol : . \"{\" Layout Symbol Layout Symbol Layout \"}\" Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" . Layout Symbol Layout Symbol Layout \"}\" Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout . Symbol Layout Symbol Layout \"}\" Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol . Layout Symbol Layout \"}\" Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout . Symbol Layout \"}\" Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol . Layout \"}\" Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout . \"}\" Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout \"}\" . Layout \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout \"}\" Layout . \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Layout Symbol Layout Symbol Layout \"}\" Layout \"+\".",
    },
    Slot {
        display_name: "Symbol : . \"(\" Layout Symbol+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" . Layout Symbol+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout . Symbol+ Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol+ . Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol+ Layout . \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol+ Layout \")\".",
    },
    Slot {
        display_name: "Symbol : . Identifier",
    },
    Slot {
        display_name: "Symbol : Identifier.",
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
        display_name: "Regex : . \"(\" Layout Regex Layout (\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" . Layout Regex Layout (\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout . Regex Layout (\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex . Layout (\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex Layout . (\"|\" Regex)+ Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex Layout (\"|\" Regex)+ . Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex Layout (\"|\" Regex)+ Layout . \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout Regex Layout (\"|\" Regex)+ Layout \")\".",
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
        display_name: "Regex : . \"\"\" Layout Char Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" . Layout Char Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout . Char Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout Char . Layout \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout Char Layout . \"\"\"",
    },
    Slot {
        display_name: "Regex : \"\"\" Layout Char Layout \"\"\".",
    },
    Slot {
        display_name: "CharClass : . \"!\"? Layout \"[\" Layout (Range | RangeChar)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? . Layout \"[\" Layout (Range | RangeChar)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout . \"[\" Layout (Range | RangeChar)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" . Layout (Range | RangeChar)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout . (Range | RangeChar)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout (Range | RangeChar)+ . Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout (Range | RangeChar)+ Layout . \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout (Range | RangeChar)+ Layout \"]\".",
    },
    Slot {
        display_name: "Range : . RangeChar Layout \"-\" Layout RangeChar",
    },
    Slot {
        display_name: "Range : RangeChar . Layout \"-\" Layout RangeChar",
    },
    Slot {
        display_name: "Range : RangeChar Layout . \"-\" Layout RangeChar",
    },
    Slot {
        display_name: "Range : RangeChar Layout \"-\" . Layout RangeChar",
    },
    Slot {
        display_name: "Range : RangeChar Layout \"-\" Layout . RangeChar",
    },
    Slot {
        display_name: "Range : RangeChar Layout \"-\" Layout RangeChar.",
    },
    Slot {
        display_name: "SyntaxRule+ : . SyntaxRule+ Layout SyntaxRule",
    },
    Slot {
        display_name: "SyntaxRule+ : SyntaxRule+ . Layout SyntaxRule",
    },
    Slot {
        display_name: "SyntaxRule+ : SyntaxRule+ Layout . SyntaxRule",
    },
    Slot {
        display_name: "SyntaxRule+ : SyntaxRule+ Layout SyntaxRule.",
    },
    Slot {
        display_name: "SyntaxRule+ : . SyntaxRule",
    },
    Slot {
        display_name: "SyntaxRule+ : SyntaxRule.",
    },
    Slot {
        display_name: "SyntaxRule+? : . SyntaxRule+",
    },
    Slot {
        display_name: "SyntaxRule+? : SyntaxRule+.",
    },
    Slot {
        display_name: "SyntaxRule+? : .",
    },
    Slot {
        display_name: "SyntaxRule* : . SyntaxRule+?",
    },
    Slot {
        display_name: "SyntaxRule* : SyntaxRule+?.",
    },
    Slot {
        display_name: "RegexBlock? : . RegexBlock",
    },
    Slot {
        display_name: "RegexBlock? : RegexBlock.",
    },
    Slot {
        display_name: "RegexBlock? : .",
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
        display_name: "RegexRule+ : . RegexRule+ Layout RegexRule",
    },
    Slot {
        display_name: "RegexRule+ : RegexRule+ . Layout RegexRule",
    },
    Slot {
        display_name: "RegexRule+ : RegexRule+ Layout . RegexRule",
    },
    Slot {
        display_name: "RegexRule+ : RegexRule+ Layout RegexRule.",
    },
    Slot {
        display_name: "RegexRule+ : . RegexRule",
    },
    Slot {
        display_name: "RegexRule+ : RegexRule.",
    },
    Slot {
        display_name: "RegexRule+? : . RegexRule+",
    },
    Slot {
        display_name: "RegexRule+? : RegexRule+.",
    },
    Slot {
        display_name: "RegexRule+? : .",
    },
    Slot {
        display_name: "RegexRule* : . RegexRule+?",
    },
    Slot {
        display_name: "RegexRule* : RegexRule+?.",
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
        display_name: "Symbol+ : . Symbol+ Layout Symbol",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ . Layout Symbol",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ Layout . Symbol",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ Layout Symbol.",
    },
    Slot {
        display_name: "Symbol+ : . Symbol",
    },
    Slot {
        display_name: "Symbol+ : Symbol.",
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
        display_name: "(\"|\" Symbol) : . \"|\" Layout Symbol",
    },
    Slot {
        display_name: "(\"|\" Symbol) : \"|\" . Layout Symbol",
    },
    Slot {
        display_name: "(\"|\" Symbol) : \"|\" Layout . Symbol",
    },
    Slot {
        display_name: "(\"|\" Symbol) : \"|\" Layout Symbol.",
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
        display_name: "(Range | RangeChar) : . Range",
    },
    Slot {
        display_name: "(Range | RangeChar) : Range.",
    },
    Slot {
        display_name: "(Range | RangeChar) : . RangeChar",
    },
    Slot {
        display_name: "(Range | RangeChar) : RangeChar.",
    },
    Slot {
        display_name: "(Range | RangeChar)+ : . (Range | RangeChar)+ Layout (Range | RangeChar)",
    },
    Slot {
        display_name: "(Range | RangeChar)+ : (Range | RangeChar)+ . Layout (Range | RangeChar)",
    },
    Slot {
        display_name: "(Range | RangeChar)+ : (Range | RangeChar)+ Layout . (Range | RangeChar)",
    },
    Slot {
        display_name: "(Range | RangeChar)+ : (Range | RangeChar)+ Layout (Range | RangeChar).",
    },
    Slot {
        display_name: "(Range | RangeChar)+ : . (Range | RangeChar)",
    },
    Slot {
        display_name: "(Range | RangeChar)+ : (Range | RangeChar).",
    },
    Slot {
        display_name: "StartGrammar : . Layout Grammar Layout",
    },
    Slot {
        display_name: "StartGrammar : Layout . Grammar Layout",
    },
    Slot {
        display_name: "StartGrammar : Layout Grammar . Layout",
    },
    Slot {
        display_name: "StartGrammar : Layout Grammar Layout.",
    },
    Slot {
        display_name: "StartSyntaxRule : . Layout SyntaxRule Layout",
    },
    Slot {
        display_name: "StartSyntaxRule : Layout . SyntaxRule Layout",
    },
    Slot {
        display_name: "StartSyntaxRule : Layout SyntaxRule . Layout",
    },
    Slot {
        display_name: "StartSyntaxRule : Layout SyntaxRule Layout.",
    },
    Slot {
        display_name: "StartRegexBlock : . Layout RegexBlock Layout",
    },
    Slot {
        display_name: "StartRegexBlock : Layout . RegexBlock Layout",
    },
    Slot {
        display_name: "StartRegexBlock : Layout RegexBlock . Layout",
    },
    Slot {
        display_name: "StartRegexBlock : Layout RegexBlock Layout.",
    },
    Slot {
        display_name: "StartRegexRule : . Layout RegexRule Layout",
    },
    Slot {
        display_name: "StartRegexRule : Layout . RegexRule Layout",
    },
    Slot {
        display_name: "StartRegexRule : Layout RegexRule . Layout",
    },
    Slot {
        display_name: "StartRegexRule : Layout RegexRule Layout.",
    },
    Slot {
        display_name: "StartPriorityLevel : . Layout PriorityLevel Layout",
    },
    Slot {
        display_name: "StartPriorityLevel : Layout . PriorityLevel Layout",
    },
    Slot {
        display_name: "StartPriorityLevel : Layout PriorityLevel . Layout",
    },
    Slot {
        display_name: "StartPriorityLevel : Layout PriorityLevel Layout.",
    },
    Slot {
        display_name: "StartAlternative : . Layout Alternative Layout",
    },
    Slot {
        display_name: "StartAlternative : Layout . Alternative Layout",
    },
    Slot {
        display_name: "StartAlternative : Layout Alternative . Layout",
    },
    Slot {
        display_name: "StartAlternative : Layout Alternative Layout.",
    },
    Slot {
        display_name: "StartSymbol : . Layout Symbol Layout",
    },
    Slot {
        display_name: "StartSymbol : Layout . Symbol Layout",
    },
    Slot {
        display_name: "StartSymbol : Layout Symbol . Layout",
    },
    Slot {
        display_name: "StartSymbol : Layout Symbol Layout.",
    },
    Slot {
        display_name: "StartRegex : . Layout Regex Layout",
    },
    Slot {
        display_name: "StartRegex : Layout . Regex Layout",
    },
    Slot {
        display_name: "StartRegex : Layout Regex . Layout",
    },
    Slot {
        display_name: "StartRegex : Layout Regex Layout.",
    },
    Slot {
        display_name: "StartCharClass : . Layout CharClass Layout",
    },
    Slot {
        display_name: "StartCharClass : Layout . CharClass Layout",
    },
    Slot {
        display_name: "StartCharClass : Layout CharClass . Layout",
    },
    Slot {
        display_name: "StartCharClass : Layout CharClass Layout.",
    },
    Slot {
        display_name: "StartRange : . Layout Range Layout",
    },
    Slot {
        display_name: "StartRange : Layout . Range Layout",
    },
    Slot {
        display_name: "StartRange : Layout Range . Layout",
    },
    Slot {
        display_name: "StartRange : Layout Range Layout.",
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
            //Grammar : . "grammar" Layout Identifier Layout Grammar_Star_0 Layout Grammar_Opt_1
            SlotId(0) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(5), i, j);
                        //Grammar : "grammar" . Layout Identifier Layout Grammar_Star_0 Layout Grammar_Opt_1
                        let next_slot_id = SlotId(1);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Grammar : "grammar" . Layout Identifier Layout Grammar_Star_0 Layout Grammar_Opt_1
            SlotId(1) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Grammar : "grammar" Layout . Identifier Layout Grammar_Star_0 Layout Grammar_Opt_1
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Grammar : "grammar" Layout . Identifier Layout Grammar_Star_0 Layout Grammar_Opt_1
            SlotId(2) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Grammar : "grammar" Layout Identifier . Layout Grammar_Star_0 Layout Grammar_Opt_1
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Grammar : "grammar" Layout Identifier . Layout Grammar_Star_0 Layout Grammar_Opt_1
            SlotId(3) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Grammar : "grammar" Layout Identifier Layout . Grammar_Star_0 Layout Grammar_Opt_1
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Grammar : "grammar" Layout Identifier Layout . Grammar_Star_0 Layout Grammar_Opt_1
            SlotId(4) => {
                self.create(NonterminalId(12), result, gss_node_id, SlotId(5));
            }
            //Grammar : "grammar" Layout Identifier Layout Grammar_Star_0 . Layout Grammar_Opt_1
            SlotId(5) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Grammar : "grammar" Layout Identifier Layout Grammar_Star_0 Layout . Grammar_Opt_1
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Grammar : "grammar" Layout Identifier Layout Grammar_Star_0 Layout . Grammar_Opt_1
            SlotId(6) => {
                self.create(NonterminalId(13), result, gss_node_id, SlotId(7));
            }
            //Grammar : "grammar" Layout Identifier Layout Grammar_Star_0 Layout Grammar_Opt_1.
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
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //SyntaxRule : . Identifier Layout "=" Layout SyntaxRule_Star_1
            SlotId(8) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //SyntaxRule : Identifier . Layout "=" Layout SyntaxRule_Star_1
                        let next_slot_id = SlotId(9);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(8),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : Identifier . Layout "=" Layout SyntaxRule_Star_1
            SlotId(9) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //SyntaxRule : Identifier Layout . "=" Layout SyntaxRule_Star_1
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //SyntaxRule : Identifier Layout . "=" Layout SyntaxRule_Star_1
            SlotId(10) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(6), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(6), i, j);
                        //SyntaxRule : Identifier Layout "=" . Layout SyntaxRule_Star_1
                        let next_slot_id = SlotId(11);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"=\"",
                            i,
                            SlotId(10),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : Identifier Layout "=" . Layout SyntaxRule_Star_1
            SlotId(11) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //SyntaxRule : Identifier Layout "=" Layout . SyntaxRule_Star_1
                        let next_slot_id = SlotId(12);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(11),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : Identifier Layout "=" Layout . SyntaxRule_Star_1
            SlotId(12) => {
                self.create(NonterminalId(16), result, gss_node_id, SlotId(13));
            }
            //SyntaxRule : Identifier Layout "=" Layout SyntaxRule_Star_1.
            SlotId(13) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
                let end_slot_id = SlotId(13);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexBlock : . "regex" Layout "{" Layout RegexBlock_Star_2 Layout "}"
            SlotId(14) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"regex\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"regex\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //RegexBlock : "regex" . Layout "{" Layout RegexBlock_Star_2 Layout "}"
                        let next_slot_id = SlotId(15);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"regex\"",
                            i,
                            SlotId(14),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" . Layout "{" Layout RegexBlock_Star_2 Layout "}"
            SlotId(15) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexBlock : "regex" Layout . "{" Layout RegexBlock_Star_2 Layout "}"
                        let next_slot_id = SlotId(16);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(15),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" Layout . "{" Layout RegexBlock_Star_2 Layout "}"
            SlotId(16) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //RegexBlock : "regex" Layout "{" . Layout RegexBlock_Star_2 Layout "}"
                        let next_slot_id = SlotId(17);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(16),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" Layout "{" . Layout RegexBlock_Star_2 Layout "}"
            SlotId(17) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexBlock : "regex" Layout "{" Layout . RegexBlock_Star_2 Layout "}"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //RegexBlock : "regex" Layout "{" Layout . RegexBlock_Star_2 Layout "}"
            SlotId(18) => {
                self.create(NonterminalId(19), result, gss_node_id, SlotId(19));
            }
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_2 . Layout "}"
            SlotId(19) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_2 Layout . "}"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_2 Layout . "}"
            SlotId(20) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_2 Layout "}".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(20),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_2 Layout "}".
            SlotId(21) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let end_slot_id = SlotId(21);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexRule : . Identifier Layout "=" Layout RegexRule_Plus_3
            SlotId(22) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //RegexRule : Identifier . Layout "=" Layout RegexRule_Plus_3
                        let next_slot_id = SlotId(23);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(22),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier . Layout "=" Layout RegexRule_Plus_3
            SlotId(23) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexRule : Identifier Layout . "=" Layout RegexRule_Plus_3
                        let next_slot_id = SlotId(24);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(23),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier Layout . "=" Layout RegexRule_Plus_3
            SlotId(24) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(6), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(6), i, j);
                        //RegexRule : Identifier Layout "=" . Layout RegexRule_Plus_3
                        let next_slot_id = SlotId(25);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"=\"",
                            i,
                            SlotId(24),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier Layout "=" . Layout RegexRule_Plus_3
            SlotId(25) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexRule : Identifier Layout "=" Layout . RegexRule_Plus_3
                        let next_slot_id = SlotId(26);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(25),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier Layout "=" Layout . RegexRule_Plus_3
            SlotId(26) => {
                self.create(NonterminalId(21), result, gss_node_id, SlotId(27));
            }
            //RegexRule : Identifier Layout "=" Layout RegexRule_Plus_3.
            SlotId(27) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let end_slot_id = SlotId(27);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //PriorityLevel : . PriorityLevel_Star_3
            SlotId(28) => {
                self.create(NonterminalId(24), result, gss_node_id, SlotId(29));
            }
            //PriorityLevel : PriorityLevel_Star_3.
            SlotId(29) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(29);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative : . Alternative_Star_4
            SlotId(30) => {
                self.create(NonterminalId(27), result, gss_node_id, SlotId(31));
            }
            //Alternative : Alternative_Star_4.
            SlotId(31) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(31);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . Symbol Layout "*"
            SlotId(32) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(33));
            }
            //Symbol : Symbol . Layout "*"
            SlotId(33) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : Symbol Layout . "*"
                        let next_slot_id = SlotId(34);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(33),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout . "*"
            SlotId(34) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol : Symbol Layout "*".
                        let next_slot_id = SlotId(35);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(34),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "*".
            SlotId(35) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(35);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . Symbol Layout "+"
            SlotId(36) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(37));
            }
            //Symbol : Symbol . Layout "+"
            SlotId(37) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : Symbol Layout . "+"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : Symbol Layout . "+"
            SlotId(38) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol : Symbol Layout "+".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(38),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "+".
            SlotId(39) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(39);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . Symbol Layout "?"
            SlotId(40) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(41));
            }
            //Symbol : Symbol . Layout "?"
            SlotId(41) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : Symbol Layout . "?"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : Symbol Layout . "?"
            SlotId(42) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol : Symbol Layout "?".
                        let next_slot_id = SlotId(43);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"?\"",
                            i,
                            SlotId(42),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "?".
            SlotId(43) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(43);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . "(" Layout Symbol Layout Symbol_Plus_7 Layout ")"
            SlotId(44) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Symbol : "(" . Layout Symbol Layout Symbol_Plus_7 Layout ")"
                        let next_slot_id = SlotId(45);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(44),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Layout Symbol Layout Symbol_Plus_7 Layout ")"
            SlotId(45) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "(" Layout . Symbol Layout Symbol_Plus_7 Layout ")"
                        let next_slot_id = SlotId(46);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(45),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout . Symbol Layout Symbol_Plus_7 Layout ")"
            SlotId(46) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(47));
            }
            //Symbol : "(" Layout Symbol . Layout Symbol_Plus_7 Layout ")"
            SlotId(47) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "(" Layout Symbol Layout . Symbol_Plus_7 Layout ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "(" Layout Symbol Layout . Symbol_Plus_7 Layout ")"
            SlotId(48) => {
                self.create(NonterminalId(29), result, gss_node_id, SlotId(49));
            }
            //Symbol : "(" Layout Symbol Layout Symbol_Plus_7 . Layout ")"
            SlotId(49) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "(" Layout Symbol Layout Symbol_Plus_7 Layout . ")"
                        let next_slot_id = SlotId(50);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(49),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Symbol Layout Symbol_Plus_7 Layout . ")"
            SlotId(50) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Symbol : "(" Layout Symbol Layout Symbol_Plus_7 Layout ")".
                        let next_slot_id = SlotId(51);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(50),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Symbol Layout Symbol_Plus_7 Layout ")".
            SlotId(51) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(51);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . """ Layout String Layout """
            SlotId(52) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol : """ . Layout String Layout """
                        let next_slot_id = SlotId(53);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(52),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ . Layout String Layout """
            SlotId(53) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : """ Layout . String Layout """
                        let next_slot_id = SlotId(54);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(53),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ Layout . String Layout """
            SlotId(54) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol : """ Layout String . Layout """
                        let next_slot_id = SlotId(55);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "String",
                            i,
                            SlotId(54),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ Layout String . Layout """
            SlotId(55) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : """ Layout String Layout . """
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : """ Layout String Layout . """
            SlotId(56) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol : """ Layout String Layout """.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(56),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ Layout String Layout """.
            SlotId(57) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(57);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "*"
            SlotId(58) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "*"
                        let next_slot_id = SlotId(59);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(58),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "*"
            SlotId(59) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "*"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "*"
            SlotId(60) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(61));
            }
            //Symbol : "{" Layout Symbol . Layout Symbol Layout "}" Layout "*"
            SlotId(61) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "*"
                        let next_slot_id = SlotId(62);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(61),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "*"
            SlotId(62) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(63));
            }
            //Symbol : "{" Layout Symbol Layout Symbol . Layout "}" Layout "*"
            SlotId(63) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "*"
                        let next_slot_id = SlotId(64);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(63),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "*"
            SlotId(64) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "*"
                        let next_slot_id = SlotId(65);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(64),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "*"
            SlotId(65) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "*"
                        let next_slot_id = SlotId(66);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(65),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "*"
            SlotId(66) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "*".
                        let next_slot_id = SlotId(67);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(66),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "*".
            SlotId(67) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(67);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(68) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "+"
                        let next_slot_id = SlotId(69);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(68),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(69) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "+"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(70) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(71));
            }
            //Symbol : "{" Layout Symbol . Layout Symbol Layout "}" Layout "+"
            SlotId(71) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "+"
                        let next_slot_id = SlotId(72);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(71),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "+"
            SlotId(72) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(73));
            }
            //Symbol : "{" Layout Symbol Layout Symbol . Layout "}" Layout "+"
            SlotId(73) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "+"
                        let next_slot_id = SlotId(74);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(73),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "+"
            SlotId(74) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "+"
                        let next_slot_id = SlotId(75);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(74),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "+"
            SlotId(75) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "+"
                        let next_slot_id = SlotId(76);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(75),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "+"
            SlotId(76) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "+".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(76),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "+".
            SlotId(77) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(77);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . "(" Layout Alternative_Plus_6 Layout ")"
            SlotId(78) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Symbol : "(" . Layout Alternative_Plus_6 Layout ")"
                        let next_slot_id = SlotId(79);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(78),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Layout Alternative_Plus_6 Layout ")"
            SlotId(79) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "(" Layout . Alternative_Plus_6 Layout ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(79),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout . Alternative_Plus_6 Layout ")"
            SlotId(80) => {
                self.create(NonterminalId(25), result, gss_node_id, SlotId(81));
            }
            //Symbol : "(" Layout Alternative_Plus_6 . Layout ")"
            SlotId(81) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol : "(" Layout Alternative_Plus_6 Layout . ")"
                        let next_slot_id = SlotId(82);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(81),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Alternative_Plus_6 Layout . ")"
            SlotId(82) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Symbol : "(" Layout Alternative_Plus_6 Layout ")".
                        let next_slot_id = SlotId(83);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(82),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Alternative_Plus_6 Layout ")".
            SlotId(83) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(83);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . Identifier
            SlotId(84) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Symbol : Identifier.
                        let next_slot_id = SlotId(85);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(84),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Identifier.
            SlotId(85) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(85);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex : . Regex Layout "+"
            SlotId(86) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(87));
            }
            //Regex : Regex . Layout "+"
            SlotId(87) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : Regex Layout . "+"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex : Regex Layout . "+"
            SlotId(88) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Regex : Regex Layout "+".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(88),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "+".
            SlotId(89) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(89);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex : . Regex Layout "*"
            SlotId(90) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(91));
            }
            //Regex : Regex . Layout "*"
            SlotId(91) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : Regex Layout . "*"
                        let next_slot_id = SlotId(92);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(91),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout . "*"
            SlotId(92) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Regex : Regex Layout "*".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(92),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "*".
            SlotId(93) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(93);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex : . Regex Layout "?"
            SlotId(94) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(95));
            }
            //Regex : Regex . Layout "?"
            SlotId(95) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : Regex Layout . "?"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(95),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout . "?"
            SlotId(96) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Regex : Regex Layout "?".
                        let next_slot_id = SlotId(97);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"?\"",
                            i,
                            SlotId(96),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "?".
            SlotId(97) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(97);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex : . "(" Layout Regex Layout Regex_Plus_8 Layout ")"
            SlotId(98) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Regex : "(" . Layout Regex Layout Regex_Plus_8 Layout ")"
                        let next_slot_id = SlotId(99);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(98),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout Regex Layout Regex_Plus_8 Layout ")"
            SlotId(99) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : "(" Layout . Regex Layout Regex_Plus_8 Layout ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex : "(" Layout . Regex Layout Regex_Plus_8 Layout ")"
            SlotId(100) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(101));
            }
            //Regex : "(" Layout Regex . Layout Regex_Plus_8 Layout ")"
            SlotId(101) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : "(" Layout Regex Layout . Regex_Plus_8 Layout ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex : "(" Layout Regex Layout . Regex_Plus_8 Layout ")"
            SlotId(102) => {
                self.create(NonterminalId(31), result, gss_node_id, SlotId(103));
            }
            //Regex : "(" Layout Regex Layout Regex_Plus_8 . Layout ")"
            SlotId(103) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : "(" Layout Regex Layout Regex_Plus_8 Layout . ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex : "(" Layout Regex Layout Regex_Plus_8 Layout . ")"
            SlotId(104) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Regex : "(" Layout Regex Layout Regex_Plus_8 Layout ")".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(104),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout Regex Layout Regex_Plus_8 Layout ")".
            SlotId(105) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(105);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex : . "(" Layout RegexRule_Plus_4 Layout ")"
            SlotId(106) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Regex : "(" . Layout RegexRule_Plus_4 Layout ")"
                        let next_slot_id = SlotId(107);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(106),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout RegexRule_Plus_4 Layout ")"
            SlotId(107) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : "(" Layout . RegexRule_Plus_4 Layout ")"
                        let next_slot_id = SlotId(108);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(107),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout . RegexRule_Plus_4 Layout ")"
            SlotId(108) => {
                self.create(NonterminalId(20), result, gss_node_id, SlotId(109));
            }
            //Regex : "(" Layout RegexRule_Plus_4 . Layout ")"
            SlotId(109) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : "(" Layout RegexRule_Plus_4 Layout . ")"
                        let next_slot_id = SlotId(110);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(109),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_4 Layout . ")"
            SlotId(110) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Regex : "(" Layout RegexRule_Plus_4 Layout ")".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(110),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_4 Layout ")".
            SlotId(111) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(111);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex : . CharClass
            SlotId(112) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(113));
            }
            //Regex : CharClass.
            SlotId(113) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(113);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex : . """ Layout Char Layout """
            SlotId(114) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Regex : """ . Layout Char Layout """
                        let next_slot_id = SlotId(115);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(114),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ . Layout Char Layout """
            SlotId(115) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : """ Layout . Char Layout """
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(115),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout . Char Layout """
            SlotId(116) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Regex : """ Layout Char . Layout """
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(116),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout Char . Layout """
            SlotId(117) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : """ Layout Char Layout . """
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(117),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout Char Layout . """
            SlotId(118) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Regex : """ Layout Char Layout """.
                        let next_slot_id = SlotId(119);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(118),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout Char Layout """.
            SlotId(119) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(119);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //CharClass : . CharClass_Opt_6 Layout "[" Layout CharClass_Plus_9 Layout "]"
            SlotId(120) => {
                self.create(NonterminalId(32), result, gss_node_id, SlotId(121));
            }
            //CharClass : CharClass_Opt_6 . Layout "[" Layout CharClass_Plus_9 Layout "]"
            SlotId(121) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //CharClass : CharClass_Opt_6 Layout . "[" Layout CharClass_Plus_9 Layout "]"
                        let next_slot_id = SlotId(122);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(121),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_6 Layout . "[" Layout CharClass_Plus_9 Layout "]"
            SlotId(122) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"[\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"[\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //CharClass : CharClass_Opt_6 Layout "[" . Layout CharClass_Plus_9 Layout "]"
                        let next_slot_id = SlotId(123);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"[\"",
                            i,
                            SlotId(122),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_6 Layout "[" . Layout CharClass_Plus_9 Layout "]"
            SlotId(123) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //CharClass : CharClass_Opt_6 Layout "[" Layout . CharClass_Plus_9 Layout "]"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //CharClass : CharClass_Opt_6 Layout "[" Layout . CharClass_Plus_9 Layout "]"
            SlotId(124) => {
                self.create(NonterminalId(34), result, gss_node_id, SlotId(125));
            }
            //CharClass : CharClass_Opt_6 Layout "[" Layout CharClass_Plus_9 . Layout "]"
            SlotId(125) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //CharClass : CharClass_Opt_6 Layout "[" Layout CharClass_Plus_9 Layout . "]"
                        let next_slot_id = SlotId(126);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(125),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_6 Layout "[" Layout CharClass_Plus_9 Layout . "]"
            SlotId(126) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"]\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"]\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //CharClass : CharClass_Opt_6 Layout "[" Layout CharClass_Plus_9 Layout "]".
                        let next_slot_id = SlotId(127);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"]\"",
                            i,
                            SlotId(126),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_6 Layout "[" Layout CharClass_Plus_9 Layout "]".
            SlotId(127) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(127);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Range : . RangeChar Layout "-" Layout RangeChar
            SlotId(128) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Range : RangeChar . Layout "-" Layout RangeChar
                        let next_slot_id = SlotId(129);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(128),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar . Layout "-" Layout RangeChar
            SlotId(129) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Range : RangeChar Layout . "-" Layout RangeChar
                        let next_slot_id = SlotId(130);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(129),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar Layout . "-" Layout RangeChar
            SlotId(130) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"-\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"-\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Range : RangeChar Layout "-" . Layout RangeChar
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"-\"",
                            i,
                            SlotId(130),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar Layout "-" . Layout RangeChar
            SlotId(131) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Range : RangeChar Layout "-" Layout . RangeChar
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(131),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar Layout "-" Layout . RangeChar
            SlotId(132) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Range : RangeChar Layout "-" Layout RangeChar.
                        let next_slot_id = SlotId(133);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(132),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar Layout "-" Layout RangeChar.
            SlotId(133) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(133);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Grammar_Plus_0 : . Grammar_Plus_0 Layout SyntaxRule
            SlotId(134) => {
                self.create(NonterminalId(10), result, gss_node_id, SlotId(135));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 . Layout SyntaxRule
            SlotId(135) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
                        let next_slot_id = SlotId(136);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(135),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
            SlotId(136) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(137));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout SyntaxRule.
            SlotId(137) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(137);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Grammar_Plus_0 : . SyntaxRule
            SlotId(138) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(139));
            }
            //Grammar_Plus_0 : SyntaxRule.
            SlotId(139) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(139);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Grammar_Opt_0 : . Grammar_Plus_0
            SlotId(140) => {
                self.create(NonterminalId(10), result, gss_node_id, SlotId(141));
            }
            //Grammar_Opt_0 : Grammar_Plus_0.
            SlotId(141) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(141);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Grammar_Opt_0 : .
            SlotId(142) => {
                let end_slot_id = SlotId(142);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(23), input_index, input_index);
                let nonterminal_id = NonterminalId(11);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Grammar_Star_0 : . Grammar_Opt_0
            SlotId(143) => {
                self.create(NonterminalId(11), result, gss_node_id, SlotId(144));
            }
            //Grammar_Star_0 : Grammar_Opt_0.
            SlotId(144) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
                let end_slot_id = SlotId(144);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Grammar_Opt_1 : . RegexBlock
            SlotId(145) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(146));
            }
            //Grammar_Opt_1 : RegexBlock.
            SlotId(146) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
                let end_slot_id = SlotId(146);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Grammar_Opt_1 : .
            SlotId(147) => {
                let end_slot_id = SlotId(147);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(23), input_index, input_index);
                let nonterminal_id = NonterminalId(13);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //SyntaxRule_Plus_1 : . SyntaxRule_Plus_1 Layout ">" Layout PriorityLevel
            SlotId(148) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(149));
            }
            //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 . Layout ">" Layout PriorityLevel
            SlotId(149) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 Layout . ">" Layout PriorityLevel
                        let next_slot_id = SlotId(150);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(149),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 Layout . ">" Layout PriorityLevel
            SlotId(150) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(7), i, j);
                        //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 Layout ">" . Layout PriorityLevel
                        let next_slot_id = SlotId(151);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\">\"",
                            i,
                            SlotId(150),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 Layout ">" . Layout PriorityLevel
            SlotId(151) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 Layout ">" Layout . PriorityLevel
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 Layout ">" Layout . PriorityLevel
            SlotId(152) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(153));
            }
            //SyntaxRule_Plus_1 : SyntaxRule_Plus_1 Layout ">" Layout PriorityLevel.
            SlotId(153) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(153);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //SyntaxRule_Plus_1 : . PriorityLevel
            SlotId(154) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(155));
            }
            //SyntaxRule_Plus_1 : PriorityLevel.
            SlotId(155) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(155);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //SyntaxRule_Opt_2 : . SyntaxRule_Plus_1
            SlotId(156) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(157));
            }
            //SyntaxRule_Opt_2 : SyntaxRule_Plus_1.
            SlotId(157) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
                let end_slot_id = SlotId(157);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //SyntaxRule_Opt_2 : .
            SlotId(158) => {
                let end_slot_id = SlotId(158);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(23), input_index, input_index);
                let nonterminal_id = NonterminalId(15);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //SyntaxRule_Star_1 : . SyntaxRule_Opt_2
            SlotId(159) => {
                self.create(NonterminalId(15), result, gss_node_id, SlotId(160));
            }
            //SyntaxRule_Star_1 : SyntaxRule_Opt_2.
            SlotId(160) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
                let end_slot_id = SlotId(160);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexBlock_Plus_2 : . RegexBlock_Plus_2 Layout RegexRule
            SlotId(161) => {
                self.create(NonterminalId(17), result, gss_node_id, SlotId(162));
            }
            //RegexBlock_Plus_2 : RegexBlock_Plus_2 . Layout RegexRule
            SlotId(162) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexBlock_Plus_2 : RegexBlock_Plus_2 Layout . RegexRule
                        let next_slot_id = SlotId(163);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(162),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock_Plus_2 : RegexBlock_Plus_2 Layout . RegexRule
            SlotId(163) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(164));
            }
            //RegexBlock_Plus_2 : RegexBlock_Plus_2 Layout RegexRule.
            SlotId(164) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(164);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexBlock_Plus_2 : . RegexRule
            SlotId(165) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(166));
            }
            //RegexBlock_Plus_2 : RegexRule.
            SlotId(166) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(166);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexBlock_Opt_3 : . RegexBlock_Plus_2
            SlotId(167) => {
                self.create(NonterminalId(17), result, gss_node_id, SlotId(168));
            }
            //RegexBlock_Opt_3 : RegexBlock_Plus_2.
            SlotId(168) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
                let end_slot_id = SlotId(168);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexBlock_Opt_3 : .
            SlotId(169) => {
                let end_slot_id = SlotId(169);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(23), input_index, input_index);
                let nonterminal_id = NonterminalId(18);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexBlock_Star_2 : . RegexBlock_Opt_3
            SlotId(170) => {
                self.create(NonterminalId(18), result, gss_node_id, SlotId(171));
            }
            //RegexBlock_Star_2 : RegexBlock_Opt_3.
            SlotId(171) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(171);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout Regex
            SlotId(172) => {
                self.create(NonterminalId(20), result, gss_node_id, SlotId(173));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 . Layout Regex
            SlotId(173) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . Regex
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(173),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . Regex
            SlotId(174) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(175));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout Regex.
            SlotId(175) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
                let end_slot_id = SlotId(175);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexRule_Plus_4 : . Regex
            SlotId(176) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(177));
            }
            //RegexRule_Plus_4 : Regex.
            SlotId(177) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
                let end_slot_id = SlotId(177);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexRule_Plus_3 : . RegexRule_Plus_3 Layout "|" Layout RegexRule_Plus_4
            SlotId(178) => {
                self.create(NonterminalId(21), result, gss_node_id, SlotId(179));
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 . Layout "|" Layout RegexRule_Plus_4
            SlotId(179) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexRule_Plus_3 : RegexRule_Plus_3 Layout . "|" Layout RegexRule_Plus_4
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout . "|" Layout RegexRule_Plus_4
            SlotId(180) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" . Layout RegexRule_Plus_4
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(180),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" . Layout RegexRule_Plus_4
            SlotId(181) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" Layout . RegexRule_Plus_4
                        let next_slot_id = SlotId(182);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(181),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" Layout . RegexRule_Plus_4
            SlotId(182) => {
                self.create(NonterminalId(20), result, gss_node_id, SlotId(183));
            }
            //RegexRule_Plus_3 : RegexRule_Plus_3 Layout "|" Layout RegexRule_Plus_4.
            SlotId(183) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
                let end_slot_id = SlotId(183);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //RegexRule_Plus_3 : . RegexRule_Plus_4
            SlotId(184) => {
                self.create(NonterminalId(20), result, gss_node_id, SlotId(185));
            }
            //RegexRule_Plus_3 : RegexRule_Plus_4.
            SlotId(185) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
                let end_slot_id = SlotId(185);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //PriorityLevel_Plus_5 : . PriorityLevel_Plus_5 Layout "|" Layout Alternative
            SlotId(186) => {
                self.create(NonterminalId(22), result, gss_node_id, SlotId(187));
            }
            //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 . Layout "|" Layout Alternative
            SlotId(187) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 Layout . "|" Layout Alternative
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 Layout . "|" Layout Alternative
            SlotId(188) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 Layout "|" . Layout Alternative
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(188),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 Layout "|" . Layout Alternative
            SlotId(189) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 Layout "|" Layout . Alternative
                        let next_slot_id = SlotId(190);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(189),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 Layout "|" Layout . Alternative
            SlotId(190) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(191));
            }
            //PriorityLevel_Plus_5 : PriorityLevel_Plus_5 Layout "|" Layout Alternative.
            SlotId(191) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
                let end_slot_id = SlotId(191);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //PriorityLevel_Plus_5 : . Alternative
            SlotId(192) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(193));
            }
            //PriorityLevel_Plus_5 : Alternative.
            SlotId(193) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
                let end_slot_id = SlotId(193);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //PriorityLevel_Opt_4 : . PriorityLevel_Plus_5
            SlotId(194) => {
                self.create(NonterminalId(22), result, gss_node_id, SlotId(195));
            }
            //PriorityLevel_Opt_4 : PriorityLevel_Plus_5.
            SlotId(195) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(195);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //PriorityLevel_Opt_4 : .
            SlotId(196) => {
                let end_slot_id = SlotId(196);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(23), input_index, input_index);
                let nonterminal_id = NonterminalId(23);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //PriorityLevel_Star_3 : . PriorityLevel_Opt_4
            SlotId(197) => {
                self.create(NonterminalId(23), result, gss_node_id, SlotId(198));
            }
            //PriorityLevel_Star_3 : PriorityLevel_Opt_4.
            SlotId(198) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
                let end_slot_id = SlotId(198);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative_Plus_6 : . Alternative_Plus_6 Layout Symbol
            SlotId(199) => {
                self.create(NonterminalId(25), result, gss_node_id, SlotId(200));
            }
            //Alternative_Plus_6 : Alternative_Plus_6 . Layout Symbol
            SlotId(200) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Alternative_Plus_6 : Alternative_Plus_6 Layout . Symbol
                        let next_slot_id = SlotId(201);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(200),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Plus_6 : Alternative_Plus_6 Layout . Symbol
            SlotId(201) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(202));
            }
            //Alternative_Plus_6 : Alternative_Plus_6 Layout Symbol.
            SlotId(202) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(202);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative_Plus_6 : . Symbol
            SlotId(203) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(204));
            }
            //Alternative_Plus_6 : Symbol.
            SlotId(204) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(204);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative_Opt_5 : . Alternative_Plus_6
            SlotId(205) => {
                self.create(NonterminalId(25), result, gss_node_id, SlotId(206));
            }
            //Alternative_Opt_5 : Alternative_Plus_6.
            SlotId(206) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
                let end_slot_id = SlotId(206);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative_Opt_5 : .
            SlotId(207) => {
                let end_slot_id = SlotId(207);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(23), input_index, input_index);
                let nonterminal_id = NonterminalId(26);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative_Star_4 : . Alternative_Opt_5
            SlotId(208) => {
                self.create(NonterminalId(26), result, gss_node_id, SlotId(209));
            }
            //Alternative_Star_4 : Alternative_Opt_5.
            SlotId(209) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
                let end_slot_id = SlotId(209);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol_Group_0 : . "|" Layout Symbol
            SlotId(210) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Symbol_Group_0 : "|" . Layout Symbol
                        let next_slot_id = SlotId(211);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(210),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" . Layout Symbol
            SlotId(211) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol_Group_0 : "|" Layout . Symbol
                        let next_slot_id = SlotId(212);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(211),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" Layout . Symbol
            SlotId(212) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(213));
            }
            //Symbol_Group_0 : "|" Layout Symbol.
            SlotId(213) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
                let end_slot_id = SlotId(213);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol_Plus_7 : . Symbol_Plus_7 Layout Symbol_Group_0
            SlotId(214) => {
                self.create(NonterminalId(29), result, gss_node_id, SlotId(215));
            }
            //Symbol_Plus_7 : Symbol_Plus_7 . Layout Symbol_Group_0
            SlotId(215) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol_Plus_7 : Symbol_Plus_7 Layout . Symbol_Group_0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol_Plus_7 : Symbol_Plus_7 Layout . Symbol_Group_0
            SlotId(216) => {
                self.create(NonterminalId(28), result, gss_node_id, SlotId(217));
            }
            //Symbol_Plus_7 : Symbol_Plus_7 Layout Symbol_Group_0.
            SlotId(217) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
                let end_slot_id = SlotId(217);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol_Plus_7 : . Symbol_Group_0
            SlotId(218) => {
                self.create(NonterminalId(28), result, gss_node_id, SlotId(219));
            }
            //Symbol_Plus_7 : Symbol_Group_0.
            SlotId(219) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
                let end_slot_id = SlotId(219);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex_Group_1 : . "|" Layout Regex
            SlotId(220) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Regex_Group_1 : "|" . Layout Regex
                        let next_slot_id = SlotId(221);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(220),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_1 : "|" . Layout Regex
            SlotId(221) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex_Group_1 : "|" Layout . Regex
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex_Group_1 : "|" Layout . Regex
            SlotId(222) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(223));
            }
            //Regex_Group_1 : "|" Layout Regex.
            SlotId(223) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(30);
                let end_slot_id = SlotId(223);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex_Plus_8 : . Regex_Plus_8 Layout Regex_Group_1
            SlotId(224) => {
                self.create(NonterminalId(31), result, gss_node_id, SlotId(225));
            }
            //Regex_Plus_8 : Regex_Plus_8 . Layout Regex_Group_1
            SlotId(225) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex_Plus_8 : Regex_Plus_8 Layout . Regex_Group_1
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex_Plus_8 : Regex_Plus_8 Layout . Regex_Group_1
            SlotId(226) => {
                self.create(NonterminalId(30), result, gss_node_id, SlotId(227));
            }
            //Regex_Plus_8 : Regex_Plus_8 Layout Regex_Group_1.
            SlotId(227) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
                let end_slot_id = SlotId(227);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Regex_Plus_8 : . Regex_Group_1
            SlotId(228) => {
                self.create(NonterminalId(30), result, gss_node_id, SlotId(229));
            }
            //Regex_Plus_8 : Regex_Group_1.
            SlotId(229) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
                let end_slot_id = SlotId(229);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //CharClass_Opt_6 : . "!"
            SlotId(230) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //CharClass_Opt_6 : "!".
                        let next_slot_id = SlotId(231);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(230),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Opt_6 : "!".
            SlotId(231) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(32);
                let end_slot_id = SlotId(231);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //CharClass_Opt_6 : .
            SlotId(232) => {
                let end_slot_id = SlotId(232);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(23), input_index, input_index);
                let nonterminal_id = NonterminalId(32);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    input_index,
                    input_index,
                    epsilon_node_id,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //CharClass_Alt_0 : . Range
            SlotId(233) => {
                self.create(NonterminalId(9), result, gss_node_id, SlotId(234));
            }
            //CharClass_Alt_0 : Range.
            SlotId(234) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
                let end_slot_id = SlotId(234);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //CharClass_Alt_0 : . RangeChar
            SlotId(235) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //CharClass_Alt_0 : RangeChar.
                        let next_slot_id = SlotId(236);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(235),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Alt_0 : RangeChar.
            SlotId(236) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
                let end_slot_id = SlotId(236);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //CharClass_Plus_9 : . CharClass_Plus_9 Layout CharClass_Alt_0
            SlotId(237) => {
                self.create(NonterminalId(34), result, gss_node_id, SlotId(238));
            }
            //CharClass_Plus_9 : CharClass_Plus_9 . Layout CharClass_Alt_0
            SlotId(238) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //CharClass_Plus_9 : CharClass_Plus_9 Layout . CharClass_Alt_0
                        let next_slot_id = SlotId(239);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(238),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Plus_9 : CharClass_Plus_9 Layout . CharClass_Alt_0
            SlotId(239) => {
                self.create(NonterminalId(33), result, gss_node_id, SlotId(240));
            }
            //CharClass_Plus_9 : CharClass_Plus_9 Layout CharClass_Alt_0.
            SlotId(240) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
                let end_slot_id = SlotId(240);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //CharClass_Plus_9 : . CharClass_Alt_0
            SlotId(241) => {
                self.create(NonterminalId(33), result, gss_node_id, SlotId(242));
            }
            //CharClass_Plus_9 : CharClass_Alt_0.
            SlotId(242) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
                let end_slot_id = SlotId(242);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartGrammar : . Layout Grammar Layout
            SlotId(243) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartGrammar : Layout . Grammar Layout
                        let next_slot_id = SlotId(244);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(243),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout . Grammar Layout
            SlotId(244) => {
                self.create(NonterminalId(0), result, gss_node_id, SlotId(245));
            }
            //StartGrammar : Layout Grammar . Layout
            SlotId(245) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartGrammar : Layout Grammar Layout.
                        let next_slot_id = SlotId(246);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(245),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout Grammar Layout.
            SlotId(246) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(35);
                let end_slot_id = SlotId(246);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartSyntaxRule : . Layout SyntaxRule Layout
            SlotId(247) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartSyntaxRule : Layout . SyntaxRule Layout
                        let next_slot_id = SlotId(248);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(247),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout . SyntaxRule Layout
            SlotId(248) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(249));
            }
            //StartSyntaxRule : Layout SyntaxRule . Layout
            SlotId(249) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartSyntaxRule : Layout SyntaxRule Layout.
                        let next_slot_id = SlotId(250);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(249),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout SyntaxRule Layout.
            SlotId(250) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(36);
                let end_slot_id = SlotId(250);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartRegexBlock : . Layout RegexBlock Layout
            SlotId(251) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRegexBlock : Layout . RegexBlock Layout
                        let next_slot_id = SlotId(252);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(251),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexBlock : Layout . RegexBlock Layout
            SlotId(252) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(253));
            }
            //StartRegexBlock : Layout RegexBlock . Layout
            SlotId(253) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRegexBlock : Layout RegexBlock Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //StartRegexBlock : Layout RegexBlock Layout.
            SlotId(254) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(37);
                let end_slot_id = SlotId(254);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartRegexRule : . Layout RegexRule Layout
            SlotId(255) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRegexRule : Layout . RegexRule Layout
                        let next_slot_id = SlotId(256);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(255),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout . RegexRule Layout
            SlotId(256) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(257));
            }
            //StartRegexRule : Layout RegexRule . Layout
            SlotId(257) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRegexRule : Layout RegexRule Layout.
                        let next_slot_id = SlotId(258);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(257),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout RegexRule Layout.
            SlotId(258) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(38);
                let end_slot_id = SlotId(258);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartPriorityLevel : . Layout PriorityLevel Layout
            SlotId(259) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartPriorityLevel : Layout . PriorityLevel Layout
                        let next_slot_id = SlotId(260);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(259),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout . PriorityLevel Layout
            SlotId(260) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(261));
            }
            //StartPriorityLevel : Layout PriorityLevel . Layout
            SlotId(261) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartPriorityLevel : Layout PriorityLevel Layout.
                        let next_slot_id = SlotId(262);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(261),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout PriorityLevel Layout.
            SlotId(262) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(39);
                let end_slot_id = SlotId(262);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartAlternative : . Layout Alternative Layout
            SlotId(263) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartAlternative : Layout . Alternative Layout
                        let next_slot_id = SlotId(264);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(263),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout . Alternative Layout
            SlotId(264) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(265));
            }
            //StartAlternative : Layout Alternative . Layout
            SlotId(265) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartAlternative : Layout Alternative Layout.
                        let next_slot_id = SlotId(266);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(265),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout Alternative Layout.
            SlotId(266) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(40);
                let end_slot_id = SlotId(266);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartSymbol : . Layout Symbol Layout
            SlotId(267) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartSymbol : Layout . Symbol Layout
                        let next_slot_id = SlotId(268);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //StartSymbol : Layout . Symbol Layout
            SlotId(268) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(269));
            }
            //StartSymbol : Layout Symbol . Layout
            SlotId(269) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartSymbol : Layout Symbol Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //StartSymbol : Layout Symbol Layout.
            SlotId(270) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(41);
                let end_slot_id = SlotId(270);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartRegex : . Layout Regex Layout
            SlotId(271) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRegex : Layout . Regex Layout
                        let next_slot_id = SlotId(272);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(271),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout . Regex Layout
            SlotId(272) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(273));
            }
            //StartRegex : Layout Regex . Layout
            SlotId(273) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRegex : Layout Regex Layout.
                        let next_slot_id = SlotId(274);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(273),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout Regex Layout.
            SlotId(274) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(42);
                let end_slot_id = SlotId(274);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartCharClass : . Layout CharClass Layout
            SlotId(275) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartCharClass : Layout . CharClass Layout
                        let next_slot_id = SlotId(276);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(275),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout . CharClass Layout
            SlotId(276) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(277));
            }
            //StartCharClass : Layout CharClass . Layout
            SlotId(277) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartCharClass : Layout CharClass Layout.
                        let next_slot_id = SlotId(278);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(277),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout CharClass Layout.
            SlotId(278) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
                let end_slot_id = SlotId(278);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //StartRange : . Layout Range Layout
            SlotId(279) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRange : Layout . Range Layout
                        let next_slot_id = SlotId(280);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(279),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout . Range Layout
            SlotId(280) => {
                self.create(NonterminalId(9), result, gss_node_id, SlotId(281));
            }
            //StartRange : Layout Range . Layout
            SlotId(281) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //StartRange : Layout Range Layout.
                        let next_slot_id = SlotId(282);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(281),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout Range Layout.
            SlotId(282) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(44);
                let end_slot_id = SlotId(282);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
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
    ) {
        match nonterminal_id {
            //Grammar
            NonterminalId(0) => {
                //Grammar : . "grammar" Layout Identifier Layout Grammar_Star_0 Layout Grammar_Opt_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule
            NonterminalId(1) => {
                //SyntaxRule : . Identifier Layout "=" Layout SyntaxRule_Star_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(8),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock
            NonterminalId(2) => {
                //RegexBlock : . "regex" Layout "{" Layout RegexBlock_Star_2 Layout "}"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexRule
            NonterminalId(3) => {
                //RegexRule : . Identifier Layout "=" Layout RegexRule_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(22),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel
            NonterminalId(4) => {
                //PriorityLevel : . PriorityLevel_Star_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(28),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative
            NonterminalId(5) => {
                //Alternative : . Alternative_Star_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(30),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol
            NonterminalId(6) => {
                //Symbol : . Symbol Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(32),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Symbol Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(36),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Symbol Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(40),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Layout Symbol Layout Symbol_Plus_7 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(44),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . """ Layout String Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(52),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(58),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(68),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Layout Alternative_Plus_6 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(78),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(84),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex
            NonterminalId(7) => {
                //Regex : . Regex Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(86),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(90),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(94),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . "(" Layout Regex Layout Regex_Plus_8 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(98),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . "(" Layout RegexRule_Plus_4 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(106),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . CharClass
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(112),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . """ Layout Char Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(114),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass
            NonterminalId(8) => {
                //CharClass : . CharClass_Opt_6 Layout "[" Layout CharClass_Plus_9 Layout "]"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(120),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Range
            NonterminalId(9) => {
                //Range : . RangeChar Layout "-" Layout RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(128),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Plus_0
            NonterminalId(10) => {
                //Grammar_Plus_0 : . Grammar_Plus_0 Layout SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(134),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Plus_0 : . SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(138),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Opt_0
            NonterminalId(11) => {
                //Grammar_Opt_0 : . Grammar_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(140),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(142),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Star_0
            NonterminalId(12) => {
                //Grammar_Star_0 : . Grammar_Opt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(143),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Opt_1
            NonterminalId(13) => {
                //Grammar_Opt_1 : . RegexBlock
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(145),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(147),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule_Plus_1
            NonterminalId(14) => {
                //SyntaxRule_Plus_1 : . SyntaxRule_Plus_1 Layout ">" Layout PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(148),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //SyntaxRule_Plus_1 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(154),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule_Opt_2
            NonterminalId(15) => {
                //SyntaxRule_Opt_2 : . SyntaxRule_Plus_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(156),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //SyntaxRule_Opt_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(158),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule_Star_1
            NonterminalId(16) => {
                //SyntaxRule_Star_1 : . SyntaxRule_Opt_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(159),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock_Plus_2
            NonterminalId(17) => {
                //RegexBlock_Plus_2 : . RegexBlock_Plus_2 Layout RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(161),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexBlock_Plus_2 : . RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(165),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock_Opt_3
            NonterminalId(18) => {
                //RegexBlock_Opt_3 : . RegexBlock_Plus_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(167),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexBlock_Opt_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(169),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock_Star_2
            NonterminalId(19) => {
                //RegexBlock_Star_2 : . RegexBlock_Opt_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(170),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexRule_Plus_4
            NonterminalId(20) => {
                //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(172),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexRule_Plus_4 : . Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(176),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexRule_Plus_3
            NonterminalId(21) => {
                //RegexRule_Plus_3 : . RegexRule_Plus_3 Layout "|" Layout RegexRule_Plus_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(178),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexRule_Plus_3 : . RegexRule_Plus_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(184),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Plus_5
            NonterminalId(22) => {
                //PriorityLevel_Plus_5 : . PriorityLevel_Plus_5 Layout "|" Layout Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(186),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Plus_5 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(192),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Opt_4
            NonterminalId(23) => {
                //PriorityLevel_Opt_4 : . PriorityLevel_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(194),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Opt_4 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(196),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Star_3
            NonterminalId(24) => {
                //PriorityLevel_Star_3 : . PriorityLevel_Opt_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(197),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Plus_6
            NonterminalId(25) => {
                //Alternative_Plus_6 : . Alternative_Plus_6 Layout Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(199),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Plus_6 : . Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(203),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Opt_5
            NonterminalId(26) => {
                //Alternative_Opt_5 : . Alternative_Plus_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(205),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Opt_5 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(207),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Star_4
            NonterminalId(27) => {
                //Alternative_Star_4 : . Alternative_Opt_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(208),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol_Group_0
            NonterminalId(28) => {
                //Symbol_Group_0 : . "|" Layout Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(210),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol_Plus_7
            NonterminalId(29) => {
                //Symbol_Plus_7 : . Symbol_Plus_7 Layout Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(214),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol_Plus_7 : . Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(218),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Group_1
            NonterminalId(30) => {
                //Regex_Group_1 : . "|" Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(220),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Plus_8
            NonterminalId(31) => {
                //Regex_Plus_8 : . Regex_Plus_8 Layout Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(224),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex_Plus_8 : . Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(228),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Opt_6
            NonterminalId(32) => {
                //CharClass_Opt_6 : . "!"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(230),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Opt_6 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(232),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Alt_0
            NonterminalId(33) => {
                //CharClass_Alt_0 : . Range
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(233),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Alt_0 : . RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(235),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Plus_9
            NonterminalId(34) => {
                //CharClass_Plus_9 : . CharClass_Plus_9 Layout CharClass_Alt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(237),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Plus_9 : . CharClass_Alt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(241),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartGrammar
            NonterminalId(35) => {
                //StartGrammar : . Layout Grammar Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(243),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartSyntaxRule
            NonterminalId(36) => {
                //StartSyntaxRule : . Layout SyntaxRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(247),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRegexBlock
            NonterminalId(37) => {
                //StartRegexBlock : . Layout RegexBlock Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(251),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRegexRule
            NonterminalId(38) => {
                //StartRegexRule : . Layout RegexRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(255),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartPriorityLevel
            NonterminalId(39) => {
                //StartPriorityLevel : . Layout PriorityLevel Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(259),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartAlternative
            NonterminalId(40) => {
                //StartAlternative : . Layout Alternative Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(263),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartSymbol
            NonterminalId(41) => {
                //StartSymbol : . Layout Symbol Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(267),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRegex
            NonterminalId(42) => {
                //StartRegex : . Layout Regex Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(271),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartCharClass
            NonterminalId(43) => {
                //StartCharClass : . Layout CharClass Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(275),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRange
            NonterminalId(44) => {
                //StartRange : . Layout Range Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(279),
                    sppf_node_id: None,
                    gss_node_id,
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
}
pub struct IggyParser<'i> {
    start_nonterminal: NonterminalId,
    scanner: IggyScanner<'i>,
    descriptors: Vec<Descriptor>,
    gss_nodes: Vec<GSSNode>,
    //A vector from nonterminal_ids to a tuple (input_index, gss_node_id)
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 45],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 45],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 283],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 24],
    intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
    intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
    nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
    nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
    #[cfg(feature = "debug-trace")]
    pub trace_events: Option<Vec<TraceEvent>>,
}
impl<'i> IggyParser<'i> {
    pub fn new(input: &'i Input, start_nonterminal: NonterminalId) -> Self {
        init_logger();
        Self {
            start_nonterminal,
            scanner: IggyScanner::new(input),
            gss_nodes_index: [const { vec![] }; 45],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 45],
            intermediate_nodes_index: [const { InlineMap::Empty }; 283],
            terminal_nodes_index: [const { InlineMap::Empty }; 24],
            stats: Stats::default(),
            intermediate_nodes_children: vec![],
            intermediate_nodes_children_map: OnceCell::new(),
            nonterminal_nodes_children: vec![],
            nonterminal_nodes_children_map: OnceCell::new(),
            #[cfg(feature = "debug-trace")]
            trace_events: None,
        }
    }
}

