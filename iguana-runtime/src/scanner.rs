use crate::{
    arena::{Arena, ArenaVec},
    dfa::Dfa,
    ids::TerminalId,
    prediction::{AlternativeSet, Prediction},
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

/// The number of match ends a memo entry keeps inline. A match end is the
/// input index where a terminal's match ends. Different terminals can match
/// at the same input index, but our data shows that more than three rarely
/// do.
const INLINE_ENDS: usize = 3;

/// The number of slots a position reserves in `MatchMemo::spilled_ends` for
/// its 4th match, enough for its 4th to 7th matches. If more than 7 terminals
/// match at the same input index, a segment twice as long is allocated for
/// them at the end of the buffer.
const FIRST_SEGMENT_LEN: usize = 4;

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
/// - `ends` holds the ends of the first `INLINE_ENDS` matches, and
///   `end_terminals` the terminal of each.
/// - `spill_start` is the offset in `MatchMemo::spilled_ends` where the
///   segment that holds the ends of further matches starts.
/// - `end_count` is the number of matches.
///
/// A terminal's bit sits at word `id / 64`, bit `id % 64`.
#[derive(Clone, Copy)]
pub struct MatchMemoEntry<const W: usize> {
    tried: [u64; W],
    matched: [u64; W],
    ends: [u32; INLINE_ENDS],
    end_terminals: [TerminalId; INLINE_ENDS],
    spill_start: u32,
    end_count: u16,
}

impl<const W: usize> Default for MatchMemoEntry<W> {
    fn default() -> Self {
        Self {
            tried: [0; W],
            matched: [0; W],
            ends: [0; INLINE_ENDS],
            end_terminals: [TerminalId(0); INLINE_ENDS],
            spill_start: 0,
            end_count: 0,
        }
    }
}

impl<const W: usize> MatchMemoEntry<W> {
    fn set_tried(&mut self, terminal: TerminalId) {
        let i = terminal.index();
        self.tried[i / 64] |= 1u64 << (i % 64);
    }

    fn set_matched(&mut self, terminal: TerminalId) {
        let i = terminal.index();
        self.matched[i / 64] |= 1u64 << (i % 64);
    }

    #[inline]
    fn lookup(&self, terminal: TerminalId, spilled_ends: &[(TerminalId, u32)]) -> Option<Lookup> {
        let (word, bit) = (terminal.index() / 64, terminal.index() % 64);
        let tried = (self.tried[word] >> bit) & 1;
        let matched = (self.matched[word] >> bit) & 1;
        match (tried << 1) | matched {
            0b00 => None,
            0b10 => Some(Lookup::Fail),
            _ => {
                let inline = usize::from(self.end_count).min(INLINE_ENDS);
                let end = match self.end_terminals[..inline]
                    .iter()
                    .position(|&t| t == terminal)
                {
                    Some(k) => self.ends[k],
                    None => self.spilled_end(terminal, spilled_ends),
                };
                Some(Lookup::Match(end))
            }
        }
    }

    fn insert_match(
        &mut self,
        terminal: TerminalId,
        end: u32,
        spilled_ends: &mut ArenaVec<'_, (TerminalId, u32)>,
    ) {
        self.set_tried(terminal);
        self.set_matched(terminal);
        let k = usize::from(self.end_count);
        if k < INLINE_ENDS {
            self.end_terminals[k] = terminal;
            self.ends[k] = end;
        } else {
            self.spill_end(terminal, end, spilled_ends);
        }
        self.end_count += 1;
    }

    // A match after the first `INLINE_ENDS` at a position is rare. Its paths
    // are kept out of `lookup` and `insert_match`, which are inlined into the
    // hot scanner methods.
    #[cold]
    #[inline(never)]
    fn spilled_end(&self, terminal: TerminalId, spilled_ends: &[(TerminalId, u32)]) -> u32 {
        let start = self.spill_start as usize;
        let count = usize::from(self.end_count) - INLINE_ENDS;
        spilled_ends[start..start + count]
            .iter()
            .find(|&&(t, _)| t == terminal)
            .map(|&(_, end)| end)
            .expect("a matched terminal has an end")
    }

    #[cold]
    #[inline(never)]
    fn spill_end(
        &mut self,
        terminal: TerminalId,
        end: u32,
        spilled_ends: &mut ArenaVec<'_, (TerminalId, u32)>,
    ) {
        let start = self.spill_start as usize;
        let count = usize::from(self.end_count) - INLINE_ENDS;
        // The first spilled end reserves a new segment. A full segment moves to
        // a new segment twice as long, and the old segment stays unused.
        if count == 0 || (count >= FIRST_SEGMENT_LEN && count.is_power_of_two()) {
            let new_start = spilled_ends.len();
            spilled_ends.extend_from_within(start..start + count);
            spilled_ends.resize(
                new_start + (2 * count).max(FIRST_SEGMENT_LEN),
                (TerminalId(0), 0),
            );
            self.spill_start = u32::try_from(new_start).expect("spilled ends fit in u32");
        }
        spilled_ends[self.spill_start as usize + count] = (terminal, end);
    }
}

/// Memoization table for `match_token`, indexed by input position.
///
/// Terminal matches are pure functions of `(terminal, position)`, so each
/// pair is computed at most once, and both matches and failures are reused
/// across the parse.
///
/// The ends of matches after the first `INLINE_ENDS` at a position are in
/// `spilled_ends`, in a segment of consecutive slots that the position's entry
/// points to. The first segment has `FIRST_SEGMENT_LEN` slots. When a segment
/// is full, its ends move to a new segment twice as long at the end of
/// `spilled_ends`.
///
/// `W` is the number of 64-bit words of each bitset. `W * 64` must be greater
/// than every terminal id, including that of the end-of-input terminal.
pub struct MatchMemo<'arena, const W: usize> {
    entries: ArenaVec<'arena, MatchMemoEntry<W>>,
    spilled_ends: ArenaVec<'arena, (TerminalId, u32)>,
}

