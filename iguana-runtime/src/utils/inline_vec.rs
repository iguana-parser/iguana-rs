use std::{iter, slice};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum InlineVec<T> {
    #[default]
    Empty,
    Single(T),
    Multiple(Vec<T>),
}

impl<T> InlineVec<T> {
    pub fn push(&mut self, value: T) {
        match self {
            InlineVec::Empty => *self = InlineVec::Single(value),
            InlineVec::Single(_) => match std::mem::take(self) {
                InlineVec::Single(current_value) => {
                    let mut v = Vec::with_capacity(8);
                    v.push(current_value);
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
            InlineVec::Multiple(v) => v.first(),
        }
    }
}

pub enum Iter<'a, T> {
    Empty(iter::Empty<&'a T>),
    Single(iter::Once<&'a T>),
    Multiple(slice::Iter<'a, T>),
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Empty(empty) => empty.next(),
            Iter::Single(once) => once.next(),
            Iter::Multiple(iter) => iter.next(),
        }
    }
}

impl<'a, T> IntoIterator for &'a InlineVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            InlineVec::Empty => Iter::Empty(iter::empty()),
            InlineVec::Single(x) => Iter::Single(iter::once(x)),
            InlineVec::Multiple(v) => Iter::Multiple(v.iter()),
        }
    }
}

#[macro_export]
macro_rules! inline_vec {
    () => {
        $crate::utils::inline_vec::InlineVec::Empty
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
        let l: InlineVec<usize> = InlineVec::default();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn test_add_to_empty() {
        let mut l = InlineVec::default();
        l.push(1);
        assert_eq!(l.len(), 1);
        let elements: Vec<&usize> = l.into_iter().collect();
        assert_eq!(elements, vec![&1]);
    }

    #[test]
    fn test_add_to_single() {
        let mut l = InlineVec::Single(1);
        l.push(2);
        l.push(3);
        assert_eq!(l.len(), 3);
        let elements: Vec<&usize> = l.into_iter().collect();
        assert_eq!(elements, vec![&1, &2, &3]);
    }

    #[test]
    fn empty_form() {
        let v: InlineVec<usize> = inline_vec![];
        assert_eq!(v, InlineVec::Empty);
    }

    #[test]
    fn single_without_trailing_comma() {
        let v = inline_vec![1];
        assert_eq!(v, InlineVec::Single(1));
    }

    #[test]
    fn single_with_trailing_comma() {
        let v = inline_vec![1,];
        assert_eq!(v, InlineVec::Single(1));
    }

    #[test]
    fn multiple_without_trailing_comma() {
        let v = inline_vec![1, 2, 3];
        assert_eq!(v, InlineVec::Multiple(vec![1, 2, 3]));
    }

    #[test]
    fn multiple_with_trailing_comma() {
        let v = inline_vec![1, 2, 3,];
        assert_eq!(v, InlineVec::Multiple(vec![1, 2, 3]));
    }
}
