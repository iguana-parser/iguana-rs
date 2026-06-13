// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/amb/amb.iggy --output tests/amb
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests amb::

// Precedence with labeled recursive operands (`lhs:E`/`rhs:E`). The desugaring
// must see through the `Labeled` wrapper to recognize the recursive ends, or it
// silently no-ops and the grammar stays ambiguous. With the fix, `1+2*3` has a
// single parse, `1+(2*3)`, and the `lhs`/`rhs` labels are preserved.

use amb::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/amb");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_num() {
    check("1", "num");
}

#[test]
fn test_add() {
    check("1+2", "add");
}

#[test]
fn test_mul() {
    check("2*3", "mul");
}

// `*` binds tighter than `+`: the single parse is `1+(2*3)`.
#[test]
fn test_add_mul() {
    check("1+2*3", "add_mul");
}

// Mirror of the above: `(1*2)+3`.
#[test]
fn test_mul_add() {
    check("1*2+3", "mul_add");
}

// Left-associative `+`: `(1+2)+3`.
#[test]
fn test_add_add() {
    check("1+2+3", "add_add");
}

// Left-associative `*`: `(1*2)*3`.
#[test]
fn test_mul_mul() {
    check("1*2*3", "mul_mul");
}
