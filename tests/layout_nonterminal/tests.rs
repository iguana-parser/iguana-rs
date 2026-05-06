// To regenerate parser:  cargo xtask test-gen layout_nonterminal
// To update golden files: REGENERATE=1 cargo test -p layout_nonterminal

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use layout_nonterminal::{parse_s, parse_tree::to_sexpr};
fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}
#[test]
fn no_layout() {
    check("x", "no_layout");
}

#[test]
fn leading_whitespace() {
    check("   x", "leading_whitespace");
}

#[test]
fn leading_comment() {
    check("// hi\nx", "leading_comment");
}
