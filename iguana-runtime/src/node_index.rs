use std::hash::BuildHasher;

use rustc_hash::FxBuildHasher;

use crate::{
    arena::{Arena, ArenaIdVec, ArenaTable},
    gss::GSSNode,
    ids::{EnvId, GssNodeId, Id, NonterminalId, SPPFNodeId, SlotId, TerminalId},
    input::Span,
    sppf::SPPFNode,
};

/// The fields that identify a node in a [`NodeIndex`].
pub trait NodeKey: Copy + Eq {
    type Id: Id;
    type Node<'arena>;

    /// The initial capacity of the index, as a divisor of the input length. An
    /// index that outgrows its capacity rehashes, and an index that is too
    /// large takes extra memory.
    const CAPACITY_DIVISOR: usize;

    /// The key of `node`, or `None` if the index does not hold nodes of the
    /// kind of `node`.
    fn of(node: &Self::Node<'_>) -> Option<Self>;

    fn hash(self) -> u64;
}

/// A map from a key to the id of the node with that key.
///
/// The index stores only the node ids. It reads the key of an indexed node
/// from the node itself, so `get` and `insert` take the vector that holds the
/// nodes.
pub struct NodeIndex<'arena, K: NodeKey> {
    table: ArenaTable<'arena, K::Id>,
}

pub type TerminalNodeIndex<'arena> = NodeIndex<'arena, TerminalNodeKey>;
pub type IntermediateNodeIndex<'arena> = NodeIndex<'arena, IntermediateNodeKey>;
pub type GSSNodeIndex<'arena> = NodeIndex<'arena, GSSNodeKey>;

impl<'arena, K: NodeKey> NodeIndex<'arena, K> {
    pub fn new(input_len: usize, arena: &'arena Arena) -> Self {
        Self {
            table: arena.table_with_capacity(input_len / K::CAPACITY_DIVISOR),
        }
    }

    /// The id of the node with `key`, or `None` if the index does not have a
    /// node with that key.
    #[inline]
    pub fn get(&self, nodes: &ArenaIdVec<'arena, K::Id, K::Node<'arena>>, key: K) -> Option<K::Id> {
        self.table
            .find(key.hash(), |&id| K::of(&nodes[id]) == Some(key))
            .copied()
    }

    /// Records `id` as the node with `key`. The index does not check for the
    /// existence of the key, so the caller should make sure the index does not
    /// already have a node with that key before inserting. The node `id` in
    /// `nodes` must have that key.
    #[inline]
    pub fn insert(
        &mut self,
        nodes: &ArenaIdVec<'arena, K::Id, K::Node<'arena>>,
        key: K,
        id: K::Id,
    ) {
        debug_assert!(K::of(&nodes[id]) == Some(key));
        self.table.insert_unique(key.hash(), id, |&id| {
            K::of(&nodes[id])
                .expect("a node index holds only nodes with a key")
                .hash()
        });
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

/// The key of a terminal node: its terminal and span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalNodeKey {
    pub terminal_id: TerminalId,
    pub span: Span,
}

impl NodeKey for TerminalNodeKey {
    type Id = SPPFNodeId;
    type Node<'arena> = SPPFNode;

    const CAPACITY_DIVISOR: usize = 2;

    #[inline]
    fn of(node: &SPPFNode) -> Option<Self> {
        match node {
            SPPFNode::Terminal(node) => Some(Self {
                terminal_id: node.terminal_id,
                span: node.span,
            }),
            _ => None,
        }
    }

    /// The Fx hash of a `u64` that holds the terminal id in its high 32 bits
    /// and the left extent in its low 32 bits. The right extent is not part of
    /// the hash, because the scanner matches a terminal at a position with at
    /// most one right extent.
    #[inline]
    fn hash(self) -> u64 {
        let terminal_id = self.terminal_id.index() as u64;
        FxBuildHasher.hash_one((terminal_id << 32) | u64::from(self.span.left_extent))
    }
}

/// The key of an intermediate node: its slot, span, and env.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntermediateNodeKey {
    pub slot_id: SlotId,
    pub span: Span,
    pub env: Option<EnvId>,
}

impl NodeKey for IntermediateNodeKey {
    type Id = SPPFNodeId;
    type Node<'arena> = SPPFNode;

    const CAPACITY_DIVISOR: usize = 1;

    #[inline]
    fn of(node: &SPPFNode) -> Option<Self> {
        match node {
            SPPFNode::Intermediate(node) => Some(Self {
                slot_id: node.slot_id,
                span: node.span,
                env: node.env,
            }),
            _ => None,
        }
    }

