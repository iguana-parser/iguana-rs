use std::{array, iter, slice};

#[derive(Debug, Default, Clone)]
pub enum InlineSet<T: Clone + Eq> {
    #[default]
    Empty,
    Single(T),
    Pair(T, T),
    Multiple(Vec<T>),
}

impl<T: Clone + Eq> InlineSet<T> {
    pub fn push(&mut self, value: T) {
        match self {
            InlineSet::Empty => *self = InlineSet::Single(value),
            InlineSet::Single(_) => match std::mem::take(self) {
                InlineSet::Single(t) => *self = InlineSet::Pair(t, value),
                _ => unreachable!(),
            },
            InlineSet::Pair(_, _) => match std::mem::take(self) {
                InlineSet::Pair(first, second) => {
                    let mut v = Vec::with_capacity(8);
                    v.push(first);
                    v.push(second);
                    v.push(value);
                    *self = InlineSet::Multiple(v)
                }
                _ => unreachable!(),
            },
            InlineSet::Multiple(items) => items.push(value),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            InlineSet::Empty => 0,
            InlineSet::Single(_) => 1,
            InlineSet::Pair(_, _) => 2,
            InlineSet::Multiple(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn contains(&self, value: &T) -> bool {
        match self {
            InlineSet::Empty => false,
            InlineSet::Single(v) => v == value,
            InlineSet::Pair(a, b) => a == value || b == value,
            InlineSet::Multiple(vec) => vec.contains(value),
        }
    }
}

pub enum Iter<'a, T> {
    Empty(iter::Empty<&'a T>),
    Single(iter::Once<&'a T>),
    Pair(array::IntoIter<&'a T, 2>),
    Multiple(slice::Iter<'a, T>),
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Empty(iter) => iter.next(),
            Iter::Single(iter) => iter.next(),
            Iter::Pair(iter) => iter.next(),
            Iter::Multiple(iter) => iter.next(),
        }
    }
}

impl<'a, T: Clone + Eq> IntoIterator for &'a InlineSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            InlineSet::Empty => Iter::Empty(iter::empty()),
            InlineSet::Single(x) => Iter::Single(iter::once(x)),
            InlineSet::Pair(a, b) => Iter::Pair([a, b].into_iter()),
            InlineSet::Multiple(v) => Iter::Multiple(v.iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::inline_set::InlineSet;

    #[test]
    fn test_default() {
        let l: InlineSet<usize> = InlineSet::default();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn test_add_to_empty() {
        let mut l = InlineSet::default();
        l.push(1);
        assert_eq!(l.len(), 1);
        let elements: Vec<&usize> = l.into_iter().collect();
        assert_eq!(elements, vec![&1]);
    }

    #[test]
    fn test_add_to_single() {
        let mut l = InlineSet::Single(1);
        l.push(2);
        assert_eq!(l.len(), 2);
        assert!(matches!(l, InlineSet::Pair(1, 2)));
        let elements: Vec<&usize> = l.into_iter().collect();
        assert_eq!(elements, vec![&1, &2]);
    }

    #[test]
    fn test_add_to_pair() {
        let mut l = InlineSet::Pair(1, 2);
        l.push(3);
        assert_eq!(l.len(), 3);
        assert!(matches!(l, InlineSet::Multiple(_)));
        let elements: Vec<&usize> = l.into_iter().collect();
        assert_eq!(elements, vec![&1, &2, &3]);
    }

    #[test]
    fn test_contains_pair() {
        let l = InlineSet::Pair(1, 2);
        assert!(l.contains(&1));
        assert!(l.contains(&2));
        assert!(!l.contains(&3));
    }
}
