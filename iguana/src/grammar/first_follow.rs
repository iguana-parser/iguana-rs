use rustc_hash::{FxHashMap, FxHashSet};

use crate::grammar::{
    def::Grammar,
    symbols::{Definition, Nonterminal, Symbol, Terminal},
};

/// Calculates the nullable nonterminals in the provided grammar.
///
/// A nonterminal is nullable if
/// - it directly derives epsilon, i.e., has an alternative with no symbols in its body.
/// - it has an alternative where all the symbols in the body are nullable.
pub fn calc_nullables(grammar: &Grammar) -> FxHashSet<&Nonterminal> {
    let mut nullables = FxHashSet::default();
    let mut changed = true;
    while changed {
        changed = false;
        for nonterminal in grammar.nonterminals() {
            if nullables.contains(nonterminal) {
                continue;
            }
            for alternative in grammar.alternatives(nonterminal) {
                if alternative
                    .symbols
                    .iter()
                    .all(|s| is_nullable(s, grammar, &nullables))
                {
                    nullables.insert(nonterminal);
                    changed = true;
                    break;
                }
            }
        }
    }
    nullables
}

fn is_nullable(s: &Symbol, grammar: &Grammar, nullables: &FxHashSet<&Nonterminal>) -> bool {
    match s {
        Symbol::Labeled { symbol, .. } | Symbol::Binding { symbol, .. } => {
            is_nullable(symbol, grammar, nullables)
        }
        Symbol::Identifier(id) => {
            let def_id = id.resolve();
            match grammar.definition(def_id) {
                Definition::Terminal(terminal) => grammar
                    .lexical_rule(terminal)
                    .map_or(false, |rule| rule.regex.is_nullable()),
                Definition::Nonterminal(nt) => nullables.contains(nt),
            }
        }
        Symbol::Literal(_) => false,
        Symbol::Group(symbols) => symbols.iter().all(|s| is_nullable(s, grammar, nullables)),
        Symbol::Opt(_) | Symbol::Star(_, _) => true,
        Symbol::Alt(symbols) => symbols.iter().any(|s| is_nullable(s, grammar, nullables)),
        Symbol::Plus(symbol, _) => is_nullable(symbol, grammar, nullables),
        Symbol::Except { symbol, .. }
        | Symbol::FollowRestriction { symbol, .. }
        | Symbol::PrecedeRestriction { symbol, .. }
        | Symbol::Exclude { symbol, .. } => is_nullable(symbol, grammar, nullables),
        Symbol::Call { .. } | Symbol::Condition(_) | Symbol::Return(_) => false,
    }
}

/// Calculates the FIRST sets for all nonterminals in the grammar.
///
/// FIRST(A) is the set of terminals that can appear as the first terminal
/// in a string derived from A. This is computed as a fixed-point iteration:
/// for each alternative of each nonterminal, walk the symbols left to right,
/// adding terminals to the FIRST set, and continuing past nullable symbols.
pub fn calc_first_sets<'a>(grammar: &'a Grammar) -> FxHashMap<&'a Nonterminal, FxHashSet<&'a Terminal>> {
    let nullables = calc_nullables(grammar);
    let mut first_sets: FxHashMap<&Nonterminal, FxHashSet<&Terminal>> = FxHashMap::default();
    for nonterminal in grammar.nonterminals() {
        first_sets.insert(nonterminal, FxHashSet::default());
    }

    let mut changed = true;
    while changed {
        changed = false;
        for nonterminal in grammar.nonterminals() {
            for alternative in grammar.alternatives(nonterminal) {
                for symbol in &alternative.symbols {
                    let added = add_first_of_symbol(symbol, grammar, &nullables, &mut first_sets, nonterminal);
                    changed |= added;
                    if !is_nullable(symbol, grammar, &nullables) {
                        break;
                    }
                }
            }
        }
    }
    first_sets
}

