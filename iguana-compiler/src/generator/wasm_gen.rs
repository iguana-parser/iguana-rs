use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::grammar_utils::{parse_tree_builder_ident, parser_ident},
    grammar::def::Grammar,
    utils::to_snake_case,
};

/// Generate the `lib.rs` of the wasm wrapper crate: a `wasm-bindgen` entry
/// point that runs the generated parser and returns the result as a JSON
/// envelope. The body is the same for every grammar; only the parser crate's
/// name and its parser and parse-tree-builder types vary.
pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", to_snake_case(&grammar.name));
    let parser = parser_ident(&grammar.name);
    let parse_tree_builder = parse_tree_builder_ident(&grammar.name);
    quote! {
        use wasm_bindgen::prelude::*;

        use iguana_runtime::{
            Instant,
            arena::Arena,
            cli::ParseOutput,
            input::Input,
            parser::{GLLResult, Parser},
        };
        use #grammar_name::{
            grammar_data::nonterminal_id,
            parse_tree::{#parse_tree_builder, create_parse_tree, to_json},
            parser::#parser,
        };

        #[doc = r" Parses `input` from the nonterminal named `start`, returning the runtime's"]
        #[doc = r" `ParseOutput` as JSON. A parse that succeeded returns the timings and the"]
        #[doc = r" parse-tree JSON; a parse that failed returns the error span and message."]
        #[doc = r" An unrecognized start nonterminal cannot run at all, so it returns an error."]
        #[wasm_bindgen]
        pub fn parse(input: &str, start: &str) -> Result<String, JsError> {
            // A start nonterminal A has a generated StartA wrapper that handles
            // layout and EOF; fall back to A directly when it is not one.
            let start_nonterminal_id = nonterminal_id(&format!("Start{start}"))
                .or_else(|| nonterminal_id(start))
                .ok_or_else(|| JsError::new(&format!("unknown start nonterminal: {start}")))?;

            let input = Input::from(input);
            let tree_arena = Arena::new();
            let parse_tree_builder = #parse_tree_builder::new(&tree_arena);
            let vec_arena = Arena::new();
            let mut parser = #parser::new(&input, start_nonterminal_id, &vec_arena);
            match parser.run() {
                GLLResult::Success(success) => {
                    let tree_start = Instant::now();
                    let tree = create_parse_tree(
                        success.sppf_node_id,
                        start_nonterminal_id,
                        &parser,
                        &parse_tree_builder,
                    );
                    let tree_construction_ms = tree_start.elapsed().as_millis() as u32;
                    let envelope = ParseOutput {
                        error: None,
                        parse_ms: Some(success.duration.as_millis() as u32),
                        tree_construction_ms: Some(tree_construction_ms),
                        parse_tree: Some(to_json(tree)),
                    };
                    Ok(serde_json::to_string(&envelope).unwrap())
                }
                GLLResult::Failure(error) => {
                    let envelope = ParseOutput {
                        error: Some(parser.to_parse_error(&error)),
                        parse_ms: None,
                        tree_construction_ms: None,
                        parse_tree: None,
                    };
                    Ok(serde_json::to_string(&envelope).unwrap())
                }
            }
        }
    }
}
