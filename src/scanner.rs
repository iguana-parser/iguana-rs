use crate::{parser::TerminalId, sppf::TerminalNode};

pub struct Token {
    pub token_type: TerminalId,
    pub range: Range,
}

pub struct Range {
    pub start: u32,
    pub end: u32,
}

pub trait Scanner {
    fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32>;
    fn match_any(
        &self,
        terminal_ids: &Vec<TerminalId>,
        input_index: u32,
    ) -> Option<(u32, TerminalId)> {
        for terminal_id in terminal_ids {
            if let Some(next_index) = self.match_token(*terminal_id, input_index) {
                return Some((next_index, *terminal_id));
            }
        }
        None
    }
    fn char_at(&self, i: u32) -> Option<char>;
    /// Matches layout definitions, usually whitespace and comments, before a token
    fn match_leading_layout(&self, input_index: u32) -> (u32, Vec<TerminalNode>);
    /// Matches layout definitions, usually whitespace and comments, before a token
    /// Stops matching if the last matched token ends with a newline ('\n') character.
    /// This is a heuristic where to attach each token.
    fn match_trailing_layout(&self, input_index: u32) -> (u32, Vec<TerminalNode>);
    fn match_char(&self, i: u32, c: char) -> Option<u32> {
        let ch = self.char_at(i)?;
        if ch == c { Some(i + 1) } else { None }
    }
    fn match_char_range(&self, i: u32, s: char, e: char) -> Option<u32> {
        let ch = self.char_at(i)?;
        if ch < s || ch > e { None } else { Some(i + 1) }
    }
}
