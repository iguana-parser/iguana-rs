use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use iguana::gss::{GSSEdge, GSSNode};
use iguana::ids::{GssNodeId, NonterminalId, SlotId};
use iguana::trace::TraceEvent;
use serde::{Deserialize, Serialize};
use specta::Type;

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

    pub fn slot(&self, id: SlotId) -> &str {
        &self.slots[id.index()]
    }
}

/// A stack frame in the GLL call stack.
#[derive(Debug, Clone, Serialize, Type)]
pub struct StackFrame {
    pub slot_name: String,
}

/// A descriptor in the pending set.
#[derive(Debug, Clone, Serialize, Type)]
pub struct Descriptor {
    pub slot_name: String,
    pub input_index: u32,
    pub gss_node_id: u32,
}

/// Trace replay for debugging.
/// Steps are only Start and ProcessingDescriptor events.
/// Other events update state but are not steps.
pub struct TraceReplay {
    events: Vec<TraceEvent>,
    /// Indices into `events` that are steps (Start or ProcessingDescriptor)
    step_indices: Vec<usize>,
    /// Current step (0-indexed into step_indices)
    current_step: usize,
    /// GSS nodes reconstructed from trace
    gss_nodes: Vec<GSSNode>,
    /// Pending descriptor set
    descriptor_set: Vec<Descriptor>,
    /// Current descriptor being processed
    current_descriptor: Option<Descriptor>,
    symbols: SymbolTable,
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

        // Build index of step events (Start and ProcessingDescriptor)
        let step_indices: Vec<usize> = events
            .iter()
            .enumerate()
            .filter_map(|(i, event)| match event {
                TraceEvent::Start(..) | TraceEvent::ProcessingDescriptor(..) => Some(i),
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

    /// Get the pending descriptor set.
    pub fn descriptor_set(&self) -> &[Descriptor] {
        &self.descriptor_set
    }

    /// Get the current descriptor being processed.
    pub fn current_descriptor(&self) -> Option<&Descriptor> {
        self.current_descriptor.as_ref()
    }

    /// Get the current descriptor as a formatted string.
    pub fn current_descriptor_string(&self) -> Option<String> {
        let desc = self.current_descriptor.as_ref()?;
        Some(format!(
            "({}, {}, u{})",
            desc.slot_name, desc.input_index, desc.gss_node_id
        ))
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
            TraceEvent::Start(nonterminal_id, input_index, gss_node_id) => {
                // Start creates the initial descriptor
                self.current_descriptor = Some(Descriptor {
                    slot_name: format!("Start({})", self.symbols.nonterminal(*nonterminal_id)),
                    input_index: *input_index,
                    gss_node_id: gss_node_id.0,
                });
            }
            TraceEvent::ProcessingDescriptor(slot_id, input_index, gss_node_id, _) => {
                // Set current descriptor
                self.current_descriptor = Some(Descriptor {
                    slot_name: self.symbols.slot(*slot_id).to_string(),
                    input_index: *input_index,
                    gss_node_id: gss_node_id.0,
                });
                // Remove from pending set (if present)
                self.descriptor_set.retain(|d| {
                    !(d.slot_name == self.symbols.slot(*slot_id)
                        && d.input_index == *input_index
                        && d.gss_node_id == gss_node_id.0)
                });
            }
            TraceEvent::DescriptorAdded(slot_id, input_index, gss_node_id, _) => {
                // Add to pending descriptor set
                self.descriptor_set.push(Descriptor {
                    slot_name: self.symbols.slot(*slot_id).to_string(),
                    input_index: *input_index,
                    gss_node_id: gss_node_id.0,
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
            _ => {}
        }
    }

    /// Build stack trace from current descriptor.
    /// Returns frames from top (current) to bottom (root).
    pub fn build_stack_trace(&self) -> Option<Vec<StackFrame>> {
        let current = self.current_descriptor.as_ref()?;

        let mut frames = Vec::new();

        // First frame is the current slot
        frames.push(StackFrame {
            slot_name: current.slot_name.clone(),
        });

        // Walk back through GSS edges
        let mut current_gss = GssNodeId(current.gss_node_id);
        while let Some(node) = self.gss_nodes.get(current_gss.index()) {
            // Follow first edge (single execution thread model)
            let Some(edge) = node.edges().first() else {
                break; // Reached root (no outgoing edges)
            };

            frames.push(StackFrame {
                slot_name: self.symbols.slot(edge.return_slot).to_string(),
            });
            current_gss = edge.dest_id;
        }

        Some(frames)
    }
}
