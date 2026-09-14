use crate::grammar::{def::Grammar, symbols::Nonterminal};

/// Generate the `manifest.json` the viewer reads before any parse: the grammar
/// name, the nonterminals the user can start from, the layout-rule name, and a
/// sample input. The start list matches the CLI's `--list-nonterminals`: the
/// user-declared nonterminals in grammar source order, without the layout
/// nonterminal and the derived ones (start wrappers, EBNF expansions,
/// desugarings), none of which has an entry point.
pub fn generate(grammar: &Grammar) -> String {
    let layout_name = grammar
        .layout
        .as_ref()
        .and_then(|s| s.as_identifier())
        .map(|i| i.name.as_str());

    let mut start_nonterminals: Vec<&Nonterminal> = grammar
        .nonterminals()
        .filter(|n| !n.is_derived() && Some(n.name.as_str()) != layout_name)
        .collect();
    start_nonterminals.sort_by_key(|n| grammar.source_index(&n.name));
    let start_names: Vec<&str> = start_nonterminals.iter().map(|n| n.name.as_str()).collect();

    let manifest = serde_json::json!({
        "grammar": grammar.name,
        "start_nonterminals": start_names,
        "layout_name": layout_name,
        "sample_input": "",
    });
    serde_json::to_string_pretty(&manifest).unwrap()
}
