use crate::ids::TerminalId;

pub struct State<'a> {
    pub transitions: &'a [(char, char, u32)],
    pub accept: Option<TerminalId>,
    /// Whether this state is an accept state of one of the terminal's excepts.
    pub excluded: bool,
}

impl<'a> State<'a> {
    pub const fn new(transitions: &'a [(char, char, u32)], accept: Option<TerminalId>) -> Self {
        Self {
            transitions,
            accept,
            excluded: false,
        }
    }

    pub const fn new_excluded(
        transitions: &'a [(char, char, u32)],
        accept: Option<TerminalId>,
    ) -> Self {
        Self {
            transitions,
            accept,
            excluded: true,
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
