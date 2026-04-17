// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/star_with_sep/star_with_sep.iggy --output tests/star_with_sep
// To update golden files: REGENERATE=1 cargo test -p star_with_sep

use star_with_sep::{grammar_data, parse, parse_tree::to_sexpr};
use iguana_runtime::ids::NonterminalId;
use iguana_runtime::input::Input;
use iguana_runtime::testing::{check_golden_file, golden_path};

fn check(start_nonterminal: NonterminalId, input: &str, test_name: &str) {
    let input = Input::from(input);
    let result = parse(&input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}

#[test]
fn test_empty() {
    check(grammar_data::S, "", "empty");
}

#[test]
fn test_one() {
    check(grammar_data::S, "a", "one");
}

#[test]
fn test_many() {
    check(grammar_data::S, "a,a,a", "many");
}
