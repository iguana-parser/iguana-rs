// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/exclude_by_label/exclude_by_label.iggy --output tests/exclude_by_label
// To update golden files: REGENERATE=1 cargo test -p exclude_by_label

use exclude_by_label::{grammar_data, parse, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse(&input, grammar_data::EXPR).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn single_arg_call() {
    check("f(x)", "single_arg_call");
}

#[test]
fn multi_arg_call() {
    check("f(x,y)", "multi_arg_call");
}

#[test]
fn comma_expr() {
    check("x,y", "comma_expr");
}

#[test]
fn chained_call() {
    check("f(x,y)(z)", "chained_call");
}
