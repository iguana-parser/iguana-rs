// Document symbols for iggy grammars.
//
// Lists jump-target definitions in the file:
// - Syntax rules                            -> SymbolKind::CLASS
// - @NoLayout syntax rules                  -> SymbolKind::INTERFACE
// - Regex rules                             -> SymbolKind::ENUM
// - Alternative labels (#Name)              -> SymbolKind::CONSTRUCTOR (children of their rule)
//
// Skipped: layout def, field labels (left:, right:), symbol references in bodies.

use by_address::ByAddress;
use iguana::grammar::def::{GrammarDef, LayoutStrategy};
use iguana::grammar::symbols::DefinitionId;
use iguana_runtime::{input::Input, sppf::Span};
use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

use crate::spans::GrammarSpans;

pub fn document_symbols(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
) -> Vec<DocumentSymbol> {
    let mut out = Vec::new();

    let num_lexical = grammar_def.lexical_rules.len();

    for (i, rule) in grammar_def.syntax_rules.iter().enumerate() {
        let Some(meta) = spans.syntax_rules.get(&ByAddress(rule)) else {
            continue;
        };
        let Some(rule_span) = meta.span else {
            continue;
        };

        let mut range = to_range(rule_span, input);
        if let Some(first) = meta.leading_comments.first() {
            let (l, c) = input.line_column(first.left_extent);
            range.start = Position::new(l, c);
        }
        if let Some(trailing) = meta.trailing_comment {
            let (l, c) = input.line_column(trailing.right_extent);
            range.end = Position::new(l, c);
        }

        let def_id = DefinitionId((num_lexical + i) as u16);
        let head_span = spans
            .definition_spans
            .get(&def_id)
            .copied()
            .unwrap_or(rule_span);

        let mut children: Vec<DocumentSymbol> = Vec::new();
        for level in &rule.priority_levels {
            for alt in &level.alternatives {
                if let Some(ref label) = alt.label {
                    if let Some(alt_meta) = spans.alternatives.get(&ByAddress(alt)) {
                        if let Some(alt_span) = alt_meta.span {
                            #[allow(deprecated)]
                            children.push(DocumentSymbol {
                                name: label.clone(),
                                detail: None,
                                kind: SymbolKind::CONSTRUCTOR,
                                tags: None,
                                deprecated: None,
                                range: to_range(alt_span, input),
                                selection_range: to_range(alt_span, input),
                                children: None,
                            });
                        }
                    }
                }
            }
        }

        #[allow(deprecated)]
        out.push(DocumentSymbol {
            name: rule.head.name.clone(),
            detail: None,
            // INTERFACE distinguishes @NoLayout rules from regular syntax rules
            // (CLASS) in the outline view.
            kind: if matches!(rule.layout, LayoutStrategy::None) {
                SymbolKind::INTERFACE
            } else {
                SymbolKind::CLASS
            },
            tags: None,
            deprecated: None,
            range,
            selection_range: to_range(head_span, input),
            children: if children.is_empty() {
                None
            } else {
                Some(children)
            },
        });
    }

    for (i, rule) in grammar_def.lexical_rules.iter().enumerate() {
        let Some(meta) = spans.lexical_rules.get(&ByAddress(rule)) else {
            continue;
        };
        let Some(rule_span) = meta.span else {
            continue;
        };

        let mut range = to_range(rule_span, input);
        if let Some(first) = meta.leading_comments.first() {
            let (l, c) = input.line_column(first.left_extent);
            range.start = Position::new(l, c);
        }
        if let Some(trailing) = meta.trailing_comment {
            let (l, c) = input.line_column(trailing.right_extent);
            range.end = Position::new(l, c);
        }

        let def_id = DefinitionId(i as u16);
        let head_span = spans
            .definition_spans
            .get(&def_id)
            .copied()
            .unwrap_or(rule_span);

        #[allow(deprecated)]
        out.push(DocumentSymbol {
            name: rule.head.name.clone(),
            detail: None,
            kind: SymbolKind::ENUM,
            tags: None,
            deprecated: None,
            range,
            selection_range: to_range(head_span, input),
            children: None,
        });
    }

    out
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

    fn symbols(source: &str) -> Vec<DocumentSymbol> {
        use iguana_runtime::input::Input;
        let source = source.strip_prefix('\n').unwrap_or(source);
        let input = Input::from(source);
        let crate::BuildResult::Success { ref tree, .. } = crate::build(&input) else {
            return vec![];
        };
        let Some(grammar_def) = crate::build_grammar_def(tree, &input) else {
            return vec![];
        };
        let spans = crate::build_spans(&grammar_def, tree, &input);
        document_symbols(&grammar_def, &spans, &input)
    }

    #[test]
    fn nonterminals_and_terminals() {
        let s = symbols(r#"
grammar T

Expr
  = "x"

@regex
Number = [0-9]+
"#);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].name, "Expr");
        assert_eq!(s[0].kind, SymbolKind::CLASS);
        assert_eq!(s[1].name, "Number");
        assert_eq!(s[1].kind, SymbolKind::ENUM);
    }

    #[test]
    fn labels_become_children() {
        let s = symbols(r#"
grammar T

Expr
  = l:Expr "+" r:Expr #Add
  | l:Expr "*" r:Expr #Mul
  | Number #Lit

@regex
Number = [0-9]+
"#);
        assert_eq!(s.len(), 2);
        let children = s[0].children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].name, "Add");
        assert_eq!(children[0].kind, SymbolKind::CONSTRUCTOR);
        assert_eq!(children[1].name, "Mul");
        assert_eq!(children[2].name, "Lit");
        assert_eq!(s[1].name, "Number");
        assert_eq!(s[1].kind, SymbolKind::ENUM);
    }

    #[test]
    fn unlabeled_alternative_no_child() {
        let s = symbols(r#"
grammar T

A
  = "x"
"#);
        assert_eq!(s.len(), 1);
        assert!(s[0].children.is_none());
    }

    #[test]
    fn parse_failure_returns_empty() {
        assert!(symbols("not a grammar {{{").is_empty());
    }

    #[test]
    fn leading_comment_block_extends_range_start() {
        let s = symbols(r#"
grammar T

// first
// second
Expr
  = "x"
"#);
        assert_eq!(s.len(), 1);
        // Range should start at line 2 (zero-based), col 0 — the `// first`
        assert_eq!(s[0].range.start.line, 2);
        assert_eq!(s[0].range.start.character, 0);
        // Selection range still points at the head
        assert_eq!(s[0].selection_range.start.line, 4);
    }

    #[test]
    fn trailing_same_line_comment_extends_range_end() {
        let s = symbols(r#"
grammar T

Expr
  = "x" // trailing
"#);
        assert_eq!(s.len(), 1);
        // Range end should sit past the trailing comment.
        assert_eq!(s[0].range.end.line, 3);
        // Character should be after `// trailing`
        assert!(s[0].range.end.character >= 18, "got {}", s[0].range.end.character);
    }

    /// Rule ranges must not overlap: each rule's range.end must be <= the
    /// next rule's range.start. This catches the bug where parse tree spans
    /// include trailing layout and bleed into the next rule.
    #[test]
    fn rule_ranges_do_not_overlap() {
        let s = symbols(r#"
grammar T

A
  = B C?

B
  = "x"*

C
  = "y"+
"#);
        assert_eq!(s.len(), 3);
        for i in 0..s.len() - 1 {
            let end = &s[i].range.end;
            let start = &s[i + 1].range.start;
            assert!(
                (end.line, end.character) <= (start.line, start.character),
                "{} range end ({},{}) overlaps {} range start ({},{})",
                s[i].name,
                end.line,
                end.character,
                s[i + 1].name,
                start.line,
                start.character,
            );
        }
    }

    /// The last rule in the file must not have a range extending past the
    /// last line of actual content (the bug that triggered the println in
    /// line_column when right_extent == input.len()).
    #[test]
    fn last_rule_range_does_not_exceed_content() {
        let s = symbols(r#"
grammar T

@regex
Id = [a-z]+
"#);
        assert_eq!(s.len(), 1);
        // Line 3 (zero-based) is `Id = [a-z]+`, the range end must sit on
        // that line, not beyond it.
        assert_eq!(s[0].range.end.line, 3);
    }

    #[test]
    fn multiple_rules_with_nullable_symbols() {
        let s = symbols(r#"
grammar T

A
  = "a" B?

B
  = "b"
"#);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].name, "A");
        assert_eq!(s[1].name, "B");
        // A's range must end before B's range starts.
        assert!(
            s[0].range.end.line < s[1].range.start.line
                || (s[0].range.end.line == s[1].range.start.line
                    && s[0].range.end.character <= s[1].range.start.character),
        );
    }

    #[test]
    fn nested_optional_and_star_symbols() {
        let s = symbols(r#"
grammar T

A
  = ("x" B?)*

B
  = "y"
"#);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].name, "A");
        assert_eq!(s[1].name, "B");
        assert!(s[0].range.end.line < s[1].range.start.line);
    }

    #[test]
    fn blank_line_separated_comment_is_not_leading() {
        let s = symbols(r#"
grammar T

// floating

Expr
  = "x"
"#);
        assert_eq!(s.len(), 1);
        // The comment is separated from Expr by a blank line, so it's NOT
        // attached. Range should start at the head.
        assert_eq!(s[0].range.start.line, 4);
        assert_eq!(s[0].range.start.character, 0);
    }
}
