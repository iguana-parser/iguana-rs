// To regenerate parser:  cargo run -p iguana -- test gen follow_restriction
// To update golden files: REGENERATE=1 cargo test -p follow_restriction

use follow_restriction::{parse, parse_tree::to_sexpr};
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let tree = parse(input, "S").expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str) {
    assert!(parse(input, "S").is_none(), "Expected parse to fail for input: {input}");
}

#[test]
fn test_single_char_accepted() {
    // "a" followed by end of input — no Char follows, accepted
    check("a", "single_char");
}

#[test]
fn test_single_char_followed_by_char_rejected() {
    // "ab" — Char followed by Char, the first Char is rejected by !>> Char,
    // and the second Char has nothing after it but S expects only one Id
    check_fails("ab");
}
