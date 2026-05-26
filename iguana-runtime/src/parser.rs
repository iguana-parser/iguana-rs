use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    descriptor::Descriptor,
    env::{Env, EnvId},
    gss::{GSSEdge, GSSNode},
    ids::{BindingId, GssNodeId, NonterminalId, SlotId, TerminalId},
    input::Input,
    record,
    sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
};

#[cfg(feature = "debug-trace")]
use crate::trace::TraceEvent;

/// Initial-capacity multipliers for the SPPF/GSS accumulators in generated
/// parsers, applied to `input.len()` in `Parser::new` to avoid `Vec` growth
/// on the hot path.
pub const SPPF_CAPACITY_MULTIPLIER: usize = 8;
pub const GSS_CAPACITY_MULTIPLIER: usize = 2;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn start_env(&mut self) -> Option<EnvId>;
    fn lookup_start_nonterminal_node(
        &self,
        right_extent: u32,
        start_gss_node_id: GssNodeId,
    ) -> Option<SPPFNodeId>;
    fn get_gss_node(&self, nonterminal_id: NonterminalId, input_index: u32) -> Option<GssNodeId>;
    fn add_gss_node(
        &mut self,
        nonterminal_id: NonterminalId,
        input_index: u32,
        gss_node_id: GssNodeId,
    );
    fn add_start_gss_node(
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
    fn post_conditions(
        &mut self,
        slot: SlotId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<ParseErrorKind>;
    /// Checks whether the input at the given position is in the follow set of the nonterminal.
    fn follow_set_check(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> bool;
    /// Returns the terminal IDs in the follow set of the given nonterminal.
    fn follow_set_terminals(&self, nonterminal_id: NonterminalId) -> Vec<TerminalId>;

    /// Formats a parse error into a human-readable message.
    fn format_error(&self, error: &ParseError) -> (u32, u32, String) {
        let input = self.input();
        let (line, column) = input.line_column(error.input_index);
        let found = if error.input_index >= input.len() {
            "EOF".to_string()
        } else {
            let ch = input.char_at(error.input_index).unwrap();
            format!("'{ch}'")
        };
        let message = match &error.kind {
            ParseErrorKind::UnexpectedToken { expected } => {
                let names: Vec<_> = expected.iter().map(|t| Self::terminal_name(*t)).collect();
                match names.len() {
                    0 => format!("Unexpected {found}"),
                    1 => format!("Expected {} but found {found}", names[0]),
                    _ => format!("Expected one of {} but found {found}", names.join(", ")),
                }
            }
            ParseErrorKind::ExcludedMatch { excluded_by } => {
                let names: Vec<_> = excluded_by
                    .iter()
                    .map(|t| Self::terminal_name(*t))
                    .collect();
                format!("Match excluded by {}", names.join(", "))
            }
            ParseErrorKind::ForbiddenFollow { forbidden } => {
                let names: Vec<_> = forbidden.iter().map(|t| Self::terminal_name(*t)).collect();
                format!("Forbidden follow: {}", names.join(", "))
            }
        };
        (line, column, message)
    }

    /// Returns the first parse error at the farthest input position, if any.
    fn parse_error(&self) -> Option<&ParseError>;
    /// Records a parse error at the given input position.
    ///
    /// Only errors at the farthest input position seen so far are kept; calls at
    /// strictly lower positions are discarded without invoking `kind`. A higher
    /// position clears the prior level. Taking `kind` as a closure lets call sites
    /// avoid building the `ParseErrorKind` (and its `Vec<TerminalId>`) on the drop
    /// path, which is the common case during GLL parsing.
    fn add_parse_error(
        &mut self,
        input_index: u32,
        slot_id: SlotId,
        gss_node_id: Option<GssNodeId>,
        kind: impl FnOnce() -> ParseErrorKind,
    );
    /// Delegates to the scanner's match_token.
    fn match_token(&mut self, terminal_id: TerminalId, input_index: u32) -> Option<u32>;

    /// Matches a terminal at the given input position.
    /// On success, creates a terminal node and returns the end position and node id.
    /// On failure, records a parse error and returns None.
    fn match_terminal(
        &mut self,
        terminal_id: TerminalId,
        input_index: u32,
        slot_id: SlotId,
        gss_node_id: Option<GssNodeId>,
        terminal_name: &str,
    ) -> Option<(u32, SPPFNodeId)> {
        record!(self, MatchingTerminal, terminal_name, input_index);
        let j = self.match_token(terminal_id, input_index).or_else(|| {
            self.add_parse_error(input_index, slot_id, gss_node_id, || {
                ParseErrorKind::UnexpectedToken {
                    expected: vec![terminal_id],
                }
            });
            None
        })?;
        record!(self, MatchSuccess, terminal_name, input_index, j);
        let node = self.get_or_create_terminal_node(terminal_id, input_index, j);
        Some((j, node))
    }

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
        binding: Option<BindingId>,
    ) {
        let existing_gss_node = self.gss_node(existing_gss_node_id);
        let left_extent = existing_gss_node.index;
        let popped_elements = std::mem::take(
            self.gss_node_mut(existing_gss_node_id)
                .popped_elements_mut(),
        );

        for (&(right_extent, return_value), &nonterminal_node_id) in popped_elements.iter() {
            if let Some(error_kind) = self.post_conditions(return_slot, left_extent, right_extent) {
                self.add_parse_error(
                    right_extent,
                    return_slot,
                    Some(existing_gss_node_id),
                    || error_kind,
                );
                continue;
            }
            let right_child = (nonterminal_node_id, right_extent);
            let new_node = self.merge(left_child, right_child, return_slot);
            // Restore the caller's env from the edge and extend it with the
            // callee's return value bound to the variable name, if present.
            let env = match (env, binding, return_value) {
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
        binding: Option<BindingId>,
    ) {
        let origin = self.gss_node_mut(origin_gss_node_id);
        let gss_edge = GSSEdge::new(result, return_slot, dest_gss_node_id, env, binding);
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
        let right_extent = self.sppf_node(nonterminal_node_id).right_extent();
        record!(
            self,
            Pop,
            gss_node_id,
            slot_id,
            nonterminal_node_id,
            return_value
        );
        let gss = self.gss_node(gss_node_id);
        let nonterminal_id = gss.nonterminal_id;
        if gss.contains_popped_element(right_extent, return_value) {
            record!(self, NodeAlreadyInPoppedElements);
            return;
        }
        let left_extent = gss.index;
        if !self.follow_set_check(nonterminal_id, right_extent) {
            let expected = self.follow_set_terminals(nonterminal_id);
            self.add_parse_error(right_extent, slot_id, Some(gss_node_id), || {
                ParseErrorKind::UnexpectedToken { expected }
            });
            return;
        }
        let right_child = (nonterminal_node_id, right_extent);
        record!(
            self,
            AddToPoppedElements,
            gss_node_id,
            nonterminal_node_id,
            return_value
        );
        let gss = self.gss_node_mut(gss_node_id);
        gss.insert_popped_element(right_extent, return_value, nonterminal_node_id);
        let edge_count = gss.edges().len();
        for i in 0..edge_count {
            let edge = self.gss_node(gss_node_id).edges().get(i).unwrap().clone();
            if let Some(error_kind) =
                self.post_conditions(edge.return_slot, left_extent, right_extent)
            {
                self.add_parse_error(right_extent, edge.return_slot, Some(gss_node_id), || {
                    error_kind
                });
                continue;
            }
            let left_child = edge
                .sppf_node_id()
                .map(|id| (id, self.sppf_node(id).left_extent()));
            let new_node_id = self.merge(left_child, right_child, edge.return_slot);
            let env = match (edge.env_id(), edge.binding_id(), return_value) {
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

    /// Returns the node id to drive the caller's continuation with.
    fn merge(
        &mut self,
        left_child: Option<(SPPFNodeId, u32)>,
        right_child: (SPPFNodeId, u32),
        slot_id: SlotId,
    ) -> SPPFNodeId {
        let (right_child_id, right_extent) = right_child;
        if let Some((left_child_id, left_extent)) = left_child {
            self.get_or_create_intermediate_node(
                slot_id,
                left_extent,
                right_extent,
                left_child_id,
                right_child_id,
                false,
            )
        } else {
            right_child_id
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

    /// Looks up the nonterminal node identified by `nonterminal_id` and the
    /// span `(left_extent, right_extent)` in the current GSS node's
    /// popped-elements map. On hit, marks the existing node ambiguous and
    /// attaches `child`. On miss, creates a fresh node.
    ///
    /// Only called in the GLL path. LL(1) parses do not call this function
    /// because an LL(1) nonterminal is unambiguous by definition, so the
    /// ambiguity-attach branch is unreachable.
    fn get_or_create_nonterminal_node(
        &mut self,
        nonterminal_id: NonterminalId,
        return_slot: SlotId,
        left_extent: u32,
        right_extent: u32,
        child: SPPFNodeId,
        gss_node_id: GssNodeId,
    ) -> SPPFNodeId {
        if let Some(existing_node_id) = self
            .gss_node(gss_node_id)
            .find_popped_element(right_extent, None)
        {
            record!(self, NonterminalNodeFound, existing_node_id);
            let node = self.sppf_node_mut(existing_node_id);
            let SPPFNode::Nonterminal(node) = node else {
                unreachable!("Expects a nonterminal node");
            };
            node.ambiguous = true;
            self.add_nonterminal_node_child(existing_node_id, child);
            return existing_node_id;
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
        self.add_nonterminal_node(nonterminal_node)
    }

    /// Looks up or creates the intermediate node identified by `slot_id` and
    /// the span (`left_extent`, `right_extent`). `is_ll1` selects the path:
    ///
    /// - `false` (GLL): look up the node. If it exists, return the existing
    ///   id and do nothing else: no child attachment, no ambiguity flagging,
    ///   no equality check. Two GSS contexts arriving at the same
    ///   `(slot, span)` under the unified parameterized form produce the
    ///   same packed child pair, so attaching a duplicate or marking the
    ///   node ambiguous would be spurious work that breaks parse tree
    ///   extraction. The new caller's continuation still fires in its own
    ///   GSS context because the descriptor queue tracks per-caller state.
    ///   See `docs/operator_precedence_desugaring.md` section 10. If no
    ///   node exists, build it and insert into `intermediate_nodes_index`.
    /// - `true` (LL(1)): skip the lookup and the index insert. Build the
    ///   node and push it onto `sppf_nodes`. LL(1) intermediate nodes are
    ///   never queried: the deterministic LL(1) parse cannot re-enter the
    ///   same `(slot_id, span)`, and GLL never reads them.
    fn get_or_create_intermediate_node(
        &mut self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
        left_child: SPPFNodeId,
        right_child: SPPFNodeId,
        is_ll1: bool,
    ) -> SPPFNodeId {
        let intermediate_node = IntermediateNode {
            slot_id,
            span: Span {
                left_extent,
                right_extent,
            },
            child: (left_child, right_child),
            ambiguous: false,
        };

        if is_ll1 {
            return self.add_intermediate_node(intermediate_node, false);
        }

        if let Some(existing_node_id) =
            self.lookup_intermediate_node(slot_id, left_extent, right_extent)
        {
            record!(self, IntermediateNodeFound, existing_node_id);
            return existing_node_id;
        }
        self.add_intermediate_node(intermediate_node, true)
    }

    /// Combines the left child (result from previous slots) and a right child into
    /// an intermediate node. Returns the right extent and the node id.
    #[inline]
    fn create_intermediate_node(
        &mut self,
        result: Option<SPPFNodeId>,
        right_child_id: SPPFNodeId,
        next_slot_id: SlotId,
    ) -> (u32, SPPFNodeId) {
        let right_extent = self.sppf_node(right_child_id).right_extent();
        let left_child_id = result.expect("Result should not be None.");
        let left_extent = self.sppf_node(left_child_id).left_extent();
        let new_node = self.get_or_create_intermediate_node(
            next_slot_id,
            left_extent,
            right_extent,
            left_child_id,
            right_child_id,
            false,
        );
        (right_extent, new_node)
    }

    /// Extracts extents from the result node and creates the nonterminal SPPF
    /// node via the GLL get-or-create path.
    #[inline]
    fn create_nonterminal_node(
        &mut self,
        result: Option<SPPFNodeId>,
        nonterminal_id: NonterminalId,
        end_slot_id: SlotId,
        gss_node_id: GssNodeId,
    ) -> SPPFNodeId {
        let result = result.expect("Result should not be None.");
        let node = self.sppf_node(result);
        let left_extent = node.left_extent();
        let right_extent = node.right_extent();
        self.get_or_create_nonterminal_node(
            nonterminal_id,
            end_slot_id,
            left_extent,
            right_extent,
            result,
            gss_node_id,
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

    fn add_intermediate_node(
        &mut self,
        intermediate_node: IntermediateNode,
        add_to_index: bool,
    ) -> SPPFNodeId;

    fn add_terminal_node(&mut self, node: TerminalNode) -> SPPFNodeId;

    fn run(&mut self) -> ParseResult {
        let start = Instant::now();
        let start_input_index = 0;
        let start_nonterminal_id = self.start_nonterminal();
        let start_gss_node_id = self.new_gss_node(start_nonterminal_id, start_input_index);
        let start_env = self.start_env();
        self.add_first_descriptors(
            start_nonterminal_id,
            start_input_index,
            start_gss_node_id,
            start_env,
        );
        self.add_start_gss_node(start_nonterminal_id, start_input_index, start_gss_node_id);
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
            self.lookup_start_nonterminal_node(right_extent, start_gss_node_id)
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
                .map(|((right_extent, _), _)| *right_extent)
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

    fn lookup(&self, name: BindingId, env_id: EnvId) -> i32;

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
