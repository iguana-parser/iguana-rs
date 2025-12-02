use std::time::Instant;

use log::{debug, info, trace};
use rustc_hash::FxHashMap;

use crate::{
    descriptor::Descriptor,
    gss::{EdgeResult, GSSEdge, GSSNode},
    sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
};

pub trait Parser<'i> {
    fn execute(&mut self, slot_id: SlotId, result: Option<SPPFNodeId>, gss_node_id: usize);
    fn add_first_descriptors(&mut self, nonterminal_id: NonterminalId, gss_node_id: usize);
    fn nonterminal_name(&self, nonterminal_id: NonterminalId) -> &str;
    fn terminal_name(&self, terminal_id: TerminalId) -> &str;
    fn slot_name(&self, slot_id: SlotId) -> &str;
    fn get_gss_node(&self, nonterminal_id: NonterminalId, input_index: u32) -> Option<usize>;
    fn add_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32, gss_node_id: usize);
    fn new_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> usize;
    fn gss_node(&self, id: usize) -> &GSSNode;
    fn sppf_node(&self, id: SPPFNodeId) -> &SPPFNode;
    fn sppf_node_mut(&mut self, id: SPPFNodeId) -> &mut SPPFNode;
    fn gss_node_mut(&mut self, id: usize) -> &mut GSSNode;
    fn add_descriptor(&mut self, descriptor: Descriptor);
    fn next_descriptor(&mut self) -> Option<Descriptor>;
    fn input_len(&self) -> u32;
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

    /// Creates a new GSS node if it does not exist for parsing a nonterminal.
    /// If a GSS node with the same nonterminal name and input index exists, just adds an edge.
    /// This corresponds to a function call in recursive-descent parsers.
    ///
    /// # Arguments
    ///
    /// * `nonterminal_id` - The nonterminal id.
    /// * `result` - The current result, corresponding to the portion of the production rule before the call.
    ///   If the nonterminal is called at position 0, i.e., it's the first symbol in the production rule,
    ///   the result is `None`
    /// * `gss_node_id` - The id of the current GSS node.
    /// * `return_slot` - The grammar slot immediately after the nonterminal being called. This is used to record
    ///   the grammar slot to continue parsing when the call returns (pop action).
    fn create(
        &mut self,
        nonterminal_id: NonterminalId,
        result: Option<SPPFNodeId>,
        gss_node_id: usize,
        return_slot: SlotId,
    ) {
        let sppf_node = result.map(|id| self.sppf_node(id));
        let gss_node = self.gss_node(gss_node_id);
        trace!(
            "Create {}, {}, {}",
            sppf_node
                .map(|n| self.sppf_node_to_string(n))
                .unwrap_or("$".to_owned()),
            self.gss_to_string(gss_node_id),
            self.slot_name(return_slot)
        );
        let edge_result = sppf_node.map(|n| EdgeResult {
            node_id: result.unwrap(),
            left_extent: n.left_extent(),
        });
        let i = match sppf_node {
            Some(node) => node.right_extent(),
            None => gss_node.index,
        };
        // If there is already a GSS node for this call, just add the edge
        if let Some(exiting_gss_node_id) = self.get_gss_node(nonterminal_id, i) {
            trace!(
                "GSS node ({},{}) found",
                self.nonterminal_name(nonterminal_id),
                i
            );
            let popped_elements =
                std::mem::take(self.gss_node_mut(exiting_gss_node_id).popped_elements_mut());

            // For each popped element of the current GSS node add a descriptor with the return label.
            for popped_element in popped_elements.iter() {
                let popped_node = self.sppf_node(*popped_element);
                if let Some(new_node) = self.merge(
                    result,
                    *popped_element,
                    return_slot,
                    edge_result.clone().map(|r| r.left_extent),
                    popped_node.right_extent(),
                ) {
                    self.add_descriptor(Descriptor::new(return_slot, Some(new_node), gss_node_id));
                }
            }
            *self.gss_node_mut(exiting_gss_node_id).popped_elements_mut() = popped_elements;

            self.add_gss_edge(exiting_gss_node_id, gss_node_id, edge_result, return_slot);
        } else {
            trace!(
                "GSS node ({},{}) not found",
                self.nonterminal_name(nonterminal_id),
                i
            );
            let new_gss_node_id = self.new_gss_node(nonterminal_id, i);
            self.add_gss_edge(new_gss_node_id, gss_node_id, edge_result, return_slot);
            self.add_first_descriptors(nonterminal_id, new_gss_node_id);
            self.add_gss_node(nonterminal_id, i, new_gss_node_id);
        }
    }

    fn add_gss_edge(
        &mut self,
        origin_gss_id: usize,
        dest_id: usize,
        result: Option<EdgeResult>,
        return_slot: SlotId,
    ) {
        let origin = self.gss_node_mut(origin_gss_id);
        let gss_edge = GSSEdge::new(result, return_slot, dest_id);
        origin.add_edge(gss_edge);
        trace!(
            "GSS edge added from {} to {} with return label {}",
            self.gss_to_string(origin_gss_id),
            self.gss_to_string(dest_id),
            self.slot_name(return_slot)
        );
        self.stats_mut().gss_edges_count += 1;
    }

    fn pop(&mut self, gss_node_id: usize, node_id: SPPFNodeId) {
        trace!(
            "Pop: {} with result {}",
            self.gss_to_string(gss_node_id),
            self.sppf_node_to_string(self.sppf_node(node_id))
        );
        let gss = self.gss_node(gss_node_id);
        if gss.contains_popped_element(&node_id) {
            trace!("Node already in popped elements");
            return;
        }
        let node = self.sppf_node(node_id);
        let right_extent = node.right_extent();
        trace!(
            "Added {} to {}'s popped elements",
            self.sppf_node_to_string(self.sppf_node(node_id)),
            self.gss_to_string(gss_node_id)
        );
        let gss = self.gss_node_mut(gss_node_id);
        gss.add_to_popped_elements(node_id);
        let edges = gss.edges().clone();
        for edge in edges.iter() {
            if let Some(new_node_id) = self.merge(
                edge.result.as_ref().map(|r| r.node_id),
                node_id,
                edge.return_slot,
                edge.result.as_ref().map(|r| r.left_extent),
                right_extent,
            ) {
                self.add_descriptor(Descriptor {
                    slot_id: edge.return_slot,
                    result: Some(new_node_id),
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

    fn gss_to_string(&self, gss_node_id: usize) -> String {
        let gss_node = self.gss_node(gss_node_id);
        format!(
            "({},{})",
            self.nonterminal_name(gss_node.nonterminal_id),
            gss_node.index
        )
    }

    fn sppf_node_to_string(&self, sppf_node: &SPPFNode) -> String {
        match sppf_node {
            SPPFNode::Terminal(t) => {
                format!(
                    "({}, {}, {})",
                    self.terminal_name(t.terminal_id),
                    t.span.left_extent,
                    t.span.right_extent
                )
            }
            SPPFNode::Nonterminal(n) => format!(
                "({}, {}, {})",
                self.nonterminal_name(n.nonterminal_id),
                n.span.left_extent,
                n.span.right_extent
            ),
            SPPFNode::Intermediate(i) => {
                format!(
                    "({}, {}, {})",
                    self.slot_name(i.slot_id),
                    i.span.left_extent,
                    i.span.right_extent
                )
            }
        }
    }

    fn descriptor_to_string(&self, descriptor: &Descriptor) -> String {
        format!(
            "({}, {}, {})",
            self.slot_name(descriptor.slot_id),
            self.gss_to_string(descriptor.gss_node_id),
            if let Some(result) = descriptor.result {
                self.sppf_node_to_string(self.sppf_node(result))
            } else {
                "$".to_string()
            }
        )
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
            trace!(
                "Nonterminal node found {}",
                self.sppf_node_to_string(self.sppf_node(existing_node_id))
            );
            let node = self.sppf_node_mut(existing_node_id);
            let SPPFNode::Nonterminal(node) = node else {
                unreachable!("It's a nonterminal node");
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
            trace!(
                "Intermediate node found {}",
                self.sppf_node_to_string(self.sppf_node(existing_node_id))
            );
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
            trace!(
                "Terminal node found {}",
                self.sppf_node_to_string(self.sppf_node(existing_node_id))
            );
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

    fn run(&mut self, start_nonterminal_id: NonterminalId) -> Option<SPPFNodeId> {
        let start = Instant::now();
        let start_gss_node_id = self.new_gss_node(start_nonterminal_id, 0);
        self.add_first_descriptors(start_nonterminal_id, start_gss_node_id);
        self.add_gss_node(start_nonterminal_id, 0, start_gss_node_id);
        while let Some(descriptor) = self.next_descriptor() {
            self.execute(
                descriptor.slot_id,
                descriptor.result,
                descriptor.gss_node_id,
            );
        }
        debug!("Processing descriptors finished.");
        let duration = start.elapsed();
        let right_extent = self.input_len();
        if let Some(node_id) = self.lookup_nonterminal_node(start_nonterminal_id, 0, right_extent) {
            info!("Parse successful. ({} ms)", duration.as_millis());
            debug!("{:?}", self.stats());
            Some(node_id)
        } else {
            info!("Parse failed.");
            None
        }
    }
    fn gss_nodes(&self) -> impl Iterator<Item = &GSSNode>;

    fn intermediate_nodes_children_map(
        &self,
    ) -> &FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>;

    fn nonterminal_nodes_children_map(&self) -> &FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>;
}

pub fn init_logger() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "{}: {}", record.level(), record.args())
        })
        .init();
}

#[derive(Default, Debug)]
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

/// A unique identifier for a nonterminal in the grammar.
///
/// This is a type-safe wrapper around an index into the grammar's nonterminal list.
/// Uses `u16` since real-world grammars rarely exceed a few hundred nonterminals
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NonterminalId(pub u16);

impl NonterminalId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for NonterminalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl quote::ToTokens for NonterminalId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = self.0;
        tokens.extend(quote::quote! { NonterminalId(#id) });
    }
}

/// A unique identifier for a grammar slot. Grammar slots of of the form A → ⍺ . β, similar
/// to LR items.
///
/// This is a type-safe wrapper around an index into the grammar's grammar slot list.
/// Uses `u16` since real-world grammars rarely exceed a few thousand grammar slots.
#[derive(Debug, Clone, Copy)]
pub struct SlotId(pub u16);

impl SlotId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl quote::ToTokens for SlotId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = self.0;
        tokens.extend(quote::quote! { SlotId(#id) });
    }
}

/// A unique identifier for a terminal in the grammar.
///
/// This is a type-safe wrapper around an index into the grammar's terminal list.
/// Uses `u16` since real-world grammars rarely exceed a few hundred terminals.
#[derive(Debug, Clone, Copy)]
pub struct TerminalId(pub u16);

impl TerminalId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for TerminalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl quote::ToTokens for TerminalId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = self.0;
        tokens.extend(quote::quote! { TerminalId(#id) });
    }
}
