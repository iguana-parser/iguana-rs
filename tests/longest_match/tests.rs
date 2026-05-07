// To regenerate parser:  cargo xtask test-gen longest_match
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests longest_match::

// Regression test for the LL(1) longest-match dispatch.
//
// `X = "<" | "<="` has two alternatives whose terminal-level prediction sets
// are disjoint as Terminal sets ({"<"} and {"<="}) but operationally overlap:
// at any position with input `<=`, both `match_token("<")` and
// `match_token("<=")` succeed. The LL(1) optimization must pick the longer
// match (`"<="`), otherwise input `<=x` fails because the parser commits to
// `"<"` and then cannot match `"x"` at the `=`.
//
// The bug only fires when the prefix-overlap nonterminal is reached via a
// sub-call from another LL(1) site — here, S's descriptor for `S : . X "x"`
// calls `parse_x_ll1` directly.

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use longest_match::{parse_s, parse_tree::to_sexpr};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/longest_match");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

fn check_fails(input: &str) {
    let input_str = input;
    let input = Input::from(input);
    let ctx = ParseContext::new();
    assert!(
        parse_s(&input, &ctx).is_err(),
        "Expected parse to fail for input: {input_str}"
    );
}

#[test]
fn lt() {
    check("<x", "lt");
}

#[test]
fn le() {
    // Regression: with the old LL(1) cascade the parser committed to "<"
    // before exploring "<=" and the parse failed.
    check("<=x", "le");
}

#[test]
fn no_match() {
    check_fails(">=x");
}
