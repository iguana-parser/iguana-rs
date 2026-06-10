// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/except_nonterminal_longest_match/except_nonterminal_longest_match.iggy --output tests/except_nonterminal_longest_match
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests except_nonterminal_longest_match::

use except_nonterminal_longest_match::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/except_nonterminal_longest_match"
);

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
fn test_two_ids() {
    check("abcd", "two_ids");
}

// Name produces candidate spans of length one and two at the same position;
// the except must judge each span by the text it covers. On "iffy" the
// span "if" is a Keyword, so no split into two Ids survives. Comparing
// against the except's longest match (4 chars here) would let "if" through.
#[test]
fn test_candidate_inside_longer_except_match_rejected() {
    check_fails("iffy");
}

#[test]
fn test_excluded_candidate_rejected() {
    check_fails("ifab");
}
