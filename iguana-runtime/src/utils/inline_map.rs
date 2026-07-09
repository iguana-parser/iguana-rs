use std::{array, hash::Hash, iter};

use bumpalo::Bump;
use hashbrown::{HashMap as AMap, hash_map};
use rustc_hash::FxBuildHasher;

/// Initial capacity of the map inside `Multiple`. Hashbrown sizes the table by
/// rounding up to a power of two at the 7/8 load factor, so 7 lands at the
/// 8-bucket size class (room for 7 entries before rehash). 8 would jump to 16
/// buckets and double the memory footprint.
const MULTIPLE_CAPACITY: usize = 7;

/// An insertion-only map optimized for the common case of holding 0, 1, or 2
/// entries without any heap allocation. Once it grows beyond 2 entries it
/// spills into a `HashMap` allocated from an arena.
///
/// The arena is passed rather than stored, so the inline variants stay
/// allocator-free and minimal-width. Spilled tables live in the arena and
/// free all at once, not one drop each.
#[derive(Debug, Default)]
pub enum InlineMap<'arena, K: Clone + Eq + Hash, V: Clone> {
    #[default]
    Empty,
    Single((K, V)),
    Pair((K, V), (K, V)),
    Multiple(AMap<K, V, FxBuildHasher, &'arena Bump>),
}

impl<'arena, K: Clone + Eq + Hash, V: Clone> InlineMap<'arena, K, V> {
    /// Inserts `(key, value)`. Allocates the spilled table from `arena` on the
    /// `Pair`-to-`Multiple` transition; smaller variants ignore it. The caller
    /// must ensure `key` is absent; inserting a duplicate promotes the map early
    /// and leaves that key's lookup unspecified.
    pub fn insert(&mut self, key: K, value: V, arena: &'arena Bump) {
        match self {
            InlineMap::Empty => *self = InlineMap::Single((key, value)),
            InlineMap::Single(_) => match std::mem::take(self) {
                InlineMap::Single(p0) => *self = InlineMap::Pair(p0, (key, value)),
                _ => unreachable!(),
            },
            InlineMap::Pair(_, _) => match std::mem::take(self) {
                InlineMap::Pair(p0, p1) => {
                    let mut map =
                        AMap::with_capacity_and_hasher_in(MULTIPLE_CAPACITY, FxBuildHasher, arena);
                    map.insert(p0.0, p0.1);
                    map.insert(p1.0, p1.1);
                    map.insert(key, value);
                    *self = InlineMap::Multiple(map);
                }
                _ => unreachable!(),
            },
            InlineMap::Multiple(items) => {
                items.insert(key, value);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self {
            InlineMap::Empty => None,
            InlineMap::Single((k, v)) => (k == key).then_some(v),
            InlineMap::Pair((k0, v0), (k1, v1)) => {
                if k0 == key {
                    Some(v0)
                } else if k1 == key {
                    Some(v1)
                } else {
                    None
                }
            }
            InlineMap::Multiple(map) => map.get(key),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            InlineMap::Empty => 0,
            InlineMap::Single(_) => 1,
            InlineMap::Pair(_, _) => 2,
            InlineMap::Multiple(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.into_iter()
    }
}

pub enum Iter<'a, K, V> {
    Empty(iter::Empty<(&'a K, &'a V)>),
    Single(iter::Once<(&'a K, &'a V)>),
    Pair(array::IntoIter<(&'a K, &'a V), 2>),
    Multiple(hash_map::Iter<'a, K, V>),
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Empty(empty) => empty.next(),
            Iter::Single(once) => once.next(),
            Iter::Pair(iter) => iter.next(),
            Iter::Multiple(iter) => iter.next(),
        }
    }
}

impl<'a, 'arena, K: Clone + Eq + Hash, V: Clone> IntoIterator for &'a InlineMap<'arena, K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            InlineMap::Empty => Iter::Empty(iter::empty()),
            InlineMap::Single(x) => Iter::Single(iter::once((&x.0, &x.1))),
            InlineMap::Pair(a, b) => Iter::Pair([(&a.0, &a.1), (&b.0, &b.1)].into_iter()),
            InlineMap::Multiple(v) => Iter::Multiple(v.iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bumpalo::Bump;

    use crate::utils::inline_map::InlineMap;

    #[test]
    fn test_default() {
        let l: InlineMap<usize, usize> = InlineMap::default();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn test_add_to_empty() {
        let arena = Bump::new();
        let mut map = InlineMap::default();
        map.insert(1, 2, &arena);
        assert_eq!(map.len(), 1);
        let elements: Vec<(&usize, &usize)> = map.into_iter().collect();
        assert_eq!(elements, vec![(&1, &2)]);
    }

    #[test]
    fn test_grows_through_pair() {
        let arena = Bump::new();
        let mut map: InlineMap<usize, usize> = InlineMap::default();
        map.insert(1, 10, &arena);
        assert!(matches!(map, InlineMap::Single(_)));
        map.insert(2, 20, &arena);
        assert!(matches!(map, InlineMap::Pair(_, _)));
        map.insert(3, 30, &arena);
        assert!(matches!(map, InlineMap::Multiple(_)));
        let collected: HashMap<&usize, &usize> = map.iter().collect();
        assert_eq!(collected, HashMap::from([(&1, &10), (&2, &20), (&3, &30)]));
    }

    #[test]
    fn test_add_to_single() {
        let arena = Bump::new();
        let mut l = InlineMap::Single((1, 2));
        l.insert(2, 3, &arena);
        l.insert(3, 4, &arena);
        l.insert(4, 5, &arena);
        l.insert(5, 6, &arena);
        assert_eq!(l.len(), 5);
        let elements: HashMap<&usize, &usize> = l.into_iter().collect();
        assert_eq!(
            elements,
            HashMap::from([(&1, &2), (&2, &3), (&3, &4), (&4, &5), (&5, &6)])
        );
    }
}
