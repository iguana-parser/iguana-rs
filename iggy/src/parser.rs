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
pub const NONTERMINALS: [Nonterminal; 53] = [
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
        display: "SyntaxRule+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Grammar_Opt_1",
        display: "SyntaxRule+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Grammar_Star_0",
        display: "SyntaxRule*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Grammar_Opt_2",
        display: "RegexBlock?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "LayoutDef_Plus_1",
        display: "Identifier+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "LayoutDef_Opt_3",
        display: "Identifier+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "LayoutDef_Star_1",
        display: "Identifier*",
        kind: Some(EbnfKind::Star),
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
        name: "RegexBlock_Plus_3",
        display: "RegexRule+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "RegexBlock_Opt_5",
        display: "RegexRule+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "RegexBlock_Star_3",
        display: "RegexRule*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "RegexRule_Plus_5",
        display: "Regex+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "RegexRule_Plus_4",
        display: "{Regex+ \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "PriorityLevel_Plus_6",
        display: "{Alternative \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_6",
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
        name: "Alternative_Opt_7",
        display: "Symbol+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Alternative_Star_5",
        display: "Symbol*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Opt_8",
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
        name: "Regex_Group_1",
        display: "(\"|\" Regex)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Regex_Plus_9",
        display: "(\"|\" Regex)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "CharClass_Opt_9",
        display: "\"!\"?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "CharClass_Plus_10",
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
        name: "StartRangeElement",
        display: "StartRangeElement",
        kind: None,
    },
    Nonterminal {
        name: "StartRange",
        display: "StartRange",
        kind: None,
    },
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Grammar" => NonterminalId(0), "LayoutDef" => NonterminalId(1), "SyntaxRule" =>
    NonterminalId(2), "RegexBlock" => NonterminalId(3), "RegexRule" => NonterminalId(4),
    "PriorityLevel" => NonterminalId(5), "Alternative" => NonterminalId(6), "Symbol" =>
    NonterminalId(7), "Regex" => NonterminalId(8), "CharClass" => NonterminalId(9),
    "RangeElement" => NonterminalId(10), "Range" => NonterminalId(11), "Grammar_Opt_0" =>
    NonterminalId(12), "Grammar_Plus_0" => NonterminalId(13), "Grammar_Opt_1" =>
    NonterminalId(14), "Grammar_Star_0" => NonterminalId(15), "Grammar_Opt_2" =>
    NonterminalId(16), "LayoutDef_Plus_1" => NonterminalId(17), "LayoutDef_Opt_3" =>
    NonterminalId(18), "LayoutDef_Star_1" => NonterminalId(19), "SyntaxRule_Plus_2" =>
    NonterminalId(20), "SyntaxRule_Opt_4" => NonterminalId(21), "SyntaxRule_Star_2" =>
    NonterminalId(22), "RegexBlock_Plus_3" => NonterminalId(23), "RegexBlock_Opt_5" =>
    NonterminalId(24), "RegexBlock_Star_3" => NonterminalId(25), "RegexRule_Plus_5" =>
    NonterminalId(26), "RegexRule_Plus_4" => NonterminalId(27), "PriorityLevel_Plus_6" =>
    NonterminalId(28), "PriorityLevel_Opt_6" => NonterminalId(29), "PriorityLevel_Star_4"
    => NonterminalId(30), "Alternative_Plus_7" => NonterminalId(31), "Alternative_Opt_7"
    => NonterminalId(32), "Alternative_Star_5" => NonterminalId(33), "Alternative_Opt_8"
    => NonterminalId(34), "Symbol_Group_0" => NonterminalId(35), "Symbol_Plus_8" =>
    NonterminalId(36), "Regex_Group_1" => NonterminalId(37), "Regex_Plus_9" =>
    NonterminalId(38), "CharClass_Opt_9" => NonterminalId(39), "CharClass_Plus_10" =>
    NonterminalId(40), "StartGrammar" => NonterminalId(41), "StartLayoutDef" =>
    NonterminalId(42), "StartSyntaxRule" => NonterminalId(43), "StartRegexBlock" =>
    NonterminalId(44), "StartRegexRule" => NonterminalId(45), "StartPriorityLevel" =>
    NonterminalId(46), "StartAlternative" => NonterminalId(47), "StartSymbol" =>
    NonterminalId(48), "StartRegex" => NonterminalId(49), "StartCharClass" =>
    NonterminalId(50), "StartRangeElement" => NonterminalId(51), "StartRange" =>
    NonterminalId(52)
};
pub const TERMINALS: [Terminal; 27] = [
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
    Terminal { name: "\":\"" },
    Terminal { name: "\"!\"" },
    Terminal { name: "\"[\"" },
    Terminal { name: "\"]\"" },
    Terminal { name: "\"-\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 322] = [
    Slot {
        display_name: "Grammar : . \"grammar\" Layout Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" . Layout Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout . Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier . Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout . LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout LayoutDef? . Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout LayoutDef? Layout . SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout LayoutDef? Layout SyntaxRule* . Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout LayoutDef? Layout SyntaxRule* Layout . RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?.",
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
        display_name: "Symbol : . Identifier Layout \":\" Layout Symbol",
    },
    Slot {
        display_name: "Symbol : Identifier . Layout \":\" Layout Symbol",
    },
    Slot {
        display_name: "Symbol : Identifier Layout . \":\" Layout Symbol",
    },
    Slot {
        display_name: "Symbol : Identifier Layout \":\" . Layout Symbol",
    },
    Slot {
        display_name: "Symbol : Identifier Layout \":\" Layout . Symbol",
    },
    Slot {
        display_name: "Symbol : Identifier Layout \":\" Layout Symbol.",
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
        display_name: "CharClass : . \"!\"? Layout \"[\" Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? . Layout \"[\" Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout . \"[\" Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" . Layout RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout . RangeElement+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout RangeElement+ . Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout RangeElement+ Layout . \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout RangeElement+ Layout \"]\".",
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
        display_name: "LayoutDef? : . LayoutDef",
    },
    Slot {
        display_name: "LayoutDef? : LayoutDef.",
    },
    Slot {
        display_name: "LayoutDef? : .",
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
        display_name: "Label? : . Label",
    },
    Slot {
        display_name: "Label? : Label.",
    },
    Slot {
        display_name: "Label? : .",
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
        display_name: "StartLayoutDef : . Layout LayoutDef Layout",
    },
    Slot {
        display_name: "StartLayoutDef : Layout . LayoutDef Layout",
    },
    Slot {
        display_name: "StartLayoutDef : Layout LayoutDef . Layout",
    },
    Slot {
        display_name: "StartLayoutDef : Layout LayoutDef Layout.",
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
        display_name: "StartRangeElement : . Layout RangeElement Layout",
    },
    Slot {
        display_name: "StartRangeElement : Layout . RangeElement Layout",
    },
    Slot {
        display_name: "StartRangeElement : Layout RangeElement . Layout",
    },
    Slot {
        display_name: "StartRangeElement : Layout RangeElement Layout.",
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
            //Grammar : . "grammar" Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(0) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(6), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(6), i, j);
                        //Grammar : "grammar" . Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" . Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(1) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Grammar : "grammar" Layout . Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout . Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(2) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Grammar : "grammar" Layout Identifier . Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout Identifier . Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(3) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Grammar : "grammar" Layout Identifier Layout . Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout Identifier Layout . Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(4) => {
                self.create_grammar_opt_0(result, gss_node_id, SlotId(5));
            }
            //Grammar : "grammar" Layout Identifier Layout Grammar_Opt_0 . Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(5) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Grammar : "grammar" Layout Identifier Layout Grammar_Opt_0 Layout . Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout Identifier Layout Grammar_Opt_0 Layout . Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(6) => {
                self.create_grammar_star_0(result, gss_node_id, SlotId(7));
            }
            //Grammar : "grammar" Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 . Layout Grammar_Opt_2
            SlotId(7) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Grammar : "grammar" Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout . Grammar_Opt_2
                        let next_slot_id = SlotId(8);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(7),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout . Grammar_Opt_2
            SlotId(8) => {
                self.create_grammar_opt_2(result, gss_node_id, SlotId(9));
            }
            //Grammar : "grammar" Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2.
            SlotId(9) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
                let end_slot_id = SlotId(9);
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
            //LayoutDef : . "layout" Layout LayoutDef_Star_1
            SlotId(10) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"layout\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"layout\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(7), i, j);
                        //LayoutDef : "layout" . Layout LayoutDef_Star_1
                        let next_slot_id = SlotId(11);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"layout\"",
                            i,
                            SlotId(10),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef : "layout" . Layout LayoutDef_Star_1
            SlotId(11) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //LayoutDef : "layout" Layout . LayoutDef_Star_1
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
            //LayoutDef : "layout" Layout . LayoutDef_Star_1
            SlotId(12) => {
                self.create_layout_def_star_1(result, gss_node_id, SlotId(13));
            }
            //LayoutDef : "layout" Layout LayoutDef_Star_1.
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
            //SyntaxRule : . Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(14) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //SyntaxRule : Identifier . Layout "=" Layout SyntaxRule_Star_2
                        let next_slot_id = SlotId(15);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(14),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : Identifier . Layout "=" Layout SyntaxRule_Star_2
            SlotId(15) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //SyntaxRule : Identifier Layout . "=" Layout SyntaxRule_Star_2
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
            //SyntaxRule : Identifier Layout . "=" Layout SyntaxRule_Star_2
            SlotId(16) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //SyntaxRule : Identifier Layout "=" . Layout SyntaxRule_Star_2
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
                            "\"=\"",
                            i,
                            SlotId(16),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : Identifier Layout "=" . Layout SyntaxRule_Star_2
            SlotId(17) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //SyntaxRule : Identifier Layout "=" Layout . SyntaxRule_Star_2
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
            //SyntaxRule : Identifier Layout "=" Layout . SyntaxRule_Star_2
            SlotId(18) => {
                self.create_syntax_rule_star_2(result, gss_node_id, SlotId(19));
            }
            //SyntaxRule : Identifier Layout "=" Layout SyntaxRule_Star_2.
            SlotId(19) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let end_slot_id = SlotId(19);
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
            //RegexBlock : . "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}"
            SlotId(20) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"regex\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"regex\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //RegexBlock : "regex" . Layout "{" Layout RegexBlock_Star_3 Layout "}"
                        let next_slot_id = SlotId(21);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"regex\"",
                            i,
                            SlotId(20),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" . Layout "{" Layout RegexBlock_Star_3 Layout "}"
            SlotId(21) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexBlock : "regex" Layout . "{" Layout RegexBlock_Star_3 Layout "}"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //RegexBlock : "regex" Layout . "{" Layout RegexBlock_Star_3 Layout "}"
            SlotId(22) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //RegexBlock : "regex" Layout "{" . Layout RegexBlock_Star_3 Layout "}"
                        let next_slot_id = SlotId(23);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(22),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" Layout "{" . Layout RegexBlock_Star_3 Layout "}"
            SlotId(23) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexBlock : "regex" Layout "{" Layout . RegexBlock_Star_3 Layout "}"
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
            //RegexBlock : "regex" Layout "{" Layout . RegexBlock_Star_3 Layout "}"
            SlotId(24) => {
                self.create_regex_block_star_3(result, gss_node_id, SlotId(25));
            }
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 . Layout "}"
            SlotId(25) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout . "}"
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
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout . "}"
            SlotId(26) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}".
                        let next_slot_id = SlotId(27);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(26),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}".
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
            //RegexRule : . Identifier Layout "=" Layout RegexRule_Plus_4
            SlotId(28) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //RegexRule : Identifier . Layout "=" Layout RegexRule_Plus_4
                        let next_slot_id = SlotId(29);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(28),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier . Layout "=" Layout RegexRule_Plus_4
            SlotId(29) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexRule : Identifier Layout . "=" Layout RegexRule_Plus_4
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //RegexRule : Identifier Layout . "=" Layout RegexRule_Plus_4
            SlotId(30) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //RegexRule : Identifier Layout "=" . Layout RegexRule_Plus_4
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"=\"",
                            i,
                            SlotId(30),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier Layout "=" . Layout RegexRule_Plus_4
            SlotId(31) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexRule : Identifier Layout "=" Layout . RegexRule_Plus_4
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //RegexRule : Identifier Layout "=" Layout . RegexRule_Plus_4
            SlotId(32) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(33));
            }
            //RegexRule : Identifier Layout "=" Layout RegexRule_Plus_4.
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
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //PriorityLevel : . PriorityLevel_Star_4
            SlotId(34) => {
                self.create_priority_level_star_4(result, gss_node_id, SlotId(35));
            }
            //PriorityLevel : PriorityLevel_Star_4.
            SlotId(35) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //Alternative : . Alternative_Star_5 Layout Alternative_Opt_8
            SlotId(36) => {
                self.create_alternative_star_5(result, gss_node_id, SlotId(37));
            }
            //Alternative : Alternative_Star_5 . Layout Alternative_Opt_8
            SlotId(37) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Alternative : Alternative_Star_5 Layout . Alternative_Opt_8
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
            //Alternative : Alternative_Star_5 Layout . Alternative_Opt_8
            SlotId(38) => {
                self.create_alternative_opt_8(result, gss_node_id, SlotId(39));
            }
            //Alternative : Alternative_Star_5 Layout Alternative_Opt_8.
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
            //Symbol : . Symbol Layout "*"
            SlotId(40) => {
                self.create_symbol(result, gss_node_id, SlotId(41));
            }
            //Symbol : Symbol . Layout "*"
            SlotId(41) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : Symbol Layout . "*"
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
            //Symbol : Symbol Layout . "*"
            SlotId(42) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol : Symbol Layout "*".
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
                            "\"*\"",
                            i,
                            SlotId(42),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "*".
            SlotId(43) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
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
            //Symbol : . Symbol Layout "+"
            SlotId(44) => {
                self.create_symbol(result, gss_node_id, SlotId(45));
            }
            //Symbol : Symbol . Layout "+"
            SlotId(45) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : Symbol Layout . "+"
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
            //Symbol : Symbol Layout . "+"
            SlotId(46) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Symbol : Symbol Layout "+".
                        let next_slot_id = SlotId(47);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(46),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "+".
            SlotId(47) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(47);
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
            SlotId(48) => {
                self.create_symbol(result, gss_node_id, SlotId(49));
            }
            //Symbol : Symbol . Layout "?"
            SlotId(49) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : Symbol Layout . "?"
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
            //Symbol : Symbol Layout . "?"
            SlotId(50) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Symbol : Symbol Layout "?".
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
                            "\"?\"",
                            i,
                            SlotId(50),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "?".
            SlotId(51) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
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
            //Symbol : . "(" Layout Symbol Layout Symbol_Plus_8 Layout ")"
            SlotId(52) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol : "(" . Layout Symbol Layout Symbol_Plus_8 Layout ")"
                        let next_slot_id = SlotId(53);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(52),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Layout Symbol Layout Symbol_Plus_8 Layout ")"
            SlotId(53) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "(" Layout . Symbol Layout Symbol_Plus_8 Layout ")"
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
            //Symbol : "(" Layout . Symbol Layout Symbol_Plus_8 Layout ")"
            SlotId(54) => {
                self.create_symbol(result, gss_node_id, SlotId(55));
            }
            //Symbol : "(" Layout Symbol . Layout Symbol_Plus_8 Layout ")"
            SlotId(55) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "(" Layout Symbol Layout . Symbol_Plus_8 Layout ")"
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
            //Symbol : "(" Layout Symbol Layout . Symbol_Plus_8 Layout ")"
            SlotId(56) => {
                self.create_symbol_plus_8(result, gss_node_id, SlotId(57));
            }
            //Symbol : "(" Layout Symbol Layout Symbol_Plus_8 . Layout ")"
            SlotId(57) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "(" Layout Symbol Layout Symbol_Plus_8 Layout . ")"
                        let next_slot_id = SlotId(58);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(57),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Symbol Layout Symbol_Plus_8 Layout . ")"
            SlotId(58) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Symbol : "(" Layout Symbol Layout Symbol_Plus_8 Layout ")".
                        let next_slot_id = SlotId(59);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(58),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Symbol Layout Symbol_Plus_8 Layout ")".
            SlotId(59) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(59);
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
            SlotId(60) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Symbol : """ . Layout String Layout """
                        let next_slot_id = SlotId(61);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(60),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ . Layout String Layout """
            SlotId(61) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : """ Layout . String Layout """
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
            //Symbol : """ Layout . String Layout """
            SlotId(62) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol : """ Layout String . Layout """
                        let next_slot_id = SlotId(63);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(62),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ Layout String . Layout """
            SlotId(63) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : """ Layout String Layout . """
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
            //Symbol : """ Layout String Layout . """
            SlotId(64) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Symbol : """ Layout String Layout """.
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
                            "\"\"\"",
                            i,
                            SlotId(64),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ Layout String Layout """.
            SlotId(65) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(65);
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
            SlotId(66) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "*"
                        let next_slot_id = SlotId(67);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(66),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "*"
            SlotId(67) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "*"
                        let next_slot_id = SlotId(68);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(67),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "*"
            SlotId(68) => {
                self.create_symbol(result, gss_node_id, SlotId(69));
            }
            //Symbol : "{" Layout Symbol . Layout Symbol Layout "}" Layout "*"
            SlotId(69) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "*"
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
            //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "*"
            SlotId(70) => {
                self.create_symbol(result, gss_node_id, SlotId(71));
            }
            //Symbol : "{" Layout Symbol Layout Symbol . Layout "}" Layout "*"
            SlotId(71) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "*"
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "*"
            SlotId(72) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "*"
                        let next_slot_id = SlotId(73);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(72),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "*"
            SlotId(73) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "*"
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "*"
            SlotId(74) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "*".
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
                            "\"*\"",
                            i,
                            SlotId(74),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "*".
            SlotId(75) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(75);
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
            SlotId(76) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "+"
                        let next_slot_id = SlotId(77);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(76),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(77) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "+"
                        let next_slot_id = SlotId(78);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(77),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(78) => {
                self.create_symbol(result, gss_node_id, SlotId(79));
            }
            //Symbol : "{" Layout Symbol . Layout Symbol Layout "}" Layout "+"
            SlotId(79) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "+"
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
            //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "+"
            SlotId(80) => {
                self.create_symbol(result, gss_node_id, SlotId(81));
            }
            //Symbol : "{" Layout Symbol Layout Symbol . Layout "}" Layout "+"
            SlotId(81) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "+"
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "+"
            SlotId(82) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "+"
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
                            "\"}\"",
                            i,
                            SlotId(82),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "+"
            SlotId(83) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "+"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "+"
            SlotId(84) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "+".
                        let next_slot_id = SlotId(85);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(84),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "+".
            SlotId(85) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
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
            //Symbol : . "(" Layout Alternative_Plus_7 Layout ")"
            SlotId(86) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol : "(" . Layout Alternative_Plus_7 Layout ")"
                        let next_slot_id = SlotId(87);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(86),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Layout Alternative_Plus_7 Layout ")"
            SlotId(87) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "(" Layout . Alternative_Plus_7 Layout ")"
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
            //Symbol : "(" Layout . Alternative_Plus_7 Layout ")"
            SlotId(88) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(89));
            }
            //Symbol : "(" Layout Alternative_Plus_7 . Layout ")"
            SlotId(89) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : "(" Layout Alternative_Plus_7 Layout . ")"
                        let next_slot_id = SlotId(90);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(89),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Alternative_Plus_7 Layout . ")"
            SlotId(90) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Symbol : "(" Layout Alternative_Plus_7 Layout ")".
                        let next_slot_id = SlotId(91);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(90),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Alternative_Plus_7 Layout ")".
            SlotId(91) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(91);
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
            //Symbol : . Identifier Layout ":" Layout Symbol
            SlotId(92) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Symbol : Identifier . Layout ":" Layout Symbol
                        let next_slot_id = SlotId(93);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(92),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Identifier . Layout ":" Layout Symbol
            SlotId(93) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : Identifier Layout . ":" Layout Symbol
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(93),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Identifier Layout . ":" Layout Symbol
            SlotId(94) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\":\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\":\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //Symbol : Identifier Layout ":" . Layout Symbol
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\":\"",
                            i,
                            SlotId(94),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Identifier Layout ":" . Layout Symbol
            SlotId(95) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol : Identifier Layout ":" Layout . Symbol
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
            //Symbol : Identifier Layout ":" Layout . Symbol
            SlotId(96) => {
                self.create_symbol(result, gss_node_id, SlotId(97));
            }
            //Symbol : Identifier Layout ":" Layout Symbol.
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
            //Symbol : . Identifier
            SlotId(98) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Symbol : Identifier.
                        let next_slot_id = SlotId(99);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(98),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Identifier.
            SlotId(99) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
                let end_slot_id = SlotId(99);
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
            SlotId(100) => {
                self.create_regex(result, gss_node_id, SlotId(101));
            }
            //Regex : Regex . Layout "+"
            SlotId(101) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : Regex Layout . "+"
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
            //Regex : Regex Layout . "+"
            SlotId(102) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Regex : Regex Layout "+".
                        let next_slot_id = SlotId(103);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(102),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "+".
            SlotId(103) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(103);
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
            SlotId(104) => {
                self.create_regex(result, gss_node_id, SlotId(105));
            }
            //Regex : Regex . Layout "*"
            SlotId(105) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : Regex Layout . "*"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex : Regex Layout . "*"
            SlotId(106) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Regex : Regex Layout "*".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Regex : Regex Layout "*".
            SlotId(107) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(107);
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
            SlotId(108) => {
                self.create_regex(result, gss_node_id, SlotId(109));
            }
            //Regex : Regex . Layout "?"
            SlotId(109) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : Regex Layout . "?"
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
            //Regex : Regex Layout . "?"
            SlotId(110) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Regex : Regex Layout "?".
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
                            "\"?\"",
                            i,
                            SlotId(110),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "?".
            SlotId(111) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
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
            //Regex : . "(" Layout Regex Layout Regex_Plus_9 Layout ")"
            SlotId(112) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Regex : "(" . Layout Regex Layout Regex_Plus_9 Layout ")"
                        let next_slot_id = SlotId(113);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(112),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout Regex Layout Regex_Plus_9 Layout ")"
            SlotId(113) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : "(" Layout . Regex Layout Regex_Plus_9 Layout ")"
                        let next_slot_id = SlotId(114);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(113),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout . Regex Layout Regex_Plus_9 Layout ")"
            SlotId(114) => {
                self.create_regex(result, gss_node_id, SlotId(115));
            }
            //Regex : "(" Layout Regex . Layout Regex_Plus_9 Layout ")"
            SlotId(115) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : "(" Layout Regex Layout . Regex_Plus_9 Layout ")"
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
            //Regex : "(" Layout Regex Layout . Regex_Plus_9 Layout ")"
            SlotId(116) => {
                self.create_regex_plus_9(result, gss_node_id, SlotId(117));
            }
            //Regex : "(" Layout Regex Layout Regex_Plus_9 . Layout ")"
            SlotId(117) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : "(" Layout Regex Layout Regex_Plus_9 Layout . ")"
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
            //Regex : "(" Layout Regex Layout Regex_Plus_9 Layout . ")"
            SlotId(118) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Regex : "(" Layout Regex Layout Regex_Plus_9 Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(118),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout Regex Layout Regex_Plus_9 Layout ")".
            SlotId(119) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
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
            //Regex : . "(" Layout RegexRule_Plus_5 Layout ")"
            SlotId(120) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Regex : "(" . Layout RegexRule_Plus_5 Layout ")"
                        let next_slot_id = SlotId(121);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(120),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout RegexRule_Plus_5 Layout ")"
            SlotId(121) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : "(" Layout . RegexRule_Plus_5 Layout ")"
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
            //Regex : "(" Layout . RegexRule_Plus_5 Layout ")"
            SlotId(122) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(123));
            }
            //Regex : "(" Layout RegexRule_Plus_5 . Layout ")"
            SlotId(123) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : "(" Layout RegexRule_Plus_5 Layout . ")"
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
            //Regex : "(" Layout RegexRule_Plus_5 Layout . ")"
            SlotId(124) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Regex : "(" Layout RegexRule_Plus_5 Layout ")".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(124),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_5 Layout ")".
            SlotId(125) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(125);
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
            SlotId(126) => {
                self.create_char_class(result, gss_node_id, SlotId(127));
            }
            //Regex : CharClass.
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
            //Regex : . """ Layout Char Layout """
            SlotId(128) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Regex : """ . Layout Char Layout """
                        let next_slot_id = SlotId(129);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(128),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ . Layout Char Layout """
            SlotId(129) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : """ Layout . Char Layout """
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
            //Regex : """ Layout . Char Layout """
            SlotId(130) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Regex : """ Layout Char . Layout """
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
                            "Char",
                            i,
                            SlotId(130),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout Char . Layout """
            SlotId(131) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : """ Layout Char Layout . """
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
            //Regex : """ Layout Char Layout . """
            SlotId(132) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Regex : """ Layout Char Layout """.
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
                            "\"\"\"",
                            i,
                            SlotId(132),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout Char Layout """.
            SlotId(133) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
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
            //CharClass : . CharClass_Opt_9 Layout "[" Layout CharClass_Plus_10 Layout "]"
            SlotId(134) => {
                self.create_char_class_opt_9(result, gss_node_id, SlotId(135));
            }
            //CharClass : CharClass_Opt_9 . Layout "[" Layout CharClass_Plus_10 Layout "]"
            SlotId(135) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //CharClass : CharClass_Opt_9 Layout . "[" Layout CharClass_Plus_10 Layout "]"
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
            //CharClass : CharClass_Opt_9 Layout . "[" Layout CharClass_Plus_10 Layout "]"
            SlotId(136) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"[\"", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"[\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //CharClass : CharClass_Opt_9 Layout "[" . Layout CharClass_Plus_10 Layout "]"
                        let next_slot_id = SlotId(137);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(136),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_9 Layout "[" . Layout CharClass_Plus_10 Layout "]"
            SlotId(137) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //CharClass : CharClass_Opt_9 Layout "[" Layout . CharClass_Plus_10 Layout "]"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //CharClass : CharClass_Opt_9 Layout "[" Layout . CharClass_Plus_10 Layout "]"
            SlotId(138) => {
                self.create_char_class_plus_10(result, gss_node_id, SlotId(139));
            }
            //CharClass : CharClass_Opt_9 Layout "[" Layout CharClass_Plus_10 . Layout "]"
            SlotId(139) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //CharClass : CharClass_Opt_9 Layout "[" Layout CharClass_Plus_10 Layout . "]"
                        let next_slot_id = SlotId(140);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(139),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_9 Layout "[" Layout CharClass_Plus_10 Layout . "]"
            SlotId(140) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"]\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"]\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //CharClass : CharClass_Opt_9 Layout "[" Layout CharClass_Plus_10 Layout "]".
                        let next_slot_id = SlotId(141);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(140),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_9 Layout "[" Layout CharClass_Plus_10 Layout "]".
            SlotId(141) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
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
            //RangeElement : . Range
            SlotId(142) => {
                self.create_range(result, gss_node_id, SlotId(143));
            }
            //RangeElement : Range.
            SlotId(143) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(143);
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
            //RangeElement : . RangeChar
            SlotId(144) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //RangeElement : RangeChar.
                        let next_slot_id = SlotId(145);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(144),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RangeElement : RangeChar.
            SlotId(145) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(145);
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
            SlotId(146) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Range : RangeChar . Layout "-" Layout RangeChar
                        let next_slot_id = SlotId(147);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(146),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar . Layout "-" Layout RangeChar
            SlotId(147) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Range : RangeChar Layout . "-" Layout RangeChar
                        let next_slot_id = SlotId(148);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(147),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar Layout . "-" Layout RangeChar
            SlotId(148) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"-\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"-\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Range : RangeChar Layout "-" . Layout RangeChar
                        let next_slot_id = SlotId(149);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(148),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar Layout "-" . Layout RangeChar
            SlotId(149) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Range : RangeChar Layout "-" Layout . RangeChar
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
            //Range : RangeChar Layout "-" Layout . RangeChar
            SlotId(150) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Range : RangeChar Layout "-" Layout RangeChar.
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
                            "RangeChar",
                            i,
                            SlotId(150),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : RangeChar Layout "-" Layout RangeChar.
            SlotId(151) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(151);
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
            //Grammar_Opt_0 : . LayoutDef
            SlotId(152) => {
                self.create_layout_def(result, gss_node_id, SlotId(153));
            }
            //Grammar_Opt_0 : LayoutDef.
            SlotId(153) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
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
            //Grammar_Opt_0 : .
            SlotId(154) => {
                let end_slot_id = SlotId(154);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(12);
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
            //Grammar_Plus_0 : . Grammar_Plus_0 Layout SyntaxRule
            SlotId(155) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(156));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 . Layout SyntaxRule
            SlotId(156) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
                        let next_slot_id = SlotId(157);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(156),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
            SlotId(157) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(158));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout SyntaxRule.
            SlotId(158) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
                let end_slot_id = SlotId(158);
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
            SlotId(159) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(160));
            }
            //Grammar_Plus_0 : SyntaxRule.
            SlotId(160) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
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
            //Grammar_Opt_1 : . Grammar_Plus_0
            SlotId(161) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(162));
            }
            //Grammar_Opt_1 : Grammar_Plus_0.
            SlotId(162) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(162);
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
            SlotId(163) => {
                let end_slot_id = SlotId(163);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(14);
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
            //Grammar_Star_0 : . Grammar_Opt_1
            SlotId(164) => {
                self.create_grammar_opt_1(result, gss_node_id, SlotId(165));
            }
            //Grammar_Star_0 : Grammar_Opt_1.
            SlotId(165) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
                let end_slot_id = SlotId(165);
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
            //Grammar_Opt_2 : . RegexBlock
            SlotId(166) => {
                self.create_regex_block(result, gss_node_id, SlotId(167));
            }
            //Grammar_Opt_2 : RegexBlock.
            SlotId(167) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
                let end_slot_id = SlotId(167);
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
            //Grammar_Opt_2 : .
            SlotId(168) => {
                let end_slot_id = SlotId(168);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(16);
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
            //LayoutDef_Plus_1 : . LayoutDef_Plus_1 Layout Identifier
            SlotId(169) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(170));
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 . Layout Identifier
            SlotId(170) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
                        let next_slot_id = SlotId(171);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(170),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
            SlotId(171) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
                        let next_slot_id = SlotId(172);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(171),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
            SlotId(172) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(172);
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
            //LayoutDef_Plus_1 : . Identifier
            SlotId(173) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //LayoutDef_Plus_1 : Identifier.
                        let next_slot_id = SlotId(174);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(173),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : Identifier.
            SlotId(174) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(174);
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
            //LayoutDef_Opt_3 : . LayoutDef_Plus_1
            SlotId(175) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(176));
            }
            //LayoutDef_Opt_3 : LayoutDef_Plus_1.
            SlotId(176) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
                let end_slot_id = SlotId(176);
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
            //LayoutDef_Opt_3 : .
            SlotId(177) => {
                let end_slot_id = SlotId(177);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
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
            //LayoutDef_Star_1 : . LayoutDef_Opt_3
            SlotId(178) => {
                self.create_layout_def_opt_3(result, gss_node_id, SlotId(179));
            }
            //LayoutDef_Star_1 : LayoutDef_Opt_3.
            SlotId(179) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(179);
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
            //SyntaxRule_Plus_2 : . SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel
            SlotId(180) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(181));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 . Layout ">" Layout PriorityLevel
            SlotId(181) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
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
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
            SlotId(182) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
                        let next_slot_id = SlotId(183);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(182),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
            SlotId(183) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
            SlotId(184) => {
                self.create_priority_level(result, gss_node_id, SlotId(185));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel.
            SlotId(185) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
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
            //SyntaxRule_Plus_2 : . PriorityLevel
            SlotId(186) => {
                self.create_priority_level(result, gss_node_id, SlotId(187));
            }
            //SyntaxRule_Plus_2 : PriorityLevel.
            SlotId(187) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
                let end_slot_id = SlotId(187);
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
            //SyntaxRule_Opt_4 : . SyntaxRule_Plus_2
            SlotId(188) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(189));
            }
            //SyntaxRule_Opt_4 : SyntaxRule_Plus_2.
            SlotId(189) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
                let end_slot_id = SlotId(189);
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
            //SyntaxRule_Opt_4 : .
            SlotId(190) => {
                let end_slot_id = SlotId(190);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(21);
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
            //SyntaxRule_Star_2 : . SyntaxRule_Opt_4
            SlotId(191) => {
                self.create_syntax_rule_opt_4(result, gss_node_id, SlotId(192));
            }
            //SyntaxRule_Star_2 : SyntaxRule_Opt_4.
            SlotId(192) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
                let end_slot_id = SlotId(192);
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
            //RegexBlock_Plus_3 : . RegexBlock_Plus_3 Layout RegexRule
            SlotId(193) => {
                self.create_regex_block_plus_3(result, gss_node_id, SlotId(194));
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 . Layout RegexRule
            SlotId(194) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout . RegexRule
                        let next_slot_id = SlotId(195);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(194),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout . RegexRule
            SlotId(195) => {
                self.create_regex_rule(result, gss_node_id, SlotId(196));
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout RegexRule.
            SlotId(196) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(196);
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
            //RegexBlock_Plus_3 : . RegexRule
            SlotId(197) => {
                self.create_regex_rule(result, gss_node_id, SlotId(198));
            }
            //RegexBlock_Plus_3 : RegexRule.
            SlotId(198) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
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
            //RegexBlock_Opt_5 : . RegexBlock_Plus_3
            SlotId(199) => {
                self.create_regex_block_plus_3(result, gss_node_id, SlotId(200));
            }
            //RegexBlock_Opt_5 : RegexBlock_Plus_3.
            SlotId(200) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
                let end_slot_id = SlotId(200);
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
            //RegexBlock_Opt_5 : .
            SlotId(201) => {
                let end_slot_id = SlotId(201);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(24);
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
            //RegexBlock_Star_3 : . RegexBlock_Opt_5
            SlotId(202) => {
                self.create_regex_block_opt_5(result, gss_node_id, SlotId(203));
            }
            //RegexBlock_Star_3 : RegexBlock_Opt_5.
            SlotId(203) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(203);
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
            //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout Regex
            SlotId(204) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(205));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 . Layout Regex
            SlotId(205) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . Regex
                        let next_slot_id = SlotId(206);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(205),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . Regex
            SlotId(206) => {
                self.create_regex(result, gss_node_id, SlotId(207));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout Regex.
            SlotId(207) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
                let end_slot_id = SlotId(207);
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
            //RegexRule_Plus_5 : . Regex
            SlotId(208) => {
                self.create_regex(result, gss_node_id, SlotId(209));
            }
            //RegexRule_Plus_5 : Regex.
            SlotId(209) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
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
            //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5
            SlotId(210) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(211));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 . Layout "|" Layout RegexRule_Plus_5
            SlotId(211) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . "|" Layout RegexRule_Plus_5
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
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . "|" Layout RegexRule_Plus_5
            SlotId(212) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" . Layout RegexRule_Plus_5
                        let next_slot_id = SlotId(213);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(212),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" . Layout RegexRule_Plus_5
            SlotId(213) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout . RegexRule_Plus_5
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout . RegexRule_Plus_5
            SlotId(214) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(215));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5.
            SlotId(215) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
                let end_slot_id = SlotId(215);
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
            //RegexRule_Plus_4 : . RegexRule_Plus_5
            SlotId(216) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(217));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_5.
            SlotId(217) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
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
            //PriorityLevel_Plus_6 : . PriorityLevel_Plus_6 Layout "|" Layout Alternative
            SlotId(218) => {
                self.create_priority_level_plus_6(result, gss_node_id, SlotId(219));
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 . Layout "|" Layout Alternative
            SlotId(219) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout . "|" Layout Alternative
                        let next_slot_id = SlotId(220);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(219),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout . "|" Layout Alternative
            SlotId(220) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" . Layout Alternative
                        let next_slot_id = SlotId(221);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(220),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" . Layout Alternative
            SlotId(221) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout . Alternative
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
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout . Alternative
            SlotId(222) => {
                self.create_alternative(result, gss_node_id, SlotId(223));
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout Alternative.
            SlotId(223) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
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
            //PriorityLevel_Plus_6 : . Alternative
            SlotId(224) => {
                self.create_alternative(result, gss_node_id, SlotId(225));
            }
            //PriorityLevel_Plus_6 : Alternative.
            SlotId(225) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
                let end_slot_id = SlotId(225);
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
            //PriorityLevel_Opt_6 : . PriorityLevel_Plus_6
            SlotId(226) => {
                self.create_priority_level_plus_6(result, gss_node_id, SlotId(227));
            }
            //PriorityLevel_Opt_6 : PriorityLevel_Plus_6.
            SlotId(227) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
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
            //PriorityLevel_Opt_6 : .
            SlotId(228) => {
                let end_slot_id = SlotId(228);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(29);
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
            //PriorityLevel_Star_4 : . PriorityLevel_Opt_6
            SlotId(229) => {
                self.create_priority_level_opt_6(result, gss_node_id, SlotId(230));
            }
            //PriorityLevel_Star_4 : PriorityLevel_Opt_6.
            SlotId(230) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(30);
                let end_slot_id = SlotId(230);
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
            //Alternative_Plus_7 : . Alternative_Plus_7 Layout Symbol
            SlotId(231) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(232));
            }
            //Alternative_Plus_7 : Alternative_Plus_7 . Layout Symbol
            SlotId(232) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Alternative_Plus_7 : Alternative_Plus_7 Layout . Symbol
                        let next_slot_id = SlotId(233);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(232),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Plus_7 : Alternative_Plus_7 Layout . Symbol
            SlotId(233) => {
                self.create_symbol(result, gss_node_id, SlotId(234));
            }
            //Alternative_Plus_7 : Alternative_Plus_7 Layout Symbol.
            SlotId(234) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
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
            //Alternative_Plus_7 : . Symbol
            SlotId(235) => {
                self.create_symbol(result, gss_node_id, SlotId(236));
            }
            //Alternative_Plus_7 : Symbol.
            SlotId(236) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
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
            //Alternative_Opt_7 : . Alternative_Plus_7
            SlotId(237) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(238));
            }
            //Alternative_Opt_7 : Alternative_Plus_7.
            SlotId(238) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(32);
                let end_slot_id = SlotId(238);
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
            //Alternative_Opt_7 : .
            SlotId(239) => {
                let end_slot_id = SlotId(239);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
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
            //Alternative_Star_5 : . Alternative_Opt_7
            SlotId(240) => {
                self.create_alternative_opt_7(result, gss_node_id, SlotId(241));
            }
            //Alternative_Star_5 : Alternative_Opt_7.
            SlotId(241) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
                let end_slot_id = SlotId(241);
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
            //Alternative_Opt_8 : . Label
            SlotId(242) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Label", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Label", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Alternative_Opt_8 : Label.
                        let next_slot_id = SlotId(243);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Label",
                            i,
                            SlotId(242),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Opt_8 : Label.
            SlotId(243) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
                let end_slot_id = SlotId(243);
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
            //Alternative_Opt_8 : .
            SlotId(244) => {
                let end_slot_id = SlotId(244);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(34);
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
            //Symbol_Group_0 : . "|" Layout Symbol
            SlotId(245) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol_Group_0 : "|" . Layout Symbol
                        let next_slot_id = SlotId(246);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(245),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" . Layout Symbol
            SlotId(246) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol_Group_0 : "|" Layout . Symbol
                        let next_slot_id = SlotId(247);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(246),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" Layout . Symbol
            SlotId(247) => {
                self.create_symbol(result, gss_node_id, SlotId(248));
            }
            //Symbol_Group_0 : "|" Layout Symbol.
            SlotId(248) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(35);
                let end_slot_id = SlotId(248);
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
            //Symbol_Plus_8 : . Symbol_Plus_8 Layout Symbol_Group_0
            SlotId(249) => {
                self.create_symbol_plus_8(result, gss_node_id, SlotId(250));
            }
            //Symbol_Plus_8 : Symbol_Plus_8 . Layout Symbol_Group_0
            SlotId(250) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol_Plus_8 : Symbol_Plus_8 Layout . Symbol_Group_0
                        let next_slot_id = SlotId(251);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(250),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Plus_8 : Symbol_Plus_8 Layout . Symbol_Group_0
            SlotId(251) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(252));
            }
            //Symbol_Plus_8 : Symbol_Plus_8 Layout Symbol_Group_0.
            SlotId(252) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(36);
                let end_slot_id = SlotId(252);
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
            //Symbol_Plus_8 : . Symbol_Group_0
            SlotId(253) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(254));
            }
            //Symbol_Plus_8 : Symbol_Group_0.
            SlotId(254) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(36);
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
            //Regex_Group_1 : . "|" Layout Regex
            SlotId(255) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Regex_Group_1 : "|" . Layout Regex
                        let next_slot_id = SlotId(256);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(255),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_1 : "|" . Layout Regex
            SlotId(256) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex_Group_1 : "|" Layout . Regex
                        let next_slot_id = SlotId(257);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(256),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_1 : "|" Layout . Regex
            SlotId(257) => {
                self.create_regex(result, gss_node_id, SlotId(258));
            }
            //Regex_Group_1 : "|" Layout Regex.
            SlotId(258) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(37);
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
            //Regex_Plus_9 : . Regex_Plus_9 Layout Regex_Group_1
            SlotId(259) => {
                self.create_regex_plus_9(result, gss_node_id, SlotId(260));
            }
            //Regex_Plus_9 : Regex_Plus_9 . Layout Regex_Group_1
            SlotId(260) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex_Plus_9 : Regex_Plus_9 Layout . Regex_Group_1
                        let next_slot_id = SlotId(261);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(260),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Plus_9 : Regex_Plus_9 Layout . Regex_Group_1
            SlotId(261) => {
                self.create_regex_group_1(result, gss_node_id, SlotId(262));
            }
            //Regex_Plus_9 : Regex_Plus_9 Layout Regex_Group_1.
            SlotId(262) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(38);
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
            //Regex_Plus_9 : . Regex_Group_1
            SlotId(263) => {
                self.create_regex_group_1(result, gss_node_id, SlotId(264));
            }
            //Regex_Plus_9 : Regex_Group_1.
            SlotId(264) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(38);
                let end_slot_id = SlotId(264);
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
            //CharClass_Opt_9 : . "!"
            SlotId(265) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //CharClass_Opt_9 : "!".
                        let next_slot_id = SlotId(266);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(265),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Opt_9 : "!".
            SlotId(266) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(39);
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
            //CharClass_Opt_9 : .
            SlotId(267) => {
                let end_slot_id = SlotId(267);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(26), input_index, input_index);
                let nonterminal_id = NonterminalId(39);
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
            //CharClass_Plus_10 : . CharClass_Plus_10 Layout RangeElement
            SlotId(268) => {
                self.create_char_class_plus_10(result, gss_node_id, SlotId(269));
            }
            //CharClass_Plus_10 : CharClass_Plus_10 . Layout RangeElement
            SlotId(269) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //CharClass_Plus_10 : CharClass_Plus_10 Layout . RangeElement
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
            //CharClass_Plus_10 : CharClass_Plus_10 Layout . RangeElement
            SlotId(270) => {
                self.create_range_element(result, gss_node_id, SlotId(271));
            }
            //CharClass_Plus_10 : CharClass_Plus_10 Layout RangeElement.
            SlotId(271) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(40);
                let end_slot_id = SlotId(271);
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
            //CharClass_Plus_10 : . RangeElement
            SlotId(272) => {
                self.create_range_element(result, gss_node_id, SlotId(273));
            }
            //CharClass_Plus_10 : RangeElement.
            SlotId(273) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(40);
                let end_slot_id = SlotId(273);
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
            SlotId(274) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartGrammar : Layout . Grammar Layout
                        let next_slot_id = SlotId(275);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(274),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout . Grammar Layout
            SlotId(275) => {
                self.create_grammar(result, gss_node_id, SlotId(276));
            }
            //StartGrammar : Layout Grammar . Layout
            SlotId(276) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartGrammar : Layout Grammar Layout.
                        let next_slot_id = SlotId(277);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(276),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout Grammar Layout.
            SlotId(277) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(41);
                let end_slot_id = SlotId(277);
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
            //StartLayoutDef : . Layout LayoutDef Layout
            SlotId(278) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartLayoutDef : Layout . LayoutDef Layout
                        let next_slot_id = SlotId(279);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(278),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartLayoutDef : Layout . LayoutDef Layout
            SlotId(279) => {
                self.create_layout_def(result, gss_node_id, SlotId(280));
            }
            //StartLayoutDef : Layout LayoutDef . Layout
            SlotId(280) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartLayoutDef : Layout LayoutDef Layout.
                        let next_slot_id = SlotId(281);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(280),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartLayoutDef : Layout LayoutDef Layout.
            SlotId(281) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(42);
                let end_slot_id = SlotId(281);
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
            SlotId(282) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartSyntaxRule : Layout . SyntaxRule Layout
                        let next_slot_id = SlotId(283);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(282),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout . SyntaxRule Layout
            SlotId(283) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(284));
            }
            //StartSyntaxRule : Layout SyntaxRule . Layout
            SlotId(284) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartSyntaxRule : Layout SyntaxRule Layout.
                        let next_slot_id = SlotId(285);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(284),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout SyntaxRule Layout.
            SlotId(285) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
                let end_slot_id = SlotId(285);
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
            SlotId(286) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRegexBlock : Layout . RegexBlock Layout
                        let next_slot_id = SlotId(287);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(286),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexBlock : Layout . RegexBlock Layout
            SlotId(287) => {
                self.create_regex_block(result, gss_node_id, SlotId(288));
            }
            //StartRegexBlock : Layout RegexBlock . Layout
            SlotId(288) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRegexBlock : Layout RegexBlock Layout.
                        let next_slot_id = SlotId(289);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(288),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexBlock : Layout RegexBlock Layout.
            SlotId(289) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(44);
                let end_slot_id = SlotId(289);
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
            SlotId(290) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRegexRule : Layout . RegexRule Layout
                        let next_slot_id = SlotId(291);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(290),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout . RegexRule Layout
            SlotId(291) => {
                self.create_regex_rule(result, gss_node_id, SlotId(292));
            }
            //StartRegexRule : Layout RegexRule . Layout
            SlotId(292) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRegexRule : Layout RegexRule Layout.
                        let next_slot_id = SlotId(293);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(292),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout RegexRule Layout.
            SlotId(293) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(45);
                let end_slot_id = SlotId(293);
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
            SlotId(294) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartPriorityLevel : Layout . PriorityLevel Layout
                        let next_slot_id = SlotId(295);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(294),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout . PriorityLevel Layout
            SlotId(295) => {
                self.create_priority_level(result, gss_node_id, SlotId(296));
            }
            //StartPriorityLevel : Layout PriorityLevel . Layout
            SlotId(296) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartPriorityLevel : Layout PriorityLevel Layout.
                        let next_slot_id = SlotId(297);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(296),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout PriorityLevel Layout.
            SlotId(297) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(46);
                let end_slot_id = SlotId(297);
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
            SlotId(298) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartAlternative : Layout . Alternative Layout
                        let next_slot_id = SlotId(299);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(298),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout . Alternative Layout
            SlotId(299) => {
                self.create_alternative(result, gss_node_id, SlotId(300));
            }
            //StartAlternative : Layout Alternative . Layout
            SlotId(300) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartAlternative : Layout Alternative Layout.
                        let next_slot_id = SlotId(301);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(300),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout Alternative Layout.
            SlotId(301) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(47);
                let end_slot_id = SlotId(301);
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
            SlotId(302) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartSymbol : Layout . Symbol Layout
                        let next_slot_id = SlotId(303);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(302),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSymbol : Layout . Symbol Layout
            SlotId(303) => {
                self.create_symbol(result, gss_node_id, SlotId(304));
            }
            //StartSymbol : Layout Symbol . Layout
            SlotId(304) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartSymbol : Layout Symbol Layout.
                        let next_slot_id = SlotId(305);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(304),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSymbol : Layout Symbol Layout.
            SlotId(305) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(48);
                let end_slot_id = SlotId(305);
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
            SlotId(306) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRegex : Layout . Regex Layout
                        let next_slot_id = SlotId(307);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(306),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout . Regex Layout
            SlotId(307) => {
                self.create_regex(result, gss_node_id, SlotId(308));
            }
            //StartRegex : Layout Regex . Layout
            SlotId(308) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRegex : Layout Regex Layout.
                        let next_slot_id = SlotId(309);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(308),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout Regex Layout.
            SlotId(309) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(49);
                let end_slot_id = SlotId(309);
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
            SlotId(310) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartCharClass : Layout . CharClass Layout
                        let next_slot_id = SlotId(311);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(310),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout . CharClass Layout
            SlotId(311) => {
                self.create_char_class(result, gss_node_id, SlotId(312));
            }
            //StartCharClass : Layout CharClass . Layout
            SlotId(312) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartCharClass : Layout CharClass Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(312),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout CharClass Layout.
            SlotId(313) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(50);
                let end_slot_id = SlotId(313);
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
            //StartRangeElement : . Layout RangeElement Layout
            SlotId(314) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRangeElement : Layout . RangeElement Layout
                        let next_slot_id = SlotId(315);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(314),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRangeElement : Layout . RangeElement Layout
            SlotId(315) => {
                self.create_range_element(result, gss_node_id, SlotId(316));
            }
            //StartRangeElement : Layout RangeElement . Layout
            SlotId(316) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRangeElement : Layout RangeElement Layout.
                        let next_slot_id = SlotId(317);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(316),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRangeElement : Layout RangeElement Layout.
            SlotId(317) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(51);
                let end_slot_id = SlotId(317);
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
            SlotId(318) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRange : Layout . Range Layout
                        let next_slot_id = SlotId(319);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(318),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout . Range Layout
            SlotId(319) => {
                self.create_range(result, gss_node_id, SlotId(320));
            }
            //StartRange : Layout Range . Layout
            SlotId(320) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //StartRange : Layout Range Layout.
                        let next_slot_id = SlotId(321);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(320),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout Range Layout.
            SlotId(321) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(52);
                let end_slot_id = SlotId(321);
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
                //Grammar : . "grammar" Layout Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //LayoutDef
            NonterminalId(1) => {
                //LayoutDef : . "layout" Layout LayoutDef_Star_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(10),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule
            NonterminalId(2) => {
                //SyntaxRule : . Identifier Layout "=" Layout SyntaxRule_Star_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock
            NonterminalId(3) => {
                //RegexBlock : . "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(20),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexRule
            NonterminalId(4) => {
                //RegexRule : . Identifier Layout "=" Layout RegexRule_Plus_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(28),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel
            NonterminalId(5) => {
                //PriorityLevel : . PriorityLevel_Star_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(34),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative
            NonterminalId(6) => {
                //Alternative : . Alternative_Star_5 Layout Alternative_Opt_8
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(36),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol
            NonterminalId(7) => {
                //Symbol : . Symbol Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(40),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Symbol Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(44),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Symbol Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(48),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Layout Symbol Layout Symbol_Plus_8 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(52),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . """ Layout String Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(60),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(66),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(76),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Layout Alternative_Plus_7 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(86),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Identifier Layout ":" Layout Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(92),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(98),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex
            NonterminalId(8) => {
                //Regex : . Regex Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(100),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(104),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(108),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . "(" Layout Regex Layout Regex_Plus_9 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(112),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . "(" Layout RegexRule_Plus_5 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(120),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . CharClass
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(126),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . """ Layout Char Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(128),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass
            NonterminalId(9) => {
                //CharClass : . CharClass_Opt_9 Layout "[" Layout CharClass_Plus_10 Layout "]"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(134),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RangeElement
            NonterminalId(10) => {
                //RangeElement : . Range
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(142),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RangeElement : . RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(144),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Range
            NonterminalId(11) => {
                //Range : . RangeChar Layout "-" Layout RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(146),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Opt_0
            NonterminalId(12) => {
                //Grammar_Opt_0 : . LayoutDef
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(152),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(154),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Plus_0
            NonterminalId(13) => {
                //Grammar_Plus_0 : . Grammar_Plus_0 Layout SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(155),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Plus_0 : . SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(159),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Opt_1
            NonterminalId(14) => {
                //Grammar_Opt_1 : . Grammar_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(161),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(163),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Star_0
            NonterminalId(15) => {
                //Grammar_Star_0 : . Grammar_Opt_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(164),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Opt_2
            NonterminalId(16) => {
                //Grammar_Opt_2 : . RegexBlock
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(166),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Opt_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(168),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //LayoutDef_Plus_1
            NonterminalId(17) => {
                //LayoutDef_Plus_1 : . LayoutDef_Plus_1 Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(169),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //LayoutDef_Plus_1 : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(173),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //LayoutDef_Opt_3
            NonterminalId(18) => {
                //LayoutDef_Opt_3 : . LayoutDef_Plus_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(175),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //LayoutDef_Opt_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(177),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //LayoutDef_Star_1
            NonterminalId(19) => {
                //LayoutDef_Star_1 : . LayoutDef_Opt_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(178),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule_Plus_2
            NonterminalId(20) => {
                //SyntaxRule_Plus_2 : . SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(180),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //SyntaxRule_Plus_2 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(186),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule_Opt_4
            NonterminalId(21) => {
                //SyntaxRule_Opt_4 : . SyntaxRule_Plus_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(188),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //SyntaxRule_Opt_4 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(190),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //SyntaxRule_Star_2
            NonterminalId(22) => {
                //SyntaxRule_Star_2 : . SyntaxRule_Opt_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(191),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock_Plus_3
            NonterminalId(23) => {
                //RegexBlock_Plus_3 : . RegexBlock_Plus_3 Layout RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(193),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexBlock_Plus_3 : . RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(197),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock_Opt_5
            NonterminalId(24) => {
                //RegexBlock_Opt_5 : . RegexBlock_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(199),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexBlock_Opt_5 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(201),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexBlock_Star_3
            NonterminalId(25) => {
                //RegexBlock_Star_3 : . RegexBlock_Opt_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(202),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexRule_Plus_5
            NonterminalId(26) => {
                //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(204),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexRule_Plus_5 : . Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(208),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //RegexRule_Plus_4
            NonterminalId(27) => {
                //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(210),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //RegexRule_Plus_4 : . RegexRule_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(216),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Plus_6
            NonterminalId(28) => {
                //PriorityLevel_Plus_6 : . PriorityLevel_Plus_6 Layout "|" Layout Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(218),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Plus_6 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(224),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Opt_6
            NonterminalId(29) => {
                //PriorityLevel_Opt_6 : . PriorityLevel_Plus_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(226),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Opt_6 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(228),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Star_4
            NonterminalId(30) => {
                //PriorityLevel_Star_4 : . PriorityLevel_Opt_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(229),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Plus_7
            NonterminalId(31) => {
                //Alternative_Plus_7 : . Alternative_Plus_7 Layout Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(231),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Plus_7 : . Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(235),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Opt_7
            NonterminalId(32) => {
                //Alternative_Opt_7 : . Alternative_Plus_7
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(237),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Opt_7 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(239),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Star_5
            NonterminalId(33) => {
                //Alternative_Star_5 : . Alternative_Opt_7
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(240),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Opt_8
            NonterminalId(34) => {
                //Alternative_Opt_8 : . Label
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(242),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Opt_8 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(244),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol_Group_0
            NonterminalId(35) => {
                //Symbol_Group_0 : . "|" Layout Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(245),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol_Plus_8
            NonterminalId(36) => {
                //Symbol_Plus_8 : . Symbol_Plus_8 Layout Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(249),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol_Plus_8 : . Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(253),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Group_1
            NonterminalId(37) => {
                //Regex_Group_1 : . "|" Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(255),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Plus_9
            NonterminalId(38) => {
                //Regex_Plus_9 : . Regex_Plus_9 Layout Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(259),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex_Plus_9 : . Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(263),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Opt_9
            NonterminalId(39) => {
                //CharClass_Opt_9 : . "!"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(265),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Opt_9 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(267),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Plus_10
            NonterminalId(40) => {
                //CharClass_Plus_10 : . CharClass_Plus_10 Layout RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(268),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Plus_10 : . RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(272),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartGrammar
            NonterminalId(41) => {
                //StartGrammar : . Layout Grammar Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(274),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartLayoutDef
            NonterminalId(42) => {
                //StartLayoutDef : . Layout LayoutDef Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(278),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartSyntaxRule
            NonterminalId(43) => {
                //StartSyntaxRule : . Layout SyntaxRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(282),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRegexBlock
            NonterminalId(44) => {
                //StartRegexBlock : . Layout RegexBlock Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(286),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRegexRule
            NonterminalId(45) => {
                //StartRegexRule : . Layout RegexRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(290),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartPriorityLevel
            NonterminalId(46) => {
                //StartPriorityLevel : . Layout PriorityLevel Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(294),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartAlternative
            NonterminalId(47) => {
                //StartAlternative : . Layout Alternative Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(298),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartSymbol
            NonterminalId(48) => {
                //StartSymbol : . Layout Symbol Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(302),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRegex
            NonterminalId(49) => {
                //StartRegex : . Layout Regex Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(306),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartCharClass
            NonterminalId(50) => {
                //StartCharClass : . Layout CharClass Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(310),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRangeElement
            NonterminalId(51) => {
                //StartRangeElement : . Layout RangeElement Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(314),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRange
            NonterminalId(52) => {
                //StartRange : . Layout Range Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(318),
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
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 53],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 53],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 322],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 27],
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
            gss_nodes_index: [const { vec![] }; 53],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 53],
            intermediate_nodes_index: [const { InlineMap::Empty }; 322],
            terminal_nodes_index: [const { InlineMap::Empty }; 27],
            stats: Stats::default(),
            intermediate_nodes_children: vec![],
            intermediate_nodes_children_map: OnceCell::new(),
            nonterminal_nodes_children: vec![],
            nonterminal_nodes_children_map: OnceCell::new(),
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
    fn create_syntax_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(2), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(3), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(4), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(5), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(6), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(7), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(8), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(9), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(10), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(11), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(12), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_plus_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(13), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(14), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_star_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(15), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(16), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_plus_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(17), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_opt_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(18), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_star_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(19), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_plus_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(20), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_opt_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(21), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_star_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(22), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_plus_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(23), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_opt_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(24), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_star_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(25), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_plus_5(
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
    fn create_priority_level_plus_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(28), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_opt_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(29), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_star_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(30), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_plus_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(31), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_opt_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(32), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_star_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(33), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_opt_8(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(34), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_group_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(35), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_plus_8(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(36), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_group_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(37), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_plus_9(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(38), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_opt_9(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(39), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_plus_10(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(40), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_grammar(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(41), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_layout_def(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(42), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_syntax_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(43), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex_block(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(44), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(45), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(46), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(47), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_symbol(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(48), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(49), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(50), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(51), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(52), sppf_node_id, gss_node_id, return_slot);
    }
}
