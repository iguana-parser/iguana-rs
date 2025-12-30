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

/// Trace replay for debugging.
/// Loads trace events and reconstructs GSS incrementally.
pub struct TraceReplay {
    events: Vec<TraceEvent>,
    gss_nodes: Vec<GSSNode>,
    current_step: usize,
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

        Ok(Self {
            events,
            gss_nodes: Vec::new(),
            current_step: 0,
            symbols,
        })
    }

    /// Total number of events.
    pub fn total_steps(&self) -> usize {
        self.events.len()
    }

    /// Current step (0-indexed).
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Get the current event, if any.
    pub fn current_event(&self) -> Option<&TraceEvent> {
        self.events.get(self.current_step)
    }

    /// Step forward, applying the next event to the GSS.
    /// Returns true if stepped, false if at end.
    pub fn step_forward(&mut self) -> bool {
        if self.current_step >= self.events.len() {
            return false;
        }

        self.apply_event_at(self.current_step);
        self.current_step += 1;
        true
    }

    /// Step to a specific position, rebuilding GSS state.
    pub fn step_to(&mut self, target: usize) {
        let target = target.min(self.events.len());

        if target <= self.current_step {
            // Need to rebuild from scratch
            self.gss_nodes.clear();
            self.current_step = 0;
        }

        while self.current_step < target {
            self.apply_event_at(self.current_step);
            self.current_step += 1;
        }
    }

    /// Apply the trace event at the given index to update GSS state.
    fn apply_event_at(&mut self, index: usize) {
        match self.events[index] {
            TraceEvent::GSSNodeCreated(nonterminal_id, input_index) => {
                let id = GssNodeId(self.gss_nodes.len() as u32);
                self.gss_nodes
                    .push(GSSNode::new(id, nonterminal_id, input_index));
            }
            TraceEvent::GSSNodeAdded(src_id, dest_id, return_slot) => {
                if let Some(node) = self.gss_nodes.get_mut(src_id.index()) {
                    node.add_edge(GSSEdge::new(None, return_slot, dest_id));
                }
            }
            _ => {}
        }
    }

    /// Build stack trace from current descriptor.
    /// Returns frames from top (current) to bottom (root).
    pub fn build_stack_trace(&self) -> Option<Vec<StackFrame>> {
        let (slot_id, _, gss_node_id) = self.current_descriptor()?;

        let mut frames = Vec::new();

        // First frame is the current slot
        frames.push(StackFrame {
            slot_name: self.symbols.slot(slot_id).to_string(),
        });

        // Walk back through GSS edges
        let mut current_gss = gss_node_id;
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

    /// Get the current descriptor info if the current event is ProcessingDescriptor.
    pub fn current_descriptor(&self) -> Option<(SlotId, u32, GssNodeId)> {
        match self.current_event()? {
            TraceEvent::ProcessingDescriptor(slot_id, input_index, gss_node_id, _) => {
                Some((*slot_id, *input_index, *gss_node_id))
            }
            _ => None,
        }
    }
}
