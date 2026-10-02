use rustc_hash::{FxHashMap, FxHashSet};

use crate::generator::GenConfig;
use crate::generator::id::TerminalIds;
use crate::grammar::def::Grammar;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::{Definition, Identifier, Nonterminal, Terminal};
use crate::ids::TerminalId;

pub enum TerminalSetKind<'a> {
    /// FOLLOW set of the nonterminal.
    Follow(&'a Nonterminal),
    /// FIRST set of the nonterminal, the union of the FIRST sets of its
    /// alternatives. This is only used in the LL(1) path, where the LL(1)
    /// prediction runs `longest_match` on it.
    First(&'a Nonterminal),
    /// FIRST set of the alternative at the slot. GLL runs `match_any` on it
    /// to choose which alternatives to schedule.
    FirstAlt(Slot<'a>),
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
    /// The terminals that a nonterminal starts with: its FIRST set, plus its
    /// FOLLOW set if the nonterminal is nullable. Used only in the GLL path,
    /// when the prediction set is tested with `match_any` before creating a
    /// GSS node.
    Prediction(&'a Nonterminal),
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

/// Builds the terminal sets of every nonterminal and assigns their ids.
/// Returns the sets and the number of distinct `match_any` set ids, which
/// are the ids below that count.
pub fn terminal_sets<'a>(
    grammar: &'a Grammar,
    ff: &FirstFollowSets,
    terminal_ids: &TerminalIds,
    config: &GenConfig,
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
    // The terminals of each nonterminal's combined FIRST set, collected from
    // its alternatives on the way.
    let mut combined_first = vec![];
    for nonterminal in grammar.nonterminals() {
        sets.push(match_any_set(
            TerminalSetKind::Follow(nonterminal),
            ff.follow_set(nonterminal).cloned().collect(),
        ));
        let mut first = vec![];
        let alternatives = grammar.alternatives(nonterminal);
        for alternative in alternatives {
            let alternative_first: Vec<Terminal> = ff.first_set(alternative).into_iter().collect();
            first.extend(alternative_first.iter().cloned());
            // The generator does not create a FIRST set static for a single
            // alternative. A single alternative is scheduled without testing
            // its FIRST set, because the call already tested the nonterminal's
            // prediction set, which for a single alternative is the same set.
            if alternatives.len() > 1 {
                sets.push(match_any_set(
                    TerminalSetKind::FirstAlt(Slot::new(nonterminal, alternative, 0)),
                    alternative_first,
                ));
            }
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
        combined_first.push((nonterminal, first));
    }
    // The nonterminals that are called in the GLL path, i.e., the ones the
    // parser creates a GSS node for. Only a GLL call tests a prediction set,
    // so only these nonterminals get one.
    let gll_nonterminals: FxHashSet<_> = grammar
        .nonterminals()
        .flat_map(|nt| grammar.alternatives(nt))
        .flat_map(|alt| &alt.symbols)
        .filter_map(|symbol| symbol.as_identifier())
        .filter_map(
            |identifier| match grammar.definition(identifier.resolve()) {
                Definition::Nonterminal(nt) if !(config.ll1_optimization && ff.is_ll1(nt)) => {
                    Some(nt)
                }
                _ => None,
            },
        )
        .collect();
    for (nonterminal, terminals) in &combined_first {
        if !gll_nonterminals.contains(nonterminal) {
            continue;
        }
        let mut prediction = terminals.clone();
        if ff.is_nonterminal_nullable(nonterminal) {
            prediction.extend(ff.follow_set(nonterminal).cloned());
        }
        sets.push(match_any_set(
            TerminalSetKind::Prediction(nonterminal),
            prediction,
        ));
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
    for (nonterminal, terminals) in combined_first {
        if !(config.ll1_optimization && ff.is_ll1(nonterminal)) {
            continue;
        }
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
    use crate::iggy::parse_grammar;

    /// The heads of the `FirstAlt` sets and the nonterminals with a prediction
    /// set, each sorted by name. Every prediction set must have a `match_any`
    /// memo id, since only a GLL call tests one.
    fn set_summary(source: &str, ll1_optimization: bool) -> (Vec<String>, Vec<String>) {
        let grammar: Grammar = parse_grammar(source).unwrap().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);
        let mut terminal_ids = TerminalIds::default();
        for terminal in grammar.terminals() {
            terminal_ids.insert(terminal.clone());
        }
        let config = GenConfig {
            ll1_optimization,
            ..GenConfig::default()
        };
        let (sets, count) = terminal_sets(&grammar, &ff, &terminal_ids, &config);
        let mut first_alts = vec![];
        let mut predictions = vec![];
        for set in &sets {
            match &set.kind {
                TerminalSetKind::FirstAlt(slot) => first_alts.push(slot.head().name.clone()),
                TerminalSetKind::Prediction(nt) => {
                    assert!(
                        set.id < count,
                        "prediction set of {} without a memo id",
                        nt.name
                    );
                    predictions.push(nt.name.clone());
                }
                _ => {}
            }
        }
        first_alts.sort();
        predictions.sort();
        (first_alts, predictions)
    }

    #[test]
    fn only_gll_nonterminals_have_prediction_sets() {
        // A is not LL(1). `C?` becomes the nullable nonterminal Opt_0. Nothing
        // calls the start wrappers, so they do not have prediction sets.
        let source = "grammar G\nS = A B C?\nA = \"a\" | \"a\" \"x\"\nB = \"b\"\nC = \"c\"\n";
        let expected_first_alts = ["A", "A", "Opt_0", "Opt_0"];
        let (first_alts, predictions) = set_summary(source, false);
        assert_eq!(first_alts, expected_first_alts);
        assert_eq!(predictions, ["A", "B", "C", "Opt_0", "S"]);
        // With the LL(1) path, B, C, and Opt_0 are parsed by their LL(1)
        // functions, not through a GLL call.
        let (first_alts, predictions) = set_summary(source, true);
        assert_eq!(first_alts, expected_first_alts);
        assert_eq!(predictions, ["A", "S"]);
    }
}
