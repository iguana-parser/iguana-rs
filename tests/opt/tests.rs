// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/opt/opt.iggy --output tests/opt
// To update golden files: REGENERATE=1 cargo test -p opt

use opt::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_empty() {
    check("", "empty");
}

#[test]
fn test_present() {
    check("a", "present");
}
