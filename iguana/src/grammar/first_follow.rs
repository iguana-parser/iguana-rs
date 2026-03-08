use std::collections::HashSet;

use crate::grammar::{
    def::Grammar,
    symbols::{Nonterminal, Symbol},
};

/// Calculates the nullable nonterminals in the provided grammar.
///
/// A nonterminal is nullable if
/// - it directly derives epsilon, i.e., has an alternative with no symbols in its body.
/// - it has an alternative where all the symbols in the body are nullable.
fn calc_nullables(grammar: &Grammar) -> HashSet<&Nonterminal> {
    let mut nullables = HashSet::new();
    let mut changed = true;
    while changed {
        for nonterminal in grammar.nonterminals() {
            for alternative in grammar.alternatives(nonterminal) {
                if alternative
                    .symbols
                    .iter()
                    .all(|s| is_nullable(s, &nullables))
                {
                    changed |= nullables.insert(nonterminal);
                    if changed {
                        break;
                    }
                }
            }
        }
    }
    nullables
}

fn is_nullable(s: &Symbol, nullables: &HashSet<&Nonterminal>) -> bool {
    match s {
        Symbol::Labeled { symbol, .. } => is_nullable(symbol, nullables),
        Symbol::Identifier(_) => false,
        Symbol::Literal(_) => false,
        Symbol::Group(symbols) => symbols.iter().all(|s| is_nullable(s, nullables)),
        Symbol::Opt(_) => true,
        Symbol::Alt(symbols) => symbols.iter().any(|s| is_nullable(s, nullables)),
        Symbol::Star(_, _) => true,
        Symbol::Plus(symbol, _) => is_nullable(symbol, nullables),
        Symbol::Binding { symbol, .. } => is_nullable(symbol, nullables),
        Symbol::Except { symbol, .. } | Symbol::FollowRestriction { symbol, .. } | Symbol::PrecedeRestriction { symbol, .. } | Symbol::Exclude { symbol, .. } => {
            is_nullable(symbol, nullables)
        }
        Symbol::Call { .. } => false,
        Symbol::Condition(_) => false,
        Symbol::Return(_) => false,
    }
}

fn calc_first_sets(grammar: &Grammar) {
    let mut changed = true;
    while changed {
        changed = false;
        for nonterminal in grammar.nonterminals() {}
    }
}

fn add_first_set(first_set: &mut HashSet<char>) {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_nullables() {}
}
