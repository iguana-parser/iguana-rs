use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

use crate::{
    descriptor::Descriptor, gss::{EdgeResult, GSSEdge, GSSNode}, ids::{GssNodeId, NonterminalId, SlotId, TerminalId}, input::Input, record, sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode}
};

#[cfg(feature = "debug-trace")]
use crate::trace::TraceEvent;

pub enum ParseResult {
    Success(ParseSuccess),
    Failure(),
}

pub struct ParseSuccess {
    pub sppf_node_id: SPPFNodeId,
    pub duration: Duration,
    pub stats: Stats,
}

pub trait Parser<'i> {
    fn nonterminal_display_name(nonterminal_id: NonterminalId) -> &'static str;
    fn nonterminal_id(name: &str) -> Option<NonterminalId>;
    fn terminal_name(terminal_id: TerminalId) -> &'static str;
    fn slot_name(slot_id: SlotId) -> &'static str;
    fn epsilon() -> TerminalId;
    fn execute(
        &mut self,
        input_index: u32,
        slot_id: SlotId,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
    );
    fn add_first_descriptors(
        &mut self,
        nonterminal_id: NonterminalId,
        input_index: u32,
        gss_node_id: GssNodeId,
    );
    fn start_nonterminal(&self) -> NonterminalId;
    fn get_gss_node(&self, nonterminal_id: NonterminalId, input_index: u32) -> Option<GssNodeId>;
    fn add_gss_node(
        &mut self,
        nonterminal_id: NonterminalId,
        input_index: u32,
        gss_node_id: GssNodeId,
    );
    fn new_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> GssNodeId;
    fn gss_node(&self, id: GssNodeId) -> &GSSNode;
    fn sppf_node(&self, id: SPPFNodeId) -> &SPPFNode;
    fn sppf_node_mut(&mut self, id: SPPFNodeId) -> &mut SPPFNode;
    fn gss_node_mut(&mut self, id: GssNodeId) -> &mut GSSNode;
    fn add_descriptor(&mut self, descriptor: Descriptor);
    fn next_descriptor(&mut self) -> Option<Descriptor>;
    fn input(&self) -> &'i Input;
    fn stats(&self) -> &Stats;
    fn stats_mut(&mut self) -> &mut Stats;
    fn add_nonterminal_node_child(&mut self, node: SPPFNodeId, child: SPPFNodeId);
    fn add_intermediate_node_child(
        &mut self,
        node: SPPFNodeId,
        child1: SPPFNodeId,
        child2: SPPFNodeId,
    );

    /// Looks up an existing nonterminal node for the specified nonterminal_id and span (left_extent, right_extent).
    /// Returns None if no such node exists.
    fn lookup_nonterminal_node(
        &self,
        nonterminal_id: NonterminalId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<SPPFNodeId>;

    /// Looks up an existing intermediate node for the specified slot_id and span (left_extent, right_extent).
    /// Returns None if no such node exists.
    fn lookup_intermediate_node(
        &self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<SPPFNodeId>;

    /// Looks up an existing terminal node for the specified `slot_id` and span (`left_extent`, `right_extent`).
    /// Returns None if no such node exists.
    fn lookup_terminal_node(
        &self,
        terminal_id: TerminalId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<SPPFNodeId>;

    /// Creates a new GSS node if it does not exist.
    /// If a GSS node with the same nonterminal name and input index exists, just adds an edge.
    /// `create` corresponds to a function call in recursive-descent parsers.
    ///
    /// # Arguments
    ///
    /// * `nonterminal_id` - The nonterminal id.
    /// * `sppf_node_id` - The current sppf_node, corresponding to the result of parsing before the call.
    ///   If the nonterminal is called at position 0, i.e., it's the first symbol in the production rule,
    ///   the sppf_node_id is `None`
    /// * `gss_node_id` - The id of the current GSS node.
    /// * `return_slot` - The grammar slot immediately after the nonterminal being called. This is used to record
    ///   the grammar slot to continue parsing when the call returns (pop action).
    fn create(
        &mut self,
        nonterminal_id: NonterminalId,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        return_slot: SlotId,
    ) {
        record!(self, Call, sppf_node_id, gss_node_id, return_slot);
        let sppf_node = sppf_node_id.map(|id| self.sppf_node(id));
        let edge_result = sppf_node.map(|n| EdgeResult {
            node_id: sppf_node_id.unwrap(),
            left_extent: n.left_extent(),
        });
        let gss_node = self.gss_node(gss_node_id);
        let i = match sppf_node {
            Some(node) => node.right_extent(),
            None => gss_node.index,
        };
        // If there is already a GSS node for this call, just add the edge
        if let Some(exiting_gss_node_id) = self.get_gss_node(nonterminal_id, i) {
            record!(self, GSSNodeFound, nonterminal_id, i);
            let popped_elements =
                std::mem::take(self.gss_node_mut(exiting_gss_node_id).popped_elements_mut());

            // For each popped element of the current GSS node add a descriptor with the return label.
            for popped_element in popped_elements.iter() {
                let popped_node = self.sppf_node(*popped_element);
                let right_extent = popped_node.right_extent();
                if let Some(new_node) = self.merge(
                    sppf_node_id,
                    *popped_element,
                    return_slot,
                    edge_result.clone().map(|r| r.left_extent),
                    right_extent,
                ) {
                    self.add_descriptor(Descriptor {
                        input_index: right_extent,
                        slot_id: return_slot,
                        sppf_node_id: Some(new_node),
                        gss_node_id,
                    });
                }
            }
            *self.gss_node_mut(exiting_gss_node_id).popped_elements_mut() = popped_elements;

            self.add_gss_edge(exiting_gss_node_id, gss_node_id, edge_result, return_slot);
        } else {
            record!(self, GSSNodeNotFound, nonterminal_id, i);
            let new_gss_node_id = self.new_gss_node(nonterminal_id, i);
            self.add_gss_edge(new_gss_node_id, gss_node_id, edge_result, return_slot);
            self.add_first_descriptors(nonterminal_id, i, new_gss_node_id);
            self.add_gss_node(nonterminal_id, i, new_gss_node_id);
        }
    }

    fn add_gss_edge(
        &mut self,
        origin_gss_node_id: GssNodeId,
        dest_gss_node_id: GssNodeId,
        result: Option<EdgeResult>,
        return_slot: SlotId,
    ) {
        let origin = self.gss_node_mut(origin_gss_node_id);
        let gss_edge = GSSEdge::new(result, return_slot, dest_gss_node_id);
        origin.add_edge(gss_edge);
        record!(
            self,
            GSSNodeAdded,
            origin_gss_node_id,
            dest_gss_node_id,
            return_slot
        );
        self.stats_mut().gss_edges_count += 1;
    }

    fn pop(&mut self, gss_node_id: GssNodeId, slot_id: SlotId, sppf_node_id: SPPFNodeId) {
        record!(self, Pop, gss_node_id, slot_id, sppf_node_id);
        let gss = self.gss_node(gss_node_id);
        if gss.contains_popped_element(&sppf_node_id) {
            record!(self, NodeAlreadyInPoppedElements);
        }
        let node = self.sppf_node(sppf_node_id);
        let right_extent = node.right_extent();
        record!(self, AddToPoppedElements, gss_node_id, sppf_node_id);
        let gss = self.gss_node_mut(gss_node_id);
        gss.add_to_popped_elements(sppf_node_id);
        let edges = gss.edges().clone();
        for edge in edges.iter() {
            if let Some(new_node_id) = self.merge(
                edge.result.as_ref().map(|r| r.node_id),
                sppf_node_id,
                edge.return_slot,
                edge.result.as_ref().map(|r| r.left_extent),
                right_extent,
            ) {
                self.add_descriptor(Descriptor {
                    input_index: right_extent,
                    slot_id: edge.return_slot,
                    sppf_node_id: Some(new_node_id),
                    gss_node_id: edge.dest_id,
                });
            }
        }
    }

    /// Returns None if the intermediate node already exists.
    /// A new descriptor should only be added when merge returns Some(n).
    fn merge(
        &mut self,
        left_child: Option<SPPFNodeId>,
        right_child: SPPFNodeId,
        slot_id: SlotId,
        left_extent: Option<u32>,
        right_extent: u32,
    ) -> Option<SPPFNodeId> {
        if let (Some(left_child), Some(left_extent)) = (left_child, left_extent) {
            self.create_intermediate_node_or_attach_children(
                slot_id,
                left_extent,
                right_extent,
                left_child,
                right_child,
            )
        } else {
            Some(right_child)
        }
    }

    fn gss_to_string(&self, gss_node_id: GssNodeId) -> String {
        let gss_node = self.gss_node(gss_node_id);
        format!(
            "({},{})",
            Self::nonterminal_display_name(gss_node.nonterminal_id),
            gss_node.index
        )
    }

    fn sppf_node_to_string(&self, sppf_node: &SPPFNode) -> String {
        match sppf_node {
            SPPFNode::Terminal(t) => {
                format!(
                    "({}, {}, {})",
                    Self::terminal_name(t.terminal_id),
                    t.span.left_extent,
                    t.span.right_extent
                )
            }
            SPPFNode::Nonterminal(n) => format!(
                "({}, {}, {})",
                Self::nonterminal_display_name(n.nonterminal_id),
                n.span.left_extent,
                n.span.right_extent
            ),
            SPPFNode::Intermediate(i) => {
                format!(
                    "({}, {}, {})",
                    Self::slot_name(i.slot_id),
                    i.span.left_extent,
                    i.span.right_extent
                )
            }
        }
    }

    /// Looks up the nonterminal node identified by `nonterminal_id` and the span
    /// (`left_extent`, `right_extent`). If no such node exists, it is created and
    /// added to the index; see `add_nonterminal_node`.
    ///
    /// If the node already exists, the `child` is added to its list of children,
    /// and returns None. This only occurs when there is an ambiguity.
    fn create_nonterminal_node_or_attach_children(
        &mut self,
        nonterminal_id: NonterminalId,
        return_slot: SlotId,
        left_extent: u32,
        right_extent: u32,
        child: SPPFNodeId,
    ) -> Option<SPPFNodeId> {
        if let Some(existing_node_id) =
            self.lookup_nonterminal_node(nonterminal_id, left_extent, right_extent)
        {
            record!(self, NonterminalNodeFound, existing_node_id);
            let node = self.sppf_node_mut(existing_node_id);
            let SPPFNode::Nonterminal(node) = node else {
                unreachable!("Expects a nonterminal node");
            };
            // Only count an ambiguous node once, i.e., when the second child is attached.
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
        Some(self.add_nonterminal_node(nonterminal_node))
    }

    /// Looks up the intermediate node identified by `slot_id` and the span
    /// (`left_extent`, `right_extent`). If no such node exists, it is created and
    /// added to the index; see `add_intermediate_node`.
    ///
    /// If the node already exists, the `(left_child, right_child)` is added to its list of children,
    /// and returns None. This only occurs when there is an ambiguity.
    fn create_intermediate_node_or_attach_children(
        &mut self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
        left_child: SPPFNodeId,
        right_child: SPPFNodeId,
    ) -> Option<SPPFNodeId> {
        if let Some(existing_node_id) =
            self.lookup_intermediate_node(slot_id, left_extent, right_extent)
        {
            record!(self, IntermediateNodeFound, existing_node_id);
            let SPPFNode::Intermediate(node) = self.sppf_node_mut(existing_node_id) else {
                unreachable!("It's a nonterminal node");
            };
            // Only count an ambiguous node once, i.e., when the second child is attached.
            if !node.ambiguous {
                node.ambiguous = true;
                self.stats_mut().ambiguous_nodes += 1;
            }
            self.add_intermediate_node_child(existing_node_id, left_child, right_child);
            return None;
        }
        let intermediate_node = IntermediateNode {
            slot_id,
            span: Span {
                left_extent,
                right_extent,
            },
            child: (left_child, right_child),
            ambiguous: false,
        };
        Some(self.add_intermediate_node(intermediate_node))
    }

    fn get_or_create_terminal_node(
        &mut self,
        terminal_id: TerminalId,
        left_extent: u32,
        right_extent: u32,
    ) -> SPPFNodeId {
        if let Some(existing_node_id) =
            self.lookup_terminal_node(terminal_id, left_extent, right_extent)
        {
            record!(self, TerminalNodeFound, existing_node_id);
            return existing_node_id;
        }
        let terminal_node = TerminalNode {
            terminal_id,
            span: Span {
                left_extent,
                right_extent,
            },
        };
        self.add_terminal_node(terminal_node)
    }

    fn add_nonterminal_node(&mut self, nonterminal_node: NonterminalNode) -> SPPFNodeId;

    fn add_intermediate_node(&mut self, intermediate_node: IntermediateNode) -> SPPFNodeId;

    fn add_terminal_node(&mut self, node: TerminalNode) -> SPPFNodeId;

    fn run(&mut self) -> ParseResult {
        let start = Instant::now();
        let start_input_index = 0;
        let start_nonterminal_id = self.start_nonterminal();
        let start_gss_node_id = self.new_gss_node(start_nonterminal_id, start_input_index);
        self.add_first_descriptors(start_nonterminal_id, start_input_index, start_gss_node_id);
        self.add_gss_node(start_nonterminal_id, start_input_index, start_gss_node_id);
        while let Some(descriptor) = self.next_descriptor() {
            self.execute(
                descriptor.input_index,
                descriptor.slot_id,
                descriptor.sppf_node_id,
                descriptor.gss_node_id,
            );
        }
        let duration = start.elapsed();
        let right_extent = self.input().len();
        if let Some(sppf_node_id) =
            self.lookup_nonterminal_node(start_nonterminal_id, 0, right_extent)
        {
            ParseResult::Success(ParseSuccess {
                sppf_node_id,
                duration,
                stats: self.stats().clone(),
            })
        } else {
            ParseResult::Failure()
        }
    }
    fn gss_nodes(&self) -> impl Iterator<Item = &GSSNode>;

    fn intermediate_nodes_children_map(
        &self,
    ) -> &FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>;

    fn nonterminal_nodes_children_map(&self) -> &FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>;

    #[cfg(feature = "debug-trace")]
    fn add_trace_event(&mut self, event: TraceEvent);
}

pub fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "{}: {}", record.level(), record.args())
        })
        .try_init();
}

#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub descriptors_count: usize,
    pub gss_nodes_count: usize,
    pub gss_edges_count: usize,
    pub nonterminal_nodes_count: usize,
    pub intermediate_nodes_count: usize,
    pub terminal_nodes_count: usize,
    pub ambiguous_nodes: usize,
}

impl Stats {
    pub fn count_all_sppf_nodes(&self) -> usize {
        self.nonterminal_nodes_count + self.intermediate_nodes_count + self.terminal_nodes_count
    }
}
