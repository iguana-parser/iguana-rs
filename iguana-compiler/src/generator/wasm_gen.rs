use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::grammar_utils::{grammar_ident, parser_ident},
    grammar::def::Grammar,
    utils::to_snake_case,
};

/// Generate the `lib.rs` of the wasm wrapper crate: a `wasm-bindgen` entry
/// point that runs the generated parser and returns the result as a JSON
/// envelope. The body is the same for every grammar; only the parser crate's
/// name and its grammar and parser types vary.
pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", to_snake_case(&grammar.name));
    let grammar_type = grammar_ident(&grammar.name);
    let parser = parser_ident(&grammar.name);
    quote! {
        use wasm_bindgen::prelude::*;

        use iguana_runtime::{
            arena::Arena,
            cli::ParseOutput,
            grammar::Grammar,
            input::Input,
            parse_tree::to_json,
        };
        use #grammar_name::{#grammar_type, #parser};

        #[doc = r" Parses `input` from the nonterminal named `start`, returning the runtime's"]
        #[doc = r" `ParseOutput` as JSON. A parse that succeeded returns the timings and the"]
        #[doc = r" parse-tree JSON; a parse that failed returns the error span and message."]
        #[doc = r" An unrecognized start nonterminal cannot run at all, so it returns an error."]
        #[wasm_bindgen]
        pub fn parse(input: &str, start: &str) -> Result<String, JsError> {
            let start_nonterminal_id = #grammar_type::start_nonterminal_id(start)
                .ok_or_else(|| JsError::new(&format!("unknown start nonterminal: {start}")))?;

            let input = Input::from(input);
            let tree_arena = Arena::new();
            let parser_arena = Arena::new();
            let parser = #parser::new(&input, &parser_arena);
            let envelope = match parser.parse(start_nonterminal_id, &tree_arena) {
                Ok(success) => ParseOutput {
                    error: None,
                    parse_ms: Some(success.parse_duration.as_millis() as u32),
                    tree_construction_ms: Some(success.tree_construction_duration.as_millis() as u32),
                    parse_tree: Some(to_json(success.tree, #grammar_type::LAYOUT_NAME)),
                },
                Err(error) => ParseOutput {
                    error: Some(error),
                    parse_ms: None,
                    tree_construction_ms: None,
                    parse_tree: None,
                },
            };
            Ok(serde_json::to_string(&envelope).unwrap())
        }
    }
}
