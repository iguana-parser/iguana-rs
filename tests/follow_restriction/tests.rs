// To regenerate parser:  cargo xtask test-gen follow_restriction
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests follow_restriction::

use follow_restriction::{parse_s, parse_t, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

// Nonterminal case: Char+ !>> Char (pop path)

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/follow_restriction");

#[test]
fn single_id() {
    let input = Input::from("abc");
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, "single_id"));
}

#[test]
fn two_ids() {
    let input = Input::from("abc def");
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, "two_ids"));
}

// Terminal case: Char !>> Char (slot path)

#[test]
fn single_char() {
    let input = Input::from("a");
    let ctx = ParseContext::new();
    let result = parse_t(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, "single_char"));
}

#[test]
fn rejects_char_followed_by_char() {
    let input = Input::from("ab");
    let ctx = ParseContext::new();
    assert!(
        parse_t(&input, &ctx).is_err(),
        "Expected parse to fail for input: ab"
    );
}
