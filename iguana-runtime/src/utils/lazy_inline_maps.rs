use std::hash::Hash;

use crate::arena::Arena;
use crate::utils::inline_map::InlineMap;

/// A `LazyInlineMaps` is an arena-allocated array where each element is an
/// optional reference to an arena-allocated `InlineMap`.
///
/// The parser uses it for its lookup indexes: there is one map per nonterminal,
/// slot or terminal, and each map is keyed by an input position or span. There
/// are two reasons for this design:
/// 1. The parser holds only a reference to the array, so moving the parser does
///    not copy it, and allocating it is a bump in the arena.
/// 2. Most of the maps stay empty. A `None` takes one pointer and needs no drop,
///    while an empty `InlineMap` takes 48 bytes (on 64-bit targets) and has to be
///    dropped.
///
/// The maps are never dropped; resetting the arena frees them. Keys and values
/// are `Copy`, so they have no destructors that would need to run.
#[derive(Debug)]
pub struct LazyInlineMaps<'arena, K: Copy + Eq + Hash, V: Copy> {
    maps: &'arena mut [Option<&'arena mut InlineMap<'arena, K, V>>],
}

impl<'arena, K: Copy + Eq + Hash, V: Copy> LazyInlineMaps<'arena, K, V> {
    pub fn new(len: usize, arena: &'arena Arena) -> Self {
        Self {
            maps: arena.alloc_slice((0..len).map(|_| None)),
        }
    }

    #[inline]
    pub fn get(&self, index: usize, key: &K) -> Option<V> {
        self.maps[index].as_deref()?.get(key).copied()
    }

    /// Inserts `(key, value)` into map `index`, allocating the map on its first
    /// insert. As with `InlineMap`, `key` must be absent.
    #[inline]
    pub fn insert(&mut self, index: usize, key: K, value: V, arena: &'arena Arena) {
        self.maps[index]
            .get_or_insert_with(|| arena.alloc(InlineMap::Empty))
            .insert(key, value, arena);
    }

    /// The length of each map, in order; 0 for a map not yet allocated.
    pub fn map_lens(&self) -> impl Iterator<Item = usize> + '_ {
        self.maps
            .iter()
            .map(|map| map.as_deref().map_or(0, InlineMap::len))
    }
}

#[cfg(test)]
mod tests {
    use crate::arena::Arena;

    use crate::utils::lazy_inline_maps::LazyInlineMaps;

    #[test]
    fn maps_are_independent() {
        let arena = Arena::new();
        let mut maps: LazyInlineMaps<u32, u32> = LazyInlineMaps::new(4, &arena);
        assert_eq!(maps.map_lens().collect::<Vec<_>>(), [0, 0, 0, 0]);
        assert_eq!(maps.get(1, &7), None);

        maps.insert(1, 7, 70, &arena);
        maps.insert(3, 7, 700, &arena);
        assert_eq!(maps.get(1, &7), Some(70));
        assert_eq!(maps.get(3, &7), Some(700));
        assert_eq!(maps.get(0, &7), None);
        assert_eq!(maps.get(1, &8), None);
        assert_eq!(maps.map_lens().collect::<Vec<_>>(), [0, 1, 0, 1]);
    }

    #[test]
    fn a_map_spills_past_two_entries() {
        let arena = Arena::new();
        let mut maps: LazyInlineMaps<u32, u32> = LazyInlineMaps::new(1, &arena);
        for k in 0..20 {
            maps.insert(0, k, k * 10, &arena);
        }
        for k in 0..20 {
            assert_eq!(maps.get(0, &k), Some(k * 10));
        }
        assert_eq!(maps.get(0, &20), None);
        assert_eq!(maps.map_lens().collect::<Vec<_>>(), [20]);
    }

    #[test]
    fn an_empty_array_is_allowed() {
        let arena = Arena::new();
        let maps: LazyInlineMaps<u32, u32> = LazyInlineMaps::new(0, &arena);
        assert_eq!(maps.map_lens().count(), 0);
    }
}
