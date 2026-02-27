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
pub const NONTERMINALS: [Nonterminal; 64] = [
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
        name: "Annotation",
        display: "Annotation",
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
        name: "SyntaxRule_Opt_4",
        display: "Annotation?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "SyntaxRule_Plus_2",
        display: "{PriorityLevel \">\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "SyntaxRule_Opt_5",
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
        name: "RegexBlock_Opt_6",
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
        name: "RegexRule_Plus_6",
        display: "PostCondition+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "RegexRule_Opt_7",
        display: "PostCondition+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "RegexRule_Star_4",
        display: "PostCondition*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_8",
        display: "Associativity?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "PriorityLevel_Plus_7",
        display: "{Alternative \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_9",
        display: "{Alternative \"|\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "PriorityLevel_Star_5",
        display: "{Alternative \"|\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Plus_8",
        display: "Symbol+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Alternative_Opt_10",
        display: "Symbol+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Alternative_Star_6",
        display: "Symbol*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Opt_11",
        display: "Label?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Symbol_Group_0",
        display: "(\"|\" Symbol)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Symbol_Plus_9",
        display: "(\"|\" Symbol)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Regex_Group_1",
        display: "(\"|\" Regex)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Regex_Plus_10",
        display: "(\"|\" Regex)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "CharClass_Opt_12",
        display: "\"!\"?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "CharClass_Plus_11",
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
        name: "StartAnnotation",
        display: "StartAnnotation",
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
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Grammar" => NonterminalId(0), "LayoutDef" => NonterminalId(1), "SyntaxRule" =>
    NonterminalId(2), "Annotation" => NonterminalId(3), "RegexBlock" => NonterminalId(4),
    "RegexRule" => NonterminalId(5), "PostCondition" => NonterminalId(6), "PriorityLevel"
    => NonterminalId(7), "Associativity" => NonterminalId(8), "Alternative" =>
    NonterminalId(9), "Regex" => NonterminalId(10), "CharClass" => NonterminalId(11),
    "RangeElement" => NonterminalId(12), "Range" => NonterminalId(13), "Grammar_Opt_0" =>
    NonterminalId(14), "Grammar_Plus_0" => NonterminalId(15), "Grammar_Opt_1" =>
    NonterminalId(16), "Grammar_Star_0" => NonterminalId(17), "Grammar_Opt_2" =>
    NonterminalId(18), "LayoutDef_Plus_1" => NonterminalId(19), "LayoutDef_Opt_3" =>
    NonterminalId(20), "LayoutDef_Star_1" => NonterminalId(21), "SyntaxRule_Opt_4" =>
    NonterminalId(22), "SyntaxRule_Plus_2" => NonterminalId(23), "SyntaxRule_Opt_5" =>
    NonterminalId(24), "SyntaxRule_Star_2" => NonterminalId(25), "RegexBlock_Plus_3" =>
    NonterminalId(26), "RegexBlock_Opt_6" => NonterminalId(27), "RegexBlock_Star_3" =>
    NonterminalId(28), "RegexRule_Plus_5" => NonterminalId(29), "RegexRule_Plus_4" =>
    NonterminalId(30), "RegexRule_Plus_6" => NonterminalId(31), "RegexRule_Opt_7" =>
    NonterminalId(32), "RegexRule_Star_4" => NonterminalId(33), "PriorityLevel_Opt_8" =>
    NonterminalId(34), "PriorityLevel_Plus_7" => NonterminalId(35), "PriorityLevel_Opt_9"
    => NonterminalId(36), "PriorityLevel_Star_5" => NonterminalId(37),
    "Alternative_Plus_8" => NonterminalId(38), "Alternative_Opt_10" => NonterminalId(39),
    "Alternative_Star_6" => NonterminalId(40), "Alternative_Opt_11" => NonterminalId(41),
    "Symbol_Group_0" => NonterminalId(42), "Symbol_Plus_9" => NonterminalId(43),
    "Regex_Group_1" => NonterminalId(44), "Regex_Plus_10" => NonterminalId(45),
    "CharClass_Opt_12" => NonterminalId(46), "CharClass_Plus_11" => NonterminalId(47),
    "StartGrammar" => NonterminalId(48), "StartLayoutDef" => NonterminalId(49),
    "StartSyntaxRule" => NonterminalId(50), "StartAnnotation" => NonterminalId(51),
    "StartRegexBlock" => NonterminalId(52), "StartRegexRule" => NonterminalId(53),
    "StartPostCondition" => NonterminalId(54), "StartPriorityLevel" => NonterminalId(55),
    "StartAssociativity" => NonterminalId(56), "StartAlternative" => NonterminalId(57),
    "StartSymbol" => NonterminalId(58), "StartRegex" => NonterminalId(59),
    "StartCharClass" => NonterminalId(60), "StartRangeElement" => NonterminalId(61),
    "StartRange" => NonterminalId(62), "Symbol" => NonterminalId(63)
};
pub const TERMINALS: [Terminal; 36] = [
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
    Terminal { name: "\"regex\"" },
    Terminal { name: "\"{\"" },
    Terminal { name: "\"}\"" },
    Terminal { name: "\"|\"" },
    Terminal { name: "\"\\\"" },
    Terminal { name: "\"!>>\"" },
    Terminal { name: "\"left\"" },
    Terminal { name: "\"right\"" },
    Terminal { name: "\"none\"" },
    Terminal { name: "\"\"\"" },
    Terminal { name: "\"*\"" },
    Terminal { name: "\"+\"" },
    Terminal { name: "\"?\"" },
    Terminal { name: "\":\"" },
    Terminal { name: "\"'\"" },
    Terminal { name: "\"!\"" },
    Terminal { name: "\"[\"" },
    Terminal { name: "\"]\"" },
    Terminal { name: "\"-\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 421] = [
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
        display_name: "RegexRule : . Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : Identifier . Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout . \"=\" Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" . Layout body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout . body:{Regex+ \"|\"}+ Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ . Layout PostCondition*",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout . PostCondition*",
    },
    Slot {
        display_name: "RegexRule : Identifier Layout \"=\" Layout body:{Regex+ \"|\"}+ Layout PostCondition*.",
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
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
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
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
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
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
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
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
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
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
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
            //SyntaxRule : . SyntaxRule_Opt_4 Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(14) => {
                self.create_syntax_rule_opt_4(result, gss_node_id, SlotId(15));
            }
            //SyntaxRule : SyntaxRule_Opt_4 . Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(15) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //SyntaxRule : SyntaxRule_Opt_4 Layout . head:Identifier Layout "=" Layout SyntaxRule_Star_2
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
            //SyntaxRule : SyntaxRule_Opt_4 Layout . head:Identifier Layout "=" Layout SyntaxRule_Star_2
            SlotId(16) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier . Layout "=" Layout SyntaxRule_Star_2
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
                            "Identifier",
                            i,
                            SlotId(16),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier . Layout "=" Layout SyntaxRule_Star_2
            SlotId(17) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier Layout . "=" Layout SyntaxRule_Star_2
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
            //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier Layout . "=" Layout SyntaxRule_Star_2
            SlotId(18) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier Layout "=" . Layout SyntaxRule_Star_2
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
                            "\"=\"",
                            i,
                            SlotId(18),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier Layout "=" . Layout SyntaxRule_Star_2
            SlotId(19) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier Layout "=" Layout . SyntaxRule_Star_2
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
            //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier Layout "=" Layout . SyntaxRule_Star_2
            SlotId(20) => {
                self.create_syntax_rule_star_2(result, gss_node_id, SlotId(21));
            }
            //SyntaxRule : SyntaxRule_Opt_4 Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Annotation : . "@NoLayout"
            SlotId(22) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"@NoLayout\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"@NoLayout\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Annotation : "@NoLayout".
                        let next_slot_id = SlotId(23);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"@NoLayout\"",
                            i,
                            SlotId(22),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@NoLayout".
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
            //Annotation : . "@Layout" Layout "(" Layout Identifier Layout ")"
            SlotId(24) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"@Layout\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"@Layout\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Annotation : "@Layout" . Layout "(" Layout Identifier Layout ")"
                        let next_slot_id = SlotId(25);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"@Layout\"",
                            i,
                            SlotId(24),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" . Layout "(" Layout Identifier Layout ")"
            SlotId(25) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Annotation : "@Layout" Layout . "(" Layout Identifier Layout ")"
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
            //Annotation : "@Layout" Layout . "(" Layout Identifier Layout ")"
            SlotId(26) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Annotation : "@Layout" Layout "(" . Layout Identifier Layout ")"
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
                            "\"(\"",
                            i,
                            SlotId(26),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" . Layout Identifier Layout ")"
            SlotId(27) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Annotation : "@Layout" Layout "(" Layout . Identifier Layout ")"
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
            //Annotation : "@Layout" Layout "(" Layout . Identifier Layout ")"
            SlotId(28) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Annotation : "@Layout" Layout "(" Layout Identifier . Layout ")"
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
                            "Identifier",
                            i,
                            SlotId(28),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" Layout Identifier . Layout ")"
            SlotId(29) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Annotation : "@Layout" Layout "(" Layout Identifier Layout . ")"
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
            //Annotation : "@Layout" Layout "(" Layout Identifier Layout . ")"
            SlotId(30) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Annotation : "@Layout" Layout "(" Layout Identifier Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(30),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Annotation : "@Layout" Layout "(" Layout Identifier Layout ")".
            SlotId(31) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let end_slot_id = SlotId(31);
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
            SlotId(32) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"regex\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"regex\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //RegexBlock : "regex" . Layout "{" Layout RegexBlock_Star_3 Layout "}"
                        let next_slot_id = SlotId(33);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"regex\"",
                            i,
                            SlotId(32),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" . Layout "{" Layout RegexBlock_Star_3 Layout "}"
            SlotId(33) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexBlock : "regex" Layout . "{" Layout RegexBlock_Star_3 Layout "}"
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
            //RegexBlock : "regex" Layout . "{" Layout RegexBlock_Star_3 Layout "}"
            SlotId(34) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //RegexBlock : "regex" Layout "{" . Layout RegexBlock_Star_3 Layout "}"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(34),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" Layout "{" . Layout RegexBlock_Star_3 Layout "}"
            SlotId(35) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexBlock : "regex" Layout "{" Layout . RegexBlock_Star_3 Layout "}"
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
            //RegexBlock : "regex" Layout "{" Layout . RegexBlock_Star_3 Layout "}"
            SlotId(36) => {
                self.create_regex_block_star_3(result, gss_node_id, SlotId(37));
            }
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 . Layout "}"
            SlotId(37) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout . "}"
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
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout . "}"
            SlotId(38) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}".
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
                            "\"}\"",
                            i,
                            SlotId(38),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexBlock : "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}".
            SlotId(39) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //RegexRule : . Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
            SlotId(40) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //RegexRule : Identifier . Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
                        let next_slot_id = SlotId(41);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(40),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier . Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
            SlotId(41) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexRule : Identifier Layout . "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
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
            //RegexRule : Identifier Layout . "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
            SlotId(42) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //RegexRule : Identifier Layout "=" . Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"=\"",
                            i,
                            SlotId(42),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule : Identifier Layout "=" . Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
            SlotId(43) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexRule : Identifier Layout "=" Layout . body:RegexRule_Plus_4 Layout RegexRule_Star_4
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
            //RegexRule : Identifier Layout "=" Layout . body:RegexRule_Plus_4 Layout RegexRule_Star_4
            SlotId(44) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(45));
            }
            //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 . Layout RegexRule_Star_4
            SlotId(45) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout . RegexRule_Star_4
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout . RegexRule_Star_4
            SlotId(46) => {
                self.create_regex_rule_star_4(result, gss_node_id, SlotId(47));
            }
            //RegexRule : Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4.
            SlotId(47) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //PostCondition : . "\" Layout Identifier
            SlotId(48) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\\\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\\\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //PostCondition : "\" . Layout Identifier
                        let next_slot_id = SlotId(49);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\\\"",
                            i,
                            SlotId(48),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "\" . Layout Identifier
            SlotId(49) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //PostCondition : "\" Layout . Identifier
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //PostCondition : "\" Layout . Identifier
            SlotId(50) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //PostCondition : "\" Layout Identifier.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(50),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "\" Layout Identifier.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //PostCondition : . "!>>" Layout Identifier
            SlotId(52) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!>>\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!>>\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //PostCondition : "!>>" . Layout Identifier
                        let next_slot_id = SlotId(53);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!>>\"",
                            i,
                            SlotId(52),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PostCondition : "!>>" . Layout Identifier
            SlotId(53) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //PostCondition : "!>>" Layout . Identifier
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //PostCondition : "!>>" Layout . Identifier
            SlotId(54) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //PostCondition : "!>>" Layout Identifier.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
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
            //PostCondition : "!>>" Layout Identifier.
            SlotId(55) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(55);
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
            //PriorityLevel : . PriorityLevel_Opt_8 Layout PriorityLevel_Star_5
            SlotId(56) => {
                self.create_priority_level_opt_8(result, gss_node_id, SlotId(57));
            }
            //PriorityLevel : PriorityLevel_Opt_8 . Layout PriorityLevel_Star_5
            SlotId(57) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //PriorityLevel : PriorityLevel_Opt_8 Layout . PriorityLevel_Star_5
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //PriorityLevel : PriorityLevel_Opt_8 Layout . PriorityLevel_Star_5
            SlotId(58) => {
                self.create_priority_level_star_5(result, gss_node_id, SlotId(59));
            }
            //PriorityLevel : PriorityLevel_Opt_8 Layout PriorityLevel_Star_5.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Associativity : . "left"
            SlotId(60) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"left\"", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"left\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Associativity : "left".
                        let next_slot_id = SlotId(61);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"left\"",
                            i,
                            SlotId(60),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "left".
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
            //Associativity : . "right"
            SlotId(62) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"right\"", i);
                match self.scanner.match_token(TerminalId(22), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"right\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(22), i, j);
                        //Associativity : "right".
                        let next_slot_id = SlotId(63);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"right\"",
                            i,
                            SlotId(62),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "right".
            SlotId(63) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
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
            //Associativity : . "none"
            SlotId(64) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"none\"", i);
                match self.scanner.match_token(TerminalId(23), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"none\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(23), i, j);
                        //Associativity : "none".
                        let next_slot_id = SlotId(65);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"none\"",
                            i,
                            SlotId(64),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Associativity : "none".
            SlotId(65) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
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
            //Alternative : . Alternative_Star_6 Layout Alternative_Opt_11
            SlotId(66) => {
                self.create_alternative_star_6(result, gss_node_id, SlotId(67));
            }
            //Alternative : Alternative_Star_6 . Layout Alternative_Opt_11
            SlotId(67) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Alternative : Alternative_Star_6 Layout . Alternative_Opt_11
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
            //Alternative : Alternative_Star_6 Layout . Alternative_Opt_11
            SlotId(68) => {
                self.create_alternative_opt_11(result, gss_node_id, SlotId(69));
            }
            //Alternative : Alternative_Star_6 Layout Alternative_Opt_11.
            SlotId(69) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(69);
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
            SlotId(70) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : Identifier . return 0
                        let next_slot_id = SlotId(71);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(70),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : Identifier . return 0
            SlotId(71) => {
                self.execute(input_index, SlotId(72), result, gss_node_id, env);
            }
            //Symbol(p: i32) : Identifier return 0.
            SlotId(72) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
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
            //Symbol(p: i32) : . "(" Layout Alternative_Plus_8 Layout ")" return 0
            SlotId(73) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol(p: i32) : "(" . Layout Alternative_Plus_8 Layout ")" return 0
                        let next_slot_id = SlotId(74);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(73),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" . Layout Alternative_Plus_8 Layout ")" return 0
            SlotId(74) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "(" Layout . Alternative_Plus_8 Layout ")" return 0
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
            //Symbol(p: i32) : "(" Layout . Alternative_Plus_8 Layout ")" return 0
            SlotId(75) => {
                self.create_alternative_plus_8(result, gss_node_id, SlotId(76));
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_8 . Layout ")" return 0
            SlotId(76) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "(" Layout Alternative_Plus_8 Layout . ")" return 0
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
            //Symbol(p: i32) : "(" Layout Alternative_Plus_8 Layout . ")" return 0
            SlotId(77) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol(p: i32) : "(" Layout Alternative_Plus_8 Layout ")" . return 0
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
                            "\")\"",
                            i,
                            SlotId(77),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_8 Layout ")" . return 0
            SlotId(78) => {
                self.execute(input_index, SlotId(79), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "(" Layout Alternative_Plus_8 Layout ")" return 0.
            SlotId(79) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
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
            //Symbol(p: i32) : . "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" return 0
            SlotId(80) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" return 0
                        let next_slot_id = SlotId(81);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(80),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" . Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" return 0
            SlotId(81) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" return 0
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
            //Symbol(p: i32) : "(" Layout . first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" return 0
            SlotId(82) => {
                self.create_symbol(result, gss_node_id, SlotId(83), env, None, 0);
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) . Layout rest:Symbol_Plus_9 Layout ")" return 0
            SlotId(83) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_9 Layout ")" return 0
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
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout . rest:Symbol_Plus_9 Layout ")" return 0
            SlotId(84) => {
                self.create_symbol_plus_9(result, gss_node_id, SlotId(85));
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 . Layout ")" return 0
            SlotId(85) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout . ")" return 0
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
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout . ")" return 0
            SlotId(86) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" . return 0
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
                            "\")\"",
                            i,
                            SlotId(86),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" . return 0
            SlotId(87) => {
                self.execute(input_index, SlotId(88), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" return 0.
            SlotId(88) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(88);
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
            SlotId(89) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Symbol(p: i32) : """ . Layout String Layout """ return 0
                        let next_slot_id = SlotId(90);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(89),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ . Layout String Layout """ return 0
            SlotId(90) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : """ Layout . String Layout """ return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(90),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout . String Layout """ return 0
            SlotId(91) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Symbol(p: i32) : """ Layout String . Layout """ return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "String",
                            i,
                            SlotId(91),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout String . Layout """ return 0
            SlotId(92) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : """ Layout String Layout . """ return 0
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
            //Symbol(p: i32) : """ Layout String Layout . """ return 0
            SlotId(93) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Symbol(p: i32) : """ Layout String Layout """ . return 0
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
                            "\"\"\"",
                            i,
                            SlotId(93),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : """ Layout String Layout """ . return 0
            SlotId(94) => {
                self.execute(input_index, SlotId(95), result, gss_node_id, env);
            }
            //Symbol(p: i32) : """ Layout String Layout """ return 0.
            SlotId(95) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(95);
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
            SlotId(96) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                        let next_slot_id = SlotId(97);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(96),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(97) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
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
                            "Layout",
                            i,
                            SlotId(97),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(98) => {
                self.create_symbol(result, gss_node_id, SlotId(99), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(99) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "*" return 0
            SlotId(100) => {
                self.create_symbol(result, gss_node_id, SlotId(101), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "*" return 0
            SlotId(101) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "*" return 0
            SlotId(102) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(102),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "*" return 0
            SlotId(103) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "*" return 0
            SlotId(104) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
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
                            "\"*\"",
                            i,
                            SlotId(104),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" . return 0
            SlotId(105) => {
                self.execute(input_index, SlotId(106), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0.
            SlotId(106) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(106);
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
            SlotId(107) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(108);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(107),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" . Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(108) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                        let next_slot_id = SlotId(109);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(108),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout . symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(109) => {
                self.create_symbol(result, gss_node_id, SlotId(110), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) . Layout sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(110) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout . sep:Symbol(0) Layout "}" Layout "+" return 0
            SlotId(111) => {
                self.create_symbol(result, gss_node_id, SlotId(112), env, None, 0);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) . Layout "}" Layout "+" return 0
            SlotId(112) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout . "}" Layout "+" return 0
            SlotId(113) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
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
                            "\"}\"",
                            i,
                            SlotId(113),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" . Layout "+" return 0
            SlotId(114) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
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
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout . "+" return 0
            SlotId(115) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
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
                            "\"+\"",
                            i,
                            SlotId(115),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" . return 0
            SlotId(116) => {
                self.execute(input_index, SlotId(117), result, gss_node_id, env);
            }
            //Symbol(p: i32) : "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0.
            SlotId(117) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(117);
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
            SlotId(118) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(119), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "*" return 0
            SlotId(119) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(120),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "*" return 0
            SlotId(120) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(121), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "*" return 0
            SlotId(121) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "*" return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "*" return 0
            SlotId(122) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" . return 0
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(122),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" . return 0
            SlotId(123) => {
                self.execute(input_index, SlotId(124), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" return 0.
            SlotId(124) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(124);
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
            SlotId(125) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(126), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "+" return 0
            SlotId(126) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(127),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "+" return 0
            SlotId(127) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(128), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "+" return 0
            SlotId(128) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "+" return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "+" return 0
            SlotId(129) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" . return 0
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
                            "\"+\"",
                            i,
                            SlotId(129),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" . return 0
            SlotId(130) => {
                self.execute(input_index, SlotId(131), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" return 0.
            SlotId(131) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
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
            //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0
            SlotId(132) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(133), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "?" return 0
            SlotId(134) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(135), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "?" return 0
            SlotId(135) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "?" return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "?" return 0
            SlotId(136) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" . return 0
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
                            "\"?\"",
                            i,
                            SlotId(136),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" . return 0
            SlotId(137) => {
                self.execute(input_index, SlotId(138), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0.
            SlotId(138) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(138);
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
            SlotId(139) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(140), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier return 0
            SlotId(140) => {
                self.create_symbol(
                    result,
                    gss_node_id,
                    SlotId(141),
                    env,
                    Some("l"),
                    self.lookup("p", env.unwrap()),
                );
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "\" Layout Identifier return 0
            SlotId(141) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(142), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "\" Layout Identifier return 0
            SlotId(142) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "\" Layout Identifier return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "\" Layout Identifier return 0
            SlotId(143) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\\\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\\\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" . Layout Identifier return 0
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
                            "\"\\\"",
                            i,
                            SlotId(143),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" . Layout Identifier return 0
            SlotId(144) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout . Identifier return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout . Identifier return 0
            SlotId(145) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier . return 0
                        let next_slot_id = SlotId(146);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(145),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier . return 0
            SlotId(146) => {
                self.execute(input_index, SlotId(147), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier return 0.
            SlotId(147) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
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
            //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
            SlotId(148) => {
                if 2 >= self.lookup("p", env.unwrap()) {
                    self.execute(input_index, SlotId(149), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] . l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) . [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
            SlotId(150) => {
                if (self.lookup("l", env.unwrap()) == 0) || (self.lookup("l", env.unwrap()) >= 2) {
                    self.execute(input_index, SlotId(151), result, gss_node_id, env);
                }
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] . Layout "!>>" Layout Identifier return 0
            SlotId(151) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "!>>" Layout Identifier return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout . "!>>" Layout Identifier return 0
            SlotId(152) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!>>\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!>>\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" . Layout Identifier return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" . Layout Identifier return 0
            SlotId(153) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout . Identifier return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout . Identifier return 0
            SlotId(154) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier . return 0
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
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier . return 0
            SlotId(155) => {
                self.execute(input_index, SlotId(156), result, gss_node_id, env);
            }
            //Symbol(p: i32) : [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0.
            SlotId(156) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
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
            //Symbol(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
            SlotId(157) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
                        let next_slot_id = SlotId(158);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(157),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier . Layout ":" Layout Symbol(1) return 1
            SlotId(158) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
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
                            "Layout",
                            i,
                            SlotId(158),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier Layout . ":" Layout Symbol(1) return 1
            SlotId(159) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\":\"", i);
                match self.scanner.match_token(TerminalId(28), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\":\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(28), i, j);
                        //Symbol(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
                        let next_slot_id = SlotId(160);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(159),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol(p: i32) : label:Identifier Layout ":" . Layout Symbol(1) return 1
            SlotId(160) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
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
            //Symbol(p: i32) : label:Identifier Layout ":" Layout . Symbol(1) return 1
            SlotId(161) => {
                self.create_symbol(result, gss_node_id, SlotId(162), env, None, 1);
            }
            //Symbol(p: i32) : label:Identifier Layout ":" Layout Symbol(1) . return 1
            SlotId(162) => {
                self.execute(input_index, SlotId(163), result, gss_node_id, env);
            }
            //Symbol(p: i32) : label:Identifier Layout ":" Layout Symbol(1) return 1.
            SlotId(163) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(63);
                let end_slot_id = SlotId(163);
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
            SlotId(164) => {
                self.create_regex(result, gss_node_id, SlotId(165));
            }
            //Regex : Regex . Layout "+"
            SlotId(165) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : Regex Layout . "+"
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
            //Regex : Regex Layout . "+"
            SlotId(166) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(26), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(26), i, j);
                        //Regex : Regex Layout "+".
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
                            "\"+\"",
                            i,
                            SlotId(166),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "+".
            SlotId(167) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
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
            //Regex : . Regex Layout "*"
            SlotId(168) => {
                self.create_regex(result, gss_node_id, SlotId(169));
            }
            //Regex : Regex . Layout "*"
            SlotId(169) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : Regex Layout . "*"
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
            //Regex : Regex Layout . "*"
            SlotId(170) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(25), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(25), i, j);
                        //Regex : Regex Layout "*".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(170),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "*".
            SlotId(171) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(171);
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
            SlotId(172) => {
                self.create_regex(result, gss_node_id, SlotId(173));
            }
            //Regex : Regex . Layout "?"
            SlotId(173) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : Regex Layout . "?"
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
                            "Layout",
                            i,
                            SlotId(173),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout . "?"
            SlotId(174) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(27), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(27), i, j);
                        //Regex : Regex Layout "?".
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
                            "\"?\"",
                            i,
                            SlotId(174),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "?".
            SlotId(175) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
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
            //Regex : . "(" Layout first:Regex Layout rest:Regex_Plus_10 Layout ")"
            SlotId(176) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Regex : "(" . Layout first:Regex Layout rest:Regex_Plus_10 Layout ")"
                        let next_slot_id = SlotId(177);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(176),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout first:Regex Layout rest:Regex_Plus_10 Layout ")"
            SlotId(177) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : "(" Layout . first:Regex Layout rest:Regex_Plus_10 Layout ")"
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
            //Regex : "(" Layout . first:Regex Layout rest:Regex_Plus_10 Layout ")"
            SlotId(178) => {
                self.create_regex(result, gss_node_id, SlotId(179));
            }
            //Regex : "(" Layout first:Regex . Layout rest:Regex_Plus_10 Layout ")"
            SlotId(179) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : "(" Layout first:Regex Layout . rest:Regex_Plus_10 Layout ")"
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
            //Regex : "(" Layout first:Regex Layout . rest:Regex_Plus_10 Layout ")"
            SlotId(180) => {
                self.create_regex_plus_10(result, gss_node_id, SlotId(181));
            }
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_10 . Layout ")"
            SlotId(181) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_10 Layout . ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_10 Layout . ")"
            SlotId(182) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_10 Layout ")".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(182),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout first:Regex Layout rest:Regex_Plus_10 Layout ")".
            SlotId(183) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(183);
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
            SlotId(184) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Regex : "(" . Layout RegexRule_Plus_5 Layout ")"
                        let next_slot_id = SlotId(185);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(184),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout RegexRule_Plus_5 Layout ")"
            SlotId(185) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : "(" Layout . RegexRule_Plus_5 Layout ")"
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
            //Regex : "(" Layout . RegexRule_Plus_5 Layout ")"
            SlotId(186) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(187));
            }
            //Regex : "(" Layout RegexRule_Plus_5 . Layout ")"
            SlotId(187) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : "(" Layout RegexRule_Plus_5 Layout . ")"
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
            //Regex : "(" Layout RegexRule_Plus_5 Layout . ")"
            SlotId(188) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Regex : "(" Layout RegexRule_Plus_5 Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(188),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout RegexRule_Plus_5 Layout ")".
            SlotId(189) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
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
            //Regex : . CharClass
            SlotId(190) => {
                self.create_char_class(result, gss_node_id, SlotId(191));
            }
            //Regex : CharClass.
            SlotId(191) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(191);
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
            SlotId(192) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"'\"", i);
                match self.scanner.match_token(TerminalId(29), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"'\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(29), i, j);
                        //Regex : "'" . Layout Char Layout "'"
                        let next_slot_id = SlotId(193);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"'\"",
                            i,
                            SlotId(192),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" . Layout Char Layout "'"
            SlotId(193) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : "'" Layout . Char Layout "'"
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
            //Regex : "'" Layout . Char Layout "'"
            SlotId(194) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Regex : "'" Layout Char . Layout "'"
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
                            "Char",
                            i,
                            SlotId(194),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout Char . Layout "'"
            SlotId(195) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : "'" Layout Char Layout . "'"
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
            //Regex : "'" Layout Char Layout . "'"
            SlotId(196) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"'\"", i);
                match self.scanner.match_token(TerminalId(29), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"'\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(29), i, j);
                        //Regex : "'" Layout Char Layout "'".
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
                            "\"'\"",
                            i,
                            SlotId(196),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "'" Layout Char Layout "'".
            SlotId(197) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
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
            //Regex : . """ Layout String Layout """
            SlotId(198) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Regex : """ . Layout String Layout """
                        let next_slot_id = SlotId(199);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(198),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ . Layout String Layout """
            SlotId(199) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : """ Layout . String Layout """
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
            //Regex : """ Layout . String Layout """
            SlotId(200) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Regex : """ Layout String . Layout """
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "String",
                            i,
                            SlotId(200),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String . Layout """
            SlotId(201) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex : """ Layout String Layout . """
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
            //Regex : """ Layout String Layout . """
            SlotId(202) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(24), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(24), i, j);
                        //Regex : """ Layout String Layout """.
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
                            "\"\"\"",
                            i,
                            SlotId(202),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : """ Layout String Layout """.
            SlotId(203) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
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
            //CharClass : . neg:CharClass_Opt_12 Layout "[" Layout CharClass_Plus_11 Layout "]"
            SlotId(204) => {
                self.create_char_class_opt_12(result, gss_node_id, SlotId(205));
            }
            //CharClass : neg:CharClass_Opt_12 . Layout "[" Layout CharClass_Plus_11 Layout "]"
            SlotId(205) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //CharClass : neg:CharClass_Opt_12 Layout . "[" Layout CharClass_Plus_11 Layout "]"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //CharClass : neg:CharClass_Opt_12 Layout . "[" Layout CharClass_Plus_11 Layout "]"
            SlotId(206) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"[\"", i);
                match self.scanner.match_token(TerminalId(31), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"[\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(31), i, j);
                        //CharClass : neg:CharClass_Opt_12 Layout "[" . Layout CharClass_Plus_11 Layout "]"
                        let next_slot_id = SlotId(207);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(206),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_12 Layout "[" . Layout CharClass_Plus_11 Layout "]"
            SlotId(207) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //CharClass : neg:CharClass_Opt_12 Layout "[" Layout . CharClass_Plus_11 Layout "]"
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
            //CharClass : neg:CharClass_Opt_12 Layout "[" Layout . CharClass_Plus_11 Layout "]"
            SlotId(208) => {
                self.create_char_class_plus_11(result, gss_node_id, SlotId(209));
            }
            //CharClass : neg:CharClass_Opt_12 Layout "[" Layout CharClass_Plus_11 . Layout "]"
            SlotId(209) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //CharClass : neg:CharClass_Opt_12 Layout "[" Layout CharClass_Plus_11 Layout . "]"
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
            //CharClass : neg:CharClass_Opt_12 Layout "[" Layout CharClass_Plus_11 Layout . "]"
            SlotId(210) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"]\"", i);
                match self.scanner.match_token(TerminalId(32), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"]\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(32), i, j);
                        //CharClass : neg:CharClass_Opt_12 Layout "[" Layout CharClass_Plus_11 Layout "]".
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
                            "\"]\"",
                            i,
                            SlotId(210),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : neg:CharClass_Opt_12 Layout "[" Layout CharClass_Plus_11 Layout "]".
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
            //RangeElement : . Range
            SlotId(212) => {
                self.create_range(result, gss_node_id, SlotId(213));
            }
            //RangeElement : Range.
            SlotId(213) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
                let end_slot_id = SlotId(213);
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
            SlotId(214) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //RangeElement : RangeChar.
                        let next_slot_id = SlotId(215);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(214),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RangeElement : RangeChar.
            SlotId(215) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
                let end_slot_id = SlotId(215);
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
            SlotId(216) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Range : start:RangeChar . Layout "-" Layout end:RangeChar
                        let next_slot_id = SlotId(217);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(216),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar . Layout "-" Layout end:RangeChar
            SlotId(217) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Range : start:RangeChar Layout . "-" Layout end:RangeChar
                        let next_slot_id = SlotId(218);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(217),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout . "-" Layout end:RangeChar
            SlotId(218) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"-\"", i);
                match self.scanner.match_token(TerminalId(33), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"-\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(33), i, j);
                        //Range : start:RangeChar Layout "-" . Layout end:RangeChar
                        let next_slot_id = SlotId(219);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(218),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" . Layout end:RangeChar
            SlotId(219) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Range : start:RangeChar Layout "-" Layout . end:RangeChar
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Range : start:RangeChar Layout "-" Layout . end:RangeChar
            SlotId(220) => {
                let i = input_index;
                record!(self, MatchingTerminal, "RangeChar", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "RangeChar", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Range : start:RangeChar Layout "-" Layout end:RangeChar.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "RangeChar",
                            i,
                            SlotId(220),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Range : start:RangeChar Layout "-" Layout end:RangeChar.
            SlotId(221) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
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
            //Grammar_Opt_0 : . LayoutDef
            SlotId(222) => {
                self.create_layout_def(result, gss_node_id, SlotId(223));
            }
            //Grammar_Opt_0 : LayoutDef.
            SlotId(223) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(223);
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
            SlotId(224) => {
                let end_slot_id = SlotId(224);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
                let nonterminal_id = NonterminalId(14);
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
            SlotId(225) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(226));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 . Layout SyntaxRule
            SlotId(226) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
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
                            "Layout",
                            i,
                            SlotId(226),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout . SyntaxRule
            SlotId(227) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(228));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout SyntaxRule.
            SlotId(228) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
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
            //Grammar_Plus_0 : . SyntaxRule
            SlotId(229) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(230));
            }
            //Grammar_Plus_0 : SyntaxRule.
            SlotId(230) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
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
            //Grammar_Opt_1 : . Grammar_Plus_0
            SlotId(231) => {
                self.create_grammar_plus_0(result, gss_node_id, SlotId(232));
            }
            //Grammar_Opt_1 : Grammar_Plus_0.
            SlotId(232) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
                let end_slot_id = SlotId(232);
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
            SlotId(233) => {
                let end_slot_id = SlotId(233);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
                let nonterminal_id = NonterminalId(16);
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
            SlotId(234) => {
                self.create_grammar_opt_1(result, gss_node_id, SlotId(235));
            }
            //Grammar_Star_0 : Grammar_Opt_1.
            SlotId(235) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(235);
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
            SlotId(236) => {
                self.create_regex_block(result, gss_node_id, SlotId(237));
            }
            //Grammar_Opt_2 : RegexBlock.
            SlotId(237) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
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
            //Grammar_Opt_2 : .
            SlotId(238) => {
                let end_slot_id = SlotId(238);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
                let nonterminal_id = NonterminalId(18);
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
            SlotId(239) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(240));
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 . Layout Identifier
            SlotId(240) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
                        let next_slot_id = SlotId(241);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(240),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout . Identifier
            SlotId(241) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
                        let next_slot_id = SlotId(242);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(241),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : LayoutDef_Plus_1 Layout Identifier.
            SlotId(242) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(242);
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
            SlotId(243) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //LayoutDef_Plus_1 : Identifier.
                        let next_slot_id = SlotId(244);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(243),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //LayoutDef_Plus_1 : Identifier.
            SlotId(244) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
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
            //LayoutDef_Opt_3 : . LayoutDef_Plus_1
            SlotId(245) => {
                self.create_layout_def_plus_1(result, gss_node_id, SlotId(246));
            }
            //LayoutDef_Opt_3 : LayoutDef_Plus_1.
            SlotId(246) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
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
            //LayoutDef_Opt_3 : .
            SlotId(247) => {
                let end_slot_id = SlotId(247);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
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
            //LayoutDef_Star_1 : . LayoutDef_Opt_3
            SlotId(248) => {
                self.create_layout_def_opt_3(result, gss_node_id, SlotId(249));
            }
            //LayoutDef_Star_1 : LayoutDef_Opt_3.
            SlotId(249) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
                let end_slot_id = SlotId(249);
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
            //SyntaxRule_Opt_4 : . Annotation
            SlotId(250) => {
                self.create_annotation(result, gss_node_id, SlotId(251));
            }
            //SyntaxRule_Opt_4 : Annotation.
            SlotId(251) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
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
            //SyntaxRule_Opt_4 : .
            SlotId(252) => {
                let end_slot_id = SlotId(252);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
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
            SlotId(253) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(254));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 . Layout ">" Layout PriorityLevel
            SlotId(254) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
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
                            "Layout",
                            i,
                            SlotId(254),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout . ">" Layout PriorityLevel
            SlotId(255) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
                        let next_slot_id = SlotId(256);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(255),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" . Layout PriorityLevel
            SlotId(256) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout . PriorityLevel
            SlotId(257) => {
                self.create_priority_level(result, gss_node_id, SlotId(258));
            }
            //SyntaxRule_Plus_2 : SyntaxRule_Plus_2 Layout ">" Layout PriorityLevel.
            SlotId(258) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(258);
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
            SlotId(259) => {
                self.create_priority_level(result, gss_node_id, SlotId(260));
            }
            //SyntaxRule_Plus_2 : PriorityLevel.
            SlotId(260) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(260);
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
            //SyntaxRule_Opt_5 : . SyntaxRule_Plus_2
            SlotId(261) => {
                self.create_syntax_rule_plus_2(result, gss_node_id, SlotId(262));
            }
            //SyntaxRule_Opt_5 : SyntaxRule_Plus_2.
            SlotId(262) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
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
            //SyntaxRule_Opt_5 : .
            SlotId(263) => {
                let end_slot_id = SlotId(263);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
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
            //SyntaxRule_Star_2 : . SyntaxRule_Opt_5
            SlotId(264) => {
                self.create_syntax_rule_opt_5(result, gss_node_id, SlotId(265));
            }
            //SyntaxRule_Star_2 : SyntaxRule_Opt_5.
            SlotId(265) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(265);
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
            SlotId(266) => {
                self.create_regex_block_plus_3(result, gss_node_id, SlotId(267));
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 . Layout RegexRule
            SlotId(267) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout . RegexRule
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
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout . RegexRule
            SlotId(268) => {
                self.create_regex_rule(result, gss_node_id, SlotId(269));
            }
            //RegexBlock_Plus_3 : RegexBlock_Plus_3 Layout RegexRule.
            SlotId(269) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
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
            //RegexBlock_Plus_3 : . RegexRule
            SlotId(270) => {
                self.create_regex_rule(result, gss_node_id, SlotId(271));
            }
            //RegexBlock_Plus_3 : RegexRule.
            SlotId(271) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
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
            //RegexBlock_Opt_6 : . RegexBlock_Plus_3
            SlotId(272) => {
                self.create_regex_block_plus_3(result, gss_node_id, SlotId(273));
            }
            //RegexBlock_Opt_6 : RegexBlock_Plus_3.
            SlotId(273) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
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
            //RegexBlock_Opt_6 : .
            SlotId(274) => {
                let end_slot_id = SlotId(274);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
                let nonterminal_id = NonterminalId(27);
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
            //RegexBlock_Star_3 : . RegexBlock_Opt_6
            SlotId(275) => {
                self.create_regex_block_opt_6(result, gss_node_id, SlotId(276));
            }
            //RegexBlock_Star_3 : RegexBlock_Opt_6.
            SlotId(276) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
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
            //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout Regex
            SlotId(277) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(278));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 . Layout Regex
            SlotId(278) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . Regex
                        let next_slot_id = SlotId(279);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(278),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout . Regex
            SlotId(279) => {
                self.create_regex(result, gss_node_id, SlotId(280));
            }
            //RegexRule_Plus_5 : RegexRule_Plus_5 Layout Regex.
            SlotId(280) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
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
            //RegexRule_Plus_5 : . Regex
            SlotId(281) => {
                self.create_regex(result, gss_node_id, SlotId(282));
            }
            //RegexRule_Plus_5 : Regex.
            SlotId(282) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
                let end_slot_id = SlotId(282);
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
            SlotId(283) => {
                self.create_regex_rule_plus_4(result, gss_node_id, SlotId(284));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 . Layout "|" Layout RegexRule_Plus_5
            SlotId(284) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . "|" Layout RegexRule_Plus_5
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout . "|" Layout RegexRule_Plus_5
            SlotId(285) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" . Layout RegexRule_Plus_5
                        let next_slot_id = SlotId(286);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(285),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" . Layout RegexRule_Plus_5
            SlotId(286) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout . RegexRule_Plus_5
                        let next_slot_id = SlotId(287);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(286),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout . RegexRule_Plus_5
            SlotId(287) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(288));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5.
            SlotId(288) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(30);
                let end_slot_id = SlotId(288);
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
            SlotId(289) => {
                self.create_regex_rule_plus_5(result, gss_node_id, SlotId(290));
            }
            //RegexRule_Plus_4 : RegexRule_Plus_5.
            SlotId(290) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(30);
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
            //RegexRule_Plus_6 : . RegexRule_Plus_6 Layout PostCondition
            SlotId(291) => {
                self.create_regex_rule_plus_6(result, gss_node_id, SlotId(292));
            }
            //RegexRule_Plus_6 : RegexRule_Plus_6 . Layout PostCondition
            SlotId(292) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //RegexRule_Plus_6 : RegexRule_Plus_6 Layout . PostCondition
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
            //RegexRule_Plus_6 : RegexRule_Plus_6 Layout . PostCondition
            SlotId(293) => {
                self.create_post_condition(result, gss_node_id, SlotId(294));
            }
            //RegexRule_Plus_6 : RegexRule_Plus_6 Layout PostCondition.
            SlotId(294) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
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
            //RegexRule_Plus_6 : . PostCondition
            SlotId(295) => {
                self.create_post_condition(result, gss_node_id, SlotId(296));
            }
            //RegexRule_Plus_6 : PostCondition.
            SlotId(296) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
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
            //RegexRule_Opt_7 : . RegexRule_Plus_6
            SlotId(297) => {
                self.create_regex_rule_plus_6(result, gss_node_id, SlotId(298));
            }
            //RegexRule_Opt_7 : RegexRule_Plus_6.
            SlotId(298) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(32);
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
            //RegexRule_Opt_7 : .
            SlotId(299) => {
                let end_slot_id = SlotId(299);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
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
            //RegexRule_Star_4 : . RegexRule_Opt_7
            SlotId(300) => {
                self.create_regex_rule_opt_7(result, gss_node_id, SlotId(301));
            }
            //RegexRule_Star_4 : RegexRule_Opt_7.
            SlotId(301) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
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
            //PriorityLevel_Opt_8 : . Associativity
            SlotId(302) => {
                self.create_associativity(result, gss_node_id, SlotId(303));
            }
            //PriorityLevel_Opt_8 : Associativity.
            SlotId(303) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
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
            //PriorityLevel_Opt_8 : .
            SlotId(304) => {
                let end_slot_id = SlotId(304);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
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
            //PriorityLevel_Plus_7 : . PriorityLevel_Plus_7 Layout "|" Layout Alternative
            SlotId(305) => {
                self.create_priority_level_plus_7(result, gss_node_id, SlotId(306));
            }
            //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 . Layout "|" Layout Alternative
            SlotId(306) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 Layout . "|" Layout Alternative
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
            //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 Layout . "|" Layout Alternative
            SlotId(307) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 Layout "|" . Layout Alternative
                        let next_slot_id = SlotId(308);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(307),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 Layout "|" . Layout Alternative
            SlotId(308) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 Layout "|" Layout . Alternative
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 Layout "|" Layout . Alternative
            SlotId(309) => {
                self.create_alternative(result, gss_node_id, SlotId(310));
            }
            //PriorityLevel_Plus_7 : PriorityLevel_Plus_7 Layout "|" Layout Alternative.
            SlotId(310) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(35);
                let end_slot_id = SlotId(310);
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
            //PriorityLevel_Plus_7 : . Alternative
            SlotId(311) => {
                self.create_alternative(result, gss_node_id, SlotId(312));
            }
            //PriorityLevel_Plus_7 : Alternative.
            SlotId(312) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(35);
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
            //PriorityLevel_Opt_9 : . PriorityLevel_Plus_7
            SlotId(313) => {
                self.create_priority_level_plus_7(result, gss_node_id, SlotId(314));
            }
            //PriorityLevel_Opt_9 : PriorityLevel_Plus_7.
            SlotId(314) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(36);
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
            //PriorityLevel_Opt_9 : .
            SlotId(315) => {
                let end_slot_id = SlotId(315);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
                let nonterminal_id = NonterminalId(36);
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
            //PriorityLevel_Star_5 : . PriorityLevel_Opt_9
            SlotId(316) => {
                self.create_priority_level_opt_9(result, gss_node_id, SlotId(317));
            }
            //PriorityLevel_Star_5 : PriorityLevel_Opt_9.
            SlotId(317) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(37);
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
            //Alternative_Plus_8 : . Alternative_Plus_8 Layout Symbol(0)
            SlotId(318) => {
                self.create_alternative_plus_8(result, gss_node_id, SlotId(319));
            }
            //Alternative_Plus_8 : Alternative_Plus_8 . Layout Symbol(0)
            SlotId(319) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Alternative_Plus_8 : Alternative_Plus_8 Layout . Symbol(0)
                        let next_slot_id = SlotId(320);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(319),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Plus_8 : Alternative_Plus_8 Layout . Symbol(0)
            SlotId(320) => {
                self.create_symbol(result, gss_node_id, SlotId(321), env, None, 0);
            }
            //Alternative_Plus_8 : Alternative_Plus_8 Layout Symbol(0).
            SlotId(321) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(38);
                let end_slot_id = SlotId(321);
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
            //Alternative_Plus_8 : . Symbol(0)
            SlotId(322) => {
                self.create_symbol(result, gss_node_id, SlotId(323), env, None, 0);
            }
            //Alternative_Plus_8 : Symbol(0).
            SlotId(323) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(38);
                let end_slot_id = SlotId(323);
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
            //Alternative_Opt_10 : . Alternative_Plus_8
            SlotId(324) => {
                self.create_alternative_plus_8(result, gss_node_id, SlotId(325));
            }
            //Alternative_Opt_10 : Alternative_Plus_8.
            SlotId(325) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(39);
                let end_slot_id = SlotId(325);
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
            SlotId(326) => {
                let end_slot_id = SlotId(326);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
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
            //Alternative_Star_6 : . Alternative_Opt_10
            SlotId(327) => {
                self.create_alternative_opt_10(result, gss_node_id, SlotId(328));
            }
            //Alternative_Star_6 : Alternative_Opt_10.
            SlotId(328) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(40);
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
            //Alternative_Opt_11 : . Label
            SlotId(329) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Label", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Label", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(5), i, j);
                        //Alternative_Opt_11 : Label.
                        let next_slot_id = SlotId(330);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Label",
                            i,
                            SlotId(329),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Opt_11 : Label.
            SlotId(330) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(41);
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
            //Alternative_Opt_11 : .
            SlotId(331) => {
                let end_slot_id = SlotId(331);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
                let nonterminal_id = NonterminalId(41);
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
            SlotId(332) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Symbol_Group_0 : "|" . Layout Symbol(0)
                        let next_slot_id = SlotId(333);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(332),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" . Layout Symbol(0)
            SlotId(333) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol_Group_0 : "|" Layout . Symbol(0)
                        let next_slot_id = SlotId(334);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(333),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Group_0 : "|" Layout . Symbol(0)
            SlotId(334) => {
                self.create_symbol(result, gss_node_id, SlotId(335), env, None, 0);
            }
            //Symbol_Group_0 : "|" Layout Symbol(0).
            SlotId(335) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(42);
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
            //Symbol_Plus_9 : . Symbol_Plus_9 Layout Symbol_Group_0
            SlotId(336) => {
                self.create_symbol_plus_9(result, gss_node_id, SlotId(337));
            }
            //Symbol_Plus_9 : Symbol_Plus_9 . Layout Symbol_Group_0
            SlotId(337) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Symbol_Plus_9 : Symbol_Plus_9 Layout . Symbol_Group_0
                        let next_slot_id = SlotId(338);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(337),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol_Plus_9 : Symbol_Plus_9 Layout . Symbol_Group_0
            SlotId(338) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(339));
            }
            //Symbol_Plus_9 : Symbol_Plus_9 Layout Symbol_Group_0.
            SlotId(339) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
                let end_slot_id = SlotId(339);
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
            //Symbol_Plus_9 : . Symbol_Group_0
            SlotId(340) => {
                self.create_symbol_group_0(result, gss_node_id, SlotId(341));
            }
            //Symbol_Plus_9 : Symbol_Group_0.
            SlotId(341) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(43);
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
            //Regex_Group_1 : . "|" Layout Regex
            SlotId(342) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //Regex_Group_1 : "|" . Layout Regex
                        let next_slot_id = SlotId(343);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(342),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_1 : "|" . Layout Regex
            SlotId(343) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex_Group_1 : "|" Layout . Regex
                        let next_slot_id = SlotId(344);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(343),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Group_1 : "|" Layout . Regex
            SlotId(344) => {
                self.create_regex(result, gss_node_id, SlotId(345));
            }
            //Regex_Group_1 : "|" Layout Regex.
            SlotId(345) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(44);
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
            //Regex_Plus_10 : . Regex_Plus_10 Layout Regex_Group_1
            SlotId(346) => {
                self.create_regex_plus_10(result, gss_node_id, SlotId(347));
            }
            //Regex_Plus_10 : Regex_Plus_10 . Layout Regex_Group_1
            SlotId(347) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //Regex_Plus_10 : Regex_Plus_10 Layout . Regex_Group_1
                        let next_slot_id = SlotId(348);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(347),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex_Plus_10 : Regex_Plus_10 Layout . Regex_Group_1
            SlotId(348) => {
                self.create_regex_group_1(result, gss_node_id, SlotId(349));
            }
            //Regex_Plus_10 : Regex_Plus_10 Layout Regex_Group_1.
            SlotId(349) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(45);
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
            //Regex_Plus_10 : . Regex_Group_1
            SlotId(350) => {
                self.create_regex_group_1(result, gss_node_id, SlotId(351));
            }
            //Regex_Plus_10 : Regex_Group_1.
            SlotId(351) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(45);
                let end_slot_id = SlotId(351);
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
            //CharClass_Opt_12 : . "!"
            SlotId(352) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(30), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(30), i, j);
                        //CharClass_Opt_12 : "!".
                        let next_slot_id = SlotId(353);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(352),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Opt_12 : "!".
            SlotId(353) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(46);
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
            //CharClass_Opt_12 : .
            SlotId(354) => {
                let end_slot_id = SlotId(354);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(35), input_index, input_index);
                let nonterminal_id = NonterminalId(46);
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
            //CharClass_Plus_11 : . CharClass_Plus_11 Layout RangeElement
            SlotId(355) => {
                self.create_char_class_plus_11(result, gss_node_id, SlotId(356));
            }
            //CharClass_Plus_11 : CharClass_Plus_11 . Layout RangeElement
            SlotId(356) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //CharClass_Plus_11 : CharClass_Plus_11 Layout . RangeElement
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
            //CharClass_Plus_11 : CharClass_Plus_11 Layout . RangeElement
            SlotId(357) => {
                self.create_range_element(result, gss_node_id, SlotId(358));
            }
            //CharClass_Plus_11 : CharClass_Plus_11 Layout RangeElement.
            SlotId(358) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(47);
                let end_slot_id = SlotId(358);
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
            //CharClass_Plus_11 : . RangeElement
            SlotId(359) => {
                self.create_range_element(result, gss_node_id, SlotId(360));
            }
            //CharClass_Plus_11 : RangeElement.
            SlotId(360) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(47);
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
            //StartGrammar : . Layout start:Grammar Layout
            SlotId(361) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartGrammar : Layout . start:Grammar Layout
                        let next_slot_id = SlotId(362);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(361),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout . start:Grammar Layout
            SlotId(362) => {
                self.create_grammar(result, gss_node_id, SlotId(363));
            }
            //StartGrammar : Layout start:Grammar . Layout
            SlotId(363) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartGrammar : Layout start:Grammar Layout.
                        let next_slot_id = SlotId(364);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(363),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout start:Grammar Layout.
            SlotId(364) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(48);
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
            //StartLayoutDef : . Layout start:LayoutDef Layout
            SlotId(365) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartLayoutDef : Layout . start:LayoutDef Layout
                        let next_slot_id = SlotId(366);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(365),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartLayoutDef : Layout . start:LayoutDef Layout
            SlotId(366) => {
                self.create_layout_def(result, gss_node_id, SlotId(367));
            }
            //StartLayoutDef : Layout start:LayoutDef . Layout
            SlotId(367) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartLayoutDef : Layout start:LayoutDef Layout.
                        let next_slot_id = SlotId(368);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(367),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartLayoutDef : Layout start:LayoutDef Layout.
            SlotId(368) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(49);
                let end_slot_id = SlotId(368);
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
            SlotId(369) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartSyntaxRule : Layout . start:SyntaxRule Layout
                        let next_slot_id = SlotId(370);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(369),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout . start:SyntaxRule Layout
            SlotId(370) => {
                self.create_syntax_rule(result, gss_node_id, SlotId(371));
            }
            //StartSyntaxRule : Layout start:SyntaxRule . Layout
            SlotId(371) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartSyntaxRule : Layout start:SyntaxRule Layout.
                        let next_slot_id = SlotId(372);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(371),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSyntaxRule : Layout start:SyntaxRule Layout.
            SlotId(372) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(50);
                let end_slot_id = SlotId(372);
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
            SlotId(373) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartAnnotation : Layout . start:Annotation Layout
                        let next_slot_id = SlotId(374);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(373),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAnnotation : Layout . start:Annotation Layout
            SlotId(374) => {
                self.create_annotation(result, gss_node_id, SlotId(375));
            }
            //StartAnnotation : Layout start:Annotation . Layout
            SlotId(375) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartAnnotation : Layout start:Annotation Layout.
                        let next_slot_id = SlotId(376);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(375),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAnnotation : Layout start:Annotation Layout.
            SlotId(376) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(51);
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
            //StartRegexBlock : . Layout start:RegexBlock Layout
            SlotId(377) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRegexBlock : Layout . start:RegexBlock Layout
                        let next_slot_id = SlotId(378);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(377),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexBlock : Layout . start:RegexBlock Layout
            SlotId(378) => {
                self.create_regex_block(result, gss_node_id, SlotId(379));
            }
            //StartRegexBlock : Layout start:RegexBlock . Layout
            SlotId(379) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRegexBlock : Layout start:RegexBlock Layout.
                        let next_slot_id = SlotId(380);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(379),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexBlock : Layout start:RegexBlock Layout.
            SlotId(380) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(52);
                let end_slot_id = SlotId(380);
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
            SlotId(381) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRegexRule : Layout . start:RegexRule Layout
                        let next_slot_id = SlotId(382);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartRegexRule : Layout . start:RegexRule Layout
            SlotId(382) => {
                self.create_regex_rule(result, gss_node_id, SlotId(383));
            }
            //StartRegexRule : Layout start:RegexRule . Layout
            SlotId(383) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRegexRule : Layout start:RegexRule Layout.
                        let next_slot_id = SlotId(384);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(383),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegexRule : Layout start:RegexRule Layout.
            SlotId(384) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(53);
                let end_slot_id = SlotId(384);
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
            SlotId(385) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartPostCondition : Layout . start:PostCondition Layout
                        let next_slot_id = SlotId(386);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(385),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPostCondition : Layout . start:PostCondition Layout
            SlotId(386) => {
                self.create_post_condition(result, gss_node_id, SlotId(387));
            }
            //StartPostCondition : Layout start:PostCondition . Layout
            SlotId(387) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartPostCondition : Layout start:PostCondition Layout.
                        let next_slot_id = SlotId(388);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(387),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPostCondition : Layout start:PostCondition Layout.
            SlotId(388) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(54);
                let end_slot_id = SlotId(388);
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
            SlotId(389) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartPriorityLevel : Layout . start:PriorityLevel Layout
                        let next_slot_id = SlotId(390);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(389),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout . start:PriorityLevel Layout
            SlotId(390) => {
                self.create_priority_level(result, gss_node_id, SlotId(391));
            }
            //StartPriorityLevel : Layout start:PriorityLevel . Layout
            SlotId(391) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartPriorityLevel : Layout start:PriorityLevel Layout.
                        let next_slot_id = SlotId(392);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(391),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout start:PriorityLevel Layout.
            SlotId(392) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(55);
                let end_slot_id = SlotId(392);
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
            SlotId(393) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartAssociativity : Layout . start:Associativity Layout
                        let next_slot_id = SlotId(394);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(393),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAssociativity : Layout . start:Associativity Layout
            SlotId(394) => {
                self.create_associativity(result, gss_node_id, SlotId(395));
            }
            //StartAssociativity : Layout start:Associativity . Layout
            SlotId(395) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartAssociativity : Layout start:Associativity Layout.
                        let next_slot_id = SlotId(396);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(395),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAssociativity : Layout start:Associativity Layout.
            SlotId(396) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(56);
                let end_slot_id = SlotId(396);
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
            SlotId(397) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartAlternative : Layout . start:Alternative Layout
                        let next_slot_id = SlotId(398);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartAlternative : Layout . start:Alternative Layout
            SlotId(398) => {
                self.create_alternative(result, gss_node_id, SlotId(399));
            }
            //StartAlternative : Layout start:Alternative . Layout
            SlotId(399) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartAlternative : Layout start:Alternative Layout.
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
            //StartAlternative : Layout start:Alternative Layout.
            SlotId(400) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(57);
                let end_slot_id = SlotId(400);
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
            SlotId(401) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartSymbol : Layout . start:Symbol(0) Layout
                        let next_slot_id = SlotId(402);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartSymbol : Layout . start:Symbol(0) Layout
            SlotId(402) => {
                self.create_symbol(result, gss_node_id, SlotId(403), env, None, 0);
            }
            //StartSymbol : Layout start:Symbol(0) . Layout
            SlotId(403) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartSymbol : Layout start:Symbol(0) Layout.
                        let next_slot_id = SlotId(404);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(403),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSymbol : Layout start:Symbol(0) Layout.
            SlotId(404) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(58);
                let end_slot_id = SlotId(404);
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
            SlotId(405) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRegex : Layout . start:Regex Layout
                        let next_slot_id = SlotId(406);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(405),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout . start:Regex Layout
            SlotId(406) => {
                self.create_regex(result, gss_node_id, SlotId(407));
            }
            //StartRegex : Layout start:Regex . Layout
            SlotId(407) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRegex : Layout start:Regex Layout.
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
                            "Layout",
                            i,
                            SlotId(407),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout start:Regex Layout.
            SlotId(408) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(59);
                let end_slot_id = SlotId(408);
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
            SlotId(409) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartCharClass : Layout . start:CharClass Layout
                        let next_slot_id = SlotId(410);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(409),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout . start:CharClass Layout
            SlotId(410) => {
                self.create_char_class(result, gss_node_id, SlotId(411));
            }
            //StartCharClass : Layout start:CharClass . Layout
            SlotId(411) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartCharClass : Layout start:CharClass Layout.
                        let next_slot_id = SlotId(412);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(411),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout start:CharClass Layout.
            SlotId(412) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(60);
                let end_slot_id = SlotId(412);
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
            SlotId(413) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRangeElement : Layout . start:RangeElement Layout
                        let next_slot_id = SlotId(414);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartRangeElement : Layout . start:RangeElement Layout
            SlotId(414) => {
                self.create_range_element(result, gss_node_id, SlotId(415));
            }
            //StartRangeElement : Layout start:RangeElement . Layout
            SlotId(415) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRangeElement : Layout start:RangeElement Layout.
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
            //StartRangeElement : Layout start:RangeElement Layout.
            SlotId(416) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(61);
                let end_slot_id = SlotId(416);
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
            SlotId(417) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRange : Layout . start:Range Layout
                        let next_slot_id = SlotId(418);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartRange : Layout . start:Range Layout
            SlotId(418) => {
                self.create_range(result, gss_node_id, SlotId(419));
            }
            //StartRange : Layout start:Range . Layout
            SlotId(419) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(34), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(34), i, j);
                        //StartRange : Layout start:Range Layout.
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
            //StartRange : Layout start:Range Layout.
            SlotId(420) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(62);
                let end_slot_id = SlotId(420);
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
                //SyntaxRule : . SyntaxRule_Opt_4 Layout head:Identifier Layout "=" Layout SyntaxRule_Star_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Annotation
            NonterminalId(3) => {
                //Annotation : . "@NoLayout"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(22),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Annotation : . "@Layout" Layout "(" Layout Identifier Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(24),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexBlock
            NonterminalId(4) => {
                //RegexBlock : . "regex" Layout "{" Layout RegexBlock_Star_3 Layout "}"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(32),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule
            NonterminalId(5) => {
                //RegexRule : . Identifier Layout "=" Layout body:RegexRule_Plus_4 Layout RegexRule_Star_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(40),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PostCondition
            NonterminalId(6) => {
                //PostCondition : . "\" Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(48),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PostCondition : . "!>>" Layout Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(52),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel
            NonterminalId(7) => {
                //PriorityLevel : . PriorityLevel_Opt_8 Layout PriorityLevel_Star_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(56),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Associativity
            NonterminalId(8) => {
                //Associativity : . "left"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(60),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Associativity : . "right"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(62),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Associativity : . "none"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(64),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative
            NonterminalId(9) => {
                //Alternative : . Alternative_Star_6 Layout Alternative_Opt_11
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(66),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol
            NonterminalId(63) => {
                //Symbol(p: i32) : . Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(70),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "(" Layout Alternative_Plus_8 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(73),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "(" Layout first:Symbol(0) Layout rest:Symbol_Plus_9 Layout ")" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(80),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . """ Layout String Layout """ return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(89),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(96),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . "{" Layout symbol:Symbol(0) Layout sep:Symbol(0) Layout "}" Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(107),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "*" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(118),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "+" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(125),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "?" return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(132),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "\" Layout Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(139),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . [2 >= p] l=Symbol(p) [l == 0 || l >= 2] Layout "!>>" Layout Identifier return 0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(148),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol(p: i32) : . label:Identifier Layout ":" Layout Symbol(1) return 1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(157),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex
            NonterminalId(10) => {
                //Regex : . Regex Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(164),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . Regex Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(168),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . Regex Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(172),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "(" Layout first:Regex Layout rest:Regex_Plus_10 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(176),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "(" Layout RegexRule_Plus_5 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(184),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . CharClass
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(190),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . "'" Layout Char Layout "'"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(192),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex : . """ Layout String Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(198),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass
            NonterminalId(11) => {
                //CharClass : . neg:CharClass_Opt_12 Layout "[" Layout CharClass_Plus_11 Layout "]"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(204),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RangeElement
            NonterminalId(12) => {
                //RangeElement : . Range
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(212),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RangeElement : . RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(214),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Range
            NonterminalId(13) => {
                //Range : . start:RangeChar Layout "-" Layout end:RangeChar
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(216),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_0
            NonterminalId(14) => {
                //Grammar_Opt_0 : . LayoutDef
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(222),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(224),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Plus_0
            NonterminalId(15) => {
                //Grammar_Plus_0 : . Grammar_Plus_0 Layout SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(225),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Plus_0 : . SyntaxRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(229),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_1
            NonterminalId(16) => {
                //Grammar_Opt_1 : . Grammar_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(231),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(233),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Star_0
            NonterminalId(17) => {
                //Grammar_Star_0 : . Grammar_Opt_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(234),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Grammar_Opt_2
            NonterminalId(18) => {
                //Grammar_Opt_2 : . RegexBlock
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(236),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Grammar_Opt_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(238),
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
                    slot_id: SlotId(239),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //LayoutDef_Plus_1 : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(243),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Opt_3
            NonterminalId(20) => {
                //LayoutDef_Opt_3 : . LayoutDef_Plus_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(245),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //LayoutDef_Opt_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(247),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //LayoutDef_Star_1
            NonterminalId(21) => {
                //LayoutDef_Star_1 : . LayoutDef_Opt_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(248),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Opt_4
            NonterminalId(22) => {
                //SyntaxRule_Opt_4 : . Annotation
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(250),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Opt_4 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(252),
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
                    slot_id: SlotId(253),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Plus_2 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(259),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Opt_5
            NonterminalId(24) => {
                //SyntaxRule_Opt_5 : . SyntaxRule_Plus_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(261),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //SyntaxRule_Opt_5 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(263),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //SyntaxRule_Star_2
            NonterminalId(25) => {
                //SyntaxRule_Star_2 : . SyntaxRule_Opt_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(264),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexBlock_Plus_3
            NonterminalId(26) => {
                //RegexBlock_Plus_3 : . RegexBlock_Plus_3 Layout RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(266),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexBlock_Plus_3 : . RegexRule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(270),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexBlock_Opt_6
            NonterminalId(27) => {
                //RegexBlock_Opt_6 : . RegexBlock_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(272),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexBlock_Opt_6 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(274),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexBlock_Star_3
            NonterminalId(28) => {
                //RegexBlock_Star_3 : . RegexBlock_Opt_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(275),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_5
            NonterminalId(29) => {
                //RegexRule_Plus_5 : . RegexRule_Plus_5 Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(277),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_5 : . Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(281),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_4
            NonterminalId(30) => {
                //RegexRule_Plus_4 : . RegexRule_Plus_4 Layout "|" Layout RegexRule_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(283),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_4 : . RegexRule_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(289),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Plus_6
            NonterminalId(31) => {
                //RegexRule_Plus_6 : . RegexRule_Plus_6 Layout PostCondition
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(291),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Plus_6 : . PostCondition
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(295),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Opt_7
            NonterminalId(32) => {
                //RegexRule_Opt_7 : . RegexRule_Plus_6
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(297),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //RegexRule_Opt_7 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(299),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //RegexRule_Star_4
            NonterminalId(33) => {
                //RegexRule_Star_4 : . RegexRule_Opt_7
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(300),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Opt_8
            NonterminalId(34) => {
                //PriorityLevel_Opt_8 : . Associativity
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(302),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Opt_8 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(304),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Plus_7
            NonterminalId(35) => {
                //PriorityLevel_Plus_7 : . PriorityLevel_Plus_7 Layout "|" Layout Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(305),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Plus_7 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(311),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Opt_9
            NonterminalId(36) => {
                //PriorityLevel_Opt_9 : . PriorityLevel_Plus_7
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(313),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //PriorityLevel_Opt_9 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(315),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //PriorityLevel_Star_5
            NonterminalId(37) => {
                //PriorityLevel_Star_5 : . PriorityLevel_Opt_9
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(316),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Plus_8
            NonterminalId(38) => {
                //Alternative_Plus_8 : . Alternative_Plus_8 Layout Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(318),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Plus_8 : . Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(322),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Opt_10
            NonterminalId(39) => {
                //Alternative_Opt_10 : . Alternative_Plus_8
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(324),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Opt_10 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(326),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Star_6
            NonterminalId(40) => {
                //Alternative_Star_6 : . Alternative_Opt_10
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(327),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Alternative_Opt_11
            NonterminalId(41) => {
                //Alternative_Opt_11 : . Label
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(329),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Alternative_Opt_11 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(331),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Group_0
            NonterminalId(42) => {
                //Symbol_Group_0 : . "|" Layout Symbol(0)
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(332),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Symbol_Plus_9
            NonterminalId(43) => {
                //Symbol_Plus_9 : . Symbol_Plus_9 Layout Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(336),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Symbol_Plus_9 : . Symbol_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(340),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex_Group_1
            NonterminalId(44) => {
                //Regex_Group_1 : . "|" Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(342),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Regex_Plus_10
            NonterminalId(45) => {
                //Regex_Plus_10 : . Regex_Plus_10 Layout Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(346),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Regex_Plus_10 : . Regex_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(350),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass_Opt_12
            NonterminalId(46) => {
                //CharClass_Opt_12 : . "!"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(352),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //CharClass_Opt_12 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(354),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //CharClass_Plus_11
            NonterminalId(47) => {
                //CharClass_Plus_11 : . CharClass_Plus_11 Layout RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(355),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //CharClass_Plus_11 : . RangeElement
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(359),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartGrammar
            NonterminalId(48) => {
                //StartGrammar : . Layout start:Grammar Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(361),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartLayoutDef
            NonterminalId(49) => {
                //StartLayoutDef : . Layout start:LayoutDef Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(365),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartSyntaxRule
            NonterminalId(50) => {
                //StartSyntaxRule : . Layout start:SyntaxRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(369),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAnnotation
            NonterminalId(51) => {
                //StartAnnotation : . Layout start:Annotation Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(373),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegexBlock
            NonterminalId(52) => {
                //StartRegexBlock : . Layout start:RegexBlock Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(377),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegexRule
            NonterminalId(53) => {
                //StartRegexRule : . Layout start:RegexRule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(381),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartPostCondition
            NonterminalId(54) => {
                //StartPostCondition : . Layout start:PostCondition Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(385),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartPriorityLevel
            NonterminalId(55) => {
                //StartPriorityLevel : . Layout start:PriorityLevel Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(389),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAssociativity
            NonterminalId(56) => {
                //StartAssociativity : . Layout start:Associativity Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(393),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartAlternative
            NonterminalId(57) => {
                //StartAlternative : . Layout start:Alternative Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(397),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartSymbol
            NonterminalId(58) => {
                //StartSymbol : . Layout start:Symbol(0) Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(401),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRegex
            NonterminalId(59) => {
                //StartRegex : . Layout start:Regex Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(405),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartCharClass
            NonterminalId(60) => {
                //StartCharClass : . Layout start:CharClass Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(409),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRangeElement
            NonterminalId(61) => {
                //StartRangeElement : . Layout start:RangeElement Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(413),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartRange
            NonterminalId(62) => {
                //StartRange : . Layout start:Range Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(417),
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
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 64],
    //GSS index for nonterminal Symbol
    gss_nodes_index_symbol: Vec<(u32, i32, GssNodeId)>,
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 64],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 421],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 36],
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
            gss_nodes_index: [const { vec![] }; 64],
            gss_nodes_index_symbol: vec![],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 64],
            intermediate_nodes_index: [const { InlineMap::Empty }; 421],
            terminal_nodes_index: [const { InlineMap::Empty }; 36],
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
    fn create_annotation(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(3), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block(
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
    fn create_post_condition(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(6), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(7), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_associativity(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(8), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(9), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(10), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(11), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(12), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(13), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(14), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_plus_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(15), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(16), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_star_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(17), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_grammar_opt_2(
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
    fn create_layout_def_opt_3(
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
    fn create_syntax_rule_opt_4(
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
    fn create_syntax_rule_opt_5(
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
    fn create_regex_block_plus_3(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(26), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_opt_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(27), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_block_star_3(
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
    fn create_regex_rule_plus_4(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(30), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_plus_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(31), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_opt_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(32), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_rule_star_4(
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
    fn create_priority_level_plus_7(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(35), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_opt_9(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(36), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_priority_level_star_5(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(37), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_plus_8(
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
    fn create_alternative_star_6(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(40), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_alternative_opt_11(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(41), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_symbol_group_0(
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
    fn create_regex_group_1(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(44), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_regex_plus_10(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(45), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_opt_12(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(46), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_char_class_plus_11(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(47), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_grammar(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(48), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_layout_def(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(49), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_syntax_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(50), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_annotation(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(51), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex_block(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(52), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex_rule(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(53), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_post_condition(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(54), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_priority_level(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(55), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_associativity(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(56), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_alternative(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(57), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_symbol(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(58), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_regex(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(59), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_char_class(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(60), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range_element(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(61), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_range(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(62), sppf_node_id, gss_node_id, return_slot);
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
            record!(self, GSSNodeFound, NonterminalId(63), i);
            self.add_edge_to_existing_gss_node(
                existing_gss_node_id,
                gss_node_id,
                left_child,
                return_slot,
                env,
                binding,
            );
        } else {
            record!(self, GSSNodeNotFound, NonterminalId(63), i);
            let new_gss_node_id = self.new_gss_node(NonterminalId(63), i);
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
            self.add_first_descriptors(NonterminalId(63), i, new_gss_node_id, Some(env_id));
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

