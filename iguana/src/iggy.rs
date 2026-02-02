use iggy::{
    parse_tree::{
        Alternative as PtAlternative, CharClass as PtCharClass, CharClassAlt0, CharClassOpt7,
        CharClassPlus7, Grammar as PtGrammar, GrammarOpt1, IggyParseTreeBuilder, ParseTree,
        PriorityLevel as PtPriorityLevel, Range as PtRange, Regex as PtRegex, RegexBlock,
        RegexRule as PtRegexRule, RegexRulePlus3, RegexRulePlus4, StartGrammar,
        Symbol as PtSymbol, SyntaxRule as PtSyntaxRule, create_parse_tree,
    },
    parser::IggyParser,
};
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
    sppf::Span,
};

use crate::grammar::{
    def::{Alternative, GrammarDef, LexicalRule, PriorityLevel, SyntaxRule},
    regex::{CharClass, CharRange, Regex},
    symbols::{Identifier, Nonterminal, Symbol, Terminal},
};

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

pub fn parse_grammar(source: &str) -> Result<GrammarDef, Error> {
    let input = Input::from(source);
    let parse_tree = parse(&input)?;
    build_grammar(&parse_tree, &input)
}

fn parse(input: &Input) -> Result<StartGrammar, Error> {
    let start_nonterminal_name = "StartGrammar";
    let start_nonterminal_id = IggyParser::nonterminal_id(start_nonterminal_name).unwrap();
    let mut parser = IggyParser::new(input, start_nonterminal_id);
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

fn build_grammar(start_grammar: &StartGrammar, input: &Input) -> Result<GrammarDef, Error> {
    // TODO: implement parse tree traversal once fields are accessible
    todo!("build grammar from parse tree")
}
