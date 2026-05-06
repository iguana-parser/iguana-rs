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

/// A unique identifier for a character class in the scanner.
///
/// This is a type-safe wrapper around an index into the scanner's character class list.
/// Uses `u16` since real-world grammars rarely exceed a few hundred character classes.
#[derive(Debug, Clone, Copy)]
pub struct CharClassId(pub u16);

impl CharClassId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

/// A unique identifier for a GSS node.
///
/// This is a type-safe wrapper around an index into the parser's GSS node list.
/// Uses `u32` since GSS nodes are of the form (nonterminal_id, input_index), which
/// is bounded by the input length (u32).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct GssNodeId(pub u32);

impl GssNodeId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for GssNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
