// Document symbols for iggy grammars.
//
// Lists jump-target definitions in the file:
// - Nonterminal rule heads (syntax rules)   -> SymbolKind::CLASS
// - Terminal rule heads (regex rules)       -> SymbolKind::ENUM
// - Alternative labels (#Name)              -> SymbolKind::CONSTRUCTOR (children of their rule)
//
// Skipped: layout def, field labels (left:, right:), symbol references in bodies.

use crate::layout::{leading_comments, trailing_comment};
use crate::ParseResult;
use iggy::parse_tree::*;
use iguana_runtime::input::Input;
use iguana_runtime::sppf::Span;
use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

/// Walk down the right spine of `node`, skipping `Layout` children, and
/// return the right_extent of the rightmost non-layout `Token` leaf. Iggy's
/// typed parse tree folds trailing layout into each non-leaf node's `.span()`,
/// so for things like a top-level rule, `node.span().right_extent` overshoots
/// into the next rule. This walk finds the actual end of the node's content.
fn rightmost_token_end(node: ParseTreeRef<'_>) -> Option<u32> {
    if let ParseTreeRef::Token(t) = node {
        return Some(t.span().right_extent);
    }
    for child in node.children().iter().rev() {
        if matches!(child, ParseTreeRef::Layout(_)) {
            continue;
        }
        if let Some(end) = rightmost_token_end(*child) {
            return Some(end);
        }
    }
    None
}

pub fn document_symbols(result: &ParseResult) -> Vec<DocumentSymbol> {
    let Some(ref tree) = result.tree else {
        return vec![];
    };
    let ParseTree::StartGrammar(start) = tree else {
        return vec![];
    };
    let grammar = &start.start;

    let mut walker = Walker {
        input: &result.input,
        last_layout: None,
        out: Vec::new(),
    };
    walker.walk(grammar.as_parse_tree_ref());
    walker.out
}

/// Depth-first traversal of the grammar parse tree. Maintains a single piece
/// of state — the most recently visited `Layout` node, no matter how deeply
/// nested. Whenever we arrive at a `Rule`, that variable holds the layout
/// that immediately precedes the rule's head, which we use to attach leading
/// `// ...` comments. After recursing through the rule's subtree, the same
/// variable now holds the rule's *trailing* layout (the last layout visited
/// inside the subtree), which we use to attach a trailing same-line comment.
struct Walker<'a> {
    input: &'a Input,
    last_layout: Option<&'a Layout>,
    out: Vec<DocumentSymbol>,
}

impl<'a> Walker<'a> {
    fn walk(&mut self, node: ParseTreeRef<'a>) {
        match node {
            ParseTreeRef::Layout(l) => {
                // Skip empty layouts so they don't overwrite the previous
                // meaningful one.
                if !l.span().is_empty() {
                    self.last_layout = Some(l);
                }
                // Don't descend into layouts.
            }
            ParseTreeRef::Rule(rule) => {
                let (mut symbol, head_start, content_end) = match rule {
                    Rule::SyntaxRule { syntax_rule, .. } => {
                        let s = syntax_rule_symbol(syntax_rule, self.input);
                        let head_start = syntax_rule.head.span().left_extent;
                        let end = rightmost_token_end(syntax_rule.as_parse_tree_ref())
                            .unwrap_or(syntax_rule.head.span().right_extent);
                        (s, head_start, end)
                    }
                    Rule::RegexRule { regex_rule, .. } => {
                        let s = regex_rule_symbol(regex_rule, self.input);
                        let head_start = regex_rule.identifier.span().left_extent;
                        let end = rightmost_token_end(regex_rule.as_parse_tree_ref())
                            .unwrap_or(regex_rule.identifier.span().right_extent);
                        (s, head_start, end)
                    }
                };

                // Leading: extend range start to include consecutive `//`
                // comments immediately above the rule's head.
                if let Some(layout) = self.last_layout {
                    let leading = leading_comments(layout, self.input, head_start);
                    if let Some(first) = leading.first() {
                        let (l, c) = self.input.line_column(first.span().left_extent);
                        symbol.range.start = Position::new(l, c);
                    }
                }

                // Recurse into the rule subtree purely as a side effect: we
                // need to keep visiting Layout nodes so `last_layout` ends up
                // pointing at this rule's trailing layout (the last layout
                // visited inside the subtree).
                for child in node.children().iter() {
                    self.walk(*child);
                }

                // Trailing: extend range end to include a same-line `// ...`
                // comment sitting right after the rule's content.
                if let Some(layout) = self.last_layout {
                    if let Some(c) = trailing_comment(layout, self.input, content_end) {
                        let (l, col) = self.input.line_column(c.span().right_extent);
                        symbol.range.end = Position::new(l, col);
                    }
                }

                self.out.push(symbol);
            }
            _ => {
                for child in node.children().iter() {
                    self.walk(*child);
                }
            }
        }
    }
}

