//! Regex → DFA compilation pipeline.
//!
//! Stages:
//! - `nfa`: Thompson construction from the `Regex` AST. Multiple regexes are
//!   unioned under a shared start state, with each terminal's accept state
//!   tagged by its `TerminalId`.
//! - `dfa`: subset construction from an `Nfa`, producing a DFA whose
//!   transitions carry disjoint `CharRange` atoms.

pub mod dfa;
pub mod nfa;

pub use dfa::Dfa;
pub use nfa::Nfa;
