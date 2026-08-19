// The result types a generated parse method returns. Generated crates
// re-export them, so they are public API for anyone using a generated parser.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::input::{Input, Span};

/// A parse failure, as a span and a message naming what was expected.
/// `Display` prints the message with the span offsets, so `{e}` works
/// without the source. `render` takes the input and produces the located,
/// caret-annotated form.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ParseError {
    /// The input range the failure covers. A failure at the end of the input
    /// has an empty span there, so the range always stays within the input.
    pub span: Span,
    pub message: String,
}

impl ParseError {
    /// Renders the error against `input`: the message with its line and
    /// column, then the offending line with a caret marking the span. An
    /// empty span still draws one caret.
    pub fn render(&self, input: &Input) -> String {
        let (line, column) = input.line_column(self.span.left_extent);
        let width = (self.span.right_extent - self.span.left_extent).max(1);
        format!(
            "Parse error at line {}, column {}: {}\n{}",
            line + 1,
            column + 1,
            self.message,
            input.line_and_caret(self.span.left_extent, width),
        )
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at {}..{}: {}",
            self.span.left_extent, self.span.right_extent, self.message
        )
    }
}

impl Error for ParseError {}

/// A successful parse: the tree, the phase durations, and an ambiguity hint.
pub struct ParseSuccess<T> {
    pub tree: T,
    pub parse_duration: Duration,
    pub tree_construction_duration: Duration,
    /// True if an ambiguity node was added during parsing. The node may sit
    /// in a dead branch that is not reachable from the root, so a true value
    /// is a hint rather than a verdict: the caller confirms it by walking the
    /// parse tree (`contains_ambiguity`). A false value means the parse is
    /// unambiguous.
    pub ambiguity_node_added: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(span: Span, message: &str) -> ParseError {
        ParseError {
            span,
            message: message.to_string(),
        }
    }

    #[test]
    fn test_display_prints_the_message_with_the_span_offsets() {
        let e = error(Span::new(12, 15), "Expected Identifier");
        assert_eq!(e.to_string(), "Parse error at 12..15: Expected Identifier");
    }

    #[test]
    fn test_render_locates_a_span_inside_a_line() {
        let input = Input::from("let x = 42;");
        let e = error(Span::new(4, 5), "Expected Identifier");
        let expected = "\
Parse error at line 1, column 5: Expected Identifier
let x = 42;
    ^";
        assert_eq!(e.render(&input), expected);
    }

    #[test]
    fn test_render_marks_a_multi_character_span() {
        let input = Input::from("foo bar");
        let e = error(Span::new(4, 7), "Expected Number");
        let expected = "\
Parse error at line 1, column 5: Expected Number
foo bar
    ^^^";
        assert_eq!(e.render(&input), expected);
    }

    #[test]
    fn test_render_puts_a_caret_after_the_last_character_at_end_of_input() {
        let input = Input::from("grammar");
        let e = error(Span::new(7, 7), "Expected Identifier");
        let expected = "\
Parse error at line 1, column 8: Expected Identifier
grammar
       ^";
        assert_eq!(e.render(&input), expected);
    }

    #[test]
    fn test_render_handles_end_of_input_after_a_trailing_newline() {
        let input = Input::from("a\n");
        let e = error(Span::new(2, 2), "Expected Rule");
        let expected = "\
Parse error at line 1, column 3: Expected Rule
a
  ^";
        assert_eq!(e.render(&input), expected);
    }

    #[test]
    fn test_render_handles_empty_input() {
        let input = Input::from("");
        let e = error(Span::new(0, 0), "Expected Grammar");
        assert_eq!(
            e.render(&input),
            "Parse error at line 1, column 1: Expected Grammar\n"
        );
    }
}
