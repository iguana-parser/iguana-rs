// To regenerate parser:  cargo run -p iguana -- test gen follow_restriction_multiple
// To update golden files: REGENERATE=1 cargo test -p follow_restriction_multiple

use follow_restriction_multiple::{grammar_data, parse, parse_tree::to_sexpr};
use iguana_runtime::{input::Input, testing::{check_golden_file, golden_path}};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let tree = parse(&input, grammar_data::S).expect("Parse failed");
    let actual = to_sexpr(tree.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

// Single Id: letters only
#[test]
fn test_alpha_only() {
    check("abc", "alpha_only");
}

// Single Id: digits only
#[test]
fn test_digits_only() {
    check("123", "digits_only");
}

// Single Id: letters and digits, !>> Alpha and !>> Digit force longest match
#[test]
fn test_alpha_digit_mixed() {
    check("abc123", "alpha_digit_mixed");
}
