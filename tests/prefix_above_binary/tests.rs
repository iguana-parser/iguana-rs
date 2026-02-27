// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/prefix_above_binary/prefix_above_binary.iggy --output tests/prefix_above_binary
// To update golden files: REGENERATE=1 cargo test -p prefix_above_binary

use prefix_above_binary::{parse, parse_tree::to_sexpr};
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let tree = parse(input, "S").expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

// (-a) + a: prefix '-' binds tighter than '+'
#[test]
fn test_neg_a_add_a() {
    check("-a+a", "neg_a_add_a");
}

#[test]
fn test_neg_neg_a() {
    check("- -a", "neg_neg_a");
}
