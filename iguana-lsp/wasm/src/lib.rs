//! wasm-bindgen wrapper over iguana-lsp's analysis, for the read-only iggy
//! grammar viewer on the website.
//!
//! Each entry point takes the grammar source (plus a cursor position where the
//! feature needs one) and returns the LSP result as a JSON string. The boundary
//! is all strings: every return type is an `lsp_types` value that already
//! serializes to the shape Monaco's providers expect.
//!
//! The module keeps the analysis of the most recent source and answers requests
//! about that source without parsing it again. The module does not keep older
//! analyses, because a page shows one grammar.

use std::cell::RefCell;
use std::rc::Rc;

use iguana_lsp::analysis::Analysis;
use lsp_types::{Location, Position, SemanticTokens, Uri};
use wasm_bindgen::prelude::*;

/// Placeholder document URI. The frontend ignores the URI of a location,
/// because the viewer holds a single grammar.
const URI: &str = "file:///grammar.iggy";

thread_local! {
    static LATEST: RefCell<Option<Rc<Analysis>>> = const { RefCell::new(None) };
}

/// The analysis of `source`, built only when `source` differs from the source
/// of the previous call.
///
/// A panic traps in wasm without running destructors, and the module stays in
/// use after the trap. A borrow of `LATEST` held across the panic would stay
/// set and fail every later call. The analysis is behind an `Rc` so that the
/// borrow ends before the analysis is built and before any answer is computed.
fn analysis(source: &str) -> Rc<Analysis> {
    match LATEST.with_borrow(Clone::clone) {
        Some(analysis) if analysis.source() == source => analysis,
        _ => {
            let analysis = Rc::new(Analysis::new(source));
            LATEST.set(Some(Rc::clone(&analysis)));
            analysis
        }
    }
}

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
        data: analysis(source).semantic_tokens().to_vec(),
    };
    serde_json::to_string(&tokens).unwrap()
}

/// Diagnostics for `source`, as a JSON array of LSP diagnostics. A grammar that
/// does not parse has one diagnostic at the parse error.
#[wasm_bindgen]
pub fn diagnostics(source: &str) -> String {
    serde_json::to_string(analysis(source).diagnostics()).unwrap()
}

/// Document symbols for `source`, as a JSON array of nested LSP symbols.
#[wasm_bindgen]
pub fn document_symbols(source: &str) -> String {
    serde_json::to_string(analysis(source).document_symbols()).unwrap()
}

/// Folding ranges for `source`, as a JSON array of LSP folding ranges.
#[wasm_bindgen]
pub fn folding(source: &str) -> String {
    serde_json::to_string(analysis(source).folding_ranges()).unwrap()
}

/// The definition location of the symbol at `(line, character)`, as a JSON LSP
/// location, or `null` when there is no symbol at that position.
#[wasm_bindgen]
pub fn definition(source: &str, line: u32, character: u32) -> String {
    let uri: Uri = URI.parse().unwrap();
    let location = analysis(source)
        .definition(Position::new(line, character))
        .map(|range| Location::new(uri, range));
    serde_json::to_string(&location).unwrap()
}

/// All references to the symbol at `(line, character)`, as a JSON array of LSP
/// locations. The defining rule head is included when `include_declaration`.
#[wasm_bindgen]
pub fn references(source: &str, line: u32, character: u32, include_declaration: bool) -> String {
    let uri: Uri = URI.parse().unwrap();
    let locations: Vec<_> = analysis(source)
        .references(Position::new(line, character), include_declaration)
        .into_iter()
        .map(|range| Location::new(uri.clone(), range))
        .collect();
    serde_json::to_string(&locations).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &str = "grammar T\n\nA\n  = \"x\"\n";
    const SECOND: &str = "grammar T\n\nB\n  = \"y\"\n";

    #[test]
    fn same_source_reuses_the_analysis() {
        assert!(Rc::ptr_eq(&analysis(FIRST), &analysis(FIRST)));
    }

    #[test]
    fn different_source_rebuilds_the_analysis() {
        let first = analysis(FIRST);
        let second = analysis(SECOND);
        assert!(!Rc::ptr_eq(&first, &second));
        assert_eq!(second.document_symbols()[0].name, "B");
        assert!(!Rc::ptr_eq(&first, &analysis(FIRST)));
    }

    #[test]
    fn answers_follow_the_source_argument() {
        assert!(definition(FIRST, 2, 0).contains("\"line\":2"));
        assert_eq!(definition(SECOND, 0, 100), "null");
        assert!(document_symbols(FIRST).contains("\"name\":\"A\""));
        assert!(document_symbols(SECOND).contains("\"name\":\"B\""));
    }

    #[test]
    fn parse_error_is_a_diagnostic() {
        assert!(diagnostics("grammar").contains("\"line\":0,\"character\":7"));
        assert_eq!(diagnostics(FIRST), "[]");
    }
}
