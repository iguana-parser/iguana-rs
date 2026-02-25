// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/except_terminal/except_terminal.iggy --output tests/except_terminal
// To update golden files: REGENERATE=1 cargo test -p except_terminal

use except_terminal::{parse, parse_tree::to_sexpr};
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {
    let tree = parse(input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(start_nonterminal: &str, input: &str) {
    assert!(parse(input, start_nonterminal).is_none(), "Expected parse to fail for input: {input}");
}

#[test]
fn test_identifier() {
    check("S", "abc", "identifier");
}

#[test]
fn test_keyword_if_rejected() {
    check_fails("S", "if");
}

#[test]
fn test_keyword_else_rejected() {
    check_fails("S", "else");
}

#[test]
fn test_keyword_while_rejected() {
    check_fails("S", "while");
}

#[test]
fn test_keyword_prefix_accepted() {
    check("S", "ifx", "keyword_prefix");
}

#[test]
fn test_single_char() {
    check("S", "x", "single_char");
}
