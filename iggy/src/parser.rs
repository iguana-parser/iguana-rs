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
pub const NONTERMINALS: [Nonterminal; 59] = [
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
        name: "Except",
        display: "Except",
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
        name: "RegexRule_Opt_6",
        display: "Except?",
        kind: Some(EbnfKind::Opt),
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
        name: "CharClass_Opt_11",
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
        name: "StartExcept",
        display: "StartExcept",
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
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Grammar" => NonterminalId(0), "LayoutDef" => NonterminalId(1), "SyntaxRule" =>
    NonterminalId(2), "RegexBlock" => NonterminalId(3), "RegexRule" => NonterminalId(4),
    "Except" => NonterminalId(5), "PriorityLevel" => NonterminalId(6), "Associativity" =>
    NonterminalId(7), "Alternative" => NonterminalId(8), "Regex" => NonterminalId(9),
    "CharClass" => NonterminalId(10), "RangeElement" => NonterminalId(11), "Range" =>
    NonterminalId(12), "Grammar_Opt_0" => NonterminalId(13), "Grammar_Plus_0" =>
    NonterminalId(14), "Grammar_Opt_1" => NonterminalId(15), "Grammar_Star_0" =>
    NonterminalId(16), "Grammar_Opt_2" => NonterminalId(17), "LayoutDef_Plus_1" =>
    NonterminalId(18), "LayoutDef_Opt_3" => NonterminalId(19), "LayoutDef_Star_1" =>
    NonterminalId(20), "SyntaxRule_Plus_2" => NonterminalId(21), "SyntaxRule_Opt_4" =>
    NonterminalId(22), "SyntaxRule_Star_2" => NonterminalId(23), "RegexBlock_Plus_3" =>
    NonterminalId(24), "RegexBlock_Opt_5" => NonterminalId(25), "RegexBlock_Star_3" =>
    NonterminalId(26), "RegexRule_Plus_5" => NonterminalId(27), "RegexRule_Plus_4" =>
    NonterminalId(28), "RegexRule_Opt_6" => NonterminalId(29), "PriorityLevel_Opt_7" =>
    NonterminalId(30), "PriorityLevel_Plus_6" => NonterminalId(31), "PriorityLevel_Opt_8"
    => NonterminalId(32), "PriorityLevel_Star_4" => NonterminalId(33),
    "Alternative_Plus_7" => NonterminalId(34), "Alternative_Opt_9" => NonterminalId(35),
    "Alternative_Star_5" => NonterminalId(36), "Alternative_Opt_10" => NonterminalId(37),
    "Symbol_Group_0" => NonterminalId(38), "Symbol_Plus_8" => NonterminalId(39),
    "Regex_Group_1" => NonterminalId(40), "Regex_Plus_9" => NonterminalId(41),
    "CharClass_Opt_11" => NonterminalId(42), "CharClass_Plus_10" => NonterminalId(43),
    "StartGrammar" => NonterminalId(44), "StartLayoutDef" => NonterminalId(45),
    "StartSyntaxRule" => NonterminalId(46), "StartRegexBlock" => NonterminalId(47),
    "StartRegexRule" => NonterminalId(48), "StartExcept" => NonterminalId(49),
    "StartPriorityLevel" => NonterminalId(50), "StartAssociativity" => NonterminalId(51),
    "StartAlternative" => NonterminalId(52), "StartSymbol" => NonterminalId(53),
    "StartRegex" => NonterminalId(54), "StartCharClass" => NonterminalId(55),
    "StartRangeElement" => NonterminalId(56), "StartRange" => NonterminalId(57), "Symbol"
    => NonterminalId(58)
};
pub const TERMINALS: [Terminal; 34] = [
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
    Terminal { name: "\"regex\"" },
    Terminal { name: "\"{\"" },
    Terminal { name: "\"}\"" },
    Terminal { name: "\"|\"" },
    Terminal { name: "\"\\\"" },
    Terminal { name: "\"left\"" },
    Terminal { name: "\"right\"" },
    Terminal { name: "\"none\"" },
    Terminal { name: "\"(\"" },
    Terminal { name: "\")\"" },
    Terminal { name: "\"\"\"" },
    Terminal { name: "\"*\"" },
    Terminal { name: "\"+\"" },
    Terminal { name: "\"?\"" },
    Terminal { name: "\"!>>\"" },
    Terminal { name: "\":\"" },
    Terminal { name: "\"'\"" },
    Terminal { name: "\"!\"" },
    Terminal { name: "\"[\"" },
    Terminal { name: "\"]\"" },
    Terminal { name: "\"-\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 390] = [
    Slot {
        display_name: "Grammar : . \"grammar\" Layout name:Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" . Layout name:Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout . name:Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier . Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout . LayoutDef? Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? . Layout SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? Layout . SyntaxRule* Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? Layout SyntaxRule* . Layout RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? Layout SyntaxRule* Layout . RegexBlock?",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout name:Identifier Layout LayoutDef? Layout SyntaxRule* Layout RegexBlock?.",
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
        display_name: "SyntaxRule : . head:Identifier Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : head:Identifier . Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : head:Identifier Layout . \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : head:Identifier Layout \"=\" . Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : head:Identifier Layout \"=\" Layout . {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "SyntaxRule : head:Identifier Layout \"=\" Layout {PriorityLevel \">\"}*.",
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
        display_name: "RegexRule : . Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout Except?",
    },
    Slot {
        display_name: "RegexRule : Identifier . Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout Except?",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout . \"=\" Layout body:{Regex+ \"|\"}+ Layout Except?",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" . Layout body:{Regex+ \"|\"}+ Layout Except?",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout . body:{Regex+ \"|\"}+ Layout Except?",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ . Layout Except?",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout . Except?",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout Except?.",
    },
    Slot {
        display_name: "Except : . \"\\\" Layout Identifier",
    },
    Slot {
        display_name: "Except : \"\\\" . Layout Identifier",
    },
    Slot {
        display_name: "Except : \"\\\" Layout . Identifier",
    },
    Slot {
        display_name: "Except : \"\\\" Layout Identifier.",
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
        display_name: "Symbol : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . \"*\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"*\" . return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"*\" return 0.",
    },
    Slot {
        display_name: "Symbol : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . \"+\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"+\" . return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"+\" return 0.",
    },
    Slot {
        display_name: "Symbol : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . \"?\" return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"?\" . return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"?\" return 0.",
    },
    Slot {
        display_name: "Symbol : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"\\\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout \"\\\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout \"\\\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout \"\\\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . \"\\\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"\\\" . Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"\\\" Layout . Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"\\\" Layout Identifier . return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"\\\" Layout Identifier return 0.",
    },
    Slot {
        display_name: "Symbol : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . \"!>>\" Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"!>>\" . Layout Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"!>>\" Layout . Identifier return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"!>>\" Layout Identifier . return 0",
    },
    Slot {
        display_name: "Symbol : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout \"!>>\" Layout Identifier return 0.",
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
        display_name: "Except? : . Except",
    },
    Slot {
        display_name: "Except? : Except.",
    },
    Slot {
        display_name: "Except? : .",
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
        display_name: "StartRegexBlock : . Layout start:RegexBlock Layout",
    },
    Slot {
        display_name: "StartRegexBlock : Layout . start:RegexBlock Layout",
    },
    Slot {
        display_name: "StartRegexBlock : Layout start:RegexBlock . Layout",
    },
    Slot {
        display_name: "StartRegexBlock : Layout start:RegexBlock Layout.",
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
        display_name: "StartExcept : . Layout start:Except Layout",
    },
    Slot {
        display_name: "StartExcept : Layout . start:Except Layout",
    },
    Slot {
        display_name: "StartExcept : Layout start:Except . Layout",
    },
    Slot {
        display_name: "StartExcept : Layout start:Except Layout.",
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
            //Grammar : . "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(0) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(7), i, j);
                        //Grammar : "grammar" . Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" . Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(1) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Grammar : "grammar" Layout . name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout . name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(2) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Grammar : "grammar" Layout name:Identifier . Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout name:Identifier . Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(3) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Grammar : "grammar" Layout name:Identifier Layout . Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout name:Identifier Layout . Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(4) => {
                self.create_grammar_opt_0(result, gss_node_id, SlotId(5));
            }
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 . Layout Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(5) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout . Grammar_Star_0 Layout Grammar_Opt_2
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
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout . Grammar_Star_0 Layout Grammar_Opt_2
            SlotId(6) => {
                self.create_grammar_star_0(result, gss_node_id, SlotId(7));
            }
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 . Layout Grammar_Opt_2
            SlotId(7) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout . Grammar_Opt_2
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout . Grammar_Opt_2
            SlotId(8) => {
                self.create_grammar_opt_2(result, gss_node_id, SlotId(9));
            }
            //Grammar : "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //LayoutDef : . "layout" Layout LayoutDef_Star_1
            SlotId(10) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"layout\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"layout\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //LayoutDef : "layout" . Layout LayoutDef_Star_1
                        let next_slot_id = SlotId(11);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //SyntaxRule : . head:Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(14) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //SyntaxRule : head:Identifier . Layout "=" Layout SyntaxRule_Star_2
                        let next_slot_id = SlotId(15);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //SyntaxRule : head:Identifier . Layout "=" Layout SyntaxRule_Star_2
            SlotId(15) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //SyntaxRule : head:Identifier Layout . "=" Layout SyntaxRule_Star_2
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //SyntaxRule : head:Identifier Layout . "=" Layout SyntaxRule_Star_2
            SlotId(16) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //SyntaxRule : head:Identifier Layout "=" . Layout SyntaxRule_Star_2
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //SyntaxRule : head:Identifier Layout "=" . Layout SyntaxRule_Star_2
            SlotId(17) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //SyntaxRule : head:Identifier Layout "=" Layout . SyntaxRule_Star_2
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
            //SyntaxRule : head:Identifier Layout "=" Layout . SyntaxRule_Star_2
            SlotId(18) => {
                self.create_syntax_rule_star_2(result, gss_node_id, SlotId(19));
            }
            //SyntaxRule : head:Identifier Layout "=" Layout SyntaxRule_Star_2.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexBlock : . "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}"
            SlotId(20) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"regex\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"regex\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //RegexBlock : "regex" . Layout "{" Layout RegexBlock_Star_3 Layout "}"
                        let next_slot_id = SlotId(21);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
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
            //RegexBlock : "regex" Layout . "{" Layout RegexBlock_Star_3 Layout "}"
            SlotId(22) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //RegexRule : . Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
            SlotId(28) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //RegexRule : Identifier . Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
                        let next_slot_id = SlotId(29);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //RegexRule : Identifier . Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
            SlotId(29) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //RegexRule : Identifier Layout . "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
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
            //RegexRule : Identifier Layout . "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
            SlotId(30) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //RegexRule : Identifier Layout "=" . Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
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
                            "\"=\"",
                            i,
                            SlotId(30),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier Layout "=" . Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
            SlotId(31) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //RegexRule : Identifier Layout "=" Layout . body:RegexRule_Plus_4 Layout RegexRule_Opt_6
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
            //RegexRule : Identifier Layout "=" Layout . body:RegexRule_Plus_4 Layout RegexRule_Opt_6
            SlotId(32) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(33));
            }
            //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 . Layout RegexRule_Opt_6
            SlotId(33) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout . RegexRule_Opt_6
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout . RegexRule_Opt_6
            SlotId(34) => {
                self.create_regex_rule_opt_6(result, gss_node_id, SlotId(35));
            }
            //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6.
            SlotId(35) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(35);
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
            //Except : . "\" Layout Identifier
            SlotId(36) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\\\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\\\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Except : "\" . Layout Identifier
                        let next_slot_id = SlotId(37);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\\\"",
                            i,
                            SlotId(36),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Except : "\" . Layout Identifier
            SlotId(37) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Except : "\" Layout . Identifier
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
            //Except : "\" Layout . Identifier
            SlotId(38) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Except : "\" Layout Identifier.
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
                            "Identifier",
                            i,
                            SlotId(38),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Except : "\" Layout Identifier.
            SlotId(39) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(39);
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
            SlotId(40) => {
                self.create_priority_level_opt_7(result, gss_node_id, SlotId(41));
            }
            //PriorityLevel : PriorityLevel_Opt_7 . Layout PriorityLevel_Star_4
            SlotId(41) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //PriorityLevel : PriorityLevel_Opt_7 Layout . PriorityLevel_Star_4
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
            //PriorityLevel : PriorityLevel_Opt_7 Layout . PriorityLevel_Star_4
            SlotId(42) => {
                self.create_priority_level_star_4(result, gss_node_id, SlotId(43));
            }
            //PriorityLevel : PriorityLevel_Opt_7 Layout PriorityLevel_Star_4.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Associativity : . "left"
            SlotId(44) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"left\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"left\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Associativity : "left".
                        let next_slot_id = SlotId(45);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"left\"",
                            i,
                            SlotId(44),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "left".
            SlotId(45) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
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
            //Associativity : . "right"
            SlotId(46) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"right\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"right\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Associativity : "right".
                        let next_slot_id = SlotId(47);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"right\"",
                            i,
                            SlotId(46),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "right".
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Associativity : . "none"
            SlotId(48) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"none\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"none\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Associativity : "none".
                        let next_slot_id = SlotId(49);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"none\"",
                            i,
                            SlotId(48),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "none".
            SlotId(49) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
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
            //Alternative : . Alternative_Star_5 Layout Alternative_Opt_10
            SlotId(50) => {
                self.create_alternative_star_5(result, gss_node_id, SlotId(51));
            }
            //Alternative : Alternative_Star_5 . Layout Alternative_Opt_10
            SlotId(51) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Alternative : Alternative_Star_5 Layout . Alternative_Opt_10
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
            //Alternative : Alternative_Star_5 Layout . Alternative_Opt_10
            SlotId(52) => {
                self.create_alternative_opt_10(result, gss_node_id, SlotId(53));
            }
            //Alternative : Alternative_Star_5 Layout Alternative_Opt_10.
            SlotId(53) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
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
            //Symbol(p: i32) : . Identifier return 0
            SlotId(54) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : Identifier . return 0
                        let next_slot_id = SlotId(55);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(54),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : Identifier . return 0
            SlotId(55) => {
                self.execute(input_index, SlotId(56), result, gss_node_id, env);
            }
            //Symbol(p: i32) : Identifier return 0.
            SlotId(56) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(56);
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
            SlotId(57) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Symbol(p: i32) : "(" . Layout Alternative_Plus_7 Layout ")" return 0
                        let next_slot_id = SlotId(58);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(57),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" . Layout Alternative_Plus_7 Layout ")" return 0
            SlotId(58) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "(" Layout . Alternative_Plus_7 Layout ")" return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(58),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout . Alternative_Plus_7 Layout ")" return 0
            SlotId(59) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(60));
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 . Layout ")" return 0
            SlotId(60) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout . ")" return 0
                        let next_slot_id = SlotId(61);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(60),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout . ")" return 0
            SlotId(61) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" . return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(61),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" . return 0
            SlotId(62) => {
                self.execute(input_index, SlotId(63), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_7 Layout ")" return 0.
            SlotId(63) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(63);
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
            SlotId(64) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Symbol(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(65);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(64),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(65) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Symbol(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(66) => {
                self.create_symbol(result, gss_node_id, SlotId(67), env, None, 0);
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) . Layout rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(67) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_8 Layout ")" return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_8 Layout ")" return 0
            SlotId(68) => {
                self.create_symbol_plus_8(result, gss_node_id, SlotId(69));
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 . Layout ")" return 0
            SlotId(69) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout . ")" return 0
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
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout . ")" return 0
            SlotId(70) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" . return 0
                        let next_slot_id = SlotId(71);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(70),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" . return 0
            SlotId(71) => {
                self.execute(input_index, SlotId(72), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0.
            SlotId(72) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(72);
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
            SlotId(73) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol(p: i32) : """ . Layout String Layout """ return 0
                        let next_slot_id = SlotId(74);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(73),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ . Layout String Layout """ return 0
            SlotId(74) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : """ Layout . String Layout """ return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(74),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout . String Layout """ return 0
            SlotId(75) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Symbol(p: i32) : """ Layout String . Layout """ return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "String",
                            i,
                            SlotId(75),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout String . Layout """ return 0
            SlotId(76) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : """ Layout String Layout . """ return 0
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
            //Symbol(p: i32) : """ Layout String Layout . """ return 0
            SlotId(77) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol(p: i32) : """ Layout String Layout """ . return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(77),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout String Layout """ . return 0
            SlotId(78) => {
                self.execute(input_index, SlotId(79), result, gss_node_id, env);
            }
            //Symbol(p: i32) : """ Layout String Layout """ return 0.
            SlotId(79) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(79);
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
            SlotId(80) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(81);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(80),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(81) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(82) => {
                self.create_symbol(result, gss_node_id, SlotId(83), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(83) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(84) => {
                self.create_symbol(result, gss_node_id, SlotId(85), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "*" return 0
            SlotId(85) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
            SlotId(86) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
                        let next_slot_id = SlotId(87);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(86),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
            SlotId(87) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
            SlotId(88) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
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
                            "\"*\"",
                            i,
                            SlotId(88),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
            SlotId(89) => {
                self.execute(input_index, SlotId(90), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0.
            SlotId(90) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
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
            //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(91) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(92);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(91),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(92) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
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
            //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(93) => {
                self.create_symbol(result, gss_node_id, SlotId(94), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(94) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(95) => {
                self.create_symbol(result, gss_node_id, SlotId(96), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "+" return 0
            SlotId(96) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(96),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
            SlotId(97) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
                        let next_slot_id = SlotId(98);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(97),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
            SlotId(98) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
                        let next_slot_id = SlotId(99);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(98),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
            SlotId(99) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
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
                            "\"+\"",
                            i,
                            SlotId(99),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
            SlotId(100) => {
                self.execute(input_index, SlotId(101), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0.
            SlotId(101) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(101);
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
            //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" return 0
            SlotId(102) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(103), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "*" return 0
            SlotId(103) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(104),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "*" return 0
            SlotId(104) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(105), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "*" return 0
            SlotId(105) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "*" return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "*" return 0
            SlotId(106) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" . return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" . return 0
            SlotId(107) => {
                self.execute(input_index, SlotId(108), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" return 0.
            SlotId(108) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
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
            //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" return 0
            SlotId(109) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(110), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "+" return 0
            SlotId(110) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(111),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "+" return 0
            SlotId(111) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(112), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "+" return 0
            SlotId(112) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "+" return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "+" return 0
            SlotId(113) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" . return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(113),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" . return 0
            SlotId(114) => {
                self.execute(input_index, SlotId(115), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" return 0.
            SlotId(115) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(115);
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
            //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0
            SlotId(116) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(117), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0
            SlotId(117) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(118),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "?" return 0
            SlotId(118) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(119), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "?" return 0
            SlotId(119) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "?" return 0
                        let next_slot_id = SlotId(120);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(119),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "?" return 0
            SlotId(120) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" . return 0
                        let next_slot_id = SlotId(121);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(120),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" . return 0
            SlotId(121) => {
                self.execute(input_index, SlotId(122), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0.
            SlotId(122) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(122);
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
            //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier return 0
            SlotId(123) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(124), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier return 0
            SlotId(124) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(125),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "\" Layout Identifier return 0
            SlotId(125) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(126), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "\" Layout Identifier return 0
            SlotId(126) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "\" Layout Identifier return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(126),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "\" Layout Identifier return 0
            SlotId(127) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\\\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\\\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" . Layout Identifier return 0
                        let next_slot_id = SlotId(128);
                        let left_child_id = result.expect("Result should not be None.");
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
                            "\"\\\"",
                            i,
                            SlotId(127),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" . Layout Identifier return 0
            SlotId(128) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout . Identifier return 0
                        let next_slot_id = SlotId(129);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(128),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout . Identifier return 0
            SlotId(129) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier . return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(129),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier . return 0
            SlotId(130) => {
                self.execute(input_index, SlotId(131), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier return 0.
            SlotId(131) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(131);
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
            //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
            SlotId(132) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(133), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
            SlotId(133) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(134),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
            SlotId(134) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(135), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "!>>" Layout Identifier return 0
            SlotId(135) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "!>>" Layout Identifier return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "!>>" Layout Identifier return 0
            SlotId(136) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!>>\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!>>\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" . Layout Identifier return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!>>\"",
                            i,
                            SlotId(136),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" . Layout Identifier return 0
            SlotId(137) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout . Identifier return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout . Identifier return 0
            SlotId(138) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier . return 0
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
                            "Identifier",
                            i,
                            SlotId(138),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier . return 0
            SlotId(139) => {
                self.execute(input_index, SlotId(140), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0.
            SlotId(140) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
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
            //Symbol(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
            SlotId(141) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
                        let next_slot_id = SlotId(142);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(141),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
            SlotId(142) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
                        let next_slot_id = SlotId(143);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(142),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
            SlotId(143) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\":\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\":\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Symbol(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
                        let next_slot_id = SlotId(144);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(143),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
            SlotId(144) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
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
            //Symbol(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
            SlotId(145) => {
                self.create_symbol(result, gss_node_id, SlotId(146), env, None, 1);
            }
            //Symbol(p: i32) : label:Identifier Layout ":" Layout Symbol(1) . return 1
            SlotId(146) => {
                self.execute(input_index, SlotId(147), result, gss_node_id, env);
            }
            //Symbol(p: i32) : label:Identifier Layout ":" Layout Symbol(1) return 1.
            SlotId(147) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(147);
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
            SlotId(148) => {
                self.create_regex(result, gss_node_id, SlotId(149));
            }
            //Regex : Regex . Layout "+"
            SlotId(149) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : Regex Layout . "+"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Regex : Regex Layout . "+"
            SlotId(150) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Regex : Regex Layout "+".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(150),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "+".
            SlotId(151) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(151);
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
            SlotId(152) => {
                self.create_regex(result, gss_node_id, SlotId(153));
            }
            //Regex : Regex . Layout "*"
            SlotId(153) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : Regex Layout . "*"
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
            //Regex : Regex Layout . "*"
            SlotId(154) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Regex : Regex Layout "*".
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
                            "\"*\"",
                            i,
                            SlotId(154),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "*".
            SlotId(155) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(155);
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
            SlotId(156) => {
                self.create_regex(result, gss_node_id, SlotId(157));
            }
            //Regex : Regex . Layout "?"
            SlotId(157) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : Regex Layout . "?"
                        let next_slot_id = SlotId(158);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(157),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout . "?"
            SlotId(158) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Regex : Regex Layout "?".
                        let next_slot_id = SlotId(159);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(158),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "?".
            SlotId(159) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(159);
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
            //Regex : . "(" Layout first:Regex Layout rest:Regex_Plus_9 Layout ")"
            SlotId(160) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Regex : "(" . Layout first:Regex Layout rest:Regex_Plus_9 Layout ")"
                        let next_slot_id = SlotId(161);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(160),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout first:Regex Layout rest:Regex_Plus_9 Layout ")"
            SlotId(161) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : "(" Layout . first:Regex Layout rest:Regex_Plus_9 Layout ")"
                        let next_slot_id = SlotId(162);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(161),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout . first:Regex Layout rest:Regex_Plus_9 Layout ")"
            SlotId(162) => {
                self.create_regex(result, gss_node_id, SlotId(163));
            }
            //Regex : "(" Layout first:Regex . Layout rest:Regex_Plus_9 Layout ")"
            SlotId(163) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : "(" Layout first:Regex Layout . rest:Regex_Plus_9 Layout ")"
                        let next_slot_id = SlotId(164);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(163),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout first:Regex Layout . rest:Regex_Plus_9 Layout ")"
            SlotId(164) => {
                self.create_regex_plus_9(result, gss_node_id, SlotId(165));
            }
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_9 . Layout ")"
            SlotId(165) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_9 Layout . ")"
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
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_9 Layout . ")"
            SlotId(166) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_9 Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(166),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_9 Layout ")".
            SlotId(167) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(167);
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
            //Regex : . "(" Layout RegexRule_Plus_5 Layout ")"
            SlotId(168) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Regex : "(" . Layout RegexRule_Plus_5 Layout ")"
                        let next_slot_id = SlotId(169);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(168),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout RegexRule_Plus_5 Layout ")"
            SlotId(169) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : "(" Layout . RegexRule_Plus_5 Layout ")"
                        let next_slot_id = SlotId(170);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(169),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout . RegexRule_Plus_5 Layout ")"
            SlotId(170) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(171));
            }
            //Regex : "(" Layout RegexRule_Plus_5 . Layout ")"
            SlotId(171) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : "(" Layout RegexRule_Plus_5 Layout . ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(171),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_5 Layout . ")"
            SlotId(172) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //Regex : "(" Layout RegexRule_Plus_5 Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(172),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_5 Layout ")".
            SlotId(173) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(173);
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
            SlotId(174) => {
                self.create_char_class(result, gss_node_id, SlotId(175));
            }
            //Regex : CharClass.
            SlotId(175) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(175);
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
            SlotId(176) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"'\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"'\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Regex : "'" . Layout Char Layout "'"
                        let next_slot_id = SlotId(177);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"'\"",
                            i,
                            SlotId(176),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" . Layout Char Layout "'"
            SlotId(177) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : "'" Layout . Char Layout "'"
                        let next_slot_id = SlotId(178);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(177),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout . Char Layout "'"
            SlotId(178) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Regex : "'" Layout Char . Layout "'"
                        let next_slot_id = SlotId(179);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(178),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout Char . Layout "'"
            SlotId(179) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : "'" Layout Char Layout . "'"
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
            //Regex : "'" Layout Char Layout . "'"
            SlotId(180) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"'\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"'\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Regex : "'" Layout Char Layout "'".
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
                            "\"'\"",
                            i,
                            SlotId(180),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout Char Layout "'".
            SlotId(181) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
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
            //Regex : . """ Layout String Layout """
            SlotId(182) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Regex : """ . Layout String Layout """
                        let next_slot_id = SlotId(183);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(182),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ . Layout String Layout """
            SlotId(183) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : """ Layout . String Layout """
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
            //Regex : """ Layout . String Layout """
            SlotId(184) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Regex : """ Layout String . Layout """
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
                            "String",
                            i,
                            SlotId(184),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String . Layout """
            SlotId(185) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex : """ Layout String Layout . """
                        let next_slot_id = SlotId(186);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(185),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String Layout . """
            SlotId(186) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Regex : """ Layout String Layout """.
                        let next_slot_id = SlotId(187);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(186),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String Layout """.
            SlotId(187) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(187);
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
            //CharClass : . neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_10 Layout "]"
            SlotId(188) => {
                self.create_char_class_opt_11(result, gss_node_id, SlotId(189));
            }
            //CharClass : neg:CharClass_Opt_11 . Layout "[" Layout CharClass_Plus_10 Layout "]"
            SlotId(189) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout . "[" Layout CharClass_Plus_10 Layout "]"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //CharClass : neg:CharClass_Opt_11 Layout . "[" Layout CharClass_Plus_10 Layout "]"
            SlotId(190) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"[\"", i);
                match self.scanner.match_token(TerminalId(29), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"[\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(29), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" . Layout CharClass_Plus_10 Layout "]"
                        let next_slot_id = SlotId(191);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(190),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" . Layout CharClass_Plus_10 Layout "]"
            SlotId(191) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" Layout . CharClass_Plus_10 Layout "]"
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
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout . CharClass_Plus_10 Layout "]"
            SlotId(192) => {
                self.create_char_class_plus_10(result, gss_node_id, SlotId(193));
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_10 . Layout "]"
            SlotId(193) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_10 Layout . "]"
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
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_10 Layout . "]"
            SlotId(194) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"]\"", i);
                match self.scanner.match_token(TerminalId(30), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"]\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(30), i, j);
                        //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_10 Layout "]".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"]\"",
                            i,
                            SlotId(194),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_10 Layout "]".
            SlotId(195) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(195);
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
            SlotId(196) => {
                self.create_range(result, gss_node_id, SlotId(197));
            }
            //RangeElement : Range.
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
            //RangeElement : . RangeChar
            SlotId(198) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //RangeElement : RangeChar.
                        let next_slot_id = SlotId(199);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(198),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RangeElement : RangeChar.
            SlotId(199) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(199);
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
            SlotId(200) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Range : start:RangeChar . Layout "-" Layout end:RangeChar
                        let next_slot_id = SlotId(201);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(200),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar . Layout "-" Layout end:RangeChar
            SlotId(201) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Range : start:RangeChar Layout . "-" Layout end:RangeChar
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
            //Range : start:RangeChar Layout . "-" Layout end:RangeChar
            SlotId(202) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"-\"", i);
                match self.scanner.match_token(TerminalId(31), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"-\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(31), i, j);
                        //Range : start:RangeChar Layout "-" . Layout end:RangeChar
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
                            "\"-\"",
                            i,
                            SlotId(202),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" . Layout end:RangeChar
            SlotId(203) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Range : start:RangeChar Layout "-" Layout . end:RangeChar
                        let next_slot_id = SlotId(204);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(203),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" Layout . end:RangeChar
            SlotId(204) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Range : start:RangeChar Layout "-" Layout end:RangeChar.
                        let next_slot_id = SlotId(205);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(204),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" Layout end:RangeChar.
            SlotId(205) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
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
            //Grammar_Opt_0 : . LayoutDef
            SlotId(206) => {
                self.create_layout_def(result, gss_node_id, SlotId(207));
            }
            //Grammar_Opt_0 : LayoutDef.
            SlotId(207) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
                let end_slot_id = SlotId(207);
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
            SlotId(208) => {
                let end_slot_id = SlotId(208);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
                let nonterminal_id = NonterminalId(13);
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
            //Grammar_Plus_0 : . Grammar_Plus_0 Layout SyntaxRule
            SlotId(209) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(210));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 . Layout SyntaxRule
            SlotId(210) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
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
                            "Layout",
                            i,
                            SlotId(210),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
            SlotId(211) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(212));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout SyntaxRule.
            SlotId(212) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(212);
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
            //Grammar_Plus_0 : . SyntaxRule
            SlotId(213) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(214));
            }
            //Grammar_Plus_0 : SyntaxRule.
            SlotId(214) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(214);
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
            SlotId(215) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(216));
            }
            //Grammar_Opt_1 : Grammar_Plus_0.
            SlotId(216) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
                let end_slot_id = SlotId(216);
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
            SlotId(217) => {
                let end_slot_id = SlotId(217);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
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
            //Grammar_Star_0 : . Grammar_Opt_1
            SlotId(218) => {
                self.create_grammar_opt_1(result, gss_node_id, SlotId(219));
            }
            //Grammar_Star_0 : Grammar_Opt_1.
            SlotId(219) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
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
            //Grammar_Opt_2 : . RegexBlock
            SlotId(220) => {
                self.create_regex_block(result, gss_node_id, SlotId(221));
            }
            //Grammar_Opt_2 : RegexBlock.
            SlotId(221) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(221);
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
            //Grammar_Opt_2 : .
            SlotId(222) => {
                let end_slot_id = SlotId(222);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
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
            //LayoutDef_Plus_1 : . LayoutDef_Plus_1 Layout Identifier
            SlotId(223) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(224));
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 . Layout Identifier
            SlotId(224) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
                        let next_slot_id = SlotId(225);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(224),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
            SlotId(225) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
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
                            "Identifier",
                            i,
                            SlotId(225),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
            SlotId(226) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
                let end_slot_id = SlotId(226);
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
            SlotId(227) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //LayoutDef_Plus_1 : Identifier.
                        let next_slot_id = SlotId(228);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(227),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : Identifier.
            SlotId(228) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
                let end_slot_id = SlotId(228);
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
            //LayoutDef_Opt_3 : . LayoutDef_Plus_1
            SlotId(229) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(230));
            }
            //LayoutDef_Opt_3 : LayoutDef_Plus_1.
            SlotId(230) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(230);
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
            //LayoutDef_Opt_3 : .
            SlotId(231) => {
                let end_slot_id = SlotId(231);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
                let nonterminal_id = NonterminalId(19);
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
            //LayoutDef_Star_1 : . LayoutDef_Opt_3
            SlotId(232) => {
                self.create_layout_def_opt_3(result, gss_node_id, SlotId(233));
            }
            //LayoutDef_Star_1 : LayoutDef_Opt_3.
            SlotId(233) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
                let end_slot_id = SlotId(233);
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
            //SyntaxRule_Plus_2 : . SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel
            SlotId(234) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(235));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 . Layout ">" Layout PriorityLevel
            SlotId(235) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
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
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
            SlotId(236) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
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
                            "\">\"",
                            i,
                            SlotId(236),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
            SlotId(237) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
                        let next_slot_id = SlotId(238);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(237),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
            SlotId(238) => {
                self.create_priority_level(result, gss_node_id, SlotId(239));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel.
            SlotId(239) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
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
            //SyntaxRule_Plus_2 : . PriorityLevel
            SlotId(240) => {
                self.create_priority_level(result, gss_node_id, SlotId(241));
            }
            //SyntaxRule_Plus_2 : PriorityLevel.
            SlotId(241) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
                let end_slot_id = SlotId(241);
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
            SlotId(242) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(243));
            }
            //SyntaxRule_Opt_4 : SyntaxRule_Plus_2.
            SlotId(243) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
                let end_slot_id = SlotId(243);
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
            SlotId(244) => {
                let end_slot_id = SlotId(244);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
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
            //SyntaxRule_Star_2 : . SyntaxRule_Opt_4
            SlotId(245) => {
                self.create_syntax_rule_opt_4(result, gss_node_id, SlotId(246));
            }
            //SyntaxRule_Star_2 : SyntaxRule_Opt_4.
            SlotId(246) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
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
            //RegexBlock_Plus_3 : . RegexBlock_Plus_3 Layout RegexRule
            SlotId(247) => {
                self.create_regex_block_plus_3(result, gss_node_id, SlotId(248));
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 . Layout RegexRule
            SlotId(248) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout . RegexRule
                        let next_slot_id = SlotId(249);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(248),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout . RegexRule
            SlotId(249) => {
                self.create_regex_rule(result, gss_node_id, SlotId(250));
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout RegexRule.
            SlotId(250) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
                let end_slot_id = SlotId(250);
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
            //RegexBlock_Plus_3 : . RegexRule
            SlotId(251) => {
                self.create_regex_rule(result, gss_node_id, SlotId(252));
            }
            //RegexBlock_Plus_3 : RegexRule.
            SlotId(252) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
                let end_slot_id = SlotId(252);
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
            //RegexBlock_Opt_5 : . RegexBlock_Plus_3
            SlotId(253) => {
                self.create_regex_block_plus_3(result, gss_node_id, SlotId(254));
            }
            //RegexBlock_Opt_5 : RegexBlock_Plus_3.
            SlotId(254) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(254);
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
            //RegexBlock_Opt_5 : .
            SlotId(255) => {
                let end_slot_id = SlotId(255);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
                let nonterminal_id = NonterminalId(25);
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
            //RegexBlock_Star_3 : . RegexBlock_Opt_5
            SlotId(256) => {
                self.create_regex_block_opt_5(result, gss_node_id, SlotId(257));
            }
            //RegexBlock_Star_3 : RegexBlock_Opt_5.
            SlotId(257) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
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
            //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout Regex
            SlotId(258) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(259));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 . Layout Regex
            SlotId(259) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . Regex
                        let next_slot_id = SlotId(260);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(259),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . Regex
            SlotId(260) => {
                self.create_regex(result, gss_node_id, SlotId(261));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout Regex.
            SlotId(261) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
                let end_slot_id = SlotId(261);
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
            //RegexRule_Plus_5 : . Regex
            SlotId(262) => {
                self.create_regex(result, gss_node_id, SlotId(263));
            }
            //RegexRule_Plus_5 : Regex.
            SlotId(263) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
                let end_slot_id = SlotId(263);
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
            //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5
            SlotId(264) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(265));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 . Layout "|" Layout RegexRule_Plus_5
            SlotId(265) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . "|" Layout RegexRule_Plus_5
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . "|" Layout RegexRule_Plus_5
            SlotId(266) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" . Layout RegexRule_Plus_5
                        let next_slot_id = SlotId(267);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(266),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" . Layout RegexRule_Plus_5
            SlotId(267) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout . RegexRule_Plus_5
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
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout . RegexRule_Plus_5
            SlotId(268) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(269));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5.
            SlotId(269) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
                let end_slot_id = SlotId(269);
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
            //RegexRule_Plus_4 : . RegexRule_Plus_5
            SlotId(270) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(271));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_5.
            SlotId(271) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
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
            //RegexRule_Opt_6 : . Except
            SlotId(272) => {
                self.create_except(result, gss_node_id, SlotId(273));
            }
            //RegexRule_Opt_6 : Except.
            SlotId(273) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
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
            //RegexRule_Opt_6 : .
            SlotId(274) => {
                let end_slot_id = SlotId(274);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
                let nonterminal_id = NonterminalId(29);
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
            //PriorityLevel_Opt_7 : . Associativity
            SlotId(275) => {
                self.create_associativity(result, gss_node_id, SlotId(276));
            }
            //PriorityLevel_Opt_7 : Associativity.
            SlotId(276) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(30);
                let end_slot_id = SlotId(276);
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
            SlotId(277) => {
                let end_slot_id = SlotId(277);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
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
            //PriorityLevel_Plus_6 : . PriorityLevel_Plus_6 Layout "|" Layout Alternative
            SlotId(278) => {
                self.create_priority_level_plus_6(result, gss_node_id, SlotId(279));
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 . Layout "|" Layout Alternative
            SlotId(279) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout . "|" Layout Alternative
                        let next_slot_id = SlotId(280);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(279),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout . "|" Layout Alternative
            SlotId(280) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" . Layout Alternative
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(280),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" . Layout Alternative
            SlotId(281) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout . Alternative
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout . Alternative
            SlotId(282) => {
                self.create_alternative(result, gss_node_id, SlotId(283));
            }
            //PriorityLevel_Plus_6 : PriorityLevel_Plus_6 Layout "|" Layout Alternative.
            SlotId(283) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
                let end_slot_id = SlotId(283);
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
            SlotId(284) => {
                self.create_alternative(result, gss_node_id, SlotId(285));
            }
            //PriorityLevel_Plus_6 : Alternative.
            SlotId(285) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
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
            //PriorityLevel_Opt_8 : . PriorityLevel_Plus_6
            SlotId(286) => {
                self.create_priority_level_plus_6(result, gss_node_id, SlotId(287));
            }
            //PriorityLevel_Opt_8 : PriorityLevel_Plus_6.
            SlotId(287) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(32);
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
            //PriorityLevel_Opt_8 : .
            SlotId(288) => {
                let end_slot_id = SlotId(288);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
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
            //PriorityLevel_Star_4 : . PriorityLevel_Opt_8
            SlotId(289) => {
                self.create_priority_level_opt_8(result, gss_node_id, SlotId(290));
            }
            //PriorityLevel_Star_4 : PriorityLevel_Opt_8.
            SlotId(290) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
                let end_slot_id = SlotId(290);
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
            SlotId(291) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(292));
            }
            //Alternative_Plus_7 : Alternative_Plus_7 . Layout Symbol(0)
            SlotId(292) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Alternative_Plus_7 : Alternative_Plus_7 Layout . Symbol(0)
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Alternative_Plus_7 : Alternative_Plus_7 Layout . Symbol(0)
            SlotId(293) => {
                self.create_symbol(result, gss_node_id, SlotId(294), env, None, 0);
            }
            //Alternative_Plus_7 : Alternative_Plus_7 Layout Symbol(0).
            SlotId(294) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
                let end_slot_id = SlotId(294);
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
            SlotId(295) => {
                self.create_symbol(result, gss_node_id, SlotId(296), env, None, 0);
            }
            //Alternative_Plus_7 : Symbol(0).
            SlotId(296) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
                let end_slot_id = SlotId(296);
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
            SlotId(297) => {
                self.create_alternative_plus_7(result, gss_node_id, SlotId(298));
            }
            //Alternative_Opt_9 : Alternative_Plus_7.
            SlotId(298) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(35);
                let end_slot_id = SlotId(298);
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
            SlotId(299) => {
                let end_slot_id = SlotId(299);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
                let nonterminal_id = NonterminalId(35);
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
            SlotId(300) => {
                self.create_alternative_opt_9(result, gss_node_id, SlotId(301));
            }
            //Alternative_Star_5 : Alternative_Opt_9.
            SlotId(301) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(36);
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
            //Alternative_Opt_10 : . Label
            SlotId(302) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Label", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Label", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(5), i, j);
                        //Alternative_Opt_10 : Label.
                        let next_slot_id = SlotId(303);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Label",
                            i,
                            SlotId(302),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Opt_10 : Label.
            SlotId(303) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(37);
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
            //Alternative_Opt_10 : .
            SlotId(304) => {
                let end_slot_id = SlotId(304);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
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
            //Symbol_Group_0 : . "|" Layout Symbol(0)
            SlotId(305) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol_Group_0 : "|" . Layout Symbol(0)
                        let next_slot_id = SlotId(306);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(305),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" . Layout Symbol(0)
            SlotId(306) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol_Group_0 : "|" Layout . Symbol(0)
                        let next_slot_id = SlotId(307);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(306),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" Layout . Symbol(0)
            SlotId(307) => {
                self.create_symbol(result, gss_node_id, SlotId(308), env, None, 0);
            }
            //Symbol_Group_0 : "|" Layout Symbol(0).
            SlotId(308) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(38);
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
            //Symbol_Plus_8 : . Symbol_Plus_8 Layout Symbol_Group_0
            SlotId(309) => {
                self.create_symbol_plus_8(result, gss_node_id, SlotId(310));
            }
            //Symbol_Plus_8 : Symbol_Plus_8 . Layout Symbol_Group_0
            SlotId(310) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Symbol_Plus_8 : Symbol_Plus_8 Layout . Symbol_Group_0
                        let next_slot_id = SlotId(311);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(310),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Plus_8 : Symbol_Plus_8 Layout . Symbol_Group_0
            SlotId(311) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(312));
            }
            //Symbol_Plus_8 : Symbol_Plus_8 Layout Symbol_Group_0.
            SlotId(312) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(39);
                let end_slot_id = SlotId(312);
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
            SlotId(313) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(314));
            }
            //Symbol_Plus_8 : Symbol_Group_0.
            SlotId(314) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(39);
                let end_slot_id = SlotId(314);
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
            //Regex_Group_1 : . "|" Layout Regex
            SlotId(315) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Regex_Group_1 : "|" . Layout Regex
                        let next_slot_id = SlotId(316);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(315),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_1 : "|" . Layout Regex
            SlotId(316) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex_Group_1 : "|" Layout . Regex
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Regex_Group_1 : "|" Layout . Regex
            SlotId(317) => {
                self.create_regex(result, gss_node_id, SlotId(318));
            }
            //Regex_Group_1 : "|" Layout Regex.
            SlotId(318) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(40);
                let end_slot_id = SlotId(318);
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
            //Regex_Plus_9 : . Regex_Plus_9 Layout Regex_Group_1
            SlotId(319) => {
                self.create_regex_plus_9(result, gss_node_id, SlotId(320));
            }
            //Regex_Plus_9 : Regex_Plus_9 . Layout Regex_Group_1
            SlotId(320) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //Regex_Plus_9 : Regex_Plus_9 Layout . Regex_Group_1
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Regex_Plus_9 : Regex_Plus_9 Layout . Regex_Group_1
            SlotId(321) => {
                self.create_regex_group_1(result, gss_node_id, SlotId(322));
            }
            //Regex_Plus_9 : Regex_Plus_9 Layout Regex_Group_1.
            SlotId(322) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(41);
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
            //Regex_Plus_9 : . Regex_Group_1
            SlotId(323) => {
                self.create_regex_group_1(result, gss_node_id, SlotId(324));
            }
            //Regex_Plus_9 : Regex_Group_1.
            SlotId(324) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(41);
                let end_slot_id = SlotId(324);
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
            SlotId(325) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(28), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(28), i, j);
                        //CharClass_Opt_11 : "!".
                        let next_slot_id = SlotId(326);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(325),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Opt_11 : "!".
            SlotId(326) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(42);
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
            //CharClass_Opt_11 : .
            SlotId(327) => {
                let end_slot_id = SlotId(327);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(33), input_index, input_index);
                let nonterminal_id = NonterminalId(42);
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
            //CharClass_Plus_10 : . CharClass_Plus_10 Layout RangeElement
            SlotId(328) => {
                self.create_char_class_plus_10(result, gss_node_id, SlotId(329));
            }
            //CharClass_Plus_10 : CharClass_Plus_10 . Layout RangeElement
            SlotId(329) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //CharClass_Plus_10 : CharClass_Plus_10 Layout . RangeElement
                        let next_slot_id = SlotId(330);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(329),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Plus_10 : CharClass_Plus_10 Layout . RangeElement
            SlotId(330) => {
                self.create_range_element(result, gss_node_id, SlotId(331));
            }
            //CharClass_Plus_10 : CharClass_Plus_10 Layout RangeElement.
            SlotId(331) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
                let end_slot_id = SlotId(331);
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
            //CharClass_Plus_10 : . RangeElement
            SlotId(332) => {
                self.create_range_element(result, gss_node_id, SlotId(333));
            }
            //CharClass_Plus_10 : RangeElement.
            SlotId(333) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
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
            //StartGrammar : . Layout start:Grammar Layout
            SlotId(334) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartGrammar : Layout . start:Grammar Layout
                        let next_slot_id = SlotId(335);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(334),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout . start:Grammar Layout
            SlotId(335) => {
                self.create_grammar(result, gss_node_id, SlotId(336));
            }
            //StartGrammar : Layout start:Grammar . Layout
            SlotId(336) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartGrammar : Layout start:Grammar Layout.
                        let next_slot_id = SlotId(337);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(336),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout start:Grammar Layout.
            SlotId(337) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(44);
                let end_slot_id = SlotId(337);
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
            SlotId(338) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartLayoutDef : Layout . start:LayoutDef Layout
                        let next_slot_id = SlotId(339);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartLayoutDef : Layout . start:LayoutDef Layout
            SlotId(339) => {
                self.create_layout_def(result, gss_node_id, SlotId(340));
            }
            //StartLayoutDef : Layout start:LayoutDef . Layout
            SlotId(340) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartLayoutDef : Layout start:LayoutDef Layout.
                        let next_slot_id = SlotId(341);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(340),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartLayoutDef : Layout start:LayoutDef Layout.
            SlotId(341) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(45);
                let end_slot_id = SlotId(341);
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
            SlotId(342) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartSyntaxRule : Layout . start:SyntaxRule Layout
                        let next_slot_id = SlotId(343);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartSyntaxRule : Layout . start:SyntaxRule Layout
            SlotId(343) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(344));
            }
            //StartSyntaxRule : Layout start:SyntaxRule . Layout
            SlotId(344) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartSyntaxRule : Layout start:SyntaxRule Layout.
                        let next_slot_id = SlotId(345);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(344),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout start:SyntaxRule Layout.
            SlotId(345) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(46);
                let end_slot_id = SlotId(345);
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
            //StartRegexBlock : . Layout start:RegexBlock Layout
            SlotId(346) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRegexBlock : Layout . start:RegexBlock Layout
                        let next_slot_id = SlotId(347);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(346),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexBlock : Layout . start:RegexBlock Layout
            SlotId(347) => {
                self.create_regex_block(result, gss_node_id, SlotId(348));
            }
            //StartRegexBlock : Layout start:RegexBlock . Layout
            SlotId(348) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRegexBlock : Layout start:RegexBlock Layout.
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
            //StartRegexBlock : Layout start:RegexBlock Layout.
            SlotId(349) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(47);
                let end_slot_id = SlotId(349);
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
            SlotId(350) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRegexRule : Layout . start:RegexRule Layout
                        let next_slot_id = SlotId(351);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(350),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout . start:RegexRule Layout
            SlotId(351) => {
                self.create_regex_rule(result, gss_node_id, SlotId(352));
            }
            //StartRegexRule : Layout start:RegexRule . Layout
            SlotId(352) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRegexRule : Layout start:RegexRule Layout.
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
            //StartRegexRule : Layout start:RegexRule Layout.
            SlotId(353) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(48);
                let end_slot_id = SlotId(353);
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
            //StartExcept : . Layout start:Except Layout
            SlotId(354) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartExcept : Layout . start:Except Layout
                        let next_slot_id = SlotId(355);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(354),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartExcept : Layout . start:Except Layout
            SlotId(355) => {
                self.create_except(result, gss_node_id, SlotId(356));
            }
            //StartExcept : Layout start:Except . Layout
            SlotId(356) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartExcept : Layout start:Except Layout.
                        let next_slot_id = SlotId(357);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(356),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartExcept : Layout start:Except Layout.
            SlotId(357) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(49);
                let end_slot_id = SlotId(357);
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
            SlotId(358) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartPriorityLevel : Layout . start:PriorityLevel Layout
                        let next_slot_id = SlotId(359);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartPriorityLevel : Layout . start:PriorityLevel Layout
            SlotId(359) => {
                self.create_priority_level(result, gss_node_id, SlotId(360));
            }
            //StartPriorityLevel : Layout start:PriorityLevel . Layout
            SlotId(360) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartPriorityLevel : Layout start:PriorityLevel Layout.
                        let next_slot_id = SlotId(361);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(360),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout start:PriorityLevel Layout.
            SlotId(361) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(50);
                let end_slot_id = SlotId(361);
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
            SlotId(362) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartAssociativity : Layout . start:Associativity Layout
                        let next_slot_id = SlotId(363);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartAssociativity : Layout . start:Associativity Layout
            SlotId(363) => {
                self.create_associativity(result, gss_node_id, SlotId(364));
            }
            //StartAssociativity : Layout start:Associativity . Layout
            SlotId(364) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartAssociativity : Layout start:Associativity Layout.
                        let next_slot_id = SlotId(365);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(364),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAssociativity : Layout start:Associativity Layout.
            SlotId(365) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(51);
                let end_slot_id = SlotId(365);
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
            SlotId(366) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartAlternative : Layout . start:Alternative Layout
                        let next_slot_id = SlotId(367);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(366),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout . start:Alternative Layout
            SlotId(367) => {
                self.create_alternative(result, gss_node_id, SlotId(368));
            }
            //StartAlternative : Layout start:Alternative . Layout
            SlotId(368) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartAlternative : Layout start:Alternative Layout.
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
            //StartAlternative : Layout start:Alternative Layout.
            SlotId(369) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(52);
                let end_slot_id = SlotId(369);
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
            SlotId(370) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartSymbol : Layout . start:Symbol(0) Layout
                        let next_slot_id = SlotId(371);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(370),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSymbol : Layout . start:Symbol(0) Layout
            SlotId(371) => {
                self.create_symbol(result, gss_node_id, SlotId(372), env, None, 0);
            }
            //StartSymbol : Layout start:Symbol(0) . Layout
            SlotId(372) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartSymbol : Layout start:Symbol(0) Layout.
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
            //StartSymbol : Layout start:Symbol(0) Layout.
            SlotId(373) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(53);
                let end_slot_id = SlotId(373);
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
            SlotId(374) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRegex : Layout . start:Regex Layout
                        let next_slot_id = SlotId(375);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(374),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout . start:Regex Layout
            SlotId(375) => {
                self.create_regex(result, gss_node_id, SlotId(376));
            }
            //StartRegex : Layout start:Regex . Layout
            SlotId(376) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRegex : Layout start:Regex Layout.
                        let next_slot_id = SlotId(377);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(376),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout start:Regex Layout.
            SlotId(377) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(54);
                let end_slot_id = SlotId(377);
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
            SlotId(378) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartCharClass : Layout . start:CharClass Layout
                        let next_slot_id = SlotId(379);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(378),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout . start:CharClass Layout
            SlotId(379) => {
                self.create_char_class(result, gss_node_id, SlotId(380));
            }
            //StartCharClass : Layout start:CharClass . Layout
            SlotId(380) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartCharClass : Layout start:CharClass Layout.
                        let next_slot_id = SlotId(381);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(380),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout start:CharClass Layout.
            SlotId(381) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(55);
                let end_slot_id = SlotId(381);
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
            SlotId(382) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRangeElement : Layout . start:RangeElement Layout
                        let next_slot_id = SlotId(383);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(382),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRangeElement : Layout . start:RangeElement Layout
            SlotId(383) => {
                self.create_range_element(result, gss_node_id, SlotId(384));
            }
            //StartRangeElement : Layout start:RangeElement . Layout
            SlotId(384) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRangeElement : Layout start:RangeElement Layout.
                        let next_slot_id = SlotId(385);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(384),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRangeElement : Layout start:RangeElement Layout.
            SlotId(385) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(56);
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
            //StartRange : . Layout start:Range Layout
            SlotId(386) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRange : Layout . start:Range Layout
                        let next_slot_id = SlotId(387);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(386),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout . start:Range Layout
            SlotId(387) => {
                self.create_range(result, gss_node_id, SlotId(388));
            }
            //StartRange : Layout start:Range . Layout
            SlotId(388) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //StartRange : Layout start:Range Layout.
                        let next_slot_id = SlotId(389);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(388),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRange : Layout start:Range Layout.
            SlotId(389) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(57);
                let end_slot_id = SlotId(389);
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
                //Grammar : . "grammar" Layout name:Identifier Layout Grammar_Opt_0 Layout Grammar_Star_0 Layout Grammar_Opt_2
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
                    slot_id: SlotId(10),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule
            NonterminalId(2) => {
                //SyntaxRule : . head:Identifier Layout "=" Layout SyntaxRule_Star_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
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
                    env,
                });
            }
            //RegexRule
            NonterminalId(4) => {
                //RegexRule : . Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Opt_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(28),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Except
            NonterminalId(5) => {
                //Except : . "\" Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(36),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel
            NonterminalId(6) => {
                //PriorityLevel : . PriorityLevel_Opt_7 Layout PriorityLevel_Star_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(40),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Associativity
            NonterminalId(7) => {
                //Associativity : . "left"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(44),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Associativity : . "right"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(46),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Associativity : . "none"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(48),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative
            NonterminalId(8) => {
                //Alternative : . Alternative_Star_5 Layout Alternative_Opt_10
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(50),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol
            NonterminalId(58) => {
                //Symbol(p: i32) : . Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(54),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "(" Layout Alternative_Plus_7 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(57),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_8 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(64),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . """ Layout String Layout """ return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(73),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(80),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(91),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(102),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(109),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(116),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(123),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(132),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(141),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex
            NonterminalId(9) => {
                //Regex : . Regex Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(148),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . Regex Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(152),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . Regex Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(156),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "(" Layout first:Regex Layout rest:Regex_Plus_9 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(160),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "(" Layout RegexRule_Plus_5 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(168),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . CharClass
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(174),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "'" Layout Char Layout "'"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(176),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . """ Layout String Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(182),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass
            NonterminalId(10) => {
                //CharClass : . neg:CharClass_Opt_11 Layout "[" Layout CharClass_Plus_10 Layout "]"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(188),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RangeElement
            NonterminalId(11) => {
                //RangeElement : . Range
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(196),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RangeElement : . RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(198),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Range
            NonterminalId(12) => {
                //Range : . start:RangeChar Layout "-" Layout end:RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(200),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_0
            NonterminalId(13) => {
                //Grammar_Opt_0 : . LayoutDef
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(206),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(208),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Plus_0
            NonterminalId(14) => {
                //Grammar_Plus_0 : . Grammar_Plus_0 Layout SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(209),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Plus_0 : . SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(213),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_1
            NonterminalId(15) => {
                //Grammar_Opt_1 : . Grammar_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(215),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(217),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Star_0
            NonterminalId(16) => {
                //Grammar_Star_0 : . Grammar_Opt_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(218),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_2
            NonterminalId(17) => {
                //Grammar_Opt_2 : . RegexBlock
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(220),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(222),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Plus_1
            NonterminalId(18) => {
                //LayoutDef_Plus_1 : . LayoutDef_Plus_1 Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(223),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //LayoutDef_Plus_1 : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(227),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Opt_3
            NonterminalId(19) => {
                //LayoutDef_Opt_3 : . LayoutDef_Plus_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(229),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //LayoutDef_Opt_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(231),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Star_1
            NonterminalId(20) => {
                //LayoutDef_Star_1 : . LayoutDef_Opt_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(232),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Plus_2
            NonterminalId(21) => {
                //SyntaxRule_Plus_2 : . SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(234),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Plus_2 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(240),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Opt_4
            NonterminalId(22) => {
                //SyntaxRule_Opt_4 : . SyntaxRule_Plus_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(242),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Opt_4 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(244),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Star_2
            NonterminalId(23) => {
                //SyntaxRule_Star_2 : . SyntaxRule_Opt_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(245),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexBlock_Plus_3
            NonterminalId(24) => {
                //RegexBlock_Plus_3 : . RegexBlock_Plus_3 Layout RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(247),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexBlock_Plus_3 : . RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(251),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexBlock_Opt_5
            NonterminalId(25) => {
                //RegexBlock_Opt_5 : . RegexBlock_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(253),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexBlock_Opt_5 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(255),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexBlock_Star_3
            NonterminalId(26) => {
                //RegexBlock_Star_3 : . RegexBlock_Opt_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(256),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_5
            NonterminalId(27) => {
                //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(258),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_5 : . Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(262),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_4
            NonterminalId(28) => {
                //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(264),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_4 : . RegexRule_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(270),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Opt_6
            NonterminalId(29) => {
                //RegexRule_Opt_6 : . Except
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(272),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Opt_6 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(274),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Opt_7
            NonterminalId(30) => {
                //PriorityLevel_Opt_7 : . Associativity
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(275),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Opt_7 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(277),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Plus_6
            NonterminalId(31) => {
                //PriorityLevel_Plus_6 : . PriorityLevel_Plus_6 Layout "|" Layout Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(278),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Plus_6 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(284),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Opt_8
            NonterminalId(32) => {
                //PriorityLevel_Opt_8 : . PriorityLevel_Plus_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(286),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Opt_8 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(288),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Star_4
            NonterminalId(33) => {
                //PriorityLevel_Star_4 : . PriorityLevel_Opt_8
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(289),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Plus_7
            NonterminalId(34) => {
                //Alternative_Plus_7 : . Alternative_Plus_7 Layout Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(291),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Plus_7 : . Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(295),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Opt_9
            NonterminalId(35) => {
                //Alternative_Opt_9 : . Alternative_Plus_7
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(297),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Opt_9 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(299),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Star_5
            NonterminalId(36) => {
                //Alternative_Star_5 : . Alternative_Opt_9
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(300),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Opt_10
            NonterminalId(37) => {
                //Alternative_Opt_10 : . Label
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(302),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Opt_10 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(304),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Group_0
            NonterminalId(38) => {
                //Symbol_Group_0 : . "|" Layout Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(305),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Plus_8
            NonterminalId(39) => {
                //Symbol_Plus_8 : . Symbol_Plus_8 Layout Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(309),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_Plus_8 : . Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(313),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex_Group_1
            NonterminalId(40) => {
                //Regex_Group_1 : . "|" Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(315),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex_Plus_9
            NonterminalId(41) => {
                //Regex_Plus_9 : . Regex_Plus_9 Layout Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(319),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex_Plus_9 : . Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(323),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass_Opt_11
            NonterminalId(42) => {
                //CharClass_Opt_11 : . "!"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(325),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //CharClass_Opt_11 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(327),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass_Plus_10
            NonterminalId(43) => {
                //CharClass_Plus_10 : . CharClass_Plus_10 Layout RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(328),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //CharClass_Plus_10 : . RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(332),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartGrammar
            NonterminalId(44) => {
                //StartGrammar : . Layout start:Grammar Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(334),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartLayoutDef
            NonterminalId(45) => {
                //StartLayoutDef : . Layout start:LayoutDef Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(338),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartSyntaxRule
            NonterminalId(46) => {
                //StartSyntaxRule : . Layout start:SyntaxRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(342),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegexBlock
            NonterminalId(47) => {
                //StartRegexBlock : . Layout start:RegexBlock Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(346),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegexRule
            NonterminalId(48) => {
                //StartRegexRule : . Layout start:RegexRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(350),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartExcept
            NonterminalId(49) => {
                //StartExcept : . Layout start:Except Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(354),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartPriorityLevel
            NonterminalId(50) => {
                //StartPriorityLevel : . Layout start:PriorityLevel Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(358),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAssociativity
            NonterminalId(51) => {
                //StartAssociativity : . Layout start:Associativity Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(362),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAlternative
            NonterminalId(52) => {
                //StartAlternative : . Layout start:Alternative Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(366),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartSymbol
            NonterminalId(53) => {
                //StartSymbol : . Layout start:Symbol(0) Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(370),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegex
            NonterminalId(54) => {
                //StartRegex : . Layout start:Regex Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(374),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartCharClass
            NonterminalId(55) => {
                //StartCharClass : . Layout start:CharClass Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(378),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRangeElement
            NonterminalId(56) => {
                //StartRangeElement : . Layout start:RangeElement Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(382),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRange
            NonterminalId(57) => {
                //StartRange : . Layout start:Range Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(386),
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
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 59],
    //GSS index for nonterminal Symbol
    gss_nodes_index_symbol: Vec<(u32, i32, GssNodeId)>,
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 59],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 390],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 34],
    intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
    intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
    nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
    nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
    nonterminal_nodes_index_symbol: FxHashMap<Span, InlineVec<(i32, SPPFNodeId)>>,
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
            gss_nodes_index: [const { vec![] }; 59],
            gss_nodes_index_symbol: vec![],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 59],
            intermediate_nodes_index: [const { InlineMap::Empty }; 390],
            terminal_nodes_index: [const { InlineMap::Empty }; 34],
            stats: Stats::default(),
            intermediate_nodes_children: vec![],
            intermediate_nodes_children_map: OnceCell::new(),
            nonterminal_nodes_children: vec![],
            nonterminal_nodes_children_map: OnceCell::new(),
            nonterminal_nodes_index_symbol: FxHashMap::default(),
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
    fn create_except(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(5), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(6), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_associativity(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(7), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(8), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(9), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(10), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(11), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(12), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(13), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_plus_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(14), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(15), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_star_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(16), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(17), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_plus_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(18), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_opt_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(19), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_layout_def_star_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(20), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_plus_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(21), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_opt_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(22), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_syntax_rule_star_2(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(23), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_plus_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(24), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_opt_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(25), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_star_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(26), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_plus_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(27), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_plus_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(28), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_opt_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(29), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_opt_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(30), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_plus_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(31), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_opt_8(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(32), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_star_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(33), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_plus_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(34), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_opt_9(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(35), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_star_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(36), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_opt_10(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(37), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_group_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(38), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_plus_8(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(39), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_group_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(40), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_plus_9(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(41), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_opt_11(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(42), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_plus_10(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(43), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_grammar(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(44), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_layout_def(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(45), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_syntax_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(46), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex_block(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(47), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(48), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_except(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(49), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(50), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_associativity(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(51), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(52), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_symbol(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(53), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(54), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(55), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(56), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(57), sppf_node_id, gss_node_id, return_slot);
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
            record!(self, GSSNodeFound, NonterminalId(58), i);
            self.add_edge_to_existing_gss_node(
                existing_gss_node_id,
                gss_node_id,
                left_child,
                return_slot,
                env,
                binding,
            );
        } else {
            record!(self, GSSNodeNotFound, NonterminalId(58), i);
            let new_gss_node_id = self.new_gss_node(NonterminalId(58), i);
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
            self.add_first_descriptors(NonterminalId(58), i, new_gss_node_id, Some(env_id));
            self.add_gss_node_symbol(i, p, new_gss_node_id);
        }
    }
    fn get_gss_node_symbol(&self, input_index: u32, p: i32) -> Option<GssNodeId> {
        self.gss_nodes_index_symbol
            .iter()
            .find(|(i, a0, _)| *i == input_index && *a0 == p)
            .map(|x| x.2)
    }
    fn add_gss_node_symbol(&mut self, input_index: u32, p: i32, gss_node_id: GssNodeId) {
        self.gss_nodes_index_symbol
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
}

