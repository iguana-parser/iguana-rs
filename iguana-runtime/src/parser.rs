use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

use crate::{
    descriptor::Descriptor,
    env::{Env, EnvId},
    gss::{GSSEdge, GSSNode, PoppedElement},
    ids::{GssNodeId, NonterminalId, SlotId, TerminalId},
    input::Input,
    record,
    sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
};

#[cfg(feature = "debug-trace")]
use crate::trace::TraceEvent;

pub enum ParseResult {
    Success(ParseSuccess),
    Failure(ParseError),
}

pub struct ParseSuccess {
    pub sppf_node_id: SPPFNodeId,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub input_index: u32,
    pub slot_id: SlotId,
    pub gss_node_id: Option<GssNodeId>,
    pub kind: ParseErrorKind,
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    /// Terminal match failed: expected one of these terminals at this position.
    UnexpectedToken { expected: Vec<TerminalId> },
    /// Nonterminal except (`\`): matched a nonterminal but it was excluded.
    ExcludedMatch { excluded_by: Vec<TerminalId> },
    /// Follow restriction (`!>>`): the symbol after the match is forbidden.
    ForbiddenFollow { forbidden: Vec<TerminalId> },
}

pub trait Parser<'i> {
    fn nonterminal_display_name(nonterminal_id: NonterminalId) -> &'static str;
    fn terminal_name(terminal_id: TerminalId) -> &'static str;
    fn slot_name(slot_id: SlotId) -> &'static str;
    fn epsilon() -> TerminalId;
    fn eof() -> TerminalId;
    fn execute(
        &mut self,
        input_index: u32,
        slot_id: SlotId,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        env: Option<EnvId>,
    );
    fn add_first_descriptors(
        &mut self,
        nonterminal_id: NonterminalId,
        input_index: u32,
        gss_node_id: GssNodeId,
        env: Option<EnvId>,
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
    fn add_first_descriptor(
        &mut self,
        slot_id: SlotId,
        input_index: u32,
        gss_node_id: GssNodeId,
        env: Option<EnvId>,
    ) {
        self.add_descriptor(Descriptor {
            input_index,
            slot_id,
            sppf_node_id: None,
            gss_node_id,
            env,
        });
    }
    fn next_descriptor(&mut self) -> Option<Descriptor>;
    fn input(&self) -> &'i Input;

    fn sppf_nodes(&self) -> &[SPPFNode];

    #[cfg(feature = "instrument")]
    fn increment_descriptor_count(&mut self);
    #[cfg(feature = "instrument")]
    fn count_descriptors(&self) -> usize;
    #[cfg(feature = "instrument")]
    fn count_gss_nodes(&self) -> usize;
    #[cfg(feature = "instrument")]
    fn count_gss_edges(&self) -> usize;
    #[cfg(feature = "instrument")]
    fn count_nonterminal_nodes(&self) -> usize;
    #[cfg(feature = "instrument")]
    fn count_intermediate_nodes(&self) -> usize;
    #[cfg(feature = "instrument")]
    fn count_terminal_nodes(&self) -> usize;
    #[cfg(feature = "instrument")]
    fn count_ambiguous_nodes(&self) -> usize;
    #[cfg(feature = "instrument")]
    fn record_stats(&self) -> crate::instrument::Stats;
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

    /// Checks whether the post-conditions for a given slot are satisfied.
    /// Called during pop and edge processing to filter results (e.g., nonterminal except).
    /// Returns `None` if the result should be accepted, or `Some(error_kind)` if rejected.
    fn post_conditions(&self, slot: SlotId, left_extent: u32, right_extent: u32) -> Option<ParseErrorKind>;
    /// Checks whether the input at the given position is in the follow set of the nonterminal.
    fn follow_set_check(&self, nonterminal_id: NonterminalId, input_index: u32) -> bool;
    /// Returns the terminal IDs in the follow set of the given nonterminal.
    fn follow_set_terminals(&self, nonterminal_id: NonterminalId) -> Vec<TerminalId>;
    /// Returns the parse error at the farthest input position, if any.
    fn parse_error(&self) -> Option<&ParseError>;
    /// Records a parse error at the given input position.
    fn add_parse_error(&mut self, input_index: u32, slot_id: SlotId, gss_node_id: Option<GssNodeId>, kind: ParseErrorKind);

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
        env: Option<EnvId>,
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
        // If there is already a GSS node for this call, just add the edge
        if let Some(exiting_gss_node_id) = self.get_gss_node(nonterminal_id, i) {
            record!(self, GSSNodeFound, nonterminal_id, i);
            self.add_edge_to_existing_gss_node(
                exiting_gss_node_id,
                gss_node_id,
                left_child,
                return_slot,
                env,
                None,
            );
        } else {
            record!(self, GSSNodeNotFound, nonterminal_id, i);
            let new_gss_node_id = self.new_gss_node(nonterminal_id, i);
            self.add_gss_edge(
                new_gss_node_id,
                gss_node_id,
                sppf_node_id,
                return_slot,
                env,
                None,
            );
            self.add_first_descriptors(nonterminal_id, i, new_gss_node_id, None);
            self.add_gss_node(nonterminal_id, i, new_gss_node_id);
        }
    }

    fn add_edge_to_existing_gss_node(
        &mut self,
        existing_gss_node_id: GssNodeId,
        gss_node_id: GssNodeId,
        left_child: Option<(SPPFNodeId, u32)>,
        return_slot: SlotId,
        env: Option<EnvId>,
        binding: Option<&'static str>,
    ) {
        let existing_gss_node = self.gss_node(existing_gss_node_id);
        let left_extent = existing_gss_node.index;
        let nonterminal_id = existing_gss_node.nonterminal_id;
        let popped_elements = std::mem::take(
            self.gss_node_mut(existing_gss_node_id)
                .popped_elements_mut(),
        );

        // For each popped element of the current GSS node add a descriptor with the return label.
        for popped_element in popped_elements.iter() {
            let popped_node = self.sppf_node(popped_element.nonterminal_node_id);
            let right_extent = popped_node.right_extent();
            if !self.follow_set_check(nonterminal_id, right_extent) {
                let expected = self.follow_set_terminals(nonterminal_id);
                self.add_parse_error(right_extent, return_slot, Some(existing_gss_node_id), ParseErrorKind::UnexpectedToken { expected });
                continue;
            }
            if let Some(error_kind) = self.post_conditions(return_slot, left_extent, right_extent) {
                self.add_parse_error(right_extent, return_slot, Some(existing_gss_node_id), error_kind);
                continue;
            }
            let right_child = (popped_element.nonterminal_node_id, right_extent);
            if let Some(new_node) = self.merge(left_child, right_child, return_slot) {
                // Restore the caller's env from the edge and extend it with the
                // callee's return value bound to the variable name, if present.
                let env = match (env, binding, popped_element.return_value) {
                    (Some(env_id), Some(name), Some(return_value)) => {
                        let (new_env_id, new_env) = self.clone_env(env_id);
                        new_env.bind(name, return_value);
                        Some(new_env_id)
                    }
                    (Some(env_id), _, _) => Some(env_id),
                    _ => None,
                };
                self.add_descriptor(Descriptor {
                    input_index: right_extent,
                    slot_id: return_slot,
                    sppf_node_id: Some(new_node),
                    gss_node_id,
                    env,
                });
            }
        }
        *self
            .gss_node_mut(existing_gss_node_id)
            .popped_elements_mut() = popped_elements;

        self.add_gss_edge(
            existing_gss_node_id,
            gss_node_id,
            left_child.map(|(id, _)| id),
            return_slot,
            env,
            binding,
        );
    }

    fn add_gss_edge(
        &mut self,
        origin_gss_node_id: GssNodeId,
        dest_gss_node_id: GssNodeId,
        result: Option<SPPFNodeId>,
        return_slot: SlotId,
        env: Option<EnvId>,
        binding: Option<&'static str>,
    ) {
        let origin = self.gss_node_mut(origin_gss_node_id);
        let gss_edge = GSSEdge {
            sppf_node_id: result,
            return_slot,
            dest_id: dest_gss_node_id,
            env,
            binding,
        };
        origin.add_edge(gss_edge);
        record!(
            self,
            GSSNodeAdded,
            origin_gss_node_id,
            dest_gss_node_id,
            return_slot
        );
    }

    fn pop(
        &mut self,
        gss_node_id: GssNodeId,
        slot_id: SlotId,
        nonterminal_node_id: SPPFNodeId,
        return_value: Option<i32>,
    ) {
        let popped_element = PoppedElement {
            nonterminal_node_id,
            return_value,
        };
        record!(self, Pop, gss_node_id, slot_id, popped_element);
        let gss = self.gss_node(gss_node_id);
        let nonterminal_id = gss.nonterminal_id;
        if gss.contains_popped_element(&popped_element) {
            record!(self, NodeAlreadyInPoppedElements);
            return;
        }
        let left_extent = gss.index;
        let node = self.sppf_node(popped_element.nonterminal_node_id);
        let right_extent = node.right_extent();
        if !self.follow_set_check(nonterminal_id, right_extent) {
            let expected = self.follow_set_terminals(nonterminal_id);
            self.add_parse_error(right_extent, slot_id, Some(gss_node_id), ParseErrorKind::UnexpectedToken { expected });
            return;
        }
        let right_child = (popped_element.nonterminal_node_id, right_extent);
        record!(self, AddToPoppedElements, gss_node_id, popped_element);
        let gss = self.gss_node_mut(gss_node_id);
        gss.add_to_popped_elements(popped_element);
        let edges = gss.edges().clone();
        for edge in edges.iter() {
            if let Some(error_kind) = self.post_conditions(edge.return_slot, left_extent, right_extent) {
                self.add_parse_error(right_extent, edge.return_slot, Some(gss_node_id), error_kind);
                continue;
            }
            let left_child = edge
                .sppf_node_id
                .map(|id| (id, self.sppf_node(id).left_extent()));
            if let Some(new_node_id) = self.merge(left_child, right_child, edge.return_slot) {
                let env = match (edge.env, edge.binding, popped_element.return_value) {
                    (Some(env_id), Some(name), Some(rv)) => {
                        let (new_env_id, env) = self.clone_env(env_id);
                        env.bind(name, rv);
                        Some(new_env_id)
                    }
                    (Some(env_id), _, _) => Some(env_id),
                    _ => None,
                };
                self.add_descriptor(Descriptor {
                    input_index: right_extent,
                    slot_id: edge.return_slot,
                    sppf_node_id: Some(new_node_id),
                    gss_node_id: edge.dest_id,
                    env,
                });
            }
        }
    }

    /// Returns None if the intermediate node already exists.
    /// A new descriptor should only be added when merge returns Some(n).
    fn merge(
        &mut self,
        left_child: Option<(SPPFNodeId, u32)>,
        right_child: (SPPFNodeId, u32),
        slot_id: SlotId,
    ) -> Option<SPPFNodeId> {
        let (right_child_id, right_extent) = right_child;
        if let Some((left_child_id, left_extent)) = left_child {
            self.get_or_create_intermediate_node(
                slot_id,
                left_extent,
                right_extent,
                left_child_id,
                right_child_id,
                true,
            )
        } else {
            Some(right_child_id)
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
    /// If the node already exists and `attach_ambiguity` is true (GLL path),
    /// `child` is added to its list of children and returns None. This only
    /// occurs when there is an ambiguity.
    ///
    /// If `attach_ambiguity` is false (LL1 path), the existing node is returned
    /// as-is. LL1 nonterminals can be called multiple times from GLL with the
    /// same input position, producing identical children, so attaching would
    /// create false ambiguities.
    fn get_or_create_nonterminal_node(
        &mut self,
        nonterminal_id: NonterminalId,
        return_slot: SlotId,
        left_extent: u32,
        right_extent: u32,
        child: SPPFNodeId,
        attach_ambiguity: bool,
    ) -> Option<SPPFNodeId> {
        if let Some(existing_node_id) =
            self.lookup_nonterminal_node(nonterminal_id, left_extent, right_extent)
        {
            if attach_ambiguity {
                record!(self, NonterminalNodeFound, existing_node_id);
                let node = self.sppf_node_mut(existing_node_id);
                let SPPFNode::Nonterminal(node) = node else {
                    unreachable!("Expects a nonterminal node");
                };
                node.ambiguous = true;
                self.add_nonterminal_node_child(existing_node_id, child);
                return None;
            }
            return Some(existing_node_id);
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
    /// If the node already exists and `attach_ambiguity` is true (GLL path),
    /// `(left_child, right_child)` is added to its list of children and returns
    /// None. This only occurs when there is an ambiguity.
    ///
    /// If `attach_ambiguity` is false (LL1 path), the existing node is returned
    /// as-is. LL1 nonterminals can be called multiple times from GLL with the
    /// same input position, producing identical children, so attaching would
    /// create false ambiguities.
    fn get_or_create_intermediate_node(
        &mut self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
        left_child: SPPFNodeId,
        right_child: SPPFNodeId,
        attach_ambiguity: bool,
    ) -> Option<SPPFNodeId> {
        if let Some(existing_node_id) =
            self.lookup_intermediate_node(slot_id, left_extent, right_extent)
        {
            if attach_ambiguity {
                record!(self, IntermediateNodeFound, existing_node_id);
                let SPPFNode::Intermediate(node) = self.sppf_node_mut(existing_node_id) else {
                    unreachable!("It's a nonterminal node");
                };
                node.ambiguous = true;
                self.add_intermediate_node_child(existing_node_id, left_child, right_child);
                return None;
            }
            return Some(existing_node_id);
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

    /// Combines the left child (result from previous slots) and a right child into
    /// an intermediate node. Returns the right extent and new node id, or None if
    /// the node already existed and ambiguity was recorded.
    #[inline]
    fn create_intermediate_node(
        &mut self,
        result: Option<SPPFNodeId>,
        right_child_id: SPPFNodeId,
        next_slot_id: SlotId,
    ) -> Option<(u32, SPPFNodeId)> {
        let right_extent = self.sppf_node(right_child_id).right_extent();
        let left_child_id = result.expect("Result should not be None.");
        let left_extent = self.sppf_node(left_child_id).left_extent();
        self.get_or_create_intermediate_node(
            next_slot_id, left_extent, right_extent, left_child_id, right_child_id, true,
        )
        .map(|new_node| (right_extent, new_node))
    }

    /// Extracts extents from the result node and creates the nonterminal SPPF
    /// node, handling ambiguity. Returns the nonterminal node id, or None if the
    /// node already existed and ambiguity was recorded.
    #[inline]
    fn create_nonterminal_node(
        &mut self,
        result: Option<SPPFNodeId>,
        nonterminal_id: NonterminalId,
        end_slot_id: SlotId,
    ) -> Option<SPPFNodeId> {
        let result = result.expect("Result should not be None.");
        let node = self.sppf_node(result);
        let left_extent = node.left_extent();
        let right_extent = node.right_extent();
        self.get_or_create_nonterminal_node(
            nonterminal_id, end_slot_id, left_extent, right_extent, result, true,
        )
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
        self.add_first_descriptors(
            start_nonterminal_id,
            start_input_index,
            start_gss_node_id,
            None,
        );
        self.add_gss_node(start_nonterminal_id, start_input_index, start_gss_node_id);
        while let Some(descriptor) = self.next_descriptor() {
            self.execute(
                descriptor.input_index,
                descriptor.slot_id,
                descriptor.sppf_node_id,
                descriptor.gss_node_id,
                descriptor.env,
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
            })
        } else if let Some(error) = self.parse_error() {
            ParseResult::Failure(error.clone())
        } else {
            // No error was recorded, but the start nonterminal doesn't span the full input.
            // Find the farthest position reached by the start nonterminal.
            let start_gss = self.gss_node(start_gss_node_id);
            let farthest = start_gss
                .popped_elements()
                .iter()
                .map(|pe| self.sppf_node(pe.nonterminal_node_id).right_extent())
                .max()
                .unwrap_or(0);
            ParseResult::Failure(ParseError {
                input_index: farthest,
                slot_id: SlotId(0),
                gss_node_id: Some(start_gss_node_id),
                kind: ParseErrorKind::UnexpectedToken { expected: vec![] },
            })
        }
    }
    fn gss_nodes(&self) -> impl Iterator<Item = &GSSNode>;

    fn intermediate_nodes_children_map(
        &self,
    ) -> &FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>;

    fn nonterminal_nodes_children_map(&self) -> &FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>;

    fn new_env(&mut self) -> (EnvId, &mut Env);

    fn clone_env(&mut self, source: EnvId) -> (EnvId, &mut Env);

    fn lookup(&self, name: &str, env_id: EnvId) -> i32;

    fn envs(&self) -> &[Env];

    #[cfg(feature = "debug-trace")]
    fn add_trace_event(&mut self, event: TraceEvent);
}

pub fn init_logger() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = env_logger::Builder::from_default_env()
            .format(|buf, record| {
                use std::io::Write;
                writeln!(buf, "{}: {}", record.level(), record.args())
            })
            .try_init();
    });
}

