use std::hash::BuildHasher;

use rustc_hash::FxBuildHasher;
use serde::Serialize;

use crate::{
    arena::{Arena, ArenaIdVec, ArenaTable},
    ids::{EnvId, GssNodeId, NonterminalId, SPPFNodeId, SlotId},
    utils::{id_vec::Placeholder, inline_map::InlineMap, inline_vec::InlineVec},
};

/// Key for the per-GSS popped-elements map. Two pops at the same
/// `(right_extent, return_value)` produce the same nonterminal SPPF node;
/// different parameter contexts live in different GSS nodes and therefore
/// in different maps.
pub type PoppedElementKey = (u32, Option<i32>);

#[derive(Debug, Serialize)]
pub struct GSSNode<'arena> {
    pub nonterminal_id: NonterminalId,
    pub index: u32,
    #[serde(skip)]
    edges: InlineVec<'arena, GSSEdge, 16>,
    #[serde(skip)]
    popped_elements: InlineMap<'arena, PoppedElementKey, SPPFNodeId>,
}

impl<'arena> GSSNode<'arena> {
    pub fn new(nonterminal_id: NonterminalId, index: u32) -> Self {
        Self {
            nonterminal_id,
            index,
            edges: InlineVec::default(),
            popped_elements: InlineMap::default(),
        }
    }

    pub fn add_edge(&mut self, gss_edge: GSSEdge, arena: &'arena Arena) {
        self.edges.push(gss_edge, arena);
    }

    pub fn insert_popped_element(
        &mut self,
        right_extent: u32,
        return_value: Option<i32>,
        nonterminal_node_id: SPPFNodeId,
        arena: &'arena Arena,
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

impl Placeholder for GSSNode<'_> {
    fn placeholder() -> Self {
        GSSNode::new(NonterminalId(0), 0)
    }
}

/// The initial capacity of a `GSSIndex`, as a divisor of the input length. An
/// index that outgrows its capacity rehashes, and an index that is too large
/// takes extra memory.
const GSS_INDEX_CAPACITY_DIVISOR: usize = 2;

/// Maps a pair of nonterminal_id and input_index to a gss_node_id.
pub struct GSSIndex<'arena> {
    table: ArenaTable<'arena, GssNodeId>,
}

impl<'arena> GSSIndex<'arena> {
    pub fn new(input_len: usize, arena: &'arena Arena) -> Self {
        Self {
            table: arena.table_with_capacity(input_len / GSS_INDEX_CAPACITY_DIVISOR),
        }
    }

    /// The GSS node id for the pair `nonterminal_id` at `input_index`, or `None`
    /// if the index does not have a GSS node for that pair.
    ///
    /// The index stores only the GSS node ids. It reads the nonterminal and the
    /// input position of an indexed node from the node itself, so `get` takes
    /// the vector that holds the GSS nodes.
    #[inline]
    pub fn get(
        &self,
        gss_nodes: &ArenaIdVec<'arena, GssNodeId, GSSNode<'arena>>,
        nonterminal_id: NonterminalId,
        input_index: u32,
    ) -> Option<GssNodeId> {
        self.table
            .find(Self::hash(nonterminal_id, input_index), |&id| {
                let node = &gss_nodes[id];
                node.nonterminal_id == nonterminal_id && node.index == input_index
            })
            .copied()
    }

    /// Records `gss_node_id` corresponding to the pair `nonterminal_id` at
    /// `input_index`. The index does not check for the existence of the pair,
    /// so the caller should make sure the index does not already have a GSS node
    /// for the pair before inserting. The node `gss_node_id` in `gss_nodes` must
    /// have that nonterminal and input position.
    #[inline]
    pub fn insert(
        &mut self,
        gss_nodes: &ArenaIdVec<'arena, GssNodeId, GSSNode<'arena>>,
        nonterminal_id: NonterminalId,
        input_index: u32,
        gss_node_id: GssNodeId,
    ) {
        debug_assert!(
            gss_nodes[gss_node_id].nonterminal_id == nonterminal_id
                && gss_nodes[gss_node_id].index == input_index
        );
        self.table.insert_unique(
            Self::hash(nonterminal_id, input_index),
            gss_node_id,
            |&id| {
                let node = &gss_nodes[id];
                Self::hash(node.nonterminal_id, node.index)
            },
        );
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// The hash of a pair: the Fx hash of a `u64` that holds the nonterminal
    /// id in its high 32 bits and the input position in its low 32 bits.
    #[inline]
    fn hash(nonterminal_id: NonterminalId, input_index: u32) -> u64 {
        FxBuildHasher.hash_one(((nonterminal_id.index() as u64) << 32) | u64::from(input_index))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GSSEdge {
    pub sppf_node_id: Option<SPPFNodeId>,
    pub return_slot: SlotId,
    pub dest_id: GssNodeId,
    // The caller's env at the time of the call, saved during `create`.
    // During `pop`, when iterating over edges, this env is restored and
    // extended with the callee's return value (if a binding is present).
    pub env_id: Option<EnvId>,
}

impl GSSEdge {
    #[inline]
    pub fn new(
        sppf_node_id: Option<SPPFNodeId>,
        return_slot: SlotId,
        dest_id: GssNodeId,
        env_id: Option<EnvId>,
    ) -> Self {
        Self {
            sppf_node_id,
            return_slot,
            dest_id,
            env_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arena::Arena,
        gss::{GSSIndex, GSSNode},
        ids::NonterminalId,
    };

    #[test]
    fn finds_the_inserted_node() {
        let arena = Arena::new();
        let mut gss_nodes = arena.id_vec();
        let mut index = GSSIndex::new(16, &arena);
        assert_eq!(index.get(&gss_nodes, NonterminalId(3), 5), None);

        let id = gss_nodes.push(GSSNode::new(NonterminalId(3), 5));
        index.insert(&gss_nodes, NonterminalId(3), 5, id);
        assert_eq!(index.get(&gss_nodes, NonterminalId(3), 5), Some(id));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn nonterminals_and_positions_are_independent() {
        let arena = Arena::new();
        let mut gss_nodes = arena.id_vec();
        let mut index = GSSIndex::new(16, &arena);
        let mut add = |nonterminal_id, input_index| {
            let id = gss_nodes.push(GSSNode::new(nonterminal_id, input_index));
            index.insert(&gss_nodes, nonterminal_id, input_index, id);
            id
        };
        let a = add(NonterminalId(1), 7);
        let b = add(NonterminalId(2), 7);
        let c = add(NonterminalId(1), 8);

        assert_eq!(index.get(&gss_nodes, NonterminalId(1), 7), Some(a));
        assert_eq!(index.get(&gss_nodes, NonterminalId(2), 7), Some(b));
        assert_eq!(index.get(&gss_nodes, NonterminalId(1), 8), Some(c));
        assert_eq!(index.get(&gss_nodes, NonterminalId(2), 8), None);
    }

    #[test]
    fn grows_past_its_initial_capacity() {
        let arena = Arena::new();
        let mut gss_nodes = arena.id_vec();
        let mut index = GSSIndex::new(4, &arena);
        let mut ids = Vec::new();
        for i in 0..100u32 {
            let nonterminal_id = NonterminalId((i % 7) as u16);
            let id = gss_nodes.push(GSSNode::new(nonterminal_id, i));
            index.insert(&gss_nodes, nonterminal_id, i, id);
            ids.push(id);
        }
        for i in 0..100u32 {
            assert_eq!(
                index.get(&gss_nodes, NonterminalId((i % 7) as u16), i),
                Some(ids[i as usize])
            );
        }
        assert_eq!(index.len(), 100);
    }
}
