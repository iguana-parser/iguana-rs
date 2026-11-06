use std::time::Duration;

use iguana_compiler::grammar::symbols::DefinitionId;
use iguana_compiler::spans::build_spans;
use iguana_runtime::{arena::Arena, input::Input};
use lsp_types::{Diagnostic, DocumentSymbol, FoldingRange, Position, Range, SemanticToken};

use crate::diagnostics::{diagnostics, to_diagnostic};
use crate::document_symbols::document_symbols;
use crate::folding::folding_ranges;
use crate::name_resolution::NameResolutionIndex;
use crate::semantic_tokens::semantic_tokens;
use crate::{BuildResult, build, build_grammar_def, to_range};

#[derive(Clone, Copy)]
pub struct ParseDurations {
    pub parse: Duration,
    pub tree_construction: Duration,
}

/// A grammar source and the answers to language-feature requests about it.
///
/// The answers are empty when the source does not parse or is ambiguous. A
/// source that does not parse also has one diagnostic at the parse error.
pub struct Analysis {
    source: String,
    parse_durations: Option<ParseDurations>,
    input: Input,
    names: NameResolutionIndex,
    semantic_tokens: Vec<SemanticToken>,
    document_symbols: Vec<DocumentSymbol>,
    folding_ranges: Vec<FoldingRange>,
    diagnostics: Vec<Diagnostic>,
}

impl Analysis {
    pub fn new(source: &str) -> Self {
        let input = Input::from(source);
        let tree_arena = Arena::new();
        match build(&input, &tree_arena) {
            BuildResult::Success {
                tree,
                parse_duration,
                tree_construction_duration,
            } => {
                let grammar_def = build_grammar_def(tree, &input);
                let spans = build_spans(&grammar_def, tree, &input);
                Self {
                    source: source.to_string(),
                    parse_durations: Some(ParseDurations {
                        parse: parse_duration,
                        tree_construction: tree_construction_duration,
                    }),
                    names: NameResolutionIndex::new(&grammar_def, &spans),
                    semantic_tokens: semantic_tokens(tree, &input),
                    document_symbols: document_symbols(&grammar_def, &spans, &input),
                    folding_ranges: folding_ranges(&grammar_def, &spans, &input),
                    diagnostics: diagnostics(&grammar_def, &spans, &input),
                    input,
                }
            }
            BuildResult::Ambiguous => Self::without_grammar(source, input, vec![]),
            BuildResult::Error(error) => {
                let diagnostic = to_diagnostic(error, &input);
                Self::without_grammar(source, input, vec![diagnostic])
            }
        }
    }

    fn without_grammar(source: &str, input: Input, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            source: source.to_string(),
            parse_durations: None,
            input,
            names: NameResolutionIndex::default(),
            semantic_tokens: vec![],
            document_symbols: vec![],
            folding_ranges: vec![],
            diagnostics,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    /// The time spent on parsing, or `None` when the source does not parse or
    /// is ambiguous.
    pub fn parse_durations(&self) -> Option<ParseDurations> {
        self.parse_durations
    }

    pub fn semantic_tokens(&self) -> &[SemanticToken] {
        &self.semantic_tokens
    }

    pub fn document_symbols(&self) -> &[DocumentSymbol] {
        &self.document_symbols
    }

    pub fn folding_ranges(&self) -> &[FoldingRange] {
        &self.folding_ranges
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// The rule head of the name at `position`, or `None` when `position` is
    /// not on a resolved name.
    pub fn definition(&self, position: Position) -> Option<Range> {
        let definition = self.definition_id_at(position)?;
        let &head = self.names.definitions.get(&definition)?;
        Some(to_range(head, &self.input))
    }

    /// The references to the name at `position`, in source order. The list
    /// starts with the rule head when `include_declaration` is true, and it is
    /// empty when `position` is not on a resolved name.
    pub fn references(&self, position: Position, include_declaration: bool) -> Vec<Range> {
        let Some(definition) = self.definition_id_at(position) else {
            return vec![];
        };

        let mut ranges = vec![];
        if include_declaration {
            if let Some(&head) = self.names.definitions.get(&definition) {
                ranges.push(to_range(head, &self.input));
            }
        }
        if let Some(references) = self.names.references.get(&definition) {
            for &span in references {
                ranges.push(to_range(span, &self.input));
            }
        }
        ranges
    }

    fn definition_id_at(&self, position: Position) -> Option<DefinitionId> {
        let offset = self.input.offset(position.line, position.character)?;
        self.names.definition_id_at(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iguana_compiler::spans::GrammarSpans;
    use lsp_types::DiagnosticSeverity;
    use std::{fs, path::Path};

    fn analyze(source: &str) -> Analysis {
        Analysis::new(source.strip_prefix('\n').unwrap_or(source))
    }

    fn find_refs(source: &str, line: u32, column: u32, include_declaration: bool) -> Vec<Range> {
        analyze(source).references(Position::new(line, column), include_declaration)
    }

    fn find_def(source: &str, line: u32, column: u32) -> Option<Range> {
        analyze(source).definition(Position::new(line, column))
    }

    const TWO_RULES: &str = r#"
grammar T

A
  = B

B
  = "x"
"#;

    #[test]
    fn reference_from_rule_body() {
        let refs = find_refs(TWO_RULES, 3, 4, false);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].start, Position::new(3, 4));
    }

    #[test]
    fn reference_from_rule_head() {
        let refs = find_refs(TWO_RULES, 5, 0, false);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].start, Position::new(3, 4));
    }

