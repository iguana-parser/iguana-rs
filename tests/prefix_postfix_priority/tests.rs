// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/prefix_postfix_priority/prefix_postfix_priority.iggy --output tests/prefix_postfix_priority
// To update golden files: REGENERATE=1 cargo test -p prefix_postfix_priority

use prefix_postfix_priority::{parse_s, parse_tree::to_sexpr};
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