    #[inline]
    fn hash(self) -> u64 {
        // The goal is the fewest hasher writes, because FxHasher does one
        // multiply per write. The key fits in two `u64` values:
        // - The span is one `u64`. `Span::hash` writes the left extent in its
        //   high 32 bits and the right extent in its low 32 bits.
        // - The slot id, a `u16`, goes in the high 32 bits of the other `u64`.
        // - The env goes in the low 32 bits of that `u64`. `EnvId` is a
        //   `NonZeroU32`, so `None` becomes 0, which no `EnvId` has.
        // Hashing the key as a tuple writes each field and the `Option`
        // discriminant separately.
        let env = self.env.map_or(0, |env| env.index() as u64);
        FxBuildHasher.hash_one(((u64::from(self.slot_id.0) << 32) | env, self.span))
    }
}

/// The key of a GSS node: its nonterminal and input position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GSSNodeKey {
    pub nonterminal_id: NonterminalId,
    pub input_index: u32,
}

impl NodeKey for GSSNodeKey {
    type Id = GssNodeId;
    type Node<'arena> = GSSNode<'arena>;

    const CAPACITY_DIVISOR: usize = 2;

    #[inline]
    fn of(node: &GSSNode<'_>) -> Option<Self> {
        Some(Self {
            nonterminal_id: node.nonterminal_id,
            input_index: node.index,
        })
    }

