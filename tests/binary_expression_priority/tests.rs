// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/binary_expression_priority/binary_expression_priority.iggy --output tests/binary_expression_priority
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests binary_expression_priority::

use binary_expression_priority::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/binary_expression_priority"
);

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_lit() {
    check("a", "lit");
}

#[test]
fn test_add() {
    check("a+a", "add");
}

#[test]
fn test_mul() {
    check("a*a", "mul");
}

#[test]
fn test_mul_add() {
    check("a*a+a", "mul_add");
}

#[test]
fn test_add_mul() {
    check("a+a*a", "add_mul");
}

#[test]
fn test_sub() {
    check("a-a", "sub");
}
