use std::time::Duration;

use iguana_compiler::grammar::symbols::DefinitionId;
use iguana_compiler::spans::build_spans;
use iguana_runtime::{arena::Arena, input::Input, utf16::Utf16Columns};
use lsp_types::{Diagnostic, DocumentSymbol, FoldingRange, Position, Range, SemanticToken};

use crate::diagnostics::{diagnostics, to_diagnostic};
use crate::document_symbols::document_symbols;
use crate::folding::folding_ranges;
use crate::name_resolution::NameResolutionIndex;
use crate::semantic_tokens::{semantic_tokens, semantic_tokens_from_scanner};
use crate::{BuildResult, build, build_grammar_def, to_offset, to_range};

#[derive(Clone, Copy)]
pub struct ParseDurations {
    pub parse: Duration,
    pub tree_construction: Duration,
}

/// A grammar source and the answers to language-feature requests about it.
///
/// The answers are empty when the source does not parse or is ambiguous,
/// except the semantic tokens, which then come from the scanner alone. A
/// source that does not parse also has one diagnostic at the parse error.
pub struct Analysis {
    source: String,
    parse_durations: Option<ParseDurations>,
    input: Input,
    columns: Utf16Columns,
    names: NameResolutionIndex,
    semantic_tokens: Vec<SemanticToken>,
    document_symbols: Vec<DocumentSymbol>,
    folding_ranges: Vec<FoldingRange>,
    diagnostics: Vec<Diagnostic>,
}

impl Analysis {
    pub fn new(source: &str) -> Self {
        let input = Input::from(source);
        let columns = Utf16Columns::new(&input);
        let tree_arena = Arena::new();
        match build(&input, &tree_arena) {
            BuildResult::Success {
                tree,
                parse_duration,
                tree_construction_duration,
            } => {
                let grammar_def = build_grammar_def(tree, &input);
                let spans = build_spans(&grammar_def, tree, &input);
                let names = NameResolutionIndex::new(&grammar_def, &spans);
                Self {
                    source: source.to_string(),
                    parse_durations: Some(ParseDurations {
                        parse: parse_duration,
                        tree_construction: tree_construction_duration,
                    }),
                    semantic_tokens: semantic_tokens(
                        tree,
                        &input,
                        &columns,
                        &grammar_def,
                        &spans,
                        &names,
                    ),
                    names,
                    document_symbols: document_symbols(&grammar_def, &spans, &input, &columns),
                    folding_ranges: folding_ranges(&grammar_def, &spans, &input),
                    diagnostics: diagnostics(&grammar_def, &spans, &input, &columns),
                    input,
                    columns,
                }
            }
            BuildResult::Ambiguous => Self::without_grammar(source, input, columns, vec![]),
            BuildResult::Error(error) => {
                let diagnostic = to_diagnostic(error, &input, &columns);
                Self::without_grammar(source, input, columns, vec![diagnostic])
            }
        }
    }

