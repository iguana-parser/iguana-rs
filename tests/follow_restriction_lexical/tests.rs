// To regenerate parser:  cargo xtask test-gen follow_restriction_lexical
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests follow_restriction_lexical::

use follow_restriction_lexical::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/follow_restriction_lexical");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

fn check_fails(input: &str) {
    let input_str = input;
    let input = Input::from(input);
    let ctx = ParseContext::new();
    assert!(
        parse_s(&input, &ctx).is_err(),
        "Expected parse to fail for input: {input_str}"
    );
}

#[test]
fn num_then_id() {
    // "123 abc" — Num not followed by Alpha (space separates), then Id
    check("123 abc", "num_then_id");
}

#[test]
fn rejects_num_followed_by_alpha() {
    // "123abc" — Num 123 is immediately followed by Alpha, rejected by !>>
    check_fails("123abc");
}
