use std::hash::BuildHasher;

use rustc_hash::FxBuildHasher;

use crate::arena::{Arena, ArenaIdVec, ArenaTable};
use crate::ids::{EnvId, Id, NonterminalId, SPPFNodeId, SlotId, TerminalId};
use crate::input::Span;
use crate::utils::id_vec::Placeholder;

#[derive(Debug)]
pub enum SPPFNode {
    Terminal(TerminalNode),
    Nonterminal(NonterminalNode),
    Intermediate(IntermediateNode),
}

impl SPPFNode {
    pub fn left_extent(&self) -> u32 {
        match self {
            SPPFNode::Terminal(t) => t.span.left_extent,
            SPPFNode::Nonterminal(n) => n.span.left_extent,
            SPPFNode::Intermediate(i) => i.span.left_extent,
        }
    }

    pub fn right_extent(&self) -> u32 {
        match self {
            SPPFNode::Terminal(t) => t.span.right_extent,
            SPPFNode::Nonterminal(n) => n.span.right_extent,
            SPPFNode::Intermediate(i) => i.span.right_extent,
        }
    }

    pub fn is_ambiguous(&self) -> bool {
        match self {
            SPPFNode::Terminal(_) => false,
            SPPFNode::Nonterminal(n) => n.ambiguous,
            SPPFNode::Intermediate(i) => i.ambiguous,
        }
    }
}

impl Placeholder for SPPFNode {
    fn placeholder() -> Self {
        SPPFNode::Terminal(TerminalNode {
            terminal_id: TerminalId(0),
            span: Span::default(),
        })
    }
}

// The node structs are `repr(C)` with `span` first, so that `span` is at the
// same offset in every `SPPFNode` variant and `left_extent`/`right_extent`
// compile to a single load. Without it, the compiler moves the `NonZeroU32`
// child ids to the front for their niche, `span` lands at a different offset
// in each variant, and every extent read computes its address from the tag.
#[derive(Debug)]
#[repr(C)]
pub struct TerminalNode {
    pub span: Span,
    pub terminal_id: TerminalId,
}

#[derive(Debug)]
#[repr(C)]
pub struct NonterminalNode {
    pub span: Span,
    pub child: SPPFNodeId,
    pub nonterminal_id: NonterminalId,
    /// Corresponds to the grammar position at the end of an alternative, from which
    /// the `child` node is attached to this nonterminal node.
    pub return_slot: SlotId,
    pub ambiguous: bool,
}

// `env` comes before `slot_id` and `ambiguous`. `SPPFNode` stores its tag in
// the unused values of `ambiguous`, which works only while `ambiguous` lies
// outside the 20 bytes of a `NonterminalNode`. With `env` last, the tag would
// take 4 more bytes.
#[derive(Debug)]
#[repr(C)]
pub struct IntermediateNode {
    pub span: Span,
    pub child: (SPPFNodeId, SPPFNodeId),
    /// The env of the derivation that created the node. Only the
    /// intermediate-node index reads it, to keep the nodes of different envs
    /// apart.
    pub env: Option<EnvId>,
    pub slot_id: SlotId,
    pub ambiguous: bool,
}

const _: () = assert!(size_of::<SPPFNode>() == 24);

// The initial capacities of the SPPF-node indexes, as divisors of the input
// length. An index that outgrows its capacity rehashes, and an index that is
// too large takes extra memory.
const TERMINAL_NODE_INDEX_CAPACITY_DIVISOR: usize = 2;
const INTERMEDIATE_NODE_INDEX_CAPACITY_DIVISOR: usize = 1;

/// Maps a pair of terminal_id and span to the id of the terminal node with
/// that terminal and span.
pub struct TerminalNodeIndex<'arena> {
    table: ArenaTable<'arena, SPPFNodeId>,
}

impl<'arena> TerminalNodeIndex<'arena> {
    pub fn new(input_len: usize, arena: &'arena Arena) -> Self {
        Self {
            table: arena.table_with_capacity(input_len / TERMINAL_NODE_INDEX_CAPACITY_DIVISOR),
        }
    }

    /// The id of the terminal node for the pair `terminal_id` and `span`, or
    /// `None` if the index does not have a terminal node for that pair.
    ///
    /// The index stores only the node ids. It reads the terminal and the span
    /// of an indexed node from the node itself, so `get` takes the vector that
    /// holds the SPPF nodes.
    #[inline]
    pub fn get(
        &self,
        sppf_nodes: &ArenaIdVec<'arena, SPPFNodeId, SPPFNode>,
        terminal_id: TerminalId,
        span: Span,
    ) -> Option<SPPFNodeId> {
        self.table
            .find(Self::hash(terminal_id, span.left_extent), |&id| {
                matches!(&sppf_nodes[id], SPPFNode::Terminal(node)
                    if node.terminal_id == terminal_id && node.span == span)
            })
            .copied()
    }