    fn without_grammar(
        source: &str,
        input: Input,
        columns: Utf16Columns,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            source: source.to_string(),
            parse_durations: None,
            semantic_tokens: semantic_tokens_from_scanner(&input, &columns),
            input,
            columns,
            names: NameResolutionIndex::default(),
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
        Some(to_range(head, &self.input, &self.columns))
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
                ranges.push(to_range(head, &self.input, &self.columns));
            }
        }
        if let Some(references) = self.names.references.get(&definition) {
            for &span in references {
                ranges.push(to_range(span, &self.input, &self.columns));
            }
        }
        ranges
    }

    fn definition_id_at(&self, position: Position) -> Option<DefinitionId> {
        let offset = to_offset(position, &self.input, &self.columns)?;
        self.names.definition_id_at(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_tokens::token_type;
    use iggy::highlighting::Group;
    use iguana_compiler::spans::GrammarSpans;
    use lsp_types::DiagnosticSeverity;
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn analyze(source: &str) -> Analysis {
        Analysis::new(source.trim_prefix('\n'))
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

    // A position's character is a column in UTF-16. The emoji is two code
    // units, so `B` is at character 9.
    #[test]
    fn a_position_is_in_utf16_code_units() {
        let source = "grammar G\nA = \"😀\" B\nB = \"b\"";
        let range = find_def(source, 1, 9).unwrap();
        assert_eq!(range.start, Position::new(2, 0));
        let refs = find_refs(source, 2, 0, false);
        assert_eq!(refs[0].start, Position::new(1, 9));
    }

    #[test]
    fn parse_error_has_one_diagnostic_and_only_lexical_tokens() {
        let analysis = Analysis::new("grammar");
        assert_eq!(analysis.diagnostics().len(), 1);
        assert_eq!(
            analysis.diagnostics()[0].severity,
            Some(DiagnosticSeverity::ERROR)
        );
        assert_eq!(analysis.diagnostics()[0].range.start, Position::new(0, 7));
        assert!(analysis.parse_durations().is_none());
        // The keyword `grammar`, from the scanner.
        let tokens = analysis.semantic_tokens();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, token_type(Group::Keyword));
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
        let columns = Utf16Columns::new(&input);
        let tree_arena = Arena::new();
        let BuildResult::Success { tree, .. } = build(&input, &tree_arena) else {
            panic!("{} does not parse", path.display());
        };
        let grammar_def = build_grammar_def(tree, &input);
        let spans = build_spans(&grammar_def, tree, &input);
        let names = NameResolutionIndex::new(&grammar_def, &spans);

        for offset in 0..input.len() {
            let position = crate::to_position(offset, &input, &columns);
            let definition = scan_for_definition(&names, &spans, offset);
            let head = definition.and_then(|id| names.definitions.get(&id));
            let references: Vec<_> = definition
                .and_then(|id| names.references.get(&id))
                .into_iter()
                .flatten()
                .map(|&span| to_range(span, &input, &columns))
                .collect();

            assert_eq!(
                analysis.definition(position),
                head.map(|&span| to_range(span, &input, &columns)),
                "{}:{}:{}",
                path.display(),
                position.line,
                position.character,
            );
            assert_eq!(
                analysis.references(position, false),
                references,
                "{}:{}:{}",
                path.display(),
                position.line,
                position.character,
            );
        }
    }

    #[test]
    fn lookup_matches_scan_on_every_grammar() {
        for grammar in every_grammar() {
            assert_lookup_matches_scan(&grammar);
        }
    }

    /// Each rule head and resolved reference has a semantic token of its
    /// rule's group, and no name is `unresolved`.
    #[test]
    fn every_rule_name_has_a_token_on_every_grammar() {
        for grammar in every_grammar() {
            let source = fs::read_to_string(&grammar).unwrap();
            let analysis = Analysis::new(&source);
            let tokens_of_group = |group| {
                analysis
                    .semantic_tokens()
                    .iter()
                    .filter(|token| token.token_type == token_type(group))
                    .count()
            };
            let lexical = analysis
                .names
                .rule_names()
                .iter()
                .filter(|&&(_, definition)| analysis.names.is_lexical_rule(definition))
                .count();
            let syntax = analysis.names.rule_names().len() - lexical;
            // The restrictions of lexical rules are not in the name index
            // (`name_groups`), so the lexical tokens can outnumber the
            // lexical names.
            let syntax_tokens = tokens_of_group(Group::RuleSyntax);
            let lexical_tokens = tokens_of_group(Group::RuleLexical);
            assert!(
                syntax_tokens == syntax && lexical_tokens >= lexical,
                "{}: {syntax_tokens} syntax and {lexical_tokens} lexical tokens for {syntax} and {lexical} names",
                grammar.display(),
            );
            assert_eq!(
                tokens_of_group(Group::Unresolved),
                0,
                "{}",
                grammar.display()
            );
        }
    }

    /// Iggy's own grammar and every test grammar.
    fn every_grammar() -> Vec<PathBuf> {
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
        grammars
    }
}
