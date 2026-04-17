use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{generator::utils::to_first_uppercase, grammar::def::Grammar};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = &grammar.name;
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(grammar_name));
    let parser = format_ident!("{}Parser", to_first_uppercase(grammar_name));

    quote! {
        pub mod grammar_data;
        pub mod parser;
        pub mod parse_tree;
        pub mod scanner;
        pub mod types;

        use std::time::Duration;
        use iguana_runtime::{
            ids::NonterminalId,
            input::Input,
            parser::{ParseResult, Parser},
        };
        use grammar_data::NONTERMINALS;
        use parse_tree::{ParseTree, #parse_tree_builder, create_parse_tree};
        use parser::#parser;

        pub struct ParseSuccess {
            pub tree: ParseTree,
            pub parse_duration: Duration,
            pub tree_construction_duration: Duration,
        }

        pub fn parse(input: &Input, start_nonterminal: NonterminalId) -> Option<ParseSuccess> {
            let mut parser = #parser::new(input, start_nonterminal);
            match parser.run() {
                ParseResult::Success(success) => {
                    let parse_duration = success.duration;
                    let tree_start = std::time::Instant::now();
                    let name = NONTERMINALS[start_nonterminal.index()].name;
                    let tree = create_parse_tree(
                        success.sppf_node_id,
                        name,
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
                ParseResult::Failure(_) => None,
            }
        }
    }
}
