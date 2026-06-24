// Folding ranges for iggy grammars.
//
// Each syntax rule and lexical rule that spans more than one line produces a
// folding range. The range starts at the rule head line and ends at the last
// line of the rule body (or trailing comment). Annotations (@Regex, @NoLayout,
// etc.) stay visible above the fold.

use by_address::ByAddress;
use iguana_compiler::grammar::def::GrammarDef;
use iguana_compiler::grammar::symbols::DefinitionId;
use iguana_runtime::input::Input;
use lsp_types::{FoldingRange, FoldingRangeKind};

use crate::spans::GrammarSpans;

pub fn folding_ranges(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
) -> Vec<FoldingRange> {
    let mut out = Vec::new();
    let num_lexical = grammar_def.lexical_rules.len();

    for (i, rule) in grammar_def.syntax_rules.iter().enumerate() {
        let Some(meta) = spans.syntax_rules.get(&ByAddress(rule)) else {
            continue;
        };
        let Some(rule_span) = meta.span else {
            continue;
        };

        let def_id = DefinitionId((num_lexical + i) as u16);
        let head_span = spans.definition_spans.get(&def_id);
        // Fold from the rule head, not the annotation. Rule spans include
        // leading annotations (@NoLayout, etc.), but those should stay
        // visible above the fold.
        let start_line = head_span
            .map(|s| input.line_column(s.left_extent).0)
            .unwrap_or_else(|| input.line_column(rule_span.left_extent).0);
        let mut end_line = input.line_column(rule_span.right_extent).0;
        if let Some(trailing) = meta.trailing_comment {
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

    for (i, rule) in grammar_def.lexical_rules.iter().enumerate() {
        let Some(meta) = spans.lexical_rules.get(&ByAddress(rule)) else {
            continue;
        };
        let Some(rule_span) = meta.span else {
            continue;
        };

        let def_id = DefinitionId(i as u16);
        let head_span = spans.definition_spans.get(&def_id);
        // Same as syntax rules: fold from the head, not @Regex etc.
        let start_line = head_span
            .map(|s| input.line_column(s.left_extent).0)
            .unwrap_or_else(|| input.line_column(rule_span.left_extent).0);
        let mut end_line = input.line_column(rule_span.right_extent).0;
        if let Some(trailing) = meta.trailing_comment {
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
        let ctx = iguana_runtime::parse_tree::ParseContext::new();
        let crate::BuildResult::Success { tree, .. } = crate::build(&input, &ctx) else {
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
