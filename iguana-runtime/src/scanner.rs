use crate::{
    arena::{Arena, ArenaVec},
    dfa::Dfa,
    ids::TerminalId,
    prediction::{AlternativeSet, Prediction},
    utils::inline_map::InlineMap,
};

pub struct Token {
    pub token_type: TerminalId,
    pub range: Range,
}

pub struct Range {
    pub start: u32,
    pub end: u32,
}

/// Recorded result of a previous `match_token` call.
pub enum Lookup {
    /// The terminal was tried at this position and did not match.
    Fail,
    /// The terminal matched, ending at this exclusive position.
    Match(u32),
}

/// Per-position memo cell. For each terminal it records whether `match_token`
/// has already run here, and if so whether the terminal matched.
///
/// Those are three states: not run, ran and matched, ran and did not match.
/// The fields hold them:
///
/// - `tried` is a bitset indexed by terminal id that marks the terminal as run
///   at this position.
/// - `matched` is a bitset indexed by terminal id that marks the terminal as
///   matched. A bit is set only together with the terminal's bit in `tried`.
/// - `ends` holds the end of each match.
///
/// A terminal's bit sits at word `id / 64`, bit `id % 64`.
pub struct MatchMemoEntry<'arena, const W: usize> {
    tried: [u64; W],
    matched: [u64; W],
    ends: InlineMap<'arena, TerminalId, u32>,
}

impl<'arena, const W: usize> Default for MatchMemoEntry<'arena, W> {
    fn default() -> Self {
        Self {
            tried: [0; W],
            matched: [0; W],
            ends: InlineMap::Empty,
        }
    }
}

impl<'arena, const W: usize> MatchMemoEntry<'arena, W> {
    fn is_tried(&self, terminal: TerminalId) -> bool {
        let i = terminal.index();
        self.tried[i / 64] & (1u64 << (i % 64)) != 0
    }

    fn set_tried(&mut self, terminal: TerminalId) {
        let i = terminal.index();
        self.tried[i / 64] |= 1u64 << (i % 64);
    }

    fn is_matched(&self, terminal: TerminalId) -> bool {
        let i = terminal.index();
        self.matched[i / 64] & (1u64 << (i % 64)) != 0
    }

    fn set_matched(&mut self, terminal: TerminalId) {
        let i = terminal.index();
        self.matched[i / 64] |= 1u64 << (i % 64);
    }

    #[inline]
    fn lookup(&self, terminal: TerminalId) -> Option<Lookup> {
        if !self.is_tried(terminal) {
            return None;
        }
        if !self.is_matched(terminal) {
            return Some(Lookup::Fail);
        }
        let end = self
            .ends
            .get(&terminal)
            .expect("a matched terminal has an end");
        Some(Lookup::Match(*end))
    }
}

/// Memoization table for `match_token`, indexed by input position.
///
/// Terminal matches are pure functions of `(terminal, position)`, so each
/// pair is computed at most once and reused across the parse.
pub struct MatchMemo<'arena, const W: usize> {
    entries: ArenaVec<'arena, MatchMemoEntry<'arena, W>>,
}

impl<'arena, const W: usize> MatchMemo<'arena, W> {
    // Do not inline. The table is built once per parse, so the call costs
    // nothing. Inlined into a generated scanner's `new`, the fill loop copies
    // each whole default entry through the stack instead of writing only its
    // initialized fields, which doubles the parser's initialization time.
    #[inline(never)]
    pub fn new(input_len: usize, arena: &'arena Arena) -> Self {
        let mut entries = arena.vec_with_capacity(input_len + 1);
        entries.resize_with(input_len + 1, MatchMemoEntry::default);
        Self { entries }
    }

    // Inlined into the generated scanner's `match_token`, which calls it on
    // every memo lookup. Without the hint the compiler keeps it as a separate
    // function, and every lookup pays for the call.
    #[inline]
    pub fn get(&self, terminal: TerminalId, position: u32) -> Option<Lookup> {
        self.entries[position as usize].lookup(terminal)
    }

    pub fn insert_match(
        &mut self,
        terminal: TerminalId,
        position: u32,
        end: u32,
        arena: &'arena Arena,
    ) {
        let entry = &mut self.entries[position as usize];
        entry.set_tried(terminal);
        entry.set_matched(terminal);
        entry.ends.insert(terminal, end, arena);
    }

    pub fn insert_fail(&mut self, terminal: TerminalId, position: u32) {
        self.entries[position as usize].set_tried(terminal);
    }

    /// The `tried` and the `matched` bitsets at `position`, indexed by terminal
    /// id.
    #[inline]
    pub fn bits(&self, position: u32) -> (&[u64; W], &[u64; W]) {
        let entry = &self.entries[position as usize];
        (&entry.tried, &entry.matched)
    }
}

