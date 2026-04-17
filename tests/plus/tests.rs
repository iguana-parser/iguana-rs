// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/plus/plus.iggy --output tests/plus
// To update golden files: REGENERATE=1 cargo test -p plus

use plus::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_s(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_one() {
    check("a", "one");
}

#[test]
fn test_many() {
    check("aaa", "many");
}
