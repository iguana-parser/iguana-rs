// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/star_with_sep/star_with_sep.iggy --output tests/star_with_sep
// To update golden files: REGENERATE=1 cargo test -p star_with_sep

use star_with_sep::{parse_s, parse_tree::to_sexpr};
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
fn test_one() {
    check("a", "one");
}

#[test]
fn test_many() {
    check("a,a,a", "many");
}
