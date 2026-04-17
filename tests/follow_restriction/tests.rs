// To regenerate parser:  cargo run -p iguana -- test gen follow_restriction
// To update golden files: REGENERATE=1 cargo test -p follow_restriction

use follow_restriction::{grammar_data, parse, parse_tree::to_sexpr};
use iguana_runtime::ids::NonterminalId;
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, start: NonterminalId, test_name: &str) {
    let input = Input::from(input);
    let result = parse(&input, start).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str, start: NonterminalId) {
    let input_str = input;
    let input = Input::from(input);
    assert!(parse(&input, start).is_none(), "Expected parse to fail for input: {input_str}");
}

// Nonterminal case: Char+ !>> Char (pop path)

#[test]
fn single_id() {
    check("abc", grammar_data::S, "single_id");
}

#[test]
fn two_ids() {
    check("abc def", grammar_data::S, "two_ids");
}

// Terminal case: Char !>> Char (slot path)

#[test]
fn single_char() {
    check("a", grammar_data::T, "single_char");
}

#[test]
fn rejects_char_followed_by_char() {
    check_fails("ab", grammar_data::T);
}
