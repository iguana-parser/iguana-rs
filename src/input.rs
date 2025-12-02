use std::{fs, io, path::Path};

pub struct Input {
    pub source: Vec<char>,
    start_offsets: Vec<usize>,
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        let source: Vec<char> = value.chars().collect();
        let line_columns = Self::calc_line_start_offsets(&source);
        Input {
            source,
            start_offsets: line_columns,
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
    pub fn len(&self) -> usize {
        self.source.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    // TODO rename this method
    pub fn is_in_range(&self, index: u32) -> bool {
        (index as usize) < self.source.len()
    }
    pub fn char_at(&self, index: u32) -> Option<char> {
        if !self.is_in_range(index) {
            None
        } else {
            Some(self.source[index as usize])
        }
    }
    pub fn substring(&self, start: u32, end: u32) -> String {
        self.source[start as usize..end as usize].iter().collect()
    }
    /// Returns the offset (index from the beginning of input) for the given
    /// line/column.
    pub fn offset(&self, line: usize, column: usize) -> usize {
        self.start_offsets[line] + column
    }
    // Returns the line and coumn corresponding to the current offset.
    pub fn line_column(&self, offset: usize) -> (usize, usize) {
        let mut line = 0;
        while line < self.start_offsets.len() - 1 {
            if self.start_offsets[line + 1] > offset {
                break;
            }
            line += 1;
        }
        (line, offset - self.start_offsets[line])
    }
    fn calc_line_start_offsets(chars: &[char]) -> Vec<usize> {
        let mut lines = vec![];
        let mut start_index = 0;
        let mut i = 0;
        for c in chars {
            i += 1;
            if *c == '\n' {
                lines.push(start_index);
                start_index = i;
            }
        }
        // Push the last line, as it may not end with a newline
        lines.push(start_index);
        lines
    }
}
