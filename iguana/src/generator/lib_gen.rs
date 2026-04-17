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
        use iguana_runtime::{
            input::Input,
            parser::{ParseResult, Parser},
        };
        use parse_tree::{ParseTree, #parse_tree_builder, create_parse_tree};
        use parser::#parser;

        #[derive(Debug)]
        pub struct ParseError {
            pub line: u32,
            pub column: u32,
            pub message: String,
        }

        impl Display for ParseError {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.message)
            }
        }

        impl Error for ParseError {}

        fn to_parse_error(input: &Input, error: &iguana_runtime::parser::ParseError) -> ParseError {
            if error.input_index >= input.len() {
                ParseError {
                    line: 0,
                    column: 0,
                    message: "Unexpected end of input".to_string(),
                }
            } else {
                let (line, column) = input.line_column(error.input_index);
                ParseError {
                    line,
                    column,
                    message: format!("Parse error at line {line}, column {column}"),
                }
            }
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

    let (return_type, nt_const, nt_name, variant) = if let Some(start_nt) = start_nt {
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
            name.clone(),
            format_ident!("{}", to_pascal_case(name)),
        )
    } else {
        let name = &nt.name;
        let nt_type = format_ident!("{}", to_pascal_case(name));
        (
            quote! { parse_tree::#nt_type },
            format_ident!("{}", to_snake_case(name).to_uppercase()),
            name.clone(),
            nt_type,
        )
    };

    quote! {
        pub fn #fn_name(input: &Input) -> Result<#return_type, ParseError> {
            let mut parser = #parser::new(input, grammar_data::#nt_const);
            match parser.run() {
                ParseResult::Success(success) => {
                    let tree = create_parse_tree(
                        success.sppf_node_id,
                        #nt_name,
                        &parser,
                        &#parse_tree_builder,
                    );
                    let ParseTree::#variant(node) = tree else { unreachable!() };
                    Ok(node)
                }
                ParseResult::Failure(error) => Err(to_parse_error(input, &error)),
            }
        }
    }
}
