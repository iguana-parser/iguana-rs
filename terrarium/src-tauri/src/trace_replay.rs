use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use iguana::descriptor::Descriptor;
use iguana::gss::{GSSEdge, GSSNode};
use iguana::ids::{GssNodeId, NonterminalId, SlotId};
use iguana::sppf::SPPFNodeId;
use iguana::trace::TraceEvent;
use serde::{Deserialize, Serialize};
use specta::Type;

/// The type of parsing action (step) being displayed.
#[derive(Debug, Clone)]
pub enum DebugAction {
    /// Processing a descriptor from the worklist
    ProcessingDescriptor(Descriptor),
    /// Pop action: completed a rule, creating a nonterminal node
    Pop {
        slot_id: SlotId,
        gss_node_id: GssNodeId,
        sppf_node_id: SPPFNodeId,
    },
    /// Match failed: terminal didn't match at input position
    MatchFailed {
        terminal_name: String,
        input_index: u32,
        slot_id: SlotId,
        gss_node_id: GssNodeId,
        sppf_node_id: Option<SPPFNodeId>,
    },
    /// Matching leading layout (whitespace before token)
    MatchingLeadingLayout { input_index: u32 },
    /// Matching trailing layout (whitespace after token)
    MatchingTrailingLayout { input_index: u32 },
    /// Layout matched successfully
    MatchedLayout { next_index: Option<u32> },
    /// Attempting to match a terminal
    MatchingTerminal { terminal_name: String, input_index: u32 },
    /// Terminal matched successfully
    MatchSuccess { terminal_name: String, input_index: u32, next_index: u32 },
}

/// SPPF node for debug visualization, reconstructed incrementally from trace events.
///
/// This is separate from iguana's `SPPFDotNode` because debug mode replays trace events
/// step-by-step and builds the SPPF incrementally. Each step may add new nodes, so we
/// need the `children` field to track parent-child relationships as they're discovered.
/// In contrast, `SPPFDotNode` is built by traversing the completed SPPF tree after parsing,
/// where edges are stored separately.
#[derive(Debug, Clone, Serialize, Type)]
pub struct DebugSPPFNode {
    pub id: u32,
    pub kind: DebugSPPFNodeKind,
    pub label: String,
    pub left_extent: u32,
    pub right_extent: u32,
    pub children: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub enum DebugSPPFNodeKind {
    Terminal,
    Nonterminal,
    Intermediate,
}

/// GSS node for debug visualization.
#[derive(Debug, Clone, Serialize, Type)]
pub struct DebugGSSNode {
    pub id: u32,
    /// Format: "(Nonterminal, InputIndex)"
    pub label: String,
}

/// GSS edge for debug visualization.
#[derive(Debug, Clone, Serialize, Type)]
pub struct DebugGSSEdge {
    pub src: u32,
    pub dest: u32,
    /// Return slot name
    pub label: String,
}

/// Debug GSS info returned to the frontend.
#[derive(Debug, Clone, Serialize, Type)]
pub struct DebugGSSInfo {
    pub nodes: Vec<DebugGSSNode>,
    pub edges: Vec<DebugGSSEdge>,
    /// The current GSS node ID from the action being processed
    pub current_gss_node_id: Option<u32>,
}

/// Error info for the dropdown list.
#[derive(Debug, Clone, Serialize, Type)]
pub struct ErrorInfo {
    /// Step index (0-indexed into step_indices)
    pub step_index: u32,
    /// Position in the input where the error occurred
    pub input_index: u32,
    /// Name of the terminal that failed to match
    pub terminal_name: String,
}

/// Entry in the event log.
#[derive(Debug, Clone, Serialize, Type)]
pub struct EventLogEntry {
    /// Index of this event in the trace
    pub event_index: u32,
    /// Step index if this event is steppable, None otherwise
    pub step_index: Option<u32>,
    /// Formatted message for this event
    pub message: String,
    /// Event type for styling (e.g., "processing", "match_success", "match_failed", "gss", "sppf", "layout")
    pub event_type: String,
}

/// Symbol table loaded from `--write-symbols` output.
/// Array indices correspond to IDs.
#[derive(Debug, Deserialize, Type)]
pub struct SymbolTable {
    pub nonterminals: Vec<String>,
    pub terminals: Vec<String>,
    pub slots: Vec<String>,
}

impl SymbolTable {
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let table = serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(table)
    }

