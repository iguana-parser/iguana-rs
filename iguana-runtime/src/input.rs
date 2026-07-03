use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{fs, io, path::Path};

/// A half-open range `[left_extent, right_extent)` of input indexes.
/// `left_extent` is inclusive, `right_extent` exclusive, so `right_extent - left_extent`
/// is the width and a span covering the whole input is `[0, Input::len())`.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub struct Span {
    pub left_extent: u32,
    pub right_extent: u32,
}

impl Span {
    pub fn new(left_extent: u32, right_extent: u32) -> Self {
        Self {
            left_extent,
            right_extent,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.left_extent == self.right_extent
    }
}

impl Hash for Span {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let combined = (self.left_extent as u64) << 32 | (self.right_extent as u64);
        state.write_u64(combined);
    }
}

/// Represents the input text to be parsed.
/// The maximum input size is bounded by u32 (~4GB).
#[derive(Debug)]
pub struct Input {
    source: Vec<char>,
    // line_start_end_offsets[i] = (start_offset, end_offset) at line i
    line_start_end_offsets: Vec<(u32, u32)>,
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        assert!(
            value.len() <= u32::MAX as usize,
            "Input exceeds maximum size of {} bytes",
            u32::MAX
        );
        let source: Vec<char> = value.chars().collect();
        let line_columns = Self::calc_line_start_offsets(&source);
        Input {
            source,
            line_start_end_offsets: line_columns,
        }
    }
}

impl TryFrom<&Path> for Input {
    type Error = io::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let input = fs::read_to_string(path)?;
        Ok(Self::from(input))
    }
}

impl Input {
    pub fn len(&self) -> u32 {
        self.source.len() as u32
    }
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }
    pub fn char_at(&self, index: u32) -> Option<char> {
        if index < self.len() {
            Some(self.source[index as usize])
        } else {
            None
        }
    }
    /// Returns the text covered by `span`.
    ///
    /// # Panics
    /// Panics if the span extends past the end of the input.
    pub fn text(&self, span: Span) -> String {
        self.source[span.left_extent as usize..span.right_extent as usize]
            .iter()
            .collect()
    }
    /// Returns the offset (index from the beginning of input) for the given
    /// line/column.
    pub fn offset(&self, line: u32, column: u32) -> u32 {
        self.line_start_end_offsets[line as usize].0 + column
    }

    /// Returns the line and column corresponding to the given input index.
    pub fn line_column(&self, input_index: u32) -> (u32, u32) {
        // Empty input has no line offsets, return origin.
        if self.line_start_end_offsets.is_empty() {
            return (0, 0);
        }
        // EOF position: column just past the last character on the last line.
        if input_index == self.len() {
            let last_line = self.line_start_end_offsets.len() as u32 - 1;
            let (start, _) = self.line_start_end_offsets[last_line as usize];
            return (last_line, input_index - start);
        }
        assert!(
            input_index < self.len(),
            "input_index {} out of bounds (input length {})",
            input_index,
            self.len()
        );
        let mut low: u32 = 0;
        let mut high: u32 = self.line_start_end_offsets.len() as u32 - 1;
        while low <= high {
            let mid = low + (high - low) / 2;
            let (start, end) = self.line_start_end_offsets[mid as usize];
            if input_index >= start && input_index < end {
                return (mid, input_index - start);
            } else if input_index < start {
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }
        unreachable!()
    }
    /// End offset (exclusive) of the line containing `input_index`, excluding a
    /// trailing newline. Clamps an error highlight to a single line.
    pub fn line_end(&self, input_index: u32) -> u32 {
        if self.line_start_end_offsets.is_empty() {
            return input_index;
        }
        let (line, _) = self.line_column(input_index);
        let (_, end_offset) = self.line_start_end_offsets[line as usize];
        // Lines ending with \n have end_offset pointing past the newline.
        if self.char_at(end_offset - 1) == Some('\n') {
            end_offset - 1
        } else {
            end_offset
        }
    }
    /// Returns the source line containing `input_index` followed by a caret line
    /// spanning `len` characters from the column, joined by a newline with no
    /// trailing newline, to let a caller embed it in a larger message. Empty
    /// input has no line to show and returns an empty string.
    pub fn line_and_caret(&self, input_index: u32, len: u32) -> String {
        if self.line_start_end_offsets.is_empty() {
            return String::new();
        }
        let (line, column) = self.line_column(input_index);
        let (start_offset, _) = self.line_start_end_offsets[line as usize];
        let line_str = self.text(Span::new(start_offset, self.line_end(input_index)));
        format!(
            "{}\n{}{}",
            line_str,
            " ".repeat(column as usize),
            "^".repeat(len as usize)
        )
    }

    /// Formats a parse error message with line/column info and a caret pointing to the error position.
    pub fn format_error(&self, terminal_name: &str, input_index: u32) -> String {
        let (line, column) = self.line_column(input_index);
        format!(
            "Parse error: failed to match {} at line {}, column {}\n{}\n",
            terminal_name,
            line + 1,
            column + 1,
            self.line_and_caret(input_index, 1),
        )
    }
    /// Returns a vector v, where (s, e) = v[i] represents the start offset (inclusive)
    /// and the end offset (exclusive) at line i.
    fn calc_line_start_offsets(chars: &[char]) -> Vec<(u32, u32)> {
        let mut start_end_offsets = vec![];
        let mut start_index: u32 = 0;
        for (i, c) in chars.iter().enumerate() {
            if *c == '\n' {
                start_end_offsets.push((start_index, i as u32 + 1));
                start_index = i as u32 + 1;
            }
        }
        // Push the last line, as it may not end with a newline
        if start_index < chars.len() as u32 {
            start_end_offsets.push((start_index, chars.len() as u32));
        }
        start_end_offsets
    }
}