impl<'arena, const W: usize> MatchMemo<'arena, W> {
    // Do not inline. The table is built once per parse, so the call costs
    // nothing. Inlined into a generated scanner's `new`, the fill loop copies
    // each whole default entry through the stack instead of writing only its
    // initialized fields, which doubles the parser's initialization time.
    #[inline(never)]
    pub fn new(input_len: usize, arena: &'arena Arena) -> Self {
        let mut entries = arena.vec_with_capacity(input_len + 1);
        entries.resize(input_len + 1, MatchMemoEntry::default());
        Self {
            entries,
            spilled_ends: arena.vec(),
        }
    }

    /// The recorded result of `terminal` at `position`: `None` if it was not
    /// tried there yet, `Some(Lookup::Fail)` if it failed, and
    /// `Some(Lookup::Match(end))` if it matched.
    // Inlined into the generated scanner's `match_token`, which calls it on
    // every memo lookup. Without the hint the compiler keeps it as a separate
    // function, and every lookup pays for the call.
    #[inline]
    pub fn get(&self, terminal: TerminalId, position: u32) -> Option<Lookup> {
        self.entries[position as usize].lookup(terminal, &self.spilled_ends)
    }

    /// Records that `terminal` matched at `position`, ending at the exclusive
    /// position `end`. The memo must not already have a result for `terminal`
    /// at `position`.
    pub fn insert_match(&mut self, terminal: TerminalId, position: u32, end: u32) {
        self.entries[position as usize].insert_match(terminal, end, &mut self.spilled_ends);
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
    /// `prediction` matches at `input_index`. A generated scanner with a memo
    /// overrides this method to answer from the memo.
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
        memo.insert_match(a, 2, 5);
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
        // More matches than an entry holds, so the last two go to
        // `spilled_ends`.
        for id in 0..5 {
            memo.insert_match(TerminalId(id), 0, u32::from(id) + 10);
        }
        for id in 0..5 {
            assert!(
                matches!(memo.get(TerminalId(id), 0), Some(Lookup::Match(end)) if end == u32::from(id) + 10)
            );
        }
    }

    #[test]
    fn records_the_largest_terminal_id() {
        let arena = Arena::new();
        let mut memo: MatchMemo<'_, 1024> = MatchMemo::new(0, &arena);
        let (last, first) = (TerminalId(u16::MAX), TerminalId(0));
        memo.insert_match(last, 0, 3);
        memo.insert_match(first, 0, 5);
        assert!(matches!(memo.get(last, 0), Some(Lookup::Match(3))));
        assert!(matches!(memo.get(first, 0), Some(Lookup::Match(5))));
    }

    #[test]
    fn growing_segments_keep_the_ends_of_other_positions() {
        let arena = Arena::new();
        let mut memo: MatchMemo<'_, 1> = MatchMemo::new(2, &arena);
        // Both positions spill, and their segments grow in turn, from 4 slots
        // to 8 and 16.
        for id in 0..20 {
            memo.insert_match(TerminalId(id), 0, u32::from(id) + 10);
            memo.insert_match(TerminalId(id), 1, u32::from(id) + 100);
        }
        for id in 0..20 {
            assert!(
                matches!(memo.get(TerminalId(id), 0), Some(Lookup::Match(end)) if end == u32::from(id) + 10)
            );
            assert!(
                matches!(memo.get(TerminalId(id), 1), Some(Lookup::Match(end)) if end == u32::from(id) + 100)
            );
        }
    }

    #[test]
    fn the_spill_start_does_not_enlarge_an_entry() {
        const { assert!(size_of::<MatchMemoEntry<1>>() == 2 * 8 + 24) }
    }
}
