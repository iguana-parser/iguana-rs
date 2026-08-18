// Folding ranges for iggy grammars.
//
// Each syntax rule and lexical rule that spans more than one line produces a
// folding range. The range starts at the rule head line and ends at the last
// line of the rule body (or trailing comment). Annotations (@Regex, @NoLayout,
// etc.) stay visible above the fold.

use iguana_compiler::grammar::def::GrammarDef;
use iguana_runtime::input::Input;
use lsp_types::{FoldingRange, FoldingRangeKind};

use crate::spans::GrammarSpans;

pub fn folding_ranges(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
) -> Vec<FoldingRange> {
    let mut out = Vec::new();
    for rule in &grammar_def.syntax_rules {
        let Some(region) = spans.syntax_rule(rule) else {
            continue;
        };
        let rule_span = region.span;

        let head_span = spans.nonterminal(&rule.head).map(|region| region.span);
        // Fold from the rule head, not the annotation. Rule spans include
        // leading annotations (@NoLayout, etc.), but those should stay
        // visible above the fold.
        let start_line = head_span
            .map(|s| input.line_column(s.left_extent).0)
            .unwrap_or_else(|| input.line_column(rule_span.left_extent).0);
        let mut end_line = input.line_column(rule_span.right_extent).0;
        if let Some(trailing) = region.trailing_comment {
            end_line = input.line_column(trailing.right_extent).0;
        }

        if end_line > start_line {
            out.push(FoldingRange {
                start_line,
                start_character: None,
                end_line,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        }
    }

    for rule in &grammar_def.lexical_rules {
        let Some(region) = spans.lexical_rule(rule) else {
            continue;
        };
        let rule_span = region.span;

        let head_span = spans.terminal(&rule.head).map(|region| region.span);
        // Same as syntax rules: fold from the head, not @Regex etc.
        let start_line = head_span
            .map(|s| input.line_column(s.left_extent).0)
            .unwrap_or_else(|| input.line_column(rule_span.left_extent).0);
        let mut end_line = input.line_column(rule_span.right_extent).0;
        if let Some(trailing) = region.trailing_comment {
            end_line = input.line_column(trailing.right_extent).0;
        }

        if end_line > start_line {
            out.push(FoldingRange {
                start_line,
                start_character: None,
                end_line,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges(source: &str) -> Vec<FoldingRange> {
        use iguana_runtime::input::Input;
        let source = source.strip_prefix('\n').unwrap_or(source);
        let input = Input::from(source);
        let tree_arena = iguana_runtime::arena::Arena::new();
        let crate::BuildResult::Success { tree, .. } = crate::build(&input, &tree_arena) else {
            return vec![];
        };
        let Some(grammar_def) = crate::build_grammar_def(tree, &input) else {
            return vec![];
        };
        let spans = crate::build_spans(&grammar_def, tree, &input);
        folding_ranges(&grammar_def, &spans, &input)
    }

    #[test]
    fn multiline_syntax_rule_folds() {
        let r = ranges(
            r#"
grammar T

Expr
  = "x"
  | "y"
"#,
        );
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].start_line, 2);
        assert_eq!(r[0].end_line, 4);
    }

    #[test]
    fn annotated_single_line_lexical_rule_no_fold() {
        let r = ranges(
            r#"
grammar T

@Regex
Id = [a-z]+
"#,
        );
        // The rule head is on a single line; @Regex stays visible, no fold.
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn multiple_rules_multiple_folds() {
        let r = ranges(
            r#"
grammar T

A
  = "a"
  | "b"

B
  = "c"
  | "d"
"#,
        );
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].start_line, 2);
        assert_eq!(r[1].start_line, 6);
    }

    #[test]
    fn annotated_multiline_syntax_rule_folds_from_head() {
        let r = ranges(
            r#"
grammar T

@NoLayout
Rule
  = "a"
  | "b"
"#,
        );
        assert_eq!(r.len(), 1);
        // Fold starts at the rule head, not the @NoLayout annotation.
        assert_eq!(r[0].start_line, 3);
        assert_eq!(r[0].end_line, 5);
    }

    #[test]
    fn parse_failure_returns_empty() {
        assert!(ranges("not a grammar {{{").is_empty());
    }
}
