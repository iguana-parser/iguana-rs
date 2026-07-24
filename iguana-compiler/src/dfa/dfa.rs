use std::collections::VecDeque;

use iguana_runtime::ids::TerminalId;
use rustc_hash::{FxHashMap, FxHashSet};

use super::nfa::{self, Nfa};
use crate::grammar::regex::{CharClass, CharRange};

pub type StateId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub transitions: Vec<(CharRange, StateId)>,
    pub accept: Option<TerminalId>,
    /// Whether this state contains the accept state of an except.
    pub excluded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dfa {
    pub states: Vec<State>,
    pub start: StateId,
}

impl Dfa {
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    pub fn from_nfa(nfa: &Nfa) -> Dfa {
        DfaBuilder::new(nfa).build()
    }
}

/// Complement of `ranges` over the Unicode scalar value space
/// (`\0`..=`char::MAX`). Input may overlap; output is sorted and disjoint.
fn complement(ranges: &[CharRange]) -> Vec<CharRange> {
    let mut covered: Vec<(u32, u32)> = ranges
        .iter()
        .map(|r| (r.start as u32, r.end as u32))
        .collect();
    // The surrogate range U+D800..=U+DFFF has no `char` representation.
    // Inject it as a fake-covered interval so the sweep skips over it,
    // splitting the output into two segments around the hole instead of
    // one that crosses it.
    covered.push((0xD800, 0xDFFF));
    covered.sort_by_key(|&(start, _)| start);

    let mut result = Vec::new();
    let mut cursor: u32 = 0;
    for (start, end) in covered {
        if cursor < start {
            result.push(CharRange {
                start: char::from_u32(cursor).unwrap(),
                end: char::from_u32(start - 1).unwrap(),
            });
        }
        if end + 1 > cursor {
            cursor = end + 1;
        }
    }
    if cursor <= char::MAX as u32 {
        result.push(CharRange {
            start: char::from_u32(cursor).unwrap(),
            end: char::MAX,
        });
    }
    result
}

/// Partition `ranges` (which may overlap) into a sorted list of disjoint
/// sub-ranges whose union is the same set of characters.
fn to_non_overlapping(ranges: &[CharRange]) -> Vec<CharRange> {
    if ranges.is_empty() {
        return Vec::new();
    }
    // Each range contributes +1 at its start (coverage opens) and -1 at
    // end + 1 (coverage closes).
    let mut events: Vec<(u32, i32)> = Vec::with_capacity(ranges.len() * 2);
    for r in ranges {
        events.push((r.start as u32, 1));
        events.push((r.end as u32 + 1, -1));
    }
    events.sort_by_key(|&(pos, _)| pos);

    let mut result = Vec::new();
    let mut counter: i32 = 0;
    let mut prev: u32 = 0;
    let mut i = 0;
    while i < events.len() {
        let pos = events[i].0;
        // `counter` is the coverage over [prev, pos), the interval that just ended.
        if counter > 0 {
            // It was fully covered, so emit it as the inclusive range.
            result.push(CharRange {
                start: char::from_u32(prev).unwrap(),
                end: char::from_u32(pos - 1).unwrap(),
            });
        }
        // Apply every delta at this position before stepping: adjacent ranges
        // meet at the same `pos` and must collapse to one boundary.
        while i < events.len() && events[i].0 == pos {
            counter += events[i].1;
            i += 1;
        }
        // `counter` now describes the next interval [pos, next_event); anchor it.
        prev = pos;
    }
    result
}

/// Epsilon-closure of `seeds` in `nfa`: every state reachable from a seed by
/// following zero or more epsilon transitions. The result is sorted, so it
/// can be used directly as a hash-map key during subset construction.
fn epsilon_closure(nfa: &Nfa, seeds: impl IntoIterator<Item = nfa::StateId>) -> Vec<nfa::StateId> {
    let mut in_closure = vec![false; nfa.num_states()];
    let mut stack: Vec<nfa::StateId> = Vec::new();
    for s in seeds {
        if !in_closure[s] {
            in_closure[s] = true;
            stack.push(s);
        }
    }
    while let Some(s) = stack.pop() {
        for &t in &nfa.states[s].epsilon_transitions {
            if !in_closure[t] {
                in_closure[t] = true;
                stack.push(t);
            }
        }
    }
    (0..nfa.num_states()).filter(|&i| in_closure[i]).collect()
}

