//! wasm-bindgen wrapper over iguana-lsp's pure analysis functions, for the
//! read-only iggy grammar viewer on the website.
//!
//! Each entry point takes the grammar source (plus a cursor position where the
//! feature needs one) and returns the LSP result as a JSON string. The boundary
//! is all strings: every return type is an `lsp_types` value that already
//! serializes to the shape Monaco's providers expect. The viewer is read-only,
//! so each call re-parses from scratch; iggy grammars parse in a few
//! milliseconds, and re-parsing sidesteps caching a parse tree whose lifetime
//! cannot cross the wasm boundary.

use iguana_lsp::spans::GrammarSpans;
use iguana_lsp::{BuildResult, GrammarDef, build, build_grammar_def, build_spans};
use iguana_runtime::{input::Input, parse_tree::Bump};
use lsp_types::SemanticTokens;
use wasm_bindgen::prelude::*;

/// Placeholder document URI. The viewer holds a single grammar, so locations
/// only carry a range; the frontend ignores the URI.
const URI: &str = "file:///grammar.iggy";

/// The semantic-token legend (type names and modifiers), as LSP-shaped JSON.
/// The frontend registers it once with Monaco.
#[wasm_bindgen]
pub fn semantic_tokens_legend() -> String {
    serde_json::to_string(&iguana_lsp::semantic_tokens::legend()).unwrap()
}

/// Semantic tokens for `source`, as a JSON `SemanticTokens` whose `data` is the
/// LSP delta encoding flattened to a `u32` array. The `data` is empty when the
/// grammar does not parse or is ambiguous.
#[wasm_bindgen]
pub fn semantic_tokens(source: &str) -> String {
    let tokens = SemanticTokens {
        result_id: None,
        data: iguana_lsp::semantic_tokens::tokenize(source),
    };
    serde_json::to_string(&tokens).unwrap()
}

/// Diagnostics for `source`, as a JSON array of LSP diagnostics.
#[wasm_bindgen]
pub fn diagnostics(source: &str) -> String {
    let diags = with_spans(source, vec![], |def, spans, input| {
        iguana_lsp::diagnostics::diagnostics(def, spans, input)
    });
    serde_json::to_string(&diags).unwrap()
}

/// Document symbols for `source`, as a JSON array of nested LSP symbols.
#[wasm_bindgen]
pub fn document_symbols(source: &str) -> String {
    let symbols = with_spans(source, vec![], |def, spans, input| {
        iguana_lsp::document_symbols::document_symbols(def, spans, input)
    });
    serde_json::to_string(&symbols).unwrap()
}

/// Folding ranges for `source`, as a JSON array of LSP folding ranges.
#[wasm_bindgen]
pub fn folding(source: &str) -> String {
    let ranges = with_spans(source, vec![], |def, spans, input| {
        iguana_lsp::folding::folding_ranges(def, spans, input)
    });
    serde_json::to_string(&ranges).unwrap()
}

/// The definition location of the symbol at `(line, character)`, as a JSON LSP
/// location, or `null` when there is no symbol at that position.
#[wasm_bindgen]
pub fn definition(source: &str, line: u32, character: u32) -> String {
    let loc = with_spans(source, None, |_def, spans, input| {
        let uri = URI.parse().unwrap();
        let offset = input.offset(line, character);
        iguana_lsp::references::definition(spans, input, &uri, offset)
    });
    serde_json::to_string(&loc).unwrap()
}

/// All references to the symbol at `(line, character)`, as a JSON array of LSP
/// locations. The defining rule head is included when `include_declaration`.
#[wasm_bindgen]
pub fn references(source: &str, line: u32, character: u32, include_declaration: bool) -> String {
    let locs = with_spans(source, vec![], |_def, spans, input| {
        let uri = URI.parse().unwrap();
        let offset = input.offset(line, character);
        iguana_lsp::references::references(spans, input, &uri, offset, include_declaration)
    });
    serde_json::to_string(&locs).unwrap()
}

/// Parse `source`, build the grammar definition and span table, and hand them to
/// `f`. Returns `default` when the grammar does not parse, is ambiguous, or
/// fails to resolve.
fn with_spans<T>(
    source: &str,
    default: T,
    f: impl FnOnce(&GrammarDef, &GrammarSpans, &Input) -> T,
) -> T {
    let input = Input::from(source);
    let tree_arena = Bump::new();
    let BuildResult::Success { ref tree, .. } = build(&input, &tree_arena) else {
        return default;
    };
    let Some(grammar_def) = build_grammar_def(tree, &input) else {
        return default;
    };
    let spans = build_spans(&grammar_def, tree, &input);
    f(&grammar_def, &spans, &input)
}
