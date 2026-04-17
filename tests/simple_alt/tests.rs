// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/simple_alt/simple_alt.iggy --output tests/simple_alt
// To update golden files: REGENERATE=1 cargo test -p simple_alt

use simple_alt::{parse_a, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse_a(&input).expect("Parse failed");
    let actual = to_sexpr(result.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_c() {
    check("bc", "c");
}

#[test]
fn test_d() {
    check("bd", "d");
}
