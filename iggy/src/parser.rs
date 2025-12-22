use crate::scanner::IggyScanner;
#[cfg(feature = "debug-trace")]
use iguana::trace::TraceEvent;
use iguana::{
    descriptor::Descriptor,
    gss::GSSNode,
    ids::{NonterminalId, SlotId, TerminalId},
    input::Input,
    parser::{Parser, Stats, init_logger},
    record,
    scanner::Scanner,
    sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
    utils::inline_map::InlineMap,
};
use rustc_hash::FxHashMap;
use std::cell::OnceCell;
const NONTERMINALS: [&str; 4] = ["Grammar", "Rule", "Grammar_Plus0", "Rule_Plus1"];
const TERMINALS: [&str; 4] = ["Identifier", "WS", "grammar", ":"];
const SLOTS: [&str; 18] = [
    "Grammar : . \"grammar\" Identifier Grammar_Plus0",
    "Grammar : \"grammar\" . Identifier Grammar_Plus0",
    "Grammar : \"grammar\" Identifier . Grammar_Plus0",
    "Grammar : \"grammar\" Identifier Grammar_Plus0.",
    "Rule : . Identifier \":\" Rule_Plus1",
    "Rule : Identifier . \":\" Rule_Plus1",
    "Rule : Identifier \":\" . Rule_Plus1",
    "Rule : Identifier \":\" Rule_Plus1.",
    "Grammar_Plus0 : . Grammar_Plus0 Rule",
    "Grammar_Plus0 : Grammar_Plus0 . Rule",
    "Grammar_Plus0 : Grammar_Plus0 Rule.",
    "Grammar_Plus0 : . Rule",
    "Grammar_Plus0 : Rule.",
    "Rule_Plus1 : . Rule_Plus1 Identifier",
    "Rule_Plus1 : Rule_Plus1 . Identifier",
    "Rule_Plus1 : Rule_Plus1 Identifier.",
    "Rule_Plus1 : . Identifier",
    "Rule_Plus1 : Identifier.",
];
impl<'i> Parser<'i> for IggyParser<'i> {
    fn execute(
        &mut self,
        input_index: u32,
        slot_id: SlotId,
        result: Option<SPPFNodeId>,
        gss_node_id: usize,
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
            //Grammar : . "grammar" Identifier Grammar_Plus0
            SlotId(0) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, "grammar", i);
                match self.scanner.match_token(TerminalId(2), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, "grammar", i, j);
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
                        //Grammar : "grammar" . Identifier Grammar_Plus0
                        let next_slot_id = SlotId(1);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(self, MatchFailed, "grammar", i);
                    }
                }
            }
            //Grammar : "grammar" . Identifier Grammar_Plus0
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
                        //Grammar : "grammar" Identifier . Grammar_Plus0
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
                        record!(self, MatchFailed, "Identifier", i);
                    }
                }
            }
            //Grammar : "grammar" Identifier . Grammar_Plus0
            SlotId(2) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(3));
            }
            //Grammar : "grammar" Identifier Grammar_Plus0.
            SlotId(3) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(0);
                let return_slot = SlotId(3);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    return_slot,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, nonterminal_node_id);
                }
            }
            //Rule : . Identifier ":" Rule_Plus1
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
                        //Rule : Identifier . ":" Rule_Plus1
                        let next_slot_id = SlotId(5);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(self, MatchFailed, "Identifier", i);
                    }
                }
            }
            //Rule : Identifier . ":" Rule_Plus1
            SlotId(5) => {
                record!(self, MatchingLeadingLayout, input_index);
                let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
                record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                record!(self, MatchingTerminal, ":", i);
                match self.scanner.match_token(TerminalId(3), i) {
                    Some(j) => {
                        record!(self, MatchSuccess, ":", i, j);
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
                        //Rule : Identifier ":" . Rule_Plus1
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
                        record!(self, MatchFailed, ":", i);
                    }
                }
            }
            //Rule : Identifier ":" . Rule_Plus1
            SlotId(6) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(7));
            }
            //Rule : Identifier ":" Rule_Plus1.
            SlotId(7) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(1);
                let return_slot = SlotId(7);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    return_slot,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, nonterminal_node_id);
                }
            }
            //Grammar_Plus0 : . Grammar_Plus0 Rule
            SlotId(8) => {
                self.create(NonterminalId(2), result, gss_node_id, SlotId(9));
            }
            //Grammar_Plus0 : Grammar_Plus0 . Rule
            SlotId(9) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(10));
            }
            //Grammar_Plus0 : Grammar_Plus0 Rule.
            SlotId(10) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let return_slot = SlotId(10);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    return_slot,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, nonterminal_node_id);
                }
            }
            //Grammar_Plus0 : . Rule
            SlotId(11) => {
                self.create(NonterminalId(1), result, gss_node_id, SlotId(12));
            }
            //Grammar_Plus0 : Rule.
            SlotId(12) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(2);
                let return_slot = SlotId(12);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    return_slot,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, nonterminal_node_id);
                }
            }
            //Rule_Plus1 : . Rule_Plus1 Identifier
            SlotId(13) => {
                self.create(NonterminalId(3), result, gss_node_id, SlotId(14));
            }
            //Rule_Plus1 : Rule_Plus1 . Identifier
            SlotId(14) => {
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
                        //Rule_Plus1 : Rule_Plus1 Identifier.
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
                        record!(self, MatchFailed, "Identifier", i);
                    }
                }
            }
            //Rule_Plus1 : Rule_Plus1 Identifier.
            SlotId(15) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let return_slot = SlotId(15);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    return_slot,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, nonterminal_node_id);
                }
            }
            //Rule_Plus1 : . Identifier
            SlotId(16) => {
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
                        //Rule_Plus1 : Identifier.
                        let next_slot_id = SlotId(17);
                        let new_node = right_child_id;
                        self.execute(j, next_slot_id, Some(new_node), gss_node_id);
                    }
                    None => {
                        record!(self, MatchFailed, "Identifier", i);
                    }
                }
            }
            //Rule_Plus1 : Identifier.
            SlotId(17) => {
                let Some(result) = result else {
                    unreachable!("result cannot be None here.")
                };
                let node = self.sppf_node(result);
                let left_extent = node.left_extent();
                let right_extent = node.right_extent();
                let nonterminal_id = NonterminalId(3);
                let return_slot = SlotId(17);
                if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                    nonterminal_id,
                    return_slot,
                    left_extent,
                    right_extent,
                    result,
                ) {
                    self.pop(gss_node_id, nonterminal_node_id);
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
        gss_node_id: usize,
    ) {
        match nonterminal_id {
            //Grammar
            NonterminalId(0) => {
                //Grammar : . "grammar" Identifier Grammar_Plus0
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(0),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule
            NonterminalId(1) => {
                //Rule : . Identifier ":" Rule_Plus1
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(4),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Grammar_Plus0
            NonterminalId(2) => {
                //Grammar_Plus0 : . Grammar_Plus0 Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(8),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Grammar_Plus0 : . Rule
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(11),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            //Rule_Plus1
            NonterminalId(3) => {
                //Rule_Plus1 : . Rule_Plus1 Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(13),
                    sppf_node_id: None,
                    gss_node_id,
                });
                //Rule_Plus1 : . Identifier
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: SlotId(16),
                    sppf_node_id: None,
                    gss_node_id,
                });
            }
            _ => {
                panic!("Unknown nonterminal id: {nonterminal_id}");
            }
        }
    }
    fn nonterminal_name(&self, nonterminal_id: NonterminalId) -> &str {
        NONTERMINALS[nonterminal_id.index()]
    }
    fn terminal_name(&self, terminal_id: TerminalId) -> &str {
        TERMINALS[terminal_id.index()]
    }
    fn slot_name(&self, slot_id: SlotId) -> &str {
        SLOTS[slot_id.index()]
    }
    fn get_gss_node(&self, nonterminal_id: NonterminalId, input_index: u32) -> Option<usize> {
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
        gss_node_id: usize,
    ) {
        let gss_nodes = &mut self.gss_nodes_index[nonterminal_id.index()];
        gss_nodes.push((input_index, gss_node_id));
    }
    fn new_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> usize {
        let id = self.gss_nodes.len();
        let gss_node = GSSNode::new(id, nonterminal_id, input_index);
        record!(self, GSSNodeCreated, nonterminal_id, input_index);
        self.gss_nodes.push(gss_node);
        self.stats.gss_nodes_count += 1;
        self.gss_nodes[id].id
    }
    fn gss_node(&self, id: usize) -> &GSSNode {
        &self.gss_nodes[id]
    }
    fn gss_node_mut(&mut self, id: usize) -> &mut GSSNode {
        self.gss_nodes
            .get_mut(id)
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
            nonterminal_node.span
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
            intermediate_node.span
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
}
pub struct IggyParser<'i> {
    descriptors: Vec<Descriptor>,
    scanner: IggyScanner<'i>,
    gss_nodes: Vec<GSSNode>,
    //A vector from nonterminal_ids to a tuple (input_index, gss_node_id)
    gss_nodes_index: [Vec<(u32, usize)>; 4],
    sppf_nodes: Vec<SPPFNode>,
    stats: Stats,
    nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 4],
    intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; 18],
    terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; 4],
    intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
    intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
    nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
    nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
    #[cfg(feature = "debug-trace")]
    pub trace_events: Option<Vec<TraceEvent>>,
}
impl<'i> IggyParser<'i> {
    pub fn new(input: &'i Input) -> Self {
        init_logger();
        Self {
            gss_nodes_index: [const { vec![] }; 4],
            descriptors: vec![],
            scanner: IggyScanner::new(input),
            gss_nodes: vec![],
            sppf_nodes: vec![],
            nonterminal_nodes_index: [const { InlineMap::Empty }; 4],
            intermediate_nodes_index: [const { InlineMap::Empty }; 18],
            terminal_nodes_index: [const { InlineMap::Empty }; 4],
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

