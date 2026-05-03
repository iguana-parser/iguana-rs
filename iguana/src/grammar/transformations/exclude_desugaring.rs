use rustc_hash::FxHashMap;

use crate::grammar::{
    def::{PriorityLevel, SyntaxRule},
    symbols::{Identifier, Nonterminal, Symbol},
    transformations::{transform_syntax_rule, visit_syntax_rule},
};

pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    // Build a map from nonterminal name to its rule for lookups.
    let rules_by_name: FxHashMap<String, &SyntaxRule> = syntax_rules
        .iter()
        .map(|r| (r.head.name.clone(), r))
        .collect();

    // Collect all unique (nonterminal_name, labels) combos and generate new nonterminal names.
    let mut exclude_map: FxHashMap<(String, Vec<String>), String> = FxHashMap::default();
    for rule in &syntax_rules {
        visit_syntax_rule(rule, &mut |symbol| {
            if let Symbol::Exclude { symbol, labels } = symbol {
                let nonterminal_name = symbol
                    .as_identifier()
                    .expect("Exclude symbol should wrap an Identifier")
                    .name
                    .clone();
                let key = (nonterminal_name.clone(), labels.clone());
                if !exclude_map.contains_key(&key) {
                    let new_name = format!("{}_except_{}", nonterminal_name, labels.join("_"));
                    exclude_map.insert(key, new_name);
                }
            }
        });
    }

    if exclude_map.is_empty() {
        return syntax_rules;
    }

    // Create new nonterminals with filtered alternatives.
    let mut new_rules: Vec<SyntaxRule> = Vec::new();
    for ((nonterminal_name, labels), new_name) in &exclude_map {
        let original_rule = rules_by_name
            .get(nonterminal_name)
            .unwrap_or_else(|| panic!("Nonterminal {} not found for Exclude", nonterminal_name));

        // Keep an empty placeholder for any priority level whose alternatives
        // are all excluded, so the derived rule's level positions stay aligned
        // with the parent's. `precedence_desugaring` relies on that alignment
        // to reuse the parent's precedence numbering for the derived rule.
        let filtered_priority_levels: Vec<PriorityLevel> = original_rule
            .priority_levels
            .iter()
            .map(|pl| {
                let filtered_alternatives: Vec<_> = pl
                    .alternatives
                    .iter()
                    .filter(|alt| match &alt.label {
                        Some(label) => !labels.contains(label),
                        None => true,
                    })
                    .cloned()
                    .collect();
                PriorityLevel {
                    alternatives: filtered_alternatives,
                    associativity: pl.associativity.clone(),
                }
            })
            .collect();

        assert!(
            filtered_priority_levels
                .iter()
                .any(|pl| !pl.alternatives.is_empty()),
            "Exclude removed all alternatives from {}",
            nonterminal_name
        );

        let origin_symbol = Symbol::Exclude {
            symbol: Box::new(Symbol::Identifier(Identifier {
                name: nonterminal_name.clone(),
                definition: None,
            })),
            labels: labels.clone(),
        };

        new_rules.push(SyntaxRule {
            head: Nonterminal::with_origin(new_name, origin_symbol),
            priority_levels: filtered_priority_levels,
            layout: original_rule.layout.clone(),
            start: false,
        });
    }

    // Resolve Exclude symbols to point to the new nonterminals.
    syntax_rules
        .into_iter()
        .chain(new_rules)
        .map(|rule| transform_syntax_rule(rule, |symbol| replace_exclude(symbol, &exclude_map)))
        .collect()
}

fn replace_exclude(
    symbol: Symbol,
    exclude_map: &FxHashMap<(String, Vec<String>), String>,
) -> Symbol {
    match symbol {
        Symbol::Exclude { symbol, labels } => {
            let nonterminal_name = symbol
                .as_identifier()
                .expect("Exclude symbol should wrap an Identifier")
                .name
                .clone();
            let key = (nonterminal_name, labels.clone());
            let new_name = exclude_map
                .get(&key)
                .expect("Exclude combo should be in the map");
            Symbol::Identifier(Identifier {
                name: new_name.clone(),
                definition: None,
            })
        }
        Symbol::Except { symbol, except } => Symbol::Except {
            symbol: Box::new(replace_exclude(*symbol, exclude_map)),
            except,
        },
        Symbol::FollowRestriction {
            symbol,
            restrictions,
        } => Symbol::FollowRestriction {
            symbol: Box::new(replace_exclude(*symbol, exclude_map)),
            restrictions,
        },
        Symbol::PrecedeRestriction {
            symbol,
            restriction,
        } => Symbol::PrecedeRestriction {
            symbol: Box::new(replace_exclude(*symbol, exclude_map)),
            restriction,
        },
        Symbol::Labeled { label, symbol } => Symbol::Labeled {
            label,
            symbol: Box::new(replace_exclude(*symbol, exclude_map)),
        },
        Symbol::Binding { name, symbol } => Symbol::Binding {
            name,
            symbol: Box::new(replace_exclude(*symbol, exclude_map)),
        },
        other => other,
    }
}
