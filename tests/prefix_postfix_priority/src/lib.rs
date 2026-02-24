pub mod parse_tree;
pub mod parser;
pub mod scanner;
pub mod types;
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};
use parse_tree::{ParseTree, PrefixPostfixPriorityParseTreeBuilder, create_parse_tree};
use parser::PrefixPostfixPriorityParser;
pub fn parse(source: &str, start_nonterminal: &str) -> Option<ParseTree> {
    let input = Input::from(source);
    let start_id = PrefixPostfixPriorityParser::nonterminal_id(start_nonterminal)?;
    let mut parser = PrefixPostfixPriorityParser::new(&input, start_id);
    match parser.run() {
        ParseResult::Success(success) => Some(create_parse_tree(
            success.sppf_node_id,
            start_nonterminal,
            &parser,
            &PrefixPostfixPriorityParseTreeBuilder,
        )),
        ParseResult::Failure() => None,
    }
}

