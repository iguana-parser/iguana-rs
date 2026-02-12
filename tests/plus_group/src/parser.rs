use crate::{
    scanner::PlusGroupScanner,
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
pub const NONTERMINALS: [Nonterminal; 10] = [
    Nonterminal {
        name: "S",
        display: "S",
        kind: None,
    },
    Nonterminal {
        name: "A",
        display: "A",
        kind: None,
    },
    Nonterminal {
        name: "B",
        display: "B",
        kind: None,
    },
    Nonterminal {
        name: "C",
        display: "C",
        kind: None,
    },
    Nonterminal {
        name: "S_Group_0",
        display: "(A B C)",
        kind: Some(EbnfKind::Group),
    },
    Nonterminal {
        name: "S_Plus_0",
        display: "(A B C)+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "StartS",
        display: "StartS",
        kind: None,
    },
    Nonterminal {
        name: "StartA",
        display: "StartA",
        kind: None,
    },
    Nonterminal {
        name: "StartB",
        display: "StartB",
        kind: None,
    },
    Nonterminal {
        name: "StartC",
        display: "StartC",
        kind: None,
    },
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "S" => NonterminalId(0), "A" => NonterminalId(1), "B" => NonterminalId(2), "C" =>
    NonterminalId(3), "S_Group_0" => NonterminalId(4), "S_Plus_0" => NonterminalId(5),
    "StartS" => NonterminalId(6), "StartA" => NonterminalId(7), "StartB" =>
    NonterminalId(8), "StartC" => NonterminalId(9)
};
pub const TERMINALS: [Terminal; 5] = [
    Terminal { name: "\"a\"" },
    Terminal { name: "\"b\"" },
    Terminal { name: "\"c\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 36] = [
    Slot {
        display_name: "S : . (A B C)+",
    },
    Slot {
        display_name: "S : (A B C)+.",
    },
    Slot {
        display_name: "A : . \"a\"",
    },
    Slot {
        display_name: "A : \"a\".",
    },
    Slot {
        display_name: "B : . \"b\"",
    },
    Slot {
        display_name: "B : \"b\".",
    },
    Slot {
        display_name: "C : . \"c\"",
    },
    Slot {
        display_name: "C : \"c\".",
    },
    Slot {
        display_name: "(A B C) : . A Layout B Layout C",
    },
    Slot {
        display_name: "(A B C) : A . Layout B Layout C",
    },
    Slot {
        display_name: "(A B C) : A Layout . B Layout C",
    },
    Slot {
        display_name: "(A B C) : A Layout B . Layout C",
    },
    Slot {
        display_name: "(A B C) : A Layout B Layout . C",
    },
    Slot {
        display_name: "(A B C) : A Layout B Layout C.",
    },
    Slot {
        display_name: "(A B C)+ : . (A B C)+ Layout (A B C)",
    },
    Slot {
        display_name: "(A B C)+ : (A B C)+ . Layout (A B C)",
    },
    Slot {
        display_name: "(A B C)+ : (A B C)+ Layout . (A B C)",
    },
    Slot {
        display_name: "(A B C)+ : (A B C)+ Layout (A B C).",
    },
    Slot {
        display_name: "(A B C)+ : . (A B C)",
    },
    Slot {
        display_name: "(A B C)+ : (A B C).",
    },
    Slot {
        display_name: "StartS : . Layout start:S Layout",
    },
    Slot {
        display_name: "StartS : Layout . start:S Layout",
    },
    Slot {
        display_name: "StartS : Layout start:S . Layout",
    },
    Slot {
        display_name: "StartS : Layout start:S Layout.",
    },
    Slot {
        display_name: "StartA : . Layout start:A Layout",
    },
    Slot {
        display_name: "StartA : Layout . start:A Layout",
    },
    Slot {
        display_name: "StartA : Layout start:A . Layout",
    },
    Slot {
        display_name: "StartA : Layout start:A Layout.",
    },
    Slot {
        display_name: "StartB : . Layout start:B Layout",
    },
    Slot {
        display_name: "StartB : Layout . start:B Layout",
    },
    Slot {
        display_name: "StartB : Layout start:B . Layout",
    },
    Slot {
        display_name: "StartB : Layout start:B Layout.",
    },
    Slot {
        display_name: "StartC : . Layout start:C Layout",
    },
    Slot {
        display_name: "StartC : Layout . start:C Layout",
    },
    Slot {
        display_name: "StartC : Layout start:C . Layout",
    },
    Slot {
        display_name: "StartC : Layout start:C Layout.",
    },
];
impl<'i> Parser<'i> for PlusGroupParser<'i> {
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
            //S : . S_Plus_0
            SlotId(0) => {
                self.create_s_plus_0(result, gss_node_id, SlotId(1));
            }
            //S : S_Plus_0.
            SlotId(1) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
                let end_slot_id = SlotId(1);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //A : . "a"
            SlotId(2) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"a\"", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"a\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //A : "a".
                        let next_slot_id = SlotId(3);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"a\"",
                            i,
                            SlotId(2),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //A : "a".
            SlotId(3) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
                let end_slot_id = SlotId(3);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //B : . "b"
            SlotId(4) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"b\"", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"b\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //B : "b".
                        let next_slot_id = SlotId(5);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"b\"",
                            i,
                            SlotId(4),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //B : "b".
            SlotId(5) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let end_slot_id = SlotId(5);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //C : . "c"
            SlotId(6) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"c\"", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"c\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //C : "c".
                        let next_slot_id = SlotId(7);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\"c\"",
                            i,
                            SlotId(6),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //C : "c".
            SlotId(7) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
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
            //S_Group_0 : . A Layout B Layout C
            SlotId(8) => {
                self.create_a(result, gss_node_id, SlotId(9));
            }
            //S_Group_0 : A . Layout B Layout C
            SlotId(9) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //S_Group_0 : A Layout . B Layout C
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
            //S_Group_0 : A Layout . B Layout C
            SlotId(10) => {
                self.create_b(result, gss_node_id, SlotId(11));
            }
            //S_Group_0 : A Layout B . Layout C
            SlotId(11) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //S_Group_0 : A Layout B Layout . C
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
            //S_Group_0 : A Layout B Layout . C
            SlotId(12) => {
                self.create_c(result, gss_node_id, SlotId(13));
            }
            //S_Group_0 : A Layout B Layout C.
            SlotId(13) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
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
            //S_Plus_0 : . S_Plus_0 Layout S_Group_0
            SlotId(14) => {
                self.create_s_plus_0(result, gss_node_id, SlotId(15));
            }
            //S_Plus_0 : S_Plus_0 . Layout S_Group_0
            SlotId(15) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //S_Plus_0 : S_Plus_0 Layout . S_Group_0
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
            //S_Plus_0 : S_Plus_0 Layout . S_Group_0
            SlotId(16) => {
                self.create_s_group_0(result, gss_node_id, SlotId(17));
            }
            //S_Plus_0 : S_Plus_0 Layout S_Group_0.
            SlotId(17) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(17);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //S_Plus_0 : . S_Group_0
            SlotId(18) => {
                self.create_s_group_0(result, gss_node_id, SlotId(19));
            }
            //S_Plus_0 : S_Group_0.
            SlotId(19) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
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
            //StartS : . Layout start:S Layout
            SlotId(20) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartS : Layout . start:S Layout
                        let next_slot_id = SlotId(21);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(20),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartS : Layout . start:S Layout
            SlotId(21) => {
                self.create_s(result, gss_node_id, SlotId(22));
            }
            //StartS : Layout start:S . Layout
            SlotId(22) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartS : Layout start:S Layout.
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
                            "Layout",
                            i,
                            SlotId(22),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartS : Layout start:S Layout.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartA : . Layout start:A Layout
            SlotId(24) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartA : Layout . start:A Layout
                        let next_slot_id = SlotId(25);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(24),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartA : Layout . start:A Layout
            SlotId(25) => {
                self.create_a(result, gss_node_id, SlotId(26));
            }
            //StartA : Layout start:A . Layout
            SlotId(26) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartA : Layout start:A Layout.
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
                            "Layout",
                            i,
                            SlotId(26),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartA : Layout start:A Layout.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartB : . Layout start:B Layout
            SlotId(28) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartB : Layout . start:B Layout
                        let next_slot_id = SlotId(29);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(28),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartB : Layout . start:B Layout
            SlotId(29) => {
                self.create_b(result, gss_node_id, SlotId(30));
            }
            //StartB : Layout start:B . Layout
            SlotId(30) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartB : Layout start:B Layout.
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
                            "Layout",
                            i,
                            SlotId(30),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartB : Layout start:B Layout.
            SlotId(31) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(8);
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
            //StartC : . Layout start:C Layout
            SlotId(32) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartC : Layout . start:C Layout
                        let next_slot_id = SlotId(33);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "Layout",
                            i,
                            SlotId(32),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartC : Layout . start:C Layout
            SlotId(33) => {
                self.create_c(result, gss_node_id, SlotId(34));
            }
            //StartC : Layout start:C . Layout
            SlotId(34) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //StartC : Layout start:C Layout.
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
                            "Layout",
                            i,
                            SlotId(34),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //StartC : Layout start:C Layout.
            SlotId(35) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(9);
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
            //S
            NonterminalId(0) => {
                //S : . S_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //A
            NonterminalId(1) => {
                //A : . "a"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(2),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //B
            NonterminalId(2) => {
                //B : . "b"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(4),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //C
            NonterminalId(3) => {
                //C : . "c"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(6),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //S_Group_0
            NonterminalId(4) => {
                //S_Group_0 : . A Layout B Layout C
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(8),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //S_Plus_0
            NonterminalId(5) => {
                //S_Plus_0 : . S_Plus_0 Layout S_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(14),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //S_Plus_0 : . S_Group_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(18),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartS
            NonterminalId(6) => {
                //StartS : . Layout start:S Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(20),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartA
            NonterminalId(7) => {
                //StartA : . Layout start:A Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(24),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartB
            NonterminalId(8) => {
                //StartB : . Layout start:B Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(28),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartC
            NonterminalId(9) => {
                //StartC : . Layout start:C Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(32),
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
}
pub struct PlusGroupParser<'i> {
    start_nonterminal: NonterminalId,
    scanner: PlusGroupScanner<'i>,
    descriptors: Vec<Descriptor>,
    gss_nodes: Vec<GSSNode>,
    //A vector from nonterminal_ids to a tuple (input_index, gss_node_id)
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 10],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 10],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 36],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 5],
    intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
    intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
    nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
    nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
    envs: Vec<Env>,
    #[cfg(feature = "debug-trace")]
    pub trace_events: Option<Vec<TraceEvent>>,
}
impl<'i> PlusGroupParser<'i> {
    pub fn new(input: &'i Input, start_nonterminal: NonterminalId) -> Self {
        init_logger();
        Self {
            start_nonterminal,
            scanner: PlusGroupScanner::new(input),
            gss_nodes_index: [const { vec![] }; 10],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 10],
            intermediate_nodes_index: [const { InlineMap::Empty }; 36],
            terminal_nodes_index: [const { InlineMap::Empty }; 5],
            stats: Stats::default(),
            intermediate_nodes_children: vec![],
            intermediate_nodes_children_map: OnceCell::new(),
            nonterminal_nodes_children: vec![],
            nonterminal_nodes_children_map: OnceCell::new(),
            envs: vec![],
            #[cfg(feature = "debug-trace")]
            trace_events: None,
        }
    }
    fn create_s(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(0), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_a(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(1), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_b(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(2), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_c(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(3), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_s_group_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(4), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_s_plus_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(5), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_s(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(6), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_a(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(7), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_b(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(8), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_c(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(9), sppf_node_id, gss_node_id, return_slot);
    }
}

