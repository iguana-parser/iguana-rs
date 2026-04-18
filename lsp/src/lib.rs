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

pub struct ParseResult {
    pub tree: Option<Start<Grammar, Layout>>,
    pub input: Input,
    /// Time spent in the GLL parsing algorithm.
    pub parse_duration: Duration,
    /// Time spent constructing the typed parse tree from the SPPF.
    pub tree_construction_duration: Duration,
}

/// Build a GrammarDef from a successful parse result.
pub fn build_grammar_def(result: &ParseResult) -> Option<GrammarDef> {
    let start = result.tree.as_ref()?;
    iguana::iggy::build_grammar(start, &result.input)
        .ok()
        .map(|def| def.resolve())
}

/// Build the side table from a GrammarDef and its parse tree.
pub fn build_spans<'a>(
    grammar_def: &'a GrammarDef,
    result: &ParseResult,
) -> Option<GrammarSpans<'a>> {
    let start = result.tree.as_ref()?;
    Some(spans::build_spans(grammar_def, start, &result.input))
}

/// Parse the grammar source and return a ParseResult.
pub fn parse(source: &str) -> ParseResult {
    let input = Input::from(source);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        iggy::parse_grammar(&input)
    }))
    .ok()
    .and_then(|r| r.ok());
    match result {
        Some(success) => ParseResult {
            parse_duration: success.parse_duration,
            tree_construction_duration: success.tree_construction_duration,
            tree: Some(success.tree),
            input,
        },
        None => ParseResult {
            tree: None,
            input,
            parse_duration: Duration::ZERO,
            tree_construction_duration: Duration::ZERO,
        },
    }
}
