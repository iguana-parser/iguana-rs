pub mod parse_tree;
pub mod parser;
pub mod scanner;
pub mod types;
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};
use parse_tree::{IggyParseTreeBuilder, ParseTree, create_parse_tree};
use parser::IggyParser;
use std::time::Duration;

pub struct ParseSuccess {
    pub tree: ParseTree,
    /// Time spent in the GLL parsing algorithm (from iguana-runtime).
    pub parse_duration: Duration,
    /// Time spent constructing the typed parse tree from the SPPF.
    pub tree_construction_duration: Duration,
}

pub fn parse(input: &Input, start_nonterminal: &str) -> Option<ParseSuccess> {
    let start_id = IggyParser::nonterminal_id(start_nonterminal)?;
    let mut parser = IggyParser::new(input, start_id);
    match parser.run() {
        ParseResult::Success(success) => {
            let parse_duration = success.duration;
            let tree_start = std::time::Instant::now();
            let tree = create_parse_tree(
                success.sppf_node_id,
                start_nonterminal,
                &parser,
                &IggyParseTreeBuilder,
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

