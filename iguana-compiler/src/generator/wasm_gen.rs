use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::grammar_utils::{grammar_ident, parser_ident, scanner_ident},
    grammar::def::Grammar,
    utils::to_snake_case,
};

/// Generate the `lib.rs` of the wasm wrapper crate: a `wasm-bindgen` entry
/// point that runs the generated parser and returns the result as a JSON
/// envelope. The body is the same for every grammar; only the names of the
/// parser crate and its types, and the highlighting, vary.
///
/// `highlighting` is whether the parser crate has the module `highlighting`
/// (`lib_gen`). `match_memo` is whether the scanner takes an arena for its
/// memo.
pub fn generate(grammar: &Grammar, highlighting: bool, match_memo: bool) -> TokenStream {
    let grammar_name = format_ident!("{}", to_snake_case(&grammar.name));
    let grammar_type = grammar_ident(&grammar.name);
    let parser = parser_ident(&grammar.name);
    let scanner = scanner_ident(&grammar.name);
    let new_scanner = if match_memo {
        quote! {
            let scanner_arena = Arena::new();
            let mut scanner = #scanner::new(&input, &scanner_arena);
        }
    } else {
        quote! { let mut scanner = #scanner::new(&input); }
    };
    // Without rules, the wrapper returns an empty highlighting and does not
    // walk the tree or scan the input.
    let (highlighting_imports, tree_highlights, token_highlights) = if highlighting {
        (
            quote! { use #grammar_name::{parse_tree::Token, scanner::#scanner}; },
            quote! {
                highlight.then(|| {
                    iguana_runtime::highlight::highlight(
                        success.tree,
                        &input,
                        #grammar_name::highlighting::rules,
                    )
                })
            },
            quote! {
                highlight.then(|| {
                    #new_scanner
                    iguana_runtime::highlight::lexical_highlight(
                        &mut scanner,
                        &input,
                        Token::new,
                        #grammar_name::highlighting::token_rules,
                    )
                })
            },
        )
    } else {
        let empty = quote! {
            highlight.then(Vec::<iguana_runtime::highlight::Highlight<&'static str>>::new)
        };
        (quote! {}, empty.clone(), empty)
    };
    quote! {
        use wasm_bindgen::prelude::*;

        use iguana_runtime::{
            arena::Arena,
            grammar::Grammar,
            input::Input,
            parse_tree::to_json,
            parser::Parser,
            result::ParseSummary,
            wasm::WasmParseOutput,
        };
        use #grammar_name::{#grammar_type, #parser};
        #highlighting_imports

        #[doc = r" Parses `input` from the nonterminal named `start`, returning the runtime's"]
        #[doc = r" `WasmParseOutput` as JSON. A parse that succeeded returns the timings and"]
        #[doc = r" the parse-tree JSON, and a parse that failed returns the error span and"]
        #[doc = r" message. With `highlight`, the output also has the highlighting of the"]
        #[doc = r" tree, or of the scanner's tokens when the parse failed. An unrecognized"]
        #[doc = r" start nonterminal cannot run at all, so it returns an error."]
        #[wasm_bindgen]
        pub fn parse(input: &str, start: &str, highlight: bool) -> Result<String, JsError> {
            let start_nonterminal_id = #grammar_type::start_nonterminal_id(start)
                .ok_or_else(|| JsError::new(&format!("unknown start nonterminal: {start}")))?;

            let input = Input::from(input);
            let tree_arena = Arena::new();
            let parser_arena = Arena::new();
            let parser = #parser::new(&input, &parser_arena);
            let result = Parser::parse(parser, start_nonterminal_id, &tree_arena);
            let summary = ParseSummary::from(&result);
            let output = match result {
                Ok(success) => WasmParseOutput {
                    summary,
                    parse_tree: Some(to_json(success.tree, #grammar_type::LAYOUT_NAME)),
                    highlights: #tree_highlights,
                },
                Err(_) => WasmParseOutput {
                    summary,
                    parse_tree: None,
                    highlights: #token_highlights,
                },
            };
            Ok(serde_json::to_string(&output).unwrap())
        }
    }
}
