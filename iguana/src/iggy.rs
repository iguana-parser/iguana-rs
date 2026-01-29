//! Iggy grammar parser and converter.
//!
//! This module parses iggy grammar source files and converts them to
//! iguana's Grammar representation.

use crate::grammar::def::Grammar;
use iggy::parse_tree::Grammar as IggyParseTree;

/// Error type for iggy parsing and conversion failures.
#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

/// Parse an iggy grammar source string and convert to Grammar.
pub fn parse_grammar(source: &str) -> Result<Grammar, Error> {
    let parse_tree = parse(source)?;
    build_grammar(&parse_tree)
}

/// Parse iggy source into a parse tree.
fn parse(source: &str) -> Result<IggyParseTree, Error> {
    todo!("parse iggy source")
}

/// Convert an iggy parse tree to an iguana Grammar.
fn build_grammar(tree: &IggyParseTree) -> Result<Grammar, Error> {
    todo!("build grammar from parse tree")
}
