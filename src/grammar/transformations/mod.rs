use rustc_hash::FxHashMap;

use crate::grammar::{
    def::{Alternative, PriorityLevel, SyntaxRule},
    symbols::{DefinitionId, Identifier, Symbol},
};

pub mod ebnf_to_bnf;

pub fn resolved_identifier(
    name: String,
    symbol_table: &mut FxHashMap<String, DefinitionId>,
) -> Symbol {
    let id = DefinitionId(symbol_table.len() as u16);
    symbol_table.insert(name.clone(), id);
    Symbol::Identifier(Identifier {
        name,
        definition: Some(id),
    })
}

pub fn transform_rule<F>(rule: SyntaxRule, mut transform_symbol: F) -> SyntaxRule
where
    F: FnMut(Symbol) -> Symbol,
{
    let new_priority_levels: Vec<_> = rule
        .priority_levels
        .into_iter()
        .map(|priority_level| {
            let new_alternatives: Vec<_> = priority_level
                .alternatives
                .into_iter()
                .map(|alt| {
                    let new_symbols: Vec<_> =
                        alt.symbols.into_iter().map(&mut transform_symbol).collect();
                    Alternative::new(new_symbols)
                })
                .collect();
            PriorityLevel::new(new_alternatives)
        })
        .collect();
    SyntaxRule::new(rule.head, new_priority_levels)
}
