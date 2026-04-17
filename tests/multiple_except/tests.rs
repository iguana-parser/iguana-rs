// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/multiple_except/multiple_except.iggy --output tests/multiple_except
// To update golden files: REGENERATE=1 cargo test -p multiple_except

use multiple_except::{grammar_data, parse, parse_tree::to_sexpr};
use iguana_runtime::{ids::NonterminalId, input::Input, testing::{check_golden_file, golden_path}};

fn check(start_nonterminal: NonterminalId, input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse(&input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(start_nonterminal: NonterminalId, input: &str) {
    let input_str = input;
    let input = Input::from(input);
    assert!(parse(&input, start_nonterminal).is_none(), "Expected parse to fail for input: {input_str}");
}

// --- SyntaxIdentifier tests (nonterminal except path) ---

#[test]
fn test_syntax_identifier() {
    check(grammar_data::SYNTAX_IDENTIFIER, "abc", "syntax_identifier");
}

#[test]
fn test_syntax_keyword_rejected() {
    check_fails(grammar_data::SYNTAX_IDENTIFIER, "if");
}

#[test]
fn test_syntax_boolean_rejected() {
    check_fails(grammar_data::SYNTAX_IDENTIFIER, "true");
}

#[test]
fn test_syntax_null_rejected() {
    check_fails(grammar_data::SYNTAX_IDENTIFIER, "null");
}

#[test]
fn test_syntax_keyword_prefix_accepted() {
    check(grammar_data::SYNTAX_IDENTIFIER, "ifx", "syntax_keyword_prefix");
}

// --- LexicalIdentifier tests (lexical except path) ---

#[test]
fn test_lexical_identifier() {
    check(grammar_data::LEXICAL_IDENTIFIER, "abc", "lexical_identifier");
}

#[test]
fn test_lexical_keyword_rejected() {
    check_fails(grammar_data::LEXICAL_IDENTIFIER, "while");
}

#[test]
fn test_lexical_boolean_rejected() {
    check_fails(grammar_data::LEXICAL_IDENTIFIER, "false");
}

#[test]
fn test_lexical_null_rejected() {
    check_fails(grammar_data::LEXICAL_IDENTIFIER, "null");
}

#[test]
fn test_lexical_keyword_prefix_accepted() {
    check(grammar_data::LEXICAL_IDENTIFIER, "forall", "lexical_keyword_prefix");
}
