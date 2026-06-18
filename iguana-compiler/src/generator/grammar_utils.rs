use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::grammar::{
    def::Grammar,
    symbols::{Definition, DefinitionId, Nonterminal},
};

use crate::utils::to_pascal_case;

/// Returns the parse tree type for a nonterminal.
/// Start nonterminals: `Start<Token, &'a Layout<'a>>` or `Start<&'a Inner<'a>, &'a Layout<'a>>`.
/// Regular nonterminals: `Ident<'a>`. Every generated nonterminal enum
/// carries `'a` for its `Amb(&'a [&'a Self<'a>])` variant.
pub fn nonterminal_type(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    if grammar.is_start(nonterminal) {
        let inner_ident = nonterminal
            .origin
            .as_ref()
            .unwrap()
            .as_identifier()
            .unwrap();
        let inner = symbol_type(grammar, inner_ident.resolve());
        let layout_ident = grammar.layout.as_ref().unwrap().as_identifier().unwrap();
        let layout = symbol_type(grammar, layout_ident.resolve());
        quote! { Start<#inner, #layout> }
    } else {
        let ident = nt_ident(&nonterminal.name);
        quote! { #ident<'a> }
    }
}

/// Returns the type of a symbol as it appears in parse tree fields:
/// `Token` for terminals (inline, Copy), `&'a T<'a>` for nonterminals (by reference).
pub fn symbol_type(grammar: &Grammar, def_id: DefinitionId) -> TokenStream {
    match grammar.definition(def_id) {
        Definition::Terminal(_) => quote! { Token },
        Definition::Nonterminal(nt) => {
            let ty = nonterminal_type(grammar, nt);
            quote! { &'a #ty }
        }
    }
}

/// Returns the PascalCase identifier for a nonterminal name.
pub fn nt_ident(name: &str) -> Ident {
    format_ident!("{}", to_pascal_case(name))
}

/// Returns the PascalCase type name for a nonterminal name.
pub fn nonterminal_type_name(name: &str) -> String {
    to_pascal_case(name)
}
