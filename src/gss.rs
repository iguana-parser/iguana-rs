use crate::{
    ids::{GssNodeId, NonterminalId, SlotId},
    sppf::SPPFNodeId,
    utils::{inline_set::InlineSet, inline_vec::InlineVec},
};

#[derive(Debug)]
pub struct GSSNode {
    pub id: GssNodeId,
    pub nonterminal_id: NonterminalId,
    pub index: u32,
    edges: InlineVec<GSSEdge>,
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
pub struct EdgeResult {
    pub node_id: SPPFNodeId,
    pub left_extent: u32,
}

#[derive(Clone, Debug)]
pub struct GSSEdge {
    pub result: Option<EdgeResult>,
    pub return_slot: SlotId,
    pub dest_id: GssNodeId,
}

impl GSSEdge {
    pub fn new(result: Option<EdgeResult>, return_slot: SlotId, dest_id: GssNodeId) -> Self {
        Self {
            result,
            return_slot,
            dest_id,
        }
    }
}
