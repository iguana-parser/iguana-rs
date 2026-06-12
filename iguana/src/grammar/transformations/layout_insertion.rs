use crate::grammar::{
    def::{LayoutStrategy, SyntaxRule},
    symbols::Symbol,
    transformations::transform_rule_by_symbols,
};

pub fn transform(syntax_rules: Vec<SyntaxRule>, layout_symbol: &Symbol) -> Vec<SyntaxRule> {
    syntax_rules
        .into_iter()
        .map(|rule| {
            let layout_symbol = match &rule.layout {
                LayoutStrategy::Default => Some(layout_symbol.clone()),
                LayoutStrategy::None => None,
                LayoutStrategy::Custom(_id) => {
                    // TODO: resolve custom layout identifier to a Symbol
                    unimplemented!("@WithLayout(X) (per-rule custom layout) is not yet supported")
                }
            };
            match layout_symbol {
                Some(layout) => {
                    transform_rule_by_symbols(rule, |symbols| insert_layout(&symbols, &layout))
                }
                None => rule,
            }
        })
        .collect()
}

fn insert_layout(symbols: &[Symbol], layout: &Symbol) -> Vec<Symbol> {
    let mut result = Vec::new();
    // Insert layout only between parse tree symbols.
    // e.g., A = B [cond] C becomes B [cond] Layout C
    let mut has_prev_symbol = false;
    for symbol in symbols.iter() {
        if !symbol.is_parse_tree_symbol() {
            result.push(symbol.clone());
        } else {
            if has_prev_symbol {
                result.push(layout.clone());
            }
            result.push(symbol.clone());
            has_prev_symbol = true;
        }
    }
    result
}
