pub mod parse_tree;
pub mod parser;
pub mod scanner;
pub mod types;
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};
use parse_tree::{MultipleExceptParseTreeBuilder, ParseTree, create_parse_tree};
use parser::MultipleExceptParser;
pub fn parse(input: &Input, start_nonterminal: &str) -> Option<ParseTree> {
    let start_id = MultipleExceptParser::nonterminal_id(start_nonterminal)?;
    let mut parser = MultipleExceptParser::new(input, start_id);
    match parser.run() {
        ParseResult::Success(success) => Some(create_parse_tree(
            success.sppf_node_id,
            start_nonterminal,
            &parser,
            &MultipleExceptParseTreeBuilder,
        )),
        ParseResult::Failure() => None,
    }
}

