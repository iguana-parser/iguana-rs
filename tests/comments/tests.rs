// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/comments/comments.iggy --output tests/comments
// To update golden files: REGENERATE=1 cargo test -p comments

use comments::{parse_expr, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_expr(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
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
