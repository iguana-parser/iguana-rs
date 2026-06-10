use crate::{dfa::Dfa, ids::TerminalId, utils::inline_map::InlineMap};

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

/// Per-position memo cell.
pub struct MatchMemoEntry<const W: usize> {
    tried: [u64; W],
    matched: InlineMap<TerminalId, u32>,
}

impl<const W: usize> Default for MatchMemoEntry<W> {
    fn default() -> Self {
        Self {
            tried: [0; W],
            matched: InlineMap::Empty,
        }
    }
}

impl<const W: usize> MatchMemoEntry<W> {
    fn is_tried(&self, terminal: TerminalId) -> bool {
        let i = terminal.index();
        self.tried[i / 64] & (1u64 << (i % 64)) != 0
    }

    fn set_tried(&mut self, terminal: TerminalId) {
        let i = terminal.index();
        self.tried[i / 64] |= 1u64 << (i % 64);
    }

    fn lookup(&self, terminal: TerminalId) -> Option<Lookup> {
        if !self.is_tried(terminal) {
            return None;
        }
        match self.matched.get(&terminal) {
            Some(&end) => Some(Lookup::Match(end)),
            None => Some(Lookup::Fail),
        }
    }
}

/// Memoization table for `match_token`, indexed by input position.
///
/// Terminal matches are pure functions of `(terminal, position)`, so each
/// pair is computed at most once and reused across the parse.
pub struct MatchMemo<const W: usize> {
    entries: Vec<MatchMemoEntry<W>>,
}

impl<const W: usize> MatchMemo<W> {
    pub fn new(input_len: usize) -> Self {
        let mut entries = Vec::with_capacity(input_len + 1);
        entries.resize_with(input_len + 1, MatchMemoEntry::default);
        Self { entries }
    }

    pub fn get(&self, terminal: TerminalId, position: u32) -> Option<Lookup> {
        self.entries[position as usize].lookup(terminal)
    }

    pub fn insert_match(&mut self, terminal: TerminalId, position: u32, end: u32) {
        let entry = &mut self.entries[position as usize];
        entry.set_tried(terminal);
        entry.matched.insert(terminal, end);
    }

    pub fn insert_fail(&mut self, terminal: TerminalId, position: u32) {
        self.entries[position as usize].set_tried(terminal);
    }
}

/// Memo cell for one input position. For each set id it records whether
/// `match_any` of that set has already run here, and if so whether it matched.
///
/// Those are three states: not run, ran and matched, ran and did not match.
/// Three states need two bits, so the cell holds two bitsets indexed by set id.
/// `computed` marks the set as queried at this position; `result` holds the
/// answer, read only once `computed` is set. A set's bit sits at word `id / 64`,
/// bit `id % 64`.
pub struct MatchAnyMemoEntry<const W: usize> {
    computed: [u64; W],
    result: [u64; W],
}

impl<const W: usize> Default for MatchAnyMemoEntry<W> {
    fn default() -> Self {
        Self {
            computed: [0; W],
            result: [0; W],
        }
    }
}

/// Memoization table for `match_any`, indexed by input position.
///
/// `match_any` over a terminal set is a pure function of `(set, position)`, so
/// each set is computed at most once per position and reused across the parse.
/// Set ids are assigned by content at code generation, so two sets with the
/// same terminals share a memo bit.
pub struct MatchAnyMemo<const W: usize> {
    entries: Vec<MatchAnyMemoEntry<W>>,
}

impl<const W: usize> MatchAnyMemo<W> {
    pub fn new(input_len: usize) -> Self {
        let mut entries = Vec::with_capacity(input_len + 1);
        entries.resize_with(input_len + 1, MatchAnyMemoEntry::default);
        Self { entries }
    }

    pub fn get(&self, set_id: usize, position: u32) -> Option<bool> {
        let entry = &self.entries[position as usize];
        let word = set_id / 64;
        let bit = 1u64 << (set_id % 64);
        if entry.computed[word] & bit == 0 {
            return None;
        }
        Some(entry.result[word] & bit != 0)
    }

    pub fn insert(&mut self, set_id: usize, position: u32, matched: bool) {
        let entry = &mut self.entries[position as usize];
        let word = set_id / 64;
        let bit = 1u64 << (set_id % 64);
        entry.computed[word] |= bit;
        if matched {
            entry.result[word] |= bit;
        }
    }
}

/// A terminal set the parser tests with `match_any` or `longest_match`.
///
/// `id` is a content-deduplicated id within the set's family. `match_any` keys
/// its memo by it; `longest_match` is not memoized and ignores it, but the field
/// is kept so every set has the same shape.
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
    fn char_at(&self, i: u32) -> Option<char>;
    /// Runs `dfa` from `start`, returning the end position of the longest
    /// match, or `None` if no prefix at `start` is accepted.
    fn scan(&self, dfa: &Dfa, start: u32) -> Option<u32> {
        let mut state = dfa.start as usize;
        let mut position = start;
        let mut last_accept = None;
        loop {
            if dfa.states[state].accept.is_some() {
                last_accept = Some(position);
            }
            let Some(ch) = self.char_at(position) else {
                break;
            };
            let Some(&(_, _, next)) = dfa.states[state]
                .transitions
                .iter()
                .find(|(s, e, _)| ch >= *s && ch <= *e)
            else {
                break;
            };
            state = next as usize;
            position += 1;
        }
        last_accept
    }
}
