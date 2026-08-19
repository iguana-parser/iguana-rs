use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::{
        GenConfig,
        grammar_utils::{nonterminal_type, parse_tree_builder_ident, parser_ident},
    },
    grammar::{def::Grammar, symbols::Nonterminal},
    utils::to_snake_case,
};

pub fn generate(grammar: &Grammar, config: GenConfig) -> TokenStream {
    let grammar_name = &grammar.name;
    let parse_tree_builder = parse_tree_builder_ident(grammar_name);
    let parser = parser_ident(grammar_name);

    let parse_methods: Vec<TokenStream> = grammar
        .nonterminals()
        .filter(|nt| !nt.is_derived())
        .map(|nt| {
            let start_nt = grammar.start_nonterminal(nt);
            gen_parse_method(
                grammar,
                nt,
                start_nt,
                &parser,
                &parse_tree_builder,
                config.unsafe_mode,
            )
        })
        .collect();

    quote! {
        pub mod grammar_data;
        pub mod parser;
        pub mod parse_tree;
        pub mod scanner;
        pub mod types;

        use iguana_runtime::{
            arena::Arena,
            input::Input,
            parser::{GLLResult, Parser},
        };
        use parse_tree::*;
        use parser::#parser;

        pub use iguana_runtime::result::{ParseError, ParseSuccess};

        #(#parse_methods)*
    }
}

fn gen_parse_method(
    grammar: &Grammar,
    nt: &Nonterminal,
    start_nt: Option<&Nonterminal>,
    parser: &proc_macro2::Ident,
    parse_tree_builder: &proc_macro2::Ident,
    unsafe_mode: bool,
) -> TokenStream {
    let fn_name = format_ident!("parse_{}", to_snake_case(&nt.name));

    let target_nt = start_nt.unwrap_or(nt);
    let nt_const = format_ident!("{}", to_snake_case(&target_nt.name).to_uppercase());
    let create_fn = format_ident!("create_parse_tree_{}", to_snake_case(&target_nt.name));

    let return_type = nonterminal_type(grammar, target_nt, unsafe_mode);

    let parse_doc = format!(" Parses `input` starting from `{}`.", nt.name);

    quote! {
        #[doc = #parse_doc]
        #[doc = ""]
        #[doc = " `tree_arena` holds the constructed parse tree: the returned tree borrows it"]
        #[doc = " and lives until the arena is reset or dropped. Once the tree goes out of"]
        #[doc = " scope, the arena can be reset with `tree_arena.reset()` and reused for the"]
        #[doc = " next parse; that is the pattern for repeated parsing, as in an editor or a"]
        #[doc = " benchmark loop."]
        pub fn #fn_name<'a>(input: &Input, tree_arena: &'a Arena) -> std::result::Result<ParseSuccess<&'a #return_type>, ParseError> {
            let vec_arena = Arena::new();
            let mut parser = #parser::new(input, grammar_data::#nt_const, &vec_arena);
            match parser.run() {
                GLLResult::Success(success) => {
                    let parse_duration = success.duration;
                    let tree_start = iguana_runtime::Instant::now();
                    let parse_tree_builder = #parse_tree_builder::new(tree_arena);
                    let tree = parse_tree::#create_fn(success.sppf_node_id, &parser, &parse_tree_builder);
                    let tree_construction_duration = tree_start.elapsed();
                    let ambiguity_node_added = parser.ambiguity_node_added();
                    Ok(ParseSuccess { tree, parse_duration, tree_construction_duration, ambiguity_node_added })
                }
                GLLResult::Failure(error) => Err(parser.to_parse_error(&error)),
            }
        }
    }
}
