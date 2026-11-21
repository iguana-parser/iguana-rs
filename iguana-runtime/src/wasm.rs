//! The reply of a generated parser's WebAssembly wrapper.

use serde::Serialize;

use crate::{highlight::Highlight, result::ParseSummary};

/// The reply of the WebAssembly `parse`, which returns it as JSON: the
/// summary of the parse, with its fields at the top level, the parse tree,
/// and the highlights. `parse_tree` is the tree's JSON after a success
/// (`to_json`). `highlights` is there when the caller asks for it: the
/// highlights of the tree after a success, and of the scanner's tokens after
/// a failure.
#[derive(Serialize)]
pub struct WasmParseOutput<G> {
    #[serde(flatten)]
    pub summary: ParseSummary,
    pub parse_tree: Option<String>,
    pub highlights: Option<Vec<Highlight<G>>>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        highlight::Highlight,
        input::Span,
        result::{ParseError, ParseSummary},
        wasm::WasmParseOutput,
    };

    // The browser reads the summary's fields at the top level, beside the tree
    // and the highlights.
    #[test]
    fn the_summary_fields_are_at_the_top_level() {
        let success = WasmParseOutput {
            summary: ParseSummary {
                error: None,
                parse_ms: Some(2),
                tree_construction_ms: Some(1),
            },
            parse_tree: Some("{}".to_string()),
            highlights: Some(vec![Highlight::new(Span::new(0, 7), "keyword")]),
        };
        assert_eq!(
            serde_json::to_value(&success).unwrap(),
            json!({
                "error": null,
                "parse_ms": 2,
                "tree_construction_ms": 1,
                "parse_tree": "{}",
                "highlights": [
                    { "span": { "left_extent": 0, "right_extent": 7 }, "group": "keyword" }
                ],
            })
        );

        let failure = WasmParseOutput::<&str> {
            summary: ParseSummary {
                error: Some(ParseError {
                    span: Span::new(3, 4),
                    message: "Expected \"=\"".to_string(),
                }),
                parse_ms: None,
                tree_construction_ms: None,
            },
            parse_tree: None,
            highlights: None,
        };
        assert_eq!(
            serde_json::to_value(&failure).unwrap(),
            json!({
                "error": {
                    "span": { "left_extent": 3, "right_extent": 4 },
                    "message": "Expected \"=\"",
                },
                "parse_ms": null,
                "tree_construction_ms": null,
                "parse_tree": null,
                "highlights": null,
            })
        );
    }
}