    pub fn nonterminal(&self, id: NonterminalId) -> &str {
        &self.nonterminals[id.index()]
    }

    pub fn terminal(&self, id: &iguana::ids::TerminalId) -> String {
        self.terminals
            .get(id.index())
            .cloned()
            .unwrap_or_else(|| format!("T{}", id.index()))
    }

    pub fn slot(&self, id: SlotId) -> &str {
        &self.slots[id.index()]
    }
}

/// Trace replay for debugging.
/// Steps are ProcessingDescriptor and Pop events.
pub struct TraceReplay {
    events: Vec<TraceEvent>,
    /// Indices into `events` that are steps (ProcessingDescriptor or Pop)
    step_indices: Vec<usize>,
    /// Indices into `step_indices` that are error steps (MatchFailed)
    error_step_indices: Vec<usize>,
    /// Current step (0-indexed into step_indices)
    current_step: usize,
    /// GSS nodes reconstructed from trace
    gss_nodes: Vec<GSSNode>,
    /// Pending descriptor set (uses iguana's Descriptor with IDs)
    descriptor_set: Vec<Descriptor>,
    /// Current action being displayed (ProcessingDescriptor or Pop)
    current_action: Option<DebugAction>,
    symbols: SymbolTable,
    /// SPPF nodes reconstructed from trace
    sppf_nodes: Vec<DebugSPPFNode>,
    /// Current SPPF node ID from the action being processed
    current_sppf_node_id: Option<u32>,
}

impl TraceReplay {
    /// Load trace events and symbol table from JSON files.
    pub fn load(
        trace_path: impl AsRef<Path>,
        symbols_path: impl AsRef<Path>,
    ) -> std::io::Result<Self> {
        let file = File::open(trace_path)?;
        let reader = BufReader::new(file);
        let events: Vec<TraceEvent> = serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let symbols = SymbolTable::load(symbols_path)?;

        // Build index of step events (ProcessingDescriptor, Pop, MatchFailed, layout events, and terminal matching)
        let step_indices: Vec<usize> = events
            .iter()
            .enumerate()
            .filter_map(|(i, event)| match event {
                TraceEvent::ProcessingDescriptor(..)
                | TraceEvent::Pop(..)
                | TraceEvent::MatchFailed(..)
                | TraceEvent::MatchingLeadingLayout(..)
                | TraceEvent::MatchingTrailingLayout(..)
                | TraceEvent::MatchedLayout(..)
                | TraceEvent::MatchingTerminal(..)
                | TraceEvent::MatchSuccess(..) => Some(i),
                _ => None,
            })
            .collect();

        // Build index of error steps (indices into step_indices where event is MatchFailed)
        let error_step_indices: Vec<usize> = step_indices
            .iter()
            .enumerate()
            .filter_map(|(step_idx, &event_idx)| {
                if matches!(events[event_idx], TraceEvent::MatchFailed(..)) {
                    Some(step_idx)
                } else {
                    None
                }
            })
            .collect();

        let mut replay = Self {
            events,
            step_indices,
            error_step_indices,
            current_step: 0,
            gss_nodes: Vec::new(),
            descriptor_set: Vec::new(),
            current_action: None,
            symbols,
            sppf_nodes: Vec::new(),
            current_sppf_node_id: None,
        };

        // Apply events up to and including the first step
        if !replay.step_indices.is_empty() {
            replay.apply_events_up_to(replay.step_indices[0]);
        }

        Ok(replay)
    }

    /// Total number of steps.
    pub fn total_steps(&self) -> usize {
        self.step_indices.len()
    }

    /// Current step (0-indexed).
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Total number of error steps.
    pub fn total_errors(&self) -> usize {
        self.error_step_indices.len()
    }

    /// Get the current error index (1-indexed) if at an error, or None.
    /// Also returns the index of the nearest error for display purposes.
    pub fn current_error_index(&self) -> Option<usize> {
        self.error_step_indices
            .iter()
            .position(|&step| step == self.current_step)
            .map(|i| i + 1) // Convert to 1-indexed
    }

    /// Get the step index of the first error, if any.
    pub fn first_error_step(&self) -> Option<usize> {
        self.error_step_indices.first().copied()
    }

    /// Get the step index of the last error, if any.
    pub fn last_error_step(&self) -> Option<usize> {
        self.error_step_indices.last().copied()
    }

