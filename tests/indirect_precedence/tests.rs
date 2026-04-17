// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/indirect_precedence/indirect_precedence.iggy --output tests/indirect_precedence
// To update golden files: REGENERATE=1 cargo test -p indirect_precedence

use indirect_precedence::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_s(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_a() {
    check("a", "a");
}

#[test]
fn test_neg_a() {
    check("-a", "neg_a");
}

#[test]
fn test_a_mul_a_div_a() {
    // a * (a / a) — F expands to E "/" K, K expands to E
    check("a*a/a", "a_mul_a_div_a");
}

#[test]
fn test_neg_a_mul_a_div_a() {
    check("-a*a/a", "neg_a_mul_a_div_a");
}

#[test]
fn test_a_mul_neg_a_div_a() {
    // Tests neg inside the indirect path: a * ((-a) / a)
    check("a*-a/a", "a_mul_neg_a_div_a");
}
