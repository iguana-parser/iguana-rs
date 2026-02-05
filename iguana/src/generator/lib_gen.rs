use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{generator::utils::to_first_uppercase, grammar::def::Grammar};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = &grammar.name;
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(grammar_name));
    let parser = format_ident!("{}Parser", to_first_uppercase(grammar_name));

    quote! {
        pub mod parser;
        pub mod parse_tree;
        pub mod scanner;
        pub mod types;

        use iguana_runtime::{
            input::Input,
            parser::{ParseResult, Parser},
        };
        use parse_tree::{ParseTree, #parse_tree_builder, create_parse_tree};
        use parser::#parser;

        pub fn parse(source: &str, start_nonterminal: &str) -> Option<ParseTree> {
            let input = Input::from(source);
            let start_id = #parser::nonterminal_id(start_nonterminal)?;
            let mut parser = #parser::new(&input, start_id);
            match parser.run() {
                ParseResult::Success(success) => {
                    Some(create_parse_tree(
                        success.sppf_node_id,
                        start_nonterminal,
                        &parser,
                        &#parse_tree_builder,
                    ))
                }
                ParseResult::Failure() => None,
            }
        }
    }
}
