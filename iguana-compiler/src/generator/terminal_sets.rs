use rustc_hash::{FxHashMap, FxHashSet};

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

/// Builds the terminal sets of every nonterminal and assigns their ids.
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
