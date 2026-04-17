// To regenerate parser:  cargo run -p iguana -- test gen precede_restriction_lexical
// To update golden files: REGENERATE=1 cargo test -p precede_restriction_lexical

use precede_restriction_lexical::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_s(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str) {
    let input_str = input;
    let input = Input::from(input);
    assert!(parse_s(&input).is_err(), "Expected parse to fail for input: {input_str}");
}

#[test]
fn for_id() {
    check("for x", "for_id");
}

#[test]
fn forall() {
    check("forall", "forall");
}

#[test]
fn forall_rejects_for_branch() {
    // "forall" should NOT parse as "for" + Id("all") because
    // the precede restriction on Id rejects "all" (preceded by 'r')
    // It should only parse as the "forall" literal.
    check("forall", "forall");
}
