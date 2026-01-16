#![allow(unstable_name_collisions)]
use itertools::Itertools;

use crate::grammar::{
    def::SyntaxRule, symbols::Symbol, transformations::transform_rule_by_symbols,
};

pub fn transform(syntax_rules: Vec<SyntaxRule>, layout_def: Symbol) -> Vec<SyntaxRule> {
    syntax_rules
        .into_iter()
        .map(|rule| {
            transform_rule_by_symbols(rule, |symbols| {
                symbols
                    .into_iter()
                    .intersperse(layout_def.clone())
                    .collect()
            })
        })
        .collect()
}
