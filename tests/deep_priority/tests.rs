// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/deep_priority/deep_priority.iggy --output tests/deep_priority
// To update golden files: REGENERATE=1 cargo test -p deep_priority

use deep_priority::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_s(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_a() {
    check("a", "a");
}

#[test]
fn test_if_then_else() {
    check("if a then a else a", "if_then_else");
}

// if a then a else (a + a): '+' binds inside the else-branch
#[test]
fn test_if_then_else_add() {
    check("if a then a else a + a", "if_then_else_add");
}

// a + (if a then a else a): if-then-else as right operand of '+'
#[test]
fn test_add_if_then_else() {
    check("a + if a then a else a", "add_if_then_else");
}
