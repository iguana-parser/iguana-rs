use std::sync::LazyLock;
use std::time::Duration;
use web_time::Instant;

use allocator_api2::vec::Vec as AVec;
use bumpalo::Bump;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    descriptor::Descriptor,
    env::{Env, EnvId},
    gss::{GSSEdge, GSSNode},
    ids::{BindingId, GssNodeId, NonterminalId, SlotId, TerminalId},
    input::{Input, Span},
    record,
    sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, TerminalNode},
    utils::inline_vec::InlineVec,
};

#[cfg(feature = "debug-trace")]
use crate::trace::TraceEvent;

/// Initial-capacity parameters for the parser's arena-backed accumulators,
/// applied to `input.len()` in `Parser::new`. Growing a vector inside the
/// arena always copies the whole buffer and abandons the old one, because a
/// bump allocation cannot be extended in place. Reserved-but-untouched pages
/// are never faulted, so over-sizing is nearly free while under-sizing pays
/// that copy on the hot path. Measured on the Java corpus: SPPF nodes reach
/// 2.5-3 per input char (10 on pathological files), GSS nodes stay below 1,
/// envs reach 0.4-0.6 (about 4 on pathological files), and the descriptor
/// queue peaks at 4-23% of the input length.
pub const SPPF_CAPACITY_MULTIPLIER: usize = 8;
pub const GSS_CAPACITY_MULTIPLIER: usize = 2;
pub const ENVS_CAPACITY_MULTIPLIER: usize = 1;
pub const DESCRIPTORS_CAPACITY_DIVISOR: usize = 4;
pub const DESCRIPTORS_CAPACITY_FLOOR: usize = 1024;

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

pub trait Parser<'i, 'arena> {
    /// The unsafe mode runs the parser as if the grammar is unambiguous.
    ///
    /// Static ambiguity detection for the full class of context-free grammars is
    /// undecidable. Iguana detects ambiguities at runtime and returns a parse forest
    /// containing all the derivations. The unsafe mode may silently disambiguate,
    /// returning the first parse tree while in reality there were multiple
    /// derivations. The unsafe mode is only recommended if the user understands its
    /// implications, has a well-tested grammar, and wants better performance.
    ///
    /// The unsafe mode changes the parser behavior as follows:
    /// - The parser stops when it finds the first derivation of the start nonterminal
    ///   that spans the whole input, without exploring the pending descriptors.
    /// - The machinery for detecting and recording ambiguity nodes, e.g., SPPF node
    ///   indexes, is disabled.
    const UNSAFE: bool = false;

    fn nonterminal_display_name(nonterminal_id: NonterminalId) -> &'static str;
    fn terminal_name(terminal_id: TerminalId) -> &'static str;
    fn slot_name(slot_id: SlotId) -> &'static str;
    fn epsilon() -> TerminalId;
    /// The number of terminals, excluding the synthetic Epsilon and EOF.
    fn terminal_count() -> u16;
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
    fn gss_node(&self, id: GssNodeId) -> &GSSNode<'arena>;
    fn sppf_node(&self, id: SPPFNodeId) -> &SPPFNode;
    fn sppf_node_mut(&mut self, id: SPPFNodeId) -> &mut SPPFNode;
    fn gss_node_mut(&mut self, id: GssNodeId) -> &mut GSSNode<'arena>;
    fn add_descriptor(&mut self, descriptor: Descriptor);
    fn add_first_descriptor(
        &mut self,
        slot_id: SlotId,
        input_index: u32,
        gss_node_id: GssNodeId,
        env: Option<EnvId>,
    ) {
        self.add_descriptor(Descriptor::new(
            input_index,
            slot_id,
            None,
            gss_node_id,
            env,
        ));
    }
    fn next_descriptor(&mut self) -> Option<Descriptor>;
    /// The arena backing the parser's internal collections: GSS edges, popped
    /// elements, env bindings, and spilled index maps. Handed to `push`/`insert`
    /// at the spill boundary; reset in bulk after the parse.
    fn vec_arena(&self) -> &'arena Bump;
    /// Empties the descriptor queue. With no descriptors left, `run`'s loop ends and the parse
    /// terminates. `pop` calls it once the start nonterminal spans the full input, but only in
    /// the unsafe mode.
    fn clear_descriptors(&mut self) {}
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
    /// Appends an additional child to an existing nonterminal node.
    /// Called only when a second (or later) derivation of
    /// `(nonterminal_id, span)` is popped. A nonterminal node holds a single
    /// child inline on `NonterminalNode.child`, with that child's slot on
    /// `NonterminalNode.return_slot`. Additional children live in a side map,
    /// and a side map entry on a node marks it ambiguous.
    ///
    /// The unsafe mode produces no ambiguity and records nothing here.
    fn add_nonterminal_node_child(
        &mut self,
        node: SPPFNodeId,
        child: SPPFNodeId,
        return_slot: SlotId,
    ) {
        let _ = (node, child, return_slot);
        if !Self::UNSAFE {
            unimplemented!("overridden by the generated parser outside the unsafe mode");
        }
    }

