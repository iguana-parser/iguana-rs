use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::grammar_utils::nonterminal_type,
    grammar::{def::Grammar, symbols::Nonterminal},
    utils::{to_first_uppercase, to_snake_case},
};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = &grammar.name;
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(grammar_name));
    let parser = format_ident!("{}Parser", to_first_uppercase(grammar_name));

    let parse_methods: Vec<TokenStream> = grammar
        .nonterminals()
        .filter(|nt| !nt.is_derived())
        .map(|nt| {
            let start_nt = grammar.start_nonterminal(nt);
            gen_parse_method(grammar, nt, start_nt, &parser, &parse_tree_builder)
        })
        .collect();

    quote! {
        pub mod grammar_data;
        pub mod parser;
        pub mod parse_tree;
        pub mod scanner;
        pub mod types;

        use std::error::Error;
        use std::fmt::{self, Display, Formatter};
        use std::time::Duration;
        use iguana_runtime::{
            input::Input,
            parse_tree::ParseContext,
            parser::{ParseResult, Parser},
        };
        use parse_tree::*;
        use parser::#parser;

        #[derive(Debug)]
        pub struct ParseError {
            pub line: u32,
            pub column: u32,
            pub message: String,
        }

        impl Display for ParseError {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
            }
        }

        impl Error for ParseError {}

        pub struct ParseSuccess<T> {
            pub tree: T,
            pub parse_duration: Duration,
            pub tree_construction_duration: Duration,
        }

        #(#parse_methods)*
    }
}

fn gen_parse_method(
    grammar: &Grammar,
    nt: &Nonterminal,
    start_nt: Option<&Nonterminal>,
    parser: &proc_macro2::Ident,
    parse_tree_builder: &proc_macro2::Ident,
) -> TokenStream {
    let fn_name = format_ident!("parse_{}", to_snake_case(&nt.name));

    let target_nt = start_nt.unwrap_or(nt);
    let nt_const = format_ident!("{}", to_snake_case(&target_nt.name).to_uppercase());
    let create_fn = format_ident!("create_parse_tree_{}", to_snake_case(&target_nt.name));

    let return_type = nonterminal_type(grammar, target_nt);

    quote! {
        pub fn #fn_name<'a>(input: &Input, ctx: &'a ParseContext) -> std::result::Result<ParseSuccess<&'a #return_type>, ParseError> {
            let mut parser = #parser::new(input, grammar_data::#nt_const);
            match parser.run() {
                ParseResult::Success(success) => {
                    let parse_duration = success.duration;
                    let tree_start = std::time::Instant::now();
                    let parse_tree_builder = #parse_tree_builder::new(ctx);
                    let tree = parse_tree::#create_fn(success.sppf_node_id, &parser, &parse_tree_builder);
                    let tree_construction_duration = tree_start.elapsed();
                    Ok(ParseSuccess { tree, parse_duration, tree_construction_duration })
                }
                ParseResult::Failure(error) => {
                    let (line, column, message) = parser.format_error(&error);
                    Err(ParseError { line, column, message })
                }
            }
        }
    }
}
