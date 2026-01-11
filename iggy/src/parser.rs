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
pub const NONTERMINALS: [Nonterminal; 13] = [
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
        name: "Grammar_Star_0",
        display: "Rule*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Rule_Opt_0",
        display: "PriorityLevel?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Rule_Group_0",
        display: "(\">\" PriorityLevel)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "Rule_Star_1",
        display: "(\">\" PriorityLevel)*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "PriorityLevel_Opt_1",
        display: "Alternative?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "PriorityLevel_Group_1",
        display: "(\"|\" Alternative)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "PriorityLevel_Star_2",
        display: "(\"|\" Alternative)*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Alternative_Star_3",
        display: "Symbol*",
        kind: Some(EbnfKind::Star),
    },
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Grammar" => NonterminalId(0), "Rule" => NonterminalId(1), "PriorityLevel" =>
    NonterminalId(2), "Alternative" => NonterminalId(3), "Symbol" => NonterminalId(4),
    "Grammar_Star_0" => NonterminalId(5), "Rule_Opt_0" => NonterminalId(6),
    "Rule_Group_0" => NonterminalId(7), "Rule_Star_1" => NonterminalId(8),
    "PriorityLevel_Opt_1" => NonterminalId(9), "PriorityLevel_Group_1" =>
    NonterminalId(10), "PriorityLevel_Star_2" => NonterminalId(11), "Alternative_Star_3"
    => NonterminalId(12)
};
pub const TERMINALS: [Terminal; 6] = [
    Terminal {
        name: "\"grammar\"",
    },
    Terminal { name: "\";\"" },
    Terminal { name: "\":\"" },
    Terminal { name: "\">\"" },
    Terminal { name: "\"|\"" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 46] = [
    Slot {
        display_name: "Grammar : . \"grammar\" \"grammar\" \";\" Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" . \"grammar\" \";\" Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" \"grammar\" . \";\" Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" \"grammar\" \";\" . Rule*",
    },
    Slot {
        display_name: "Grammar : \"grammar\" \"grammar\" \";\" Rule*.",
    },
    Slot {
        display_name: "Rule : . \"grammar\" \":\" PriorityLevel? (\">\" PriorityLevel)* \";\"",
    },
    Slot {
        display_name: "Rule : \"grammar\" . \":\" PriorityLevel? (\">\" PriorityLevel)* \";\"",
    },
    Slot {
        display_name: "Rule : \"grammar\" \":\" . PriorityLevel? (\">\" PriorityLevel)* \";\"",
    },
    Slot {
        display_name: "Rule : \"grammar\" \":\" PriorityLevel? . (\">\" PriorityLevel)* \";\"",
    },
    Slot {
        display_name: "Rule : \"grammar\" \":\" PriorityLevel? (\">\" PriorityLevel)* . \";\"",
    },
    Slot {
        display_name: "Rule : \"grammar\" \":\" PriorityLevel? (\">\" PriorityLevel)* \";\".",
    },
    Slot {
        display_name: "PriorityLevel : . Alternative? (\"|\" Alternative)*",
    },
    Slot {
        display_name: "PriorityLevel : Alternative? . (\"|\" Alternative)*",
    },
    Slot {
        display_name: "PriorityLevel : Alternative? (\"|\" Alternative)*.",
    },
    Slot {
        display_name: "Alternative : . Symbol*",
    },
    Slot {
        display_name: "Alternative : Symbol*.",
    },
    Slot {
        display_name: "Symbol : . \"grammar\"",
    },
    Slot {
        display_name: "Symbol : \"grammar\".",
    },
    Slot {
        display_name: "Rule* : . Rule* Rule",
    },
    Slot {
        display_name: "Rule* : Rule* . Rule",
    },
    Slot {
        display_name: "Rule* : Rule* Rule.",
    },
    Slot {
        display_name: "Rule* : .",
    },
    Slot {
        display_name: "PriorityLevel? : . PriorityLevel",
    },
    Slot {
        display_name: "PriorityLevel? : PriorityLevel.",
    },
    Slot {
        display_name: "PriorityLevel? : .",
    },
    Slot {
        display_name: "(\">\" PriorityLevel) : . \">\" PriorityLevel",
    },
    Slot {
        display_name: "(\">\" PriorityLevel) : \">\" . PriorityLevel",
    },
    Slot {
        display_name: "(\">\" PriorityLevel) : \">\" PriorityLevel.",
    },
    Slot {
        display_name: "(\">\" PriorityLevel)* : . (\">\" PriorityLevel)* (\">\" PriorityLevel)",
    },
    Slot {
        display_name: "(\">\" PriorityLevel)* : (\">\" PriorityLevel)* . (\">\" PriorityLevel)",
    },
    Slot {
        display_name: "(\">\" PriorityLevel)* : (\">\" PriorityLevel)* (\">\" PriorityLevel).",
    },
    Slot {
        display_name: "(\">\" PriorityLevel)* : .",
    },
    Slot {
        display_name: "Alternative? : . Alternative",
    },
    Slot {
        display_name: "Alternative? : Alternative.",
    },
    Slot {
        display_name: "Alternative? : .",
    },
    Slot {
        display_name: "(\"|\" Alternative) : . \"|\" Alternative",
    },
    Slot {
        display_name: "(\"|\" Alternative) : \"|\" . Alternative",
    },
    Slot {
        display_name: "(\"|\" Alternative) : \"|\" Alternative.",
    },
    Slot {
        display_name: "(\"|\" Alternative)* : . (\"|\" Alternative)* (\"|\" Alternative)",
    },
    Slot {
        display_name: "(\"|\" Alternative)* : (\"|\" Alternative)* . (\"|\" Alternative)",
    },
    Slot {
        display_name: "(\"|\" Alternative)* : (\"|\" Alternative)* (\"|\" Alternative).",
    },
    Slot {
        display_name: "(\"|\" Alternative)* : .",
    },
    Slot {
        display_name: "Symbol* : . Symbol* Symbol",
    },
    Slot {
        display_name: "Symbol* : Symbol* . Symbol",
    },
    Slot {
        display_name: "Symbol* : Symbol* Symbol.",
    },
    Slot {
        display_name: "Symbol* : .",
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
            //Grammar : . "grammar" "grammar" ";" Grammar_Star_0
            SlotId(0) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
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
                        //Grammar : "grammar" . "grammar" ";" Grammar_Star_0
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
            //Grammar : "grammar" . "grammar" ";" Grammar_Star_0
            SlotId(1) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
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
                        //Grammar : "grammar" "grammar" . ";" Grammar_Star_0
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
                            "\"grammar\"",
                            i,
                            SlotId(1),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" "grammar" . ";" Grammar_Star_0
            SlotId(2) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\";\"", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\";\"", i, j);
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
                        //Grammar : "grammar" "grammar" ";" . Grammar_Star_0
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
                            "\";\"",
                            i,
                            SlotId(2),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Grammar : "grammar" "grammar" ";" . Grammar_Star_0
            SlotId(3) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(4));
            }
            //Grammar : "grammar" "grammar" ";" Grammar_Star_0.
            SlotId(4) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
                let end_slot_id = SlotId(4);
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
            //Rule : . "grammar" ":" Rule_Opt_0 Rule_Star_1 ";"
            SlotId(5) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
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
                        //Rule : "grammar" . ":" Rule_Opt_0 Rule_Star_1 ";"
                        let next_slot_id = SlotId(6);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"grammar\"",
                            i,
                            SlotId(5),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : "grammar" . ":" Rule_Opt_0 Rule_Star_1 ";"
            SlotId(6) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\":\"", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\":\"", i, j);
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
                        //Rule : "grammar" ":" . Rule_Opt_0 Rule_Star_1 ";"
                        let next_slot_id = SlotId(7);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(6),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : "grammar" ":" . Rule_Opt_0 Rule_Star_1 ";"
            SlotId(7) => {
                self.create(NonterminalId(6), result, gss_node_id, SlotId(8));
            }
            //Rule : "grammar" ":" Rule_Opt_0 . Rule_Star_1 ";"
            SlotId(8) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(9));
            }
            //Rule : "grammar" ":" Rule_Opt_0 Rule_Star_1 . ";"
            SlotId(9) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\";\"", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\";\"", i, j);
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
                        //Rule : "grammar" ":" Rule_Opt_0 Rule_Star_1 ";".
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
                            "\";\"",
                            i,
                            SlotId(9),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule : "grammar" ":" Rule_Opt_0 Rule_Star_1 ";".
            SlotId(10) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
                let end_slot_id = SlotId(10);
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
            //PriorityLevel : . PriorityLevel_Opt_1 PriorityLevel_Star_2
            SlotId(11) => {
                self.create(NonterminalId(9), result, gss_node_id, SlotId(12));
            }
            //PriorityLevel : PriorityLevel_Opt_1 . PriorityLevel_Star_2
            SlotId(12) => {
                self.create(NonterminalId(11), result, gss_node_id, SlotId(13));
            }
            //PriorityLevel : PriorityLevel_Opt_1 PriorityLevel_Star_2.
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
                    self.pop(gss_node_id, end_slot_id, nonterminal_node_id);
                }
            }
            //Alternative : . Alternative_Star_3
            SlotId(14) => {
                self.create(NonterminalId(12), result, gss_node_id, SlotId(15));
            }
            //Alternative : Alternative_Star_3.
            SlotId(15) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
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
            //Symbol : . "grammar"
            SlotId(16) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"grammar\"", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"grammar\"", i, j);
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
                        //Symbol : "grammar".
                        let next_slot_id = SlotId(17);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"grammar\"",
                            i,
                            SlotId(16),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Symbol : "grammar".
            SlotId(17) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //Grammar_Star_0 : . Grammar_Star_0 Rule
            SlotId(18) => {
                self.create(NonterminalId(5), result, gss_node_id, SlotId(19));
            }
            //Grammar_Star_0 : Grammar_Star_0 . Rule
            SlotId(19) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(20));
            }
            //Grammar_Star_0 : Grammar_Star_0 Rule.
            SlotId(20) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //Grammar_Star_0 : .
            SlotId(21) => {
                let end_slot_id = SlotId(21);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(5),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
                let nonterminal_id = NonterminalId(5);
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
            //Rule_Opt_0 : . PriorityLevel
            SlotId(22) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(23));
            }
            //Rule_Opt_0 : PriorityLevel.
            SlotId(23) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(6);
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
            //Rule_Opt_0 : .
            SlotId(24) => {
                let end_slot_id = SlotId(24);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(5),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
                let nonterminal_id = NonterminalId(6);
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
            //Rule_Group_0 : . ">" PriorityLevel
            SlotId(25) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\">\"", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\">\"", i, j);
                        record!(self, MatchingTrailingLayout, i);
                        let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                        record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                        let right_child_id = self.get_or_create_terminal_node(
                            TerminalId(3),
                            i,
                            j,
                            leading_layout,
                            trailing_layout,
                        );
                        //Rule_Group_0 : ">" . PriorityLevel
                        let next_slot_id = SlotId(26);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\">\"",
                            i,
                            SlotId(25),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Rule_Group_0 : ">" . PriorityLevel
            SlotId(26) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(27));
            }
            //Rule_Group_0 : ">" PriorityLevel.
            SlotId(27) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(7);
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
            //Rule_Star_1 : . Rule_Star_1 Rule_Group_0
            SlotId(28) => {
                self.create(NonterminalId(8), result, gss_node_id, SlotId(29));
            }
            //Rule_Star_1 : Rule_Star_1 . Rule_Group_0
            SlotId(29) => {
                self.create(NonterminalId(7), result, gss_node_id, SlotId(30));
            }
            //Rule_Star_1 : Rule_Star_1 Rule_Group_0.
            SlotId(30) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
                let end_slot_id = SlotId(30);
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
            //Rule_Star_1 : .
            SlotId(31) => {
                let end_slot_id = SlotId(31);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(5),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
                let nonterminal_id = NonterminalId(8);
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
            //PriorityLevel_Opt_1 : . Alternative
            SlotId(32) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(33));
            }
            //PriorityLevel_Opt_1 : Alternative.
            SlotId(33) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
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
            //PriorityLevel_Opt_1 : .
            SlotId(34) => {
                let end_slot_id = SlotId(34);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(5),
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
            //PriorityLevel_Group_1 : . "|" Alternative
            SlotId(35) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "\"|\"", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"|\"", i, j);
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
                        //PriorityLevel_Group_1 : "|" . Alternative
                        let next_slot_id = SlotId(36);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"|\"",
                            i,
                            SlotId(35),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //PriorityLevel_Group_1 : "|" . Alternative
            SlotId(36) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(37));
            }
            //PriorityLevel_Group_1 : "|" Alternative.
            SlotId(37) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(10);
                let end_slot_id = SlotId(37);
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
            //PriorityLevel_Star_2 : . PriorityLevel_Star_2 PriorityLevel_Group_1
            SlotId(38) => {
                self.create(NonterminalId(11), result, gss_node_id, SlotId(39));
            }
            //PriorityLevel_Star_2 : PriorityLevel_Star_2 . PriorityLevel_Group_1
            SlotId(39) => {
                self.create(NonterminalId(10), result, gss_node_id, SlotId(40));
            }
            //PriorityLevel_Star_2 : PriorityLevel_Star_2 PriorityLevel_Group_1.
            SlotId(40) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(11);
                let end_slot_id = SlotId(40);
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
            //PriorityLevel_Star_2 : .
            SlotId(41) => {
                let end_slot_id = SlotId(41);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(5),
                    input_index,
                    input_index,
                    vec![],
                    vec![],
                );
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
            //Alternative_Star_3 : . Alternative_Star_3 Symbol
            SlotId(42) => {
                self.create(NonterminalId(12), result, gss_node_id, SlotId(43));
            }
            //Alternative_Star_3 : Alternative_Star_3 . Symbol
            SlotId(43) => {
                self.create(NonterminalId(4), result, gss_node_id, SlotId(44));
            }
            //Alternative_Star_3 : Alternative_Star_3 Symbol.
            SlotId(44) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(12);
                let end_slot_id = SlotId(44);
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
            //Alternative_Star_3 : .
            SlotId(45) => {
                let end_slot_id = SlotId(45);
                let epsilon_node_id = self.get_or_create_terminal_node(
                    TerminalId(5),
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
                //Grammar : . "grammar" "grammar" ";" Grammar_Star_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule
            NonterminalId(1) => {
                //Rule : . "grammar" ":" Rule_Opt_0 Rule_Star_1 ";"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(5),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel
            NonterminalId(2) => {
                //PriorityLevel : . PriorityLevel_Opt_1 PriorityLevel_Star_2
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(11),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative
            NonterminalId(3) => {
                //Alternative : . Alternative_Star_3
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Symbol
            NonterminalId(4) => {
                //Symbol : . "grammar"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(16),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Star_0
            NonterminalId(5) => {
                //Grammar_Star_0 : . Grammar_Star_0 Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(18),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Star_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(21),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Opt_0
            NonterminalId(6) => {
                //Rule_Opt_0 : . PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(22),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(24),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Group_0
            NonterminalId(7) => {
                //Rule_Group_0 : . ">" PriorityLevel
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(25),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Star_1
            NonterminalId(8) => {
                //Rule_Star_1 : . Rule_Star_1 Rule_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(28),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Star_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(31),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Opt_1
            NonterminalId(9) => {
                //PriorityLevel_Opt_1 : . Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(32),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Opt_1 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(34),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Group_1
            NonterminalId(10) => {
                //PriorityLevel_Group_1 : . "|" Alternative
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(35),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //PriorityLevel_Star_2
            NonterminalId(11) => {
                //PriorityLevel_Star_2 : . PriorityLevel_Star_2 PriorityLevel_Group_1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(38),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //PriorityLevel_Star_2 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(41),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Alternative_Star_3
            NonterminalId(12) => {
                //Alternative_Star_3 : . Alternative_Star_3 Symbol
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(42),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Alternative_Star_3 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(45),
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
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 13],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 13],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 46],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 6],
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
            gss_nodes_index: [const { vec![] }; 13],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 13],
            intermediate_nodes_index: [const { InlineMap::Empty }; 46],
            terminal_nodes_index: [const { InlineMap::Empty }; 6],
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

