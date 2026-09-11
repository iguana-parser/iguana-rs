use std::{collections::HashSet, ptr};

use cycles::parse_tree::{self, *};
use iguana_runtime::{
    arena::Arena,
    input::{Input, Span},
    parse_tree::{CycleTarget, DisplayOptions, NodeKind, ParseTreeNode},
};

fn nodes(root: ParseTree<'_>) -> Vec<ParseTree<'_>> {
    let mut seen = HashSet::new();
    let mut stack = vec![root];
    let mut nodes = vec![];
    while let Some(node) = stack.pop() {
        if let Some(id) = node.node_id()
            && !seen.insert(id)
        {
            continue;
        }
        stack.extend(node.children());
        nodes.push(node);
        assert!(
            nodes.len() < 1000,
            "these small inputs must have finite, compact trees"
        );
    }
    nodes
}

fn check_graph(root: ParseTree<'_>, cyclic: bool) {
    assert_eq!(root.contains_cycle(), cyclic);
    let nodes = nodes(root);
    for node in &nodes {
        if let Some(target) = node.cycle_target() {
            assert!(node.kind() == NodeKind::Cycle);
            assert!(node.children().is_empty());
            assert_eq!(node.child_count(), 0);
            assert!(node.origin().is_none());
            assert_eq!(node.span(), target.span());
            assert_eq!(
                std::mem::discriminant(node),
                std::mem::discriminant(&target)
            );
            assert!(nodes.iter().any(|n| n.node_id() == target.node_id()));
            assert!(target.kind() != NodeKind::Cycle);
        }
    }

    // Debug and both renderings terminate. A Cycle appears as a #N# reference
    // in s-expressions and an edge in JSON, never as a rendered node.
    assert!(format!("{root:?}").len() < 100_000);
    for options in [DisplayOptions::default(), DisplayOptions::simplified()] {
        let rendered = parse_tree::to_sexpr_with(root, options);
        assert!(!rendered.contains("Cycle"), "{rendered}");
        if cyclic {
            assert!(rendered.contains("#1="), "{rendered}");
            assert!(rendered.contains("#1#"), "{rendered}");
        }
        assert!(rendered.len() < 100_000);
    }
    let graph: serde_json::Value = serde_json::from_str(&parse_tree::to_json(root)).unwrap();
    let json_nodes = graph["nodes"].as_array().unwrap();
    let edges = graph["edges"].as_array().unwrap();
    assert!(json_nodes.iter().all(|n| n["kind"] != "Cycle"));
    assert_eq!(
        json_nodes.len(),
        nodes.iter().filter(|n| !n.is_cycle()).count()
    );
    assert_eq!(
        edges.len(),
        nodes.iter().map(|n| n.children().len()).sum::<usize>()
    );
    for edge in edges {
        assert!(json_nodes.iter().any(|n| n["id"] == edge["src"]));
        assert!(json_nodes.iter().any(|n| n["id"] == edge["dest"]));
    }
}

#[test]
fn direct_cycle_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_direct(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn mutual_cycle_from_c_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_mutual_c(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn mutual_cycle_from_b_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_mutual_b(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn cycle_chain_from_c_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_chain_c(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn cycle_chain_from_b_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_chain_b(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn shared_cycle_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_shared_s(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn two_exit_cycle_from_c_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_two_exits_c(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn two_exit_cycle_from_b_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_two_exits_b(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn two_route_cycle_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_two_routes_c(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn nullable_cycle_on_empty_input_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_nullable(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn nullable_cycle_on_nonempty_input_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_nullable(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn cycle_with_ambiguous_nullable_sibling_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_sibling_c(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn parent_of_cycle_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("ab");
    let root = cycles::parse_parent_c(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn overlapping_cycles_from_c_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_overlap_c(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn overlapping_cycles_from_b_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_overlap_b(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn cycle_entered_through_an_intermediate_node_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("ab");
    let root = cycles::parse_intermediate_y(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn cycle_followed_by_input_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("bb");
    let root = cycles::parse_mid_input(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn consuming_recursion_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("bb");
    let root = cycles::parse_consuming(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, false);
    assert!(!root.contains_ambiguity());
}

#[test]
fn ordinary_rule_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_ordinary(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, false);
    assert!(!root.contains_ambiguity());
}

#[test]
fn nullable_plus_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_nullable_plus(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn nullable_star_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_nullable_star(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn nullable_separated_list_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_nullable_separated(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn optional_cycle_on_empty_input_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_optional(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn optional_cycle_on_nonempty_input_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_optional(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn grouped_cycle_builds_a_finite_graph() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_grouped(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    check_graph(root, true);
}

#[test]
fn ambiguity_does_not_require_a_cycle() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_sibling_d(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    assert!(root.contains_ambiguity());
    assert!(!root.contains_cycle());
    check_graph(root, false);
}

#[test]
fn direct_cycle_targets_the_complete_ambiguity() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_direct(&input, &arena).unwrap().tree.node;
    let Direct::Amb(alternatives) = root else {
        panic!("expected Amb")
    };
    assert_eq!(alternatives.len(), 2);
    assert!(alternatives.iter().any(|n| matches!(n, Direct::C2 { .. })));
    let reference = alternatives
        .iter()
        .find_map(|node| match node {
            Direct::C1 {
                direct: Direct::Cycle { target: cycle },
                ..
            } => Some(cycle),
            _ => None,
        })
        .unwrap();
    assert!(ptr::eq(reference.get(), root));
    // This signature checks at compile time that the target outlives a short
    // borrow of the cell.
    fn target<'a>(cycle: &CycleTarget<'a, Direct<'a>>) -> &'a Direct<'a> {
        cycle.get()
    }
    assert!(ptr::eq(target(reference), root));
}

#[test]
fn mutual_cycle_can_target_an_unambiguous_nonterminal() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_mutual_b(&input, &arena).unwrap().tree.node;
    let MutualB::B1 {
        mutual_c: MutualC::Amb(alternatives),
        ..
    } = root
    else {
        panic!("expected B1 with ambiguous C")
    };
    let cycle = alternatives
        .iter()
        .find_map(|node| match node {
            MutualC::C1 {
                mutual_b: MutualB::Cycle { target: cycle },
                ..
            } => Some(cycle),
            _ => None,
        })
        .unwrap();
    assert!(ptr::eq(cycle.get(), root));
}

#[test]
fn nullable_cycle_targets_distinguish_both_empty_spans() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_nullable(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    let targets: HashSet<_> = nodes(root)
        .into_iter()
        .filter_map(|node| {
            node.cycle_target()
                .map(|target| (target.node_id(), target.span()))
        })
        .collect();
    for span in [Span::new(0, 0), Span::new(0, 1), Span::new(1, 1)] {
        assert_eq!(targets.iter().filter(|(_, s)| *s == span).count(), 1);
    }
}

#[test]
fn memoized_cycle_keeps_its_target_outside_the_original_ancestor_path() {
    let arena = Arena::new();
    let input = Input::from("");
    let root = cycles::parse_shared_s(&input, &arena).unwrap().tree.node;
    let a = root.shared_a();
    let b = root.shared_b();
    let SharedA::Amb(alternatives) = a else {
        panic!("expected Amb")
    };
    let via_b = alternatives
        .iter()
        .find_map(|node| match node {
            SharedA::A1 { shared_b, .. } => Some(*shared_b),
            _ => None,
        })
        .unwrap();
    assert!(ptr::eq(via_b, b));
    let SharedA::Cycle { target: cycle } = b.shared_a() else {
        panic!("expected Cycle")
    };
    assert!(ptr::eq(cycle.get(), a));
    // Querying or rendering the reused subtree must also handle that target.
    assert!(b.as_parse_tree().contains_ambiguity());
    let rendered = parse_tree::to_sexpr(b.as_parse_tree());
    assert!(!rendered.contains("Cycle"));
    assert!(rendered.contains("#1="));
    assert!(rendered.contains("#1#"));
}

#[test]
fn trees_remain_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ParseTree<'static>>();
    assert_send_sync::<Direct<'static>>();
}

#[test]
fn typed_accessors_reject_cycles() {
    let arena = Arena::new();
    let input = Input::from("b");
    let root = cycles::parse_mutual_b(&input, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    let cycle = nodes(root)
        .into_iter()
        .find_map(|node| match node {
            ParseTree::MutualB(node @ MutualB::Cycle { .. }) => Some(node),
            _ => None,
        })
        .unwrap();
    assert!(std::panic::catch_unwind(|| cycle.mutual_c()).is_err());

    let empty = Input::from("");
    let root = cycles::parse_nullable_plus(&empty, &arena)
        .unwrap()
        .tree
        .as_parse_tree();
    let cycle = nodes(root)
        .into_iter()
        .find_map(|node| match node {
            ParseTree::Plus0(node @ Plus0::Cycle { .. }) => Some(node),
            _ => None,
        })
        .unwrap();
    assert!(std::panic::catch_unwind(|| cycle.empty_es().count()).is_err());
    assert!(cycle.as_parse_tree().children().is_empty());
}
