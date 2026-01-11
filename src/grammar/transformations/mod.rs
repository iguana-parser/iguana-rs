use crate::grammar::{
    def::{Alternative, PriorityLevel, SyntaxRule},
    symbols::{Nonterminal, Symbol},
};

pub mod ebnf_to_bnf;

pub fn transform_rule<F>(rule: SyntaxRule, mut transform_symbol: F) -> SyntaxRule
where
    F: FnMut(Symbol) -> Symbol,
{
    let name = rule.head.name;
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
    let origin = if let Some(s) = rule.head.origin {
        Some(transform_symbol(s))
    } else {
        None
    };
    let head = Nonterminal { name, origin };
    SyntaxRule::new(head, new_priority_levels)
}
