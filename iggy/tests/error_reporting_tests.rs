use iggy::IggyParser;
use iguana_runtime::arena::Arena;
use iguana_runtime::input::Input;

fn parse_error(source: &str) -> String {
    let input = Input::from(source);
    let parser_arena = Arena::new();
    let tree_arena = Arena::new();
    match IggyParser::new(&input, &parser_arena).parse_grammar(&tree_arena) {
        Err(e) => e.message,
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn not_a_grammar() {
    assert_eq!(parse_error("a"), "Expected \"grammar\"");
}

#[test]
fn grammar_without_name() {
    assert_eq!(parse_error("grammar"), "Expected Identifier");
}

#[test]
fn grammar_with_trailing_space() {
    assert_eq!(parse_error("grammar "), "Expected Identifier");
}
