use rustc_hash::{FxHashMap, FxHashSet};

use crate::generator::grammar_utils::is_ll1_return_slot;
use crate::generator::id::TerminalIds;
use crate::grammar::def::Grammar;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::{Definition, Identifier, Nonterminal, Symbol, Terminal};
use crate::ids::TerminalId;

pub enum TerminalSetKind<'a> {
    /// FOLLOW set of the nonterminal.
    Follow(&'a Nonterminal),
    /// The terminals that can start the rest of the alternative at a return
    /// slot (see `return_slot_lookahead`).
    ReturnSlotLookahead(Slot<'a>),
    /// FIRST set of the nonterminal, the union of the FIRST sets of its
    /// alternatives. This is only used in the LL(1) path, where the LL(1)
    /// prediction runs `longest_match` on it.
    First(&'a Nonterminal),
    /// Terminals forbidden right after the symbol at the slot (a `!>>`
    /// restriction).
    FollowRestriction(Slot<'a>),
    /// Terminals forbidden after the layout that follows the symbol at the
    /// slot (a `!>>>` restriction). The check happens at the right extent of
    /// that layout, whereas a `FollowRestriction` is checked at the symbol's
    /// right extent.
    LayoutAwareFollowRestriction(Slot<'a>),
    /// The operands of the except at the grammar slot (a `\` except). A match
    /// of the symbol is rejected when one of the operands matches the same
    /// span exactly. Except checks run by `match_exact`, and a failure
    /// reports the terminal set of this kind.
    Except(Slot<'a>),
}

/// A set of terminals that the parser passes to the scanner for matching, for
/// example a follow restriction, or that a recorded failure refers to. The
/// generator emits each set as a static in the generated parser and refers to
/// it where that action runs.
pub struct TerminalSet<'a> {
    pub kind: TerminalSetKind<'a>,
    pub terminals: Vec<Terminal>,
    /// The id of the set. The scanner memoizes each `match_any` result in a
    /// bitset per input position, at bit `id`. The bitset has
    /// `match_any_count` bits, one per distinct `match_any` set id. Two
    /// `match_any` sets with the same terminals share an id, and therefore a
    /// memo bit.
    pub id: usize,
}

/// Builds the terminal sets of every nonterminal and of every return slot with
/// a lookahead, and assigns their ids.
/// Returns the sets and the number of distinct `match_any` set ids, which
/// are the ids below that count.
pub fn terminal_sets<'a>(
    grammar: &'a Grammar,
    ff: &FirstFollowSets,
    terminal_ids: &TerminalIds,
    ll1_nonterminals: &FxHashSet<&Nonterminal>,
) -> (Vec<TerminalSet<'a>>, usize) {
    let mut sets = vec![];

    // The sets the parser tests with `match_any`, numbered from zero. Sets
    // with the same terminals share an id.
    let mut match_any_ids: FxHashMap<Vec<TerminalId>, usize> = FxHashMap::default();
    let mut match_any_set = |kind: TerminalSetKind<'a>, terminals: Vec<Terminal>| {
        let terminals = canonical_order(terminals, terminal_ids);
        let content = terminals.iter().map(|t| terminal_ids.get_id(t)).collect();
        let next_id = match_any_ids.len();
        let id = *match_any_ids.entry(content).or_insert(next_id);
        TerminalSet {
            kind,
            terminals,
            id,
        }
    };
    for nonterminal in grammar.nonterminals() {
        sets.push(match_any_set(
            TerminalSetKind::Follow(nonterminal),
            ff.follow_set(nonterminal).cloned().collect(),
        ));
        for alternative in grammar.alternatives(nonterminal) {
            for (pos, symbol) in alternative.symbols.iter().enumerate() {
                let restrictions = symbol.restrictions();
                let slot = Slot::new(nonterminal, alternative, pos);
                if !restrictions.follow.is_empty() {
                    sets.push(match_any_set(
                        TerminalSetKind::FollowRestriction(slot.clone()),
                        restriction_terminals(grammar, &restrictions.follow),
                    ));
                }
                if !restrictions.layout_aware_follow.is_empty() {
                    sets.push(match_any_set(
                        TerminalSetKind::LayoutAwareFollowRestriction(slot),
                        restriction_terminals(grammar, &restrictions.layout_aware_follow),
                    ));
                }
            }
        }
    }
    for nonterminal in grammar.nonterminals() {
        for alternative in grammar.alternatives(nonterminal) {
            for pos in 1..alternative.symbols.len() {
                let slot = Slot::new(nonterminal, alternative, pos);
                if let Some(lookahead) = return_slot_lookahead(grammar, ff, ll1_nonterminals, &slot)
                {
                    sets.push(match_any_set(
                        TerminalSetKind::ReturnSlotLookahead(slot),
                        lookahead.terminals,
                    ));
                }
            }
        }
    }
    let match_any_count = match_any_ids.len();

    // The combined FIRST sets, which `longest_match` tests, numbered after the
    // `match_any` sets. The except sets, which only a failure refers to, share
    // this family. None of them is passed to `match_any`.
    let mut first_ids: FxHashMap<Vec<TerminalId>, usize> = FxHashMap::default();
    let mut first_set = |kind: TerminalSetKind<'a>, terminals: Vec<Terminal>| {
        let content = terminals.iter().map(|t| terminal_ids.get_id(t)).collect();
        let next_id = match_any_count + first_ids.len();
        let id = *first_ids.entry(content).or_insert(next_id);
        TerminalSet {
            kind,
            terminals,
            id,
        }
    };
    for nonterminal in grammar.nonterminals() {
        if !ll1_nonterminals.contains(nonterminal) {
            continue;
        }
        let terminals = grammar
            .alternatives(nonterminal)
            .iter()
            .flat_map(|alternative| ff.first_set(alternative))
            .collect();
        let terminals = canonical_order(terminals, terminal_ids);
        sets.push(first_set(TerminalSetKind::First(nonterminal), terminals));
    }
    for nonterminal in grammar.nonterminals() {
        for alternative in grammar.alternatives(nonterminal) {
            for (pos, symbol) in alternative.symbols.iter().enumerate() {
                let excepts = &symbol.restrictions().excepts;
                if excepts.is_empty() || symbol.as_identifier().is_none() {
                    continue;
                }
                let terminals = excepts
                    .iter()
                    .map(|e| grammar.except_terminal(e).0.clone())
                    .collect();
                sets.push(first_set(
                    TerminalSetKind::Except(Slot::new(nonterminal, alternative, pos)),
                    terminals,
                ));
            }
        }
    }

    (sets, match_any_count)
}

/// The lookahead of a return slot, computed from the symbols that follow the
/// slot in its alternative.
pub struct ReturnSlotLookahead {
    /// Whether the lookahead skips the layout at this return slot. Only
    /// applicable if the layout is a terminal or LL(1).
    pub skips_layout: bool,
    /// The terminals that can start the rest of the alternative.
    pub terminals: Vec<Terminal>,
}

/// The lookahead for the return slot.
///
/// - A slot that is not a return slot, or is the return slot of an LL(1)
///   nonterminal, gives `None`.
/// - Data-dependent conditions and returns, which do not consume input, are
///   skipped.
/// - A layout at the start of the rest is skipped when it is a terminal or
///   LL(1). Any other layout there gives `None`.
/// - The lookahead set is then the FIRST set of the rest of the alternative:
///   the FIRST sets of the symbols up to and including the first non-nullable
///   one. A rest without a non-nullable symbol gives `None`.
///
/// The lookahead set ignores the restrictions (`!>>`, `!>>>`, `!<<`, `\`) and the
/// conditions in the rest of the alternative, which has two consequences:
///
/// - The set can contain a terminal that the continuation rejects. The test
///   therefore never rejects a return that could succeed, but it can accept a
///   return whose continuation fails later.
/// - A rejected return records the whole set as the expected terminals. The
///   failure can therefore list a terminal that the continuation would not
///   have reported, such as a keyword whose `!<<` restriction does not hold at
///   the position.
pub fn return_slot_lookahead(
    grammar: &Grammar,
    ff: &FirstFollowSets,
    ll1_nonterminals: &FxHashSet<&Nonterminal>,
    slot: &Slot,
) -> Option<ReturnSlotLookahead> {
    if !slot.is_return_slot(grammar) || is_ll1_return_slot(grammar, ll1_nonterminals, slot) {
        return None;
    }
    let symbols = &slot.alternative().symbols;
    let mut rest = (slot.pos()..symbols.len())
        .filter(|&pos| !matches!(symbols[pos], Symbol::Condition(_) | Symbol::Return(_)))
        .peekable();
    let layout = grammar
        .layout
        .as_ref()
        .and_then(Symbol::as_identifier)
        .map(Identifier::resolve);
    let leading_layout = layout.filter(|&layout| {
        rest.peek()
            .and_then(|&pos| symbols[pos].as_identifier())
            .is_some_and(|id| id.resolve() == layout)
    });
    let skips_layout = leading_layout.is_some();
    if let Some(layout) = leading_layout {
        let measurable = match grammar.definition(layout) {
            Definition::Terminal(_) => true,
            Definition::Nonterminal(nonterminal) => ll1_nonterminals.contains(nonterminal),
        };
        if !measurable {
            return None;
        }
        rest.next();
    }
    let mut terminals = FxHashSet::default();
    for pos in rest {
        terminals.extend(ff.first_of_symbol(&symbols[pos]));
        if !ff.is_nullable(&symbols[pos]) {
            return Some(ReturnSlotLookahead {
                skips_layout,
                terminals: terminals.into_iter().collect(),
            });
        }
    }
    None
}

/// Sorts `terminals` by ascending terminal id and removes duplicates, so that
/// the same set is emitted the same way regardless of how it was collected.
fn canonical_order(mut terminals: Vec<Terminal>, terminal_ids: &TerminalIds) -> Vec<Terminal> {
    terminals.sort_by_key(|t| terminal_ids.get_id(t).0);
    terminals.dedup();
    terminals
}

fn restriction_terminals(grammar: &Grammar, restrictions: &[Identifier]) -> Vec<Terminal> {
    restrictions
        .iter()
        .map(|r| {
            let Definition::Terminal(t) = grammar.definition(r.resolve()) else {
                panic!("follow restriction must resolve to a terminal");
            };
            t.clone()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenConfig;
    use crate::generator::grammar_utils::ll1_nonterminals;
    use crate::iggy::parse_grammar;

    /// The lookahead of the slot after the first `called` in the first
    /// alternative of `head` whose symbols include `contains`, as whether it
    /// skips the layout and the sorted names of its terminals.
    fn lookahead(
        source: &str,
        ll1_optimization: bool,
        head: &str,
        contains: &[&str],
        called: &str,
    ) -> Option<(bool, Vec<String>)> {
        let grammar: Grammar = parse_grammar(source).unwrap().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);
        let config = GenConfig {
            ll1_optimization,
            ..GenConfig::default()
        };
        let ll1_nonterminals = ll1_nonterminals(&grammar, &ff, &config);
        let name = |symbol: &Symbol| match symbol.as_identifier() {
            Some(id) => grammar.definition(id.resolve()).display_name(),
            None => symbol.to_string(),
        };
        let head = grammar.nonterminal(head).unwrap();
        let alternative = grammar
            .alternatives(head)
            .iter()
            .find(|alternative| {
                let names: Vec<_> = alternative.symbols.iter().map(name).collect();
                contains.iter().all(|c| names.iter().any(|n| n == c))
            })
            .unwrap();
        let pos = alternative
            .symbols
            .iter()
            .position(|s| name(s) == called)
            .unwrap()
            + 1;
        let slot = Slot::new(head, alternative, pos);
        return_slot_lookahead(&grammar, &ff, &ll1_nonterminals, &slot).map(|lookahead| {
            let mut names: Vec<_> = lookahead.terminals.iter().map(|t| t.name.clone()).collect();
            names.sort();
            (lookahead.skips_layout, names)
        })
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    const REGEX_LAYOUT: &str = r#"
        grammar G
        S = A "x" | "y" A | A B | A B "x" "z"
        A = "a" | "a" "b"
        B = "b" | ()
        @Layout @Regex
        WS = [\ ]*
        "#;

    const NONTERMINAL_LAYOUT: &str = r#"
        grammar G
        S = A "x"
        A = "a" | "a" "b"
        @Layout
        Layout = WhiteSpace*
        @Regex
        WhiteSpace = [\ ]+
        "#;

    #[test]
    fn a_regex_layout_is_skipped() {
        assert_eq!(
            lookahead(REGEX_LAYOUT, true, "S", &["A", r#""x""#], "A"),
            Some((true, names(&[r#""x""#])))
        );
    }

    #[test]
    fn an_ll1_layout_is_skipped_and_any_other_layout_gives_none() {
        assert_eq!(
            lookahead(NONTERMINAL_LAYOUT, true, "S", &["A"], "A"),
            Some((true, names(&[r#""x""#])))
        );
        assert_eq!(lookahead(NONTERMINAL_LAYOUT, false, "S", &["A"], "A"), None);
    }

    #[test]
    fn the_first_sets_are_collected_up_to_the_first_non_nullable_symbol() {
        // The layout between `B` and `"x"` is an ordinary nullable symbol. The
        // walk stops at `"x"`, so the set does not have `"z"`.
        assert_eq!(
            lookahead(REGEX_LAYOUT, true, "S", &["B", r#""x""#], "A"),
            Some((true, names(&[r#""b""#, r#""x""#, "WS"])))
        );
    }

    #[test]
    fn a_nullable_or_empty_rest_gives_none() {
        assert_eq!(lookahead(REGEX_LAYOUT, true, "S", &["B"], "A"), None);
        assert_eq!(lookahead(REGEX_LAYOUT, true, "S", &[r#""y""#], "A"), None);
    }

    #[test]
    fn the_return_slot_of_an_ll1_nonterminal_gives_none() {
        let source = r#"
            grammar G
            S = A "x"
            A = "a"
            @Layout @Regex
            WS = [\ ]*
            "#;
        assert_eq!(lookahead(source, true, "S", &["A"], "A"), None);
        assert!(lookahead(source, false, "S", &["A"], "A").is_some());
    }

    #[test]
    fn conditions_after_the_call_are_skipped() {
        // The desugaring places the precedence condition between the call and
        // the layout, so the layout is skipped only if the condition is.
        let source = r#"
            grammar G
            S = E
            E = "a" > E "*" E > E "+" E
            @Layout @Regex
            WS = [\ ]*
            "#;
        assert_eq!(
            lookahead(source, true, "E", &[r#""*""#], "E"),
            Some((true, names(&[r#""*""#])))
        );
    }
}
