// To regenerate parser:  cargo run -p iguana -- test gen precede_restriction
// To update golden files: REGENERATE=1 cargo test -p precede_restriction

use precede_restriction::{parse, parse_tree::to_sexpr};
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let tree = parse(input, "S").expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str) {
    assert!(parse(input, "S").is_none(), "Expected parse to fail for input: {input}");
}

// Nonterminal case: Char !<< Char+

#[test]
fn for_id() {
    check("for x", "for_id");
}

#[test]
fn forall() {
    // "forall" should NOT parse as "for" + Id("all") because
    // the precede restriction on Id rejects "all" (preceded by 'r').
    // Only the "forall" literal alternative works.
    check("forall", "forall");
}
