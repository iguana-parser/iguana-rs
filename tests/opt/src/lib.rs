pub mod parse_tree;
pub mod parser;
pub mod scanner;
pub mod types;
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};
use parse_tree::{OptParseTreeBuilder, ParseTree, create_parse_tree};
use parser::OptParser;
pub fn parse(input: &Input, start_nonterminal: &str) -> Option<ParseTree> {
    let start_id = OptParser::nonterminal_id(start_nonterminal)?;
    let mut parser = OptParser::new(input, start_id);
    match parser.run() {
        ParseResult::Success(success) => Some(create_parse_tree(
            success.sppf_node_id,
            start_nonterminal,
            &parser,
            &OptParseTreeBuilder,
        )),
        ParseResult::Failure() => None,
    }
}

