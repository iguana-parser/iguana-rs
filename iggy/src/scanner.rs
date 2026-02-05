use iguana_runtime::{
    ids::TerminalId,
    input::Input,
    scanner::Scanner,
    sppf::{Span, TerminalNode},
};
const CHAR_CLASS_0: [(char, char); 9usize] = [
    ('\\', '\\'),
    ('-', '-'),
    ('[', '['),
    (']', ']'),
    ('\t', '\t'),
    ('f', 'f'),
    ('\r', '\r'),
    ('\n', '\n'),
    (' ', ' '),
];
const CHAR_CLASS_1: [(char, char); 9usize] = [
    ('\\', '\\'),
    ('-', '-'),
    ('[', '['),
    (']', ']'),
    ('t', 't'),
    ('f', 'f'),
    ('r', 'r'),
    ('n', 'n'),
    (' ', ' '),
];
const CHAR_CLASS_2: [(char, char); 7usize] = [
    ('\'', '\''),
    ('"', '"'),
    ('\\', '\\'),
    ('t', 't'),
    ('f', 'f'),
    ('r', 'r'),
    ('n', 'n'),
];
const CHAR_CLASS_3: [(char, char); 3usize] = [('\'', '\''), ('"', '"'), ('\\', '\\')];
const CHAR_CLASS_4: [(char, char); 3usize] = [('a', 'z'), ('A', 'Z'), ('_', '_')];
const CHAR_CLASS_5: [(char, char); 4usize] = [('a', 'z'), ('A', 'Z'), ('_', '_'), ('0', '9')];
const CHAR_CLASS_6: [(char, char); 2usize] = [(' ', ' '), ('\n', '\n')];
pub struct IggyScanner<'i> {
    pub input: &'i Input,
}
impl<'i> IggyScanner<'i> {
    pub fn new(input: &'i Input) -> Self {
        Self { input }
    }
    //RangeChar
    pub fn match_terminal_0(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char_class(i, &CHAR_CLASS_0, true).or_else(|| {
            self.match_char(i, '\\')
                .and_then(|i| self.match_char_class(i, &CHAR_CLASS_1, false))
        })
    }
    //Char
    pub fn match_terminal_1(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '\\')
            .and_then(|i| self.match_char_class(i, &CHAR_CLASS_2, false))
            .or_else(|| self.match_char_class(i, &CHAR_CLASS_3, true))
    }
    //String
    pub fn match_terminal_2(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        let mut j = i;
        while let Some(k) = (|i| {
            self.match_char(i, '\\')
                .and_then(|i| self.match_char_class(i, &CHAR_CLASS_2, false))
                .or_else(|| self.match_char_class(i, &CHAR_CLASS_3, true))
        })(j)
        {
            j = k;
        }
        Some(j)
    }
    //Identifier
    pub fn match_terminal_3(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char_class(i, &CHAR_CLASS_4, false)
            .and_then(|i| {
                let mut j = i;
                while let Some(k) = (|i| self.match_char_class(i, &CHAR_CLASS_5, false))(j) {
                    j = k;
                }
                Some(j)
            })
    }
    //Label
    pub fn match_terminal_4(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '@')
            .and_then(|i| self.match_char_class(i, &CHAR_CLASS_4, false))
            .and_then(|i| {
                let mut j = i;
                while let Some(k) = (|i| self.match_char_class(i, &CHAR_CLASS_5, false))(j) {
                    j = k;
                }
                Some(j)
            })
    }
    //WS
    pub fn match_terminal_5(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        let mut j = i;
        while let Some(k) = (|i| self.match_char_class(i, &CHAR_CLASS_6, false))(j) {
            j = k;
        }
        Some(j)
    }
    //"grammar"
    pub fn match_terminal_6(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, 'g')
            .and_then(|i| self.match_char(i, 'r'))
            .and_then(|i| self.match_char(i, 'a'))
            .and_then(|i| self.match_char(i, 'm'))
            .and_then(|i| self.match_char(i, 'm'))
            .and_then(|i| self.match_char(i, 'a'))
            .and_then(|i| self.match_char(i, 'r'))
    }
    //"layout"
    pub fn match_terminal_7(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, 'l')
            .and_then(|i| self.match_char(i, 'a'))
            .and_then(|i| self.match_char(i, 'y'))
            .and_then(|i| self.match_char(i, 'o'))
            .and_then(|i| self.match_char(i, 'u'))
            .and_then(|i| self.match_char(i, 't'))
    }
    //"="
    pub fn match_terminal_8(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '=')
    }
    //">"
    pub fn match_terminal_9(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '>')
    }
    //"|"
    pub fn match_terminal_10(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '|')
    }
    //"*"
    pub fn match_terminal_11(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '*')
    }
    //"+"
    pub fn match_terminal_12(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '+')
    }
    //"?"
    pub fn match_terminal_13(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '?')
    }
    //"("
    pub fn match_terminal_14(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '(')
    }
    //")"
    pub fn match_terminal_15(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, ')')
    }
    //"\""
    pub fn match_terminal_16(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '\\')
            .and_then(|i| self.match_char(i, '"'))
    }
    //"{"
    pub fn match_terminal_17(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '{')
    }
    //"}"
    pub fn match_terminal_18(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '}')
    }
    //":"
    pub fn match_terminal_19(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, ':')
    }
    //"regex"
    pub fn match_terminal_20(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, 'r')
            .and_then(|i| self.match_char(i, 'e'))
            .and_then(|i| self.match_char(i, 'g'))
            .and_then(|i| self.match_char(i, 'e'))
            .and_then(|i| self.match_char(i, 'x'))
    }
    //"!"
    pub fn match_terminal_21(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '!')
    }
    //"["
    pub fn match_terminal_22(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '[')
    }
    //"]"
    pub fn match_terminal_23(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, ']')
    }
    //"-"
    pub fn match_terminal_24(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        self.match_char(i, '-')
    }
    //Layout
    pub fn match_terminal_25(&self, input_index: u32) -> Option<u32> {
        let i = input_index;
        let mut j = i;
        while let Some(k) = (|i| self.match_char_class(i, &CHAR_CLASS_6, false))(j) {
            j = k;
        }
        Some(j)
    }
}
impl Scanner for IggyScanner<'_> {
    fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
        match terminal_id {
            TerminalId(0) => self.match_terminal_0(input_index),
            TerminalId(1) => self.match_terminal_1(input_index),
            TerminalId(2) => self.match_terminal_2(input_index),
            TerminalId(3) => self.match_terminal_3(input_index),
            TerminalId(4) => self.match_terminal_4(input_index),
            TerminalId(5) => self.match_terminal_5(input_index),
            TerminalId(6) => self.match_terminal_6(input_index),
            TerminalId(7) => self.match_terminal_7(input_index),
            TerminalId(8) => self.match_terminal_8(input_index),
            TerminalId(9) => self.match_terminal_9(input_index),
            TerminalId(10) => self.match_terminal_10(input_index),
            TerminalId(11) => self.match_terminal_11(input_index),
            TerminalId(12) => self.match_terminal_12(input_index),
            TerminalId(13) => self.match_terminal_13(input_index),
            TerminalId(14) => self.match_terminal_14(input_index),
            TerminalId(15) => self.match_terminal_15(input_index),
            TerminalId(16) => self.match_terminal_16(input_index),
            TerminalId(17) => self.match_terminal_17(input_index),
            TerminalId(18) => self.match_terminal_18(input_index),
            TerminalId(19) => self.match_terminal_19(input_index),
            TerminalId(20) => self.match_terminal_20(input_index),
            TerminalId(21) => self.match_terminal_21(input_index),
            TerminalId(22) => self.match_terminal_22(input_index),
            TerminalId(23) => self.match_terminal_23(input_index),
            TerminalId(24) => self.match_terminal_24(input_index),
            TerminalId(25) => self.match_terminal_25(input_index),
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
        while let Some((next_index, terminal_id)) = self.match_any(&vec![TerminalId(5)], i) {
            layout_nodes.push(TerminalNode::new(terminal_id, Span::new(i, next_index)));
            i = next_index;
        }
        (i, layout_nodes)
    }
    fn match_trailing_layout(&self, input_index: u32) -> (u32, Vec<TerminalNode>) {
        let mut i = input_index;
        let mut layout_nodes = vec![];
        while let Some((next_index, terminal_id)) = self.match_any(&vec![TerminalId(5)], i) {
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