    /// The Fx hash of a `u64` that holds the nonterminal id in its high 32
    /// bits and the input position in its low 32 bits.
    #[inline]
    fn hash(self) -> u64 {
        let nonterminal_id = self.nonterminal_id.index() as u64;
        FxBuildHasher.hash_one((nonterminal_id << 32) | u64::from(self.input_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sppf::{IntermediateNode, TerminalNode};

    fn terminal_node(terminal_id: TerminalId, span: Span) -> SPPFNode {
        SPPFNode::Terminal(TerminalNode { span, terminal_id })
    }

    fn intermediate_node(slot_id: SlotId, span: Span, env: Option<EnvId>) -> SPPFNode {
        SPPFNode::Intermediate(IntermediateNode {
            span,
            child: (SPPFNodeId::from_index(1), SPPFNodeId::from_index(1)),
            env,
            slot_id,
            ambiguous: false,
        })
    }

    #[test]
    fn terminal_nodes_differ_by_terminal_and_span() {
        let arena = Arena::new();
        let mut sppf_nodes = arena.id_vec();
        let mut index = TerminalNodeIndex::new(16, &arena);
        let mut add = |terminal_id, span| {
            let id = sppf_nodes.push(terminal_node(terminal_id, span));
            index.insert(&sppf_nodes, TerminalNodeKey { terminal_id, span }, id);
            id
        };
        let a = add(TerminalId(1), Span::new(3, 5));
        let b = add(TerminalId(2), Span::new(3, 5));
        let c = add(TerminalId(1), Span::new(3, 6));
        let d = add(TerminalId(1), Span::new(4, 5));

        let get = |terminal_id, span| index.get(&sppf_nodes, TerminalNodeKey { terminal_id, span });
        assert_eq!(get(TerminalId(1), Span::new(3, 5)), Some(a));
        assert_eq!(get(TerminalId(2), Span::new(3, 5)), Some(b));
        assert_eq!(get(TerminalId(1), Span::new(3, 6)), Some(c));
        assert_eq!(get(TerminalId(1), Span::new(4, 5)), Some(d));
        assert_eq!(get(TerminalId(2), Span::new(3, 6)), None);
        assert_eq!(index.len(), 4);
    }

    #[test]
    fn intermediate_nodes_differ_by_slot_span_and_env() {
        let arena = Arena::new();
        let mut sppf_nodes = arena.id_vec();
        let mut index = IntermediateNodeIndex::new(16, &arena);
        let mut add = |slot_id, span, env| {
            let id = sppf_nodes.push(intermediate_node(slot_id, span, env));
            index.insert(&sppf_nodes, IntermediateNodeKey { slot_id, span, env }, id);
            id
        };
        let env = Some(EnvId::from_index(1));
        let other_env = Some(EnvId::from_index(2));
        let a = add(SlotId(1), Span::new(0, 4), None);
        let b = add(SlotId(1), Span::new(0, 4), env);
        let c = add(SlotId(1), Span::new(0, 4), other_env);
        let d = add(SlotId(2), Span::new(0, 4), None);
        let e = add(SlotId(1), Span::new(1, 4), None);

        let get =
            |slot_id, span, env| index.get(&sppf_nodes, IntermediateNodeKey { slot_id, span, env });
        assert_eq!(get(SlotId(1), Span::new(0, 4), None), Some(a));
        assert_eq!(get(SlotId(1), Span::new(0, 4), env), Some(b));
        assert_eq!(get(SlotId(1), Span::new(0, 4), other_env), Some(c));
        assert_eq!(get(SlotId(2), Span::new(0, 4), None), Some(d));
        assert_eq!(get(SlotId(1), Span::new(1, 4), None), Some(e));
        assert_eq!(get(SlotId(2), Span::new(0, 4), env), None);
        assert_eq!(index.len(), 5);
    }

    #[test]
    fn gss_nodes_differ_by_nonterminal_and_input_position() {
        let arena = Arena::new();
        let mut gss_nodes = arena.id_vec();
        let mut index = GSSNodeIndex::new(16, &arena);
        let key = GSSNodeKey {
            nonterminal_id: NonterminalId(1),
            input_index: 7,
        };
        assert_eq!(index.get(&gss_nodes, key), None);
        assert!(index.is_empty());

        let mut add = |nonterminal_id, input_index| {
            let id = gss_nodes.push(GSSNode::new(nonterminal_id, input_index));
            index.insert(
                &gss_nodes,
                GSSNodeKey {
                    nonterminal_id,
                    input_index,
                },
                id,
            );
            id
        };
        let a = add(NonterminalId(1), 7);
        let b = add(NonterminalId(2), 7);
        let c = add(NonterminalId(1), 8);

        let get = |nonterminal_id, input_index| {
            index.get(
                &gss_nodes,
                GSSNodeKey {
                    nonterminal_id,
                    input_index,
                },
            )
        };
        assert_eq!(get(NonterminalId(1), 7), Some(a));
        assert_eq!(get(NonterminalId(2), 7), Some(b));
        assert_eq!(get(NonterminalId(1), 8), Some(c));
        assert_eq!(get(NonterminalId(2), 8), None);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn indexes_grow_past_their_initial_capacity() {
        let arena = Arena::new();
        let mut sppf_nodes = arena.id_vec();
        let mut gss_nodes = arena.id_vec();
        let mut terminal_index = TerminalNodeIndex::new(0, &arena);
        let mut intermediate_index = IntermediateNodeIndex::new(0, &arena);
        let mut gss_index = GSSNodeIndex::new(0, &arena);
        let terminal_key = |i: u32| TerminalNodeKey {
            terminal_id: TerminalId((i % 7) as u16),
            span: Span::new(i, i + 1),
        };
        let intermediate_key = |i: u32| IntermediateNodeKey {
            slot_id: SlotId((i % 5) as u16),
            span: Span::new(0, i),
            env: None,
        };
        let gss_key = |i: u32| GSSNodeKey {
            nonterminal_id: NonterminalId((i % 3) as u16),
            input_index: i,
        };
        let mut ids = Vec::new();
        for i in 0..100 {
            let TerminalNodeKey { terminal_id, span } = terminal_key(i);
            let terminal = sppf_nodes.push(terminal_node(terminal_id, span));
            terminal_index.insert(&sppf_nodes, terminal_key(i), terminal);
            let IntermediateNodeKey { slot_id, span, env } = intermediate_key(i);
            let intermediate = sppf_nodes.push(intermediate_node(slot_id, span, env));
            intermediate_index.insert(&sppf_nodes, intermediate_key(i), intermediate);
            let GSSNodeKey {
                nonterminal_id,
                input_index,
            } = gss_key(i);
            let gss_node = gss_nodes.push(GSSNode::new(nonterminal_id, input_index));
            gss_index.insert(&gss_nodes, gss_key(i), gss_node);
            ids.push((terminal, intermediate, gss_node));
        }
        for i in 0..100 {
            let (terminal, intermediate, gss_node) = ids[i as usize];
            assert_eq!(
                terminal_index.get(&sppf_nodes, terminal_key(i)),
                Some(terminal)
            );
            assert_eq!(
                intermediate_index.get(&sppf_nodes, intermediate_key(i)),
                Some(intermediate)
            );
            assert_eq!(gss_index.get(&gss_nodes, gss_key(i)), Some(gss_node));
        }
        assert_eq!(terminal_index.len(), 100);
        assert_eq!(intermediate_index.len(), 100);
        assert_eq!(gss_index.len(), 100);
    }
}
