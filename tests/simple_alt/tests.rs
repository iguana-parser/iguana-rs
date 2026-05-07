// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/simple_alt/simple_alt.iggy --output tests/simple_alt
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests simple_alt::

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use simple_alt::{parse_a, parse_tree::to_sexpr};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/simple_alt");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_a(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_c() {
    check("bc", "c");
}

#[test]
fn test_d() {
    check("bd", "d");
}
