use iguana_compiler::grammar::def::GrammarDef;
use iguana_runtime::input::{Input, Span};
use lsp_types::{Location, Position, Range, Uri};

use crate::name_resolution::NameResolutionIndex;
use crate::spans::GrammarSpans;
use crate::symbols::find_definition_at_offset;

/// Find the definition (rule head) of the symbol at `offset`.
pub fn definition(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
    uri: &Uri,
    offset: u32,
) -> Option<Location> {
    let names = NameResolutionIndex::new(grammar_def, spans);
    let def_id = find_definition_at_offset(spans, &names, offset)?;
    let &head_span = names.definitions.get(&def_id)?;
    Some(location(uri, head_span, input))
}

/// Find all references to the symbol at `offset` in the grammar source.
/// If `include_declaration` is true, the defining rule head is included.
pub fn references(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    input: &Input,
    uri: &Uri,
    offset: u32,
    include_declaration: bool,
) -> Vec<Location> {
    let names = NameResolutionIndex::new(grammar_def, spans);
    let Some(def_id) = find_definition_at_offset(spans, &names, offset) else {
        return vec![];
    };

    let mut locs = Vec::new();

    if include_declaration {
        if let Some(&head_span) = names.definitions.get(&def_id) {
            locs.push(location(uri, head_span, input));
        }
    }

    if let Some(ref_spans) = names.references.get(&def_id) {
        for &span in ref_spans {
            locs.push(location(uri, span, input));
        }
    }

    locs
}

fn location(uri: &Uri, span: Span, input: &Input) -> Location {
    let (sl, sc) = input.line_column(span.left_extent);
    let (el, ec) = input.line_column(span.right_extent);
    Location {
        uri: uri.clone(),
        range: Range {
            start: Position::new(sl, sc),
            end: Position::new(el, ec),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_refs(source: &str, line: u32, column: u32, include_declaration: bool) -> Vec<Location> {
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
        let uri: Uri = "file:///test.iggy".parse().unwrap();
        let offset = input.offset(line, column);
        references(
            &grammar_def,
            &spans,
            &input,
            &uri,
            offset,
            include_declaration,
        )
    }

    #[test]
    fn reference_from_rule_body() {
        let refs = find_refs(
            r#"
grammar T

A
  = B

B
  = "x"
"#,
            3,
            4, // cursor on B in `= B`
            false,
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].range.start, Position::new(3, 4));
    }

    #[test]
    fn reference_from_rule_head() {
        let refs = find_refs(
            r#"
grammar T

A
  = B

B
  = "x"
"#,
            5,
            0, // cursor on B rule head
            false,
        );
        // Should find the reference in A's body
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].range.start, Position::new(3, 4));
    }

    #[test]
    fn include_declaration() {
        let refs = find_refs(
            r#"
grammar T

A
  = B

B
  = "x"
"#,
            3,
            4, // cursor on B in body
            true,
        );
        // 1 declaration (B head) + 1 reference (B in A's body)
        assert_eq!(refs.len(), 2);
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
            4, // cursor on first B
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
            5, // cursor on B inside (B | "y")*
            false,
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].range.start.line, 3);
    }

    #[test]
    fn cursor_on_literal_returns_empty() {
        let refs = find_refs(
            r#"
grammar T

A
  = "x"
"#,
            3,
            4, // cursor on "x"
            false,
        );
        assert!(refs.is_empty());
    }

    fn find_def(source: &str, line: u32, column: u32) -> Option<Location> {
        use iguana_runtime::input::Input;
        let source = source.strip_prefix('\n').unwrap_or(source);
        let input = Input::from(source);
        let tree_arena = iguana_runtime::arena::Arena::new();
        let crate::BuildResult::Success { tree, .. } = crate::build(&input, &tree_arena) else {
            return None;
        };
        let grammar_def = crate::build_grammar_def(tree, &input)?;
        let spans = crate::build_spans(&grammar_def, tree, &input);
        let uri: Uri = "file:///test.iggy".parse().unwrap();
        let offset = input.offset(line, column);
        definition(&grammar_def, &spans, &input, &uri, offset)
    }

    #[test]
    fn definition_from_body_reference() {
        let loc = find_def(
            r#"
grammar T

A
  = B

B
  = "x"
"#,
            3,
            4, // cursor on B in `= B`
        );
        let loc = loc.unwrap();
        // Should jump to B's rule head at line 5, col 0
        assert_eq!(loc.range.start, Position::new(5, 0));
        assert_eq!(loc.range.end, Position::new(5, 1));
    }

    #[test]
    fn definition_from_rule_head() {
        let loc = find_def(
            r#"
grammar T

A
  = B

B
  = "x"
"#,
            5,
            0, // cursor on B rule head
        );
        let loc = loc.unwrap();
        // Points to itself
        assert_eq!(loc.range.start, Position::new(5, 0));
    }

    #[test]
    fn definition_of_terminal() {
        let loc = find_def(
            r#"
grammar T

A
  = Number

@Regex
Number = [0-9]+
"#,
            3,
            4, // cursor on Number in A's body
        );
        let loc = loc.unwrap();
        // Should jump to Number's rule head (line after @Regex)
        assert_eq!(loc.range.start.line, 6);
        assert_eq!(loc.range.start.character, 0);
    }

    #[test]
    fn definition_on_literal_returns_none() {
        let loc = find_def(
            r#"
grammar T

A
  = "x"
"#,
            3,
            4,
        );
        assert!(loc.is_none());
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
            4, // cursor on Number in A's body
            true,
        );
        // 1 declaration (Number head) + 1 reference (Number in A's body)
        assert_eq!(refs.len(), 2);
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
            0, // cursor on the Digit rule head
            false,
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].range.start, Position::new(9, 9));
    }

    #[test]
    fn reference_from_restriction_operand() {
        let refs = find_refs(
            r#"
grammar T

S
  = Id !>> Keyword

@Regex
Id = [a-z]+

@Regex
Keyword = "if"
"#,
            9,
            0, // cursor on the Keyword rule head
            false,
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].range.start, Position::new(3, 11));
    }
}
