use std::{array, iter, slice};

/// A vector optimized for the common case of holding 0, 1, 2, or 3 elements
/// without any heap allocation. Once it grows beyond 3 elements it spills
/// into a heap-allocated `Vec`.
///
/// `SPILL_CAP` is the initial `Vec::with_capacity` used when transitioning
/// from `Triple` to `Multiple`. Tune this per use site (via a type alias) to
/// match the observed size distribution: pick a value just above the largest
/// common cluster so the first heap allocation already fits and avoids any
/// realloc. Default is 8, which is a good fit when the long tail dies off
/// before 8 elements.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum InlineVec<T, const SPILL_CAP: usize = 8> {
    #[default]
    Empty,
    Single(T),
    Pair(T, T),
    Triple(T, T, T),
    Multiple(Vec<T>),
}

impl<T, const SPILL_CAP: usize> InlineVec<T, SPILL_CAP> {
    pub fn push(&mut self, value: T) {
        match self {
            InlineVec::Empty => *self = InlineVec::Single(value),
            InlineVec::Single(_) => match std::mem::take(self) {
                InlineVec::Single(t) => *self = InlineVec::Pair(t, value),
                _ => unreachable!(),
            },
            InlineVec::Pair(_, _) => match std::mem::take(self) {
                InlineVec::Pair(first, second) => *self = InlineVec::Triple(first, second, value),
                _ => unreachable!(),
            },
            InlineVec::Triple(_, _, _) => match std::mem::take(self) {
                InlineVec::Triple(first, second, third) => {
                    let mut v = Vec::with_capacity(SPILL_CAP);
                    v.push(first);
                    v.push(second);
                    v.push(third);
                    v.push(value);
                    *self = InlineVec::Multiple(v)
                }
                _ => unreachable!(),
            },
            InlineVec::Multiple(items) => items.push(value),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            InlineVec::Empty => 0,
            InlineVec::Single(_) => 1,
            InlineVec::Pair(_, _) => 2,
            InlineVec::Triple(_, _, _) => 3,
            InlineVec::Multiple(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn first(&self) -> Option<&T> {
        match self {
            InlineVec::Empty => None,
            InlineVec::Single(v) => Some(v),
            InlineVec::Pair(first, _) => Some(first),
            InlineVec::Triple(first, _, _) => Some(first),
            InlineVec::Multiple(v) => v.first(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match self {
            InlineVec::Empty => None,
            InlineVec::Single(v) => (index == 0).then_some(v),
            InlineVec::Pair(first, second) => match index {
                0 => Some(first),
                1 => Some(second),
                _ => None,
            },
            InlineVec::Triple(first, second, third) => match index {
                0 => Some(first),
                1 => Some(second),
                2 => Some(third),
                _ => None,
            },
            InlineVec::Multiple(v) => v.get(index),
        }
    }

    /// Drops all elements. If currently spilled, keeps the heap `Vec`'s
    /// capacity so subsequent pushes within the same level reuse it.
    pub fn clear(&mut self) {
        match self {
            InlineVec::Empty => {}
            InlineVec::Single(_) | InlineVec::Pair(_, _) | InlineVec::Triple(_, _, _) => {
                *self = InlineVec::Empty
            }
            InlineVec::Multiple(v) => v.clear(),
        }
    }
}

pub enum Iter<'a, T> {
    Empty(iter::Empty<&'a T>),
    Single(iter::Once<&'a T>),
    Pair(array::IntoIter<&'a T, 2>),
    Triple(array::IntoIter<&'a T, 3>),
    Multiple(slice::Iter<'a, T>),
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Empty(iter) => iter.next(),
            Iter::Single(iter) => iter.next(),
            Iter::Pair(iter) => iter.next(),
            Iter::Triple(iter) => iter.next(),
            Iter::Multiple(iter) => iter.next(),
        }
    }
}

impl<'a, T, const SPILL_CAP: usize> IntoIterator for &'a InlineVec<T, SPILL_CAP> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            InlineVec::Empty => Iter::Empty(iter::empty()),
            InlineVec::Single(x) => Iter::Single(iter::once(x)),
            InlineVec::Pair(first, second) => Iter::Pair([first, second].into_iter()),
            InlineVec::Triple(first, second, third) => {
                Iter::Triple([first, second, third].into_iter())
            }
            InlineVec::Multiple(v) => Iter::Multiple(v.iter()),
        }
    }
}

#[macro_export]
macro_rules! inline_vec {
    () => {
        $crate::utils::inline_vec::InlineVec::Empty
    };
    ($first:expr, $second:expr $(,)?) => {
        $crate::utils::inline_vec::InlineVec::Pair($first, $second)
    };
    ($first:expr, $second:expr, $third:expr $(,)?) => {
        $crate::utils::inline_vec::InlineVec::Triple($first, $second, $third)
    };
    ($first:expr, $($rest:expr),+ $(,)?) => {
        $crate::utils::inline_vec::InlineVec::Multiple(vec![$first $(, $rest)+])
    };
    ($elem:expr $(,)?) => {
        $crate::utils::inline_vec::InlineVec::Single($elem)
    };
}

pub use inline_vec;

#[cfg(test)]
mod tests {
    use crate::utils::inline_vec::InlineVec;

    #[test]
    fn test_default() {
        let l: InlineVec<usize, 8> = InlineVec::default();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn test_add_to_empty() {
        let mut l: InlineVec<usize, 8> = InlineVec::default();
        l.push(1);
        assert_eq!(l.len(), 1);
        let elements: Vec<&usize> = l.into_iter().collect();
        assert_eq!(elements, vec![&1]);
    }

    #[test]
    fn test_add_to_single() {
        let mut l: InlineVec<i32, 8> = InlineVec::Single(1);
        l.push(2);
        assert_eq!(l, InlineVec::Pair(1, 2));
        assert_eq!(l.len(), 2);
        let elements: Vec<&i32> = l.into_iter().collect();
        assert_eq!(elements, vec![&1, &2]);
    }

    #[test]
    fn test_add_to_pair() {
        let mut l: InlineVec<i32, 8> = InlineVec::Pair(1, 2);
        l.push(3);
        assert_eq!(l, InlineVec::Triple(1, 2, 3));
        assert_eq!(l.len(), 3);
        let elements: Vec<&i32> = l.into_iter().collect();
        assert_eq!(elements, vec![&1, &2, &3]);
    }

    #[test]
    fn test_add_to_triple() {
        let mut l: InlineVec<i32, 8> = InlineVec::Triple(1, 2, 3);
        l.push(4);
        assert_eq!(l, InlineVec::Multiple(vec![1, 2, 3, 4]));
        assert_eq!(l.len(), 4);
        let elements: Vec<&i32> = l.into_iter().collect();
        assert_eq!(elements, vec![&1, &2, &3, &4]);
    }

    #[test]
    fn test_pair_first() {
        let l: InlineVec<i32, 8> = InlineVec::Pair(1, 2);
        assert_eq!(l.first(), Some(&1));
    }

    #[test]
    fn empty_form() {
        let v: InlineVec<usize, 8> = inline_vec![];
        assert_eq!(v, InlineVec::Empty);
    }

    #[test]
    fn single_without_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1];
        assert_eq!(v, InlineVec::Single(1));
    }

    #[test]
    fn single_with_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1,];
        assert_eq!(v, InlineVec::Single(1));
    }

    #[test]
    fn pair_without_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1, 2];
        assert_eq!(v, InlineVec::Pair(1, 2));
    }

    #[test]
    fn pair_with_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1, 2,];
        assert_eq!(v, InlineVec::Pair(1, 2));
    }

    #[test]
    fn triple_without_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1, 2, 3];
        assert_eq!(v, InlineVec::Triple(1, 2, 3));
    }

    #[test]
    fn triple_with_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1, 2, 3,];
        assert_eq!(v, InlineVec::Triple(1, 2, 3));
    }

    #[test]
    fn multiple_without_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1, 2, 3, 4];
        assert_eq!(v, InlineVec::Multiple(vec![1, 2, 3, 4]));
    }

    #[test]
    fn multiple_with_trailing_comma() {
        let v: InlineVec<i32, 8> = inline_vec![1, 2, 3, 4,];
        assert_eq!(v, InlineVec::Multiple(vec![1, 2, 3, 4]));
    }
}
