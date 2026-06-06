// To regenerate parser:  cargo xtask test-gen indirect_prefix
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests indirect_prefix::

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use indirect_prefix::{parse_s, parse_tree::to_sexpr};
const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/indirect_prefix");
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
fn test_fn_a() {
    check("fn a", "fn_a");
}

#[test]
fn test_fn_a_plus_a() {
    // `fn` binds loosest, so the lambda body extends to `a + a`: the only parse
    // is `fn (a + a)`, never `(fn a) + a`.
    check("fn a + a", "fn_a_plus_a");
}
