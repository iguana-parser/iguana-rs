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

        use std::time::Duration;
        use iguana_runtime::{
            input::Input,
            parser::{ParseResult, Parser},
        };
        use parse_tree::{ParseTree, #parse_tree_builder, create_parse_tree};
        use parser::#parser;

        pub struct ParseSuccess {
            pub tree: ParseTree,
            pub parse_duration: Duration,
            pub tree_construction_duration: Duration,
        }

        pub fn parse(input: &Input, start_nonterminal: &str) -> Option<ParseSuccess> {
            let start_id = #parser::nonterminal_id(start_nonterminal)?;
            let mut parser = #parser::new(input, start_id);
            match parser.run() {
                ParseResult::Success(success) => {
                    let parse_duration = success.duration;
                    let tree_start = std::time::Instant::now();
                    let tree = create_parse_tree(
                        success.sppf_node_id,
                        start_nonterminal,
                        &parser,
                        &#parse_tree_builder,
                    );
                    let tree_construction_duration = tree_start.elapsed();
                    Some(ParseSuccess {
                        tree,
                        parse_duration,
                        tree_construction_duration,
                    })
                }
                ParseResult::Failure() => None,
            }
        }
    }
}
