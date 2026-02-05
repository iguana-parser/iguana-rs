pub mod parse_tree;
pub mod parser;
pub mod scanner;
pub mod types;
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};
use parse_tree::{ParseTree, PlusParseTreeBuilder, create_parse_tree};
use parser::PlusParser;
pub fn parse(source: &str, start_nonterminal: &str) -> Option<ParseTree> {
    let input = Input::from(source);
    let start_id = PlusParser::nonterminal_id(start_nonterminal)?;
    let mut parser = PlusParser::new(&input, start_id);
    match parser.run() {
        ParseResult::Success(success) => Some(create_parse_tree(
            success.sppf_node_id,
            start_nonterminal,
            &parser,
            &PlusParseTreeBuilder,
        )),
        ParseResult::Failure() => None,
    }
}

