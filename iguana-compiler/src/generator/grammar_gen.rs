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
/// with its `Grammar` implementation, and the terminal sets the parser matches
/// against. The sets are public because the parser references only some of
/// them, and a private unused static fails the denied dead-code lint.
pub fn generate<'a>(
    grammar: &'a Grammar,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds<'a>,
    terminal_sets: &[TerminalSet],
) -> TokenStream {
    let grammar_type = grammar_ident(&grammar.name);
    let grammar_name = &grammar.name;

    let mut terminal_set_items = vec![];
    for set in terminal_sets {
        let name = format_ident!("{}", terminal_set_name(set, grammar));
        let comment = terminal_set_comment(set);
        let ids: Vec<_> = set
            .terminals
            .iter()
            .map(|t| terminal_ids.get_id(t))
            .collect();
        let set_id = Literal::usize_unsuffixed(set.id);
        terminal_set_items.push(quote! {
            #[comment = #comment]
            pub static #name: TerminalSet = TerminalSet { id: #set_id, terminals: &[#(#ids),*] };
        });
    }

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

            fn nonterminal_id(name: &str) -> Option<NonterminalId> {
                match name {
                    #(#nonterminal_id_arms)*
                    _ => None,
                }
            }
        }

        #(#terminal_set_items)*
    }
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
    };
    let names: Vec<_> = set.terminals.iter().map(|t| t.name.clone()).collect();
    format!("{position} {{ {} }}", names.join(", "))
}
