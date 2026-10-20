use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;
use syn::Ident;

use crate::generator::GenConfig;
use crate::grammar::{
    def::{Alternative, Grammar},
    first_follow::FirstFollowSets,
    slot::Slot,
    symbols::{Definition, DefinitionId, Nonterminal, Symbol},
};

use crate::utils::{to_pascal_case, to_snake_case};

/// Returns the parse tree type for a nonterminal.
/// Start nonterminals: `Start<Token, &'a Layout<'a>>` or `Start<&'a Inner<'a>, &'a Layout<'a>>`;
/// the layout type is `()` when the grammar declares no layout and for the
/// wrapper of the layout nonterminal.
/// Regular nonterminals: `Ident<'a>`. Every generated nonterminal enum
/// has `'a` for its `Amb(&'a [&'a Ident<'a>])` variant.
pub fn nonterminal_type(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    if grammar.is_start(nonterminal) {
        let inner_ident = nonterminal
            .origin
            .as_ref()
            .unwrap()
            .as_identifier()
            .unwrap();
        let inner = symbol_type(grammar, inner_ident.resolve());
        let layout = match grammar.start_layout(nonterminal) {
            Some(l) => {
                let layout_ident = l.as_identifier().unwrap();
                symbol_type(grammar, layout_ident.resolve())
            }
            None => quote! { () },
        };
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

/// Whether `slot` is the return slot of a nonterminal parsed by an LL(1)
/// function.
pub fn is_ll1_return_slot(
    grammar: &Grammar,
    ll1_nonterminals: &FxHashSet<&Nonterminal>,
    slot: &Slot,
) -> bool {
    slot.called_nonterminal(grammar)
        .is_some_and(|called| ll1_nonterminals.contains(called))
}

/// Whether the call at `pos` of `alternative` adds its binding of `name` to the
/// environment. The binding is added when a later symbol reads `name`, in a
/// condition, a call argument, or a return. The binding is not added when:
///
/// - No later symbol reads `name`.
/// - Only the conditions directly after the call read `name`. They read it from
///   the returned value.
/// - A later call shadows the binding: it binds `name` again before any
///   symbol reads it, as in `x=A x=B [x == 1]`, where the condition reads the
///   second `x`.
pub fn needs_env_binding(alternative: &Alternative, pos: usize, name: &str) -> bool {
    let symbols = &alternative.symbols;
    let return_conditions = symbols[pos + 1..]
        .iter()
        .take_while(|symbol| matches!(symbol, Symbol::Condition(_)))
        .count();
    for symbol in &symbols[pos + 1 + return_conditions..] {
        let reads = match symbol {
            Symbol::Condition(expr) | Symbol::Return(expr) => expr.reads(name),
            _ => symbol
                .call_arguments()
                .iter()
                .any(|argument| argument.reads(name)),
        };
        if reads {
            return true;
        }
        if symbol
            .binding_pattern()
            .is_some_and(|pattern| pattern.names().iter().any(|n| n == name))
        {
            return false;
        }
    }
    false
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

    //   E = "-" E #Neg
    //     > left E "+" E #Add
    //     > "~" E #Low
    //     | "a" #Lit
    //   S = "(" E !Add ")"
    //
    // After the desugarings, the call in S binds `x_pr` and `x_label` for the
    // label check after it. The left operand of #Add binds `l_pr` and `l_label`
    // for the precedence check after it. The right operands of #Neg and #Add
    // bind `r_pr` and `r_label`, and only `r_pr` is read past the conditions
    // after the call, by the return of the alternative.
    #[test]
    fn only_names_read_past_the_conditions_after_a_call_need_a_binding() {
        let source = "grammar G\nE = \"-\" E #Neg\n  > left E \"+\" E #Add\n  > \"~\" E #Low\n  | \"a\" #Lit\nS = \"(\" E !Add \")\"\n";
        let grammar: Grammar = parse_grammar(source).unwrap().try_into().unwrap();
        let mut bound = vec![];
        for nonterminal in grammar.nonterminals() {
            for alternative in grammar.alternatives(nonterminal) {
                for (pos, symbol) in alternative.symbols.iter().enumerate() {
                    let Some(pattern) = symbol.binding_pattern() else {
                        continue;
                    };
                    for name in pattern.names() {
                        bound.push((name.clone(), needs_env_binding(alternative, pos, name)));
                    }
                }
            }
        }
        bound.sort();
        let expected = [
            ("l_label", false),
            ("l_pr", false),
            ("r_label", false),
            ("r_label", false),
            ("r_pr", true),
            ("r_pr", true),
            ("x_label", false),
            ("x_pr", false),
        ];
        assert_eq!(bound, expected.map(|(name, read)| (name.to_string(), read)));
    }

    //   S = x=A x=B [x == 1]
    //   T = x=A x=B(x)
    //
    // The condition in S reads the second `x`, which shadows the first. The
    // argument of `B` in T reads the first `x`, because the arguments are
    // evaluated before `B` binds the name again.
    #[test]
    fn a_later_binding_of_the_same_name_shadows_the_first() {
        use crate::{alternative, bind, call, cond, eq, id};
        let shadowed = alternative!(
            bind!("x", id!("A")),
            bind!("x", id!("B")),
            cond!(eq!("x", 1))
        );
        assert!(!needs_env_binding(&shadowed, 0, "x"));
        let argument = alternative!(bind!("x", id!("A")), bind!("x", call!("B", "x")));
        assert!(needs_env_binding(&argument, 0, "x"));
    }
}
