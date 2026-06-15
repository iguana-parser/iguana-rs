// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/plus_except/plus_except.iggy --output tests/plus_except
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests plus_except::

// A `\` exclude on a Plus's element or separator must be enforced. The LL(1) Plus
// loop only matches the symbol and ignores the restriction, so a Plus with one is
// parsed using GLL instead. Without that, a keyword would be accepted as an
// element or separator.

use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};
use plus_except::{parse_base, parse_sep, parse_tree::to_sexpr};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/plus_except");

// `Sep = {Number Identifier \ Keyword}+`: the separator excludes keywords.
fn check_sep(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_sep(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

fn check_sep_fails(input: &str) {
    let ctx = ParseContext::new();
    assert!(
        parse_sep(&Input::from(input), &ctx).is_err(),
        "Expected parse to fail for input: {input}"
    );
}

// `Base = {Identifier \ Keyword ","}+`: each element excludes keywords.
fn check_base(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_base(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

fn check_base_fails(input: &str) {
    let ctx = ParseContext::new();
    assert!(
        parse_base(&Input::from(input), &ctx).is_err(),
        "Expected parse to fail for input: {input}"
    );
}

// `if` is a keyword, so it cannot be a separator: the parse stops after `1` and
// `if2` is left over.
#[test]
fn test_sep_keyword_rejected() {
    check_sep_fails("1if2");
}

#[test]
fn test_sep_plain() {
    check_sep("1a2", "sep_plain");
}

// `if` is a keyword, so the single element is rejected.
#[test]
fn test_base_keyword_rejected() {
    check_base_fails("if");
}

#[test]
fn test_base_plain() {
    check_base("abc", "base_plain");
}