#[cfg(test)]
mod tests {
    use crate::input::Input;

    #[test]
    fn test_line_column() {
        let input = Input::from("abc\nde");
        assert_eq!(input.line_column(0), (0, 0)); // 'a'
        assert_eq!(input.line_column(1), (0, 1)); // 'b'
        assert_eq!(input.line_column(2), (0, 2)); // 'c'
        assert_eq!(input.line_column(3), (0, 3)); // '\n'
        assert_eq!(input.line_column(4), (1, 0)); // 'd'
        assert_eq!(input.line_column(5), (1, 1)); // 'e'
    }

    #[test]
    #[should_panic(expected = "input_index 7 out of bounds")]
    fn test_line_column_out_of_bounds() {
        let input = Input::from("abc\nde");
        input.line_column(7);
    }

    #[test]
    fn test_line_column_line_ending_in_newline() {
        let input = Input::from("abc\n");
        assert_eq!(input.line_column(0), (0, 0)); // 'a'
        assert_eq!(input.line_column(1), (0, 1)); // 'b'
        assert_eq!(input.line_column(2), (0, 2)); // 'c'
        assert_eq!(input.line_column(3), (0, 3)); // '\n'
    }

    #[test]
    fn test_line_column_starting_with_newline() {
        let input = Input::from("\nabc");
        assert_eq!(input.line_column(0), (0, 0)); // '\n'
        assert_eq!(input.line_column(1), (1, 0)); // 'a'
        assert_eq!(input.line_column(2), (1, 1)); // 'b'
        assert_eq!(input.line_column(3), (1, 2)); // 'c'
    }

    #[test]
    fn test_line_column_consecutive_newlines() {
        let input = Input::from("a\n\nb");
        assert_eq!(input.line_column(0), (0, 0)); // 'a'
        assert_eq!(input.line_column(1), (0, 1)); // '\n'
        assert_eq!(input.line_column(2), (1, 0)); // '\n' (empty line)
        assert_eq!(input.line_column(3), (2, 0)); // 'b'
    }

    #[test]
    fn test_line_column_no_newlines() {
        let input = Input::from("abc");
        assert_eq!(input.line_column(0), (0, 0));
        assert_eq!(input.line_column(1), (0, 1));
        assert_eq!(input.line_column(2), (0, 2));
    }

    #[test]
    fn test_line_column_single_newline() {
        let input = Input::from("\n");
        assert_eq!(input.line_column(0), (0, 0));
    }

    #[test]
    fn test_format_error_middle_of_line() {
        let input = Input::from("let x = 42;");
        let result = input.format_error("Identifier", 4);
        let expected = "\
Parse error: failed to match Identifier at line 1, column 5
let x = 42;
    ^
";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_error_start_of_line() {
        let input = Input::from("foo bar");
        let result = input.format_error("Identifier", 0);
        let expected = "\
Parse error: failed to match Identifier at line 1, column 1
foo bar
^
";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_error_second_line() {
        let input = Input::from("first\nsecond");
        let result = input.format_error("Identifier", 8);
        let expected = "\
Parse error: failed to match Identifier at line 2, column 3
second
  ^
";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_error_end_of_line() {
        let input = Input::from("abcd\nefgh");
        let result = input.format_error("Identifier", 3);
        let expected = "\
Parse error: failed to match Identifier at line 1, column 4
abcd
   ^
";
        assert_eq!(result, expected);
    }
}
