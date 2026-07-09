use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::Bump;

fn parse_error(source: &str) -> String {
    let input = Input::from(source);
    let tree_arena = Bump::new();
    match iggy::parse_grammar(&input, &tree_arena) {
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
