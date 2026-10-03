use std::fmt::Display;

use itertools::Itertools;

use super::symbols::Identifier;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Regex {
    Char(char),
    CharRange(CharRange),
    CharClass(CharClass),
    Seq(Vec<Regex>),
    Alt(Vec<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>),
    Opt(Box<Regex>),
    Epsilon,
    /// A reference to another named `@regex` rule — inlined during grammar compilation.
    Identifier(Identifier),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CharRange {
    pub start: char,
    pub end: char,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CharClass {
    pub ranges: Vec<CharRange>,
    pub negated: bool,
}

impl Display for CharRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start.escape_debug())
        } else {
            write!(
                f,
                "{}-{}",
                self.start.escape_debug(),
                self.end.escape_debug()
            )
        }
    }
}

impl CharClass {
    pub fn char_ranges(&self) -> Vec<CharRange> {
        if self.negated {
            complement(&self.ranges)
        } else {
            self.ranges.clone()
        }
    }
}

/// Complement of `ranges` over the Unicode scalar value space
/// (`\0`..=`char::MAX`). Input may overlap; output is sorted and disjoint.
fn complement(ranges: &[CharRange]) -> Vec<CharRange> {
    let mut covered: Vec<(u32, u32)> = ranges
        .iter()
        .map(|r| (r.start as u32, r.end as u32))
        .collect();
    // The surrogate range U+D800..=U+DFFF has no `char` representation.
    // Inject it as a fake-covered interval so the sweep skips over it,
    // splitting the output into two segments around the hole instead of
    // one that crosses it.
    covered.push((0xD800, 0xDFFF));
    covered.sort_by_key(|&(start, _)| start);

    let mut result = Vec::new();
    let mut cursor: u32 = 0;
    for (start, end) in covered {
        if cursor < start {
            result.push(CharRange {
                start: char::from_u32(cursor).unwrap(),
                end: char::from_u32(start - 1).unwrap(),
            });
        }
        if end + 1 > cursor {
            cursor = end + 1;
        }
    }
    if cursor <= char::MAX as u32 {
        result.push(CharRange {
            start: char::from_u32(cursor).unwrap(),
            end: char::MAX,
        });
    }
    result
}

/// Sorts `ranges` and merges the ranges that overlap or touch. The result is
/// sorted and disjoint, with a gap between every two ranges.
pub fn merge_ranges(mut ranges: Vec<CharRange>) -> Vec<CharRange> {
    ranges.sort_by_key(|r| r.start);
    let mut merged: Vec<CharRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.start as u32 <= last.end as u32 + 1 => {
                last.end = last.end.max(range.end);
            }
            _ => merged.push(range),
        }
    }
    merged
}

impl Regex {
    /// Returns true if this regex can match the empty string.
    pub fn is_nullable(&self) -> bool {
        match self {
            Regex::Char(_) | Regex::CharRange(_) | Regex::CharClass(_) => false,
            Regex::Epsilon | Regex::Star(_) | Regex::Opt(_) => true,
            Regex::Plus(inner) => inner.is_nullable(),
            Regex::Seq(parts) => parts.iter().all(|r| r.is_nullable()),
            Regex::Alt(choices) => choices.iter().any(|r| r.is_nullable()),
            Regex::Identifier(_) => {
                unreachable!("Regex::Identifier should be inlined before calling is_nullable")
            }
        }
    }

    /// The character ranges that can begin this regular expression,
    /// sorted as disjoint ranges.
    pub fn first_chars(&self) -> Vec<CharRange> {
        let mut ranges = Vec::new();
        self.collect_first_chars(&mut ranges);
        merge_ranges(ranges)
    }

    fn collect_first_chars(&self, ranges: &mut Vec<CharRange>) {
        match self {
            Regex::Char(c) => ranges.push(CharRange { start: *c, end: *c }),
            Regex::CharRange(r) => ranges.push(*r),
            Regex::CharClass(class) => ranges.extend(class.char_ranges()),
            Regex::Seq(parts) => {
                for part in parts {
                    part.collect_first_chars(ranges);
                    if !part.is_nullable() {
                        break;
                    }
                }
            }
            Regex::Alt(choices) => {
                for choice in choices {
                    choice.collect_first_chars(ranges);
                }
            }
            Regex::Star(inner) | Regex::Plus(inner) | Regex::Opt(inner) => {
                inner.collect_first_chars(ranges)
            }
            Regex::Epsilon => {}
            Regex::Identifier(_) => {
                unreachable!("Regex::Identifier should be inlined before calling first_chars")
            }
        }
    }

