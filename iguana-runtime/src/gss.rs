use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    env::EnvId,
    ids::{GssNodeId, NonterminalId, SlotId},
    sppf::SPPFNodeId,
    utils::{inline_set::InlineSet, inline_vec::InlineVec},
};

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct GSSNode {
    pub id: GssNodeId,
    pub nonterminal_id: NonterminalId,
    pub index: u32,
    #[serde(skip)]
    #[specta(skip)]
    edges: InlineVec<GSSEdge>,
    #[serde(skip)]
    #[specta(skip)]
    popped_elements: InlineSet<SPPFNodeId>,
}

impl GSSNode {
    pub fn new(id: GssNodeId, nonterminal_id: NonterminalId, index: u32) -> Self {
        Self {
            id,
            nonterminal_id,
            index,
            edges: InlineVec::default(),
            popped_elements: InlineSet::default(),
        }
    }

    pub fn add_edge(&mut self, gss_edge: GSSEdge) {
        self.edges.push(gss_edge);
    }

    pub fn add_to_popped_elements(&mut self, result: SPPFNodeId) {
        self.popped_elements.push(result);
    }

    pub fn contains_popped_element(&self, value: &SPPFNodeId) -> bool {
        self.popped_elements.contains(value)
    }

    pub fn popped_elements(&self) -> &InlineSet<SPPFNodeId> {
        &self.popped_elements
    }

    pub fn popped_elements_mut(&mut self) -> &mut InlineSet<SPPFNodeId> {
        &mut self.popped_elements
    }

    pub fn edges(&self) -> &InlineVec<GSSEdge> {
        &self.edges
    }
}

#[derive(Clone, Debug)]
pub struct GSSEdge {
    pub sppf_node_id: Option<SPPFNodeId>,
    pub return_slot: SlotId,
    pub dest_id: GssNodeId,
    // The caller's env at the time of the call, saved during `create`.
    // During `pop`, when iterating over edges, this env is restored and
    // extended with the callee's return value (if a binding is present).
    pub env: Option<EnvId>,
    // When a call symbol has a binding (e.g., `b=B(0)` in `A := b=B(0) C`),
    // the variable name is stored on the edge during `create`. During `pop`,
    // when iterating over edges, the callee's return value is bound to this
    // name in the restored env.
    pub binding: Option<&'static str>,
}
