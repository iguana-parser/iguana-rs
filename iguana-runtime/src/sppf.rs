use crate::ids::{NonterminalId, SlotId, TerminalId};
use crate::input::Span;
use serde::{Deserialize, Serialize};
use specta::Type;

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

#[derive(Debug)]
pub struct TerminalNode {
    pub terminal_id: TerminalId,
    pub span: Span,
}

impl TerminalNode {
    pub fn new(terminal_id: TerminalId, span: Span) -> Self {
        Self { terminal_id, span }
    }
}

#[derive(Debug)]
pub struct NonterminalNode {
    pub nonterminal_id: NonterminalId,
    /// Corresponds to the grammar position at the end of an alternative, from which,
    /// the `child` node is attache to this nonterminal node.
    pub return_slot: SlotId,
    pub span: Span,
    pub ambiguous: bool,
    pub child: SPPFNodeId,
}

#[derive(Debug)]
pub struct IntermediateNode {
    pub slot_id: SlotId,
    pub span: Span,
    pub ambiguous: bool,
    pub child: (SPPFNodeId, SPPFNodeId),
}

/// A unique identifier for an SPPF node.
///
/// This is a type-safe wrapper around an index into the parser's SPPF nodes list.
/// Uses `u32` since real-world grammars rarely exceed 2^32 - 1 nodes.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SPPFNodeId(pub u32);

impl SPPFNodeId {
    /// Sentinel for an absent id in dense arrays. Real ids must be < u32::MAX.
    pub const NONE: Self = Self(u32::MAX);

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for SPPFNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
