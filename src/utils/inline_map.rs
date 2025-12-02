use std::{collections::hash_map, hash::Hash, iter};

use rustc_hash::FxHashMap;

#[derive(Debug, Default)]
pub enum InlineMap<K: Clone + Eq + Hash, V: Clone> {
    #[default]
    Empty,
    Single((K, V)),
    Multiple(FxHashMap<K, V>),
}

impl<K: Clone + Eq + Hash, V: Clone> InlineMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        match self {
            InlineMap::Empty => *self = InlineMap::Single((key, value)),
            InlineMap::Single((k, v)) => {
                let mut map = FxHashMap::default();
                map.insert(k.clone(), v.clone());
                map.insert(key, value);
                *self = InlineMap::Multiple(map)
            }
            InlineMap::Multiple(items) => {
                items.insert(key, value);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self {
            InlineMap::Empty => None,
            InlineMap::Single((k, v)) => {
                if k == key {
                    Some(v)
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
    Multiple(hash_map::Iter<'a, K, V>),
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Empty(empty) => empty.next(),
            Iter::Single(once) => once.next(),
            Iter::Multiple(iter) => iter.next(),
        }
    }
}

impl<'a, K: Clone + Eq + Hash, V: Clone> IntoIterator for &'a InlineMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            InlineMap::Empty => Iter::Empty(iter::empty()),
            InlineMap::Single(x) => Iter::Single(iter::once((&x.0, &x.1))),
            InlineMap::Multiple(v) => Iter::Multiple(v.iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::utils::inline_map::InlineMap;

    #[test]
    fn test_default() {
        let l: InlineMap<usize, usize> = InlineMap::default();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn test_add_to_empty() {
        let mut map = InlineMap::default();
        map.insert(1, 2);
        assert_eq!(map.len(), 1);
        let elements: Vec<(&usize, &usize)> = map.into_iter().collect();
        assert_eq!(elements, vec![(&1, &2)]);
    }

    #[test]
    fn test_add_to_single() {
        let mut l = InlineMap::Single((1, 2));
        l.insert(2, 3);
        l.insert(3, 4);
        assert_eq!(l.len(), 3);
        let elements: HashMap<&usize, &usize> = l.into_iter().collect();
        assert_eq!(elements, HashMap::from([(&1, &2), (&2, &3), (&3, &4)]));
    }
}