/// `CharRange`s matched by `class`, with the negation flag applied: negated
/// classes are flipped via `complement`, non-negated ones return their ranges
/// unchanged.
fn to_char_ranges(class: &CharClass) -> Vec<CharRange> {
    if class.negated {
        complement(&class.ranges)
    } else {
        class.ranges.clone()
    }
}

struct DfaBuilder<'a> {
    nfa: &'a Nfa,
    /// Whether some state in `nfa.accepts` is reachable from each NFA state.
    /// Target subsets with no live state are not materialized; `live_states`
    /// explains why.
    live: Vec<bool>,
    states: Vec<State>,
    state_ids: FxHashMap<Vec<nfa::StateId>, StateId>,
    worklist: VecDeque<(StateId, Vec<nfa::StateId>)>,
}

impl<'a> DfaBuilder<'a> {
    fn new(nfa: &'a Nfa) -> Self {
        Self {
            nfa,
            live: live_states(nfa),
            states: Vec::new(),
            state_ids: FxHashMap::default(),
            worklist: VecDeque::new(),
        }
    }

    fn build(mut self) -> Dfa {
        let start_set = epsilon_closure(self.nfa, [self.nfa.start]);
        let start = self.get_or_create_state(start_set);
        while let Some((dfa_id, nfa_set)) = self.worklist.pop_front() {
            self.process(dfa_id, &nfa_set);
        }
        Dfa {
            states: self.states,
            start,
        }
    }

    /// Returns the DFA state id for `nfa_set`, creating it (and queueing it
    /// for processing) the first time the set is seen.
    fn get_or_create_state(&mut self, nfa_set: Vec<nfa::StateId>) -> StateId {
        if let Some(&id) = self.state_ids.get(&nfa_set) {
            return id;
        }
        let accept = accept_of(self.nfa, &nfa_set);
        let excluded = excluded_of(self.nfa, &nfa_set);
        let id = self.states.len();
        self.states.push(State {
            transitions: Vec::new(),
            accept,
            excluded,
        });
        self.state_ids.insert(nfa_set.clone(), id);
        self.worklist.push_back((id, nfa_set));
        id
    }

    fn process(&mut self, dfa_id: StateId, nfa_set: &[nfa::StateId]) {
        // Expand each outgoing class once: this list is reused below for the
        // alphabet partition and for the per-atom coverage check.
        let outgoing: Vec<(Vec<CharRange>, nfa::StateId)> = nfa_set
            .iter()
            .flat_map(|&s| self.nfa.states[s].transitions.iter())
            .map(|(class, t)| (to_char_ranges(class), *t))
            .collect();
        let all_ranges: Vec<CharRange> = outgoing
            .iter()
            .flat_map(|(ranges, _)| ranges.iter().copied())
            .collect();
        // Atoms are sorted and disjoint, so transitions emitted in atom order
        // produce a sorted `transitions` vector for free.
        let atoms = to_non_overlapping(&all_ranges);
        for atom in atoms {
            let targets: Vec<nfa::StateId> = outgoing
                .iter()
                .filter(|(ranges, _)| {
                    ranges
                        .iter()
                        .any(|r| r.start <= atom.start && atom.end <= r.end)
                })
                .map(|(_, t)| *t)
                .collect();
            let next_set = epsilon_closure(self.nfa, targets);
            if !next_set.iter().any(|&s| self.live[s]) {
                continue;
            }
            let next_id = self.get_or_create_state(next_set);
            self.states[dfa_id].transitions.push((atom, next_id));
        }
    }
}

/// First `TerminalId` in `nfa.accepts` whose state appears in `nfa_set`.
/// Declaration order in `nfa.accepts` is the tie-break: earlier-registered
/// terminals win when one DFA state would accept multiple terminals.
fn accept_of(nfa: &Nfa, nfa_set: &[nfa::StateId]) -> Option<TerminalId> {
    nfa.accepts
        .iter()
        .find(|(s, _)| nfa_set.binary_search(s).is_ok())
        .map(|(_, t)| *t)
}

