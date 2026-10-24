use std::num::NonZeroU32;

use proc_macro2::Literal;
use serde::{Deserialize, Serialize};
use specta::Type;

/// A unique identifier for a nonterminal in the grammar.
///
/// This is a type-safe wrapper around an index into the grammar's nonterminal list.
/// Uses `u16` since real-world grammars rarely exceed a few hundred nonterminals
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct NonterminalId(pub u16);

impl NonterminalId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for NonterminalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl quote::ToTokens for NonterminalId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = Literal::u16_unsuffixed(self.0);
        tokens.extend(quote::quote! { NonterminalId(#id) });
    }
}

/// A unique identifier for a grammar slot. Grammar slots of of the form A → ⍺ . β, similar
/// to LR items.
///
/// This is a type-safe wrapper around an index into the grammar's grammar slot list.
/// Uses `u16` since real-world grammars rarely exceed a few thousand grammar slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SlotId(pub u16);

impl SlotId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }

    /// The id of the next slot, which moves the dot one symbol forward in the
    /// alternative. The generator numbers the slots of an alternative
    /// consecutively in position order, so the next slot is the one with the
    /// next higher id. The slot must not be at the end of its alternative.
    #[inline(always)]
    pub fn next(self) -> SlotId {
        SlotId(self.0 + 1)
    }
}

impl std::fmt::Display for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl quote::ToTokens for SlotId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = Literal::u16_unsuffixed(self.0);
        tokens.extend(quote::quote! { SlotId(#id) });
    }
}

/// A unique identifier for a terminal in the grammar.
///
/// This is a type-safe wrapper around an index into the grammar's terminal list.
/// Uses `u16` since real-world grammars rarely exceed a few hundred terminals.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalId(pub u16);

impl TerminalId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for TerminalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl quote::ToTokens for TerminalId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = Literal::u16_unsuffixed(self.0);
        tokens.extend(quote::quote! { TerminalId(#id) });
    }
}

/// A unique identifier for a binding name in the grammar.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BindingId(pub u8);

impl BindingId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for BindingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl quote::ToTokens for BindingId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = Literal::u8_unsuffixed(self.0);
        tokens.extend(quote::quote! { BindingId(#id) });
    }
}

/// An id that addresses an element of an [`IdVec`](crate::utils::id_vec::IdVec).
/// Ids start at 1. This is to allow niche optimization for the `Option<Id>` types.
pub trait Id: Copy {
    /// The id of the element at `index`, which must not be 0.
    fn from_index(index: usize) -> Self;

    /// The index of the element this id addresses.
    fn index(self) -> usize;
}

/// A unique identifier for a GSS node.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct GssNodeId(NonZeroU32);

impl Id for GssNodeId {
    #[inline]
    fn from_index(index: usize) -> Self {
        let index = u32::try_from(index).ok().and_then(NonZeroU32::new);
        Self(index.expect("an id's index is between 1 and u32::MAX"))
    }

    #[inline]
    fn index(self) -> usize {
        self.0.get() as usize
    }
}

impl std::fmt::Display for GssNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for an SPPF node.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SPPFNodeId(NonZeroU32);

impl Id for SPPFNodeId {
    #[inline]
    fn from_index(index: usize) -> Self {
        let index = u32::try_from(index).ok().and_then(NonZeroU32::new);
        Self(index.expect("an id's index is between 1 and u32::MAX"))
    }

    #[inline]
    fn index(self) -> usize {
        self.0.get() as usize
    }
}

impl std::fmt::Display for SPPFNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for an environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct EnvId(NonZeroU32);

impl Id for EnvId {
    #[inline]
    fn from_index(index: usize) -> Self {
        let index = u32::try_from(index).ok().and_then(NonZeroU32::new);
        Self(index.expect("an id's index is between 1 and u32::MAX"))
    }

    #[inline]
    fn index(self) -> usize {
        self.0.get() as usize
    }
}
