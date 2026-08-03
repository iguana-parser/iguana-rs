use rustc_hash::{FxHashMap, FxHashSet};

use crate::generator::id::TerminalIds;
use crate::grammar::def::Grammar;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::{Definition, Nonterminal, Terminal};
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
    /// alternative `alt` (a `!>>` restriction). A residual `!>>>`
    /// restriction joins this set and is checked at the same position as a
    /// plain `!>>`.
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
                let restrictions = symbol.restrictions();
                // The same identifier can appear in both lists (`X !>> T
                // !>>> T`); it enters the set once.
                let terminals: Vec<_> = restrictions
                    .follow
                    .iter()
                    .chain(
                        restrictions
                            .layout_aware_follow
                            .iter()
                            .filter(|r| !restrictions.follow.contains(r)),
                    )
                    .map(|r| {
                        let Definition::Terminal(t) = grammar.definition(r.resolve()) else {
                            panic!("follow restriction must resolve to a terminal");
                        };
                        t.clone()
                    })
                    .collect();
                if terminals.is_empty() {
                    continue;
                }
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

/// Content-deduplicated ids for one family of terminal sets.
///
/// `match_any` and `longest_match` are both order-insensitive, so two sets with
/// the same terminals share an id. Ids are assigned by content in the order
/// `terminal_sets` yields the sets, so the generated code is deterministic. The
/// two families get separate id spaces, each numbered from zero: `match_any`
/// keys its per-position memo by this id; `longest_match` is not memoized, but
/// its sets are numbered the same way so every `TerminalSet` has one id.
pub struct SetIds {
    ids: FxHashMap<String, usize>,
    count: usize,
}

impl SetIds {
    /// Ids for the sets the parser tests with `match_any`: every set except the
    /// combined FIRST set.
    pub fn match_any(sets: &[TerminalSet], terminal_ids: &TerminalIds) -> Self {
        Self::new(sets, terminal_ids, |kind| !matches!(kind, SetKind::First))
    }

    /// Ids for the combined FIRST sets, which the LL(1) path dispatches on with
    /// `longest_match`.
    pub fn longest_match(sets: &[TerminalSet], terminal_ids: &TerminalIds) -> Self {
        Self::new(sets, terminal_ids, |kind| matches!(kind, SetKind::First))
    }

    fn new(
        sets: &[TerminalSet],
        terminal_ids: &TerminalIds,
        include: impl Fn(&SetKind) -> bool,
    ) -> Self {
        let mut ids = FxHashMap::default();
        let mut content_ids: FxHashMap<Vec<TerminalId>, usize> = FxHashMap::default();
        for set in sets {
            if !include(&set.kind) {
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
            .unwrap_or_else(|| panic!("no set id for `{set_name}`"))
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

fn terminal_names(terminals: &[Terminal]) -> String {
    let names: Vec<_> = terminals.iter().map(|t| t.name.clone()).collect();
    format!("{{ {} }}", names.join(", "))
}
