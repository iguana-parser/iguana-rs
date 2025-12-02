use crate::parser::TerminalId;

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
}
