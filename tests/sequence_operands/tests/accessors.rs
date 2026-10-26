use iguana_runtime::{
    arena::Arena,
    input::{Input, Span},
    parse_tree::ParseTreeNode,
};
use sequence_operands::{SequenceOperandsParser, parse_tree::*};

/// The text that `span` covers.
fn text_of(text: &str, span: Span) -> &str {
    &text[span.left_extent as usize..span.right_extent as usize]
}

#[test]
fn a_sequence_list_with_one_named_symbol_has_its_accessor() {
    let text = "a, b ,c";
    let input = Input::from(text);
    let (arena, parser_arena) = (Arena::new(), Arena::new());
    let tree = SequenceOperandsParser::new(&input, &parser_arena)
        .parse_items(&arena)
        .unwrap()
        .tree;
    let Items::Alt0 { star_0, .. } = tree.node else {
        panic!("unexpected node {:?}", tree.node);
    };
    let items: Vec<_> = star_0.items().map(|i| text_of(text, i.span())).collect();
    assert_eq!(items, ["b", "c"]);
}

#[test]
fn the_accessor_of_a_named_terminal_skips_the_literal_tokens() {
    let text = "names , a , bc";
    let input = Input::from(text);
    let (arena, parser_arena) = (Arena::new(), Arena::new());
    let tree = SequenceOperandsParser::new(&input, &parser_arena)
        .parse_names(&arena)
        .unwrap()
        .tree;
    let Names::Alt0 { star_1, .. } = tree.node else {
        panic!("unexpected node {:?}", tree.node);
    };
    let names: Vec<_> = star_1.names().map(|n| text_of(text, n.span())).collect();
    assert_eq!(names, ["a", "bc"]);
}

#[test]
fn a_user_rule_delegates_the_accessor_of_a_sequence_list() {
    let text = ", a , b";
    let input = Input::from(text);
    let (arena, parser_arena) = (Arena::new(), Arena::new());
    let tree = SequenceOperandsParser::new(&input, &parser_arena)
        .parse_separated(&arena)
        .unwrap()
        .tree;
    let items: Vec<_> = tree.node.items().map(|i| text_of(text, i.span())).collect();
    assert_eq!(items, ["a", "b"]);
}

#[test]
fn an_element_that_starts_with_a_nonterminal_keeps_its_symbols() {
    let text = "mixed k k v v k";
    let input = Input::from(text);
    let (arena, parser_arena) = (Arena::new(), Arena::new());
    let tree = SequenceOperandsParser::new(&input, &parser_arena)
        .parse_mixed(&arena)
        .unwrap()
        .tree;
    let Mixed::Alt0 { plus_4, .. } = tree.node else {
        panic!("unexpected node {:?}", tree.node);
    };
    let names: Vec<_> = plus_4
        .as_parse_tree()
        .children()
        .iter()
        .map(|child| child.display_name())
        .filter(|name| *name != "Layout")
        .collect();
    assert_eq!(
        names,
        [
            "Key",
            "(Value Value)?",
            "Key",
            "(Value Value)?",
            "Key",
            "(Value Value)?"
        ]
    );
}

#[test]
fn an_optional_with_several_symbols_has_optional_field_accessors() {
    for (text, expected) in [("if a else b", Some("b")), ("if a", None)] {
        let input = Input::from(text);
        let (arena, parser_arena) = (Arena::new(), Arena::new());
        let tree = SequenceOperandsParser::new(&input, &parser_arena)
            .parse_else(&arena)
            .unwrap()
            .tree;
        let Else::Alt0 { opt_0, .. } = tree.node else {
            panic!("unexpected node {:?}", tree.node);
        };
        let item = opt_0.item().map(|i| text_of(text, i.span()));
        assert_eq!(item, expected);
        assert_eq!(opt_0.lit_0().is_some(), expected.is_some());
    }
}

#[test]
fn an_optional_with_one_symbol_in_parentheses_has_a_value() {
    for (text, present) in [("single a", true), ("single", false)] {
        let input = Input::from(text);
        let (arena, parser_arena) = (Arena::new(), Arena::new());
        let tree = SequenceOperandsParser::new(&input, &parser_arena)
            .parse_single(&arena)
            .unwrap()
            .tree;
        let Single::Alt0 { opt_1, .. } = tree.node else {
            panic!("unexpected node {:?}", tree.node);
        };
        assert_eq!(opt_1.value().is_some(), present);
    }
}
