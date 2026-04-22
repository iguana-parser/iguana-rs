// Helpers for working with iggy's `Layout` parse-tree nodes (whitespace +
// comments). Used by both the formatter and document-symbols.

use iggy::parse_tree::{Layout, Token};
use iguana_runtime::input::Input;

/// True if `a` and `b` (input byte indices) lie on the same line.
pub fn is_same_line(input: &Input, a: u32, b: u32) -> bool {
    let (la, _) = input.line_column(a);
    let (lb, _) = input.line_column(b);
    la == lb
}

/// Return the comment in `layout` that sits on the same line as `prev_end`,
/// if any. Used to attach a trailing `// ...` to the preceding token.
pub fn trailing_comment(layout: &Layout, input: &Input, prev_end: u32) -> Option<Token> {
    layout
        .line_comments()
        .next()
        .filter(|c| is_same_line(input, c.span().left_extent, prev_end))
}

/// Return the consecutive comments at the end of `layout` whose last line
/// sits immediately above `next_start` (no blank line in between). These are
/// the "doc comments" that belong to the next construct.
///
/// Returns the leading-block as a `Vec` (in source order). Empty if there is
/// no eligible block.
pub fn leading_comments(layout: &Layout, input: &Input, next_start: u32) -> Vec<Token> {
    let comments: Vec<Token> = layout.line_comments().collect();
    if comments.is_empty() {
        return Vec::new();
    }

    // Walk backwards from the end, accepting each comment whose line is
    // exactly one less than the next eligible line. Start with `next_start`'s
    // line; each accepted comment shifts the expected line up by one.
    let mut block: Vec<Token> = Vec::new();
    let (mut expected_line, _) = input.line_column(next_start);
    for c in comments.iter().rev() {
        let (cl, _) = input.line_column(c.span().left_extent);
        if cl + 1 == expected_line {
            block.push(*c);
            expected_line = cl;
        } else {
            break;
        }
    }
    block.reverse();
    block
}
