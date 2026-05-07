// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/expression/expression.iggy --output tests/expression
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests expression::

use expression::{parse_e, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/expression");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_e(&input, &ctx).expect("Parse failed");
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
