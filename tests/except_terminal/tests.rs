// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/except_terminal/except_terminal.iggy --output tests/except_terminal
// To update golden files: REGENERATE=1 cargo test -p except_terminal

use except_terminal::{grammar_data, parse, parse_tree::to_sexpr};
use iguana_runtime::ids::NonterminalId;
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

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

#[test]
fn test_identifier() {
    check(grammar_data::S, "abc", "identifier");
}

#[test]
fn test_keyword_if_rejected() {
    check_fails(grammar_data::S, "if");
}

#[test]
fn test_keyword_else_rejected() {
    check_fails(grammar_data::S, "else");
}

#[test]
fn test_keyword_while_rejected() {
    check_fails(grammar_data::S, "while");
}

#[test]
fn test_keyword_prefix_accepted() {
    check(grammar_data::S, "ifx", "keyword_prefix");
}

#[test]
fn test_single_char() {
    check(grammar_data::S, "x", "single_char");
}
