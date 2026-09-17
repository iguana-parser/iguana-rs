use rustc_hash::FxHashMap;

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
    /// alternatives. The LL(1) prediction runs `longest_match` on it.
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
}

/// A set of terminals that a parsing action checks the input against, for
/// example a follow restriction. The generator emits each set as a static in
/// the generated parser and refers to it where that action runs.
pub struct TerminalSet<'a> {
    pub kind: TerminalSetKind<'a>,
    pub terminals: Vec<Terminal>,
    /// The id of the set. Two `match_any` sets with the same terminals share
    /// an id, and so do two combined FIRST sets, but the two groups are
    /// numbered separately.
    ///
    /// The scanner memoizes each `match_any` result in a bitset per input
    /// position, at bit `id`. The bitset has `match_any_count` bits, one per
    /// distinct `match_any` set id.
    pub id: usize,
}

/// Builds the terminal sets of every nonterminal and assigns their ids.
/// Returns the sets and the number of distinct `match_any` set ids, which
/// are the ids below that count.
pub fn terminal_sets<'a>(
    grammar: &'a Grammar,
    ff: &FirstFollowSets,
    terminal_ids: &TerminalIds,
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
        for alternative in grammar.alternatives(nonterminal) {
            let alternative_first: Vec<Terminal> = ff.first_set(alternative).into_iter().collect();
            first.extend(alternative_first.iter().cloned());
            sets.push(match_any_set(
                TerminalSetKind::FirstAlt(Slot::new(nonterminal, alternative, 0)),
                alternative_first,
            ));
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
    let match_any_count = match_any_ids.len();

    // The combined FIRST sets, which `longest_match` tests, numbered after the
    // `match_any` sets.
    let mut first_ids: FxHashMap<Vec<TerminalId>, usize> = FxHashMap::default();
    for (nonterminal, terminals) in combined_first {
        let terminals = canonical_order(terminals, terminal_ids);
        let content = terminals.iter().map(|t| terminal_ids.get_id(t)).collect();
        let next_id = match_any_count + first_ids.len();
        let id = *first_ids.entry(content).or_insert(next_id);
        sets.push(TerminalSet {
            kind: TerminalSetKind::First(nonterminal),
            terminals,
            id,
        });
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
