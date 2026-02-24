use crate::grammar::{
    def::{Alternative, PriorityLevel, SyntaxRule},
    symbols::{Nonterminal, Symbol},
};

pub mod ebnf_to_bnf;
pub mod layout_insertion;
pub mod precedence_desugaring;

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
                .map(|alternative| {
                    let new_symbols: Vec<_> = alternative
                        .symbols
                        .into_iter()
                        .map(&mut transform_symbol)
                        .collect();
                    Alternative {
                        symbols: new_symbols,
                        label: alternative.label,
                    }
                })
                .collect();
            PriorityLevel::new(new_alternatives)
        })
        .collect();
    let origin = rule.head.origin.map(transform_symbol);
    let head = Nonterminal {
        name,
        origin,
        parameters: rule.head.parameters,
    };
    SyntaxRule::new(head, new_priority_levels)
}

pub fn transform_rule_by_symbols<F>(rule: SyntaxRule, mut transform_symbol: F) -> SyntaxRule
where
    F: FnMut(Vec<Symbol>) -> Vec<Symbol>,
{
    let new_priority_levels: Vec<_> = rule
        .priority_levels
        .into_iter()
        .map(|priority_level| {
            let new_alternatives: Vec<_> = priority_level
                .alternatives
                .into_iter()
                .map(|alternative| {
                    let new_symbols = transform_symbol(alternative.symbols);
                    Alternative {
                        symbols: new_symbols,
                        label: alternative.label,
                    }
                })
                .collect();
            PriorityLevel::new(new_alternatives)
        })
        .collect();
    SyntaxRule::new(rule.head, new_priority_levels)
}
