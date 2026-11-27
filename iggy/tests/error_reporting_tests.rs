use iggy::IggyParser;
use iguana_runtime::arena::Arena;
use iguana_runtime::input::Input;

const EXPECTED_LEVEL: &str =
    "Expected one of \"left\", \"right\", \"none\", \"(\", \"{\", \"i\", \"s\", Identifier, String";
const EXPECTED_ALTERNATIVE: &str = "Expected one of \"(\", \"{\", \"i\", \"s\", Identifier, String";

fn parse_error(source: &str) -> String {
    parse_error_at(source).1
}

/// Returns the source text from the error position onward, and the error message.
fn parse_error_at(source: &str) -> (&str, String) {
    let input = Input::from(source);
    let parser_arena = Arena::new();
    let tree_arena = Arena::new();
    match IggyParser::new(&input, &parser_arena).parse_grammar(&tree_arena) {
        Err(e) => (&source[e.span.left_extent as usize..], e.message),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn not_a_grammar() {
    assert_eq!(
        parse_error("a"),
        "Expected one of \"@IgnoreCase\", \"grammar\""
    );
}

#[test]
fn grammar_without_name() {
    assert_eq!(parse_error("grammar"), "Expected Identifier");
}

#[test]
fn grammar_with_trailing_space() {
    assert_eq!(parse_error("grammar "), "Expected Identifier");
}

#[test]
fn empty_rule_body() {
    assert_eq!(
        parse_error_at("grammar G\nA =\n"),
        ("", EXPECTED_LEVEL.to_string())
    );
}

#[test]
fn empty_rule_body_before_comment() {
    assert_eq!(
        parse_error_at("grammar G\nS = // nothing yet\n"),
        ("", EXPECTED_LEVEL.to_string())
    );
}

#[test]
fn empty_layout_rule_body() {
    assert_eq!(
        parse_error_at("grammar G\n@Layout A =\n"),
        ("", EXPECTED_LEVEL.to_string())
    );
}

#[test]
fn empty_priority_level_between_operators() {
    assert_eq!(
        parse_error_at("grammar G\nS = \"a\" > > \"b\"\n"),
        ("> \"b\"\n", EXPECTED_LEVEL.to_string())
    );
}

#[test]
fn double_greater_than() {
    assert_eq!(
        parse_error_at("grammar G\nS = \"a\" >> B\nB = \"b\"\n"),
        ("> B\nB = \"b\"\n", EXPECTED_LEVEL.to_string())
    );
}

#[test]
fn leading_priority_operator() {
    assert_eq!(
        parse_error_at("grammar G\nS = > \"a\"\n"),
        ("> \"a\"\n", EXPECTED_LEVEL.to_string())
    );
}

#[test]
fn trailing_priority_operator() {
    assert_eq!(
        parse_error_at("grammar G\nS = \"a\" >\n"),
        ("", EXPECTED_LEVEL.to_string())
    );
}

#[test]
fn associativity_without_alternatives() {
    assert_eq!(
        parse_error_at("grammar G\nS = left\n"),
        ("", EXPECTED_ALTERNATIVE.to_string())
    );
}

#[test]
fn trailing_associativity_without_alternatives() {
    assert_eq!(
        parse_error_at("grammar G\nS = \"a\" > left\n"),
        ("", EXPECTED_ALTERNATIVE.to_string())
    );
}
