use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;

use crate::generator::id::{NonterminalIds, SlotIds, TerminalIds};
use crate::grammar::def::Grammar;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::{Definition, Nonterminal, Symbol, Terminal};
use crate::utils::to_snake_case;

pub fn generate<'a>(
    grammar: &'a Grammar,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds<'a>,
) -> TokenStream {
    let ff = FirstFollowSets::new(grammar);
    let eof_id = Literal::u16_unsuffixed(terminal_ids.len() as u16 + 1);
    let mut items = vec![];

    let terminal_id_tokens = |t: &Terminal| -> TokenStream {
        if t.name == "EOF" {
            quote! { TerminalId(#eof_id) }
        } else {
            let id = terminal_ids.get_id(t);
            quote! { #id }
        }
    };
    let terminal_names = |terminals: &[Terminal]| -> String {
        let names: Vec<_> = terminals.iter().map(|t| t.name.clone()).collect();
        format!("{{ {} }}", names.join(", "))
    };

    for nonterminal in grammar.nonterminals() {
        let nt_snake = to_snake_case(&nonterminal.name);
        let nt_upper = nt_snake.to_uppercase();
        let alternatives = grammar.alternatives(nonterminal);

        // Follow set
        let follow_name = format_ident!("FOLLOW_SET_{}", nt_upper);
        let follow_terminals: Vec<_> = ff.follow_set(nonterminal).cloned().collect();
        let follow_ids: Vec<_> = follow_terminals.iter().map(&terminal_id_tokens).collect();
        let follow_comment = format!("{} {}", nonterminal.name, terminal_names(&follow_terminals),);
        items.push(quote! {
            #[comment = #follow_comment]
            pub static #follow_name: &[TerminalId] = &[#(#follow_ids),*];
        });

        // First set (union of all prediction sets, for error reporting)
        let first_name = format_ident!("FIRST_SET_{}", nt_upper);
        let first_terminals: Vec<_> = alternatives
            .iter()
            .flat_map(|alt| ff.prediction_set(nonterminal, alt))
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect();
        let first_ids: Vec<_> = first_terminals.iter().map(&terminal_id_tokens).collect();
        let first_comment = format!("{} {}", nonterminal.name, terminal_names(&first_terminals),);
        items.push(quote! {
            #[comment = #first_comment]
            pub static #first_name: &[TerminalId] = &[#(#first_ids),*];
        });

        // Prediction set for each alternative
        for (alt_index, alternative) in alternatives.iter().enumerate() {
            let prediction_set = ff.prediction_set(nonterminal, alternative);
            let pred_name = format_ident!("PREDICTION_SET_{}_ALT{}", nt_upper, alt_index);
            let pred_terminals: Vec<_> = prediction_set.iter().cloned().collect();
            let pred_ids: Vec<_> = pred_terminals.iter().map(&terminal_id_tokens).collect();
            let slot = Slot::new(nonterminal, alternative, 0);
            let pred_comment = format!("{} {}", slot.name(), terminal_names(&pred_terminals),);
            items.push(quote! {
                #[comment = #pred_comment]
                pub static #pred_name: &[TerminalId] = &[#(#pred_ids),*];
            });

            // Follow restriction sets for symbols in this alternative
            for (pos, symbol) in alternative.symbols.iter().enumerate() {
                if let Symbol::FollowRestriction { restrictions, .. } = symbol {
                    let restriction_terminals: Vec<_> = restrictions
                        .iter()
                        .map(|r| {
                            let Definition::Terminal(t) = grammar.definition(r.resolve()) else {
                                panic!("follow restriction must resolve to a terminal");
                            };
                            t.clone()
                        })
                        .collect();
                    let restriction_ids: Vec<_> = restriction_terminals
                        .iter()
                        .map(&terminal_id_tokens)
                        .collect();
                    let name = format_ident!(
                        "FOLLOW_RESTRICTION_{}_ALT{}_POS{}",
                        nt_upper,
                        alt_index,
                        pos
                    );
                    let comment = format!(
                        "{} !>> {}",
                        Slot::new(nonterminal, alternative, pos).name(),
                        terminal_names(&restriction_terminals),
                    );
                    items.push(quote! {
                        #[comment = #comment]
                        pub static #name: &[TerminalId] = &[#(#restriction_ids),*];
                    });
                }
            }
        }

        // Bundled alternatives constant for multi-alternative nonterminals
        if alternatives.len() > 1 {
            let alt_entries: Vec<_> = alternatives
                .iter()
                .enumerate()
                .map(|(alt_index, alternative)| {
                    let pred_name = format_ident!("PREDICTION_SET_{}_ALT{}", nt_upper, alt_index);
                    let first_slot = Slot::new(nonterminal, alternative, 0);
                    let first_slot_id = slot_ids.get_id(&first_slot);
                    quote! { (#pred_name, #first_slot_id) }
                })
                .collect();
            let alternatives_name = format_ident!("ALTERNATIVES_{}", nt_upper);
            items.push(quote! {
                pub static #alternatives_name: (&[(&[TerminalId], SlotId)], &[TerminalId]) = (
                    &[#(#alt_entries),*],
                    #first_name,
                );
            });
        }
    }

    // NONTERMINALS array
    let nonterminals_len = Literal::usize_unsuffixed(nonterminal_ids.len());
    let nonterminals = nonterminal_ids.nonterminals().map(|n| {
        let nonterminal_name = &n.name;
        let display_name = n.display_name();
        let derived = n.is_derived();
        quote! {
            Nonterminal {
                name: #nonterminal_name,
                display: #display_name,
                derived: #derived,
            }
        }
    });

    // User-facing nonterminal listing in grammar source order. Filter out derived
    // (start wrappers, EBNF expansions, exclude/precedence desugarings) and sort by
    // their original .iggy declaration position. Pre-computed at codegen time so
    // the CLI just iterates this static for `--list-nonterminals`.
    let mut display_order: Vec<&Nonterminal> = nonterminal_ids
        .nonterminals()
        .filter(|n| !n.is_derived())
        .collect();
    display_order.sort_by_key(|n| grammar.source_index(&n.name));
    let display_order_names: Vec<&str> = display_order.iter().map(|n| n.name.as_str()).collect();
    let display_order_len = Literal::usize_unsuffixed(display_order_names.len());

    // Individual nonterminal ID constants
    let nonterminal_id_consts: Vec<_> = nonterminal_ids
        .nonterminals()
        .enumerate()
        .map(|(i, n)| {
            let const_name = format_ident!("{}", to_snake_case(&n.name).to_uppercase());
            let index = Literal::usize_unsuffixed(i);
            quote! { pub const #const_name: NonterminalId = NonterminalId(#index); }
        })
        .collect();

    // nonterminal_id() lookup function for CLI
    let nonterminal_id_arms: Vec<_> = nonterminal_ids
        .nonterminals()
        .map(|n| {
            let name = &n.name;
            let const_name = format_ident!("{}", to_snake_case(name).to_uppercase());
            quote! { #name => Some(#const_name), }
        })
        .collect();

    // TERMINALS array
    let terminals_len = Literal::usize_unsuffixed(terminal_ids.len() + 2);
    let terminals: Vec<_> = terminal_ids
        .terminals()
        .map(|t| {
            let terminal_name = &t.name;
            quote! { Terminal { name: #terminal_name } }
        })
        .collect();

    // SLOTS array
    let slots_len = Literal::usize_unsuffixed(slot_ids.len());
    let slot_names = slot_ids.slots().map(|s| {
        let display_name = s.display_name(grammar);
        quote! {
            Slot { display_name: #display_name }
        }
    });

    quote! {
        use iguana_runtime::ids::{NonterminalId, SlotId, TerminalId};
        use crate::types::{Nonterminal, Slot, Terminal};

        pub const NONTERMINALS: [Nonterminal; #nonterminals_len] = [#(#nonterminals),*];

        #[comment = "User-declared nonterminals in `.iggy` source order. Used by `--list-nonterminals`."]
        pub const NONTERMINAL_DISPLAY_ORDER: [&str; #display_order_len] = [#(#display_order_names),*];

        #(#nonterminal_id_consts)*

        pub fn nonterminal_id(name: &str) -> Option<NonterminalId> {
            match name {
                #(#nonterminal_id_arms)*
                _ => None,
            }
        }

        pub const TERMINALS: [Terminal; #terminals_len] = [
            #(#terminals,)*
            Terminal { name: "Epsilon" },
            Terminal { name: "EOF" },
        ];

        pub const SLOTS: [Slot; #slots_len] = [#(#slot_names),*];

        #(#items)*
    }
}
