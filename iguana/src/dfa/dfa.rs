use iguana_runtime::ids::TerminalId;

use crate::grammar::regex::CharRange;

pub type StateId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub transitions: Vec<(CharRange, StateId)>,
    pub accept: Option<TerminalId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dfa {
    pub states: Vec<State>,
    pub start: StateId,
}

impl Dfa {
    pub fn num_states(&self) -> usize {
        self.states.len()
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

/// Partition `ranges` (which may overlap) into a sorted list of disjoint
/// sub-ranges whose union is the same set of characters.
fn to_non_overlapping(ranges: &[CharRange]) -> Vec<CharRange> {
    if ranges.is_empty() {
        return Vec::new();
    }
    // Each range contributes +1 at its start (coverage opens) and -1 at
    // end + 1 (coverage closes).
    let mut events: Vec<(u32, i32)> = Vec::with_capacity(ranges.len() * 2);
    for r in ranges {
        events.push((r.start as u32, 1));
        events.push((r.end as u32 + 1, -1));
    }
    events.sort_by_key(|&(pos, _)| pos);

    let mut result = Vec::new();
    let mut counter: i32 = 0;
    let mut prev: u32 = 0;
    let mut i = 0;
    while i < events.len() {
        let pos = events[i].0;
        // `counter` is the coverage over [prev, pos), the interval that just ended.
        if counter > 0 {
            // It was fully covered, so emit it as the inclusive range.
            result.push(CharRange {
                start: char::from_u32(prev).unwrap(),
                end: char::from_u32(pos - 1).unwrap(),
            });
        }
        // Apply every delta at this position before stepping — adjacent ranges
        // meet at the same `pos` and must collapse to one boundary.
        while i < events.len() && events[i].0 == pos {
            counter += events[i].1;
            i += 1;
        }
        // `counter` now describes the next interval [pos, next_event); anchor it.
        prev = pos;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cr(start: char, end: char) -> CharRange {
        CharRange { start, end }
    }

    #[test]
    fn complement_of_empty_is_unicode_split_around_the_surrogate_gap() {
        assert_eq!(
            complement(&[]),
            vec![cr('\0', '\u{D7FF}'), cr('\u{E000}', char::MAX)]
        );
    }

    #[test]
    fn complement_emits_low_middle_and_high_segments() {
        assert_eq!(
            complement(&[cr('a', 'c')]),
            vec![
                cr('\0', '`'),
                cr('d', '\u{D7FF}'),
                cr('\u{E000}', char::MAX),
            ]
        );
    }

    #[test]
    fn complement_skips_segments_adjacent_to_the_surrogate_gap() {
        assert_eq!(
            complement(&[cr('\0', '\u{D7FF}')]),
            vec![cr('\u{E000}', char::MAX)]
        );
    }

    #[test]
    fn to_non_overlapping_passes_through_disjoint_ranges() {
        assert_eq!(
            to_non_overlapping(&[cr('a', 'c'), cr('e', 'g')]),
            vec![cr('a', 'c'), cr('e', 'g')]
        );
    }

    #[test]
    fn to_non_overlapping_splits_overlapping_ranges() {
        assert_eq!(
            to_non_overlapping(&[cr('a', 'c'), cr('b', 'd')]),
            vec![cr('a', 'a'), cr('b', 'c'), cr('d', 'd')]
        );
    }

    #[test]
    fn to_non_overlapping_keeps_adjacent_ranges_separate() {
        assert_eq!(
            to_non_overlapping(&[cr('a', 'c'), cr('d', 'f')]),
            vec![cr('a', 'c'), cr('d', 'f')]
        );
    }

    #[test]
    fn to_non_overlapping_returns_empty_for_empty_input() {
        assert_eq!(to_non_overlapping(&[]), Vec::<CharRange>::new());
    }
}
