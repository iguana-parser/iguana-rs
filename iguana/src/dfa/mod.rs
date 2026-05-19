//! Regex → DFA compilation pipeline.
//!
//! Stages:
//! - `nfa`: Thompson construction from the `Regex` AST. Multiple regexes are
//!   unioned under a shared start state, with each terminal's accept state
//!   tagged by its `TerminalId`.

pub mod nfa;
