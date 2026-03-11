// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/multiple_except/multiple_except.iggy --output tests/multiple_except
// To update golden files: REGENERATE=1 cargo test -p multiple_except

use multiple_except::{parse, parse_tree::to_sexpr};
use iguana_runtime::{input::Input, testing::{check_golden_file, golden_path}};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {
    let input = Input::from(input);
    let tree = parse(&input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_example() {
    // check("Start", "input", "example");
}
