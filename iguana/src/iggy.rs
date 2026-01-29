use iggy::{
    parse_tree::{Grammar, IggyParseTreeBuilder, ParseTree, StartGrammar, create_parse_tree},
    parser::IggyParser,
};
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};

use crate::grammar::def::GrammarDef;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

/// Parse an iggy grammar source string and convert to Grammar.
pub fn parse_grammar(source: &str) -> Result<GrammarDef, Error> {
    let parse_tree = parse(source)?;
    build_grammar(&parse_tree)
}

/// Parse iggy source into a parse tree.
fn parse(source: &str) -> Result<StartGrammar, Error> {
    let start_nonterminal_name = "StartGrammar";
    let start_nonterminal_id = IggyParser::nonterminal_id(start_nonterminal_name).unwrap();
    let input = Input::from(source);
    let mut parser = IggyParser::new(&input, start_nonterminal_id);
    let result = parser.run();
    match result {
        ParseResult::Success(success) => {
            let parse_tree = create_parse_tree(
                success.sppf_node_id,
                start_nonterminal_name,
                &parser,
                &IggyParseTreeBuilder,
            );
            let ParseTree::StartGrammar(start_grammar) = parse_tree else {
                unreachable!()
            };
            Ok(start_grammar)
        }
        ParseResult::Failure() => Err(Error {
            message: "Parse error".into(),
        }),
    }
}

/// Convert an iggy parse tree to an iguana Grammar.
fn build_grammar(start_grammar: &StartGrammar) -> Result<GrammarDef, Error> {
    let grammar = start_grammar.grammar();
    todo!("build grammar from parse tree")
}