/// Whether `nfa_set` contains an except accept state.
fn excluded_of(nfa: &Nfa, nfa_set: &[nfa::StateId]) -> bool {
    nfa.except_accepts
        .iter()
        .any(|s| nfa_set.binary_search(s).is_ok())
}

/// Whether some state in `nfa.accepts` is reachable from each NFA state.
/// The builder refuses to materialize subsets containing no live state,
/// which keeps the scan bounded by the terminal instead of by its excepts.
///
/// Consider `[a-z][a-z]? \ ("if" | "iffy")` on input `iffy`. After `if` the
/// terminal cannot extend, but the except fragment for `iffy` still has
/// transitions on `f` and `y`. Walking them is wasted work: a match can only
/// end at a terminal accept, and no terminal accept is reachable anymore.
/// However, subset construction alone would still build those states. It
/// guarantees that every DFA state is reachable from the start, not that an
/// accept is reachable from every DFA state. For a single regex the two
/// coincide, since every state of a Thompson NFA lies on a path to its
/// accept; the except union breaks the coincidence, because except accepts
/// are labels rather than accepts, so an except fragment can outlive the
/// terminal fragment. Dropping the states that cannot reach an accept is
/// the textbook trim operation, fused into the construction.
///
/// The computation is backward reachability: flood from the accept states
/// over reversed edges. Except accepts are not seeds; reaching one does not
/// make a state live.
fn live_states(nfa: &Nfa) -> Vec<bool> {
    let mut reverse: Vec<Vec<nfa::StateId>> = vec![Vec::new(); nfa.num_states()];
    for (id, state) in nfa.states.iter().enumerate() {
        for &target in &state.epsilon_transitions {
            reverse[target].push(id);
        }
        for (_, target) in &state.transitions {
            reverse[*target].push(id);
        }
    }
    let mut live = vec![false; nfa.num_states()];
    let mut stack: Vec<nfa::StateId> = Vec::new();
    for &(s, _) in &nfa.accepts {
        if !live[s] {
            live[s] = true;
            stack.push(s);
        }
    }
    while let Some(s) = stack.pop() {
        for &p in &reverse[s] {
            if !live[p] {
                live[p] = true;
                stack.push(p);
            }
        }
    }
    live
}

impl Dfa {
    /// True if some string of `self`'s language is a prefix of some string of
    /// `other`'s language. A string is a prefix of itself. The language of
    /// "a" is a prefix of the language of "ab", and also of the language of
    /// [a-z]+; the languages of "if" and "int" share their first character,
    /// but neither is a prefix of the other.
    ///
    /// The implementation walks the product of the two DFAs. From the pair of
    /// start states, it follows character ranges the two DFAs share, so every
    /// reachable pair of states stands for a common string both have read.
    /// The relation holds when a reachable pair combines a language accept in
    /// `self` (the common string is a full string of `self`) with a live
    /// state in `other` (the common string extends to a full string of
    /// `other`).
    pub fn is_prefix_of(&self, other: &Dfa) -> bool {
        let live_other = other.live_states();
        let mut seen = FxHashSet::default();
        let mut stack = vec![(self.start, other.start)];
        while let Some((sa, sb)) = stack.pop() {
            if !seen.insert((sa, sb)) {
                continue;
            }
            if self.is_language_accept(sa) && live_other[sb] {
                return true;
            }
            for (ra, ta) in &self.states[sa].transitions {
                for (rb, tb) in &other.states[sb].transitions {
                    if ra.start <= rb.end && rb.start <= ra.end {
                        stack.push((*ta, *tb));
                    }
                }
            }
        }
        false
    }

    /// True if the state accepts a string of the terminal's language. The
    /// `accept` field alone does not decide this: a state can be an accept
    /// marked `excluded`, and its string is exactly what the `\` operator
    /// removed from the language.
    fn is_language_accept(&self, state: StateId) -> bool {
        let s = &self.states[state];
        s.accept.is_some() && !s.excluded
    }

