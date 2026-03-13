// To regenerate parser:  cargo run -p iguana -- test gen follow_restriction
// To update golden files: REGENERATE=1 cargo test -p follow_restriction

use follow_restriction::{parse, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, start: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse(&input, start).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str, start: &str) {
    let input_str = input;
    let input = Input::from(input);
    assert!(parse(&input, start).is_none(), "Expected parse to fail for input: {input_str}");
}

// Nonterminal case: Char+ !>> Char (pop path)

#[test]
fn single_id() {
    check("abc", "S", "single_id");
}

#[test]
fn two_ids() {
    check("abc def", "S", "two_ids");
}

// Terminal case: Char !>> Char (slot path)

#[test]
fn single_char() {
    check("a", "T", "single_char");
}

#[test]
fn rejects_char_followed_by_char() {
    check_fails("ab", "T");
}
