use std::{array, iter, slice};

use allocator_api2::vec::{IntoIter as AVecIntoIter, Vec as AVec};
use bumpalo::Bump;

/// A vector optimized for the common case of holding 0, 1, 2, or 3 elements
/// without any heap allocation. Once it grows beyond 3 elements it spills
/// into a `Vec` allocated from an arena.
///
/// `MULTIPLE_CAPACITY` is the initial capacity of the `Vec` inside `Multiple`,
/// allocated on the `Triple`-to-`Multiple` transition. A sensible value covers
/// the most common use cases, so the vector rarely has to grow. The default
/// is 8.
///
/// The arena is passed rather than stored, so the inline variants stay
/// allocator-free and minimal-width. Spilled buffers live in the arena and
/// free all at once, not one drop each.
#[derive(Debug, Default, Clone)]
pub enum InlineVec<'arena, T, const MULTIPLE_CAPACITY: usize = 8> {
    #[default]
    Empty,
    Single(T),
    Pair(T, T),
    Triple(T, T, T),
    Multiple(AVec<T, &'arena Bump>),
}

impl<'arena, T, const MULTIPLE_CAPACITY: usize> InlineVec<'arena, T, MULTIPLE_CAPACITY> {
    /// Pushes `value`. Allocates the spilled buffer from `arena` on the
    /// `Triple`-to-`Multiple` transition; smaller variants ignore it.
    pub fn push(&mut self, value: T, arena: &'arena Bump) {
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
                    let mut v = AVec::with_capacity_in(MULTIPLE_CAPACITY, arena);
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

    /// Drops all elements. If currently spilled, keeps the arena `Vec`'s
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

impl<'arena, T: PartialEq, const MULTIPLE_CAPACITY: usize> PartialEq
    for InlineVec<'arena, T, MULTIPLE_CAPACITY>
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
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

impl<'a, 'arena, T, const MULTIPLE_CAPACITY: usize> IntoIterator
    for &'a InlineVec<'arena, T, MULTIPLE_CAPACITY>
{
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

pub enum IntoIter<'arena, T> {
    Empty(iter::Empty<T>),
    Single(iter::Once<T>),
    Pair(array::IntoIter<T, 2>),
    Triple(array::IntoIter<T, 3>),
    Multiple(AVecIntoIter<T, &'arena Bump>),
}

impl<'arena, T> Iterator for IntoIter<'arena, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Empty(iter) => iter.next(),
            IntoIter::Single(iter) => iter.next(),
            IntoIter::Pair(iter) => iter.next(),
            IntoIter::Triple(iter) => iter.next(),
            IntoIter::Multiple(iter) => iter.next(),
        }
    }
}

impl<'arena, T, const MULTIPLE_CAPACITY: usize> IntoIterator
    for InlineVec<'arena, T, MULTIPLE_CAPACITY>
{
    type Item = T;
    type IntoIter = IntoIter<'arena, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            InlineVec::Empty => IntoIter::Empty(iter::empty()),
            InlineVec::Single(x) => IntoIter::Single(iter::once(x)),
            InlineVec::Pair(first, second) => IntoIter::Pair([first, second].into_iter()),
            InlineVec::Triple(first, second, third) => {
                IntoIter::Triple([first, second, third].into_iter())
            }
            InlineVec::Multiple(v) => IntoIter::Multiple(v.into_iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    use crate::utils::inline_vec::InlineVec;

    #[test]
    fn test_default() {
        let l: InlineVec<usize, 8> = InlineVec::default();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn test_add_to_empty() {
        let arena = Bump::new();
        let mut l: InlineVec<usize, 8> = InlineVec::default();
        l.push(1, &arena);
        assert_eq!(l.len(), 1);
        let elements: Vec<&usize> = l.iter().collect();
        assert_eq!(elements, vec![&1]);
    }

    #[test]
    fn test_add_to_single() {
        let arena = Bump::new();
        let mut l: InlineVec<i32, 8> = InlineVec::Single(1);
        l.push(2, &arena);
        assert_eq!(l, InlineVec::Pair(1, 2));
        assert_eq!(l.len(), 2);
        let elements: Vec<&i32> = l.iter().collect();
        assert_eq!(elements, vec![&1, &2]);
    }

    #[test]
    fn test_add_to_pair() {
        let arena = Bump::new();
        let mut l: InlineVec<i32, 8> = InlineVec::Pair(1, 2);
        l.push(3, &arena);
        assert_eq!(l, InlineVec::Triple(1, 2, 3));
        assert_eq!(l.len(), 3);
        let elements: Vec<&i32> = l.iter().collect();
        assert_eq!(elements, vec![&1, &2, &3]);
    }

    #[test]
    fn test_add_to_triple() {
        let arena = Bump::new();
        let mut l: InlineVec<i32, 8> = InlineVec::Triple(1, 2, 3);
        l.push(4, &arena);
        assert!(matches!(l, InlineVec::Multiple(_)));
        assert_eq!(l.len(), 4);
        let elements: Vec<&i32> = l.iter().collect();
        assert_eq!(elements, vec![&1, &2, &3, &4]);
    }

    #[test]
    fn test_pair_first() {
        let l: InlineVec<i32, 8> = InlineVec::Pair(1, 2);
        assert_eq!(l.first(), Some(&1));
    }

    #[test]
    fn spills_past_triple() {
        let arena = Bump::new();
        let mut l: InlineVec<i32, 8> = InlineVec::default();
        for i in 1..=5 {
            l.push(i, &arena);
        }
        assert!(matches!(l, InlineVec::Multiple(_)));
        let collected: Vec<i32> = l.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn into_iter_yields_owned_values() {
        let arena = Bump::new();
        for n in 0..=5 {
            let mut v: InlineVec<i32, 8> = InlineVec::default();
            for i in 1..=n {
                v.push(i, &arena);
            }
            let collected: Vec<i32> = v.into_iter().collect();
            assert_eq!(collected, (1..=n).collect::<Vec<_>>());
        }
    }
}
