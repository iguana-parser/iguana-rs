use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::utils::{to_first_uppercase, to_pascal_case, to_snake_case},
    grammar::{
        def::Grammar,
        symbols::Nonterminal,
    },
};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = &grammar.name;
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(grammar_name));
    let parser = format_ident!("{}Parser", to_first_uppercase(grammar_name));

    let parse_methods: Vec<TokenStream> = grammar
        .nonterminals()
        .filter(|nt| !nt.is_derived() && nt.parameters.is_empty())
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
            parser::{ParseResult, Parser},
        };
        use parse_tree::#parse_tree_builder;
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

        impl<T: parse_tree::AsParseTreeRef> ParseSuccess<T> {
            pub fn as_parse_tree_ref(&self) -> parse_tree::ParseTreeRef<'_> {
                self.tree.as_parse_tree_ref()
            }
        }

        fn to_parse_error<'i, P: Parser<'i>>(input: &Input, error: &iguana_runtime::parser::ParseError) -> ParseError {
            use iguana_runtime::parser::ParseErrorKind;

            let (line, column) = input.line_column(error.input_index);

            let found = if error.input_index >= input.len() {
                "EOF".to_string()
            } else {
                let ch = input.char_at(error.input_index).unwrap();
                format!("'{ch}'")
            };

            let message = match &error.kind {
                ParseErrorKind::UnexpectedToken { expected } => {
                    let names: Vec<&str> = expected.iter()
                        .map(|id| P::terminal_name(*id))
                        .collect();
                    match names.len() {
                        0 => format!("Unexpected {found}"),
                        1 => format!("Expected {} but found {found}", names[0]),
                        _ => format!("Expected one of {} but found {found}", names.join(", ")),
                    }
                }
                ParseErrorKind::ExcludedMatch { excluded_by } => {
                    let names: Vec<&str> = excluded_by.iter()
                        .map(|id| P::terminal_name(*id))
                        .collect();
                    format!("Match excluded by {}", names.join(", "))
                }
                ParseErrorKind::ForbiddenFollow { forbidden } => {
                    let names: Vec<&str> = forbidden.iter()
                        .map(|id| P::terminal_name(*id))
                        .collect();
                    format!("Forbidden follow: {}", names.join(", "))
                }
            };

            ParseError { line, column, message }
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

    let (return_type, nt_const, create_fn) = if let Some(start_nt) = start_nt {
        let inner_type = format_ident!("{}", to_pascal_case(&nt.name));
        // Safe: start wrappers are only created when layout is defined
        let layout_ident = grammar.layout.as_ref().unwrap().as_identifier().unwrap();
        let layout = if grammar.is_terminal(layout_ident) {
            format_ident!("Token")
        } else {
            format_ident!("{}", to_pascal_case(&layout_ident.name))
        };
        let name = &start_nt.name;
        (
            quote! { parse_tree::Start<parse_tree::#inner_type, parse_tree::#layout> },
            format_ident!("{}", to_snake_case(name).to_uppercase()),
            format_ident!("create_parse_tree_{}", to_snake_case(name)),
        )
    } else {
        let name = &nt.name;
        let nt_type = format_ident!("{}", to_pascal_case(name));
        (
            quote! { parse_tree::#nt_type },
            format_ident!("{}", to_snake_case(name).to_uppercase()),
            format_ident!("create_parse_tree_{}", to_snake_case(name)),
        )
    };

    quote! {
        pub fn #fn_name(input: &Input) -> Result<ParseSuccess<#return_type>, ParseError> {
            let mut parser = #parser::new(input, grammar_data::#nt_const);
            match parser.run() {
                ParseResult::Success(success) => {
                    let parse_duration = success.duration;
                    let tree_start = std::time::Instant::now();
                    let tree = parse_tree::#create_fn(success.sppf_node_id, &parser, &#parse_tree_builder);
                    let tree_construction_duration = tree_start.elapsed();
                    Ok(ParseSuccess { tree, parse_duration, tree_construction_duration })
                }
                ParseResult::Failure(error) => Err(to_parse_error::<#parser>(input, &error)),
            }
        }
    }
}
