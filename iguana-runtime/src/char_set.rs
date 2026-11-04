use std::cmp::Ordering;

/// An inclusive range of characters, `start..=end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharRange {
    pub start: char,
    pub end: char,
}

/// A set of characters. An ASCII character is tested against a bitmask, and
/// any other character by a binary search over sorted ranges.
#[derive(Debug, Clone, Copy)]
pub struct CharSet<'a> {
    /// The ASCII characters of the set, at the bit of their code point.
    ascii: u128,
    /// The characters of the set outside ASCII, as sorted, disjoint ranges.
    non_ascii: &'a [CharRange],
}

impl<'a> CharSet<'a> {
    pub const fn new(ascii: u128, non_ascii: &'a [CharRange]) -> Self {
        Self { ascii, non_ascii }
    }

    #[inline(always)]
    pub fn contains(&self, c: char) -> bool {
        let code = c as u32;
        if code < 128 {
            return self.ascii & (1 << code) != 0;
        }
        self.non_ascii
            .binary_search_by(|range| {
                if range.end < c {
                    Ordering::Less
                } else if range.start > c {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
            .is_ok()
    }
}

/// A list of distinct ASCII characters, in which the index of a character is
/// its position.
#[derive(Debug, Clone, Copy)]
pub struct Alphabet {
    /// The index of each ASCII character, or `None` if the character is not
    /// in the alphabet.
    indices: [Option<u8>; 128],
    len: usize,
}

impl Alphabet {
    /// Returns the alphabet of `chars`.
    ///
    /// # Panics
    /// Panics if a character of `chars` is not ASCII.
    pub const fn from_chars(chars: &[u8]) -> Self {
        let mut indices = [None; 128];
        let mut i = 0;
        while i < chars.len() {
            indices[chars[i] as usize] = Some(i as u8);
            i += 1;
        }
        Self {
            indices,
            len: chars.len(),
        }
    }

    /// Returns the index of `c`, or `None` if `c` is not in the alphabet.
    #[inline]
    pub fn index(&self, c: char) -> Option<usize> {
        if !c.is_ascii() {
            return None;
        }
        let index = self.indices[c as usize]?;
        Some(usize::from(index))
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_range(start: char, end: char) -> CharRange {
        CharRange { start, end }
    }

    #[test]
    fn an_alphabet_indexes_its_characters_by_position() {
        const ALPHABET: Alphabet = Alphabet::from_chars(b"acde");
        assert_eq!(ALPHABET.len(), 4);
        assert_eq!(ALPHABET.index('a'), Some(0));
        assert_eq!(ALPHABET.index('c'), Some(1));
        assert_eq!(ALPHABET.index('b'), None);
        assert_eq!(ALPHABET.index('C'), None);
        assert_eq!(ALPHABET.index('é'), None);
    }

    #[test]
    fn contains_tests_ascii_characters_against_the_bitmask() {
        let digits_and_a = CharSet::new((0x3ff << 48) | (1 << 97), &[]);
        assert!(digits_and_a.contains('0'));
        assert!(digits_and_a.contains('9'));
        assert!(digits_and_a.contains('a'));
        assert!(!digits_and_a.contains('b'));
        assert!(!digits_and_a.contains('\0'));
        assert!(!digits_and_a.contains('\u{7f}'));
    }

    #[test]
    fn contains_finds_characters_outside_ascii_in_the_ranges() {
        let ranges = [
            char_range('\u{80}', '\u{ff}'),
            char_range('\u{3b1}', '\u{3c9}'),
        ];
        let set = CharSet::new(0, &ranges);
        assert!(set.contains('\u{80}'));
        assert!(set.contains('\u{ff}'));
        assert!(set.contains('\u{3b1}'));
        assert!(set.contains('\u{3c9}'));
        assert!(!set.contains('\u{100}'));
        assert!(!set.contains('\u{3b0}'));
        assert!(!set.contains('\u{3ca}'));
        assert!(!set.contains('a'));
    }

    #[test]
    fn the_empty_set_contains_no_character() {
        let empty = CharSet::new(0, &[]);
        assert!(!empty.contains('a'));
        assert!(!empty.contains('\u{80}'));
        assert!(!empty.contains(char::MAX));
    }
}
