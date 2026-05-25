use crate::ids::TerminalId;

pub struct State<'a> {
    pub transitions: &'a [(char, char, u32)],
    pub accept: Option<TerminalId>,
}

impl<'a> State<'a> {
    pub const fn new(transitions: &'a [(char, char, u32)], accept: Option<TerminalId>) -> Self {
        Self {
            transitions,
            accept,
        }
    }
}

pub struct Dfa<'a> {
    pub states: &'a [State<'a>],
    pub start: u32,
}

impl<'a> Dfa<'a> {
    pub const fn new(states: &'a [State<'a>]) -> Self {
        Self { states, start: 0 }
    }
}
