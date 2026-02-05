// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/opt/opt.iggy --output tests/opt
// To update golden files: REGENERATE=1 cargo test -p opt

use opt::{parse, parse_tree::to_sexpr};
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {
    let tree = parse(input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_empty() {
    check("S", "", "empty");
}

#[test]
fn test_present() {
    check("S", "a", "present");
}
