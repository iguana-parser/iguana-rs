use crate::{ids::TerminalId, utils::inline_map::InlineMap};

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

pub trait Scanner {
    fn match_token(&mut self, terminal_id: TerminalId, input_index: u32) -> Option<u32>;
    fn match_any(&mut self, terminal_ids: &[TerminalId], input_index: u32) -> bool {
        terminal_ids
            .iter()
            .any(|id| self.match_token(*id, input_index).is_some())
    }
    /// Returns the terminal that produces the longest match at `input_index`,
    /// or `None` if none of the given terminals match.
    fn longest_match(
        &mut self,
        terminal_ids: &[TerminalId],
        input_index: u32,
    ) -> Option<TerminalId> {
        let mut terminal_id = None;
        let mut longest_match = 0;
        for &id in terminal_ids {
            if let Some(end) = self.match_token(id, input_index) {
                if terminal_id.is_none() || end > longest_match {
                    terminal_id = Some(id);
                    longest_match = end;
                }
            }
        }
        terminal_id
    }
    fn char_at(&self, i: u32) -> Option<char>;
    fn match_char(&self, i: u32, c: char) -> Option<u32> {
        let ch = self.char_at(i)?;
        if ch == c { Some(i + 1) } else { None }
    }
    fn match_char_range(&self, i: u32, s: char, e: char) -> Option<u32> {
        let ch = self.char_at(i)?;
        if ch < s || ch > e { None } else { Some(i + 1) }
    }
    fn match_char_class(&self, i: u32, ranges: &[(char, char)], negated: bool) -> Option<u32> {
        let ch = self.char_at(i)?;
        let in_range = ranges.iter().any(|(s, e)| ch >= *s && ch <= *e);
        if negated ^ in_range {
            Some(i + 1)
        } else {
            None
        }
    }
}
