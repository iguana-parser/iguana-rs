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
}

/// Simple SPPF node for debug visualization.
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

        // Build index of step events (ProcessingDescriptor, Pop, and MatchFailed)
        let step_indices: Vec<usize> = events
            .iter()
            .enumerate()
            .filter_map(|(i, event)| match event {
                TraceEvent::ProcessingDescriptor(..)
                | TraceEvent::Pop(..)
                | TraceEvent::MatchFailed(..) => Some(i),
                _ => None,
            })
            .collect();

        let mut replay = Self {
            events,
            step_indices,
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
    fn format_pop(&self, slot_id: SlotId, gss_node_id: GssNodeId, sppf_node_id: SPPFNodeId) -> String {
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
                });
            }
            TraceEvent::GSSNodeCreated(nonterminal_id, input_index) => {
                let id = GssNodeId(self.gss_nodes.len() as u32);
                self.gss_nodes
                    .push(GSSNode::new(id, *nonterminal_id, *input_index));
            }
            TraceEvent::GSSNodeAdded(src_id, dest_id, return_slot) => {
                if let Some(node) = self.gss_nodes.get_mut(src_id.index()) {
                    node.add_edge(GSSEdge::new(None, *return_slot, *dest_id));
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
            TraceEvent::MatchFailed(terminal_name, input_index, slot_id, gss_node_id, sppf_node_id) => {
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
}
