use crate::{
    scanner::IggyScanner,
    types::{EbnfKind, Nonterminal, Slot, Terminal},
};
#[cfg(feature = "debug-trace")]
use iguana::trace::TraceEvent;
use iguana::{
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
pub const NONTERMINALS: [Nonterminal; 35] = [
    Nonterminal {
        name: "Grammar",
        display: "Grammar",
        kind: None,
    },
    Nonterminal {
        name: "Rule",
        display: "Rule",
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
        name: "CharRange",
        display: "CharRange",
        kind: None,
    },
    Nonterminal {
        name: "Grammar_Plus_0",
        display: "Rule+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Grammar_Opt_0",
        display: "Rule+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Grammar_Star_0",
        display: "Rule*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Rule_Plus_1",
        display: "{PriorityLevel \">\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Rule_Opt_1",
        display: "{PriorityLevel \">\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Rule_Star_1",
        display: "{PriorityLevel \">\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Rule_Plus_3",
        display: "Regex+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Rule_Plus_2",
        display: "{Regex+ \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "PriorityLevel_Plus_4",
        display: "{Alternative \"|\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_2",
        display: "{Alternative \"|\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "PriorityLevel_Star_2",
        display: "{Alternative \"|\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Plus_5",
        display: "Symbol+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Alternative_Opt_3",
        display: "Symbol+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Alternative_Star_3",
        display: "Symbol*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Regex_Opt_4",
        display: "{Regex+ \"|\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Regex_Star_4",
        display: "{Regex+ \"|\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "CharClass_Opt_5",
        display: "\"!\"?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "CharClass_Alt_0",
        display: "(CharRange | Char)",
        kind: Some(EbnfKind::Alt),
    },
    Nonterminal {
        name: "CharClass_Plus_6",
        display: "(CharRange | Char)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "StartGrammar",
        display: "StartGrammar",
        kind: None,
    },
    Nonterminal {
        name: "StartRule",
        display: "StartRule",
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
        name: "StartCharRange",
        display: "StartCharRange",
        kind: None,
    },
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Grammar" => NonterminalId(0), "Rule" => NonterminalId(1), "PriorityLevel" =>
    NonterminalId(2), "Alternative" => NonterminalId(3), "Symbol" => NonterminalId(4),
    "Regex" => NonterminalId(5), "CharClass" => NonterminalId(6), "CharRange" =>
    NonterminalId(7), "Grammar_Plus_0" => NonterminalId(8), "Grammar_Opt_0" =>
    NonterminalId(9), "Grammar_Star_0" => NonterminalId(10), "Rule_Plus_1" =>
    NonterminalId(11), "Rule_Opt_1" => NonterminalId(12), "Rule_Star_1" =>
    NonterminalId(13), "Rule_Plus_3" => NonterminalId(14), "Rule_Plus_2" =>
    NonterminalId(15), "PriorityLevel_Plus_4" => NonterminalId(16), "PriorityLevel_Opt_2"
    => NonterminalId(17), "PriorityLevel_Star_2" => NonterminalId(18),
    "Alternative_Plus_5" => NonterminalId(19), "Alternative_Opt_3" => NonterminalId(20),
    "Alternative_Star_3" => NonterminalId(21), "Regex_Opt_4" => NonterminalId(22),
    "Regex_Star_4" => NonterminalId(23), "CharClass_Opt_5" => NonterminalId(24),
    "CharClass_Alt_0" => NonterminalId(25), "CharClass_Plus_6" => NonterminalId(26),
    "StartGrammar" => NonterminalId(27), "StartRule" => NonterminalId(28),
    "StartPriorityLevel" => NonterminalId(29), "StartAlternative" => NonterminalId(30),
    "StartSymbol" => NonterminalId(31), "StartRegex" => NonterminalId(32),
    "StartCharClass" => NonterminalId(33), "StartCharRange" => NonterminalId(34)
};
pub const TERMINALS: [Terminal; 23] = [
    Terminal { name: "Identifier" },
    Terminal { name: "String" },
    Terminal { name: "Char" },
    Terminal { name: "WS" },
    Terminal {
        name: "\"grammar\"",
    },
    Terminal { name: "\"=\"" },
    Terminal { name: "\">\"" },
    Terminal { name: "\"/\"" },
    Terminal { name: "\"|\"" },
    Terminal { name: "\"*\"" },
    Terminal { name: "\"+\"" },
    Terminal { name: "\"(\"" },
    Terminal { name: "\")\"" },
    Terminal { name: "\"\"\"" },
    Terminal { name: "\"{\"" },
    Terminal { name: "\"}\"" },
    Terminal { name: "\"?\"" },
    Terminal { name: "\"!\"" },
    Terminal { name: "\"[\"" },
    Terminal { name: "\"]\"" },
    Terminal { name: "\"-\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 226] = [
    Slot {
        display_name: "Grammar : . \"grammar\" Layout Identifier Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" . Layout Identifier Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout . Identifier Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier . Layout Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout . Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Layout Identifier Layout Rule*.",
    },
    Slot {
        display_name: "Rule : . Identifier Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier . Layout \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier Layout . \"=\" Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" . Layout {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout . {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout {PriorityLevel \">\"}*.",
    },
    Slot {
        display_name: "Rule : . Identifier Layout \"=\" Layout \"/\" Layout {Regex+ \"|\"}+ Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier . Layout \"=\" Layout \"/\" Layout {Regex+ \"|\"}+ Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout . \"=\" Layout \"/\" Layout {Regex+ \"|\"}+ Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" . Layout \"/\" Layout {Regex+ \"|\"}+ Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout . \"/\" Layout {Regex+ \"|\"}+ Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout \"/\" . Layout {Regex+ \"|\"}+ Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout \"/\" Layout . {Regex+ \"|\"}+ Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout \"/\" Layout {Regex+ \"|\"}+ . Layout \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout \"/\" Layout {Regex+ \"|\"}+ Layout . \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier Layout \"=\" Layout \"/\" Layout {Regex+ \"|\"}+ Layout \"/\".",
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
        display_name: "Symbol : . \"(\" Layout Symbol Layout \"|\" Layout Symbol Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" . Layout Symbol Layout \"|\" Layout Symbol Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout . Symbol Layout \"|\" Layout Symbol Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol . Layout \"|\" Layout Symbol Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout . \"|\" Layout Symbol Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout \"|\" . Layout Symbol Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout \"|\" Layout . Symbol Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout \"|\" Layout Symbol . Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout \"|\" Layout Symbol Layout . \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol Layout \"|\" Layout Symbol Layout \")\".",
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
        display_name: "Symbol : . \"(\" Layout Symbol* Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" . Layout Symbol* Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout . Symbol* Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol* . Layout \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol* Layout . \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Layout Symbol* Layout \")\".",
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
        display_name: "Regex : . \"(\" Layout {Regex+ \"|\"}* Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" . Layout {Regex+ \"|\"}* Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout . {Regex+ \"|\"}* Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout {Regex+ \"|\"}* . Layout \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout {Regex+ \"|\"}* Layout . \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" Layout {Regex+ \"|\"}* Layout \")\".",
    },
    Slot {
        display_name: "Regex : . CharClass",
    },
    Slot {
        display_name: "Regex : CharClass.",
    },
    Slot {
        display_name: "Regex : . Char",
    },
    Slot {
        display_name: "Regex : Char.",
    },
    Slot {
        display_name: "CharClass : . \"!\"? Layout \"[\" Layout (CharRange | Char)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? . Layout \"[\" Layout (CharRange | Char)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout . \"[\" Layout (CharRange | Char)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" . Layout (CharRange | Char)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout . (CharRange | Char)+ Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout (CharRange | Char)+ . Layout \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout (CharRange | Char)+ Layout . \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? Layout \"[\" Layout (CharRange | Char)+ Layout \"]\".",
    },
    Slot {
        display_name: "CharRange : . Char Layout \"-\" Layout Char",
    },
    Slot {
        display_name: "CharRange : Char . Layout \"-\" Layout Char",
    },
    Slot {
        display_name: "CharRange : Char Layout . \"-\" Layout Char",
    },
    Slot {
        display_name: "CharRange : Char Layout \"-\" . Layout Char",
    },
    Slot {
        display_name: "CharRange : Char Layout \"-\" Layout . Char",
    },
    Slot {
        display_name: "CharRange : Char Layout \"-\" Layout Char.",
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
        display_name: "{Regex+ \"|\"}+? : . {Regex+ \"|\"}+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+? : {Regex+ \"|\"}+.",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+? : .",
    },
    Slot {
        display_name: "{Regex+ \"|\"}* : . {Regex+ \"|\"}+?",
    },
    Slot {
        display_name: "{Regex+ \"|\"}* : {Regex+ \"|\"}+?.",
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
        display_name: "(CharRange | Char) : . CharRange",
    },
    Slot {
        display_name: "(CharRange | Char) : CharRange.",
    },
    Slot {
        display_name: "(CharRange | Char) : . Char",
    },
    Slot {
        display_name: "(CharRange | Char) : Char.",
    },
    Slot {
        display_name: "(CharRange | Char)+ : . (CharRange | Char)+ Layout (CharRange | Char)",
    },
    Slot {
        display_name: "(CharRange | Char)+ : (CharRange | Char)+ . Layout (CharRange | Char)",
    },
    Slot {
        display_name: "(CharRange | Char)+ : (CharRange | Char)+ Layout . (CharRange | Char)",
    },
    Slot {
        display_name: "(CharRange | Char)+ : (CharRange | Char)+ Layout (CharRange | Char).",
    },
    Slot {
        display_name: "(CharRange | Char)+ : . (CharRange | Char)",
    },
    Slot {
        display_name: "(CharRange | Char)+ : (CharRange | Char).",
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
        display_name: "StartRule : . Layout Rule Layout",
    },
    Slot {
        display_name: "StartRule : Layout . Rule Layout",
    },
    Slot {
        display_name: "StartRule : Layout Rule . Layout",
    },
    Slot {
        display_name: "StartRule : Layout Rule Layout.",
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
        display_name: "StartCharRange : . Layout CharRange Layout",
    },
    Slot {
        display_name: "StartCharRange : Layout . CharRange Layout",
    },
    Slot {
        display_name: "StartCharRange : Layout CharRange . Layout",
    },
    Slot {
        display_name: "StartCharRange : Layout CharRange Layout.",
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
            //Grammar : . "grammar" Layout Identifier Layout Grammar_Star_0
            SlotId(0) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Grammar : "grammar" . Layout Identifier Layout Grammar_Star_0
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
            //Grammar : "grammar" . Layout Identifier Layout Grammar_Star_0
            SlotId(1) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Grammar : "grammar" Layout . Identifier Layout Grammar_Star_0
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
            //Grammar : "grammar" Layout . Identifier Layout Grammar_Star_0
            SlotId(2) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Grammar : "grammar" Layout Identifier . Layout Grammar_Star_0
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
            //Grammar : "grammar" Layout Identifier . Layout Grammar_Star_0
            SlotId(3) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Grammar : "grammar" Layout Identifier Layout . Grammar_Star_0
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
            //Grammar : "grammar" Layout Identifier Layout . Grammar_Star_0
            SlotId(4) => {
                self.create(NonterminalId(10), result, gss_node_id, SlotId(5));
            }
            //Grammar : "grammar" Layout Identifier Layout Grammar_Star_0.
            SlotId(5) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
                let end_slot_id = SlotId(5);
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
            //Rule : . Identifier Layout "=" Layout Rule_Star_1
            SlotId(6) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Rule : Identifier . Layout "=" Layout Rule_Star_1
                        let next_slot_id = SlotId(7);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(6),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier . Layout "=" Layout Rule_Star_1
            SlotId(7) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule : Identifier Layout . "=" Layout Rule_Star_1
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
            //Rule : Identifier Layout . "=" Layout Rule_Star_1
            SlotId(8) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(5), i, j);
                        //Rule : Identifier Layout "=" . Layout Rule_Star_1
                        let next_slot_id = SlotId(9);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(8),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier Layout "=" . Layout Rule_Star_1
            SlotId(9) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule : Identifier Layout "=" Layout . Rule_Star_1
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
            //Rule : Identifier Layout "=" Layout . Rule_Star_1
            SlotId(10) => {
                self.create(NonterminalId(13), result, gss_node_id, SlotId(11));
            }
            //Rule : Identifier Layout "=" Layout Rule_Star_1.
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
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Rule : . Identifier Layout "=" Layout "/" Layout Rule_Plus_2 Layout "/"
            SlotId(12) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Rule : Identifier . Layout "=" Layout "/" Layout Rule_Plus_2 Layout "/"
                        let next_slot_id = SlotId(13);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(12),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier . Layout "=" Layout "/" Layout Rule_Plus_2 Layout "/"
            SlotId(13) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule : Identifier Layout . "=" Layout "/" Layout Rule_Plus_2 Layout "/"
                        let next_slot_id = SlotId(14);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(13),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier Layout . "=" Layout "/" Layout Rule_Plus_2 Layout "/"
            SlotId(14) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(5), i, j);
                        //Rule : Identifier Layout "=" . Layout "/" Layout Rule_Plus_2 Layout "/"
                        let next_slot_id = SlotId(15);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(14),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier Layout "=" . Layout "/" Layout Rule_Plus_2 Layout "/"
            SlotId(15) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule : Identifier Layout "=" Layout . "/" Layout Rule_Plus_2 Layout "/"
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
            //Rule : Identifier Layout "=" Layout . "/" Layout Rule_Plus_2 Layout "/"
            SlotId(16) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"/\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"/\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(7), i, j);
                        //Rule : Identifier Layout "=" Layout "/" . Layout Rule_Plus_2 Layout "/"
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
                            "\"/\"",
                            i,
                            SlotId(16),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier Layout "=" Layout "/" . Layout Rule_Plus_2 Layout "/"
            SlotId(17) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule : Identifier Layout "=" Layout "/" Layout . Rule_Plus_2 Layout "/"
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
            //Rule : Identifier Layout "=" Layout "/" Layout . Rule_Plus_2 Layout "/"
            SlotId(18) => {
                self.create(NonterminalId(15), result, gss_node_id, SlotId(19));
            }
            //Rule : Identifier Layout "=" Layout "/" Layout Rule_Plus_2 . Layout "/"
            SlotId(19) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule : Identifier Layout "=" Layout "/" Layout Rule_Plus_2 Layout . "/"
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
            //Rule : Identifier Layout "=" Layout "/" Layout Rule_Plus_2 Layout . "/"
            SlotId(20) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"/\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"/\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(7), i, j);
                        //Rule : Identifier Layout "=" Layout "/" Layout Rule_Plus_2 Layout "/".
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
                            "\"/\"",
                            i,
                            SlotId(20),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier Layout "=" Layout "/" Layout Rule_Plus_2 Layout "/".
            SlotId(21) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
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
            //PriorityLevel : . PriorityLevel_Star_2
            SlotId(22) => {
                self.create(NonterminalId(18), result, gss_node_id, SlotId(23));
            }
            //PriorityLevel : PriorityLevel_Star_2.
            SlotId(23) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let end_slot_id = SlotId(23);
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
            //Alternative : . Alternative_Star_3
            SlotId(24) => {
                self.create(NonterminalId(21), result, gss_node_id, SlotId(25));
            }
            //Alternative : Alternative_Star_3.
            SlotId(25) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let end_slot_id = SlotId(25);
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
            SlotId(26) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(27));
            }
            //Symbol : Symbol . Layout "*"
            SlotId(27) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : Symbol Layout . "*"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : Symbol Layout . "*"
            SlotId(28) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //Symbol : Symbol Layout "*".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"*\"",
                            i,
                            SlotId(28),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "*".
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
            //Symbol : . Symbol Layout "+"
            SlotId(30) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(31));
            }
            //Symbol : Symbol . Layout "+"
            SlotId(31) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : Symbol Layout . "+"
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
            //Symbol : Symbol Layout . "+"
            SlotId(32) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //Symbol : Symbol Layout "+".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"+\"",
                            i,
                            SlotId(32),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol Layout "+".
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
            //Symbol : . "(" Layout Symbol Layout "|" Layout Symbol Layout ")"
            SlotId(34) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Symbol : "(" . Layout Symbol Layout "|" Layout Symbol Layout ")"
                        let next_slot_id = SlotId(35);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(34),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Layout Symbol Layout "|" Layout Symbol Layout ")"
            SlotId(35) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "(" Layout . Symbol Layout "|" Layout Symbol Layout ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "(" Layout . Symbol Layout "|" Layout Symbol Layout ")"
            SlotId(36) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(37));
            }
            //Symbol : "(" Layout Symbol . Layout "|" Layout Symbol Layout ")"
            SlotId(37) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "(" Layout Symbol Layout . "|" Layout Symbol Layout ")"
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
            //Symbol : "(" Layout Symbol Layout . "|" Layout Symbol Layout ")"
            SlotId(38) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //Symbol : "(" Layout Symbol Layout "|" . Layout Symbol Layout ")"
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
                            "\"|\"",
                            i,
                            SlotId(38),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Symbol Layout "|" . Layout Symbol Layout ")"
            SlotId(39) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "(" Layout Symbol Layout "|" Layout . Symbol Layout ")"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "(" Layout Symbol Layout "|" Layout . Symbol Layout ")"
            SlotId(40) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(41));
            }
            //Symbol : "(" Layout Symbol Layout "|" Layout Symbol . Layout ")"
            SlotId(41) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "(" Layout Symbol Layout "|" Layout Symbol Layout . ")"
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
            //Symbol : "(" Layout Symbol Layout "|" Layout Symbol Layout . ")"
            SlotId(42) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol : "(" Layout Symbol Layout "|" Layout Symbol Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(42),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Symbol Layout "|" Layout Symbol Layout ")".
            SlotId(43) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //Symbol : . """ Layout String Layout """
            SlotId(44) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol : """ . Layout String Layout """
                        let next_slot_id = SlotId(45);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(44),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ . Layout String Layout """
            SlotId(45) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : """ Layout . String Layout """
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
            //Symbol : """ Layout . String Layout """
            SlotId(46) => {
                let i = input_index;
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Symbol : """ Layout String . Layout """
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
                            "String",
                            i,
                            SlotId(46),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ Layout String . Layout """
            SlotId(47) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : """ Layout String Layout . """
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
            //Symbol : """ Layout String Layout . """
            SlotId(48) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(13), i, j);
                        //Symbol : """ Layout String Layout """.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(48),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ Layout String Layout """.
            SlotId(49) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(49);
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
            SlotId(50) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "*"
                        let next_slot_id = SlotId(51);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(50),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "*"
            SlotId(51) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "*"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "*"
            SlotId(52) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(53));
            }
            //Symbol : "{" Layout Symbol . Layout Symbol Layout "}" Layout "*"
            SlotId(53) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "*"
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
            //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "*"
            SlotId(54) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(55));
            }
            //Symbol : "{" Layout Symbol Layout Symbol . Layout "}" Layout "*"
            SlotId(55) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "*"
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "*"
            SlotId(56) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "*"
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
                            "\"}\"",
                            i,
                            SlotId(56),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "*"
            SlotId(57) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "*"
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "*"
            SlotId(58) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "*".
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
                            "\"*\"",
                            i,
                            SlotId(58),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "*".
            SlotId(59) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(60) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(14), i, j);
                        //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "+"
                        let next_slot_id = SlotId(61);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(60),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" . Layout Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(61) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "+"
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
            //Symbol : "{" Layout . Symbol Layout Symbol Layout "}" Layout "+"
            SlotId(62) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(63));
            }
            //Symbol : "{" Layout Symbol . Layout Symbol Layout "}" Layout "+"
            SlotId(63) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "+"
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
            //Symbol : "{" Layout Symbol Layout . Symbol Layout "}" Layout "+"
            SlotId(64) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(65));
            }
            //Symbol : "{" Layout Symbol Layout Symbol . Layout "}" Layout "+"
            SlotId(65) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "+"
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout . "}" Layout "+"
            SlotId(66) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(15), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "+"
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
                            "\"}\"",
                            i,
                            SlotId(66),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" . Layout "+"
            SlotId(67) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "+"
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
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout . "+"
            SlotId(68) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "+".
                        let next_slot_id = SlotId(69);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(68),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Layout Symbol Layout Symbol Layout "}" Layout "+".
            SlotId(69) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(69);
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
            //Symbol : . "(" Layout Alternative_Star_3 Layout ")"
            SlotId(70) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Symbol : "(" . Layout Alternative_Star_3 Layout ")"
                        let next_slot_id = SlotId(71);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(70),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Layout Alternative_Star_3 Layout ")"
            SlotId(71) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "(" Layout . Alternative_Star_3 Layout ")"
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
            //Symbol : "(" Layout . Alternative_Star_3 Layout ")"
            SlotId(72) => {
                self.create(NonterminalId(21), result, gss_node_id, SlotId(73));
            }
            //Symbol : "(" Layout Alternative_Star_3 . Layout ")"
            SlotId(73) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Symbol : "(" Layout Alternative_Star_3 Layout . ")"
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
            //Symbol : "(" Layout Alternative_Star_3 Layout . ")"
            SlotId(74) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Symbol : "(" Layout Alternative_Star_3 Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(74),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Layout Alternative_Star_3 Layout ")".
            SlotId(75) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //Symbol : . Identifier
            SlotId(76) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Symbol : Identifier.
                        let next_slot_id = SlotId(77);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(76),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Identifier.
            SlotId(77) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //Regex : . Regex Layout "+"
            SlotId(78) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(79));
            }
            //Regex : Regex . Layout "+"
            SlotId(79) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Regex : Regex Layout . "+"
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
            //Regex : Regex Layout . "+"
            SlotId(80) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(10), i, j);
                        //Regex : Regex Layout "+".
                        let next_slot_id = SlotId(81);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(80),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "+".
            SlotId(81) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(81);
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
            SlotId(82) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(83));
            }
            //Regex : Regex . Layout "*"
            SlotId(83) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Regex : Regex Layout . "*"
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
            //Regex : Regex Layout . "*"
            SlotId(84) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(9), i, j);
                        //Regex : Regex Layout "*".
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
                            "\"*\"",
                            i,
                            SlotId(84),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "*".
            SlotId(85) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //Regex : . Regex Layout "?"
            SlotId(86) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(87));
            }
            //Regex : Regex . Layout "?"
            SlotId(87) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Regex : Regex Layout . "?"
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
            //Regex : Regex Layout . "?"
            SlotId(88) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(16), i, j);
                        //Regex : Regex Layout "?".
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
                            "\"?\"",
                            i,
                            SlotId(88),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex Layout "?".
            SlotId(89) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //Regex : . "(" Layout Regex_Star_4 Layout ")"
            SlotId(90) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(11), i, j);
                        //Regex : "(" . Layout Regex_Star_4 Layout ")"
                        let next_slot_id = SlotId(91);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(90),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Layout Regex_Star_4 Layout ")"
            SlotId(91) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Regex : "(" Layout . Regex_Star_4 Layout ")"
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
            //Regex : "(" Layout . Regex_Star_4 Layout ")"
            SlotId(92) => {
                self.create(NonterminalId(23), result, gss_node_id, SlotId(93));
            }
            //Regex : "(" Layout Regex_Star_4 . Layout ")"
            SlotId(93) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Regex : "(" Layout Regex_Star_4 Layout . ")"
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
            //Regex : "(" Layout Regex_Star_4 Layout . ")"
            SlotId(94) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(12), i, j);
                        //Regex : "(" Layout Regex_Star_4 Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(94),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Layout Regex_Star_4 Layout ")".
            SlotId(95) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(95);
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
            SlotId(96) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(97));
            }
            //Regex : CharClass.
            SlotId(97) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //Regex : . Char
            SlotId(98) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Regex : Char.
                        let next_slot_id = SlotId(99);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(98),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Char.
            SlotId(99) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //CharClass : . CharClass_Opt_5 Layout "[" Layout CharClass_Plus_6 Layout "]"
            SlotId(100) => {
                self.create(NonterminalId(24), result, gss_node_id, SlotId(101));
            }
            //CharClass : CharClass_Opt_5 . Layout "[" Layout CharClass_Plus_6 Layout "]"
            SlotId(101) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //CharClass : CharClass_Opt_5 Layout . "[" Layout CharClass_Plus_6 Layout "]"
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
            //CharClass : CharClass_Opt_5 Layout . "[" Layout CharClass_Plus_6 Layout "]"
            SlotId(102) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"[\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"[\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(18), i, j);
                        //CharClass : CharClass_Opt_5 Layout "[" . Layout CharClass_Plus_6 Layout "]"
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
                            "\"[\"",
                            i,
                            SlotId(102),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_5 Layout "[" . Layout CharClass_Plus_6 Layout "]"
            SlotId(103) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //CharClass : CharClass_Opt_5 Layout "[" Layout . CharClass_Plus_6 Layout "]"
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
            //CharClass : CharClass_Opt_5 Layout "[" Layout . CharClass_Plus_6 Layout "]"
            SlotId(104) => {
                self.create(NonterminalId(26), result, gss_node_id, SlotId(105));
            }
            //CharClass : CharClass_Opt_5 Layout "[" Layout CharClass_Plus_6 . Layout "]"
            SlotId(105) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //CharClass : CharClass_Opt_5 Layout "[" Layout CharClass_Plus_6 Layout . "]"
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
            //CharClass : CharClass_Opt_5 Layout "[" Layout CharClass_Plus_6 Layout . "]"
            SlotId(106) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"]\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"]\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(19), i, j);
                        //CharClass : CharClass_Opt_5 Layout "[" Layout CharClass_Plus_6 Layout "]".
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
                            "\"]\"",
                            i,
                            SlotId(106),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_5 Layout "[" Layout CharClass_Plus_6 Layout "]".
            SlotId(107) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
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
            //CharRange : . Char Layout "-" Layout Char
            SlotId(108) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //CharRange : Char . Layout "-" Layout Char
                        let next_slot_id = SlotId(109);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(108),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharRange : Char . Layout "-" Layout Char
            SlotId(109) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //CharRange : Char Layout . "-" Layout Char
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
            //CharRange : Char Layout . "-" Layout Char
            SlotId(110) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"-\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"-\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(20), i, j);
                        //CharRange : Char Layout "-" . Layout Char
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
                            "\"-\"",
                            i,
                            SlotId(110),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharRange : Char Layout "-" . Layout Char
            SlotId(111) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //CharRange : Char Layout "-" Layout . Char
                        let next_slot_id = SlotId(112);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(111),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharRange : Char Layout "-" Layout . Char
            SlotId(112) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //CharRange : Char Layout "-" Layout Char.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(112),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharRange : Char Layout "-" Layout Char.
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
            //Grammar_Plus_0 : . Grammar_Plus_0 Layout Rule
            SlotId(114) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(115));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 . Layout Rule
            SlotId(115) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Grammar_Plus_0 : Grammar_Plus_0 Layout . Rule
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
            //Grammar_Plus_0 : Grammar_Plus_0 Layout . Rule
            SlotId(116) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(117));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Layout Rule.
            SlotId(117) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(117);
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
            //Grammar_Plus_0 : . Rule
            SlotId(118) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(119));
            }
            //Grammar_Plus_0 : Rule.
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
            //Grammar_Opt_0 : . Grammar_Plus_0
            SlotId(120) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(121));
            }
            //Grammar_Opt_0 : Grammar_Plus_0.
            SlotId(121) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(121);
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
            SlotId(122) => {
                let end_slot_id = SlotId(122);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(22), input_index, input_index);
                let nonterminal_id = NonterminalId(9);
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
            SlotId(123) => {
                self.create(NonterminalId(9), result, gss_node_id, SlotId(124));
            }
            //Grammar_Star_0 : Grammar_Opt_0.
            SlotId(124) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(124);
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
            //Rule_Plus_1 : . Rule_Plus_1 Layout ">" Layout PriorityLevel
            SlotId(125) => {
                self.create(NonterminalId(11), result, gss_node_id, SlotId(126));
            }
            //Rule_Plus_1 : Rule_Plus_1 . Layout ">" Layout PriorityLevel
            SlotId(126) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule_Plus_1 : Rule_Plus_1 Layout . ">" Layout PriorityLevel
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
                            "Layout",
                            i,
                            SlotId(126),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule_Plus_1 : Rule_Plus_1 Layout . ">" Layout PriorityLevel
            SlotId(127) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(6), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(6), i, j);
                        //Rule_Plus_1 : Rule_Plus_1 Layout ">" . Layout PriorityLevel
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\">\"",
                            i,
                            SlotId(127),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule_Plus_1 : Rule_Plus_1 Layout ">" . Layout PriorityLevel
            SlotId(128) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule_Plus_1 : Rule_Plus_1 Layout ">" Layout . PriorityLevel
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Rule_Plus_1 : Rule_Plus_1 Layout ">" Layout . PriorityLevel
            SlotId(129) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(130));
            }
            //Rule_Plus_1 : Rule_Plus_1 Layout ">" Layout PriorityLevel.
            SlotId(130) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(130);
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
            //Rule_Plus_1 : . PriorityLevel
            SlotId(131) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(132));
            }
            //Rule_Plus_1 : PriorityLevel.
            SlotId(132) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(132);
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
            //Rule_Opt_1 : . Rule_Plus_1
            SlotId(133) => {
                self.create(NonterminalId(11), result, gss_node_id, SlotId(134));
            }
            //Rule_Opt_1 : Rule_Plus_1.
            SlotId(134) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
                let end_slot_id = SlotId(134);
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
            //Rule_Opt_1 : .
            SlotId(135) => {
                let end_slot_id = SlotId(135);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(22), input_index, input_index);
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
            //Rule_Star_1 : . Rule_Opt_1
            SlotId(136) => {
                self.create(NonterminalId(12), result, gss_node_id, SlotId(137));
            }
            //Rule_Star_1 : Rule_Opt_1.
            SlotId(137) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
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
            //Rule_Plus_3 : . Rule_Plus_3 Layout Regex
            SlotId(138) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(139));
            }
            //Rule_Plus_3 : Rule_Plus_3 . Layout Regex
            SlotId(139) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule_Plus_3 : Rule_Plus_3 Layout . Regex
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
            //Rule_Plus_3 : Rule_Plus_3 Layout . Regex
            SlotId(140) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(141));
            }
            //Rule_Plus_3 : Rule_Plus_3 Layout Regex.
            SlotId(141) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
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
            //Rule_Plus_3 : . Regex
            SlotId(142) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(143));
            }
            //Rule_Plus_3 : Regex.
            SlotId(143) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
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
            //Rule_Plus_2 : . Rule_Plus_2 Layout "|" Layout Rule_Plus_3
            SlotId(144) => {
                self.create(NonterminalId(15), result, gss_node_id, SlotId(145));
            }
            //Rule_Plus_2 : Rule_Plus_2 . Layout "|" Layout Rule_Plus_3
            SlotId(145) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule_Plus_2 : Rule_Plus_2 Layout . "|" Layout Rule_Plus_3
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(145),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule_Plus_2 : Rule_Plus_2 Layout . "|" Layout Rule_Plus_3
            SlotId(146) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //Rule_Plus_2 : Rule_Plus_2 Layout "|" . Layout Rule_Plus_3
                        let next_slot_id = SlotId(147);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(146),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule_Plus_2 : Rule_Plus_2 Layout "|" . Layout Rule_Plus_3
            SlotId(147) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Rule_Plus_2 : Rule_Plus_2 Layout "|" Layout . Rule_Plus_3
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
            //Rule_Plus_2 : Rule_Plus_2 Layout "|" Layout . Rule_Plus_3
            SlotId(148) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(149));
            }
            //Rule_Plus_2 : Rule_Plus_2 Layout "|" Layout Rule_Plus_3.
            SlotId(149) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
                let end_slot_id = SlotId(149);
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
            //Rule_Plus_2 : . Rule_Plus_3
            SlotId(150) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(151));
            }
            //Rule_Plus_2 : Rule_Plus_3.
            SlotId(151) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
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
            //PriorityLevel_Plus_4 : . PriorityLevel_Plus_4 Layout "|" Layout Alternative
            SlotId(152) => {
                self.create(NonterminalId(16), result, gss_node_id, SlotId(153));
            }
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 . Layout "|" Layout Alternative
            SlotId(153) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 Layout . "|" Layout Alternative
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 Layout . "|" Layout Alternative
            SlotId(154) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(8), i, j);
                        //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 Layout "|" . Layout Alternative
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(154),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 Layout "|" . Layout Alternative
            SlotId(155) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 Layout "|" Layout . Alternative
                        let next_slot_id = SlotId(156);
                        let left_child_id = result.expect("Result should not be None.");
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
                            SlotId(155),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 Layout "|" Layout . Alternative
            SlotId(156) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(157));
            }
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 Layout "|" Layout Alternative.
            SlotId(157) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
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
            //PriorityLevel_Plus_4 : . Alternative
            SlotId(158) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(159));
            }
            //PriorityLevel_Plus_4 : Alternative.
            SlotId(159) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
                let end_slot_id = SlotId(159);
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
            //PriorityLevel_Opt_2 : . PriorityLevel_Plus_4
            SlotId(160) => {
                self.create(NonterminalId(16), result, gss_node_id, SlotId(161));
            }
            //PriorityLevel_Opt_2 : PriorityLevel_Plus_4.
            SlotId(161) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
                let end_slot_id = SlotId(161);
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
            //PriorityLevel_Opt_2 : .
            SlotId(162) => {
                let end_slot_id = SlotId(162);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(22), input_index, input_index);
                let nonterminal_id = NonterminalId(17);
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
            //PriorityLevel_Star_2 : . PriorityLevel_Opt_2
            SlotId(163) => {
                self.create(NonterminalId(17), result, gss_node_id, SlotId(164));
            }
            //PriorityLevel_Star_2 : PriorityLevel_Opt_2.
            SlotId(164) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
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
            //Alternative_Plus_5 : . Alternative_Plus_5 Layout Symbol
            SlotId(165) => {
                self.create(NonterminalId(19), result, gss_node_id, SlotId(166));
            }
            //Alternative_Plus_5 : Alternative_Plus_5 . Layout Symbol
            SlotId(166) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //Alternative_Plus_5 : Alternative_Plus_5 Layout . Symbol
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(166),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Alternative_Plus_5 : Alternative_Plus_5 Layout . Symbol
            SlotId(167) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(168));
            }
            //Alternative_Plus_5 : Alternative_Plus_5 Layout Symbol.
            SlotId(168) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
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
            //Alternative_Plus_5 : . Symbol
            SlotId(169) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(170));
            }
            //Alternative_Plus_5 : Symbol.
            SlotId(170) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(170);
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
            //Alternative_Opt_3 : . Alternative_Plus_5
            SlotId(171) => {
                self.create(NonterminalId(19), result, gss_node_id, SlotId(172));
            }
            //Alternative_Opt_3 : Alternative_Plus_5.
            SlotId(172) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
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
            //Alternative_Opt_3 : .
            SlotId(173) => {
                let end_slot_id = SlotId(173);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(22), input_index, input_index);
                let nonterminal_id = NonterminalId(20);
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
            //Alternative_Star_3 : . Alternative_Opt_3
            SlotId(174) => {
                self.create(NonterminalId(20), result, gss_node_id, SlotId(175));
            }
            //Alternative_Star_3 : Alternative_Opt_3.
            SlotId(175) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
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
            //Regex_Opt_4 : . Rule_Plus_2
            SlotId(176) => {
                self.create(NonterminalId(15), result, gss_node_id, SlotId(177));
            }
            //Regex_Opt_4 : Rule_Plus_2.
            SlotId(177) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
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
            //Regex_Opt_4 : .
            SlotId(178) => {
                let end_slot_id = SlotId(178);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(22), input_index, input_index);
                let nonterminal_id = NonterminalId(22);
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
            //Regex_Star_4 : . Regex_Opt_4
            SlotId(179) => {
                self.create(NonterminalId(22), result, gss_node_id, SlotId(180));
            }
            //Regex_Star_4 : Regex_Opt_4.
            SlotId(180) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(180);
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
            //CharClass_Opt_5 : . "!"
            SlotId(181) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(17), i, j);
                        //CharClass_Opt_5 : "!".
                        let next_slot_id = SlotId(182);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(181),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Opt_5 : "!".
            SlotId(182) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
                let end_slot_id = SlotId(182);
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
            //CharClass_Opt_5 : .
            SlotId(183) => {
                let end_slot_id = SlotId(183);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(22), input_index, input_index);
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
            //CharClass_Alt_0 : . CharRange
            SlotId(184) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(185));
            }
            //CharClass_Alt_0 : CharRange.
            SlotId(185) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
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
            //CharClass_Alt_0 : . Char
            SlotId(186) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //CharClass_Alt_0 : Char.
                        let next_slot_id = SlotId(187);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(186),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Alt_0 : Char.
            SlotId(187) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
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
            //CharClass_Plus_6 : . CharClass_Plus_6 Layout CharClass_Alt_0
            SlotId(188) => {
                self.create(NonterminalId(26), result, gss_node_id, SlotId(189));
            }
            //CharClass_Plus_6 : CharClass_Plus_6 . Layout CharClass_Alt_0
            SlotId(189) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //CharClass_Plus_6 : CharClass_Plus_6 Layout . CharClass_Alt_0
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
            //CharClass_Plus_6 : CharClass_Plus_6 Layout . CharClass_Alt_0
            SlotId(190) => {
                self.create(NonterminalId(25), result, gss_node_id, SlotId(191));
            }
            //CharClass_Plus_6 : CharClass_Plus_6 Layout CharClass_Alt_0.
            SlotId(191) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
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
            //CharClass_Plus_6 : . CharClass_Alt_0
            SlotId(192) => {
                self.create(NonterminalId(25), result, gss_node_id, SlotId(193));
            }
            //CharClass_Plus_6 : CharClass_Alt_0.
            SlotId(193) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
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
            //StartGrammar : . Layout Grammar Layout
            SlotId(194) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartGrammar : Layout . Grammar Layout
                        let next_slot_id = SlotId(195);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //StartGrammar : Layout . Grammar Layout
            SlotId(195) => {
                self.create(NonterminalId(0), result, gss_node_id, SlotId(196));
            }
            //StartGrammar : Layout Grammar . Layout
            SlotId(196) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartGrammar : Layout Grammar Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(196),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartGrammar : Layout Grammar Layout.
            SlotId(197) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(27);
                let end_slot_id = SlotId(197);
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
            //StartRule : . Layout Rule Layout
            SlotId(198) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartRule : Layout . Rule Layout
                        let next_slot_id = SlotId(199);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(198),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRule : Layout . Rule Layout
            SlotId(199) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(200));
            }
            //StartRule : Layout Rule . Layout
            SlotId(200) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartRule : Layout Rule Layout.
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
            //StartRule : Layout Rule Layout.
            SlotId(201) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(28);
                let end_slot_id = SlotId(201);
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
            SlotId(202) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartPriorityLevel : Layout . PriorityLevel Layout
                        let next_slot_id = SlotId(203);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(202),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout . PriorityLevel Layout
            SlotId(203) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(204));
            }
            //StartPriorityLevel : Layout PriorityLevel . Layout
            SlotId(204) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartPriorityLevel : Layout PriorityLevel Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(204),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartPriorityLevel : Layout PriorityLevel Layout.
            SlotId(205) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(29);
                let end_slot_id = SlotId(205);
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
            SlotId(206) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartAlternative : Layout . Alternative Layout
                        let next_slot_id = SlotId(207);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(206),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout . Alternative Layout
            SlotId(207) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(208));
            }
            //StartAlternative : Layout Alternative . Layout
            SlotId(208) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartAlternative : Layout Alternative Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(208),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartAlternative : Layout Alternative Layout.
            SlotId(209) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(30);
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
            //StartSymbol : . Layout Symbol Layout
            SlotId(210) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartSymbol : Layout . Symbol Layout
                        let next_slot_id = SlotId(211);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //StartSymbol : Layout . Symbol Layout
            SlotId(211) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(212));
            }
            //StartSymbol : Layout Symbol . Layout
            SlotId(212) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartSymbol : Layout Symbol Layout.
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
                            "Layout",
                            i,
                            SlotId(212),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartSymbol : Layout Symbol Layout.
            SlotId(213) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(31);
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
            //StartRegex : . Layout Regex Layout
            SlotId(214) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartRegex : Layout . Regex Layout
                        let next_slot_id = SlotId(215);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(214),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout . Regex Layout
            SlotId(215) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(216));
            }
            //StartRegex : Layout Regex . Layout
            SlotId(216) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartRegex : Layout Regex Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(216),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartRegex : Layout Regex Layout.
            SlotId(217) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(32);
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
            //StartCharClass : . Layout CharClass Layout
            SlotId(218) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartCharClass : Layout . CharClass Layout
                        let next_slot_id = SlotId(219);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(218),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout . CharClass Layout
            SlotId(219) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(220));
            }
            //StartCharClass : Layout CharClass . Layout
            SlotId(220) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartCharClass : Layout CharClass Layout.
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
                            "Layout",
                            i,
                            SlotId(220),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharClass : Layout CharClass Layout.
            SlotId(221) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(33);
                let end_slot_id = SlotId(221);
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
            //StartCharRange : . Layout CharRange Layout
            SlotId(222) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartCharRange : Layout . CharRange Layout
                        let next_slot_id = SlotId(223);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(222),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartCharRange : Layout . CharRange Layout
            SlotId(223) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(224));
            }
            //StartCharRange : Layout CharRange . Layout
            SlotId(224) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(21), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(21), i, j);
                        //StartCharRange : Layout CharRange Layout.
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //StartCharRange : Layout CharRange Layout.
            SlotId(225) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(34);
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
                //Grammar : . "grammar" Layout Identifier Layout Grammar_Star_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule
            NonterminalId(1) => {
                //Rule : . Identifier Layout "=" Layout Rule_Star_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(6),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule : . Identifier Layout "=" Layout "/" Layout Rule_Plus_2 Layout "/"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(12),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel
            NonterminalId(2) => {
                //PriorityLevel : . PriorityLevel_Star_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(22),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative
            NonterminalId(3) => {
                //Alternative : . Alternative_Star_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(24),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol
            NonterminalId(4) => {
                //Symbol : . Symbol Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(26),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Symbol Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(30),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Layout Symbol Layout "|" Layout Symbol Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(34),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . """ Layout String Layout """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(44),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(50),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Layout Symbol Layout Symbol Layout "}" Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(60),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Layout Alternative_Star_3 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(70),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(76),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex
            NonterminalId(5) => {
                //Regex : . Regex Layout "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(78),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex Layout "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(82),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex Layout "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(86),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . "(" Layout Regex_Star_4 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(90),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . CharClass
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(96),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Char
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(98),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass
            NonterminalId(6) => {
                //CharClass : . CharClass_Opt_5 Layout "[" Layout CharClass_Plus_6 Layout "]"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(100),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharRange
            NonterminalId(7) => {
                //CharRange : . Char Layout "-" Layout Char
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(108),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Plus_0
            NonterminalId(8) => {
                //Grammar_Plus_0 : . Grammar_Plus_0 Layout Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(114),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Plus_0 : . Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(118),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Opt_0
            NonterminalId(9) => {
                //Grammar_Opt_0 : . Grammar_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(120),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(122),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Star_0
            NonterminalId(10) => {
                //Grammar_Star_0 : . Grammar_Opt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(123),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Plus_1
            NonterminalId(11) => {
                //Rule_Plus_1 : . Rule_Plus_1 Layout ">" Layout PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(125),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Plus_1 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(131),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Opt_1
            NonterminalId(12) => {
                //Rule_Opt_1 : . Rule_Plus_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(133),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(135),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Star_1
            NonterminalId(13) => {
                //Rule_Star_1 : . Rule_Opt_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(136),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Plus_3
            NonterminalId(14) => {
                //Rule_Plus_3 : . Rule_Plus_3 Layout Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(138),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Plus_3 : . Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(142),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Plus_2
            NonterminalId(15) => {
                //Rule_Plus_2 : . Rule_Plus_2 Layout "|" Layout Rule_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(144),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Plus_2 : . Rule_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(150),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Plus_4
            NonterminalId(16) => {
                //PriorityLevel_Plus_4 : . PriorityLevel_Plus_4 Layout "|" Layout Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(152),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Plus_4 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(158),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Opt_2
            NonterminalId(17) => {
                //PriorityLevel_Opt_2 : . PriorityLevel_Plus_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(160),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Opt_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(162),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Star_2
            NonterminalId(18) => {
                //PriorityLevel_Star_2 : . PriorityLevel_Opt_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(163),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Plus_5
            NonterminalId(19) => {
                //Alternative_Plus_5 : . Alternative_Plus_5 Layout Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(165),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Plus_5 : . Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(169),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Opt_3
            NonterminalId(20) => {
                //Alternative_Opt_3 : . Alternative_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(171),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Opt_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(173),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Star_3
            NonterminalId(21) => {
                //Alternative_Star_3 : . Alternative_Opt_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(174),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Opt_4
            NonterminalId(22) => {
                //Regex_Opt_4 : . Rule_Plus_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(176),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex_Opt_4 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(178),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Star_4
            NonterminalId(23) => {
                //Regex_Star_4 : . Regex_Opt_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(179),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Opt_5
            NonterminalId(24) => {
                //CharClass_Opt_5 : . "!"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(181),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Opt_5 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(183),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Alt_0
            NonterminalId(25) => {
                //CharClass_Alt_0 : . CharRange
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(184),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Alt_0 : . Char
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(186),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Plus_6
            NonterminalId(26) => {
                //CharClass_Plus_6 : . CharClass_Plus_6 Layout CharClass_Alt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(188),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Plus_6 : . CharClass_Alt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(192),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartGrammar
            NonterminalId(27) => {
                //StartGrammar : . Layout Grammar Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(194),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRule
            NonterminalId(28) => {
                //StartRule : . Layout Rule Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(198),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartPriorityLevel
            NonterminalId(29) => {
                //StartPriorityLevel : . Layout PriorityLevel Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(202),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartAlternative
            NonterminalId(30) => {
                //StartAlternative : . Layout Alternative Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(206),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartSymbol
            NonterminalId(31) => {
                //StartSymbol : . Layout Symbol Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(210),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartRegex
            NonterminalId(32) => {
                //StartRegex : . Layout Regex Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(214),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartCharClass
            NonterminalId(33) => {
                //StartCharClass : . Layout CharClass Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(218),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //StartCharRange
            NonterminalId(34) => {
                //StartCharRange : . Layout CharRange Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(222),
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
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 35],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 35],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 226],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 23],
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
            gss_nodes_index: [const { vec![] }; 35],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 35],
            intermediate_nodes_index: [const { InlineMap::Empty }; 226],
            terminal_nodes_index: [const { InlineMap::Empty }; 23],
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