    /// Records `node_id` corresponding to the pair `terminal_id` and `span`.
    /// The index does not check for the existence of the pair, so the caller
    /// should make sure the index does not already have a terminal node for
    /// the pair before inserting. The node `node_id` in `sppf_nodes` must be a
    /// terminal node with that terminal and span.
    #[inline]
    pub fn insert(
        &mut self,
        sppf_nodes: &ArenaIdVec<'arena, SPPFNodeId, SPPFNode>,
        terminal_id: TerminalId,
        span: Span,
        node_id: SPPFNodeId,
    ) {
        debug_assert!(matches!(&sppf_nodes[node_id], SPPFNode::Terminal(node)
            if node.terminal_id == terminal_id && node.span == span));
        self.table
            .insert_unique(Self::hash(terminal_id, span.left_extent), node_id, |&id| {
                match &sppf_nodes[id] {
                    SPPFNode::Terminal(node) => Self::hash(node.terminal_id, node.span.left_extent),
                    _ => unreachable!("the terminal-node index holds only terminal nodes"),
                }
            });
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// The hash of a pair: the Fx hash of a `u64` that holds the terminal id in
    /// its high 32 bits and the left extent in its low 32 bits. The right
    /// extent is not part of the hash, because the scanner matches a terminal
    /// at a position with at most one right extent.
    #[inline]
    fn hash(terminal_id: TerminalId, left_extent: u32) -> u64 {
        FxBuildHasher.hash_one(((terminal_id.index() as u64) << 32) | u64::from(left_extent))
    }
}

/// Maps a triple of slot_id, span, and env to the id of the intermediate node
/// with that slot, span, and env.
pub struct IntermediateNodeIndex<'arena> {
    table: ArenaTable<'arena, SPPFNodeId>,
}

impl<'arena> IntermediateNodeIndex<'arena> {
    pub fn new(input_len: usize, arena: &'arena Arena) -> Self {
        Self {
            table: arena.table_with_capacity(input_len / INTERMEDIATE_NODE_INDEX_CAPACITY_DIVISOR),
        }
    }

    /// The id of the intermediate node for the triple `slot_id`, `span`, and
    /// `env`, or `None` if the index does not have an intermediate node for
    /// that triple.
    ///
    /// The index stores only the node ids. It reads the slot, the span, and
    /// the env of an indexed node from the node itself, so `get` takes the
    /// vector that holds the SPPF nodes.
    #[inline]
    pub fn get(
        &self,
        sppf_nodes: &ArenaIdVec<'arena, SPPFNodeId, SPPFNode>,
        slot_id: SlotId,
        span: Span,
        env: Option<EnvId>,
    ) -> Option<SPPFNodeId> {
        self.table
            .find(Self::hash(slot_id, span, env), |&id| {
                matches!(&sppf_nodes[id], SPPFNode::Intermediate(node)
                    if node.slot_id == slot_id && node.span == span && node.env == env)
            })
            .copied()
    }

    /// Records `node_id` corresponding to the triple `slot_id`, `span`, and
    /// `env`. The index does not check for the existence of the triple, so the
    /// caller should make sure the index does not already have an intermediate
    /// node for the triple before inserting. The node `node_id` in
    /// `sppf_nodes` must be an intermediate node with that slot, span, and env.
    #[inline]
    pub fn insert(
        &mut self,
        sppf_nodes: &ArenaIdVec<'arena, SPPFNodeId, SPPFNode>,
        slot_id: SlotId,
        span: Span,
        env: Option<EnvId>,
        node_id: SPPFNodeId,
    ) {
        debug_assert!(matches!(&sppf_nodes[node_id], SPPFNode::Intermediate(node)
            if node.slot_id == slot_id && node.span == span && node.env == env));
        self.table.insert_unique(
            Self::hash(slot_id, span, env),
            node_id,
            |&id| match &sppf_nodes[id] {
                SPPFNode::Intermediate(node) => Self::hash(node.slot_id, node.span, node.env),
                _ => unreachable!("the intermediate-node index holds only intermediate nodes"),
            },
        );
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// The hash of a triple of slot_id, span, and env.
    #[inline]
    fn hash(slot_id: SlotId, span: Span, env: Option<EnvId>) -> u64 {
        // The goal is the fewest hasher writes, because FxHasher does one
        // multiply per write. The triple fits in two `u64` values:
        // - The span is one `u64`. `Span::hash` writes the left extent in its
        //   high 32 bits and the right extent in its low 32 bits.
        // - The slot id, a `u16`, goes in the high 32 bits of the other `u64`.
        // - The env goes in the low 32 bits of that `u64`. `EnvId` is a
        //   `NonZeroU32`, so `None` becomes 0, which no `EnvId` has.
        // Hashing the triple as a tuple writes each field and the `Option`
        // discriminant separately.
        let env = env.map_or(0, |env| env.index() as u64);
        FxBuildHasher.hash_one(((u64::from(slot_id.0) << 32) | env, span))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arena::Arena,
        ids::{EnvId, Id, SPPFNodeId, SlotId, TerminalId},
        input::Span,
        sppf::{
            IntermediateNode, IntermediateNodeIndex, SPPFNode, TerminalNode, TerminalNodeIndex,
        },
    };

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
            let id = sppf_nodes.push(SPPFNode::Terminal(TerminalNode { span, terminal_id }));
            index.insert(&sppf_nodes, terminal_id, span, id);
            id
        };
        let a = add(TerminalId(1), Span::new(3, 5));
        let b = add(TerminalId(2), Span::new(3, 5));
        let c = add(TerminalId(1), Span::new(3, 6));
        let d = add(TerminalId(1), Span::new(4, 5));

