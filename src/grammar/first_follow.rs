use std::collections::HashSet;

use crate::grammar::symbols::{Grammar, Nonterminal, Symbol};

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
            if let Some(alternatives) = grammar.alternatives(nonterminal) {
                for alternative in alternatives {
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
            } else {
                changed |= nullables.insert(nonterminal);
            }
        }
    }
    nullables
}

fn is_nullable(s: &Symbol, nullables: &HashSet<&Nonterminal>) -> bool {
    match s {
        Symbol::Terminal(_) => false,
        Symbol::Nonterminal(nonterminal) => nullables.contains(nonterminal),
        Symbol::Seq(seq) => seq.symbols.iter().all(|s| is_nullable(s, nullables)),
        Symbol::Opt(_) => true,
        Symbol::Alt(seq) => seq.symbols.iter().any(|s| is_nullable(s, nullables)),
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
    fn test_nullables() {

    }
}
