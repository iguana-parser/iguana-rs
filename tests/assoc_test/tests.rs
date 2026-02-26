// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/assoc_test/assoc_test.iggy --output tests/assoc_test
// To update golden files: REGENERATE=1 cargo test -p assoc_test

use assoc_test::{parse, parse_tree::to_sexpr};
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let tree = parse(input, "S").expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str) {
    assert!(parse(input, "S").is_none(), "Expected parse to fail for input: {input}");
}

// Left associativity: E '+' E and E '-' E

#[test]
fn test_left_add() {
    check("a+a", "left_add");
}

#[test]
fn test_left_add_chain() {
    check("a+a+a", "left_add_chain");
}

#[test]
fn test_left_sub_chain() {
    check("a-a-a", "left_sub_chain");
}

#[test]
fn test_left_add_sub() {
    check("a+a-a", "left_add_sub");
}

#[test]
fn test_left_long_chain() {
    check("a+a+a-a-a-a", "left_long_chain");
}

// Right associativity: E ';' E

#[test]
fn test_right_seq() {
    check("a;a", "right_seq");
}

#[test]
fn test_right_seq_chain() {
    check("a;a;a", "right_seq_chain");
}

// Non-associativity: E '<' E

#[test]
fn test_non_assoc_single() {
    check("a<a", "non_assoc_single");
}

#[test]
fn test_non_assoc_chain_fails() {
    check_fails("a<a<a");
}
