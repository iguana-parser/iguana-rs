use std::mem;

use crate::grammar::{
    def::{LayoutStrategy, SyntaxRule},
    symbols::{Identifier, Restrictions, Symbol},
    transformations::{transform_rule_by_symbols, transform_syntax_rule},
};

/// Inserts the layout symbol between the parse tree symbols of a rule, and
/// rewrites `!>>>` restrictions to `!>>` where it can.
///
/// A `!>>>` restriction forbids its terminals after the layout that follows its
/// symbol.
///
/// - When a layout symbol is inserted right after the restricted symbol, the
///   restriction moves onto that layout as a plain `!>>`, and the parser checks
///   it at the layout's right extent.
/// - When no layout symbol is inserted after the restricted symbol, which
///   happens when the last symbol in an alternative has the `!>>>`, the
///   restriction keeps it. The layout there comes from the caller, so the
///   generator emits a check that first matches the layout and then the
///   restriction.
/// - When a rule gets no layout (`@NoLayout` and the layout rule itself), the
///   `!>>>` restrictions are rewritten to `!>>`.
///
/// `layout_symbol` is `None` for a grammar without a layout rule.
pub fn transform(syntax_rules: Vec<SyntaxRule>, layout_symbol: Option<&Symbol>) -> Vec<SyntaxRule> {
    syntax_rules
        .into_iter()
        .map(|rule| match (&rule.layout, layout_symbol) {
            (LayoutStrategy::Custom(_id), _) => {
                // TODO: resolve custom layout identifier to a Symbol
                unimplemented!("@WithLayout(X) (per-rule custom layout) is not yet supported")
            }
            (LayoutStrategy::Default, Some(layout)) => {
                let layout = layout.clone();
                transform_rule_by_symbols(rule, |symbols| insert_layout(&symbols, &layout))
            }
            // No layout follows any symbol of the rule, so its `!>>>`
            // restrictions become plain `!>>` ones.
            (LayoutStrategy::Default, None) | (LayoutStrategy::None, _) => {
                transform_syntax_rule(rule, |symbol| {
                    let (symbol, restrictions) = strip_layout_aware_restrictions(symbol);
                    add_follow_restriction(symbol, restrictions)
                })
            }
        })
        .collect()
}

fn insert_layout(symbols: &[Symbol], layout: &Symbol) -> Vec<Symbol> {
    let mut result = Vec::new();
    // Insert layout only between parse tree symbols.
    // e.g., A = B [cond] C becomes B [cond] Layout C
    let mut has_prev_symbol = false;
    let last = symbols
        .iter()
        .rposition(|symbol| symbol.is_parse_tree_symbol());
    // The `!>>>` restrictions of the previous parse tree symbol. The layout
    // symbol inserted after that symbol gets them as its `!>>`.
    let mut layout_aware_restrictions = Vec::new();
    for (index, symbol) in symbols.iter().enumerate() {
        if !symbol.is_parse_tree_symbol() {
            result.push(symbol.clone());
            continue;
        }
        if has_prev_symbol {
            result.push(add_follow_restriction(
                layout.clone(),
                mem::take(&mut layout_aware_restrictions),
            ));
        }
        if Some(index) == last {
            result.push(symbol.clone());
        } else {
            let (symbol, restrictions) = strip_layout_aware_restrictions(symbol.clone());
            layout_aware_restrictions = restrictions;
            result.push(symbol);
        }
        has_prev_symbol = true;
    }
    result
}

/// Strips the layout-aware follow restrictions (`!>>>`) off a symbol,
/// returning the symbol and the restriction identifiers. As a `Restricted`
/// node can sit inside a `Labeled` or `Binding` wrapper, the walk descends
/// through those wrappers.
fn strip_layout_aware_restrictions(symbol: Symbol) -> (Symbol, Vec<Identifier>) {
    match symbol {
        Symbol::Labeled { label, symbol } => {
            let (symbol, stripped) = strip_layout_aware_restrictions(*symbol);
            let symbol = Symbol::Labeled {
                label,
                symbol: Box::new(symbol),
            };
            (symbol, stripped)
        }
        Symbol::Binding { name, symbol } => {
            let (symbol, stripped) = strip_layout_aware_restrictions(*symbol);
            let symbol = Symbol::Binding {
                name,
                symbol: Box::new(symbol),
            };
            (symbol, stripped)
        }
        Symbol::Restricted {
            symbol,
            mut restrictions,
        } => {
            let stripped = mem::take(&mut restrictions.layout_aware_follow);
            (Symbol::restricted(*symbol, restrictions), stripped)
        }
        symbol => (symbol, Vec::new()),
    }
}

fn add_follow_restriction(symbol: Symbol, follow: Vec<Identifier>) -> Symbol {
    if follow.is_empty() {
        return symbol;
    }
    match symbol {
        Symbol::Labeled { label, symbol } => Symbol::Labeled {
            label,
            symbol: Box::new(add_follow_restriction(*symbol, follow)),
        },
        Symbol::Binding { name, symbol } => Symbol::Binding {
            name,
            symbol: Box::new(add_follow_restriction(*symbol, follow)),
        },
        Symbol::Restricted {
            symbol,
            mut restrictions,
        } => {
            for id in follow {
                if !restrictions.follow.contains(&id) {
                    restrictions.follow.push(id);
                }
            }
            Symbol::Restricted {
                symbol,
                restrictions,
            }
        }
        symbol => Symbol::restricted(
            symbol,
            Restrictions {
                follow,
                ..Default::default()
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grammar::symbols::Restrictions, id, restriction_ids};

    /// A rule that gets no layout has its `!>>>` restrictions rewritten to
    /// `!>>`. When the symbol already has the same restriction as a `!>>`,
    /// the rewrite must not list it twice.
    #[test]
    fn test_rewritten_layout_aware_restriction_is_not_listed_twice() {
        let symbol = Symbol::restricted(
            id!("A"),
            Restrictions {
                follow: restriction_ids!("B"),
                layout_aware_follow: restriction_ids!("B"),
                ..Default::default()
            },
        );
        let (symbol, stripped) = strip_layout_aware_restrictions(symbol);
        let symbol = add_follow_restriction(symbol, stripped);
        let follow: Vec<&str> = symbol
            .restrictions()
            .follow
            .iter()
            .map(|id| id.name.as_str())
            .collect();
        assert_eq!(follow, ["B"]);
    }
}
