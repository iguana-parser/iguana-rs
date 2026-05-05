use crate::ids::TerminalId;

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
    fn match_any(&self, terminal_ids: &[TerminalId], input_index: u32) -> bool {
        terminal_ids
            .iter()
            .any(|id| self.match_token(*id, input_index).is_some())
    }
    /// Returns the terminal that produces the longest match at `input_index`,
    /// or `None` if none of the given terminals match.
    fn longest_match(
        &self,
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
