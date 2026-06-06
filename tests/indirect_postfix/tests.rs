// To regenerate parser:  cargo xtask test-gen indirect_postfix
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests indirect_postfix::

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use indirect_postfix::{parse_s, parse_tree::to_sexpr};
const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/indirect_postfix");
fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_a() {
    check("a", "a");
}

#[test]
fn test_a_plus_a() {
    check("a + a", "a_plus_a");
}

#[test]
fn test_a_bang() {
    check("a !", "a_bang");
}

#[test]
fn test_a_plus_a_bang() {
    // `!` binds loosest, so the postfix takes the whole sum: the only parse is
    // `(a + a) !`, never `a + (a !)`.
    check("a + a !", "a_plus_a_bang");
}