/// A terminal set the parser tests with `match_any` or `longest_match`.
///
/// `id` is a content-deduplicated id within the set's family. `match_any`
/// indexes its first-character sets and terminal masks by it; `longest_match`
/// ignores it, but the field is kept so every set has the same shape.
#[derive(Debug, PartialEq, Eq)]
pub struct TerminalSet {
    pub id: usize,
    pub terminals: &'static [TerminalId],
}

pub trait Scanner {
    fn match_token(&mut self, terminal_id: TerminalId, input_index: u32) -> Option<u32>;
    /// Returns the terminal in `set` that produces the longest match at
    /// `input_index`, or `None` if none match.
    fn longest_match(&mut self, set: &TerminalSet, input_index: u32) -> Option<TerminalId> {
        let mut terminal_id = None;
        let mut longest = 0;
        for &id in set.terminals {
            if let Some(end) = self.match_token(id, input_index) {
                if terminal_id.is_none() || end > longest {
                    terminal_id = Some(id);
                    longest = end;
                }
            }
        }
        terminal_id
    }
    /// The alternatives that `prediction` predicts at `input_index`: those
    /// predicted by a terminal that matches there. Empty when no terminal of
    /// `prediction` matches at `input_index`.
    #[inline]
    fn predict<A: AlternativeSet>(&mut self, prediction: &Prediction, input_index: u32) -> A {
        let mut alternatives = A::default();
        for &(terminal, alternative) in prediction.terminals_at(self.char_at(input_index)) {
            // An alternative already predicted does not need its terminal
            // matched again.
            let alternative = usize::from(alternative);
            if !alternatives.contains(alternative)
                && self.match_token(terminal, input_index).is_some()
            {
                alternatives.insert(alternative);
            }
        }
        alternatives
    }
    fn char_at(&self, i: u32) -> Option<char>;
    /// Runs `dfa` from `start`, returning the end position of the longest
    /// match, or `None` if no prefix at `start` is accepted. A longest match
    /// ending in an excluded state is rejected: an except of the DFA's
    /// terminal matches the same prefix.
    fn scan(&self, dfa: &Dfa, start: u32) -> Option<u32> {
        let mut state = dfa.start as usize;
        let mut position = start;
        let mut last_accept = None;
        let mut excluded = false;
        loop {
            let current = &dfa.states[state];
            if current.accept.is_some() {
                last_accept = Some(position);
                excluded = current.excluded;
            }
            let Some(ch) = self.char_at(position) else {
                break;
            };
            let Some(&(_, _, next)) = current
                .transitions
                .iter()
                .find(|(s, e, _)| ch >= *s && ch <= *e)
            else {
                break;
            };
            state = next as usize;
            position += 1;
        }
        if excluded { None } else { last_accept }
    }

    /// Whether `dfa` accepts exactly the span `[start, end)`: the walk
    /// consumes every character of the span and ends in an accept state.
    /// Unlike `scan` there is no maximal munch; nothing past `end` is read.
    fn scan_exact(&self, dfa: &Dfa, start: u32, end: u32) -> bool {
        let mut state = dfa.start as usize;
        for position in start..end {
            let Some(ch) = self.char_at(position) else {
                return false;
            };
            let Some(&(_, _, next)) = dfa.states[state]
                .transitions
                .iter()
                .find(|(s, e, _)| ch >= *s && ch <= *e)
            else {
                return false;
            };
            state = next as usize;
        }
        let last = &dfa.states[state];
        last.accept.is_some() && !last.excluded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_matches_and_failures() {
        let arena = Arena::new();
        let mut memo: MatchMemo<'_, 2> = MatchMemo::new(4, &arena);
        let (a, b, c) = (TerminalId(1), TerminalId(70), TerminalId(3));
        assert!(memo.get(a, 2).is_none());
        memo.insert_match(a, 2, 5, &arena);
        memo.insert_fail(b, 2);
        assert!(matches!(memo.get(a, 2), Some(Lookup::Match(5))));
        assert!(matches!(memo.get(b, 2), Some(Lookup::Fail)));
        assert!(memo.get(c, 2).is_none());
        assert!(memo.get(a, 1).is_none());
        let (tried, matched) = memo.bits(2);
        assert_eq!(*tried, [1 << 1, 1 << 6]);
        assert_eq!(*matched, [1 << 1, 0]);
    }

    #[test]
    fn finds_the_end_of_each_of_several_matches() {
        let arena = Arena::new();
        let mut memo: MatchMemo<'_, 1> = MatchMemo::new(1, &arena);
        for id in 0..5 {
            memo.insert_match(TerminalId(id), 0, u32::from(id) + 10, &arena);
        }
        for id in 0..5 {
            assert!(
                matches!(memo.get(TerminalId(id), 0), Some(Lookup::Match(end)) if end == u32::from(id) + 10)
            );
        }
    }
}
