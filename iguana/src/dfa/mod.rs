//! Regex → DFA compilation pipeline.
//!
//! Stages:
//! - `nfa`: Thompson construction from the `Regex` AST. Multiple regexes are
//!   unioned under a shared start state, with each terminal's accept state
//!   tagged by its `TerminalId`. Except regexes join the same union, with
//!   their accept states listed separately.
//! - `dfa`: subset construction from an `Nfa`, producing a DFA whose
//!   transitions carry disjoint `CharRange` atoms. Accept states where an
//!   except also accepts are marked excluded, and states from which no
//!   accept is reachable are never built, so a scan stops where the
//!   terminal dies instead of following except fragments further.

pub mod dfa;
pub mod nfa;

pub use dfa::Dfa;
pub use nfa::Nfa;
