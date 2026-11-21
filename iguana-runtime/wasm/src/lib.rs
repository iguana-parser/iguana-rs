//! wasm-bindgen wrapper over the runtime, for the editors in the browser.
//!
//! The runtime's offsets and columns are in characters, and a JavaScript
//! editor's columns are in UTF-16 code units. `TextPositions` gives an editor
//! the line and the column in UTF-16 of an offset, and the offset at a line
//! and a column in UTF-16, so the JavaScript does not convert anything
//! itself.

use iguana_runtime::input::Input;
use iguana_runtime::utf16::Utf16Columns;
use wasm_bindgen::prelude::*;

/// The positions of a text in an editor: a line and a column in UTF-16, both
/// from 0.
#[wasm_bindgen]
pub struct TextPositions {
    input: Input,
    columns: Utf16Columns,
}

#[wasm_bindgen]
impl TextPositions {
    #[wasm_bindgen(constructor)]
    pub fn new(text: &str) -> TextPositions {
        let input = Input::from(text);
        let columns = Utf16Columns::new(&input);
        TextPositions { input, columns }
    }

    /// The line and the column in UTF-16 of the offset `offset`. An offset past
    /// the end of the text is the end of the text. After a final newline, the
    /// end of the text is at the start of the line after it, where an editor
    /// shows it.
    pub fn position(&self, offset: u32) -> Vec<u32> {
        let input = &self.input;
        let offset = offset.min(input.len());
        if offset > 0 && offset == input.len() && input.char_at(offset - 1) == Some('\n') {
            let (line, _) = input.line_column(offset - 1);
            return vec![line + 1, 0];
        }
        let (line, column) = input.line_column(offset);
        vec![line, self.columns.utf16_column(line, column)]
    }

    /// The line and the UTF-16 column of each offset in `offsets`
    /// (`position`), flattened to `[line, column, line, column, …]`. One call
    /// converts all the offsets, instead of one call across the WebAssembly
    /// boundary per offset.
    pub fn positions(&self, offsets: &[u32]) -> Vec<u32> {
        offsets
            .iter()
            .flat_map(|&offset| self.position(offset))
            .collect()
    }

    /// The offset at `line` and the column in UTF-16 `column`. A column past the
    /// end of its line is the end of the line, and a line past the last line
    /// is the end of the text.
    pub fn offset(&self, line: u32, column: u32) -> u32 {
        let input = &self.input;
        let Some(start) = input.offset(line, 0) else {
            return input.len();
        };
        let line_length = input.line_end(start) - start;
        start + self.columns.char_column(line, column).min(line_length)
    }
}

#[cfg(test)]
mod tests {
    use super::TextPositions;

    #[test]
    fn a_position_has_a_column_in_utf16() {
        let text = TextPositions::new("a😀x\nb");
        assert_eq!(text.position(2), [0, 3]);
        assert_eq!(text.position(4), [1, 0]);
        assert_eq!(text.offset(0, 3), 2);
        // A column in the middle of a surrogate pair is the column of its
        // character.
        assert_eq!(text.offset(0, 2), 1);
    }

    #[test]
    fn positions_are_the_position_of_each_offset() {
        let text = TextPositions::new("a😀x\nb\n");
        assert_eq!(text.positions(&[2, 4, 6]), [0, 3, 1, 0, 2, 0]);
    }

    #[test]
    fn the_end_after_a_final_newline_is_on_the_next_line() {
        let text = TextPositions::new("a\n");
        assert_eq!(text.position(2), [1, 0]);
        assert_eq!(text.position(9), [1, 0]);
        assert_eq!(TextPositions::new("").position(0), [0, 0]);
    }

    #[test]
    fn a_position_past_the_end_of_its_line_is_the_end_of_the_line() {
        let text = TextPositions::new("ab\ncd");
        assert_eq!(text.offset(0, 9), 2);
        assert_eq!(text.offset(1, 9), 5);
        assert_eq!(text.offset(1, u32::MAX), 5);
        assert_eq!(text.offset(5, 0), 5);
    }
}
