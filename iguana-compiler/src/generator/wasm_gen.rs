use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    grammar::def::Grammar,
    utils::{to_first_uppercase, to_snake_case},
};

/// Generate the `lib.rs` of the wasm wrapper crate: a `wasm-bindgen` entry
/// point that runs the generated parser and returns the result as a JSON
/// envelope. The body is the same for every grammar; only the parser crate's
/// name and its parser and parse-tree-builder types vary.
pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", to_snake_case(&grammar.name));
    let parser = format_ident!("{}Parser", to_first_uppercase(&grammar.name));
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(&grammar.name));
    quote! {
        use wasm_bindgen::prelude::*;

        use iguana_runtime::{
            Instant,
            input::Input,
            parse_tree::ParseContext,
            parser::{ParseResult, Parser},
        };
        use #grammar_name::{
            grammar_data::nonterminal_id,
            parse_tree::{#parse_tree_builder, create_parse_tree, to_json},
            parser::#parser,
        };

        #[doc = r" Parses `input` from the nonterminal named `start`, returning a JSON result"]
        #[doc = r" envelope. A parse that ran returns `success`, the timings, and the parse-tree"]
        #[doc = r" JSON; a parse that failed returns `success: false` with the error location."]
        #[doc = r" An unrecognized start nonterminal cannot run at all, so it returns an error."]
        #[wasm_bindgen]
        pub fn parse(input: &str, start: &str) -> Result<String, JsError> {
            // A start nonterminal A has a generated StartA wrapper that handles
            // layout and EOF; fall back to A directly when it is not one.
            let start_nonterminal_id = nonterminal_id(&format!("Start{start}"))
                .or_else(|| nonterminal_id(start))
                .ok_or_else(|| JsError::new(&format!("unknown start nonterminal: {start}")))?;

            let input = Input::from(input);
            let ctx = ParseContext::new();
            let parse_tree_builder = #parse_tree_builder::new(&ctx);
            let mut parser = #parser::new(&input, start_nonterminal_id);
            match parser.run() {
                ParseResult::Success(success) => {
                    let tree_start = Instant::now();
                    let tree = create_parse_tree(
                        success.sppf_node_id,
                        start_nonterminal_id,
                        &parser,
                        &parse_tree_builder,
                    );
                    let tree_construction_ms = tree_start.elapsed().as_millis() as u32;
                    let envelope = serde_json::json!({
                        "success": true,
                        "error": serde_json::Value::Null,
                        "error_info": serde_json::Value::Null,
                        "duration_ms": success.duration.as_millis() as u32,
                        "tree_construction_ms": tree_construction_ms,
                        "parse_tree": to_json(tree),
                    });
                    Ok(envelope.to_string())
                }
                ParseResult::Failure(error) => {
                    let (line, column, message) = parser.format_error(&error);
                    let len = parser.error_span_len(error.input_index);
                    let envelope = serde_json::json!({
                        "success": false,
                        "error": format!("Parse error at line {line}, column {column}: {message}"),
                        "error_info": {
                            "line": line,
                            "column": column,
                            "len": len,
                            "message": message,
                        },
                        "duration_ms": serde_json::Value::Null,
                        "tree_construction_ms": serde_json::Value::Null,
                        "parse_tree": serde_json::Value::Null,
                    });
                    Ok(envelope.to_string())
                }
            }
        }
    }
}
