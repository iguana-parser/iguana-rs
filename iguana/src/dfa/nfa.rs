//! Thompson NFA construction from the `Regex` AST.
//!
//! `Nfa::from_regex` builds an NFA for a single regex tagged with one
//! `TerminalId`. For anything more complex (e.g., a union NFA over many
//! tagged regexes), use `NfaBuilder` directly.

use iguana_runtime::ids::TerminalId;

use crate::grammar::regex::{CharClass, CharRange, Regex};

pub type StateId = usize;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    pub epsilon_transitions: Vec<StateId>,
    pub transitions: Vec<(CharClass, StateId)>,
}

impl State {
    fn add_epsilon_transition(&mut self, to: StateId) {
        self.epsilon_transitions.push(to);
    }

    fn add_transition(&mut self, class: CharClass, to: StateId) {
        self.transitions.push((class, to));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nfa {
    pub states: Vec<State>,
    pub start: StateId,
    /// The order encodes priority: when subset construction finds a DFA state
    /// whose NFA-state set contains multiple accepts, the first match in this
    /// vector wins.
    pub accepts: Vec<(StateId, TerminalId)>,
}

impl Nfa {
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    pub fn from_regex(regex: &Regex, terminal_id: TerminalId) -> Nfa {
        let mut builder = NfaBuilder::new();
        let frag = builder.build(regex);
        builder.finish(frag.start, vec![(frag.accept, terminal_id)])
    }
}

/// A pair of `StateId`s: the entry and exit of a sub-NFA built by Thompson
/// construction. Both index into the surrounding `NfaBuilder`'s arena.
struct Fragment {
    start: StateId,
    accept: StateId,
}

struct NfaBuilder {
    states: Vec<State>,
}

impl NfaBuilder {
    fn new() -> Self {
        Self { states: Vec::new() }
    }

    fn new_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.push(State::default());
        id
    }

    fn build(&mut self, regex: &Regex) -> Fragment {
        match regex {
            Regex::Char(c) => {
                let start = self.new_state();
                let accept = self.new_state();
                self.states[start].add_transition(
                    CharClass {
                        ranges: vec![CharRange { start: *c, end: *c }],
                        negated: false,
                    },
                    accept,
                );
                Fragment { start, accept }
            }
            Regex::CharRange(r) => {
                let start = self.new_state();
                let accept = self.new_state();
                self.states[start].add_transition(
                    CharClass {
                        ranges: vec![*r],
                        negated: false,
                    },
                    accept,
                );
                Fragment { start, accept }
            }
            Regex::CharClass(cc) => {
                let start = self.new_state();
                let accept = self.new_state();
                self.states[start].add_transition(cc.clone(), accept);
                Fragment { start, accept }
            }
            Regex::Epsilon => {
                let state = self.new_state();
                Fragment {
                    start: state,
                    accept: state,
                }
            }
            Regex::Seq(parts) => {
                let mut iter = parts.iter();
                let Some(first) = iter.next() else {
                    let state = self.new_state();
                    return Fragment {
                        start: state,
                        accept: state,
                    };
                };
                let mut frag = self.build(first);
                for part in iter {
                    let next = self.build(part);
                    self.states[frag.accept].add_epsilon_transition(next.start);
                    frag.accept = next.accept;
                }
                frag
            }
            Regex::Alt(choices) => {
                let start = self.new_state();
                let accept = self.new_state();
                for choice in choices {
                    let frag = self.build(choice);
                    self.states[start].add_epsilon_transition(frag.start);
                    self.states[frag.accept].add_epsilon_transition(accept);
                }
                Fragment { start, accept }
            }
            Regex::Opt(inner) => {
                let start = self.new_state();
                let accept = self.new_state();
                let frag = self.build(inner);
                self.states[start].add_epsilon_transition(frag.start);
                self.states[frag.accept].add_epsilon_transition(accept);
                self.states[start].add_epsilon_transition(accept);
                Fragment { start, accept }
            }
            Regex::Star(inner) => {
                let start = self.new_state();
                let accept = self.new_state();
                let frag = self.build(inner);
                self.states[start].add_epsilon_transition(frag.start);
                self.states[start].add_epsilon_transition(accept);
                self.states[frag.accept].add_epsilon_transition(frag.start);
                self.states[frag.accept].add_epsilon_transition(accept);
                Fragment { start, accept }
            }
            Regex::Plus(inner) => {
                let frag = self.build(inner);
                let accept = self.new_state();
                self.states[frag.accept].add_epsilon_transition(frag.start);
                self.states[frag.accept].add_epsilon_transition(accept);
                Fragment {
                    start: frag.start,
                    accept,
                }
            }
            Regex::Identifier(id) => panic!(
                "Regex::Identifier({}) reached NFA construction; references must be inlined first",
                id.name
            ),
        }
    }

    fn finish(self, start: StateId, accepts: Vec<(StateId, TerminalId)>) -> Nfa {
        Nfa {
            states: self.states,
            start,
            accepts,
        }
    }
}

impl Default for NfaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: u16) -> TerminalId {
        TerminalId(id)
    }

    fn cc(c: char) -> CharClass {
        CharClass {
            ranges: vec![CharRange { start: c, end: c }],
            negated: false,
        }
    }

    /// Construct an NFA from a flat multi-map description. Each transition is
    /// `(from, label, to)`: `Some(class)` is a labeled transition, `None` is
    /// an ε-transition. Edges are inserted in list order, so within each
    /// `from` state the order is preserved.
    fn nfa_from(
        num_states: usize,
        start: StateId,
        accepts: &[(StateId, TerminalId)],
        transitions: &[(StateId, Option<CharClass>, StateId)],
    ) -> Nfa {
        let mut states: Vec<State> = (0..num_states).map(|_| State::default()).collect();
        for (from, label, to) in transitions {
            match label {
                Some(class) => states[*from].add_transition(class.clone(), *to),
                None => states[*from].add_epsilon_transition(*to),
            }
        }
        Nfa {
            states,
            start,
            accepts: accepts.to_vec(),
        }
    }

    #[test]
    fn char_atom() {
        let actual = Nfa::from_regex(&Regex::char('a'), t(0));
        let expected = nfa_from(2, 0, &[(1, t(0))], &[(0, Some(cc('a')), 1)]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn char_class_preserves_negation() {
        let class = CharClass {
            ranges: vec![CharRange {
                start: '\r',
                end: '\r',
            }],
            negated: true,
        };
        let actual = Nfa::from_regex(&Regex::CharClass(class.clone()), t(0));
        let expected = nfa_from(2, 0, &[(1, t(0))], &[(0, Some(class), 1)]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn epsilon_is_a_single_state() {
        let actual = Nfa::from_regex(&Regex::Epsilon, t(0));
        let expected = nfa_from(1, 0, &[(0, t(0))], &[]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn seq_chains_via_epsilon() {
        let actual = Nfa::from_regex(&Regex::seq(vec![Regex::char('a'), Regex::char('b')]), t(0));
        let expected = nfa_from(
            4,
            0,
            &[(3, t(0))],
            &[(0, Some(cc('a')), 1), (1, None, 2), (2, Some(cc('b')), 3)],
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn alt_branches_from_start_and_joins_at_accept() {
        let actual = Nfa::from_regex(&Regex::alt(vec![Regex::char('a'), Regex::char('b')]), t(0));
        let expected = nfa_from(
            6,
            0,
            &[(1, t(0))],
            &[
                (0, None, 2),
                (3, None, 1),
                (0, None, 4),
                (5, None, 1),
                (2, Some(cc('a')), 3),
                (4, Some(cc('b')), 5),
            ],
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn opt_adds_a_bypass_from_start_to_accept() {
        let actual = Nfa::from_regex(&Regex::Opt(Box::new(Regex::char('a'))), t(0));
        let expected = nfa_from(
            4,
            0,
            &[(1, t(0))],
            &[
                (0, None, 2),
                (3, None, 1),
                (0, None, 1),
                (2, Some(cc('a')), 3),
            ],
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn star_has_bypass_and_loop() {
        let actual = Nfa::from_regex(&Regex::star(Regex::char('a')), t(0));
        let expected = nfa_from(
            4,
            0,
            &[(1, t(0))],
            &[
                (0, None, 2),
                (0, None, 1),
                (3, None, 2),
                (3, None, 1),
                (2, Some(cc('a')), 3),
            ],
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn plus_has_loop_but_no_bypass() {
        let actual = Nfa::from_regex(&Regex::plus(Regex::char('a')), t(0));
        let expected = nfa_from(
            3,
            0,
            &[(2, t(0))],
            &[(1, None, 0), (1, None, 2), (0, Some(cc('a')), 1)],
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn union_branches_from_shared_start() {
        let r1 = Regex::char('a');
        let r2 = Regex::char('b');
        let mut builder = NfaBuilder::new();
        let start = builder.new_state();
        let mut accepts = Vec::new();
        for (regex, terminal_id) in [(&r1, t(0)), (&r2, t(1))] {
            let frag = builder.build(regex);
            builder.states[start].add_epsilon_transition(frag.start);
            accepts.push((frag.accept, terminal_id));
        }
        let actual = builder.finish(start, accepts);
        let expected = nfa_from(
            5,
            0,
            &[(2, t(0)), (4, t(1))],
            &[
                (0, None, 1),
                (0, None, 3),
                (1, Some(cc('a')), 2),
                (3, Some(cc('b')), 4),
            ],
        );
        assert_eq!(actual, expected);
    }
}
