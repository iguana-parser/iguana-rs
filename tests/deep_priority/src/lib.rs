pub mod parse_tree;
pub mod parser;
pub mod scanner;
pub mod types;
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};
use parse_tree::{DeepPriorityParseTreeBuilder, ParseTree, create_parse_tree};
use parser::DeepPriorityParser;
pub fn parse(input: &Input, start_nonterminal: &str) -> Option<ParseTree> {
    let start_id = DeepPriorityParser::nonterminal_id(start_nonterminal)?;
    let mut parser = DeepPriorityParser::new(input, start_id);
    match parser.run() {
        ParseResult::Success(success) => Some(create_parse_tree(
            success.sppf_node_id,
            start_nonterminal,
            &parser,
            &DeepPriorityParseTreeBuilder,
        )),
        ParseResult::Failure() => None,
    }
}

