// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/prefix_postfix_priority/prefix_postfix_priority.iggy --output tests/prefix_postfix_priority
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests prefix_postfix_priority::

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use prefix_postfix_priority::{parse_s, parse_tree::to_sexpr};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/prefix_postfix_priority");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_a() {
    check("a", "a");
}

#[test]
fn test_neg_a() {
    check("-a", "neg_a");
}

#[test]
fn test_a_bang() {
    check("a!", "a_bang");
}

#[test]
fn test_a_mul_a_add_a() {
    check("a*a+a", "a_mul_a_add_a");
}

#[test]
fn test_neg_a_add_a() {
    check("-a+a", "neg_a_add_a");
}

#[test]
fn test_a_bang_add_a() {
    check("a!+a", "a_bang_add_a");
}

#[test]
fn test_neg_a_bang() {
    check("-a!", "neg_a_bang");
}
