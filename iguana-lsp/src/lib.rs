pub mod analysis;
mod diagnostics;
mod document_symbols;
mod folding;
pub mod format;
mod name_resolution;
pub mod semantic_tokens;

use iggy::IggyParser;
use iggy::parse_tree::{Grammar, Layout, Start};
use iguana_compiler::grammar::def::GrammarDef;
use iguana_compiler::validation::GrammarError;
use iguana_runtime::{
    arena::Arena,
    input::{Input, Span},
    parse_tree::ParseTreeNode,
};
use lsp_types::{Position, Range};
use std::time::Duration;

pub(crate) enum BuildResult<'a> {
    Success {
        tree: &'a Start<&'a Grammar<'a>, &'a Layout<'a>>,
        parse_duration: Duration,
        tree_construction_duration: Duration,
    },
    /// The grammar parsed but is ambiguous. The variant does not have a span,
    /// so the analysis does not report a diagnostic.
    Ambiguous,
    Error(GrammarError),
}

/// Build a GrammarDef from a successful parse tree.
pub(crate) fn build_grammar_def(
    tree: &Start<&Grammar<'_>, &Layout<'_>>,
    input: &Input,
) -> GrammarDef {
    iguana_compiler::iggy::build_grammar(tree, input).resolve()
}

/// Parse the grammar source and build the result. A panic in the parser becomes
/// `BuildResult::Error` on native targets. `catch_unwind` does not catch a
/// panic on `wasm32-unknown-unknown`, where the panic traps.
pub(crate) fn build<'a>(input: &Input, tree_arena: &'a Arena) -> BuildResult<'a> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let parser_arena = Arena::new();
        IggyParser::new(input, &parser_arena).parse_grammar(tree_arena)
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
                    tree: tree_arena.alloc(success.tree),
                    parse_duration: success.parse_duration,
                    tree_construction_duration: success.tree_construction_duration,
                }
            }
        }
        Ok(Err(error)) => BuildResult::Error(GrammarError {
            message: error.message,
            span: error.span,
        }),
        Err(_) => BuildResult::Error(GrammarError {
            message: "Internal error during parsing".to_string(),
            span: Span::new(0, 0),
        }),
    }
}

pub(crate) fn to_range(span: Span, input: &Input) -> Range {
    let (start_line, start_column) = input.line_column(span.left_extent);
    let (end_line, end_column) = input.line_column(span.right_extent);
    Range {
        start: Position::new(start_line, start_column),
        end: Position::new(end_line, end_column),
    }
}
