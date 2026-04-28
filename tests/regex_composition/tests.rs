// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/regex_composition/regex_composition.iggy --output tests/regex_composition
// To update golden files: REGENERATE=1 cargo test -p regex_composition

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use regex_composition::{parse_s, parse_tree::to_sexpr};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_single_letter() {
    check("x", "single_letter");
}

#[test]
fn test_identifier() {
    check("abc", "identifier");
}

#[test]
fn test_identifier_with_digits() {
    check("foo42", "identifier_with_digits");
}

#[test]
fn test_underscore_start() {
    check("_bar", "underscore_start");
}

#[test]
fn test_mixed() {
    check("a1_B2", "mixed");
}

#[test]
fn test_digit_only_fails() {
    let input = Input::from("123");
    let ctx = ParseContext::new();
    assert!(
        parse_s(&input, &ctx).is_err(),
        "Should not parse: identifiers cannot start with a digit"
    );
}
