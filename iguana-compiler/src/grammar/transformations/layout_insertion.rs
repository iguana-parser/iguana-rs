use std::mem;

use crate::grammar::{
    def::{LayoutStrategy, SyntaxRule},
    symbols::{Identifier, Symbol},
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

fn strip_layout_aware_restrictions(symbol: Symbol) -> (Symbol, Vec<Identifier>) {
    match symbol {
        Symbol::FollowRestriction {
            symbol,
            restrictions,
            layout_aware: true,
        } => {
            let (symbol, mut stripped) = strip_layout_aware_restrictions(*symbol);
            stripped.extend(restrictions);
            (symbol, stripped)
        }
        Symbol::FollowRestriction {
            symbol,
            restrictions,
            layout_aware: false,
        } => {
            let (symbol, stripped) = strip_layout_aware_restrictions(*symbol);
            let symbol = Symbol::FollowRestriction {
                symbol: Box::new(symbol),
                restrictions,
                layout_aware: false,
            };
            (symbol, stripped)
        }
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
        Symbol::Except { symbol, except } => {
            let (symbol, stripped) = strip_layout_aware_restrictions(*symbol);
            let symbol = Symbol::Except {
                symbol: Box::new(symbol),
                except,
            };
            (symbol, stripped)
        }
        Symbol::PrecedeRestriction {
            symbol,
            restriction,
        } => {
            let (symbol, stripped) = strip_layout_aware_restrictions(*symbol);
            let symbol = Symbol::PrecedeRestriction {
                symbol: Box::new(symbol),
                restriction,
            };
            (symbol, stripped)
        }
        symbol => (symbol, Vec::new()),
    }
}

fn add_follow_restriction(symbol: Symbol, restrictions: Vec<Identifier>) -> Symbol {
    if restrictions.is_empty() {
        return symbol;
    }
    match symbol {
        Symbol::Labeled { label, symbol } => Symbol::Labeled {
            label,
            symbol: Box::new(add_follow_restriction(*symbol, restrictions)),
        },
        Symbol::Binding { name, symbol } => Symbol::Binding {
            name,
            symbol: Box::new(add_follow_restriction(*symbol, restrictions)),
        },
        Symbol::FollowRestriction {
            symbol,
            restrictions: mut existing,
            layout_aware: false,
        } => {
            existing.extend(restrictions);
            Symbol::FollowRestriction {
                symbol,
                restrictions: existing,
                layout_aware: false,
            }
        }
        symbol => Symbol::FollowRestriction {
            symbol: Box::new(symbol),
            restrictions,
            layout_aware: false,
        },
    }
}
