pub mod diagnostics;
pub mod document_symbols;
pub mod folding;
pub mod format;
pub mod layout;
pub mod references;
pub mod semantic_tokens;
pub mod spans;
pub mod symbols;

use iggy::parse_tree::{Grammar, Layout, Start};
use iguana::grammar::def::GrammarDef;
use iguana_runtime::input::Input;
use spans::GrammarSpans;
use std::time::Duration;

pub enum BuildResult {
    Success {
        tree: Start<Grammar, Layout>,
        parse_duration: Duration,
        tree_construction_duration: Duration,
    },
    Error {
        line: u32,
        column: u32,
        message: String,
    },
}

/// Build a GrammarDef from a successful parse tree.
pub fn build_grammar_def(tree: &Start<Grammar, Layout>, input: &Input) -> Option<GrammarDef> {
    iguana::iggy::build_grammar(tree, input)
        .ok()
        .map(|def| def.resolve())
}

/// Build the side table from a GrammarDef and its parse tree.
pub fn build_spans<'a>(
    grammar_def: &'a GrammarDef,
    tree: &Start<Grammar, Layout>,
    input: &Input,
) -> GrammarSpans<'a> {
    spans::build_spans(grammar_def, tree, input)
}

/// Parse the grammar source and build the result.
pub fn build(input: &Input) -> BuildResult {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        iggy::parse_grammar(input)
    }));
    match result {
        Ok(Ok(success)) => BuildResult::Success {
            tree: success.tree,
            parse_duration: success.parse_duration,
            tree_construction_duration: success.tree_construction_duration,
        },
        Ok(Err(error)) => BuildResult::Error {
            line: error.line,
            column: error.column,
            message: error.message,
        },
        Err(_) => BuildResult::Error {
            line: 0,
            column: 0,
            message: "Internal error during parsing".to_string(),
        },
    }
}