    /// A vector, indexed by state, of whether the state can reach a language
    /// accept. A live state can extend to a full string of the language; a
    /// dead state cannot, whatever follows. For the DFA of `"if" \ "if"`
    /// every state is dead: the only accept is excluded, so the language is
    /// empty.
    ///
    /// The implementation walks backward from the language accept states
    /// over reversed transitions.
    fn live_states(&self) -> Vec<bool> {
        let n = self.states.len();
        let mut reverse: Vec<Vec<StateId>> = vec![Vec::new(); n];
        for (i, state) in self.states.iter().enumerate() {
            for (_, target) in &state.transitions {
                reverse[*target].push(i);
            }
        }
        let mut live = vec![false; n];
        let mut stack = Vec::new();
        for (i, state_live) in live.iter_mut().enumerate() {
            if self.is_language_accept(i) {
                *state_live = true;
                stack.push(i);
            }
        }
        while let Some(s) = stack.pop() {
            for &p in &reverse[s] {
                if !live[p] {
                    live[p] = true;
                    stack.push(p);
                }
            }
        }
        live
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::regex::Regex;

    fn cr(start: char, end: char) -> CharRange {
        CharRange { start, end }
    }

    fn t(id: u16) -> TerminalId {
        TerminalId(id)
    }

    fn nfa_with(states: Vec<nfa::State>) -> Nfa {
        Nfa {
            states,
            start: 0,
            accepts: vec![],
            except_accepts: vec![],
        }
    }

    fn eps(targets: Vec<nfa::StateId>) -> nfa::State {
        nfa::State {
            epsilon_transitions: targets,
            transitions: vec![],
        }
    }

    /// The DFA state reached from the start by consuming `input`, or `None`
    /// if a character has no transition.
    fn state_after<'a>(dfa: &'a Dfa, input: &str) -> Option<&'a State> {
        let mut state = dfa.start;
        for ch in input.chars() {
            let (_, next) = dfa.states[state]
                .transitions
                .iter()
                .find(|(r, _)| r.start <= ch && ch <= r.end)?;
            state = *next;
        }
        Some(&dfa.states[state])
    }

    #[test]
    fn complement_of_empty_is_unicode_split_around_the_surrogate_gap() {
        assert_eq!(
            complement(&[]),
            vec![cr('\0', '\u{D7FF}'), cr('\u{E000}', char::MAX)]
        );
    }

    #[test]
    fn complement_emits_low_middle_and_high_segments() {
        assert_eq!(
            complement(&[cr('a', 'c')]),
            vec![
                cr('\0', '`'),
                cr('d', '\u{D7FF}'),
                cr('\u{E000}', char::MAX),
            ]
        );
    }

    #[test]
    fn complement_skips_segments_adjacent_to_the_surrogate_gap() {
        assert_eq!(
            complement(&[cr('\0', '\u{D7FF}')]),
            vec![cr('\u{E000}', char::MAX)]
        );
    }

    #[test]
    fn to_non_overlapping_passes_through_disjoint_ranges() {
        assert_eq!(
            to_non_overlapping(&[cr('a', 'c'), cr('e', 'g')]),
            vec![cr('a', 'c'), cr('e', 'g')]
        );
    }

    #[test]
    fn to_non_overlapping_splits_overlapping_ranges() {
        assert_eq!(
            to_non_overlapping(&[cr('a', 'c'), cr('b', 'd')]),
            vec![cr('a', 'a'), cr('b', 'c'), cr('d', 'd')]
        );
    }

    #[test]
    fn to_non_overlapping_keeps_adjacent_ranges_separate() {
        assert_eq!(
            to_non_overlapping(&[cr('a', 'c'), cr('d', 'f')]),
            vec![cr('a', 'c'), cr('d', 'f')]
        );
    }

    #[test]
    fn to_non_overlapping_returns_empty_for_empty_input() {
        assert_eq!(to_non_overlapping(&[]), Vec::<CharRange>::new());
    }

    #[test]
    fn epsilon_closure_of_a_state_with_no_outgoing_epsilons_is_just_the_seed() {
        let nfa = nfa_with(vec![nfa::State::default()]);
        assert_eq!(epsilon_closure(&nfa, [0]), vec![0]);
    }

    #[test]
    fn epsilon_closure_follows_a_chain_transitively() {
        let nfa = nfa_with(vec![eps(vec![1]), eps(vec![2]), nfa::State::default()]);
        assert_eq!(epsilon_closure(&nfa, [0]), vec![0, 1, 2]);
    }

    #[test]
    fn epsilon_closure_terminates_on_cycles() {
        let nfa = nfa_with(vec![eps(vec![1]), eps(vec![0])]);
        assert_eq!(epsilon_closure(&nfa, [0]), vec![0, 1]);
    }

    #[test]
    fn epsilon_closure_unions_reachability_from_multiple_seeds() {
        let nfa = nfa_with(vec![
            eps(vec![1]),
            nfa::State::default(),
            eps(vec![3]),
            nfa::State::default(),
        ]);
        assert_eq!(epsilon_closure(&nfa, [0, 2]), vec![0, 1, 2, 3]);
    }

    #[test]
    fn to_char_ranges_passes_non_negated_ranges_through() {
        let class = CharClass {
            ranges: vec![cr('a', 'c'), cr('x', 'z')],
            negated: false,
        };
        assert_eq!(to_char_ranges(&class), vec![cr('a', 'c'), cr('x', 'z')]);
    }

    #[test]
    fn to_char_ranges_complements_negated_ranges() {
        let class = CharClass {
            ranges: vec![cr('a', 'c')],
            negated: true,
        };
        assert_eq!(
            to_char_ranges(&class),
            vec![
                cr('\0', '`'),
                cr('d', '\u{D7FF}'),
                cr('\u{E000}', char::MAX),
            ],
        );
    }

    #[test]
    fn overlapping_classes_split_into_disjoint_atoms() {
        let dfa = Dfa::from_nfa(&Nfa::from_regex(
            &Regex::alt(vec![
                Regex::CharClass(CharClass {
                    ranges: vec![cr('a', 'c')],
                    negated: false,
                }),
                Regex::CharClass(CharClass {
                    ranges: vec![cr('b', 'd')],
                    negated: false,
                }),
            ]),
            t(0),
        ));
        // Outgoing alphabet covers a..=d, split at the overlap boundary into
        // three atoms; every atom must lead to an accepting state.
        let start_state = &dfa.states[dfa.start];
        let start_ranges: Vec<CharRange> =
            start_state.transitions.iter().map(|(r, _)| *r).collect();
        assert_eq!(start_ranges, vec![cr('a', 'a'), cr('b', 'c'), cr('d', 'd')]);
        for (_, target) in &start_state.transitions {
            assert_eq!(dfa.states[*target].accept, Some(t(0)));
        }
    }

    #[test]
    fn earlier_accept_wins_when_a_dfa_state_covers_two_nfa_accepts() {
        // Both branches accept 'a'; declaration order should pick terminal 0.
        let mut nfa = Nfa::from_regex(&Regex::char('a'), t(0));
        let second = Nfa::from_regex(&Regex::char('a'), t(1));
        let offset = nfa.num_states();
        for state in &second.states {
            nfa.states.push(nfa::State {
                epsilon_transitions: state
                    .epsilon_transitions
                    .iter()
                    .map(|s| s + offset)
                    .collect(),
                transitions: state
                    .transitions
                    .iter()
                    .map(|(c, s)| (c.clone(), s + offset))
                    .collect(),
            });
        }
        let new_start = nfa.states.len();
        nfa.states.push(nfa::State {
            epsilon_transitions: vec![nfa.start, second.start + offset],
            transitions: vec![],
        });
        nfa.start = new_start;
        nfa.accepts
            .extend(second.accepts.iter().map(|(s, t)| (s + offset, *t)));
        let dfa = Dfa::from_nfa(&nfa);
        let start = &dfa.states[dfa.start];
        let (_, target) = start
            .transitions
            .iter()
            .find(|(r, _)| r.start <= 'a' && 'a' <= r.end)
            .unwrap();
        assert_eq!(dfa.states[*target].accept, Some(t(0)));
    }

    /// `[a-z][a-z]? \ ("if" | "iffy")`
    fn two_letter_id_without_keywords() -> Dfa {
        let id = Regex::seq(vec![
            Regex::range('a', 'z'),
            Regex::Opt(Box::new(Regex::range('a', 'z'))),
        ]);
        let keywords = Regex::alt(vec![Regex::literal("if"), Regex::literal("iffy")]);
        Dfa::from_nfa(&Nfa::with_excepts(&id, t(0), &[&keywords]))
    }

    #[test]
    fn except_accept_marks_the_accept_state_excluded() {
        let dfa = two_letter_id_without_keywords();
        let state = state_after(&dfa, "if").unwrap();
        assert_eq!(state.accept, Some(t(0)));
        assert!(state.excluded);
    }

    #[test]
    fn accept_states_outside_the_except_language_are_not_excluded() {
        let dfa = two_letter_id_without_keywords();
        for input in ["i", "ix"] {
            let state = state_after(&dfa, input).unwrap();
            assert_eq!(state.accept, Some(t(0)));
            assert!(!state.excluded, "{input:?} must not be excluded");
        }
    }

    #[test]
    fn walks_stop_once_only_except_states_remain() {
        // After "if" the two-letter terminal is dead; the "iffy" suffix of
        // the except must not be materialized.
        let dfa = two_letter_id_without_keywords();
        assert!(state_after(&dfa, "iff").is_none());
    }

    #[test]
    fn except_inside_a_longer_match_does_not_mark_the_end() {
        // [a-z]+ \ "if" on "iff": the accept at "if" is excluded, the accept
        // at "iff" is not, so maximal munch keeps the longer match.
        let id = Regex::plus(Regex::range('a', 'z'));
        let keyword = Regex::literal("if");
        let dfa = Dfa::from_nfa(&Nfa::with_excepts(&id, t(0), &[&keyword]));
        assert!(state_after(&dfa, "if").unwrap().excluded);
        let state = state_after(&dfa, "iff").unwrap();
        assert_eq!(state.accept, Some(t(0)));
        assert!(!state.excluded);
    }

    fn dfa(regex: Regex) -> Dfa {
        Dfa::from_nfa(&Nfa::from_regex(&regex, t(0)))
    }

    #[test]
    fn shorter_literal_prefixes_longer_only_one_way() {
        // "a" is a prefix of "ab", but "ab" is not a prefix of "a".
        assert!(dfa(Regex::literal("a")).is_prefix_of(&dfa(Regex::literal("ab"))));
        assert!(!dfa(Regex::literal("ab")).is_prefix_of(&dfa(Regex::literal("a"))));
    }

    #[test]
    fn equal_languages_prefix_each_other() {
        assert!(dfa(Regex::literal("ab")).is_prefix_of(&dfa(Regex::literal("ab"))));
    }

    #[test]
    fn disjoint_literals_do_not_prefix() {
        assert!(!dfa(Regex::literal("ab")).is_prefix_of(&dfa(Regex::literal("cd"))));
    }

    #[test]
    fn shared_first_char_without_prefix_does_not_count() {
        // "if" and "int" share their first character, but neither is a prefix
        // of the other, so a keyword-versus-keyword pair stays distinguishable.
        assert!(!dfa(Regex::literal("if")).is_prefix_of(&dfa(Regex::literal("int"))));
        assert!(!dfa(Regex::literal("int")).is_prefix_of(&dfa(Regex::literal("if"))));
    }

    #[test]
    fn integer_prefixes_float() {
        // [0-9]+ is a prefix of [0-9]+ ".", the Int/Float selector case.
        let int = Regex::plus(Regex::range('0', '9'));
        let float = Regex::seq(vec![Regex::plus(Regex::range('0', '9')), Regex::char('.')]);
        assert!(dfa(int).is_prefix_of(&dfa(float)));
    }
}
