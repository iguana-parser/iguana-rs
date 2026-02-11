use crate::grammar::{
    def::SyntaxRule, symbols::Symbol, transformations::transform_rule_by_symbols,
};

pub fn transform(syntax_rules: Vec<SyntaxRule>, layout_def: Symbol) -> Vec<SyntaxRule> {
    syntax_rules
        .into_iter()
        .map(|rule| {
            transform_rule_by_symbols(rule, |symbols| {
                let mut result = Vec::new();
                // Insert layout only between non-Condition symbols.
                // e.g., A = B [cond] C becomes B [cond] Layout C
                let mut has_prev_symbol = false;
                for symbol in symbols.iter() {
                    if matches!(symbol, Symbol::Condition(_)) {
                        result.push(symbol.clone());
                    } else {
                        if has_prev_symbol {
                            result.push(layout_def.clone());
                        }
                        result.push(symbol.clone());
                        has_prev_symbol = true;
                    }
                }
                result
            })
        })
        .collect()
}
