use iguana_runtime::{
    ids::TerminalId,
    input::Input,
    scanner::Scanner,
    sppf::{Span, TerminalNode},
};
pub struct OptScanner<'i> {
    pub input: &'i Input,
}
impl<'i> OptScanner<'i> {
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
impl Scanner for OptScanner<'_> {
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
    fn match_leading_layout(&self, input_index: u32) -> (u32, Vec<TerminalNode>) {
        let mut i = input_index;
        let mut layout_nodes = vec![];
        while let Some((next_index, terminal_id)) = self.match_any(&vec![], i) {
            layout_nodes.push(TerminalNode::new(terminal_id, Span::new(i, next_index)));
            i = next_index;
        }
        (i, layout_nodes)
    }
    fn match_trailing_layout(&self, input_index: u32) -> (u32, Vec<TerminalNode>) {
        let mut i = input_index;
        let mut layout_nodes = vec![];
        while let Some((next_index, terminal_id)) = self.match_any(&vec![], i) {
            layout_nodes.push(TerminalNode::new(terminal_id, Span::new(i, next_index)));
            i = next_index;
            //If the last matched character is a newline, do not match further
            if let Some(last_matched_char) = self.input.char_at(next_index - 1)
                && last_matched_char == '\n'
            {
                break;
            }
        }
        (i, layout_nodes)
    }
}

