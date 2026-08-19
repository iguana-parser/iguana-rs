// LSP diagnostics for iggy grammars. The checks are the compiler's grammar
// validation, so the editor and `iguana generate` report the same errors.
// Each error becomes a `Diagnostic`.

use iguana_compiler::grammar::def::GrammarDef;
use iguana_compiler::validation::{GrammarError, validate};
use iguana_runtime::input::Input;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::spans::GrammarSpans;

pub fn diagnostics<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    input: &Input,
) -> Vec<Diagnostic> {
    validate(grammar_def, spans)
        .into_iter()
        .map(|error| to_diagnostic(error, input))
        .collect()
}

/// A grammar error as an editor diagnostic, with the span converted to a
/// range of line and column positions.
pub fn to_diagnostic(error: GrammarError, input: &Input) -> Diagnostic {
    let (start_line, start_column) = input.line_column(error.span.left_extent);
    let (end_line, end_column) = input.line_column(error.span.right_extent);
    Diagnostic {
        range: Range {
            start: Position::new(start_line, start_column),
            end: Position::new(end_line, end_column),
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: error.message,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diags(source: &str) -> Vec<Diagnostic> {
        use iguana_runtime::input::Input;
        let source = source.strip_prefix('\n').unwrap_or(source);
        let input = Input::from(source);
        let tree_arena = iguana_runtime::arena::Arena::new();
        let crate::BuildResult::Success { tree, .. } = crate::build(&input, &tree_arena) else {
            return vec![];
        };
        let grammar_def = crate::build_grammar_def(tree, &input);
        let spans = crate::build_spans(&grammar_def, tree, &input);
        diagnostics(&grammar_def, &spans, &input)
    }

    /// A parse failure at the end of the input has the empty span at the
    /// input length. The conversion accepts that boundary offset and
    /// produces a zero-width range there.
    #[test]
    fn parse_error_at_end_of_input() {
        let input = Input::from("grammar");
        let tree_arena = iguana_runtime::arena::Arena::new();
        let crate::BuildResult::Error(error) = crate::build(&input, &tree_arena) else {
            panic!("expected a parse error");
        };
        let d = to_diagnostic(error, &input);
        assert_eq!(d.range.start, Position::new(0, 7));
        assert_eq!(d.range.end, Position::new(0, 7));
    }

    #[test]
    fn no_errors_on_valid_grammar() {
        let d = diags(
            r#"
grammar T

Expr
  = "x"
"#,
        );
        assert!(d.is_empty());
    }

    #[test]
    fn unresolved_identifier_reported() {
        let d = diags(
            r#"
grammar T

Expr
  = Foo
"#,
        );
        assert_eq!(d.len(), 1);
        assert!(d[0].message.contains("Foo"));
        assert_eq!(d[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn multiple_unresolved() {
        let d = diags(
            r#"
grammar T

Expr
  = Foo Bar
"#,
        );
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn resolved_reference_no_error() {
        let d = diags(
            r#"
grammar T

Expr
  = Term

Term
  = "x"
"#,
        );
        assert!(d.is_empty());
    }

    #[test]
    fn unresolved_exclude_label() {
        let d = diags(
            r#"
grammar T

Expression
  = Primary                         #Primary
  > left Expression "+" Expression
  > right Expression!NoSuchLabel "=" Expression

Primary
  = "x"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 1, "expected 1 diagnostic, got: {names:?}");
        assert!(d[0].message.contains("NoSuchLabel"));
    }

    #[test]
    fn valid_exclude_label_no_error() {
        let d = diags(
            r#"
grammar T

Expression
  = Primary                         #Primary
  > left Expression "+" Expression  #Add
  > right Expression!Add "=" Expression

Primary
  = "x"
"#,
        );
        assert!(
            d.is_empty(),
            "expected no diagnostics, got: {:?}",
            d.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn unresolved_identifier_in_priority_level() {
        let d = diags(
            r#"
grammar T

Expression
  = Primary                         #Primary
  > left Expression "+" Expression
  > right Expression Foo Expression

Primary
  = "x"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 1, "expected 1 diagnostic for Foo, got: {names:?}");
        assert!(d[0].message.contains("Foo"));
    }

    #[test]
    fn unresolved_identifier_next_to_exclude() {
        let d = diags(
            r#"
grammar T

Expression
  = Primary                                  #Primary
  > left Expression "+" Expression           #Add
  > right Expression!Add Foo Expression

Primary
  = "x"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 1, "expected 1 diagnostic for Foo, got: {names:?}");
        assert!(d[0].message.contains("Foo"));
    }

    /// One `Restricted` node holds all of a symbol's restrictions, and the
    /// spans come from a chain of parse-tree nodes. A kind split over two
    /// nodes has to pair each of its identifiers with the token it was built
    /// from, or an unresolved name in the second node goes unreported.
    #[test]
    fn unresolved_identifiers_in_a_split_restriction_chain() {
        let d = diags(
            r#"
grammar T

S
  = A !>> Undef1 \ Undef2 !>> Undef3 A

A
  = "a"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(
            d.len(),
            3,
            "expected one per undefined name, got: {names:?}"
        );
        for name in ["Undef1", "Undef2", "Undef3"] {
            assert!(
                d.iter().any(|d| d.message.contains(name)),
                "{name} went unreported, got: {names:?}",
            );
        }
    }

    /// An exclusion written outside the restrictions puts an `Exclude` node
    /// above the restriction nodes in the parse-tree chain. The grammar
    /// symbol keeps it below the `Restricted` node.
    #[test]
    fn unresolved_restriction_under_an_exclusion() {
        let d = diags(
            r#"
grammar T

S
  = A !>> Undef !Alt A

A
  = "a"    #Alt
  | "b"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(
            d.len(),
            1,
            "expected 1 diagnostic for Undef, got: {names:?}"
        );
        assert!(d[0].message.contains("Undef"));
    }

    /// Two exclusions written separately land in one label list, and both
    /// labels are checked.
    #[test]
    fn two_separate_exclusions() {
        let d = diags(
            r#"
grammar T

S
  = A !NoSuch1 !NoSuch2 A

A
  = "a"    #Alt
  | "b"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 2, "expected one per unknown label, got: {names:?}");
        for name in ["NoSuch1", "NoSuch2"] {
            assert!(
                d.iter().any(|d| d.message.contains(name)),
                "{name} went unreported, got: {names:?}",
            );
        }
    }

    /// Exclusions written on both sides of a follow restriction land in one
    /// label list, and each label diagnostic points at its own token.
    #[test]
    fn exclusions_split_around_a_follow_restriction() {
        let d = diags(
            r#"
grammar T

S
  = A!NoSuch1 !>> B !NoSuch2 A

A
  = "a"    #Alt
  | "b"

@Regex
B
  = "b"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 2, "expected one per unknown label, got: {names:?}");
        let line = "  = A!NoSuch1 !>> B !NoSuch2 A";
        for name in ["NoSuch1", "NoSuch2"] {
            let character = line.find(name).unwrap() as u32;
            assert!(
                d.iter().any(|d| d.message.contains(name)
                    && d.range.start.line == 3
                    && d.range.start.character == character),
                "{name} not reported at its own token, got: {d:?}",
            );
        }
    }

    /// A restriction name written twice is stored once, keeping its first
    /// spelling. A later restriction with a different name pairs with its
    /// own token, so its diagnostic points at that token rather than at the
    /// repeated one.
    #[test]
    fn duplicate_restriction_leaves_later_spans_aligned() {
        let d = diags(
            r#"
grammar T

S
  = A !>> X !>> X !>> Y A

A
  = "a"
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 2, "one per stored name, got: {names:?}");
        let line = "  = A !>> X !>> X !>> Y A";
        for name in ["X", "Y"] {
            let character = line.find(name).unwrap() as u32;
            assert!(
                d.iter().any(|d| d.message.contains(name)
                    && d.range.start.line == 3
                    && d.range.start.character == character),
                "{name} not reported at its own token, got: {d:?}",
            );
        }
    }

    #[test]
    fn unresolved_regex_identifier() {
        let d = diags(
            r#"
grammar T

@Regex
DecimalLiteral
  = Digits ExponentPart

@Regex
Digits
  = [0-9]+
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(
            d.len(),
            1,
            "expected 1 diagnostic for ExponentPart, got: {names:?}"
        );
        assert!(d[0].message.contains("ExponentPart"));
    }

    #[test]
    fn resolved_regex_identifier_no_error() {
        let d = diags(
            r#"
grammar T

@Regex
DecimalLiteral
  = Digits ExponentPart

@Regex
Digits
  = [0-9]+

@Regex
ExponentPart
  = [eE] [0-9]+
"#,
        );
        assert!(
            d.is_empty(),
            "expected no diagnostics, got: {:?}",
            d.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn unresolved_regex_identifier_in_nested_regex() {
        let d = diags(
            r#"
grammar T

@Regex
FloatLiteral
  = [0-9]+ Suffix?

@Regex
Digits
  = [0-9]+
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(
            d.len(),
            1,
            "expected 1 diagnostic for Suffix, got: {names:?}"
        );
        assert!(d[0].message.contains("Suffix"));
    }

    /// A lexical rule's restrictions are not part of its regex body. Their
    /// operands need separate span mappings so diagnostics mark the right
    /// names.
    #[test]
    fn restriction_on_a_lexical_rule() {
        let d = diags(
            r#"
grammar T

S
  = Kw

@Regex
Kw
  = "if" !>> Undefined
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 1, "expected 1 diagnostic, got: {names:?}");
        assert!(d[0].message.contains("Undefined"));
        assert_eq!(d[0].range.start.line, 7);
        assert_eq!(d[0].range.start.character, 13);
    }

    /// Identifier-rule references are not resolved, so this diagnostic finds
    /// the corresponding lexical rule head by name.
    #[test]
    fn layout_rule_annotated_as_an_identifier_rule() {
        let d = diags(
            r#"
grammar T

S
  = "x"

@Layout @Identifier @Regex
WS
  = [\ ]*
"#,
        );
        let names: Vec<_> = d.iter().map(|d| &d.message).collect();
        assert_eq!(d.len(), 1, "expected 1 diagnostic for WS, got: {names:?}");
        assert!(d[0].message.contains("WS"));
        assert_eq!(d[0].range.start.line, 6);
    }
}
