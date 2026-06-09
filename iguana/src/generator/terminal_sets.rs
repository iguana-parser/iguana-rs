use rustc_hash::{FxHashMap, FxHashSet};

use crate::generator::id::TerminalIds;
use crate::grammar::def::Grammar;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::{Definition, Nonterminal, Symbol, Terminal};
use crate::ids::TerminalId;
use crate::utils::to_snake_case;

/// Which first/follow set a `TerminalSet` is. The kind determines the emitted
/// static's name and comment, and whether the parser queries the set with
/// `match_any`.
pub enum SetKind {
    /// FOLLOW set of the nonterminal: the terminals that may follow it.
    Follow,
    /// FIRST set of the nonterminal, combined over all its alternatives. The
    /// LL(1) path dispatches on it with `longest_match`.
    First,
    /// FIRST set of an alternative, identified by its index. The GLL path
    /// tests each alternative's set with `match_any` to choose which
    /// alternatives to start.
    FirstAlt(usize),
    /// Terminals forbidden right after the symbol at position `pos` in
    /// alternative `alt` (a `!>>` restriction).
    FollowRestriction { alt: usize, pos: usize },
}

/// A terminal set emitted as a `static &[TerminalId]`. The name and comment are
/// derived from the nonterminal and kind; `terminals` holds the contents.
pub struct TerminalSet<'a> {
    pub nonterminal: &'a Nonterminal,
    pub kind: SetKind,
    pub terminals: Vec<Terminal>,
}

impl TerminalSet<'_> {
    /// Identifier of the emitted static, e.g. `FIRST_SET_E_ALT0`.
    pub fn name(&self) -> String {
        let nt = to_snake_case(&self.nonterminal.name).to_uppercase();
        match self.kind {
            SetKind::Follow => format!("FOLLOW_SET_{nt}"),
            SetKind::First => format!("FIRST_SET_{nt}"),
            SetKind::FirstAlt(alt) => format!("FIRST_SET_{nt}_ALT{alt}"),
            SetKind::FollowRestriction { alt, pos } => {
                format!("FOLLOW_RESTRICTION_{nt}_ALT{alt}_POS{pos}")
            }
        }
    }

    /// Documentation line above the static, e.g. `E ::= . E "+" E { "a" }`.
    pub fn comment(&self, grammar: &Grammar) -> String {
        let names = terminal_names(&self.terminals);
        match self.kind {
            SetKind::Follow | SetKind::First => format!("{} {names}", self.nonterminal.name),
            SetKind::FirstAlt(alt) => {
                let alternative = &grammar.alternatives(self.nonterminal)[alt];
                format!(
                    "{} {names}",
                    Slot::new(self.nonterminal, alternative, 0).name()
                )
            }
            SetKind::FollowRestriction { alt, pos } => {
                let alternative = &grammar.alternatives(self.nonterminal)[alt];
                format!(
                    "{} !>> {names}",
                    Slot::new(self.nonterminal, alternative, pos).name()
                )
            }
        }
    }
}

pub fn terminal_sets<'a>(grammar: &'a Grammar, ff: &FirstFollowSets) -> Vec<TerminalSet<'a>> {
    let mut sets = vec![];
    for nonterminal in grammar.nonterminals() {
        let alternatives = grammar.alternatives(nonterminal);

        sets.push(TerminalSet {
            nonterminal,
            kind: SetKind::Follow,
            terminals: ff.follow_set(nonterminal).cloned().collect(),
        });

        sets.push(TerminalSet {
            nonterminal,
            kind: SetKind::First,
            terminals: alternatives
                .iter()
                .flat_map(|alt| ff.first_set(alt))
                .collect::<FxHashSet<_>>()
                .into_iter()
                .collect(),
        });

        for (alt, alternative) in alternatives.iter().enumerate() {
            sets.push(TerminalSet {
                nonterminal,
                kind: SetKind::FirstAlt(alt),
                terminals: ff.first_set(alternative).into_iter().collect(),
            });
        }

        for (alt, alternative) in alternatives.iter().enumerate() {
            for (pos, symbol) in alternative.symbols.iter().enumerate() {
                let Symbol::FollowRestriction { restrictions, .. } = symbol else {
                    continue;
                };
                let terminals = restrictions
                    .iter()
                    .map(|r| {
                        let Definition::Terminal(t) = grammar.definition(r.resolve()) else {
                            panic!("follow restriction must resolve to a terminal");
                        };
                        t.clone()
                    })
                    .collect();
                sets.push(TerminalSet {
                    nonterminal,
                    kind: SetKind::FollowRestriction { alt, pos },
                    terminals,
                });
            }
        }
    }
    sets
}

/// Content-deduplicated ids for the terminal sets passed to `match_any`.
///
/// `match_any` is order-insensitive, so two sets with the same terminals share
/// an id and therefore a memo bit. Ids are assigned by content in the order
/// `terminal_sets` yields the sets, so the generated code is deterministic.
pub struct MatchAnySets {
    ids: FxHashMap<String, usize>,
    count: usize,
}

impl MatchAnySets {
    pub fn new(sets: &[TerminalSet], terminal_ids: &TerminalIds) -> Self {
        let mut ids = FxHashMap::default();
        let mut content_ids: FxHashMap<Vec<TerminalId>, usize> = FxHashMap::default();
        for set in sets {
            // The parser calls `longest_match` on the combined FIRST set and
            // `match_any` on every other set. Only `match_any` is memoized, so
            // the FIRST set gets no id.
            if matches!(set.kind, SetKind::First) {
                continue;
            }
            // Sort and dedup so sets that differ only in terminal order share an id.
            let mut content: Vec<TerminalId> = set
                .terminals
                .iter()
                .map(|t| terminal_ids.get_id(t))
                .collect();
            content.sort_by_key(|t| t.0);
            content.dedup();
            let next_id = content_ids.len();
            let id = *content_ids.entry(content).or_insert(next_id);
            ids.insert(set.name(), id);
        }

        Self {
            ids,
            count: content_ids.len(),
        }
    }

    pub fn id(&self, set_name: &str) -> usize {
        *self
            .ids
            .get(set_name)
            .unwrap_or_else(|| panic!("no match_any set id for `{set_name}`"))
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

fn terminal_names(terminals: &[Terminal]) -> String {
    let names: Vec<_> = terminals.iter().map(|t| t.name.clone()).collect();
    format!("{{ {} }}", names.join(", "))
}
