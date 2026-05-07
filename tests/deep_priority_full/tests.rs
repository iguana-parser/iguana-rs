// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/deep_priority_full/deep_priority_full.iggy --output tests/deep_priority_full
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests deep_priority_full::

use deep_priority_full::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/deep_priority_full");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

// if a then a else (a + a)
#[test]
fn test_if_then_else_add() {
    check("if a then a else a + a", "if_then_else_add");
}

// -(a + a): prefix '-' is above '+'
#[test]
fn test_neg_a_add_a() {
    check("-a+a", "neg_a_add_a");
}

// -(a * a): '-' is below '*'
#[test]
fn test_neg_a_mul_a() {
    check("-a*a", "neg_a_mul_a");
}

// a ; (a ; a): right-associative
#[test]
fn test_right_assoc_seq() {
    check("a;a;a", "right_assoc_seq");
}

// (a + a) + a: left-associative
#[test]
fn test_left_assoc_add() {
    check("a+a+a", "left_assoc_add");
}

// a + (if a then a else (a * a))
#[test]
fn test_add_if_then_else_mul() {
    check("a + if a then a else a * a", "add_if_then_else_mul");
}