    /// Get the step index of the next error after current step, if any.
    pub fn next_error_step(&self) -> Option<usize> {
        self.error_step_indices
            .iter()
            .find(|&&step| step > self.current_step)
            .copied()
    }

    /// Get the step index of the previous error before current step, if any.
    pub fn prev_error_step(&self) -> Option<usize> {
        self.error_step_indices
            .iter()
            .rev()
            .find(|&&step| step < self.current_step)
            .copied()
    }

    /// Get the step index of the error with the largest input index (furthest progress).
    pub fn furthest_error_step(&self) -> Option<usize> {
        self.error_step_indices
            .iter()
            .max_by_key(|&&step_idx| {
                let event_idx = self.step_indices[step_idx];
                if let TraceEvent::MatchFailed(_, input_index, ..) = &self.events[event_idx] {
                    *input_index
                } else {
                    0
                }
            })
            .copied()
    }

    /// Get all errors sorted by input index (descending) for the dropdown.
    pub fn get_errors_list(&self) -> Vec<ErrorInfo> {
        let mut errors: Vec<ErrorInfo> = self
            .error_step_indices
            .iter()
            .filter_map(|&step_idx| {
                let event_idx = self.step_indices[step_idx];
                if let TraceEvent::MatchFailed(terminal_name, input_index, ..) =
                    &self.events[event_idx]
                {
                    Some(ErrorInfo {
                        step_index: step_idx as u32,
                        input_index: *input_index,
                        terminal_name: terminal_name.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by input_index descending (furthest first)
        errors.sort_by(|a, b| b.input_index.cmp(&a.input_index));
        errors
    }

    /// Format a GSS node as "(Nonterminal, InputIndex)".
    fn format_gss_node(&self, gss_node_id: GssNodeId) -> String {
        if let Some(node) = self.gss_nodes.get(gss_node_id.index()) {
            let nt_name = self.symbols.nonterminal(node.nonterminal_id);
            format!("({}, {})", nt_name, node.index)
        } else {
            format!("(?, {})", gss_node_id.0)
        }
    }

    /// Format an SPPF node as "(label, left, right)".
    fn format_sppf_node(&self, sppf_node_id: u32) -> String {
        if let Some(node) = self.sppf_nodes.get(sppf_node_id as usize) {
            format!(
                "({}, {}, {})",
                node.label, node.left_extent, node.right_extent
            )
        } else {
            format!("(?, {})", sppf_node_id)
        }
    }

    /// Format a descriptor compactly (for pending descriptors list).
    fn format_descriptor_compact(&self, desc: &Descriptor) -> String {
        let slot_name = self.symbols.slot(desc.slot_id);
        let gss_node = self.format_gss_node(desc.gss_node_id);
        let sppf_node = match desc.sppf_node_id {
            Some(id) => self.format_sppf_node(id.0),
            None => "$".to_string(),
        };
        format!(
            "({}, {}, {}, {})",
            slot_name, desc.input_index, gss_node, sppf_node
        )
    }

    /// Format a descriptor for multi-line display (for current action).
    fn format_descriptor(&self, desc: &Descriptor) -> String {
        let slot_name = self.symbols.slot(desc.slot_id);
        let gss_node = self.format_gss_node(desc.gss_node_id);
        let sppf_node = match desc.sppf_node_id {
            Some(id) => self.format_sppf_node(id.0),
            None => "$".to_string(),
        };
        format!(
            "Processing\n  {}\n  input index {}\n  GSS node {}\n  SPPF node {}",
            slot_name, desc.input_index, gss_node, sppf_node
        )
    }

    /// Format a Pop action for multi-line display.
    fn format_pop(
        &self,
        slot_id: SlotId,
        gss_node_id: GssNodeId,
        sppf_node_id: SPPFNodeId,
    ) -> String {
        let slot_name = self.symbols.slot(slot_id);
        let gss_node = self.format_gss_node(gss_node_id);
        let sppf_node = self.format_sppf_node(sppf_node_id.0);
        format!(
            "Popped\n  GSS node {}\n  {}\n  with SPPF node {}",
            gss_node, slot_name, sppf_node
        )
    }

    /// Format a MatchFailed action for multi-line display.
    fn format_match_failed(
        &self,
        terminal_name: &str,
        input_index: u32,
        slot_id: SlotId,
        gss_node_id: GssNodeId,
        sppf_node_id: Option<SPPFNodeId>,
    ) -> String {
        let slot_name = self.symbols.slot(slot_id);
        let gss_node = self.format_gss_node(gss_node_id);
        let sppf_node = match sppf_node_id {
            Some(id) => self.format_sppf_node(id.0),
            None => "$".to_string(),
        };
        format!(
            "Match Failed\n  terminal '{}'\n  {}\n  input index {}\n  GSS node {}\n  SPPF node {}",
            terminal_name, slot_name, input_index, gss_node, sppf_node
        )
    }

    /// Get the current action as a formatted string.
    pub fn current_action_string(&self) -> Option<String> {
        match &self.current_action {
            Some(DebugAction::ProcessingDescriptor(desc)) => Some(self.format_descriptor(desc)),
            Some(DebugAction::Pop {
                slot_id,
                gss_node_id,
                sppf_node_id,
            }) => Some(self.format_pop(*slot_id, *gss_node_id, *sppf_node_id)),
            Some(DebugAction::MatchFailed {
                terminal_name,
                input_index,
                slot_id,
                gss_node_id,
                sppf_node_id,
            }) => Some(self.format_match_failed(
                terminal_name,
                *input_index,
                *slot_id,
                *gss_node_id,
                *sppf_node_id,
            )),
            Some(DebugAction::MatchingLeadingLayout { input_index }) => {
                Some(format!("Matching Leading Layout\n  input index {}", input_index))
            }
            Some(DebugAction::MatchingTrailingLayout { input_index }) => {
                Some(format!("Matching Trailing Layout\n  input index {}", input_index))
            }
            Some(DebugAction::MatchedLayout { next_index }) => {
                if let Some(next) = next_index {
                    Some(format!("Matched Layout\n  new input index {}", next))
                } else {
                    Some("Matched Layout\n  no layout found".to_string())
                }
            }
            Some(DebugAction::MatchingTerminal { terminal_name, input_index }) => {
                Some(format!("Matching Terminal\n  '{}'\n  input index {}", terminal_name, input_index))
            }
            Some(DebugAction::MatchSuccess { terminal_name, input_index, next_index }) => {
                Some(format!(
                    "Match Success\n  '{}'\n  input index {}\n  match length {}",
                    terminal_name, input_index, next_index - input_index
                ))
            }
            None => None,
        }
    }

    /// Get the current input index (position in the input being parsed).
    pub fn current_input_index(&self) -> Option<usize> {
        match &self.current_action {
            Some(DebugAction::ProcessingDescriptor(desc)) => Some(desc.input_index as usize),
            Some(DebugAction::Pop { gss_node_id, .. }) => {
                // For Pop, get the input index from the GSS node
                self.gss_nodes
                    .get(gss_node_id.index())
                    .map(|node| node.index as usize)
            }
            Some(DebugAction::MatchFailed { input_index, .. }) => Some(*input_index as usize),
            Some(DebugAction::MatchingLeadingLayout { input_index }) => Some(*input_index as usize),
            Some(DebugAction::MatchingTrailingLayout { input_index }) => Some(*input_index as usize),
            Some(DebugAction::MatchedLayout { next_index }) => next_index.map(|i| i as usize),
            Some(DebugAction::MatchingTerminal { input_index, .. }) => Some(*input_index as usize),
            Some(DebugAction::MatchSuccess { next_index, .. }) => Some(*next_index as usize),
            None => None,
        }
    }

    /// Get the current SPPF nodes for visualization.
    pub fn sppf_nodes(&self) -> &[DebugSPPFNode] {
        &self.sppf_nodes
    }

    /// Get the current SPPF node ID from the descriptor being processed.
    pub fn current_sppf_node_id(&self) -> Option<u32> {
        self.current_sppf_node_id
    }

    /// Get the pending descriptor set as formatted strings (compact format).
    pub fn descriptor_set_strings(&self) -> Vec<String> {
        self.descriptor_set
            .iter()
            .map(|desc| self.format_descriptor_compact(desc))
            .collect()
    }

    /// Step forward to the next step.
    /// Returns true if stepped, false if at end.
    pub fn step_forward(&mut self) -> bool {
        if self.current_step + 1 >= self.step_indices.len() {
            return false;
        }

        self.current_step += 1;
        let target_event_index = self.step_indices[self.current_step];

        // Apply all events from the previous step's event up to the new step's event
        let prev_event_index = self.step_indices[self.current_step - 1];
        for i in (prev_event_index + 1)..=target_event_index {
            self.apply_event(i);
        }

        true
    }

    /// Step to a specific step index.
    pub fn step_to(&mut self, target_step: usize) {
        let target_step = target_step.min(self.step_indices.len().saturating_sub(1));

        if target_step <= self.current_step {
            // Need to rebuild from scratch
            self.gss_nodes.clear();
            self.descriptor_set.clear();
            self.current_action = None;
            self.sppf_nodes.clear();
            self.current_sppf_node_id = None;
            self.current_step = 0;

            if !self.step_indices.is_empty() {
                self.apply_events_up_to(self.step_indices[0]);
            }
        }

        // Step forward to target
        while self.current_step < target_step {
            if !self.step_forward() {
                break;
            }
        }
    }

    /// Apply all events from index 0 up to and including `end_index`.
    fn apply_events_up_to(&mut self, end_index: usize) {
        for i in 0..=end_index {
            self.apply_event(i);
        }
    }

    /// Apply the trace event at the given index to update state.
    fn apply_event(&mut self, index: usize) {
        match &self.events[index] {
            TraceEvent::ProcessingDescriptor(slot_id, input_index, gss_node_id, sppf_node_id) => {
                let desc = Descriptor {
                    slot_id: *slot_id,
                    input_index: *input_index,
                    gss_node_id: *gss_node_id,
                    sppf_node_id: *sppf_node_id,
                    env: None,
                };
                // Remove from pending set (if present)
                self.descriptor_set.retain(|d| {
                    !(d.slot_id == *slot_id
                        && d.input_index == *input_index
                        && d.gss_node_id == *gss_node_id)
                });
                self.current_action = Some(DebugAction::ProcessingDescriptor(desc));
                // Track the current SPPF node from the descriptor
                self.current_sppf_node_id = sppf_node_id.map(|id| id.0);
            }
            TraceEvent::Pop(gss_node_id, slot_id, sppf_node_id) => {
                self.current_action = Some(DebugAction::Pop {
                    slot_id: *slot_id,
                    gss_node_id: *gss_node_id,
                    sppf_node_id: *sppf_node_id,
                });
                // Track the SPPF node from the Pop
                self.current_sppf_node_id = Some(sppf_node_id.0);
            }
            TraceEvent::DescriptorAdded(slot_id, input_index, gss_node_id, sppf_node_id) => {
                self.descriptor_set.push(Descriptor {
                    slot_id: *slot_id,
                    input_index: *input_index,
                    gss_node_id: *gss_node_id,
                    sppf_node_id: *sppf_node_id,
                    env: None,
                });
            }
            TraceEvent::GSSNodeCreated(nonterminal_id, input_index) => {
                let id = GssNodeId(self.gss_nodes.len() as u32);
                self.gss_nodes
                    .push(GSSNode::new(id, *nonterminal_id, *input_index));
            }
            TraceEvent::GSSNodeAdded(src_id, dest_id, return_slot) => {
                if let Some(node) = self.gss_nodes.get_mut(src_id.index()) {
                    node.add_edge(GSSEdge {
                        sppf_node_id: None,
                        return_slot: *return_slot,
                        dest_id: *dest_id,
                        env: None,
                    });
                }
            }
            TraceEvent::TerminalNodeCreated(terminal_id, span) => {
                let id = self.sppf_nodes.len() as u32;
                let label = self.symbols.terminal(terminal_id);
                self.sppf_nodes.push(DebugSPPFNode {
                    id,
                    kind: DebugSPPFNodeKind::Terminal,
                    label,
                    left_extent: span.left_extent,
                    right_extent: span.right_extent,
                    children: vec![],
                });
            }
            TraceEvent::IntermediateNodeCreated(slot_id, span, left_child, right_child) => {
                let id = self.sppf_nodes.len() as u32;
                let label = self.symbols.slot(*slot_id).to_string();
                self.sppf_nodes.push(DebugSPPFNode {
                    id,
                    kind: DebugSPPFNodeKind::Intermediate,
                    label,
                    left_extent: span.left_extent,
                    right_extent: span.right_extent,
                    children: vec![left_child.0, right_child.0],
                });
            }
            TraceEvent::NonterminalNodeCreated(nonterminal_id, span, child) => {
                let id = self.sppf_nodes.len() as u32;
                let label = self.symbols.nonterminal(*nonterminal_id).to_string();
                self.sppf_nodes.push(DebugSPPFNode {
                    id,
                    kind: DebugSPPFNodeKind::Nonterminal,
                    label,
                    left_extent: span.left_extent,
                    right_extent: span.right_extent,
                    children: vec![child.0],
                });
            }
            TraceEvent::MatchFailed(
                terminal_name,
                input_index,
                slot_id,
                gss_node_id,
                sppf_node_id,
            ) => {
                self.current_action = Some(DebugAction::MatchFailed {
                    terminal_name: terminal_name.clone(),
                    input_index: *input_index,
                    slot_id: *slot_id,
                    gss_node_id: *gss_node_id,
                    sppf_node_id: *sppf_node_id,
                });
                // Update current SPPF node from the context
                self.current_sppf_node_id = sppf_node_id.map(|id| id.0);
            }
            TraceEvent::MatchingLeadingLayout(input_index) => {
                self.current_action = Some(DebugAction::MatchingLeadingLayout {
                    input_index: *input_index,
                });
            }
            TraceEvent::MatchingTrailingLayout(input_index) => {
                self.current_action = Some(DebugAction::MatchingTrailingLayout {
                    input_index: *input_index,
                });
            }
            TraceEvent::MatchedLayout(next_index) => {
                self.current_action = Some(DebugAction::MatchedLayout {
                    next_index: *next_index,
                });
            }
            TraceEvent::MatchingTerminal(terminal_name, input_index) => {
                self.current_action = Some(DebugAction::MatchingTerminal {
                    terminal_name: terminal_name.clone(),
                    input_index: *input_index,
                });
            }
            TraceEvent::MatchSuccess(terminal_name, input_index, next_index) => {
                self.current_action = Some(DebugAction::MatchSuccess {
                    terminal_name: terminal_name.clone(),
                    input_index: *input_index,
                    next_index: *next_index,
                });
            }
            _ => {}
        }
    }

    /// Build GSS info for debug visualization.
    pub fn get_debug_gss_info(&self) -> DebugGSSInfo {
        let nodes: Vec<DebugGSSNode> = self
            .gss_nodes
            .iter()
            .map(|node| DebugGSSNode {
                id: node.id.0,
                label: self.format_gss_node(node.id),
            })
            .collect();

        let mut edges = Vec::new();
        for gss_node in &self.gss_nodes {
            for edge in gss_node.edges() {
                edges.push(DebugGSSEdge {
                    src: gss_node.id.0,
                    dest: edge.dest_id.0,
                    label: self.symbols.slot(edge.return_slot).to_string(),
                });
            }
        }

        // Get current GSS node from the current action
        let current_gss_node_id = match &self.current_action {
            Some(DebugAction::ProcessingDescriptor(desc)) => Some(desc.gss_node_id.0),
            Some(DebugAction::Pop { gss_node_id, .. }) => Some(gss_node_id.0),
            Some(DebugAction::MatchFailed { gss_node_id, .. }) => Some(gss_node_id.0),
            // Layout and terminal matching actions don't have an associated GSS node
            Some(DebugAction::MatchingLeadingLayout { .. })
            | Some(DebugAction::MatchingTrailingLayout { .. })
            | Some(DebugAction::MatchedLayout { .. })
            | Some(DebugAction::MatchingTerminal { .. })
            | Some(DebugAction::MatchSuccess { .. }) => None,
            None => None,
        };

        DebugGSSInfo {
            nodes,
            edges,
            current_gss_node_id,
        }
    }

    /// Build stack trace from current action.
    /// Returns slot names from top (current) to bottom (root).
    pub fn build_stack_trace(&self) -> Option<Vec<String>> {
        let mut frames = Vec::new();
        let start_gss_node_id;

        match &self.current_action {
            Some(DebugAction::ProcessingDescriptor(desc)) => {
                // First frame is the current slot
                frames.push(self.symbols.slot(desc.slot_id).to_string());
                start_gss_node_id = desc.gss_node_id;
            }
            Some(DebugAction::Pop {
                slot_id,
                gss_node_id,
                ..
            }) => {
                // For Pop, show the slot (which has dot at end, e.g., "A ::= 'a' .")
                frames.push(self.symbols.slot(*slot_id).to_string());
                start_gss_node_id = *gss_node_id;
            }
            Some(DebugAction::MatchFailed {
                slot_id,
                gss_node_id,
                ..
            }) => {
                // For MatchFailed, show the slot where the match was attempted
                frames.push(self.symbols.slot(*slot_id).to_string());
                start_gss_node_id = *gss_node_id;
            }
            // Layout and terminal matching actions don't have a meaningful stack trace
            Some(DebugAction::MatchingLeadingLayout { .. })
            | Some(DebugAction::MatchingTrailingLayout { .. })
            | Some(DebugAction::MatchedLayout { .. })
            | Some(DebugAction::MatchingTerminal { .. })
            | Some(DebugAction::MatchSuccess { .. }) => return None,
            None => return None,
        }

        // Walk back through GSS edges
        let mut current_gss = start_gss_node_id;
        while let Some(node) = self.gss_nodes.get(current_gss.index()) {
            // Follow first edge (single execution thread model)
            let Some(edge) = node.edges().first() else {
                // Reached root (no outgoing edges) - add the start nonterminal
                let nt_name = self.symbols.nonterminal(node.nonterminal_id);
                frames.push(format!("{}.", nt_name));
                break;
            };

            frames.push(self.symbols.slot(edge.return_slot).to_string());
            current_gss = edge.dest_id;
        }

        Some(frames)
    }

    /// Build the complete event log from all trace events.
    /// Each entry includes whether it's a steppable event and its step index if so.
    pub fn build_event_log(&self) -> Vec<EventLogEntry> {
        let mut step_index_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for (step_idx, &event_idx) in self.step_indices.iter().enumerate() {
            step_index_map.insert(event_idx, step_idx);
        }

        self.events
            .iter()
            .enumerate()
            .map(|(i, event)| {
                let (message, event_type) = self.format_event(event);
                EventLogEntry {
                    event_index: i as u32,
                    step_index: step_index_map.get(&i).map(|&idx| idx as u32),
                    message,
                    event_type,
                }
            })
            .collect()
    }

    /// Format a trace event for the event log.
    /// Returns (message, event_type).
    fn format_event(&self, event: &TraceEvent) -> (String, String) {
        match event {
            TraceEvent::ProcessingDescriptor(slot_id, input_index, gss_node_id, sppf_node_id) => {
                let slot_name = self.symbols.slot(*slot_id);
                let gss_node = self.format_gss_node(*gss_node_id);
                let sppf_node = match sppf_node_id {
                    Some(id) => self.format_sppf_node(id.0),
                    None => "$".to_string(),
                };
                (
                    format!("Processing ({}, {}, {}, {})", slot_name, input_index, gss_node, sppf_node),
                    "processing".to_string(),
                )
            }
            TraceEvent::DescriptorAdded(slot_id, input_index, gss_node_id, sppf_node_id) => {
                let slot_name = self.symbols.slot(*slot_id);
                let gss_node = self.format_gss_node(*gss_node_id);
                let sppf_node = match sppf_node_id {
                    Some(id) => self.format_sppf_node(id.0),
                    None => "$".to_string(),
                };
                (
                    format!("Descriptor added ({}, {}, {}, {})", slot_name, input_index, gss_node, sppf_node),
                    "descriptor".to_string(),
                )
            }
            TraceEvent::MatchingLeadingLayout(input_index) => (
                format!("Matching leading layout at {}", input_index),
                "layout".to_string(),
            ),
            TraceEvent::MatchingTrailingLayout(input_index) => (
                format!("Matching trailing layout at {}", input_index),
                "layout".to_string(),
            ),
            TraceEvent::MatchingTerminal(terminal_name, input_index) => (
                format!("Matching '{}' at {}", terminal_name, input_index),
                "matching".to_string(),
            ),
            TraceEvent::MatchSuccess(terminal_name, input_index, next_index) => (
                format!("Matched '{}' at {} (len {})", terminal_name, input_index, next_index - input_index),
                "match_success".to_string(),
            ),
            TraceEvent::MatchFailed(terminal_name, input_index, ..) => (
                format!("Match failed '{}' at {}", terminal_name, input_index),
                "match_failed".to_string(),
            ),
            TraceEvent::MatchedLayout(next_index) => {
                if let Some(next) = next_index {
                    (format!("Matched layout → {}", next), "layout".to_string())
                } else {
                    ("No layout found".to_string(), "layout".to_string())
                }
            }
            TraceEvent::GSSNodeCreated(nonterminal_id, input_index) => {
                let nt_name = self.symbols.nonterminal(*nonterminal_id);
                (
                    format!("GSS node ({}, {}) created", nt_name, input_index),
                    "gss".to_string(),
                )
            }
            TraceEvent::GSSNodeFound(nonterminal_id, input_index) => {
                let nt_name = self.symbols.nonterminal(*nonterminal_id);
                (
                    format!("GSS node ({}, {}) found", nt_name, input_index),
                    "gss".to_string(),
                )
            }
            TraceEvent::GSSNodeNotFound(nonterminal_id, input_index) => {
                let nt_name = self.symbols.nonterminal(*nonterminal_id);
                (
                    format!("GSS node ({}, {}) not found", nt_name, input_index),
                    "gss".to_string(),
                )
            }
            TraceEvent::GSSNodeAdded(src_id, dest_id, return_slot) => {
                let slot_name = self.symbols.slot(*return_slot);
                (
                    format!("GSS edge {} → {} [{}]", src_id.0, dest_id.0, slot_name),
                    "gss".to_string(),
                )
            }
            TraceEvent::TerminalNodeCreated(terminal_id, span) => {
                let terminal_name = self.symbols.terminal(terminal_id);
                (
                    format!("SPPF terminal ({}, {}, {})", terminal_name, span.left_extent, span.right_extent),
                    "sppf".to_string(),
                )
            }
            TraceEvent::NonterminalNodeCreated(nonterminal_id, span, _child) => {
                let nt_name = self.symbols.nonterminal(*nonterminal_id);
                (
                    format!("SPPF nonterminal ({}, {}, {})", nt_name, span.left_extent, span.right_extent),
                    "sppf".to_string(),
                )
            }
            TraceEvent::IntermediateNodeCreated(slot_id, span, ..) => {
                let slot_name = self.symbols.slot(*slot_id);
                (
                    format!("SPPF intermediate ({}, {}, {})", slot_name, span.left_extent, span.right_extent),
                    "sppf".to_string(),
                )
            }
            TraceEvent::TerminalNodeFound(sppf_node_id) => (
                format!("SPPF terminal node {} found", sppf_node_id.0),
                "sppf".to_string(),
            ),
            TraceEvent::NonterminalNodeFound(sppf_node_id) => (
                format!("SPPF nonterminal node {} found", sppf_node_id.0),
                "sppf".to_string(),
            ),
            TraceEvent::IntermediateNodeFound(sppf_node_id) => (
                format!("SPPF intermediate node {} found", sppf_node_id.0),
                "sppf".to_string(),
            ),
            TraceEvent::Pop(gss_node_id, slot_id, sppf_node_id) => {
                let gss_node = self.format_gss_node(*gss_node_id);
                let slot_name = self.symbols.slot(*slot_id);
                (
                    format!("Pop {} [{}] with SPPF {}", gss_node, slot_name, sppf_node_id.0),
                    "pop".to_string(),
                )
            }
            TraceEvent::AddToPoppedElements(gss_node_id, sppf_node_id) => (
                format!("Add SPPF {} to popped elements of GSS {}", sppf_node_id.0, gss_node_id.0),
                "pop".to_string(),
            ),
            TraceEvent::NodeAlreadyInPoppedElements => (
                "Node already in popped elements".to_string(),
                "pop".to_string(),
            ),
            TraceEvent::Call(sppf_node_id, gss_node_id, slot_id) => {
                let gss_node = self.format_gss_node(*gss_node_id);
                let slot_name = self.symbols.slot(*slot_id);
                let sppf = sppf_node_id.map(|id| id.0.to_string()).unwrap_or_else(|| "$".to_string());
                (
                    format!("Call {} {} [{}]", sppf, gss_node, slot_name),
                    "call".to_string(),
                )
            }
            TraceEvent::ParseSuccess(duration) => (
                format!("Parse succeeded in {}ms", duration.as_millis()),
                "success".to_string(),
            ),
            TraceEvent::ParseFailed(duration) => (
                format!("Parse failed in {}ms", duration.as_millis()),
                "failed".to_string(),
            ),
        }
    }
}
