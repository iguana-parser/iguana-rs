// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/except_terminal/except_terminal.iggy --output tests/except_terminal
// To update golden files: REGENERATE=1 cargo test -p except_terminal

use except_terminal::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str) {
    let input_str = input;
    let input = Input::from(input);
    let ctx = ParseContext::new();
    assert!(parse_s(&input, &ctx).is_err(), "Expected parse to fail for input: {input_str}");
}

#[test]
fn test_identifier() {
    check("abc", "identifier");
}

#[test]
fn test_keyword_if_rejected() {
    check_fails("if");
}

#[test]
fn test_keyword_else_rejected() {
    check_fails("else");
}

#[test]
fn test_keyword_while_rejected() {
    check_fails("while");
}

#[test]
fn test_keyword_prefix_accepted() {
    check("ifx", "keyword_prefix");
}

#[test]
fn test_single_char() {
    check("x", "single_char");
}
