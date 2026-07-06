use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::grammar::{
    def::Grammar,
    symbols::{Definition, DefinitionId, Nonterminal},
};

use crate::utils::to_pascal_case;

/// True when the generated enum for a nonterminal has a lifetime.
/// Normally, every enum takes a lifetime because of its `Amb(&'a [&'a Self])` variant.
/// The unsafe mode drops `Amb`, so only an enum with a nonterminal-typed field (`&'a T`) takes `'a`, and
/// a token-only nonterminal, one whose alternatives hold only tokens, e.g., `Mod = "public" | "static"`,
/// or nothing, e.g., `Empty = ()`, does not.
pub fn nonterminal_has_lifetime(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
    unsafe_mode: bool,
) -> bool {
    !unsafe_mode
        || grammar.alternatives(nonterminal).iter().any(|alt| {
            alt.symbols
                .iter()
                // Only parse-tree symbols become enum fields.
                .filter(|s| s.is_parse_tree_symbol())
                .any(|s| {
                    matches!(
                        grammar.definition(s.resolved_def()),
                        Definition::Nonterminal(_)
                    )
                })
        })
}

/// Returns the parse tree type for a nonterminal.
/// Start nonterminals: `Start<Token, &'a Layout<'a>>` or `Start<&'a Inner<'a>, &'a Layout<'a>>`.
/// Regular nonterminals: the nonterminal's own type, with `<'a>` when the
/// enum has a lifetime (see [`nonterminal_has_lifetime`]).
pub fn nonterminal_type(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
    unsafe_mode: bool,
) -> TokenStream {
    if grammar.is_start(nonterminal) {
        let inner_ident = nonterminal
            .origin
            .as_ref()
            .unwrap()
            .as_identifier()
            .unwrap();
        let inner = symbol_type(grammar, inner_ident.resolve(), unsafe_mode);
        let layout_ident = grammar.layout.as_ref().unwrap().as_identifier().unwrap();
        let layout = symbol_type(grammar, layout_ident.resolve(), unsafe_mode);
        quote! { Start<#inner, #layout> }
    } else {
        let ident = nt_ident(&nonterminal.name);
        if nonterminal_has_lifetime(grammar, nonterminal, unsafe_mode) {
            quote! { #ident<'a> }
        } else {
            quote! { #ident }
        }
    }
}

/// Returns the type of a symbol as it appears in parse tree fields:
/// `Token` for terminals (inline, Copy), `&'a T<'a>` or `&'a T` for
/// nonterminals (by reference).
pub fn symbol_type(grammar: &Grammar, def_id: DefinitionId, unsafe_mode: bool) -> TokenStream {
    match grammar.definition(def_id) {
        Definition::Terminal(_) => quote! { Token },
        Definition::Nonterminal(nt) => {
            let ty = nonterminal_type(grammar, nt, unsafe_mode);
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
