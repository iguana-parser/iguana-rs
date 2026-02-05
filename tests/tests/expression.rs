use expression::{
    parse_tree::{create_parse_tree, to_sexpr, ExpressionParseTreeBuilder, ParseTree},
    parser::ExpressionParser,
};
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
};
use std::path::PathBuf;

fn parse(source: &str) -> Option<ParseTree> {
    let input = Input::from(source);
    let start_nonterminal_name = "StartE";
    let start_nonterminal_id = ExpressionParser::nonterminal_id(start_nonterminal_name)?;
    let mut parser = ExpressionParser::new(&input, start_nonterminal_id);
    let result = parser.run();
    match result {
        ParseResult::Success(success) => {
            let parse_tree = create_parse_tree(
                success.sppf_node_id,
                start_nonterminal_name,
                &parser,
                &ExpressionParseTreeBuilder,
            );
            Some(parse_tree)
        }
        ParseResult::Failure() => None,
    }
}

fn golden_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/grammars/expression/parse_trees");
    path.push(format!("{}.txt", name));
    path
}

fn check(source: &str, name: &str) {
    let parse_tree = parse(source).expect("Parse failed");
    let actual = to_sexpr(parse_tree.as_parse_tree_ref());
    iguana_tests::check_parse_tree(&actual, &golden_path(name));
}

#[test]
fn test_lit() {
    check("a", "lit");
}

#[test]
fn test_add() {
    check("a+a", "add");
}

#[test]
fn test_mul() {
    check("a*a", "mul");
}

// NOTE: More complex expressions like a+a+a, a+a*a, a*a+a are ambiguous
// because the grammar lacks priority rules. They produce multiple parse trees
// which isn't yet supported by the parse tree builder.
