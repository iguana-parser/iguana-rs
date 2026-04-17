// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/expression/expression.iggy --output tests/expression
// To update golden files: REGENERATE=1 cargo test -p expression

use expression::{parse_e, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_e(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
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
