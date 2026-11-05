// The golden files do not show whether the parser shares an SPPF node. These
// tests call the node methods of the parser directly.

use deep_priority::DeepPriorityParser;
use iguana_runtime::{
    arena::Arena,
    ids::{EnvId, Id, SPPFNodeId, SlotId, TerminalId},
    input::Input,
    parser::Parser,
    sppf::{IntermediateNode, SPPFNode},
};

/// Three distinct terminal nodes, to use as children.
fn terminal_nodes(parser: &mut DeepPriorityParser<'_, '_>) -> [SPPFNodeId; 3] {
    [0, 1, 2].map(|id| parser.get_or_create_terminal_node(TerminalId(id), 0, 0))
}

fn intermediate_node<'a>(
    parser: &'a DeepPriorityParser<'_, '_>,
    id: SPPFNodeId,
) -> &'a IntermediateNode {
    match parser.sppf_node(id) {
        SPPFNode::Intermediate(node) => node,
        _ => unreachable!("expected an intermediate node"),
    }
}

#[test]
fn a_terminal_node_is_shared_by_matches_of_the_same_span() {
    let input = Input::from("");
    let arena = Arena::new();
    let mut parser = DeepPriorityParser::new(&input, &arena);

    let id = parser.get_or_create_terminal_node(TerminalId(1), 2, 4);
    assert_eq!(parser.get_or_create_terminal_node(TerminalId(1), 2, 4), id);
    assert_ne!(parser.get_or_create_terminal_node(TerminalId(1), 2, 5), id);
    assert_ne!(parser.get_or_create_terminal_node(TerminalId(2), 2, 4), id);
}

#[test]
fn an_intermediate_node_is_shared_only_by_arrivals_with_the_same_env() {
    let input = Input::from("");
    let arena = Arena::new();
    let mut parser = DeepPriorityParser::new(&input, &arena);
    let [a, b, c] = terminal_nodes(&mut parser);
    let slot = SlotId(0);

    let mut ids = Vec::new();
    for env in [None, Some(EnvId::from_index(1)), Some(EnvId::from_index(2))] {
        let id = parser
            .get_or_create_intermediate_node(slot, 0, 0, a, b, env)
            .expect("the first arrival with an env creates a node");
        assert!(!ids.contains(&id));
        ids.push(id);
        assert_eq!(intermediate_node(&parser, id).env, env);
        assert_eq!(parser.lookup_intermediate_node(slot, 0, 0, env), Some(id));
        assert_eq!(
            parser.get_or_create_intermediate_node(slot, 0, 0, a, c, env),
            None
        );
        assert!(intermediate_node(&parser, id).ambiguous);
    }
}

#[test]
fn an_ll1_intermediate_node_is_not_in_the_index() {
    let input = Input::from("");
    let arena = Arena::new();
    let mut parser = DeepPriorityParser::new(&input, &arena);
    let [a, b, c] = terminal_nodes(&mut parser);
    let slot = SlotId(0);

    let ll1 = parser.create_intermediate_node_ll1(slot, 0, 0, a, b);
    assert_eq!(parser.lookup_intermediate_node(slot, 0, 0, None), None);
    let gll = parser
        .get_or_create_intermediate_node(slot, 0, 0, a, c, None)
        .expect("the LL(1) node is not found");
    assert_ne!(gll, ll1);
}
