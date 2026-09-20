use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;

use crate::generator::grammar_utils::grammar_ident;
use crate::generator::id::{NonterminalIds, SlotIds, TerminalIds};
use crate::generator::terminal_sets::{TerminalSet, TerminalSetKind};
use crate::grammar::def::Grammar;
use crate::grammar::symbols::{Definition, Nonterminal};
use crate::utils::to_snake_case;
use iguana_runtime::ids::TerminalId;

/// Generates `grammar.rs`: the nonterminal id constants, the grammar type
/// with its `Grammar` implementation, and the terminal sets the parser passes
/// to the scanner or refers to in failures. The sets are public because the
/// parser references only some of them, and a private unused static fails the
/// denied dead-code lint.
pub fn generate<'a>(
    grammar: &'a Grammar,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds<'a>,
    terminal_sets: &[TerminalSet],
) -> TokenStream {
    let grammar_type = grammar_ident(&grammar.name);
    let grammar_name = &grammar.name;

    let terminal_set_statics = terminal_set_statics(grammar, terminal_ids, terminal_sets);
    let single_terminal_sets = single_terminal_sets(terminal_ids, terminal_sets);

    let nonterminals = nonterminal_ids.nonterminals().map(|n| {
        let nonterminal_name = &n.name;
        let display_name = n.display_name();
        quote! {
            Nonterminal {
                name: #nonterminal_name,
                display_name: #display_name,
            }
        }
    });

    let layout_name = grammar
        .layout
        .as_ref()
        .and_then(|s| s.as_identifier())
        .map(|i| i.name.as_str());

    // The entry points, in `.iggy` source order: the layout nonterminal and
    // the derived nonterminals (start wrappers, EBNF expansions, exclusion and
    // precedence desugarings) have no start wrapper and are left out, and the
    // rest sort by declaration position.
    let mut display_order: Vec<&Nonterminal> = nonterminal_ids
        .nonterminals()
        .filter(|n| !n.is_derived() && Some(n.name.as_str()) != layout_name)
        .collect();
    display_order.sort_by_key(|n| grammar.source_index(&n.name));
    let display_order_names: Vec<&str> = display_order.iter().map(|n| n.name.as_str()).collect();

    let nonterminal_id_consts: Vec<_> = nonterminal_ids
        .nonterminals()
        .enumerate()
        .map(|(i, n)| {
            let const_name = format_ident!("{}", to_snake_case(&n.name).to_uppercase());
            let index = Literal::usize_unsuffixed(i);
            quote! { pub const #const_name: NonterminalId = NonterminalId(#index); }
        })
        .collect();

    let nonterminal_id_arms: Vec<_> = nonterminal_ids
        .nonterminals()
        .map(|n| {
            let name = &n.name;
            let const_name = format_ident!("{}", to_snake_case(name).to_uppercase());
            quote! { #name => Some(#const_name), }
        })
        .collect();

    let terminals: Vec<_> = terminal_ids
        .terminals()
        .map(|t| {
            let terminal_name = &t.name;
            quote! { Terminal { name: #terminal_name } }
        })
        .collect();

    let slots = slot_ids.slots().map(|s| {
        let display_name = slot_ids.display_name(&slot_ids.get_id(s));
        quote! {
            Slot { display_name: #display_name }
        }
    });

    let layout_name = layout_name
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! { None });
    let layout_terminals = layout_terminal_ids(grammar, terminal_ids);

    quote! {
        use iguana_runtime::{
            grammar::{Grammar, Nonterminal, Slot, Terminal},
            ids::{NonterminalId, TerminalId},
            scanner::TerminalSet,
        };

        #(#nonterminal_id_consts)*

        pub struct #grammar_type;

        impl Grammar for #grammar_type {
            const NAME: &'static str = #grammar_name;

            const NONTERMINALS: &'static [Nonterminal] = &[#(#nonterminals),*];

            const DISPLAY_ORDER: &'static [&'static str] = &[#(#display_order_names),*];

            const TERMINALS: &'static [Terminal] = &[
                #(#terminals,)*
                Terminal { name: "Epsilon" },
                Terminal { name: "EOF" },
            ];

            const SLOTS: &'static [Slot] = &[#(#slots),*];

            const LAYOUT_NAME: Option<&'static str> = #layout_name;

            const LAYOUT_TERMINALS: &'static [TerminalId] = &[#(#layout_terminals),*];

            #[comment = "A failed terminal match refers to a static terminal set like every other
                         failure, so the error reporting path needs to reach a terminal set
                         from a terminal id. This slice, indexed by terminal id, serves only that."]
            const SINGLE_TERMINAL_SETS: &'static [TerminalSet] = &[#(#single_terminal_sets),*];

            fn nonterminal_id(name: &str) -> Option<NonterminalId> {
                match name {
                    #(#nonterminal_id_arms)*
                    _ => None,
                }
            }
        }

        #(#terminal_set_statics)*
    }
}

/// The statics for the terminal sets, for example:
///
///     // Grammar { WS, LineComment, EOF }
///     pub static FOLLOW_SET_GRAMMAR: TerminalSet = TerminalSet {
///         id: 0,
///         terminals: &[TerminalId(8), TerminalId(10), TerminalId(39)],
///     };
///
/// The parser passes these sets to the scanner for matching, and a recorded
/// failure refers to the set the parser expected. What each kind of set
/// contains and who uses it is documented on `TerminalSetKind`.
fn terminal_set_statics(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    terminal_sets: &[TerminalSet],
) -> Vec<TokenStream> {
    terminal_sets
        .iter()
        .map(|set| {
            let name = format_ident!("{}", terminal_set_name(set, grammar));
            let comment = terminal_set_comment(set);
            let ids: Vec<_> = set
                .terminals
                .iter()
                .map(|t| terminal_ids.get_id(t))
                .collect();
            let set_id = Literal::usize_unsuffixed(set.id);
            quote! {
                #[comment = #comment]
                pub static #name: TerminalSet = TerminalSet { id: #set_id, terminals: &[#(#ids),*] };
            }
        })
        .collect()
}

/// `SINGLE_TERMINAL_SETS` is a slice of `TerminalSet`s, each holding a single
/// terminal: entry `i` holds the set for the terminal with id `i`. Failure
/// reporting only accepts a `TerminalSet`, and this slice is the way to reach a
/// terminal set from a single terminal id. These sets are not used for
/// matching by the scanner.
fn single_terminal_sets(terminal_ids: &TerminalIds, terminal_sets: &[TerminalSet]) -> Vec<TokenStream> {
    let first_id = terminal_sets.iter().map(|set| set.id + 1).max().unwrap_or(0);
    // Plus the synthetic epsilon and EOF terminals: this slice is indexed like
    // `TERMINALS`, so it must contain those two as well.
    (0..terminal_ids.len() + 2)
        .map(|index| {
            let set_id = Literal::usize_unsuffixed(first_id + index);
            let id = TerminalId(index as u16);
            quote! { TerminalSet { id: #set_id, terminals: &[#id] } }
        })
        .collect()
}

/// The ids of the terminals reachable from the grammar's layout definition,
/// sorted by id. Empty when the grammar declares no layout.
fn layout_terminal_ids(grammar: &Grammar, terminal_ids: &TerminalIds) -> Vec<TerminalId> {
    let Some(layout) = grammar.layout.as_ref().and_then(|s| s.as_identifier()) else {
        return vec![];
    };

    let mut pending = vec![layout.resolve()];
    let mut visited = FxHashSet::default();
    let mut terminals = vec![];
    while let Some(definition_id) = pending.pop() {
        if !visited.insert(definition_id) {
            continue;
        }
        match grammar.definition(definition_id) {
            Definition::Terminal(terminal) => {
                terminals.push(terminal_ids.get_id(terminal));
            }
            Definition::Nonterminal(nonterminal) => {
                for alternative in grammar.alternatives(nonterminal) {
                    for symbol in &alternative.symbols {
                        if let Some(identifier) = symbol.as_identifier() {
                            pending.push(identifier.resolve());
                        }
                    }
                }
            }
        }
    }
    terminals.sort_unstable_by_key(|terminal| terminal.0);
    terminals
}

/// Identifier of the static for `set`, e.g. `FIRST_SET_E_ALT0`.
fn terminal_set_name(set: &TerminalSet, grammar: &Grammar) -> String {
    let upper = |nonterminal: &Nonterminal| to_snake_case(&nonterminal.name).to_uppercase();
    match &set.kind {
        TerminalSetKind::Follow(nonterminal) => format!("FOLLOW_SET_{}", upper(nonterminal)),
        TerminalSetKind::First(nonterminal) => format!("FIRST_SET_{}", upper(nonterminal)),
        TerminalSetKind::FirstAlt(slot) => format!(
            "FIRST_SET_{}_ALT{}",
            upper(slot.head()),
            slot.alternative_index(grammar)
        ),
        TerminalSetKind::FollowRestriction(slot) => format!(
            "FOLLOW_RESTRICTION_{}_ALT{}_POS{}",
            upper(slot.head()),
            slot.alternative_index(grammar),
            slot.pos()
        ),
        TerminalSetKind::LayoutAwareFollowRestriction(slot) => format!(
            "LAYOUT_AWARE_FOLLOW_RESTRICTION_{}_ALT{}_POS{}",
            upper(slot.head()),
            slot.alternative_index(grammar),
            slot.pos()
        ),
        TerminalSetKind::Except(slot) => format!(
            "EXCEPT_{}_ALT{}_POS{}",
            upper(slot.head()),
            slot.alternative_index(grammar),
            slot.pos()
        ),
        TerminalSetKind::Prediction(nonterminal) => format!("PREDICTION_SET_{}", upper(nonterminal)),
    }
}

/// Comment line above the static for `set`: the grammar position, then the
/// terminals, e.g. `E : . E "+" E { "a" }`.
fn terminal_set_comment(set: &TerminalSet) -> String {
    let position = match &set.kind {
        TerminalSetKind::Follow(nonterminal) | TerminalSetKind::First(nonterminal) => {
            nonterminal.name.clone()
        }
        TerminalSetKind::FirstAlt(slot) => slot.name(),
        TerminalSetKind::FollowRestriction(slot) => format!("{} !>>", slot.name()),
        TerminalSetKind::LayoutAwareFollowRestriction(slot) => format!("{} !>>>", slot.name()),
        TerminalSetKind::Except(slot) => format!("{} \\", slot.name()),
        TerminalSetKind::Prediction(nonterminal) => format!("{} prediction", nonterminal.name),
    };
    let names: Vec<_> = set.terminals.iter().map(|t| t.name.clone()).collect();
    format!("{position} {{ {} }}", names.join(", "))
}