fn syntax_rule_symbol(rule: &SyntaxRule, input: &Input) -> DocumentSymbol {
    let name = input.substring(rule.head.span().left_extent, rule.head.span().right_extent);
    let start = rule.head.span().left_extent;
    let end = rightmost_token_end(rule.as_parse_tree_ref())
        .unwrap_or(rule.head.span().right_extent);
    let range_span = Span { left_extent: start, right_extent: end };

    let mut children: Vec<DocumentSymbol> = Vec::new();
    for level in rule.priority_levels.priority_levels() {
        for alt in level.alternatives.alternatives() {
            if let Some(label) = alt.label.value() {
                let label_span = label.span();
                let label_text = input.substring(label_span.left_extent, label_span.right_extent);
                #[allow(deprecated)]
                let alt_start = alt.span().left_extent;
                let alt_end = rightmost_token_end(alt.as_parse_tree_ref())
                    .unwrap_or(label_span.right_extent);
                children.push(DocumentSymbol {
                    name: label_text,
                    detail: None,
                    kind: SymbolKind::CONSTRUCTOR,
                    tags: None,
                    deprecated: None,
                    range: to_range(Span { left_extent: alt_start, right_extent: alt_end }, input),
                    selection_range: to_range(label_span, input),
                    children: None,
                });
            }
        }
    }

    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail: None,
        kind: SymbolKind::CLASS,
        tags: None,
        deprecated: None,
        range: to_range(range_span, input),
        selection_range: to_range(rule.head.span(), input),
        children: if children.is_empty() { None } else { Some(children) },
    }
}

fn regex_rule_symbol(rule: &RegexRule, input: &Input) -> DocumentSymbol {
    let name = input.substring(
        rule.identifier.span().left_extent,
        rule.identifier.span().right_extent,
    );
    let start = rule.identifier.span().left_extent;
    let end = rightmost_token_end(rule.as_parse_tree_ref())
        .unwrap_or(rule.identifier.span().right_extent);
    let range_span = Span { left_extent: start, right_extent: end };
    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail: None,
        kind: SymbolKind::ENUM,
        tags: None,
        deprecated: None,
        range: to_range(range_span, input),
        selection_range: to_range(rule.identifier.span(), input),
        children: None,
    }
}

fn to_range(span: Span, input: &Input) -> Range {
    let (sl, sc) = input.line_column(span.left_extent);
    let (el, ec) = input.line_column(span.right_extent);
    Range {
        start: Position::new(sl, sc),
        end: Position::new(el, ec),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syms(source: &str) -> Vec<DocumentSymbol> {
        document_symbols(&crate::parse(source))
    }

    #[test]
    fn nonterminals_and_terminals() {
        let s = syms("grammar T\n\nExpr\n  = \"x\"\n\n@regex\nNumber = [0-9]+\n");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].name, "Expr");
        assert_eq!(s[0].kind, SymbolKind::CLASS);
        assert_eq!(s[1].name, "Number");
        assert_eq!(s[1].kind, SymbolKind::ENUM);
    }

    #[test]
    fn labels_become_children() {
        let s = syms(
            "grammar T\n\nExpr\n  = l:Expr \"+\" r:Expr #Add\n  | l:Expr \"*\" r:Expr #Mul\n  | Number #Lit\n\n@regex\nNumber = [0-9]+\n",
        );
        assert_eq!(s.len(), 2);
        let children = s[0].children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].name, "#Add");
        assert_eq!(children[0].kind, SymbolKind::CONSTRUCTOR);
        assert_eq!(children[1].name, "#Mul");
        assert_eq!(children[2].name, "#Lit");
        assert_eq!(s[1].name, "Number");
        assert_eq!(s[1].kind, SymbolKind::ENUM);
    }

    #[test]
    fn unlabeled_alternative_no_child() {
        let s = syms("grammar T\n\nA\n  = \"x\"\n");
        assert_eq!(s.len(), 1);
        assert!(s[0].children.is_none());
    }

    #[test]
    fn parse_failure_returns_empty() {
        assert!(syms("not a grammar {{{").is_empty());
    }

    #[test]
    fn leading_comment_block_extends_range_start() {
        // // first comment line
        // // second comment line
        // Expr
        //   = "x"
        let src = "grammar T\n\n// first\n// second\nExpr\n  = \"x\"\n";
        let s = syms(src);
        assert_eq!(s.len(), 1);
        // Range should start at line 2 (zero-based), col 0 — the `// first`
        assert_eq!(s[0].range.start.line, 2);
        assert_eq!(s[0].range.start.character, 0);
        // Selection range still points at the head
        assert_eq!(s[0].selection_range.start.line, 4);
    }

    #[test]
    fn trailing_same_line_comment_extends_range_end() {
        // Expr  // trailing
        //   = "x"
        // Wait, the trailing comment must be on the same line as the rule's
        // last token, which is `"x"` on line 1 (zero-based).
        let src = "grammar T\n\nExpr\n  = \"x\" // trailing\n";
        let s = syms(src);
        assert_eq!(s.len(), 1);
        // Range end should sit past the trailing comment.
        assert_eq!(s[0].range.end.line, 3);
        // Character should be after `// trailing`
        assert!(s[0].range.end.character >= 18, "got {}", s[0].range.end.character);
    }

    #[test]
    fn blank_line_separated_comment_is_not_leading() {
        // // floating
        //
        // Expr
        //   = "x"
        let src = "grammar T\n\n// floating\n\nExpr\n  = \"x\"\n";
        let s = syms(src);
        assert_eq!(s.len(), 1);
        // The comment is separated from Expr by a blank line, so it's NOT
        // attached. Range should start at the head.
        assert_eq!(s[0].range.start.line, 4);
        assert_eq!(s[0].range.start.character, 0);
    }
}
