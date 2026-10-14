use crate::ids::{NonterminalId, SPPFNodeId, SlotId, TerminalId};
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

#[derive(Debug)]
#[repr(C)]
pub struct IntermediateNode {
    pub span: Span,
    pub child: (SPPFNodeId, SPPFNodeId),
    pub slot_id: SlotId,
    pub ambiguous: bool,
}
