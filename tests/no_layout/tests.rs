// To regenerate parser:  cargo run -p iguana -- test gen no_layout
// To update golden files: REGENERATE=1 cargo test -p no_layout

use no_layout::{parse, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let tree = parse(&input, "S").expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

fn check_fails(input: &str) {
    let input_str = input;
    let input = Input::from(input);
    assert!(parse(&input, "S").is_none(), "Expected parse to fail for input: {input_str}");
}

#[test]
fn single_char() {
    check("a", "single_char");
}

#[test]
fn multiple_chars() {
    check("abc", "multiple_chars");
}

#[test]
fn rejects_spaces_in_id() {
    // With @layout(none), "a b" should NOT parse as a single Id
    // because layout is not inserted between Char+ elements.
    check_fails("a b");
}
