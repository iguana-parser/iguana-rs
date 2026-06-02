// To regenerate parser:  cargo run -p iguana -- generate --grammar tests/amb_list/amb_list.iggy --output tests/amb_list
// To update golden files: REGENERATE=1 cargo test -p iguana-tests --test grammar_tests amb_list::

use amb_list::{parse_s, parse_tree::to_sexpr};
use iguana_runtime::input::Input;
use iguana_runtime::parse_tree::ParseContext;
use iguana_runtime::parser::Parser;
use iguana_runtime::sppf::SPPFNode;
use iguana_runtime::testing::{check_golden_file, golden_path};

const GRAMMAR_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/amb_list");

fn check(input: &str, test_name: &str) {
    let input = Input::from(input);
    let ctx = ParseContext::new();
    let result = parse_s(&input, &ctx).expect("Parse failed");
    let actual = to_sexpr(result.tree.as_parse_tree());
    check_golden_file(&actual, &golden_path(GRAMMAR_DIR, test_name));
}

#[test]
fn test_one_a() {
    check("a", "one_a");
}

#[test]
fn test_two_a() {
    check("aa", "two_a");
}

#[test]
fn dump_sppf_two_a() {
    use amb_list::grammar_data;
    use amb_list::parser::AmbListParser;
    let input = Input::from("aa");
    let mut parser = AmbListParser::new(&input, grammar_data::S);
    let _ = parser.run();
    eprintln!("---- SPPF nodes ----");
    for (i, node) in parser.sppf_nodes().iter().enumerate() {
        match node {
            SPPFNode::Terminal(t) => eprintln!(
                "[{}] Terminal id={} span=[{},{}]",
                i, t.terminal_id.0, t.span.left_extent, t.span.right_extent
            ),
            SPPFNode::Nonterminal(n) => eprintln!(
                "[{}] Nonterminal nt={} return_slot={} span=[{},{}] amb={} child={}",
                i,
                n.nonterminal_id.0,
                n.return_slot.0,
                n.span.left_extent,
                n.span.right_extent,
                n.ambiguous,
                n.child.0
            ),
            SPPFNode::Intermediate(im) => eprintln!(
                "[{}] Intermediate slot={} span=[{},{}] amb={} child=({},{})",
                i,
                im.slot_id.0,
                im.span.left_extent,
                im.span.right_extent,
                im.ambiguous,
                im.child.0.0,
                im.child.1.0
            ),
        }
    }
    eprintln!("---- intermediate_nodes_children_map ----");
    for (id, children) in parser.intermediate_nodes_children_map().iter() {
        eprintln!("  [{}] extras = {:?}", id.0, children);
    }
    eprintln!("---- nonterminal_nodes_children_map ----");
    for (id, children) in parser.nonterminal_nodes_children_map().iter() {
        eprintln!("  [{}] extras = {:?}", id.0, children);
    }
}
