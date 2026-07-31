// Diagnostics for iggy grammars.
//
// Walks resolved GrammarDef symbols and reports identifiers with
// definition: None as unresolved reference errors.

use by_address::ByAddress;
use iguana_compiler::grammar::def::{GrammarDef, SyntaxRule};
use iguana_compiler::grammar::symbols::{DefinitionId, Symbol};
use iguana_runtime::input::{Input, Span};
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use rustc_hash::FxHashMap;

use crate::spans::GrammarSpans;

pub fn diagnostics(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    check_duplicate_definitions(grammar_def, spans, input, &mut out);

    grammar_def.for_each_identifier(&mut |id| {
        if id.definition.is_none() {
            if let Some(span) = spans.identifier_span(id) {
                out.push(make_diagnostic(
                    "Unresolved reference",
                    &id.name,
                    span,
                    input,
                ));
            }
        }
    });

    check_exclude_labels(grammar_def, spans, input, &mut out);

    out
}

fn check_duplicate_definitions(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
    out: &mut Vec<Diagnostic>,
) {
    let lex_count = grammar_def.lexical_rules.len();
    let mut seen = FxHashMap::default();

    for (i, rule) in grammar_def.lexical_rules.iter().enumerate() {
        let def_id = DefinitionId(i as u16);
        if let Some(&head_span) = spans.definition_spans.get(&def_id) {
            if seen.contains_key(rule.head.name.as_str()) {
                out.push(make_diagnostic(
                    "Duplicate definition",
                    &rule.head.name,
                    head_span,
                    input,
                ));
            } else {
                seen.insert(rule.head.name.as_str(), head_span);
            }
        }
    }
    for (i, rule) in grammar_def.syntax_rules.iter().enumerate() {
        let def_id = DefinitionId((lex_count + i) as u16);
        if let Some(&head_span) = spans.definition_spans.get(&def_id) {
            if seen.contains_key(rule.head.name.as_str()) {
                out.push(make_diagnostic(
                    "Duplicate definition",
                    &rule.head.name,
                    head_span,
                    input,
                ));
            } else {
                seen.insert(rule.head.name.as_str(), head_span);
            }
        }
    }
}

/// Reports labels in `Exclude` symbols (`A!label`) that don't match any
/// alternative label on the referenced nonterminal.
fn check_exclude_labels(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
    out: &mut Vec<Diagnostic>,
) {
    let rules_by_name: FxHashMap<&str, &SyntaxRule> = grammar_def
        .syntax_rules
        .iter()
        .map(|r| (r.head.name.as_str(), r))
        .collect();

    grammar_def.for_each_symbol(&mut |symbol| {
        if let Symbol::Exclude {
            symbol: inner,
            labels,
        } = symbol
        {
            if let Some(id) = inner.as_identifier() {
                if let Some(rule) = rules_by_name.get(id.name.as_str()) {
                    let label_spans = spans
                        .label_spans
                        .get(&ByAddress(symbol))
                        .map(|v| v.as_slice())
                        .unwrap_or_default();

                    let valid_labels: Vec<&str> = rule
                        .priority_levels
                        .iter()
                        .flat_map(|pl| &pl.alternatives)
                        .filter_map(|alt| alt.label.as_deref())
                        .collect();

                    for (label, span) in labels.iter().zip(label_spans) {
                        if !valid_labels.contains(&label.as_str()) {
                            out.push(make_diagnostic("Unresolved label", label, *span, input));
                        }
                    }
                }
            }
        }
    });
}

fn make_diagnostic(kind: &str, name: &str, span: Span, input: &Input) -> Diagnostic {
    let (sl, sc) = input.line_column(span.left_extent);
    let (el, ec) = input.line_column(span.right_extent);
    Diagnostic {
        range: Range {
            start: Position::new(sl, sc),
            end: Position::new(el, ec),
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: format!("{kind} '{name}'"),
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
        let Some(grammar_def) = crate::build_grammar_def(tree, &input) else {
            return vec![];
        };
        let spans = crate::build_spans(&grammar_def, tree, &input);
        diagnostics(&grammar_def, &spans, &input)
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
}