        assert_eq!(
            index.get(&sppf_nodes, TerminalId(1), Span::new(3, 5)),
            Some(a)
        );
        assert_eq!(
            index.get(&sppf_nodes, TerminalId(2), Span::new(3, 5)),
            Some(b)
        );
        assert_eq!(
            index.get(&sppf_nodes, TerminalId(1), Span::new(3, 6)),
            Some(c)
        );
        assert_eq!(
            index.get(&sppf_nodes, TerminalId(1), Span::new(4, 5)),
            Some(d)
        );
        assert_eq!(index.get(&sppf_nodes, TerminalId(2), Span::new(3, 6)), None);
        assert_eq!(index.len(), 4);
    }

    #[test]
    fn intermediate_nodes_differ_by_slot_span_and_env() {
        let arena = Arena::new();
        let mut sppf_nodes = arena.id_vec();
        let mut index = IntermediateNodeIndex::new(16, &arena);
        let mut add = |slot_id, span, env| {
            let id = sppf_nodes.push(intermediate_node(slot_id, span, env));
            index.insert(&sppf_nodes, slot_id, span, env, id);
            id
        };
        let env = Some(EnvId::from_index(1));
        let other_env = Some(EnvId::from_index(2));
        let a = add(SlotId(1), Span::new(0, 4), None);
        let b = add(SlotId(1), Span::new(0, 4), env);
        let c = add(SlotId(1), Span::new(0, 4), other_env);
        let d = add(SlotId(2), Span::new(0, 4), None);
        let e = add(SlotId(1), Span::new(1, 4), None);

        let get = |slot_id, span, env| index.get(&sppf_nodes, slot_id, span, env);
        assert_eq!(get(SlotId(1), Span::new(0, 4), None), Some(a));
        assert_eq!(get(SlotId(1), Span::new(0, 4), env), Some(b));
        assert_eq!(get(SlotId(1), Span::new(0, 4), other_env), Some(c));
        assert_eq!(get(SlotId(2), Span::new(0, 4), None), Some(d));
        assert_eq!(get(SlotId(1), Span::new(1, 4), None), Some(e));
        assert_eq!(get(SlotId(2), Span::new(0, 4), env), None);
        assert_eq!(index.len(), 5);
    }

    #[test]
    fn indexes_grow_past_their_initial_capacity() {
        let arena = Arena::new();
        let mut sppf_nodes = arena.id_vec();
        let mut terminal_index = TerminalNodeIndex::new(0, &arena);
        let mut intermediate_index = IntermediateNodeIndex::new(0, &arena);
        let mut ids = Vec::new();
        for i in 0..100u32 {
            let terminal_id = TerminalId((i % 7) as u16);
            let terminal = sppf_nodes.push(SPPFNode::Terminal(TerminalNode {
                span: Span::new(i, i + 1),
                terminal_id,
            }));
            terminal_index.insert(&sppf_nodes, terminal_id, Span::new(i, i + 1), terminal);
            let slot_id = SlotId((i % 5) as u16);
            let intermediate = sppf_nodes.push(intermediate_node(slot_id, Span::new(0, i), None));
            intermediate_index.insert(&sppf_nodes, slot_id, Span::new(0, i), None, intermediate);
            ids.push((terminal, intermediate));
        }
        for i in 0..100u32 {
            let (terminal, intermediate) = ids[i as usize];
            assert_eq!(
                terminal_index.get(&sppf_nodes, TerminalId((i % 7) as u16), Span::new(i, i + 1)),
                Some(terminal)
            );
            assert_eq!(
                intermediate_index.get(&sppf_nodes, SlotId((i % 5) as u16), Span::new(0, i), None),
                Some(intermediate)
            );
        }
        assert_eq!(terminal_index.len(), 100);
        assert_eq!(intermediate_index.len(), 100);
    }
}
