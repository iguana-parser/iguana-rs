// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/pepm16_expressions/pepm16_expressions.iggy --output tests/pepm16_expressions
// To update golden files: REGENERATE=1 cargo test -p pepm16_expressions

use pepm16_expressions::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_s(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

// --- Atoms and parentheses ---

#[test]
fn test_a() {
    check("a", "a");
}

#[test]
fn test_parens() {
    check("(a+a)*a", "parens");
}

#[test]
fn test_nested_parens() {
    check("((a))", "nested_parens");
}

// --- Postfix ---

#[test]
fn test_postfix() {
    check("a.f", "postfix");
}

#[test]
fn test_postfix_chain() {
    check("a.f.f", "postfix_chain");
}

// --- Juxtaposition ---

#[test]
fn test_juxtaposition() {
    check("a a", "juxtaposition");
}

// --- Precedence ---

// a + (a * a)
#[test]
fn test_precedence() {
    check("a+a*a", "precedence");
}

// --- Associativity ---

// (a + a) + a
#[test]
fn test_left_assoc() {
    check("a+a+a", "left_assoc");
}

// a ; (a ; a)
#[test]
fn test_right_assoc() {
    check("a;a;a", "right_assoc");
}

// --- Deep case: prefix below binary ---

// -(a + a)
#[test]
fn test_neg_add() {
    check("-a+a", "neg_add");
}

// -(a * a)
#[test]
fn test_neg_mul() {
    check("-a*a", "neg_mul");
}

// -(a.f)
#[test]
fn test_neg_postfix() {
    check("-a.f", "neg_postfix");
}

// --- Deep case: if-then-else ---

// if a then a else (a + a)
#[test]
fn test_if_then_else_add() {
    check("if a then a else a+a", "if_then_else_add");
}

// a + (if a then a else a)
#[test]
fn test_add_if_then_else() {
    check("a + if a then a else a", "add_if_then_else");
}

// a + (if a then a else (a * a))
#[test]
fn test_add_if_then_else_mul() {
    check("a + if a then a else a*a", "add_if_then_else_mul");
}

// if a then a else (a ; a)
#[test]
fn test_if_then_else_seq() {
    check("if a then a else a;a", "if_then_else_seq");
}

// --- Combinations with parentheses ---

// if (a + a) then a else a
#[test]
fn test_if_parens_cond() {
    check("if (a+a) then a else a", "if_parens_cond");
}

// if a then a else (a + a) * a
#[test]
fn test_if_then_else_add_mul() {
    check("if a then a else (a+a)*a", "if_then_else_add_mul");
}

// (if a then a else a) + a
#[test]
fn test_parens_if_add() {
    check("(if a then a else a)+a", "parens_if_add");
}

// --- Seq with if-then-else ---

// a ; (if a then a else a)
#[test]
fn test_seq_if_then_else() {
    check("a ; if a then a else a", "seq_if_then_else");
}

// --- Complex combinations ---

// -(-a)
#[test]
fn test_double_neg() {
    check("- -a", "double_neg");
}

// -(a + a) * a
#[test]
fn test_neg_add_then_mul() {
    check("(-a+a)*a", "neg_add_then_mul");
}

// if a then a else a + a ; a
#[test]
fn test_if_add_seq() {
    check("if a then a else a+a;a", "if_add_seq");
}
