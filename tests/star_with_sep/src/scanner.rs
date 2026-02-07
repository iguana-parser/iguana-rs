use iguana_runtime::{
    ids::TerminalId,
    input::Input,
    scanner::Scanner,
    sppf::{Span, TerminalNode},
};
pub struct StarWithSepScanner<'i> {
    pub input: &'i Input,
}
impl<'i> StarWithSepScanner<'i> {
    pub fn new(input: &'i Input) -> Self {
        Self { input }
    }
    //","
    pub fn match_terminal_0(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, ',')
    }
    //"a"
    pub fn match_terminal_1(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, 'a')
    }
    //Layout
    pub fn match_terminal_2(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        Some(i)
    }
}
impl Scanner for StarWithSepScanner<'_> {
    fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
        match terminal_id {
            TerminalId(0) => self.match_terminal_0(input_index),
            TerminalId(1) => self.match_terminal_1(input_index),
            TerminalId(2) => self.match_terminal_2(input_index),
            _ => {
                unreachable!("Unknown token type: {terminal_id}");
            }
        }
    }
    fn char_at(&self, i: u32) -> Option<char> {
        self.input.char_at(i)
    }
}

