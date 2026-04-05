pub mod format;
pub mod semantic_tokens;

use iggy::parse_tree::ParseTree;
use iguana_runtime::input::Input;
use std::time::Duration;

/// Result of parsing a grammar source. Holds the parse tree (if successful)
/// and the input needed for span-to-position conversions.
///
/// This is the central cached state that all language intelligence features
/// (semantic tokens, diagnostics, go-to-definition, etc.) read from.
pub struct ParseResult {
    pub tree: Option<ParseTree>,
    pub input: Input,
    /// Time spent in the GLL parsing algorithm.
    pub parse_duration: Duration,
    /// Time spent constructing the typed parse tree from the SPPF.
    pub tree_construction_duration: Duration,
}

/// Parse the grammar source and return a ParseResult.
/// This is a stateless function — caching is the caller's responsibility
/// (Terrarium backend or LSP server).
pub fn parse(source: &str) -> ParseResult {
    let input = Input::from(source);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        iggy::parse(&input, "StartGrammar")
    }))
    .ok()
    .flatten();
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
