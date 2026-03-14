// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/comments/comments.iggy --output tests/comments
// To update golden files: REGENERATE=1 cargo test -p comments

use comments::{parse, parse_tree::to_sexpr};
use iguana_runtime::{input::Input, testing::{check_golden_file, golden_path}};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {
    let input = Input::from(input);
    let tree = parse(&input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(tree.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_comment_between_tokens() {
    check("StartExpr", "x // a comment\n+ x", "comment_between_tokens");
}

#[test]
fn test_comment_at_end() {
    check("StartExpr", "x + x // trailing comment", "comment_at_end");
}

#[test]
fn test_multiple_comments() {
    check("StartExpr", "x // first\n+ // second\nx", "multiple_comments");
}
