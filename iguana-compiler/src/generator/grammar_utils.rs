use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;
use syn::Ident;

use crate::generator::GenConfig;
use crate::grammar::{
    def::Grammar,
    first_follow::FirstFollowSets,
    symbols::{Definition, DefinitionId, Nonterminal},
};

use crate::utils::{to_pascal_case, to_snake_case};

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
/// Start nonterminals: `Start<Token, &'a Layout<'a>>` or `Start<&'a Inner<'a>, &'a Layout<'a>>`;
/// the layout type is `()` when the grammar declares no layout and for the
/// wrapper of the layout nonterminal.
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
        let layout = match grammar.start_layout(nonterminal) {
            Some(l) => {
                let layout_ident = l.as_identifier().unwrap();
                symbol_type(grammar, layout_ident.resolve(), unsafe_mode)
            }
            None => quote! { () },
        };
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

/// Returns the identifier of the generated grammar type, the `Grammar` trait
/// implementation, with the grammar name in PascalCase.
pub fn grammar_ident(grammar_name: &str) -> Ident {
    format_ident!("{}Grammar", to_pascal_case(grammar_name))
}

/// Returns the identifier of the generated parser type, with the grammar name
/// in PascalCase. Every generator that names the type calls this function, so
/// the definition and the references to it agree.
pub fn parser_ident(grammar_name: &str) -> Ident {
    format_ident!("{}Parser", to_pascal_case(grammar_name))
}

/// Returns the identifier of the generated scanner type, with the grammar name
/// in PascalCase.
pub fn scanner_ident(grammar_name: &str) -> Ident {
    format_ident!("{}Scanner", to_pascal_case(grammar_name))
}

/// Returns the identifier of the generated parse tree builder type, with the
/// grammar name in PascalCase.
pub fn parse_tree_builder_ident(grammar_name: &str) -> Ident {
    format_ident!("{}ParseTreeBuilder", to_pascal_case(grammar_name))
}

/// Returns the identifier of the `Prediction` static of a nonterminal, for
/// example `PREDICTION_RULE` for `Rule`. The grammar module defines the
/// static, and the parser refers to it.
pub fn prediction_ident(nonterminal: &Nonterminal) -> Ident {
    format_ident!(
        "PREDICTION_{}",
        to_snake_case(&nonterminal.name).to_uppercase()
    )
}

/// The nonterminals parsed by an LL(1) function: the LL(1) nonterminals when
/// the `ll1` option is on, and none when it is off. The parser calls every
/// other nonterminal through the GLL path.
pub fn ll1_nonterminals<'a>(
    grammar: &'a Grammar,
    ff: &FirstFollowSets,
    config: &GenConfig,
) -> FxHashSet<&'a Nonterminal> {
    if !config.ll1_optimization {
        return FxHashSet::default();
    }
    grammar.nonterminals().filter(|nt| ff.is_ll1(nt)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iggy::parse_grammar;

    /// The names of the nonterminals parsed by an LL(1) function, sorted.
    fn ll1_names(source: &str, ll1_optimization: bool) -> Vec<String> {
        let grammar: Grammar = parse_grammar(source).unwrap().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);
        let config = GenConfig {
            ll1_optimization,
            ..GenConfig::default()
        };
        let mut names: Vec<_> = ll1_nonterminals(&grammar, &ff, &config)
            .into_iter()
            .map(|nt| nt.name.clone())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn ll1_nonterminals_are_empty_without_the_ll1_option() {
        // A is not LL(1), so neither is S, which reaches it. `C?` becomes the
        // nullable nonterminal Opt_0.
        let source = "grammar G\nS = A B C?\nA = \"a\" | \"a\" \"x\"\nB = \"b\"\nC = \"c\"\n";
        assert!(ll1_names(source, false).is_empty());
        assert_eq!(
            ll1_names(source, true),
            ["B", "C", "Opt_0", "StartB", "StartC"]
        );
    }
}
