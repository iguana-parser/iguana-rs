//! Syntax highlighting from a parse tree.
//!
//! A group defines the color that an editor uses. The highlighter assigns
//! spans of the input to groups.
//!
//! A group's name goes from general to specific, with a dot between the
//! parts, such as `string.escape`.
//!
//! A language's highlighting rules are two functions, `rules` and
//! `token_rules`. After a successful parse, `highlight` walks the parse tree
//! bottom up and runs `rules` on each node.
//!
//! When there is no parse tree, because the parse failed, `lexical_highlight`
//! runs the scanner on the input and `token_rules` on each token. A caller can
//! also use it for an ambiguous tree. The result is less accurate than the
//! highlighting from a parse tree, but a good approximation.
//!
//! The languages provide the actual highlighting rules.

use serde::Serialize;

use crate::{
    ids::TerminalId,
    input::{Input, Span},
    parse_tree::{NodeKind, ParseTreeNode, TreeVisitor, Visit, walk},
    scanner::Scanner,
};

/// A span of the input and its group. A group serializes as its name, such
/// as `string.escape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Highlight<G> {
    pub span: Span,
    pub group: G,
}

impl<G> Highlight<G> {
    pub fn new(span: Span, group: G) -> Self {
        Highlight { span, group }
    }
}

/// The name of `group`, as it serializes, such as `string.escape`.
pub fn group_name(group: impl Serialize) -> String {
    match serde_json::to_value(group) {
        Ok(serde_json::Value::String(name)) => name,
        _ => panic!("a highlight group serializes as a string"),
    }
}

/// Resolves the overlaps in `highlights`: the result is runs that do not
/// overlap, in input order. Where two highlights overlap, the shorter one
/// wins. An escape then keeps its group inside its string. Of two highlights
/// with the same span, the later one wins, which lets a node's rule override
/// the group of a token below it.
pub fn flatten<G: Copy>(highlights: &[Highlight<G>]) -> Vec<Highlight<G>> {
    let Some(start) = highlights.iter().map(|h| h.span.left_extent).min() else {
        return Vec::new();
    };
    let end = highlights
        .iter()
        .map(|h| h.span.right_extent)
        .max()
        .unwrap();
    // Paints the index of each highlight onto its characters, the longest
    // first. The sort is stable, so of two equal spans the later is painted
    // last.
    let mut order: Vec<usize> = (0..highlights.len()).collect();
    order.sort_by_key(|&i| {
        let span = highlights[i].span;
        std::cmp::Reverse(span.right_extent - span.left_extent)
    });
    const UNPAINTED: u32 = u32::MAX;
    let mut painted = vec![UNPAINTED; (end - start) as usize];
    for i in order {
        let span = highlights[i].span;
        painted[(span.left_extent - start) as usize..(span.right_extent - start) as usize]
            .fill(i as u32);
    }
    let mut runs = Vec::new();
    let mut position = 0;
    while position < painted.len() {
        let index = painted[position];
        let length = painted[position..]
            .iter()
            .take_while(|&&other| other == index)
            .count();
        if index != UNPAINTED {
            let left_extent = start + position as u32;
            let span = Span::new(left_extent, left_extent + length as u32);
            runs.push(Highlight::new(span, highlights[index as usize].group));
        }
        position += length;
    }
    runs
}

/// The spans of the escapes in `text`, the characters at `span`. An escape is
/// a backslash and the character after it, such as `\n`.
pub fn backslash_escapes(span: Span, text: &[char]) -> Vec<Span> {
    let mut escapes = Vec::new();
    let mut index = 0;
    while index < text.len() {
        if text[index] == '\\' {
            let end = (index + 2).min(text.len());
            let start = span.left_extent + index as u32;
            escapes.push(Span::new(start, span.left_extent + end as u32));
            index = end;
        } else {
            index += 1;
        }
    }
    escapes
}

/// The highlights of a parse tree of `input`, in input order. `rules` runs on
/// each nonterminal and each non-empty token, after the nodes below it. In an
/// ambiguity, only the first derivation is highlighted.
pub fn highlight<N: ParseTreeNode, H: Copy>(
    root: N,
    input: &Input,
    rules: impl Fn(N, &Input, &mut Vec<Highlight<H>>),
) -> Vec<Highlight<H>> {
    let mut visitor = HighlightVisitor {
        input,
        rules,
        highlights: Vec::new(),
    };
    walk(root, &mut visitor);
    flatten(&visitor.highlights)
}

struct HighlightVisitor<'h, R, H> {
    input: &'h Input,
    rules: R,
    highlights: Vec<Highlight<H>>,
}

