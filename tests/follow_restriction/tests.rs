// To regenerate parser:  cargo run -p iguana -- test gen follow_restriction
// To update golden files: REGENERATE=1 cargo test -p follow_restriction

use follow_restriction::{parse_s, parse_t, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

// Nonterminal case: Char+ !>> Char (pop path)

#[test]
fn single_id() {
    let input = Input::from("abc");
    let result = parse_s(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), "single_id"));
}

#[test]
fn two_ids() {
    let input = Input::from("abc def");
    let result = parse_s(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), "two_ids"));
}

// Terminal case: Char !>> Char (slot path)

#[test]
fn single_char() {
    let input = Input::from("a");
    let result = parse_t(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), "single_char"));
}

#[test]
fn rejects_char_followed_by_char() {
    let input = Input::from("ab");
    assert!(parse_t(&input).is_err(), "Expected parse to fail for input: ab");
}