    #[test]
    fn include_declaration() {
        let refs = find_refs(TWO_RULES, 3, 4, true);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].start, Position::new(5, 0));
        assert_eq!(refs[1].start, Position::new(3, 4));
    }

    #[test]
    fn multiple_references() {
        let refs = find_refs(
            r#"
grammar T

A
  = B B

B
  = "x"
"#,
            3,
            4,
            false,
        );
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn nested_symbol_reference() {
        let refs = find_refs(
            r#"
grammar T

A
  = (B | "y")*

B
  = "x"
"#,
            3,
            5, // on B inside (B | "y")*
            false,
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].start.line, 3);
    }

    #[test]
    fn reference_from_regex_body() {
        let refs = find_refs(
            r#"
grammar T

A
  = Number

@Regex
Digit = [0-9]

@Regex
Number = Digit+
"#,
            6,
            0, // on the Digit rule head
            false,
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].start, Position::new(9, 9));
    }

    const RESTRICTION: &str = r#"
grammar T

S
  = Id !>> Keyword

@Regex
Id = [a-z]+

@Regex
Keyword = "if"
"#;

    #[test]
    fn reference_from_restriction_operand() {
        let refs = find_refs(RESTRICTION, 9, 0, false);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].start, Position::new(3, 11));
    }

    #[test]
    fn definition_from_restriction_operand() {
        let range = find_def(RESTRICTION, 3, 11).unwrap();
        assert_eq!(range.start, Position::new(9, 0));
    }

    #[test]
    fn definition_from_body_reference() {
        let range = find_def(TWO_RULES, 3, 4).unwrap();
        assert_eq!(range.start, Position::new(5, 0));
        assert_eq!(range.end, Position::new(5, 1));
    }

    #[test]
    fn definition_from_rule_head() {
        let range = find_def(TWO_RULES, 5, 0).unwrap();
        assert_eq!(range.start, Position::new(5, 0));
    }

    #[test]
    fn definition_of_terminal() {
        let range = find_def(
            r#"
grammar T

A
  = Number

@Regex
Number = [0-9]+
"#,
            3,
            4,
        )
        .unwrap();
        assert_eq!(range.start, Position::new(6, 0));
    }

    #[test]
    fn terminal_reference() {
        let refs = find_refs(
            r#"
grammar T

A
  = Number

@Regex
Number = [0-9]+
"#,
            3,
            4,
            true,
        );
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn literal_has_no_definition_or_references() {
        let source = r#"
grammar T

A
  = "x"
"#;
        assert!(find_def(source, 3, 4).is_none());
        assert!(find_refs(source, 3, 4, true).is_empty());
    }

    /// The name `Long` occupies columns 4 to 7 of line 3.
    #[test]
    fn name_boundaries() {
        let source = r#"
grammar T

A
  = Long "y"

Long
  = "x"
"#;
        assert!(find_def(source, 3, 3).is_none());
        assert!(find_def(source, 3, 4).is_some());
        assert!(find_def(source, 3, 7).is_some());
        assert!(find_def(source, 3, 8).is_none());
    }

    /// A source that ends in a newline does not have a line after that newline,
    /// although an editor can place the cursor there.
    #[test]
    fn position_after_the_last_line_is_not_on_a_name() {
        let source = "grammar T\n\nA\n  = \"x\"\n";
        assert!(find_def(source, 4, 0).is_none());
        assert!(find_refs(source, 4, 0, true).is_empty());
    }

    /// Column 11 of line 0 has the same offset as column 0 of line 2, which is
    /// on the name `A`.
    #[test]
    fn position_after_the_end_of_a_line_is_not_on_a_name() {
        let source = "grammar T\n\nA\n  = \"x\"\n";
        assert!(find_def(source, 2, 0).is_some());
        assert!(find_def(source, 0, 11).is_none());
        assert!(find_refs(source, 0, 11, true).is_empty());
    }

    #[test]
    fn parse_error_has_one_diagnostic_and_no_other_answers() {
        let analysis = Analysis::new("grammar");
        assert_eq!(analysis.diagnostics().len(), 1);
        assert_eq!(
            analysis.diagnostics()[0].severity,
            Some(DiagnosticSeverity::ERROR)
        );
        assert_eq!(analysis.diagnostics()[0].range.start, Position::new(0, 7));
        assert!(analysis.parse_durations().is_none());
        assert!(analysis.semantic_tokens().is_empty());
        assert!(analysis.document_symbols().is_empty());
        assert!(analysis.folding_ranges().is_empty());
        assert!(analysis.definition(Position::new(0, 0)).is_none());
    }

    #[test]
    fn multiline_literal_has_every_answer() {
        let analysis = Analysis::new("grammar T\nS = \"abc\nx\"\n");
        assert!(analysis.diagnostics().is_empty());
        assert!(analysis.parse_durations().is_some());
        assert!(!analysis.semantic_tokens().is_empty());
        assert_eq!(analysis.document_symbols().len(), 1);
    }

    #[test]
    fn empty_source_is_not_on_a_name() {
        let analysis = Analysis::new("");
        assert!(analysis.definition(Position::new(0, 0)).is_none());
        assert!(analysis.references(Position::new(0, 0), true).is_empty());
    }

    fn scan_for_definition(
        names: &NameResolutionIndex,
        spans: &GrammarSpans<'_>,
        offset: u32,
    ) -> Option<DefinitionId> {
        for (&definition, span) in &names.definitions {
            if offset >= span.left_extent && offset < span.right_extent {
                return Some(definition);
            }
        }
        for (identifier, region) in &spans.identifiers {
            if offset >= region.span.left_extent && offset < region.span.right_extent {
                return identifier.definition;
            }
        }
        None
    }

    fn assert_lookup_matches_scan(path: &Path) {
        let source = fs::read_to_string(path).unwrap();
        let analysis = Analysis::new(&source);

        let input = Input::from(source.as_str());
        let tree_arena = Arena::new();
        let BuildResult::Success { tree, .. } = build(&input, &tree_arena) else {
            panic!("{} does not parse", path.display());
        };
        let grammar_def = build_grammar_def(tree, &input);
        let spans = build_spans(&grammar_def, tree, &input);
        let names = NameResolutionIndex::new(&grammar_def, &spans);

        for offset in 0..input.len() {
            let (line, column) = input.line_column(offset);
            let position = Position::new(line, column);
            let definition = scan_for_definition(&names, &spans, offset);
            let head = definition.and_then(|id| names.definitions.get(&id));
            let references: Vec<_> = definition
                .and_then(|id| names.references.get(&id))
                .into_iter()
                .flatten()
                .map(|&span| to_range(span, &input))
                .collect();

            assert_eq!(
                analysis.definition(position),
                head.map(|&span| to_range(span, &input)),
                "{}:{line}:{column}",
                path.display(),
            );
            assert_eq!(
                analysis.references(position, false),
                references,
                "{}:{line}:{column}",
                path.display(),
            );
        }
    }

    #[test]
    fn lookup_matches_scan_on_every_grammar() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut grammars = vec![root.join("iggy/iggy.iggy")];
        for directory in fs::read_dir(root.join("tests")).unwrap() {
            let directory = directory.unwrap().path();
            if !directory.is_dir() {
                continue;
            }
            for file in fs::read_dir(directory).unwrap() {
                let file = file.unwrap().path();
                if file
                    .extension()
                    .is_some_and(|extension| extension == "iggy")
                {
                    grammars.push(file);
                }
            }
        }
        assert!(grammars.len() > 50, "found {} grammars", grammars.len());
        for grammar in grammars {
            assert_lookup_matches_scan(&grammar);
        }
    }
}