    pub fn char(c: char) -> Self {
        Regex::Char(c)
    }

    pub fn range(start: char, end: char) -> Self {
        Regex::CharRange(CharRange { start, end })
    }

    pub fn seq(parts: Vec<Regex>) -> Self {
        Regex::Seq(parts)
    }

    pub fn alt(choices: Vec<Regex>) -> Self {
        Regex::Alt(choices)
    }

    pub fn star(regex: Regex) -> Self {
        Regex::Star(Box::new(regex))
    }

    pub fn plus(regex: Regex) -> Self {
        Regex::Plus(Box::new(regex))
    }

    pub fn char_class(ranges: Vec<CharRange>, negated: bool) -> Self {
        Regex::CharClass(CharClass { ranges, negated })
    }

    pub fn literal(s: &str) -> Self {
        Regex::Seq(s.chars().map(Regex::Char).collect())
    }
}

impl std::fmt::Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Regex::Char(c) => write!(f, "{}", c.escape_debug()),
            Regex::CharRange(r) => write!(f, "{}", r),
            Regex::Seq(parts) => {
                for part in parts {
                    write!(f, "{}", part)?;
                }
                Ok(())
            }
            // A single-branch alternation only arises from a rule body that has
            // one alternative. The grammar's nested alternation always has two or
            // more branches, so dropping the parentheses here never under-groups
            // a choice inside a rule.
            Regex::Alt(choices) if choices.len() == 1 => write!(f, "{}", choices[0]),
            Regex::Alt(choices) => {
                write!(f, "(")?;
                for (i, choice) in choices.iter().enumerate() {
                    if i > 0 {
                        write!(f, "|")?;
                    }
                    write!(f, "{}", choice)?;
                }
                write!(f, ")")
            }
            Regex::Star(inner) => {
                if needs_grouping(inner) {
                    write!(f, "({})*", inner)
                } else {
                    write!(f, "{}*", inner)
                }
            }
            Regex::Plus(inner) => {
                if needs_grouping(inner) {
                    write!(f, "({})+", inner)
                } else {
                    write!(f, "{}+", inner)
                }
            }
            Regex::CharClass(cc) => {
                let ranges_to_string = cc.ranges.iter().map(|r| r.to_string()).join(" ");
                if cc.negated {
                    write!(f, "![{}]", ranges_to_string)
                } else {
                    write!(f, "[{}]", ranges_to_string)
                }
            }
            Regex::Opt(inner) => {
                if needs_grouping(inner) {
                    write!(f, "({})?", inner)
                } else {
                    write!(f, "{}?", inner)
                }
            }
            Regex::Epsilon => write!(f, "ε"),
            Regex::Identifier(id) => write!(f, "{}", id.name),
        }
    }
}

/// Whether a `Star`/`Plus`/`Opt` operand needs parentheses to bind correctly.
/// A multi-element sequence does; an alternation prints its own parentheses, and
/// an atom needs none. Single-element sequences and alternations render as their
/// one child, so the decision sees through to that child.
fn needs_grouping(regex: &Regex) -> bool {
    match regex {
        Regex::Seq(parts) if parts.len() == 1 => needs_grouping(&parts[0]),
        Regex::Alt(choices) if choices.len() == 1 => needs_grouping(&choices[0]),
        Regex::Seq(_) => true,
        _ => false,
    }
}

#[macro_export]
macro_rules! c {
    ($c:literal) => {
        $crate::grammar::regex::Regex::Char($c)
    };
}

#[macro_export]
macro_rules! r {
    [$start:literal - $end:literal] => {
        $crate::grammar::regex::Regex::CharRange($crate::grammar::regex::CharRange {
            start: $start,
            end: $end,
        })
    };
}

#[macro_export]
macro_rules! r_seq {
    ($($part:expr),* $(,)?) => {
        $crate::grammar::regex::Regex::Seq(vec![$($part),*])
    };
}

#[macro_export]
macro_rules! r_alt {
    ($($choice:expr),* $(,)?) => {
        $crate::grammar::regex::Regex::Alt(vec![$($choice),*])
    };
}

#[macro_export]
macro_rules! r_star {
    ($inner:expr) => {
        $crate::grammar::regex::Regex::Star(Box::new($inner))
    };
}

#[macro_export]
macro_rules! r_plus {
    ($inner:expr) => {
        $crate::grammar::regex::Regex::Plus(Box::new($inner))
    };
}

