// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/amb_plus/amb_plus.iggy --output tests/amb_plus
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests amb_plus::

use amb_plus::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/amb_plus");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_one_a() {
    check("a", "one_a");
}

#[test]
fn test_two_a() {
    check("aa", "two_a");
}

#[test]
fn test_three_a() {
    check("aaa", "three_a");
}

#[test]
fn test_four_a() {
    check("aaaa", "four_a");
}
