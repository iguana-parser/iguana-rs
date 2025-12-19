use proc_macro2::Literal;

/// A unique identifier for a nonterminal in the grammar.
///
/// This is a type-safe wrapper around an index into the grammar's nonterminal list.
/// Uses `u16` since real-world grammars rarely exceed a few hundred nonterminals
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
