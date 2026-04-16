use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;

use crate::generator::id::TerminalIds;
use crate::generator::utils::to_snake_case;
use crate::grammar::def::Grammar;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::Terminal;

pub fn generate(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
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
        let follow_comment = format!(
            "{} {}",
            nonterminal.name,
            terminal_names(&follow_terminals),
        );
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
        let first_comment = format!(
            "{} {}",
            nonterminal.name,
            terminal_names(&first_terminals),
        );
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
            let pred_comment = format!(
                "{} {}",
                slot.name(),
                terminal_names(&pred_terminals),
            );
            items.push(quote! {
                #[comment = #pred_comment]
                pub static #pred_name: &[TerminalId] = &[#(#pred_ids),*];
            });
        }
    }

    quote! {
        use iguana_runtime::ids::TerminalId;

        #(#items)*
    }
}