impl<N: ParseTreeNode, H, R: Fn(N, &Input, &mut Vec<Highlight<H>>)> TreeVisitor<N>
    for HighlightVisitor<'_, R, H>
{
    fn enter(&mut self, _node: N) -> Visit {
        Visit::Children
    }

    fn exit(&mut self, node: N) {
        match node.kind() {
            NodeKind::Token if node.span().is_empty() => {}
            NodeKind::Token | NodeKind::Nonterminal => {
                (self.rules)(node, self.input, &mut self.highlights)
            }
            NodeKind::Amb | NodeKind::Cycle => {}
        }
    }

    fn children(&self, node: N) -> Vec<N> {
        match node.kind() {
            // Highlighting every derivation would give a token one highlight
            // per derivation.
            NodeKind::Amb => node.child(0).into_iter().collect(),
            // A Cycle node does not have children. It repeats a part of the
            // tree that the walk visits elsewhere.
            _ => node.children(),
        }
    }
}

/// The highlights of `input` from the scanner's tokens (`Scanner::tokens`),
/// for an input without a usable parse tree. `token` makes a token of the grammar's
/// crate from its terminal and span, and `token_rules` pushes its highlights
/// from its terminal alone. The scan can split the input differently than a
/// parse would, because it does not know where the grammar expects each
/// terminal.
pub fn lexical_highlight<S: Scanner, T, H: Copy>(
    scanner: &mut S,
    input: &Input,
    token: impl Fn(TerminalId, Span) -> T,
    token_rules: impl Fn(T, &Input, &mut Vec<Highlight<H>>),
) -> Vec<Highlight<H>> {
    let mut highlights = Vec::new();
    for scanned in scanner.tokens() {
        token_rules(
            token(scanned.terminal, scanned.span),
            input,
            &mut highlights,
        );
    }
    flatten(&highlights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::NonterminalId;

    /// The tests' groups are strings.
    type Group = &'static str;

    /// The text and group of each highlight.
    fn pieces(input: &Input, highlights: &[Highlight<Group>]) -> Vec<(String, Group)> {
        highlights
            .iter()
            .map(|highlight| (input.text(highlight.span), highlight.group))
            .collect()
    }

    fn expected(pieces: &[(&str, Group)]) -> Vec<(String, Group)> {
        pieces
            .iter()
            .map(|&(text, group)| (text.to_string(), group))
            .collect()
    }

    #[test]
    fn a_highlight_serializes_its_group_by_name() {
        #[derive(Clone, Copy, Serialize)]
        enum Escape {
            #[serde(rename = "string.escape")]
            StringEscape,
        }
        let highlight = Highlight::new(Span::new(0, 2), Escape::StringEscape);
        assert_eq!(
            serde_json::to_value(highlight).unwrap(),
            serde_json::json!({ "span": serde_json::to_value(Span::new(0, 2)).unwrap(), "group": "string.escape" })
        );
        assert_eq!(group_name(Escape::StringEscape), "string.escape");
    }

    #[test]
    fn a_backslash_escapes_the_character_after_it() {
        let input = Input::from(r#"x"a\"b\\""#);
        let span = Span::new(1, input.len());
        let escapes: Vec<_> = backslash_escapes(span, input.chars(span))
            .into_iter()
            .map(|escape| input.text(escape))
            .collect();
        assert_eq!(escapes, [r#"\""#, r"\\"]);
        // A backslash at the end is an escape of one character.
        let input = Input::from("a\\");
        let span = Span::new(0, 2);
        assert_eq!(
            backslash_escapes(span, input.chars(span)),
            [Span::new(1, 2)]
        );
    }

    fn at(start: u32, end: u32, group: Group) -> Highlight<Group> {
        Highlight::new(Span::new(start, end), group)
    }

    #[test]
    fn of_two_highlights_with_the_same_span_the_later_wins() {
        let input = Input::from("lhs : E");
        let highlights = [at(0, 3, "rule"), at(4, 5, "operator"), at(0, 3, "property")];
        assert_eq!(
            pieces(&input, &flatten(&highlights)),
            expected(&[("lhs", "property"), (":", "operator")])
        );
    }

    #[test]
    fn a_shorter_highlight_wins_inside_a_longer_one() {
        let input = Input::from(r#""a\nb\t""#);
        // One escape is pushed before the string and one after it. Both
        // override the string, and the characters around them stay `string`.
        let highlights = [
            at(2, 4, "string.escape"),
            at(0, 8, "string"),
            at(5, 7, "string.escape"),
        ];
        assert_eq!(
            pieces(&input, &flatten(&highlights)),
            expected(&[
                ("\"a", "string"),
                (r"\n", "string.escape"),
                ("b", "string"),
                (r"\t", "string.escape"),
                ("\"", "string"),
            ])
        );
    }

    #[test]
    fn of_two_highlights_that_partly_overlap_the_shorter_wins_the_overlap() {
        let input = Input::from("abcde");
        let highlights = [at(0, 2, "label"), at(1, 5, "type")];
        assert_eq!(
            pieces(&input, &flatten(&highlights)),
            expected(&[("ab", "label"), ("cde", "type")])
        );
    }

    #[test]
    fn adjacent_highlights_with_the_same_group_stay_two_runs() {
        let input = Input::from("ab");
        let highlights = [at(1, 2, "rule"), at(0, 1, "rule")];
        assert_eq!(
            pieces(&input, &flatten(&highlights)),
            expected(&[("a", "rule"), ("b", "rule")])
        );
    }

    #[test]
    fn the_gaps_between_highlights_and_empty_highlights_have_no_run() {
        let input = Input::from("a b");
        let highlights = [at(1, 1, "operator"), at(2, 3, "rule")];
        assert_eq!(
            pieces(&input, &flatten(&highlights)),
            expected(&[("b", "rule")])
        );
        assert_eq!(flatten::<Group>(&[]), []);
    }

    /// A node of a hand-built tree.
    #[derive(Clone, Copy)]
    struct Node(&'static NodeData);

    struct NodeData {
        name: &'static str,
        kind: NodeKind,
        span: (u32, u32),
        children: &'static [NodeData],
    }

    const fn token(name: &'static str, start: u32, end: u32) -> NodeData {
        NodeData {
            name,
            kind: NodeKind::Token,
            span: (start, end),
            children: &[],
        }
    }

    impl ParseTreeNode for Node {
        fn child(&self, index: usize) -> Option<Self> {
            self.0.children.get(index).map(Node)
        }
        fn child_count(&self) -> usize {
            self.0.children.len()
        }
        fn display_name(&self) -> &'static str {
            self.0.name
        }
        fn span(&self) -> Span {
            Span::new(self.0.span.0, self.0.span.1)
        }
        fn kind(&self) -> NodeKind {
            self.0.kind
        }
        fn node_id(&self) -> Option<usize> {
            None
        }
        fn nonterminal_id(&self) -> Option<NonterminalId> {
            None
        }
    }

    // `a+b` with two derivations. Only the first one, an `E`, is
    // highlighted. The rules give the `E` the group `function` and the `+`
    // the group `operator`, which is shorter and wins inside the `E`.
    #[test]
    fn the_first_derivation_of_an_ambiguity_is_highlighted() {
        static TREE: NodeData = NodeData {
            name: "E",
            kind: NodeKind::Amb,
            span: (0, 3),
            children: &[
                NodeData {
                    name: "E",
                    kind: NodeKind::Nonterminal,
                    span: (0, 3),
                    children: &[token("a", 0, 1), token("\"+\"", 1, 2), token("b", 2, 3)],
                },
                NodeData {
                    name: "S",
                    kind: NodeKind::Nonterminal,
                    span: (0, 3),
                    children: &[token("\"a+\"", 0, 2), token("b", 2, 3)],
                },
            ],
        };
        let input = Input::from("a+b");
        let rules = |node: Node, _: &Input, highlights: &mut Vec<Highlight<Group>>| {
            if node.kind() == NodeKind::Nonterminal {
                highlights.push(Highlight::new(node.span(), "function"));
            } else if node.0.name == "\"+\"" {
                highlights.push(Highlight::new(node.span(), "operator"));
            }
        };
        let highlights = highlight(Node(&TREE), &input, rules);
        assert_eq!(
            pieces(&input, &highlights),
            expected(&[("a", "function"), ("+", "operator"), ("b", "function")])
        );
    }

    // `lhs:E`, where the rule of the token `lhs` makes it a `rule`, and the
    // rule of the node above it a `property`. The node's rule runs later and
    // wins.
    #[test]
    fn a_nodes_rule_overrides_the_group_of_a_token_below_it() {
        static TREE: NodeData = NodeData {
            name: "Symbol",
            kind: NodeKind::Nonterminal,
            span: (0, 5),
            children: &[
                token("Identifier", 0, 3),
                token("\":\"", 3, 4),
                token("Identifier", 4, 5),
            ],
        };
        let input = Input::from("lhs:E");
        let rules =
            |node: Node, _: &Input, highlights: &mut Vec<Highlight<Group>>| match node.0.name {
                "Identifier" => highlights.push(Highlight::new(node.span(), "rule")),
                "Symbol" => {
                    let field = node.child(0).unwrap();
                    highlights.push(Highlight::new(field.span(), "property"));
                }
                _ => {}
            };
        let highlights = highlight(Node(&TREE), &input, rules);
        assert_eq!(
            pieces(&input, &highlights),
            expected(&[("lhs", "property"), ("E", "rule")])
        );
    }
}
