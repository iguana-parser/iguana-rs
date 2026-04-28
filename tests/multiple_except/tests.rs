// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/multiple_except/multiple_except.iggy --output tests/multiple_except
// To update golden files: REGENERATE=1 cargo test -p multiple_except

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use multiple_except::{parse_lexical_identifier, parse_syntax_identifier, parse_tree::to_sexpr};

// --- SyntaxIdentifier tests (nonterminal except path) ---

#[test]
fn test_syntax_identifier() {
    let input = Input::from("abc");
    let ctx = ParseContext::new();
    let result = parse_syntax_identifier(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(
        &actual,
        &golden_path(env!("CARGO_MANIFEST_DIR"), "syntax_identifier"),
    );
}

#[test]
fn test_syntax_keyword_rejected() {
    let input = Input::from("if");
    let ctx = ParseContext::new();
    assert!(
        parse_syntax_identifier(&input, &ctx).is_err(),
        "Expected parse to fail for input: if"
    );
}

#[test]
fn test_syntax_boolean_rejected() {
    let input = Input::from("true");
    let ctx = ParseContext::new();
    assert!(
        parse_syntax_identifier(&input, &ctx).is_err(),
        "Expected parse to fail for input: true"
    );
}

#[test]
fn test_syntax_null_rejected() {
    let input = Input::from("null");
    let ctx = ParseContext::new();
    assert!(
        parse_syntax_identifier(&input, &ctx).is_err(),
        "Expected parse to fail for input: null"
    );
}

#[test]
fn test_syntax_keyword_prefix_accepted() {
    let input = Input::from("ifx");
    let ctx = ParseContext::new();
    let result = parse_syntax_identifier(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(
        &actual,
        &golden_path(env!("CARGO_MANIFEST_DIR"), "syntax_keyword_prefix"),
    );
}

// --- LexicalIdentifier tests (lexical except path) ---

#[test]
fn test_lexical_identifier() {
    let input = Input::from("abc");
    let ctx = ParseContext::new();
    let result = parse_lexical_identifier(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(
        &actual,
        &golden_path(env!("CARGO_MANIFEST_DIR"), "lexical_identifier"),
    );
}

#[test]
fn test_lexical_keyword_rejected() {
    let input = Input::from("while");
    let ctx = ParseContext::new();
    assert!(
        parse_lexical_identifier(&input, &ctx).is_err(),
        "Expected parse to fail for input: while"
    );
}

#[test]
fn test_lexical_boolean_rejected() {
    let input = Input::from("false");
    let ctx = ParseContext::new();
    assert!(
        parse_lexical_identifier(&input, &ctx).is_err(),
        "Expected parse to fail for input: false"
    );
}

#[test]
fn test_lexical_null_rejected() {
    let input = Input::from("null");
    let ctx = ParseContext::new();
    assert!(
        parse_lexical_identifier(&input, &ctx).is_err(),
        "Expected parse to fail for input: null"
    );
}

#[test]
fn test_lexical_keyword_prefix_accepted() {
    let input = Input::from("forall");
    let ctx = ParseContext::new();
    let result = parse_lexical_identifier(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(
        &actual,
        &golden_path(env!("CARGO_MANIFEST_DIR"), "lexical_keyword_prefix"),
    );
}
