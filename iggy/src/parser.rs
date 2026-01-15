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
pub const NONTERMINALS: [Nonterminal; 27] = [
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
    "CharClass_Alt_0" => NonterminalId(25), "CharClass_Plus_6" => NonterminalId(26)
};
pub const TERMINALS: [Terminal; 22] = [
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
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 148] = [
    Slot {
        display_name: "Grammar : . \"grammar\" Identifier Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" . Identifier Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Identifier . Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" Identifier Rule*.",
    },
    Slot {
        display_name: "Rule : . Identifier \"=\" {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier . \"=\" {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier \"=\" . {PriorityLevel \">\"}*",
    },
    Slot {
        display_name: "Rule : Identifier \"=\" {PriorityLevel \">\"}*.",
    },
    Slot {
        display_name: "Rule : . Identifier \"=\" \"/\" {Regex+ \"|\"}+ \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier . \"=\" \"/\" {Regex+ \"|\"}+ \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier \"=\" . \"/\" {Regex+ \"|\"}+ \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier \"=\" \"/\" . {Regex+ \"|\"}+ \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier \"=\" \"/\" {Regex+ \"|\"}+ . \"/\"",
    },
    Slot {
        display_name: "Rule : Identifier \"=\" \"/\" {Regex+ \"|\"}+ \"/\".",
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
        display_name: "Symbol : . Symbol \"*\"",
    },
    Slot {
        display_name: "Symbol : Symbol . \"*\"",
    },
    Slot {
        display_name: "Symbol : Symbol \"*\".",
    },
    Slot {
        display_name: "Symbol : . Symbol \"+\"",
    },
    Slot {
        display_name: "Symbol : Symbol . \"+\"",
    },
    Slot {
        display_name: "Symbol : Symbol \"+\".",
    },
    Slot {
        display_name: "Symbol : . \"(\" Symbol \"|\" Symbol \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" . Symbol \"|\" Symbol \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Symbol . \"|\" Symbol \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Symbol \"|\" . Symbol \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Symbol \"|\" Symbol . \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Symbol \"|\" Symbol \")\".",
    },
    Slot {
        display_name: "Symbol : . \"\"\" String \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" . String \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" String . \"\"\"",
    },
    Slot {
        display_name: "Symbol : \"\"\" String \"\"\".",
    },
    Slot {
        display_name: "Symbol : . \"{\" Symbol Symbol \"}\" \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" . Symbol Symbol \"}\" \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol . Symbol \"}\" \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol Symbol . \"}\" \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol Symbol \"}\" . \"*\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol Symbol \"}\" \"*\".",
    },
    Slot {
        display_name: "Symbol : . \"{\" Symbol Symbol \"}\" \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" . Symbol Symbol \"}\" \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol . Symbol \"}\" \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol Symbol . \"}\" \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol Symbol \"}\" . \"+\"",
    },
    Slot {
        display_name: "Symbol : \"{\" Symbol Symbol \"}\" \"+\".",
    },
    Slot {
        display_name: "Symbol : . \"(\" Symbol* \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" . Symbol* \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Symbol* . \")\"",
    },
    Slot {
        display_name: "Symbol : \"(\" Symbol* \")\".",
    },
    Slot {
        display_name: "Symbol : . Identifier",
    },
    Slot {
        display_name: "Symbol : Identifier.",
    },
    Slot {
        display_name: "Regex : . Regex \"+\"",
    },
    Slot {
        display_name: "Regex : Regex . \"+\"",
    },
    Slot {
        display_name: "Regex : Regex \"+\".",
    },
    Slot {
        display_name: "Regex : . Regex \"*\"",
    },
    Slot {
        display_name: "Regex : Regex . \"*\"",
    },
    Slot {
        display_name: "Regex : Regex \"*\".",
    },
    Slot {
        display_name: "Regex : . Regex \"?\"",
    },
    Slot {
        display_name: "Regex : Regex . \"?\"",
    },
    Slot {
        display_name: "Regex : Regex \"?\".",
    },
    Slot {
        display_name: "Regex : . \"(\" {Regex+ \"|\"}* \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" . {Regex+ \"|\"}* \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" {Regex+ \"|\"}* . \")\"",
    },
    Slot {
        display_name: "Regex : \"(\" {Regex+ \"|\"}* \")\".",
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
        display_name: "CharClass : . \"!\"? \"[\" (CharRange | Char)+ \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? . \"[\" (CharRange | Char)+ \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? \"[\" . (CharRange | Char)+ \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? \"[\" (CharRange | Char)+ . \"]\"",
    },
    Slot {
        display_name: "CharClass : \"!\"? \"[\" (CharRange | Char)+ \"]\".",
    },
    Slot {
        display_name: "CharRange : . Char \"-\" Char",
    },
    Slot {
        display_name: "CharRange : Char . \"-\" Char",
    },
    Slot {
        display_name: "CharRange : Char \"-\" . Char",
    },
    Slot {
        display_name: "CharRange : Char \"-\" Char.",
    },
    Slot {
        display_name: "Rule+ : . Rule+ Rule",
    },
    Slot {
        display_name: "Rule+ : Rule+ . Rule",
    },
    Slot {
        display_name: "Rule+ : Rule+ Rule.",
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
        display_name: "{PriorityLevel \">\"}+ : . {PriorityLevel \">\"}+ \">\" PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ . \">\" PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ \">\" . PriorityLevel",
    },
    Slot {
        display_name: "{PriorityLevel \">\"}+ : {PriorityLevel \">\"}+ \">\" PriorityLevel.",
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
        display_name: "Regex+ : . Regex+ Regex",
    },
    Slot {
        display_name: "Regex+ : Regex+ . Regex",
    },
    Slot {
        display_name: "Regex+ : Regex+ Regex.",
    },
    Slot {
        display_name: "Regex+ : . Regex",
    },
    Slot {
        display_name: "Regex+ : Regex.",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : . {Regex+ \"|\"}+ \"|\" Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ . \"|\" Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ \"|\" . Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : {Regex+ \"|\"}+ \"|\" Regex+.",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : . Regex+",
    },
    Slot {
        display_name: "{Regex+ \"|\"}+ : Regex+.",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : . {Alternative \"|\"}+ \"|\" Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ . \"|\" Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ \"|\" . Alternative",
    },
    Slot {
        display_name: "{Alternative \"|\"}+ : {Alternative \"|\"}+ \"|\" Alternative.",
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
        display_name: "Symbol+ : . Symbol+ Symbol",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ . Symbol",
    },
    Slot {
        display_name: "Symbol+ : Symbol+ Symbol.",
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
        display_name: "(CharRange | Char)+ : . (CharRange | Char)+ (CharRange | Char)",
    },
    Slot {
        display_name: "(CharRange | Char)+ : (CharRange | Char)+ . (CharRange | Char)",
    },
    Slot {
        display_name: "(CharRange | Char)+ : (CharRange | Char)+ (CharRange | Char).",
    },
    Slot {
        display_name: "(CharRange | Char)+ : . (CharRange | Char)",
    },
    Slot {
        display_name: "(CharRange | Char)+ : (CharRange | Char).",
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
            //Grammar : . "grammar" Identifier Grammar_Star_0
            SlotId(0) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(4),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Grammar : "grammar" . Identifier Grammar_Star_0
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
            //Grammar : "grammar" . Identifier Grammar_Star_0
            SlotId(1) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(0),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Grammar : "grammar" Identifier . Grammar_Star_0
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
                            "Identifier",
                            i,
                            SlotId(1),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" Identifier . Grammar_Star_0
            SlotId(2) => {
                self.create(NonterminalId(10), result, gss_node_id, SlotId(3));
            }
            //Grammar : "grammar" Identifier Grammar_Star_0.
            SlotId(3) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
                let end_slot_id = SlotId(3);
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
            //Rule : . Identifier "=" Rule_Star_1
            SlotId(4) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(0),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule : Identifier . "=" Rule_Star_1
                        let next_slot_id = SlotId(5);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Identifier",
                            i,
                            SlotId(4),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier . "=" Rule_Star_1
            SlotId(5) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(5),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule : Identifier "=" . Rule_Star_1
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
                            "\"=\"",
                            i,
                            SlotId(5),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier "=" . Rule_Star_1
            SlotId(6) => {
                self.create(NonterminalId(13), result, gss_node_id, SlotId(7));
            }
            //Rule : Identifier "=" Rule_Star_1.
            SlotId(7) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
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
            //Rule : . Identifier "=" "/" Rule_Plus_2 "/"
            SlotId(8) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(0),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule : Identifier . "=" "/" Rule_Plus_2 "/"
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
            //Rule : Identifier . "=" "/" Rule_Plus_2 "/"
            SlotId(9) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"=\"", i);
                match self.scanner.match_token(TerminalId(5), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"=\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(5),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule : Identifier "=" . "/" Rule_Plus_2 "/"
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
                            "\"=\"",
                            i,
                            SlotId(9),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier "=" . "/" Rule_Plus_2 "/"
            SlotId(10) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"/\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"/\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(7),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule : Identifier "=" "/" . Rule_Plus_2 "/"
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
                            "\"/\"",
                            i,
                            SlotId(10),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier "=" "/" . Rule_Plus_2 "/"
            SlotId(11) => {
                self.create(NonterminalId(15), result, gss_node_id, SlotId(12));
            }
            //Rule : Identifier "=" "/" Rule_Plus_2 . "/"
            SlotId(12) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"/\"", i);
                match self.scanner.match_token(TerminalId(7), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"/\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(7),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule : Identifier "=" "/" Rule_Plus_2 "/".
                        let next_slot_id = SlotId(13);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(12),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : Identifier "=" "/" Rule_Plus_2 "/".
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
            //PriorityLevel : . PriorityLevel_Star_2
            SlotId(14) => {
                self.create(NonterminalId(18), result, gss_node_id, SlotId(15));
            }
            //PriorityLevel : PriorityLevel_Star_2.
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
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative : . Alternative_Star_3
            SlotId(16) => {
                self.create(NonterminalId(21), result, gss_node_id, SlotId(17));
            }
            //Alternative : Alternative_Star_3.
            SlotId(17) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let end_slot_id = SlotId(17);
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
            //Symbol : . Symbol "*"
            SlotId(18) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(19));
            }
            //Symbol : Symbol . "*"
            SlotId(19) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(9),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : Symbol "*".
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
                            "\"*\"",
                            i,
                            SlotId(19),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol "*".
            SlotId(20) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(20);
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
            //Symbol : . Symbol "+"
            SlotId(21) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(22));
            }
            //Symbol : Symbol . "+"
            SlotId(22) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(10),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : Symbol "+".
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
                            "\"+\"",
                            i,
                            SlotId(22),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : Symbol "+".
            SlotId(23) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //Symbol : . "(" Symbol "|" Symbol ")"
            SlotId(24) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(11),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "(" . Symbol "|" Symbol ")"
                        let next_slot_id = SlotId(25);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(24),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Symbol "|" Symbol ")"
            SlotId(25) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(26));
            }
            //Symbol : "(" Symbol . "|" Symbol ")"
            SlotId(26) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(8),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "(" Symbol "|" . Symbol ")"
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
                            "\"|\"",
                            i,
                            SlotId(26),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Symbol "|" . Symbol ")"
            SlotId(27) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(28));
            }
            //Symbol : "(" Symbol "|" Symbol . ")"
            SlotId(28) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(12),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "(" Symbol "|" Symbol ")".
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
                            "\")\"",
                            i,
                            SlotId(28),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Symbol "|" Symbol ")".
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
            //Symbol : . """ String """
            SlotId(30) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(13),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : """ . String """
                        let next_slot_id = SlotId(31);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"\"\"",
                            i,
                            SlotId(30),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ . String """
            SlotId(31) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "String", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "String", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(1),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : """ String . """
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
                            "String",
                            i,
                            SlotId(31),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ String . """
            SlotId(32) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"\"\"", i);
                match self.scanner.match_token(TerminalId(13), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"\"\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(13),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : """ String """.
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
                            "\"\"\"",
                            i,
                            SlotId(32),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : """ String """.
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
            //Symbol : . "{" Symbol Symbol "}" "*"
            SlotId(34) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(14),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "{" . Symbol Symbol "}" "*"
                        let next_slot_id = SlotId(35);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : "{" . Symbol Symbol "}" "*"
            SlotId(35) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(36));
            }
            //Symbol : "{" Symbol . Symbol "}" "*"
            SlotId(36) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(37));
            }
            //Symbol : "{" Symbol Symbol . "}" "*"
            SlotId(37) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(15),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "{" Symbol Symbol "}" . "*"
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
                            "\"}\"",
                            i,
                            SlotId(37),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Symbol Symbol "}" . "*"
            SlotId(38) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(9),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "{" Symbol Symbol "}" "*".
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
                            "\"*\"",
                            i,
                            SlotId(38),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Symbol Symbol "}" "*".
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
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Symbol : . "{" Symbol Symbol "}" "+"
            SlotId(40) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"{\"", i);
                match self.scanner.match_token(TerminalId(14), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"{\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(14),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "{" . Symbol Symbol "}" "+"
                        let next_slot_id = SlotId(41);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"{\"",
                            i,
                            SlotId(40),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" . Symbol Symbol "}" "+"
            SlotId(41) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(42));
            }
            //Symbol : "{" Symbol . Symbol "}" "+"
            SlotId(42) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(43));
            }
            //Symbol : "{" Symbol Symbol . "}" "+"
            SlotId(43) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"}\"", i);
                match self.scanner.match_token(TerminalId(15), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"}\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(15),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "{" Symbol Symbol "}" . "+"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"}\"",
                            i,
                            SlotId(43),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Symbol Symbol "}" . "+"
            SlotId(44) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(10),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "{" Symbol Symbol "}" "+".
                        let next_slot_id = SlotId(45);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(44),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "{" Symbol Symbol "}" "+".
            SlotId(45) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(45);
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
            //Symbol : . "(" Alternative_Star_3 ")"
            SlotId(46) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(11),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "(" . Alternative_Star_3 ")"
                        let next_slot_id = SlotId(47);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(46),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" . Alternative_Star_3 ")"
            SlotId(47) => {
                self.create(NonterminalId(21), result, gss_node_id, SlotId(48));
            }
            //Symbol : "(" Alternative_Star_3 . ")"
            SlotId(48) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(12),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : "(" Alternative_Star_3 ")".
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
                            "\")\"",
                            i,
                            SlotId(48),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "(" Alternative_Star_3 ")".
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
            //Symbol : . Identifier
            SlotId(50) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Identifier", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Identifier", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(0),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Symbol : Identifier.
                        let next_slot_id = SlotId(51);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
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
            //Symbol : Identifier.
            SlotId(51) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //Regex : . Regex "+"
            SlotId(52) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(53));
            }
            //Regex : Regex . "+"
            SlotId(53) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"+\"", i);
                match self.scanner.match_token(TerminalId(10), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"+\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(10),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Regex : Regex "+".
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
                            "\"+\"",
                            i,
                            SlotId(53),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex "+".
            SlotId(54) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(54);
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
            //Regex : . Regex "*"
            SlotId(55) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(56));
            }
            //Regex : Regex . "*"
            SlotId(56) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"*\"", i);
                match self.scanner.match_token(TerminalId(9), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"*\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(9),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Regex : Regex "*".
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
                            "\"*\"",
                            i,
                            SlotId(56),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex "*".
            SlotId(57) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //Regex : . Regex "?"
            SlotId(58) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(59));
            }
            //Regex : Regex . "?"
            SlotId(59) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"?\"", i);
                match self.scanner.match_token(TerminalId(16), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"?\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(16),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Regex : Regex "?".
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
                            "\"?\"",
                            i,
                            SlotId(59),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Regex "?".
            SlotId(60) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(60);
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
            //Regex : . "(" Regex_Star_4 ")"
            SlotId(61) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(11), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(11),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Regex : "(" . Regex_Star_4 ")"
                        let next_slot_id = SlotId(62);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"(\"",
                            i,
                            SlotId(61),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" . Regex_Star_4 ")"
            SlotId(62) => {
                self.create(NonterminalId(23), result, gss_node_id, SlotId(63));
            }
            //Regex : "(" Regex_Star_4 . ")"
            SlotId(63) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(12), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(12),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Regex : "(" Regex_Star_4 ")".
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
                            "\")\"",
                            i,
                            SlotId(63),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : "(" Regex_Star_4 ")".
            SlotId(64) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(64);
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
            SlotId(65) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(66));
            }
            //Regex : CharClass.
            SlotId(66) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(66);
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
            SlotId(67) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(2),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Regex : Char.
                        let next_slot_id = SlotId(68);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(67),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Regex : Char.
            SlotId(68) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(68);
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
            //CharClass : . CharClass_Opt_5 "[" CharClass_Plus_6 "]"
            SlotId(69) => {
                self.create(NonterminalId(24), result, gss_node_id, SlotId(70));
            }
            //CharClass : CharClass_Opt_5 . "[" CharClass_Plus_6 "]"
            SlotId(70) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"[\"", i);
                match self.scanner.match_token(TerminalId(18), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"[\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(18),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //CharClass : CharClass_Opt_5 "[" . CharClass_Plus_6 "]"
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"[\"",
                            i,
                            SlotId(70),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_5 "[" . CharClass_Plus_6 "]"
            SlotId(71) => {
                self.create(NonterminalId(26), result, gss_node_id, SlotId(72));
            }
            //CharClass : CharClass_Opt_5 "[" CharClass_Plus_6 . "]"
            SlotId(72) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"]\"", i);
                match self.scanner.match_token(TerminalId(19), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"]\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(19),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //CharClass : CharClass_Opt_5 "[" CharClass_Plus_6 "]".
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
                            "\"]\"",
                            i,
                            SlotId(72),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass : CharClass_Opt_5 "[" CharClass_Plus_6 "]".
            SlotId(73) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
                let end_slot_id = SlotId(73);
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
            //CharRange : . Char "-" Char
            SlotId(74) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(2),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //CharRange : Char . "-" Char
                        let next_slot_id = SlotId(75);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(74),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharRange : Char . "-" Char
            SlotId(75) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"-\"", i);
                match self.scanner.match_token(TerminalId(20), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"-\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(20),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //CharRange : Char "-" . Char
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
                            "\"-\"",
                            i,
                            SlotId(75),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharRange : Char "-" . Char
            SlotId(76) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(2),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //CharRange : Char "-" Char.
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
                            "Char",
                            i,
                            SlotId(76),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharRange : Char "-" Char.
            SlotId(77) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
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
            //Grammar_Plus_0 : . Grammar_Plus_0 Rule
            SlotId(78) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(79));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 . Rule
            SlotId(79) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(80));
            }
            //Grammar_Plus_0 : Grammar_Plus_0 Rule.
            SlotId(80) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(80);
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
            SlotId(81) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(82));
            }
            //Grammar_Plus_0 : Rule.
            SlotId(82) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(82);
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
            SlotId(83) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(84));
            }
            //Grammar_Opt_0 : Grammar_Plus_0.
            SlotId(84) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
                let end_slot_id = SlotId(84);
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
            SlotId(85) => {
                let end_slot_id = SlotId(85);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(21),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
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
            SlotId(86) => {
                self.create(NonterminalId(9), result, gss_node_id, SlotId(87));
            }
            //Grammar_Star_0 : Grammar_Opt_0.
            SlotId(87) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(87);
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
            //Rule_Plus_1 : . Rule_Plus_1 ">" PriorityLevel
            SlotId(88) => {
                self.create(NonterminalId(11), result, gss_node_id, SlotId(89));
            }
            //Rule_Plus_1 : Rule_Plus_1 . ">" PriorityLevel
            SlotId(89) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(6), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(6),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule_Plus_1 : Rule_Plus_1 ">" . PriorityLevel
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
                            "\">\"",
                            i,
                            SlotId(89),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule_Plus_1 : Rule_Plus_1 ">" . PriorityLevel
            SlotId(90) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(91));
            }
            //Rule_Plus_1 : Rule_Plus_1 ">" PriorityLevel.
            SlotId(91) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
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
            //Rule_Plus_1 : . PriorityLevel
            SlotId(92) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(93));
            }
            //Rule_Plus_1 : PriorityLevel.
            SlotId(93) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
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
            //Rule_Opt_1 : . Rule_Plus_1
            SlotId(94) => {
                self.create(NonterminalId(11), result, gss_node_id, SlotId(95));
            }
            //Rule_Opt_1 : Rule_Plus_1.
            SlotId(95) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
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
            //Rule_Opt_1 : .
            SlotId(96) => {
                let end_slot_id = SlotId(96);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(21),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
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
            SlotId(97) => {
                self.create(NonterminalId(12), result, gss_node_id, SlotId(98));
            }
            //Rule_Star_1 : Rule_Opt_1.
            SlotId(98) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(13);
                let end_slot_id = SlotId(98);
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
            //Rule_Plus_3 : . Rule_Plus_3 Regex
            SlotId(99) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(100));
            }
            //Rule_Plus_3 : Rule_Plus_3 . Regex
            SlotId(100) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(101));
            }
            //Rule_Plus_3 : Rule_Plus_3 Regex.
            SlotId(101) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
                let end_slot_id = SlotId(101);
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
            SlotId(102) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(103));
            }
            //Rule_Plus_3 : Regex.
            SlotId(103) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(14);
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
            //Rule_Plus_2 : . Rule_Plus_2 "|" Rule_Plus_3
            SlotId(104) => {
                self.create(NonterminalId(15), result, gss_node_id, SlotId(105));
            }
            //Rule_Plus_2 : Rule_Plus_2 . "|" Rule_Plus_3
            SlotId(105) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(8),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule_Plus_2 : Rule_Plus_2 "|" . Rule_Plus_3
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
                            "\"|\"",
                            i,
                            SlotId(105),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule_Plus_2 : Rule_Plus_2 "|" . Rule_Plus_3
            SlotId(106) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(107));
            }
            //Rule_Plus_2 : Rule_Plus_2 "|" Rule_Plus_3.
            SlotId(107) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
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
            //Rule_Plus_2 : . Rule_Plus_3
            SlotId(108) => {
                self.create(NonterminalId(14), result, gss_node_id, SlotId(109));
            }
            //Rule_Plus_2 : Rule_Plus_3.
            SlotId(109) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(15);
                let end_slot_id = SlotId(109);
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
            //PriorityLevel_Plus_4 : . PriorityLevel_Plus_4 "|" Alternative
            SlotId(110) => {
                self.create(NonterminalId(16), result, gss_node_id, SlotId(111));
            }
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 . "|" Alternative
            SlotId(111) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(8), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(8),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 "|" . Alternative
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
                            "\"|\"",
                            i,
                            SlotId(111),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 "|" . Alternative
            SlotId(112) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(113));
            }
            //PriorityLevel_Plus_4 : PriorityLevel_Plus_4 "|" Alternative.
            SlotId(113) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
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
            //PriorityLevel_Plus_4 : . Alternative
            SlotId(114) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(115));
            }
            //PriorityLevel_Plus_4 : Alternative.
            SlotId(115) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(16);
                let end_slot_id = SlotId(115);
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
            SlotId(116) => {
                self.create(NonterminalId(16), result, gss_node_id, SlotId(117));
            }
            //PriorityLevel_Opt_2 : PriorityLevel_Plus_4.
            SlotId(117) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(17);
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
            //PriorityLevel_Opt_2 : .
            SlotId(118) => {
                let end_slot_id = SlotId(118);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(21),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
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
            SlotId(119) => {
                self.create(NonterminalId(17), result, gss_node_id, SlotId(120));
            }
            //PriorityLevel_Star_2 : PriorityLevel_Opt_2.
            SlotId(120) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(18);
                let end_slot_id = SlotId(120);
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
            //Alternative_Plus_5 : . Alternative_Plus_5 Symbol
            SlotId(121) => {
                self.create(NonterminalId(19), result, gss_node_id, SlotId(122));
            }
            //Alternative_Plus_5 : Alternative_Plus_5 . Symbol
            SlotId(122) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(123));
            }
            //Alternative_Plus_5 : Alternative_Plus_5 Symbol.
            SlotId(123) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
                let end_slot_id = SlotId(123);
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
            SlotId(124) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(125));
            }
            //Alternative_Plus_5 : Symbol.
            SlotId(125) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(19);
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
            //Alternative_Opt_3 : . Alternative_Plus_5
            SlotId(126) => {
                self.create(NonterminalId(19), result, gss_node_id, SlotId(127));
            }
            //Alternative_Opt_3 : Alternative_Plus_5.
            SlotId(127) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(20);
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
            //Alternative_Opt_3 : .
            SlotId(128) => {
                let end_slot_id = SlotId(128);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(21),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
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
            SlotId(129) => {
                self.create(NonterminalId(20), result, gss_node_id, SlotId(130));
            }
            //Alternative_Star_3 : Alternative_Opt_3.
            SlotId(130) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(21);
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
            //Regex_Opt_4 : . Rule_Plus_2
            SlotId(131) => {
                self.create(NonterminalId(15), result, gss_node_id, SlotId(132));
            }
            //Regex_Opt_4 : Rule_Plus_2.
            SlotId(132) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(22);
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
            //Regex_Opt_4 : .
            SlotId(133) => {
                let end_slot_id = SlotId(133);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(21),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
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
            SlotId(134) => {
                self.create(NonterminalId(22), result, gss_node_id, SlotId(135));
            }
            //Regex_Star_4 : Regex_Opt_4.
            SlotId(135) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(23);
                let end_slot_id = SlotId(135);
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
            SlotId(136) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"!\"", i);
                match self.scanner.match_token(TerminalId(17), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"!\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(17),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //CharClass_Opt_5 : "!".
                        let next_slot_id = SlotId(137);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"!\"",
                            i,
                            SlotId(136),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Opt_5 : "!".
            SlotId(137) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(24);
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
            //CharClass_Opt_5 : .
            SlotId(138) => {
                let end_slot_id = SlotId(138);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(21),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
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
            SlotId(139) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(140));
            }
            //CharClass_Alt_0 : CharRange.
            SlotId(140) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(140);
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
            SlotId(141) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "Char", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Char", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(2),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //CharClass_Alt_0 : Char.
                        let next_slot_id = SlotId(142);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Char",
                            i,
                            SlotId(141),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //CharClass_Alt_0 : Char.
            SlotId(142) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(25);
                let end_slot_id = SlotId(142);
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
            //CharClass_Plus_6 : . CharClass_Plus_6 CharClass_Alt_0
            SlotId(143) => {
                self.create(NonterminalId(26), result, gss_node_id, SlotId(144));
            }
            //CharClass_Plus_6 : CharClass_Plus_6 . CharClass_Alt_0
            SlotId(144) => {
                self.create(NonterminalId(25), result, gss_node_id, SlotId(145));
            }
            //CharClass_Plus_6 : CharClass_Plus_6 CharClass_Alt_0.
            SlotId(145) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
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
            //CharClass_Plus_6 : . CharClass_Alt_0
            SlotId(146) => {
                self.create(NonterminalId(25), result, gss_node_id, SlotId(147));
            }
            //CharClass_Plus_6 : CharClass_Alt_0.
            SlotId(147) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(26);
                let end_slot_id = SlotId(147);
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
                //Grammar : . "grammar" Identifier Grammar_Star_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule
            NonterminalId(1) => {
                //Rule : . Identifier "=" Rule_Star_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(4),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule : . Identifier "=" "/" Rule_Plus_2 "/"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(8),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel
            NonterminalId(2) => {
                //PriorityLevel : . PriorityLevel_Star_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative
            NonterminalId(3) => {
                //Alternative : . Alternative_Star_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(16),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol
            NonterminalId(4) => {
                //Symbol : . Symbol "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(18),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Symbol "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(21),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Symbol "|" Symbol ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(24),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . """ String """
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(30),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Symbol Symbol "}" "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(34),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "{" Symbol Symbol "}" "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(40),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . "(" Alternative_Star_3 ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(46),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Symbol : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(50),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex
            NonterminalId(5) => {
                //Regex : . Regex "+"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(52),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex "*"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(55),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Regex "?"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(58),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . "(" Regex_Star_4 ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(61),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . CharClass
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(65),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex : . Char
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(67),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass
            NonterminalId(6) => {
                //CharClass : . CharClass_Opt_5 "[" CharClass_Plus_6 "]"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(69),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharRange
            NonterminalId(7) => {
                //CharRange : . Char "-" Char
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(74),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Plus_0
            NonterminalId(8) => {
                //Grammar_Plus_0 : . Grammar_Plus_0 Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(78),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Plus_0 : . Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(81),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Opt_0
            NonterminalId(9) => {
                //Grammar_Opt_0 : . Grammar_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(83),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(85),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Star_0
            NonterminalId(10) => {
                //Grammar_Star_0 : . Grammar_Opt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(86),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Plus_1
            NonterminalId(11) => {
                //Rule_Plus_1 : . Rule_Plus_1 ">" PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(88),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Plus_1 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(92),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Opt_1
            NonterminalId(12) => {
                //Rule_Opt_1 : . Rule_Plus_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(94),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(96),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Star_1
            NonterminalId(13) => {
                //Rule_Star_1 : . Rule_Opt_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(97),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Plus_3
            NonterminalId(14) => {
                //Rule_Plus_3 : . Rule_Plus_3 Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(99),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Plus_3 : . Regex
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(102),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Plus_2
            NonterminalId(15) => {
                //Rule_Plus_2 : . Rule_Plus_2 "|" Rule_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(104),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Plus_2 : . Rule_Plus_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(108),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Plus_4
            NonterminalId(16) => {
                //PriorityLevel_Plus_4 : . PriorityLevel_Plus_4 "|" Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(110),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Plus_4 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(114),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Opt_2
            NonterminalId(17) => {
                //PriorityLevel_Opt_2 : . PriorityLevel_Plus_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(116),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Opt_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(118),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Star_2
            NonterminalId(18) => {
                //PriorityLevel_Star_2 : . PriorityLevel_Opt_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(119),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Plus_5
            NonterminalId(19) => {
                //Alternative_Plus_5 : . Alternative_Plus_5 Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(121),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Plus_5 : . Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(124),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Opt_3
            NonterminalId(20) => {
                //Alternative_Opt_3 : . Alternative_Plus_5
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(126),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Opt_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(128),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Star_3
            NonterminalId(21) => {
                //Alternative_Star_3 : . Alternative_Opt_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(129),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Opt_4
            NonterminalId(22) => {
                //Regex_Opt_4 : . Rule_Plus_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(131),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Regex_Opt_4 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(133),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Regex_Star_4
            NonterminalId(23) => {
                //Regex_Star_4 : . Regex_Opt_4
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(134),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Opt_5
            NonterminalId(24) => {
                //CharClass_Opt_5 : . "!"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(136),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Opt_5 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(138),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Alt_0
            NonterminalId(25) => {
                //CharClass_Alt_0 : . CharRange
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(139),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Alt_0 : . Char
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(141),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //CharClass_Plus_6
            NonterminalId(26) => {
                //CharClass_Plus_6 : . CharClass_Plus_6 CharClass_Alt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(143),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //CharClass_Plus_6 : . CharClass_Alt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(146),
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
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 27],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 27],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 148],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 22],
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
            gss_nodes_index: [const { vec![] }; 27],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 27],
            intermediate_nodes_index: [const { InlineMap::Empty }; 148],
            terminal_nodes_index: [const { InlineMap::Empty }; 22],
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