#[macro_export]
macro_rules! r_opt {
    ($inner:expr) => {
        $crate::grammar::regex::Regex::Opt(Box::new($inner))
    };
}

#[macro_export]
macro_rules! cc {
    ([$($start:literal - $end:literal),* $(,)?]) => {
        $crate::grammar::regex::Regex::CharClass($crate::grammar::regex::CharClass {
            ranges: vec![
                $($crate::grammar::regex::CharRange { start: $start, end: $end }),*
            ],
            negated: false,
        })
    };
    (![$($start:literal - $end:literal),* $(,)?]) => {
        $crate::grammar::regex::Regex::CharClass($crate::grammar::regex::CharClass {
            ranges: vec![
                $($crate::grammar::regex::CharRange { start: $start, end: $end }),*
            ],
            negated: true,
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_range(start: char, end: char) -> CharRange {
        CharRange { start, end }
    }

    #[test]
    fn plus_is_nullable_when_its_operand_is_nullable() {
        assert!(Regex::Plus(Box::new(Regex::Opt(Box::new(Regex::Char('a'))))).is_nullable());
        assert!(!Regex::Plus(Box::new(Regex::Char('a'))).is_nullable());
    }

    #[test]
    fn complement_of_empty_is_unicode_split_around_the_surrogate_gap() {
        assert_eq!(
            complement(&[]),
            vec![
                char_range('\0', '\u{D7FF}'),
                char_range('\u{E000}', char::MAX)
            ]
        );
    }

    #[test]
    fn complement_emits_low_middle_and_high_segments() {
        assert_eq!(
            complement(&[char_range('a', 'c')]),
            vec![
                char_range('\0', '`'),
                char_range('d', '\u{D7FF}'),
                char_range('\u{E000}', char::MAX),
            ]
        );
    }

    #[test]
    fn complement_skips_segments_adjacent_to_the_surrogate_gap() {
        assert_eq!(
            complement(&[char_range('\0', '\u{D7FF}')]),
            vec![char_range('\u{E000}', char::MAX)]
        );
    }

    #[test]
    fn char_ranges_passes_non_negated_ranges_through() {
        let class = CharClass {
            ranges: vec![char_range('a', 'c'), char_range('x', 'z')],
            negated: false,
        };
        assert_eq!(
            class.char_ranges(),
            vec![char_range('a', 'c'), char_range('x', 'z')]
        );
    }

    #[test]
    fn char_ranges_complements_negated_ranges() {
        let class = CharClass {
            ranges: vec![char_range('a', 'c')],
            negated: true,
        };
        assert_eq!(
            class.char_ranges(),
            vec![
                char_range('\0', '`'),
                char_range('d', '\u{D7FF}'),
                char_range('\u{E000}', char::MAX),
            ],
        );
    }

    #[test]
    fn merge_ranges_sorts_and_merges_overlapping_and_touching_ranges() {
        assert_eq!(
            merge_ranges(vec![
                char_range('x', 'z'),
                char_range('a', 'c'),
                char_range('b', 'e'),
                char_range('f', 'g')
            ]),
            vec![char_range('a', 'g'), char_range('x', 'z')]
        );
    }

    #[test]
    fn first_chars_of_a_sequence_stop_at_the_first_part_that_is_not_nullable() {
        let regex = Regex::seq(vec![
            Regex::Opt(Box::new(Regex::char('-'))),
            Regex::star(Regex::char('_')),
            Regex::range('0', '9'),
            Regex::char('x'),
        ]);
        assert_eq!(
            regex.first_chars(),
            vec![
                char_range('-', '-'),
                char_range('0', '9'),
                char_range('_', '_')
            ]
        );
    }

    #[test]
    fn first_chars_of_an_alternation_are_the_union_of_its_choices() {
        let regex = Regex::alt(vec![
            Regex::literal("if"),
            Regex::literal("int"),
            Regex::plus(Regex::range('0', '9')),
        ]);
        assert_eq!(
            regex.first_chars(),
            vec![char_range('0', '9'), char_range('i', 'i')]
        );
    }

    #[test]
    fn first_chars_of_a_negated_class_are_its_complement() {
        let regex = Regex::char_class(vec![char_range('\0', '\u{7f}')], true);
        assert_eq!(
            regex.first_chars(),
            vec![
                char_range('\u{80}', '\u{D7FF}'),
                char_range('\u{E000}', char::MAX)
            ]
        );
    }

    #[test]
    fn first_chars_of_epsilon_are_empty() {
        assert_eq!(Regex::Epsilon.first_chars(), vec![]);
    }
}