    /// The extra `(child, return slot)` pairs recorded for `node`, in
    /// insertion order, read without materializing the lazily-built
    /// `nonterminal_nodes_children_map`.
    ///
    /// The unsafe mode records no extra children and returns an empty list.
    fn nonterminal_node_extra_children(&self, node: SPPFNodeId) -> Vec<(SPPFNodeId, SlotId)> {
        let _ = node;
        if Self::UNSAFE {
            Vec::new()
        } else {
            unimplemented!("overridden by the generated parser outside the unsafe mode");
        }
    }

    /// The unsafe mode produces no ambiguity and records nothing here.
    fn add_intermediate_node_child(
        &mut self,
        node: SPPFNodeId,
        child1: SPPFNodeId,
        child2: SPPFNodeId,
    ) {
        let _ = (node, child1, child2);
        if !Self::UNSAFE {
            unimplemented!("overridden by the generated parser outside the unsafe mode");
        }
    }

    /// Looks up the intermediate node for `slot_id`, span (`left_extent`,
    /// `right_extent`), and `env`. Returns `None` if no such node exists.
    ///
    /// The unsafe mode does not share nodes and finds nothing.
    fn lookup_intermediate_node(
        &self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
        env: Option<EnvId>,
    ) -> Option<SPPFNodeId> {
        let _ = (slot_id, left_extent, right_extent, env);
        if Self::UNSAFE {
            None
        } else {
            unimplemented!("overridden by the generated parser outside the unsafe mode");
        }
    }

    /// Looks up an existing terminal node for `terminal_id` and span
    /// (`left_extent`, `right_extent`). Returns `None` if no such node exists.
    ///
    /// The unsafe mode does not share nodes and finds nothing.
    fn lookup_terminal_node(
        &self,
        terminal_id: TerminalId,
        left_extent: u32,
        right_extent: u32,
    ) -> Option<SPPFNodeId> {
        let _ = (terminal_id, left_extent, right_extent);
        if Self::UNSAFE {
            None
        } else {
            unimplemented!("overridden by the generated parser outside the unsafe mode");
        }
    }

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