/// Adds the FIRST terminals of `symbol` into the FIRST set of `target`.
/// Returns true if any new terminals were added.
fn add_first_of_symbol<'a>(
    symbol: &Symbol,
    grammar: &'a Grammar,
    nullables: &FxHashSet<&Nonterminal>,
    first_sets: &mut FxHashMap<&'a Nonterminal, FxHashSet<&'a Terminal>>,
    target: &'a Nonterminal,
) -> bool {
    match symbol {
        Symbol::Identifier(id) => {
            let def_id = id.resolve();
            match grammar.definition(def_id) {
                Definition::Terminal(terminal) => {
                    first_sets.get_mut(target).unwrap().insert(terminal)
                }
                Definition::Nonterminal(nt) => {
                    // Copy FIRST(nt) into FIRST(target)
                    let source: Vec<_> = first_sets
                        .get(nt)
                        .map(|s| s.iter().copied().collect())
                        .unwrap_or_default();
                    let target_set = first_sets.get_mut(target).unwrap();
                    let mut added = false;
                    for terminal in source {
                        added |= target_set.insert(terminal);
                    }
                    added
                }
            }
        }
        Symbol::Literal(lit) => {
            // Find the terminal for this literal
            let terminal = grammar.terminals().find(|t| t.name == *lit);
            if let Some(terminal) = terminal {
                first_sets.get_mut(target).unwrap().insert(terminal)
            } else {
                false
            }
        }
        Symbol::Labeled { symbol, .. } | Symbol::Binding { symbol, .. } => {
            add_first_of_symbol(symbol, grammar, nullables, first_sets, target)
        }
        Symbol::Except { symbol, .. }
        | Symbol::FollowRestriction { symbol, .. }
        | Symbol::PrecedeRestriction { symbol, .. }
        | Symbol::Exclude { symbol, .. } => {
            add_first_of_symbol(symbol, grammar, nullables, first_sets, target)
        }
        Symbol::Group(symbols) => {
            let mut added = false;
            for s in symbols {
                added |= add_first_of_symbol(s, grammar, nullables, first_sets, target);
                if !is_nullable(s, grammar, nullables) {
                    break;
                }
            }
            added
        }
        Symbol::Alt(symbols) => {
            let mut added = false;
            for s in symbols {
                added |= add_first_of_symbol(s, grammar, nullables, first_sets, target);
            }
            added
        }
        Symbol::Opt(symbol) | Symbol::Star(symbol, _) | Symbol::Plus(symbol, _) => {
            add_first_of_symbol(symbol, grammar, nullables, first_sets, target)
        }
        Symbol::Call { name, .. } => {
            add_first_of_symbol(&Symbol::Identifier(name.clone()), grammar, nullables, first_sets, target)
        }
        Symbol::Condition(_) | Symbol::Return(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{alternative, grammar_def, id, lit, priority_level, syntax_rule};

    // ---------------------------------------------------------------
    // Grammar 1: Classic expression grammar (Dragon Book, Example 4.17)
    //
    //   E  = T Ep
    //   Ep = "+" T Ep | ε
    //   T  = F Tp
    //   Tp = "*" F Tp | ε
    //   F  = "(" E ")" | "id"
    // ---------------------------------------------------------------

    fn expression_grammar() -> Grammar {
        grammar_def!("expr",
            syntax: [
                syntax_rule!("E" => alternative!(id!("T"), id!("Ep"))),
                syntax_rule!("Ep" => priority_level!(
                    alternative!(lit!("+"), id!("T"), id!("Ep")),
                    alternative!()
                )),
                syntax_rule!("T" => alternative!(id!("F"), id!("Tp"))),
                syntax_rule!("Tp" => priority_level!(
                    alternative!(lit!("*"), id!("F"), id!("Tp")),
                    alternative!()
                )),
                syntax_rule!("F" => priority_level!(
                    alternative!(lit!("("), id!("E"), lit!(")")),
                    alternative!(lit!("id"))
                ))
            ]
        ).into()
    }

    #[test]
    fn test_nullables_expression_grammar() {
        let grammar = expression_grammar();
        let nullables = calc_nullables(&grammar);

        let ep = grammar.nonterminal("Ep").unwrap();
        let tp = grammar.nonterminal("Tp").unwrap();
        let e = grammar.nonterminal("E").unwrap();
        let t = grammar.nonterminal("T").unwrap();
        let f = grammar.nonterminal("F").unwrap();

        assert!(nullables.contains(ep));
        assert!(nullables.contains(tp));
        assert!(!nullables.contains(e));
        assert!(!nullables.contains(t));
        assert!(!nullables.contains(f));
    }

    #[test]
    fn test_first_sets_expression_grammar() {
        let grammar = expression_grammar();
        let first_sets = calc_first_sets(&grammar);

        let lparen = grammar.terminal("\"(\"").unwrap();
        let rparen = grammar.terminal("\")\"").unwrap();
        let plus = grammar.terminal("\"+\"").unwrap();
        let star = grammar.terminal("\"*\"").unwrap();
        let id_terminal = grammar.terminal("\"id\"").unwrap();
        let layout = grammar.terminal("Layout").unwrap();

        let first_e = &first_sets[grammar.nonterminal("E").unwrap()];
        assert!(first_e.contains(lparen));
        assert!(first_e.contains(id_terminal));
        assert!(!first_e.contains(layout)); // T is first symbol and not nullable
        assert!(!first_e.contains(plus));
        assert!(!first_e.contains(star));
        assert!(!first_e.contains(rparen));

        let first_ep = &first_sets[grammar.nonterminal("Ep").unwrap()];
        assert!(first_ep.contains(plus));
        assert_eq!(first_ep.len(), 1);

        let first_f = &first_sets[grammar.nonterminal("F").unwrap()];
        assert!(first_f.contains(lparen));
        assert!(first_f.contains(id_terminal));
        assert_eq!(first_f.len(), 2);

        let first_tp = &first_sets[grammar.nonterminal("Tp").unwrap()];
        assert!(first_tp.contains(star));
        assert_eq!(first_tp.len(), 1);
    }

    // ---------------------------------------------------------------
    // Grammar 2: Multiple nullable prefixes
    //
    //   S = A B C "d"
    //   A = "a" | ε
    //   B = "b" | ε
    //   C = "c" | ε
    // ---------------------------------------------------------------

    fn nullable_prefix_grammar() -> Grammar {
        grammar_def!("nullable",
            syntax: [
                syntax_rule!("S" => alternative!(id!("A"), id!("B"), id!("C"), lit!("d"))),
                syntax_rule!("A" => priority_level!(alternative!(lit!("a")), alternative!())),
                syntax_rule!("B" => priority_level!(alternative!(lit!("b")), alternative!())),
                syntax_rule!("C" => priority_level!(alternative!(lit!("c")), alternative!()))
            ]
        ).into()
    }

    #[test]
    fn test_nullables_nullable_prefix_grammar() {
        let grammar = nullable_prefix_grammar();
        let nullables = calc_nullables(&grammar);

        let a = grammar.nonterminal("A").unwrap();
        let b = grammar.nonterminal("B").unwrap();
        let c = grammar.nonterminal("C").unwrap();
        let s = grammar.nonterminal("S").unwrap();

        assert!(nullables.contains(a));
        assert!(nullables.contains(b));
        assert!(nullables.contains(c));
        assert!(!nullables.contains(s));
    }

    #[test]
    fn test_first_sets_nullable_prefix_grammar() {
        let grammar = nullable_prefix_grammar();
        let first_sets = calc_first_sets(&grammar);

        let ta = grammar.terminal("\"a\"").unwrap();
        let tb = grammar.terminal("\"b\"").unwrap();
        let tc = grammar.terminal("\"c\"").unwrap();
        let td = grammar.terminal("\"d\"").unwrap();
        let layout = grammar.terminal("Layout").unwrap();

        let first_s = &first_sets[grammar.nonterminal("S").unwrap()];
        assert!(first_s.contains(ta));
        assert!(first_s.contains(tb));
        assert!(first_s.contains(tc));
        assert!(first_s.contains(td));
        assert!(first_s.contains(layout)); // Layout is nullable, inserted between symbols

        let first_a = &first_sets[grammar.nonterminal("A").unwrap()];
        assert!(first_a.contains(ta));
        assert_eq!(first_a.len(), 1);

        let first_b = &first_sets[grammar.nonterminal("B").unwrap()];
        assert!(first_b.contains(tb));
        assert_eq!(first_b.len(), 1);

        let first_c = &first_sets[grammar.nonterminal("C").unwrap()];
        assert!(first_c.contains(tc));
        assert_eq!(first_c.len(), 1);
    }

    // ---------------------------------------------------------------
    // Grammar 3: Mutually recursive FIRST sets
    //
    //   S = A B "c"
    //   A = "a" | ε
    //   B = S "d" | ε
    // ---------------------------------------------------------------

    fn recursive_first_grammar() -> Grammar {
        grammar_def!("recursive",
            syntax: [
                syntax_rule!("S" => alternative!(id!("A"), id!("B"), lit!("c"))),
                syntax_rule!("A" => priority_level!(alternative!(lit!("a")), alternative!())),
                syntax_rule!("B" => priority_level!(
                    alternative!(id!("S"), lit!("d")),
                    alternative!()
                ))
            ]
        ).into()
    }

    #[test]
    fn test_nullables_recursive_first_grammar() {
        let grammar = recursive_first_grammar();
        let nullables = calc_nullables(&grammar);

        let a = grammar.nonterminal("A").unwrap();
        let b = grammar.nonterminal("B").unwrap();
        let s = grammar.nonterminal("S").unwrap();

        assert!(nullables.contains(a));
        assert!(nullables.contains(b));
        assert!(!nullables.contains(s));
    }

    #[test]
    fn test_first_sets_recursive_first_grammar() {
        let grammar = recursive_first_grammar();
        let first_sets = calc_first_sets(&grammar);

        let ta = grammar.terminal("\"a\"").unwrap();
        let tc = grammar.terminal("\"c\"").unwrap();
        let td = grammar.terminal("\"d\"").unwrap();
        let layout = grammar.terminal("Layout").unwrap();

        let first_s = &first_sets[grammar.nonterminal("S").unwrap()];
        assert!(first_s.contains(ta));
        assert!(first_s.contains(tc));
        assert!(first_s.contains(layout));
        assert!(!first_s.contains(td));

        let first_a = &first_sets[grammar.nonterminal("A").unwrap()];
        assert!(first_a.contains(ta));
        assert_eq!(first_a.len(), 1);

        let first_b = &first_sets[grammar.nonterminal("B").unwrap()];
        assert!(first_b.contains(ta));
        assert!(first_b.contains(tc));
        assert!(first_b.contains(layout));
        assert!(!first_b.contains(td));
    }
}
