pub mod diagnostics;
pub mod document_symbols;
pub mod folding;
pub mod format;
pub mod name_resolution;
pub mod references;
pub mod semantic_tokens;
pub mod symbols;

use iggy::parse_tree::{Grammar, Layout, Start};
pub use iguana_compiler::grammar::def::GrammarDef;
pub use iguana_compiler::{comments, spans};
use iguana_runtime::{arena::Arena, input::Input, parse_tree::ParseTreeNode};
use spans::GrammarSpans;
use std::time::Duration;

pub enum BuildResult<'a> {
    Success {
        tree: &'a Start<&'a Grammar<'a>, &'a Layout<'a>>,
        parse_duration: Duration,
        tree_construction_duration: Duration,
    },
    /// The grammar parsed but is ambiguous, so it cannot be analyzed. Callers
    /// treat this like a parse failure and skip the rest of the pipeline, but
    /// without a location to mark, so it surfaces no diagnostic.
    Ambiguous,
    Error {
        line: u32,
        column: u32,
        len: u32,
        message: String,
    },
}

/// Build a GrammarDef from a successful parse tree.
pub fn build_grammar_def(tree: &Start<&Grammar<'_>, &Layout<'_>>, input: &Input) -> GrammarDef {
    iguana_compiler::iggy::build_grammar(tree, input).resolve()
}

/// Build the side table from a GrammarDef and its parse tree.
pub fn build_spans<'a>(
    grammar_def: &'a GrammarDef,
    tree: &Start<&Grammar<'_>, &Layout<'_>>,
    input: &Input,
) -> GrammarSpans<'a> {
    spans::build_spans(grammar_def, tree, input)
}

/// Parse the grammar source and build the result.
pub fn build<'a>(input: &Input, tree_arena: &'a Arena) -> BuildResult<'a> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        iggy::parse_grammar(input, tree_arena)
    }));
    match result {
        Ok(Ok(success)) => {
            // The flag is a cheap necessary condition; confirm with a tree walk,
            // since the recorded ambiguity may sit in a dead branch the tree
            // never reaches.
            if success.ambiguity_node_added
                && success.tree.node.as_parse_tree().contains_ambiguity()
            {
                BuildResult::Ambiguous
            } else {
                BuildResult::Success {
                    tree: success.tree,
                    parse_duration: success.parse_duration,
                    tree_construction_duration: success.tree_construction_duration,
                }
            }
        }
        Ok(Err(error)) => BuildResult::Error {
            line: error.line,
            column: error.column,
            len: error.len,
            message: error.message,
        },
        Err(_) => BuildResult::Error {
            line: 0,
            column: 0,
            len: 0,
            message: "Internal error during parsing".to_string(),
        },
    }
}
