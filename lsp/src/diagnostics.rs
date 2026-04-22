// Diagnostics for iggy grammars.
//
// Walks resolved GrammarDef symbols and reports identifiers with
// definition: None as unresolved reference errors.

use iguana::grammar::def::GrammarDef;
use iguana_runtime::{input::Input, sppf::Span};
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::spans::GrammarSpans;

pub fn diagnostics(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    grammar_def.for_each_identifier(&mut |id| {
        if id.definition.is_none() {
            if let Some(span) = spans.identifier_span(id) {
                out.push(unresolved_diagnostic(&id.name, span, input));
            }
        }
    });

    out
}

fn unresolved_diagnostic(name: &str, span: Span, input: &Input) -> Diagnostic {
    let (sl, sc) = input.line_column(span.left_extent);
    let (el, ec) = input.line_column(span.right_extent);
    Diagnostic {
        range: Range {
            start: Position::new(sl, sc),
            end: Position::new(el, ec),
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: format!("Unresolved reference '{}'", name),
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
        let ctx = iguana_runtime::parse_tree::ParseContext::new();
        let crate::BuildResult::Success { ref tree, .. } = crate::build(&input, &ctx) else {
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
        let d = diags(r#"
grammar T

Expr
  = "x"
"#);
        assert!(d.is_empty());
    }

    #[test]
    fn unresolved_identifier_reported() {
        let d = diags(r#"
grammar T

Expr
  = Foo
"#);
        assert_eq!(d.len(), 1);
        assert!(d[0].message.contains("Foo"));
        assert_eq!(d[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn multiple_unresolved() {
        let d = diags(r#"
grammar T

Expr
  = Foo Bar
"#);
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn resolved_reference_no_error() {
        let d = diags(r#"
grammar T

Expr
  = Term

Term
  = "x"
"#);
        assert!(d.is_empty());
    }
}
