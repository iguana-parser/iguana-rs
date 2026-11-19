use crate::input::Input;

/// `Utf16Columns` converts a column from UTF-16 code units to Unicode code
/// points (UTF-32) and back.
///
/// In `Input`, a column is the number of characters from the start of a
/// line, where a character is a Unicode code point, a Rust `char`. In an
/// editor and the Language Server Protocol, a column is the number of UTF-16
/// code units from the start of the line, the 16-bit units a JavaScript
/// string is made of. A code point up to U+FFFF is one code unit, and a code
/// point above U+FFFF, such as most emojis, is two, which is called a
/// surrogate pair. So a column in UTF-16 is the column in characters plus the
/// number of surrogate pairs before it on the line.
pub struct Utf16Columns {
    /// The line and the column in characters of each surrogate pair, in input
    /// order.
    surrogate_pairs: Vec<(u32, u32)>,
}

impl Utf16Columns {
    pub fn new(input: &Input) -> Self {
        let surrogate_pairs = (0..input.len())
            .filter(|&offset| input.char_at(offset).is_some_and(|c| c.len_utf16() == 2))
            .map(|offset| input.line_column(offset))
            .collect();
        Utf16Columns { surrogate_pairs }
    }

    /// The column in UTF-16 of the column in characters `column` on line
    /// `line`.
    pub fn utf16_column(&self, line: u32, column: u32) -> u32 {
        let pairs = self.line_pairs(line);
        let pairs_before = pairs.partition_point(|&(_, pair_column)| pair_column < column);
        column.saturating_add(pairs_before as u32)
    }

    /// The column in characters of the column in UTF-16 `column` on line
    /// `line`. A column in the middle of a surrogate pair is the column of its
    /// character.
    pub fn char_column(&self, line: u32, column: u32) -> u32 {
        let pairs = self.line_pairs(line);
        // The surrogate pair at index i, at the column c in characters, is at
        // the column c + i in UTF-16, which grows with i. Each pair before
        // `column` is two code units and one character.
        let (mut low, mut high) = (0, pairs.len());
        while low < high {
            let middle = (low + high) / 2;
            if pairs[middle].1 + (middle as u32) < column {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        column - low as u32
    }

    fn line_pairs(&self, line: u32) -> &[(u32, u32)] {
        let start = self
            .surrogate_pairs
            .partition_point(|&(pair_line, _)| pair_line < line);
        let end = self
            .surrogate_pairs
            .partition_point(|&(pair_line, _)| pair_line <= line);
        &self.surrogate_pairs[start..end]
    }
}

#[cfg(test)]
mod tests {
    use crate::input::Input;
    use crate::utf16::Utf16Columns;

    #[test]
    fn a_utf16_column_counts_a_surrogate_pair_as_two() {
        let columns = Utf16Columns::new(&Input::from("😀x\na😀b🎉c"));
        assert_eq!(columns.utf16_column(0, 1), 2);
        // The pair on the first line does not shift the columns of the second.
        let line: Vec<u32> = (0..=5)
            .map(|column| columns.utf16_column(1, column))
            .collect();
        assert_eq!(line, [0, 1, 3, 4, 6, 7]);
        // Without surrogate pairs, the two columns are the same.
        let columns_without_pairs = Utf16Columns::new(&Input::from("aé→b"));
        assert_eq!(columns_without_pairs.utf16_column(0, 3), 3);
        assert_eq!(columns.utf16_column(0, u32::MAX), u32::MAX);
        assert_eq!(Utf16Columns::new(&Input::from("")).utf16_column(0, 0), 0);
    }

    #[test]
    fn a_char_column_is_the_character_at_a_utf16_column() {
        let input = Input::from("😀x\na😀b🎉c\n");
        let columns = Utf16Columns::new(&input);
        let line: Vec<u32> = (0..=7)
            .map(|column| columns.char_column(1, column))
            .collect();
        // The columns 2 and 5 in UTF-16 are in the middle of the surrogate
        // pairs.
        assert_eq!(line, [0, 1, 1, 2, 3, 3, 4, 5]);
        // Every character's column maps back to itself.
        for offset in 0..input.len() {
            let (line, column) = input.line_column(offset);
            assert_eq!(
                columns.char_column(line, columns.utf16_column(line, column)),
                column
            );
        }
    }

    #[test]
    fn a_column_in_utf16_after_the_last_character_of_its_line_has_no_offset() {
        // The first line has the newline at column 3, and the second line's
        // surrogate pair does not shift the first line's end. The end of the
        // text is not a character, so it has no offset.
        let input = Input::from("abc\n😀");
        let columns = Utf16Columns::new(&input);
        let offset = |line, column| input.offset(line, columns.char_column(line, column));
        assert_eq!(offset(0, 3), Some(3));
        assert_eq!(offset(0, 4), None);
        assert_eq!(offset(1, 1), Some(4));
        assert_eq!(offset(1, 2), None);
        assert_eq!(offset(0, u32::MAX), None);
    }
}
