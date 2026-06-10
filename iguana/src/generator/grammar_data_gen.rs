use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::generator::id::{NonterminalIds, SlotIds, TerminalIds};
use crate::generator::terminal_sets::{SetIds, SetKind, TerminalSet};
use crate::grammar::def::Grammar;
use crate::grammar::symbols::Nonterminal;
use crate::utils::to_snake_case;

pub fn generate<'a>(
    grammar: &'a Grammar,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds<'a>,
    terminal_sets: &[TerminalSet],
    match_any_sets: &SetIds,
    longest_match_sets: &SetIds,
) -> TokenStream {
    let mut items = vec![];

    for set in terminal_sets {
        let name = format_ident!("{}", set.name());
        let comment = set.comment(grammar);
        let ids: Vec<_> = set
            .terminals
            .iter()
            .map(|t| terminal_ids.get_id(t))
            .collect();
        // Every set is emitted as a `TerminalSet`. Its id comes from the
        // `match_any` space (which keys that memo), except the combined FIRST
        // set, which is numbered in the `longest_match` space.
        let set_id = match set.kind {
            SetKind::First => longest_match_sets.id(&set.name()),
            _ => match_any_sets.id(&set.name()),
        };
        let set_id = Literal::usize_unsuffixed(set_id);
        items.push(quote! {
            #[comment = #comment]
            pub static #name: TerminalSet = TerminalSet { id: #set_id, terminals: &[#(#ids),*] };
        });
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
        use iguana_runtime::scanner::TerminalSet;
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
