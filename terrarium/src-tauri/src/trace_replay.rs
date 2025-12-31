use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use iguana::descriptor::Descriptor;
use iguana::gss::{GSSEdge, GSSNode};
use iguana::ids::{GssNodeId, NonterminalId, SlotId};
use iguana::trace::TraceEvent;
use serde::{Deserialize, Serialize};
use specta::Type;

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
/// Steps are ProcessingDescriptor events.
pub struct TraceReplay {
    events: Vec<TraceEvent>,
    /// Indices into `events` that are steps (ProcessingDescriptor only)
    step_indices: Vec<usize>,
    /// Current step (0-indexed into step_indices)
    current_step: usize,
    /// GSS nodes reconstructed from trace
    gss_nodes: Vec<GSSNode>,
    /// Pending descriptor set (uses iguana's Descriptor with IDs)
    descriptor_set: Vec<Descriptor>,
    /// Current descriptor being processed
    current_descriptor: Option<Descriptor>,
    symbols: SymbolTable,
    /// SPPF nodes reconstructed from trace
    sppf_nodes: Vec<DebugSPPFNode>,
    /// Current SPPF node ID from the descriptor being processed
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

        // Build index of step events (ProcessingDescriptor only)
        let step_indices: Vec<usize> = events
            .iter()
            .enumerate()
            .filter_map(|(i, event)| match event {
                TraceEvent::ProcessingDescriptor(..) => Some(i),
                _ => None,
            })
            .collect();

        let mut replay = Self {
            events,
            step_indices,
            current_step: 0,
            gss_nodes: Vec::new(),
            descriptor_set: Vec::new(),
            current_descriptor: None,
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
            format!("({}, {}, {})", node.label, node.left_extent, node.right_extent)
        } else {
            format!("(?, {})", sppf_node_id)
        }
    }

    /// Format a descriptor for display: "(slot, input_index, gss_node, sppf_node)".
    fn format_descriptor(&self, desc: &Descriptor) -> String {
        let slot_name = self.symbols.slot(desc.slot_id);
        let gss_node = self.format_gss_node(desc.gss_node_id);
        let sppf_node = match desc.sppf_node_id {
            Some(id) => self.format_sppf_node(id.0),
            None => "$".to_string(),
        };
        format!("({}, {}, {}, {})", slot_name, desc.input_index, gss_node, sppf_node)
    }

    /// Get the current descriptor as a formatted string.
    pub fn current_descriptor_string(&self) -> Option<String> {
        self.current_descriptor
            .as_ref()
            .map(|desc| self.format_descriptor(desc))
    }

    /// Get the current input index (position in the input being parsed).
    pub fn current_input_index(&self) -> Option<usize> {
        self.current_descriptor
            .as_ref()
            .map(|desc| desc.input_index as usize)
    }

    /// Get the current SPPF nodes for visualization.
    pub fn sppf_nodes(&self) -> &[DebugSPPFNode] {
        &self.sppf_nodes
    }

    /// Get the current SPPF node ID from the descriptor being processed.
    pub fn current_sppf_node_id(&self) -> Option<u32> {
        self.current_sppf_node_id
    }

    /// Get the pending descriptor set as formatted strings.
    pub fn descriptor_set_strings(&self) -> Vec<String> {
        self.descriptor_set
            .iter()
            .map(|desc| self.format_descriptor(desc))
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
            self.current_descriptor = None;
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
                self.current_descriptor = Some(desc);
                // Track the current SPPF node from the descriptor
                self.current_sppf_node_id = sppf_node_id.map(|id| id.0);
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
            _ => {}
        }
    }

    /// Build stack trace from current descriptor.
    /// Returns slot names from top (current) to bottom (root).
    pub fn build_stack_trace(&self) -> Option<Vec<String>> {
        let current = self.current_descriptor.as_ref()?;

        let mut frames = Vec::new();

        // First frame is the current slot
        frames.push(self.symbols.slot(current.slot_id).to_string());

        // Walk back through GSS edges
        let mut current_gss = current.gss_node_id;
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