    /// Formats a parse error into a message and its location. The message names
    /// what was expected, not what was found, and carries no source context; a
    /// caller renders the location as a caret in a terminal or a range in an editor.
    fn format_error(&self, error: &ParseError) -> (u32, u32, String) {
        let (line, column) = self.input().line_column(error.input_index);
        let message = match &error.kind {
            ParseErrorKind::UnexpectedToken { expected } => {
                let names: Vec<_> = expected.iter().map(|t| Self::terminal_name(*t)).collect();
                match names.len() {
                    0 => "Unexpected input".to_string(),
                    1 => format!("Expected {}", names[0]),
                    _ => format!("Expected one of {}", names.join(", ")),
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

    /// Length in characters of the span to highlight for a parse error, marked
    /// by a caret in a terminal or a range in an editor. Iguana is a single-phase
    /// parser, i.e., the scanner is not run before parsing but is driven by the
    /// parser. Therefore, there is no canonical offending token at an error
    /// position. To overcome this, we take the longest match over all terminals
    /// at the error position. The length is at least one character, so a
    /// zero-length or unmatched position still highlights one character.
    fn error_span_len(&mut self, input_index: u32) -> u32 {
        let longest = (0..Self::terminal_count())
            .filter_map(|id| self.match_token(TerminalId(id), input_index))
            .max()
            .unwrap_or(input_index);
        let end = longest.min(self.input().line_end(input_index));
        end.saturating_sub(input_index).max(1)
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
    ) -> Option<(u32, SPPFNodeId)> {
        record!(self, MatchingTerminal, terminal_id, input_index);
        let j = self.match_token(terminal_id, input_index).or_else(|| {
            self.add_parse_error(input_index, slot_id, gss_node_id, || {
                ParseErrorKind::UnexpectedToken {
                    expected: vec![terminal_id],
                }
            });
            None
        })?;
        record!(self, MatchSuccess, terminal_id, input_index, j);
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
        let arena = self.vec_arena();
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
            // Restore the caller's env from the edge and extend it with the
            // callee's return value bound to the variable name, if present.
            let env = match (env, binding, return_value) {
                (Some(env_id), Some(name), Some(return_value)) => {
                    let (new_env_id, new_env) = self.clone_env(env_id);
                    new_env.bind(name, return_value, arena);
                    Some(new_env_id)
                }
                (Some(env_id), _, _) => Some(env_id),
                _ => None,
            };
            if let Some(new_node) = self.merge(left_child, right_child, return_slot, env) {
                self.add_descriptor(Descriptor::new(
                    right_extent,
                    return_slot,
                    Some(new_node),
                    gss_node_id,
                    env,
                ));
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
        binding: Option<BindingId>,
    ) {
        let arena = self.vec_arena();
        let origin = self.gss_node_mut(origin_gss_node_id);
        let gss_edge = GSSEdge::new(result, return_slot, dest_gss_node_id, env, binding);
        origin.add_edge(gss_edge, arena);
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
        let arena = self.vec_arena();
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
        gss.insert_popped_element(right_extent, return_value, nonterminal_node_id, arena);
        let edge_count = gss.edges().len();
        for i in 0..edge_count {
            let edge = *self.gss_node(gss_node_id).edges().get(i).unwrap();
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
            let env = match (edge.env_id(), edge.binding_id(), return_value) {
                (Some(env_id), Some(name), Some(rv)) => {
                    let (new_env_id, env) = self.clone_env(env_id);
                    env.bind(name, rv, arena);
                    Some(new_env_id)
                }
                (Some(env_id), _, _) => Some(env_id),
                _ => None,
            };
            if let Some(new_node_id) = self.merge(left_child, right_child, edge.return_slot, env) {
                self.add_descriptor(Descriptor::new(
                    right_extent,
                    edge.return_slot,
                    Some(new_node_id),
                    edge.dest_id,
                    env,
                ));
            }
        }
        // Stop only when the start node (`GssNodeId(0)`) spans the whole input.
        if Self::UNSAFE && gss_node_id == GssNodeId(0) && right_extent == self.input().len() {
            self.clear_descriptors();
        }
    }

    /// Returns the node id to drive the caller's continuation with. Returns
    /// `None` when the intermediate already exists at `(slot, span, env)`;
    /// the caller skips its `add_descriptor` call.
    fn merge(
        &mut self,
        left_child: Option<(SPPFNodeId, u32)>,
        right_child: (SPPFNodeId, u32),
        slot_id: SlotId,
        env: Option<EnvId>,
    ) -> Option<SPPFNodeId> {
        let (right_child_id, right_extent) = right_child;
        if let Some((left_child_id, left_extent)) = left_child {
            self.get_or_create_intermediate_node(
                slot_id,
                left_extent,
                right_extent,
                left_child_id,
                right_child_id,
                env,
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

    /// Looks up the nonterminal node identified by `nonterminal_id` and the
    /// key `(right_extent, return_value)` in the current GSS node's
    /// popped-elements map. `return_value` is `None` for a plain nonterminal
    /// and `Some(v)` for a data-dependent one that returns `v`, so calls that
    /// return different values get separate nodes. On hit, marks the existing
    /// node ambiguous and attaches `child`. On miss, creates a fresh node. The
    /// unsafe mode skips the lookup and always creates a fresh node.
    ///
    /// Only called in the GLL path. LL(1) parses do not call this function
    /// because an LL(1) nonterminal is unambiguous by definition, so the
    /// ambiguity-attach branch is unreachable.
    #[allow(clippy::too_many_arguments)]
    fn get_or_create_nonterminal_node(
        &mut self,
        nonterminal_id: NonterminalId,
        return_slot: SlotId,
        left_extent: u32,
        right_extent: u32,
        child: SPPFNodeId,
        gss_node_id: GssNodeId,
        return_value: Option<i32>,
    ) -> SPPFNodeId {
        if !Self::UNSAFE {
            if let Some(existing_node_id) = self
                .gss_node(gss_node_id)
                .find_popped_element(right_extent, return_value)
            {
                record!(self, NonterminalNodeFound, existing_node_id);
                let node = self.sppf_node_mut(existing_node_id);
                let SPPFNode::Nonterminal(node) = node else {
                    unreachable!("Expects a nonterminal node");
                };
                node.ambiguous = true;
                self.add_nonterminal_node_child(existing_node_id, child, return_slot);
                return existing_node_id;
            }
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

    /// GLL-path get-or-create for intermediate nodes, keyed by
    /// `(slot_id, span, env)`. The `env` discriminates calls to a
    /// parameterized nonterminal with different parameter values, so that
    /// arrivals from those calls get separate intermediate nodes.
    ///
    /// Returns `Some(new_id)` on a miss. Returns `None` on a hit, which
    /// signals the caller to skip scheduling a descriptor: an earlier
    /// descriptor with the same key already drove the continuation forward.
    /// When the new `(left_child, right_child)` pair differs from the
    /// existing one, the new pair is appended and the node is marked
    /// ambiguous, recording genuine intermediate-level ambiguity. The unsafe
    /// mode skips the lookup and always returns `Some`.
    fn get_or_create_intermediate_node(
        &mut self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
        left_child: SPPFNodeId,
        right_child: SPPFNodeId,
        env: Option<EnvId>,
    ) -> Option<SPPFNodeId> {
        if !Self::UNSAFE {
            if let Some(existing_node_id) =
                self.lookup_intermediate_node(slot_id, left_extent, right_extent, env)
            {
                record!(self, IntermediateNodeFound, existing_node_id);
                let SPPFNode::Intermediate(existing) = self.sppf_node(existing_node_id) else {
                    unreachable!("expected intermediate node");
                };
                if existing.child != (left_child, right_child) {
                    self.add_intermediate_node_child(existing_node_id, left_child, right_child);
                    let SPPFNode::Intermediate(existing) = self.sppf_node_mut(existing_node_id)
                    else {
                        unreachable!("expected intermediate node");
                    };
                    existing.ambiguous = true;
                }
                return None;
            }
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
        Some(self.add_intermediate_node(intermediate_node, env, !Self::UNSAFE))
    }

    /// LL(1)-path intermediate-node creation. Skips the lookup and the index
    /// insert, builds the node, and pushes it onto `sppf_nodes`. LL(1)
    /// intermediate nodes are never queried: the deterministic LL(1) parse
    /// cannot re-enter the same `(slot_id, span)`, and the GLL path never
    /// reads them.
    fn create_intermediate_node_ll1(
        &mut self,
        slot_id: SlotId,
        left_extent: u32,
        right_extent: u32,
        left_child: SPPFNodeId,
        right_child: SPPFNodeId,
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
        self.add_intermediate_node(intermediate_node, None, false)
    }

    /// Combines the left child (result from previous slots) and a right child
    /// into an intermediate node. Returns `Some((right_extent, node_id))` when
    /// a fresh intermediate is created. Returns `None` when an existing
    /// intermediate is found; the caller skips its continuation because the
    /// original descriptor already drove it forward.
    #[inline]
    fn create_intermediate_node(
        &mut self,
        result: Option<SPPFNodeId>,
        right_child_id: SPPFNodeId,
        next_slot_id: SlotId,
        env: Option<EnvId>,
    ) -> Option<(u32, SPPFNodeId)> {
        let right_extent = self.sppf_node(right_child_id).right_extent();
        let left_child_id = result.expect("Result should not be None.");
        let left_extent = self.sppf_node(left_child_id).left_extent();
        self.get_or_create_intermediate_node(
            next_slot_id,
            left_extent,
            right_extent,
            left_child_id,
            right_child_id,
            env,
        )
        .map(|node_id| (right_extent, node_id))
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
            None,
        )
    }

    fn get_or_create_terminal_node(
        &mut self,
        terminal_id: TerminalId,
        left_extent: u32,
        right_extent: u32,
    ) -> SPPFNodeId {
        if !Self::UNSAFE {
            if let Some(existing_node_id) =
                self.lookup_terminal_node(terminal_id, left_extent, right_extent)
            {
                record!(self, TerminalNodeFound, existing_node_id);
                return existing_node_id;
            }
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
        env: Option<EnvId>,
        add_to_index: bool,
    ) -> SPPFNodeId;

    fn add_terminal_node(&mut self, node: TerminalNode) -> SPPFNodeId;

    /// The parse's result: the SPPF node spanning the whole input, or
    /// `None` if no such node exists.
    ///
    /// Popped elements in GSS nodes are keyed by `(right extent, return
    /// value)`, so the start GSS node can hold one full-span node per
    /// return value. For data-dependent nonterminals, where there are
    /// multiple return values, multiple full-span nodes may exist. When
    /// multiple full-span nodes exist, i.e., the parse is ambiguous at
    /// the root, `merge_start_results` merges them into one ambiguous
    /// node.
    fn start_results(
        &mut self,
        right_extent: u32,
        start_gss_node_id: GssNodeId,
    ) -> Option<SPPFNodeId> {
        let arena = self.vec_arena();
        let mut results: InlineVec<'arena, SPPFNodeId> = InlineVec::Empty;
        for (&(right, _), &node_id) in self.gss_node(start_gss_node_id).popped_elements().iter() {
            if right == right_extent {
                results.push(node_id, arena);
            }
        }
        let &first = results.first()?;
        if !Self::UNSAFE && results.len() > 1 {
            Some(self.merge_start_results(&results))
        } else {
            Some(first)
        }
    }

    /// Merges multiple full-span results of the start nonterminal into one
    /// ambiguous node. The merged node collects every derivation of every
    /// result: the first inline, the rest through
    /// `add_nonterminal_node_child`. This is the same shape an ambiguous
    /// nonterminal node takes during the parse, so tree construction needs
    /// no special case.
    fn merge_start_results(&mut self, results: &InlineVec<'arena, SPPFNodeId>) -> SPPFNodeId {
        // A derivation is a (child, return slot) pair: the alternative's
        // body and the end slot naming the alternative.
        let mut derivations: Vec<(SPPFNodeId, SlotId)> = Vec::new();
        for &result in results.iter() {
            let SPPFNode::Nonterminal(node) = self.sppf_node(result) else {
                unreachable!("expected nonterminal node");
            };
            derivations.push((node.child, node.return_slot));
            // The grouped nonterminal_nodes_children_map is built once, on
            // first access. Reading it here would cache it before the
            // additions below, and tree construction would miss them.
            derivations.extend(self.nonterminal_node_extra_children(result));
        }
        let SPPFNode::Nonterminal(first) = self.sppf_node(*results.first().unwrap()) else {
            unreachable!("expected nonterminal node");
        };
        let nonterminal_id = first.nonterminal_id;
        let span = first.span;
        let (child, return_slot) = derivations[0];
        let merged = self.add_nonterminal_node(NonterminalNode {
            nonterminal_id,
            return_slot,
            span,
            child,
            ambiguous: true,
        });
        for &(child, return_slot) in &derivations[1..] {
            self.add_nonterminal_node_child(merged, child, return_slot);
        }
        merged
    }

    fn run(&mut self) -> ParseResult {
        let start = Instant::now();
        let start_input_index = 0;
        let start_nonterminal_id = self.start_nonterminal();
        let start_gss_node_id = self.new_gss_node(start_nonterminal_id, start_input_index);
        // The start node is created before any other GSS node, so its id is
        // `GssNodeId(0)`. `pop` relies on this id to recognize the start node when
        // it terminates an unsafe parse early.
        debug_assert_eq!(start_gss_node_id, GssNodeId(0));
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
                descriptor.sppf_node_id(),
                descriptor.gss_node_id,
                descriptor.env_id(),
            );
        }
        let duration = start.elapsed();
        let right_extent = self.input().len();
        if let Some(sppf_node_id) = self.start_results(right_extent, start_gss_node_id) {
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
    fn gss_nodes<'p>(&'p self) -> impl Iterator<Item = &'p GSSNode<'arena>>
    where
        'arena: 'p;

    /// Extra children of ambiguous intermediate nodes, grouped by parent node.
    ///
    /// The unsafe mode produces no ambiguity and returns an empty map.
    fn intermediate_nodes_children_map(
        &self,
    ) -> &FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>> {
        if Self::UNSAFE {
            static EMPTY: LazyLock<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>> =
                LazyLock::new(FxHashMap::default);
            &EMPTY
        } else {
            unimplemented!("overridden by the generated parser outside the unsafe mode");
        }
    }

    /// Extra children of ambiguous nonterminal nodes, grouped by parent node.
    ///
    /// The unsafe mode produces no ambiguity and returns an empty map.
    fn nonterminal_nodes_children_map(&self) -> &FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SlotId)>> {
        if Self::UNSAFE {
            static EMPTY: LazyLock<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SlotId)>>> =
                LazyLock::new(FxHashMap::default);
            &EMPTY
        } else {
            unimplemented!("overridden by the generated parser outside the unsafe mode");
        }
    }

    /// The children of the given SPPF node, in order.
    fn sppf_children(&self, node_id: SPPFNodeId) -> InlineVec<'arena, SPPFNodeId> {
        let arena = self.vec_arena();
        match self.sppf_node(node_id) {
            SPPFNode::Terminal(_) => InlineVec::Empty,
            SPPFNode::Nonterminal(n) => {
                // The unsafe mode never marks a node ambiguous, so the const
                // guard compiles the ambiguous arm out.
                if !Self::UNSAFE && n.ambiguous {
                    let extras = self.nonterminal_nodes_children_map().get(&node_id).unwrap();
                    let mut children = AVec::with_capacity_in(1 + extras.len(), arena);
                    children.push(n.child);
                    children.extend(extras.iter().map(|(child, _)| *child));
                    InlineVec::Multiple(children)
                } else {
                    InlineVec::Single(n.child)
                }
            }
            SPPFNode::Intermediate(i) => {
                if !Self::UNSAFE && i.ambiguous {
                    let extras = self
                        .intermediate_nodes_children_map()
                        .get(&node_id)
                        .unwrap();
                    // Intermediate node children are kept as `(left, right)` pairs.
                    // So when a node is ambiguous, each extra is another pair, and
                    // the flat child list holds two nodes per pair: the node's own
                    // pair plus two per extra.
                    let mut children = AVec::with_capacity_in(2 + 2 * extras.len(), arena);
                    children.push(i.child.0);
                    children.push(i.child.1);
                    for (left, right) in extras {
                        children.push(*left);
                        children.push(*right);
                    }
                    InlineVec::Multiple(children)
                } else {
                    InlineVec::Pair(i.child.0, i.child.1)
                }
            }
        }
    }

    fn new_env(&mut self) -> (EnvId, &mut Env<'arena>);

    fn clone_env(&mut self, source: EnvId) -> (EnvId, &mut Env<'arena>);

    fn lookup(&self, name: BindingId, env_id: EnvId) -> i32;

    fn envs(&self) -> &[Env<'arena>];

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
