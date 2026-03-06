use iguana_runtime::{
    ids::TerminalId,
    input::Input,
    scanner::Scanner,
    sppf::{Span, TerminalNode},
};
const CHAR_CLASS_0: [(char, char); 2usize] = [('a', 'z'), ('A', 'Z')];
pub struct ExceptNonterminalScanner<'i> {
    pub input: &'i Input,
}
impl<'i> ExceptNonterminalScanner<'i> {
    pub fn new(input: &'i Input) -> Self {
        Self { input }
    }
    //Identifier = ([a-z A-Z]+)
    pub fn match_terminal_0(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        let i = (|i| self.match_char_class(i, &CHAR_CLASS_0, false))(i)?;
        let mut j = i;
        while let Some(k) = (|i| self.match_char_class(i, &CHAR_CLASS_0, false))(j) {
            j = k;
        }
        Some(j)
    }
    //Keyword = (if|else|while)
    pub fn match_terminal_1(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, 'i')
            .and_then(|i| self.match_char(i, 'f'))
            .or_else(|| {
                self.match_char(i, 'e')
                    .and_then(|i| self.match_char(i, 'l'))
                    .and_then(|i| self.match_char(i, 's'))
                    .and_then(|i| self.match_char(i, 'e'))
            })
            .or_else(|| {
                self.match_char(i, 'w')
                    .and_then(|i| self.match_char(i, 'h'))
                    .and_then(|i| self.match_char(i, 'i'))
                    .and_then(|i| self.match_char(i, 'l'))
                    .and_then(|i| self.match_char(i, 'e'))
            })
    }
    //Layout = ε
    pub fn match_terminal_2(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        Some(i)
    }
}
impl Scanner for ExceptNonterminalScanner<'_> {
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

