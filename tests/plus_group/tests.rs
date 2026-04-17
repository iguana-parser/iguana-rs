// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/plus_group/plus_group.iggy --output tests/plus_group
// To update golden files: REGENERATE=1 cargo test -p plus_group

use plus_group::{parse_s, parse_tree::to_sexpr};
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
    check("abc", "one");
}

#[test]
fn test_many() {
    check("abcabc", "many");
}
