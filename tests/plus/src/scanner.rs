use iguana_runtime::{
    ids::TerminalId,
    input::Input,
    scanner::Scanner,
};
pub struct PlusScanner<'i> {
    pub input: &'i Input,
}
impl<'i> PlusScanner<'i> {
    pub fn new(input: &'i Input) -> Self {
        Self { input }
    }
    //"a"
    pub fn match_terminal_0(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, 'a')
    }
    //Layout
    pub fn match_terminal_1(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        Some(i)
    }
}
impl Scanner for PlusScanner<'_> {
    fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
        match terminal_id {
            TerminalId(0) => self.match_terminal_0(input_index),
            TerminalId(1) => self.match_terminal_1(input_index),
            _ => {
                unreachable!("Unknown token type: {terminal_id}");
            }
        }
    }
    fn char_at(&self, i: u32) -> Option<char> {
        self.input.char_at(i)
    }
}

