use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::grammar::{
    def::Grammar,
    symbols::{Definition, DefinitionId, Nonterminal, Symbol},
};

use super::utils::to_pascal_case;

/// Returns true if a nonterminal's generated type needs a lifetime parameter `'a`.
/// Nonterminal children are stored as `&'a T` references, which introduces `'a`.
/// Multi-alternative nonterminals always need `'a` because their Amb variant
/// references Self.
pub fn needs_lifetime(grammar: &Grammar, nonterminal: &Nonterminal) -> bool {
    let alternatives = grammar.alternatives(nonterminal);
    // Multi-alternative nonterminals have an Amb variant that references Self.
    if alternatives.len() > 1 {
        return true;
    }
    alternatives[0]
        .symbols
        .iter()
        .filter(|s| s.is_parse_tree_symbol())
        .any(|s| matches!(grammar.definition(s.resolved_def()), Definition::Nonterminal(_)))
}

/// Returns the parse tree type for a nonterminal, with lifetime if needed.
/// Start nonterminals: `Start<Token, &'a Layout<'a>>` or `Start<&'a Inner<'a>, &'a Layout<'a>>`.
/// Regular nonterminals: `Ident<'a>` or `Ident` depending on `needs_lifetime`.
pub fn nonterminal_type(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    if grammar.is_start(nonterminal) {
        let inner_ident = nonterminal.origin.as_ref().unwrap().as_identifier().unwrap();
        let inner = symbol_type(grammar, inner_ident.resolve());
        let layout_ident = grammar.layout.as_ref().unwrap().as_identifier().unwrap();
        let layout = symbol_type(grammar, layout_ident.resolve());
        quote! { Start<#inner, #layout> }
    } else {
        let ident = nt_ident(&nonterminal.name);
        if needs_lifetime(grammar, nonterminal) {
            quote! { #ident<'a> }
        } else {
            quote! { #ident }
        }
    }
}

/// Returns the type of a symbol as it appears in parse tree fields:
/// `Token` for terminals (inline, Copy), `&'a T<'a>` for nonterminals (by reference).
/// Exclude-derived nonterminals are resolved to their original type.
pub fn symbol_type(grammar: &Grammar, def_id: DefinitionId) -> TokenStream {
    match grammar.definition(def_id) {
        Definition::Terminal(_) => quote! { Token },
        Definition::Nonterminal(nt) => {
            let resolved = unwrap_exclude(grammar, nt);
            let ty = nonterminal_type(grammar, resolved);
            quote! { &'a #ty }
        }
    }
}

/// If the nonterminal is Exclude-derived, returns the original nonterminal
/// it was derived from. Otherwise returns the nonterminal itself.
pub fn unwrap_exclude<'a>(grammar: &'a Grammar, nt: &'a Nonterminal) -> &'a Nonterminal {
    if nt.is_exclude() {
        let name = match &nt.origin {
            Some(Symbol::Exclude { symbol, .. }) => {
                &symbol
                    .as_identifier()
                    .expect("Exclude origin should wrap an Identifier")
                    .name
            }
            _ => unreachable!("is_exclude() returned true but origin is not Exclude"),
        };
        grammar
            .nonterminal(name)
            .expect("Original nonterminal not found for Exclude")
    } else {
        nt
    }
}

/// Returns the PascalCase identifier for a nonterminal name.
pub fn nt_ident(name: &str) -> Ident {
    format_ident!("{}", to_pascal_case(name))
}

/// Returns the PascalCase type name for a nonterminal, mapping exclude-derived
/// nonterminals back to their original nonterminal's type.
pub fn nonterminal_type_name(grammar: &Grammar, name: &str) -> String {
    let resolved_name = match grammar.nonterminal(name) {
        Some(nt) => &unwrap_exclude(grammar, nt).name,
        _ => name,
    };
    to_pascal_case(resolved_name)
}
