// grammar ExcludeByLabel
//
// Expr
//   = Id #id
//   | Expr Layout "(" Layout Expr_Star_0 Layout ")" #call
//   | Expr Layout "," Layout Expr #comma
//
// Expr_Plus_0
//   = Expr_Plus_0 Layout "," Layout Expr_except_comma
//   | Expr_except_comma
//
// Expr_Opt_0
//   = Expr_Plus_0
//   |
//
// Expr_Star_0
//   = Expr_Opt_0
//
// Expr_except_comma
//   = Id #id
//   | Expr Layout "(" Layout Expr_Star_0 Layout ")" #call
//
// StartExpr
//   = Layout start:Expr Layout
//
// Id = ([a-z]+)
// "(" = (
// "," = ,
// ")" = )
// Layout = ε
use crate::{
    scanner::ExcludeByLabelScanner,
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
pub const NONTERMINALS: [Nonterminal; 6] = [
    Nonterminal {
        name: "Expr",
        display: "Expr",
        kind: None,
    },
    Nonterminal {
        name: "Expr_Plus_0",
        display: "{Expr !comma \",\"}+",
        kind: Some(EbnfKind::Plus),
    },
    Nonterminal {
        name: "Expr_Opt_0",
        display: "{Expr !comma \",\"}+?",
        kind: Some(EbnfKind::Opt),
    },
    Nonterminal {
        name: "Expr_Star_0",
        display: "{Expr !comma \",\"}*",
        kind: Some(EbnfKind::Star),
    },
    Nonterminal {
        name: "Expr_except_comma",
        display: "Expr !comma",
        kind: None,
    },
    Nonterminal {
        name: "StartExpr",
        display: "StartExpr",
        kind: None,
    },
];
static NONTERMINAL_IDS: phf::Map<&'static str, NonterminalId> = phf_map! {
    "Expr" => NonterminalId(0), "Expr_Plus_0" => NonterminalId(1), "Expr_Opt_0" =>
    NonterminalId(2), "Expr_Star_0" => NonterminalId(3), "Expr_except_comma" =>
    NonterminalId(4), "StartExpr" => NonterminalId(5)
};
pub const TERMINALS: [Terminal; 6] = [
    Terminal { name: "Id" },
    Terminal { name: "\"(\"" },
    Terminal { name: "\",\"" },
    Terminal { name: "\")\"" },
    Terminal { name: "Layout" },
    Terminal { name: "Epsilon" },
];
pub const SLOTS: [Slot; 43] = [
    Slot {
        display_name: "Expr : . Id",
    },
    Slot {
        display_name: "Expr : Id.",
    },
    Slot {
        display_name: "Expr : . Expr Layout \"(\" Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr : Expr . Layout \"(\" Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr : Expr Layout . \"(\" Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr : Expr Layout \"(\" . Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr : Expr Layout \"(\" Layout . {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr : Expr Layout \"(\" Layout {Expr !comma \",\"}* . Layout \")\"",
    },
    Slot {
        display_name: "Expr : Expr Layout \"(\" Layout {Expr !comma \",\"}* Layout . \")\"",
    },
    Slot {
        display_name: "Expr : Expr Layout \"(\" Layout {Expr !comma \",\"}* Layout \")\".",
    },
    Slot {
        display_name: "Expr : . Expr Layout \",\" Layout Expr",
    },
    Slot {
        display_name: "Expr : Expr . Layout \",\" Layout Expr",
    },
    Slot {
        display_name: "Expr : Expr Layout . \",\" Layout Expr",
    },
    Slot {
        display_name: "Expr : Expr Layout \",\" . Layout Expr",
    },
    Slot {
        display_name: "Expr : Expr Layout \",\" Layout . Expr",
    },
    Slot {
        display_name: "Expr : Expr Layout \",\" Layout Expr.",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : . {Expr !comma \",\"}+ Layout \",\" Layout Expr !comma",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : {Expr !comma \",\"}+ . Layout \",\" Layout Expr !comma",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : {Expr !comma \",\"}+ Layout . \",\" Layout Expr !comma",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : {Expr !comma \",\"}+ Layout \",\" . Layout Expr !comma",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : {Expr !comma \",\"}+ Layout \",\" Layout . Expr !comma",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : {Expr !comma \",\"}+ Layout \",\" Layout Expr !comma.",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : . Expr !comma",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+ : Expr !comma.",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+? : . {Expr !comma \",\"}+",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+? : {Expr !comma \",\"}+.",
    },
    Slot {
        display_name: "{Expr !comma \",\"}+? : .",
    },
    Slot {
        display_name: "{Expr !comma \",\"}* : . {Expr !comma \",\"}+?",
    },
    Slot {
        display_name: "{Expr !comma \",\"}* : {Expr !comma \",\"}+?.",
    },
    Slot {
        display_name: "Expr !comma : . Id",
    },
    Slot {
        display_name: "Expr !comma : Id.",
    },
    Slot {
        display_name: "Expr !comma : . Expr Layout \"(\" Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr !comma : Expr . Layout \"(\" Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr !comma : Expr Layout . \"(\" Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr !comma : Expr Layout \"(\" . Layout {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr !comma : Expr Layout \"(\" Layout . {Expr !comma \",\"}* Layout \")\"",
    },
    Slot {
        display_name: "Expr !comma : Expr Layout \"(\" Layout {Expr !comma \",\"}* . Layout \")\"",
    },
    Slot {
        display_name: "Expr !comma : Expr Layout \"(\" Layout {Expr !comma \",\"}* Layout . \")\"",
    },
    Slot {
        display_name: "Expr !comma : Expr Layout \"(\" Layout {Expr !comma \",\"}* Layout \")\".",
    },
    Slot {
        display_name: "StartExpr : . Layout start:Expr Layout",
    },
    Slot {
        display_name: "StartExpr : Layout . start:Expr Layout",
    },
    Slot {
        display_name: "StartExpr : Layout start:Expr . Layout",
    },
    Slot {
        display_name: "StartExpr : Layout start:Expr Layout.",
    },
];
impl<'i> Parser<'i> for ExcludeByLabelParser<'i> {
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
            //Expr : . Id
            SlotId(0) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Id", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Id", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Expr : Id.
                        let next_slot_id = SlotId(1);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(self, MatchFailed, "Id", i, SlotId(0), gss_node_id, result);
                    }
                }
            }
            //Expr : Id.
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
            //Expr : . Expr Layout "(" Layout Expr_Star_0 Layout ")"
            SlotId(2) => {
                self.create_expr(result, gss_node_id, SlotId(3));
            }
            //Expr : Expr . Layout "(" Layout Expr_Star_0 Layout ")"
            SlotId(3) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr : Expr Layout . "(" Layout Expr_Star_0 Layout ")"
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
            //Expr : Expr Layout . "(" Layout Expr_Star_0 Layout ")"
            SlotId(4) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Expr : Expr Layout "(" . Layout Expr_Star_0 Layout ")"
                        let next_slot_id = SlotId(5);
                        let left_child_id = result.expect("Result should not be None.");
                        let left_child = self.sppf_node(left_child_id);
                        let left_extent = left_child.left_extent();
                        if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                            next_slot_id,
                            left_extent,
                            j,
                            left_child_id,
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
                            SlotId(4),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr : Expr Layout "(" . Layout Expr_Star_0 Layout ")"
            SlotId(5) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr : Expr Layout "(" Layout . Expr_Star_0 Layout ")"
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
            //Expr : Expr Layout "(" Layout . Expr_Star_0 Layout ")"
            SlotId(6) => {
                self.create_expr_star_0(result, gss_node_id, SlotId(7));
            }
            //Expr : Expr Layout "(" Layout Expr_Star_0 . Layout ")"
            SlotId(7) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr : Expr Layout "(" Layout Expr_Star_0 Layout . ")"
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
            //Expr : Expr Layout "(" Layout Expr_Star_0 Layout . ")"
            SlotId(8) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Expr : Expr Layout "(" Layout Expr_Star_0 Layout ")".
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\")\"",
                            i,
                            SlotId(8),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr : Expr Layout "(" Layout Expr_Star_0 Layout ")".
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
            //Expr : . Expr Layout "," Layout Expr
            SlotId(10) => {
                self.create_expr(result, gss_node_id, SlotId(11));
            }
            //Expr : Expr . Layout "," Layout Expr
            SlotId(11) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr : Expr Layout . "," Layout Expr
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
            //Expr : Expr Layout . "," Layout Expr
            SlotId(12) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\",\"", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\",\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Expr : Expr Layout "," . Layout Expr
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                        }
                    }
                    None => {
                        record!(
                            self,
                            MatchFailed,
                            "\",\"",
                            i,
                            SlotId(12),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr : Expr Layout "," . Layout Expr
            SlotId(13) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr : Expr Layout "," Layout . Expr
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
                            self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //Expr : Expr Layout "," Layout . Expr
            SlotId(14) => {
                self.create_expr(result, gss_node_id, SlotId(15));
            }
            //Expr : Expr Layout "," Layout Expr.
            SlotId(15) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
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
            //Expr_Plus_0 : . Expr_Plus_0 Layout "," Layout Expr_except_comma
            SlotId(16) => {
                self.create_expr_plus_0(result, gss_node_id, SlotId(17));
            }
            //Expr_Plus_0 : Expr_Plus_0 . Layout "," Layout Expr_except_comma
            SlotId(17) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr_Plus_0 : Expr_Plus_0 Layout . "," Layout Expr_except_comma
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
            //Expr_Plus_0 : Expr_Plus_0 Layout . "," Layout Expr_except_comma
            SlotId(18) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\",\"", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\",\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(2), i, j);
                        //Expr_Plus_0 : Expr_Plus_0 Layout "," . Layout Expr_except_comma
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
                            "\",\"",
                            i,
                            SlotId(18),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr_Plus_0 : Expr_Plus_0 Layout "," . Layout Expr_except_comma
            SlotId(19) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr_Plus_0 : Expr_Plus_0 Layout "," Layout . Expr_except_comma
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
            //Expr_Plus_0 : Expr_Plus_0 Layout "," Layout . Expr_except_comma
            SlotId(20) => {
                self.create_expr_except_comma(result, gss_node_id, SlotId(21));
            }
            //Expr_Plus_0 : Expr_Plus_0 Layout "," Layout Expr_except_comma.
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
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Expr_Plus_0 : . Expr_except_comma
            SlotId(22) => {
                self.create_expr_except_comma(result, gss_node_id, SlotId(23));
            }
            //Expr_Plus_0 : Expr_except_comma.
            SlotId(23) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
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
            //Expr_Opt_0 : . Expr_Plus_0
            SlotId(24) => {
                self.create_expr_plus_0(result, gss_node_id, SlotId(25));
            }
            //Expr_Opt_0 : Expr_Plus_0.
            SlotId(25) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
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
            //Expr_Opt_0 : .
            SlotId(26) => {
                let end_slot_id = SlotId(26);
                let epsilon_node_id =
                    self.get_or_create_terminal_node(TerminalId(5), input_index, input_index);
                let nonterminal_id = NonterminalId(2);
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
            //Expr_Star_0 : . Expr_Opt_0
            SlotId(27) => {
                self.create_expr_opt_0(result, gss_node_id, SlotId(28));
            }
            //Expr_Star_0 : Expr_Opt_0.
            SlotId(28) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let end_slot_id = SlotId(28);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Expr_except_comma : . Id
            SlotId(29) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Id", i);
                match self.scanner.match_token(TerminalId(0), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Id", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(0), i, j);
                        //Expr_except_comma : Id.
                        let next_slot_id = SlotId(30);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
                    }
                    None => {
                        record!(self, MatchFailed, "Id", i, SlotId(29), gss_node_id, result);
                    }
                }
            }
            //Expr_except_comma : Id.
            SlotId(30) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(30);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //Expr_except_comma : . Expr Layout "(" Layout Expr_Star_0 Layout ")"
            SlotId(31) => {
                self.create_expr(result, gss_node_id, SlotId(32));
            }
            //Expr_except_comma : Expr . Layout "(" Layout Expr_Star_0 Layout ")"
            SlotId(32) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr_except_comma : Expr Layout . "(" Layout Expr_Star_0 Layout ")"
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
                            "Layout",
                            i,
                            SlotId(32),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr_except_comma : Expr Layout . "(" Layout Expr_Star_0 Layout ")"
            SlotId(33) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\"(\"", i);
                match self.scanner.match_token(TerminalId(1), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\"(\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(1), i, j);
                        //Expr_except_comma : Expr Layout "(" . Layout Expr_Star_0 Layout ")"
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
                            "\"(\"",
                            i,
                            SlotId(33),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr_except_comma : Expr Layout "(" . Layout Expr_Star_0 Layout ")"
            SlotId(34) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr_except_comma : Expr Layout "(" Layout . Expr_Star_0 Layout ")"
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
            //Expr_except_comma : Expr Layout "(" Layout . Expr_Star_0 Layout ")"
            SlotId(35) => {
                self.create_expr_star_0(result, gss_node_id, SlotId(36));
            }
            //Expr_except_comma : Expr Layout "(" Layout Expr_Star_0 . Layout ")"
            SlotId(36) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //Expr_except_comma : Expr Layout "(" Layout Expr_Star_0 Layout . ")"
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
                            "Layout",
                            i,
                            SlotId(36),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr_except_comma : Expr Layout "(" Layout Expr_Star_0 Layout . ")"
            SlotId(37) => {
                let i = input_index;
                record!(self, MatchingTerminal, "\")\"", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "\")\"", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(3), i, j);
                        //Expr_except_comma : Expr Layout "(" Layout Expr_Star_0 Layout ")".
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
                            "\")\"",
                            i,
                            SlotId(37),
                            gss_node_id,
                            result
                        );
                    }
                }
            }
            //Expr_except_comma : Expr Layout "(" Layout Expr_Star_0 Layout ")".
            SlotId(38) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(4);
                let end_slot_id = SlotId(38);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    let popped_element = PoppedElement {
                        nonterminal_node_id,
                        return_value: None,
                    };
                    self.pop(gss_node_id, end_slot_id, popped_element);
                }
            }
            //StartExpr : . Layout start:Expr Layout
            SlotId(39) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //StartExpr : Layout . start:Expr Layout
                        let next_slot_id = SlotId(40);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id, env);
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
            //StartExpr : Layout . start:Expr Layout
            SlotId(40) => {
                self.create_expr(result, gss_node_id, SlotId(41));
            }
            //StartExpr : Layout start:Expr . Layout
            SlotId(41) => {
                let i = input_index;
                record!(self, MatchingTerminal, "Layout", i);
                match self.scanner.match_token(TerminalId(4), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "Layout", i, j);
                        let right_child_id = self.get_or_create_terminal_node(TerminalId(4), i, j);
                        //StartExpr : Layout start:Expr Layout.
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
            //StartExpr : Layout start:Expr Layout.
            SlotId(42) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(5);
                let end_slot_id = SlotId(42);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    end_slot_id,
                    left_extent,
                    right_extent,
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
            //Expr
            NonterminalId(0) => {
                //Expr : . Id
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Expr : . Expr Layout "(" Layout Expr_Star_0 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(2),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Expr : . Expr Layout "," Layout Expr
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(10),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Expr_Plus_0
            NonterminalId(1) => {
                //Expr_Plus_0 : . Expr_Plus_0 Layout "," Layout Expr_except_comma
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(16),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Expr_Plus_0 : . Expr_except_comma
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(22),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Expr_Opt_0
            NonterminalId(2) => {
                //Expr_Opt_0 : . Expr_Plus_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(24),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Expr_Opt_0 : .
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(26),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Expr_Star_0
            NonterminalId(3) => {
                //Expr_Star_0 : . Expr_Opt_0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(27),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //Expr_except_comma
            NonterminalId(4) => {
                //Expr_except_comma : . Id
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(29),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
                //Expr_except_comma : . Expr Layout "(" Layout Expr_Star_0 Layout ")"
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(31),
                    sppf_node_id: None,
                    gss_node_id,
                    env,
                });
            }
            //StartExpr
            NonterminalId(5) => {
                //StartExpr : . Layout start:Expr Layout
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(39),
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
pub struct ExcludeByLabelParser<'i> {
    start_nonterminal: NonterminalId,
    scanner: ExcludeByLabelScanner<'i>,
    descriptors: Vec<Descriptor>,
    gss_nodes: Vec<GSSNode>,
    //A vector from nonterminal_ids to a tuple (input_index, gss_node_id)
    gss_nodes_index: [Vec<(u32, GssNodeId)>; 6],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 6],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 43],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 6],
    intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
    intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
    nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
    nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
    envs: Vec<Env>,
    #[cfg(feature = "debug-trace")]
    pub trace_events: Option<Vec<TraceEvent>>,
}
impl<'i> ExcludeByLabelParser<'i> {
    pub fn new(input: &'i Input, start_nonterminal: NonterminalId) -> Self {
        init_logger();
        Self {
            start_nonterminal,
            scanner: ExcludeByLabelScanner::new(input),
            gss_nodes_index: [const { vec![] }; 6],
            descriptors: vec![],
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 6],
            intermediate_nodes_index: [const { InlineMap::Empty }; 43],
            terminal_nodes_index: [const { InlineMap::Empty }; 6],
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
    fn create_expr(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(0), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_expr_plus_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(1), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_expr_opt_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(2), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_expr_star_0(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(3), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_expr_except_comma(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(4), sppf_node_id, gss_node_id, return_slot);
    }
    fn create_start_expr(
        &mut self,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        self.create(NonterminalId(5), sppf_node_id, gss_node_id, return_slot);
    }
}

