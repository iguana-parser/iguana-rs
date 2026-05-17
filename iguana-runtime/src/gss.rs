use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    env::EnvId,
    ids::{GssNodeId, NonterminalId, SlotId},
    sppf::SPPFNodeId,
    utils::{inline_map::InlineMap, inline_vec::InlineVec},
};

/// Key for the per-GSS popped-elements map. Two pops at the same
/// `(right_extent, return_value)` produce the same nonterminal SPPF node;
/// different parameter contexts live in different GSS nodes and therefore
/// in different maps.
pub type PoppedElementKey = (u32, Option<i32>);

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct GSSNode {
    pub id: GssNodeId,
    pub nonterminal_id: NonterminalId,
    pub index: u32,
    #[serde(skip)]
    #[specta(skip)]
    edges: InlineVec<GSSEdge, 16>,
    #[serde(skip)]
    #[specta(skip)]
    popped_elements: InlineMap<PoppedElementKey, SPPFNodeId>,
}

impl GSSNode {
    pub fn new(id: GssNodeId, nonterminal_id: NonterminalId, index: u32) -> Self {
        Self {
            id,
            nonterminal_id,
            index,
            edges: InlineVec::default(),
            popped_elements: InlineMap::default(),
        }
    }

    pub fn add_edge(&mut self, gss_edge: GSSEdge) {
        self.edges.push(gss_edge);
    }

    pub fn insert_popped_element(
        &mut self,
        right_extent: u32,
        return_value: Option<i32>,
        nonterminal_node_id: SPPFNodeId,
    ) {
        self.popped_elements
            .insert((right_extent, return_value), nonterminal_node_id);
    }

    pub fn find_popped_element(
        &self,
        right_extent: u32,
        return_value: Option<i32>,
    ) -> Option<SPPFNodeId> {
        self.popped_elements
            .get(&(right_extent, return_value))
            .copied()
    }

    pub fn contains_popped_element(&self, right_extent: u32, return_value: Option<i32>) -> bool {
        self.find_popped_element(right_extent, return_value)
            .is_some()
    }

    pub fn popped_elements(&self) -> &InlineMap<PoppedElementKey, SPPFNodeId> {
        &self.popped_elements
    }

    pub fn popped_elements_mut(&mut self) -> &mut InlineMap<PoppedElementKey, SPPFNodeId> {
        &mut self.popped_elements
    }

    pub fn edges(&self) -> &InlineVec<GSSEdge, 16> {
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
