// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/multiple_except/multiple_except.iggy --output tests/multiple_except
// To update golden files: REGENERATE=1 cargo test -p multiple_except

use multiple_except::{parse, parse_tree::to_sexpr};
use iguana_runtime::{input::Input, testing::{check_golden_file, golden_path}};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse(&input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(start_nonterminal: &str, input: &str) {
    let input_str = input;
    let input = Input::from(input);
    assert!(parse(&input, start_nonterminal).is_none(), "Expected parse to fail for input: {input_str}");
}

// --- SyntaxIdentifier tests (nonterminal except path) ---

#[test]
fn test_syntax_identifier() {
    check("SyntaxIdentifier", "abc", "syntax_identifier");
}

#[test]
fn test_syntax_keyword_rejected() {
    check_fails("SyntaxIdentifier", "if");
}

#[test]
fn test_syntax_boolean_rejected() {
    check_fails("SyntaxIdentifier", "true");
}

#[test]
fn test_syntax_null_rejected() {
    check_fails("SyntaxIdentifier", "null");
}

#[test]
fn test_syntax_keyword_prefix_accepted() {
    check("SyntaxIdentifier", "ifx", "syntax_keyword_prefix");
}

// --- LexicalIdentifier tests (lexical except path) ---

#[test]
fn test_lexical_identifier() {
    check("LexicalIdentifier", "abc", "lexical_identifier");
}

#[test]
fn test_lexical_keyword_rejected() {
    check_fails("LexicalIdentifier", "while");
}

#[test]
fn test_lexical_boolean_rejected() {
    check_fails("LexicalIdentifier", "false");
}

#[test]
fn test_lexical_null_rejected() {
    check_fails("LexicalIdentifier", "null");
}

#[test]
fn test_lexical_keyword_prefix_accepted() {
    check("LexicalIdentifier", "forall", "lexical_keyword_prefix");
}
