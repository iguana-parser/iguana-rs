use bumpalo::Bump;
use serde::Serialize;

use crate::{
    env::EnvId,
    ids::{BindingId, GssNodeId, NonterminalId, SlotId},
    sppf::SPPFNodeId,
    utils::{inline_map::InlineMap, inline_vec::InlineVec},
};

/// Key for the per-GSS popped-elements map. Two pops at the same
/// `(right_extent, return_value)` produce the same nonterminal SPPF node;
/// different parameter contexts live in different GSS nodes and therefore
/// in different maps.
pub type PoppedElementKey = (u32, Option<i32>);

#[derive(Debug, Serialize)]
pub struct GSSNode<'arena> {
    pub id: GssNodeId,
    pub nonterminal_id: NonterminalId,
    pub index: u32,
    #[serde(skip)]
    edges: InlineVec<'arena, GSSEdge, 16>,
    #[serde(skip)]
    popped_elements: InlineMap<'arena, PoppedElementKey, SPPFNodeId>,
}

impl<'arena> GSSNode<'arena> {
    pub fn new(id: GssNodeId, nonterminal_id: NonterminalId, index: u32) -> Self {
        Self {
            id,
            nonterminal_id,
            index,
            edges: InlineVec::default(),
            popped_elements: InlineMap::default(),
        }
    }

    pub fn add_edge(&mut self, gss_edge: GSSEdge, arena: &'arena Bump) {
        self.edges.push(gss_edge, arena);
    }

    pub fn insert_popped_element(
        &mut self,
        right_extent: u32,
        return_value: Option<i32>,
        nonterminal_node_id: SPPFNodeId,
        arena: &'arena Bump,
    ) {
        self.popped_elements
            .insert((right_extent, return_value), nonterminal_node_id, arena);
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

    pub fn popped_elements(&self) -> &InlineMap<'arena, PoppedElementKey, SPPFNodeId> {
        &self.popped_elements
    }

    pub fn popped_elements_mut(&mut self) -> &mut InlineMap<'arena, PoppedElementKey, SPPFNodeId> {
        &mut self.popped_elements
    }

    pub fn edges(&self) -> &InlineVec<'arena, GSSEdge, 16> {
        &self.edges
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GSSEdge {
    sppf_node_id: SPPFNodeId,
    pub return_slot: SlotId,
    pub dest_id: GssNodeId,
    // The caller's env at the time of the call, saved during `create`.
    // During `pop`, when iterating over edges, this env is restored and
    // extended with the callee's return value (if a binding is present).
    env_id: EnvId,
    // When a call symbol has a binding (e.g., `b=B(0)` in `A := b=B(0) C`),
    // the variable name is stored on the edge during `create`. During `pop`,
    // when iterating over edges, the callee's return value is bound to this
    // name in the restored env.
    binding_id: BindingId,
}

impl GSSEdge {
    #[inline]
    pub fn new(
        sppf_node_id: Option<SPPFNodeId>,
        return_slot: SlotId,
        dest_id: GssNodeId,
        env_id: Option<EnvId>,
        binding_id: Option<BindingId>,
    ) -> Self {
        Self {
            sppf_node_id: sppf_node_id.unwrap_or(SPPFNodeId::NONE),
            return_slot,
            dest_id,
            env_id: env_id.unwrap_or(EnvId::NONE),
            binding_id: binding_id.unwrap_or(BindingId::NONE),
        }
    }

    #[inline]
    pub fn sppf_node_id(&self) -> Option<SPPFNodeId> {
        if self.sppf_node_id == SPPFNodeId::NONE {
            None
        } else {
            Some(self.sppf_node_id)
        }
    }

    #[inline]
    pub fn env_id(&self) -> Option<EnvId> {
        if self.env_id == EnvId::NONE {
            None
        } else {
            Some(self.env_id)
        }
    }

    #[inline]
    pub fn binding_id(&self) -> Option<BindingId> {
        if self.binding_id == BindingId::NONE {
            None
        } else {
            Some(self.binding_id)
        }
    }
}
