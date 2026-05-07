// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/comments/comments.iggy --output tests/comments
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests comments::

use comments::{parse_expr, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/comments");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_expr(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_comment_between_tokens() {
    check("x // a comment\n+ x", "comment_between_tokens");
}

#[test]
fn test_comment_at_end() {
    check("x + x // trailing comment", "comment_at_end");
}

#[test]
fn test_multiple_comments() {
    check("x // first\n+ // second\nx", "multiple_comments");
}
