use allocator_api2::vec::{IntoIter as AVecIntoIter, Vec as AVec};
use bumpalo::Bump;
use hashbrown::{HashMap as AMap, HashTable, hash_map};
use rustc_hash::FxBuildHasher;

use crate::{
    ids::Id,
    utils::id_vec::{IdVec, Placeholder},
};

/// The arena that holds a parse tree and the parser's internal collections.
/// Everything allocated here is freed at once when the arena is reset or
/// dropped.
#[derive(Debug, Default)]
pub struct Arena(Bump);

/// A `Vec` allocated from an [`Arena`]. Backs the parser's descriptors and
/// other large accumulators, and the spilled buffers of `InlineVec`.
pub type ArenaVec<'arena, T> = AVec<T, &'arena Bump>;

/// An [`IdVec`] allocated from an [`Arena`]. Backs the parser's SPPF node,
/// GSS node, and environment stores.
pub type ArenaIdVec<'arena, I, T> = IdVec<I, T, &'arena Bump>;

/// The owning iterator of an [`ArenaVec`].
pub type ArenaVecIntoIter<'arena, T> = AVecIntoIter<T, &'arena Bump>;

/// An append-only vector allocated from an [`Arena`], stored as a list of
/// chunks. When a chunk becomes full, it stays as is, and the next push goes
/// to a new chunk with twice the capacity.
/// Growing an [`ArenaVec`] copies the whole buffer and leaves the old buffer
/// in the arena until the arena is reset, so a chunked vector avoids both the
/// copy and the dead buffer when it grows.
pub struct ChunkedArenaVec<'arena, T> {
    arena: &'arena Arena,
    /// The chunks before `current`, all full.
    full_chunks: ArenaVec<'arena, ArenaVec<'arena, T>>,
    /// The chunk that the next push goes to.
    current: ArenaVec<'arena, T>,
}

impl<'arena, T> ChunkedArenaVec<'arena, T> {
    const FIRST_CHUNK_CAPACITY: usize = 16;

    pub fn new(arena: &'arena Arena) -> Self {
        Self {
            arena,
            full_chunks: arena.vec(),
            current: arena.vec_with_capacity(Self::FIRST_CHUNK_CAPACITY),
        }
    }

    /// Appends `value`.
    #[inline]
    pub fn push(&mut self, value: T) {
        if let Err(value) = self.current.push_within_capacity(value) {
            self.push_to_new_chunk(value);
        }
    }

    /// Starts a chunk with twice the capacity of the current one and appends
    /// `value` to it. Runs once per chunk.
    #[cold]
    #[inline(never)]
    fn push_to_new_chunk(&mut self, value: T) {
        let capacity = 2 * self.current.capacity();
        let full = std::mem::replace(&mut self.current, self.arena.vec_with_capacity(capacity));
        self.full_chunks.push(full);
        self.current.push(value);
    }

    /// The number of items, in time logarithmic in that number.
    pub fn len(&self) -> usize {
        self.full_chunks
            .iter()
            .map(|chunk| chunk.len())
            .sum::<usize>()
            + self.current.len()
    }

    /// True if the vector does not hold any items.
    pub fn is_empty(&self) -> bool {
        // A new chunk receives its first item when it is started, so the
        // vector is empty exactly when the current chunk is.
        self.current.is_empty()
    }

    /// Removes every item. The current chunk keeps its capacity, and the
    /// other chunks stay in the arena until it is reset.
    pub fn clear(&mut self) {
        self.full_chunks.clear();
        self.current.clear();
    }

    /// The items in push order.
    pub fn iter(&self) -> ChunkedArenaVecIter<'_, 'arena, T> {
        self.full_chunks.iter().flatten().chain(self.current.iter())
    }
}

/// The borrowing iterator of a [`ChunkedArenaVec`], in push order.
pub type ChunkedArenaVecIter<'a, 'arena, T> = std::iter::Chain<
    std::iter::Flatten<std::slice::Iter<'a, ArenaVec<'arena, T>>>,
    std::slice::Iter<'a, T>,
>;

impl<'a, 'arena, T> IntoIterator for &'a ChunkedArenaVec<'arena, T> {
    type Item = &'a T;
    type IntoIter = ChunkedArenaVecIter<'a, 'arena, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An Fx-hashed `HashMap` allocated from an [`Arena`]. Backs the spilled
/// tables of `InlineMap` and the GSS-node indexes of parameterized
/// nonterminals.
///
/// `ArenaMap` is hashbrown's map rather than `FxHashMap`, because `std`'s
/// `HashMap` has no allocator parameter and so cannot live in an arena. The
/// hasher is the same one `FxHashMap` uses.
pub type ArenaMap<'arena, K, V> = AMap<K, V, FxBuildHasher, &'arena Bump>;

/// The borrowing iterator of an [`ArenaMap`].
pub type ArenaMapIter<'a, K, V> = hash_map::Iter<'a, K, V>;

/// A hashbrown `HashTable` allocated from an [`Arena`]. The table stores only
/// values. Its caller supplies the hash of each operation, and a closure that
/// compares or rehashes the stored values.
pub type ArenaTable<'arena, T> = HashTable<T, &'arena Bump>;

impl Arena {
    pub fn new() -> Self {
        Arena(Bump::new())
    }

    /// Moves `value` into the arena.
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.0.alloc(value)
    }

    /// Moves the items of `iter` into the arena as one contiguous slice. The
    /// iterator must know its length, so the slice is allocated once rather
    /// than grown.
    #[inline]
    pub fn alloc_slice<T, I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.0.alloc_slice_fill_iter(iter)
    }

    /// An empty vector allocated from this arena.
    #[inline]
    pub fn vec<T>(&self) -> ArenaVec<'_, T> {
        AVec::new_in(&self.0)
    }

    /// A vector allocated from this arena with room for `capacity` items.
    #[inline]
    pub fn vec_with_capacity<T>(&self, capacity: usize) -> ArenaVec<'_, T> {
        AVec::with_capacity_in(capacity, &self.0)
    }

    /// An empty id-addressed store allocated from this arena.
    #[inline]
    pub fn id_vec<I: Id, T: Placeholder>(&self) -> ArenaIdVec<'_, I, T> {
        IdVec::with_capacity_in(0, &self.0)
    }

    /// An id-addressed store allocated from this arena with room for
    /// `capacity` values.
    #[inline]
    pub fn id_vec_with_capacity<I: Id, T: Placeholder>(
        &self,
        capacity: usize,
    ) -> ArenaIdVec<'_, I, T> {
        IdVec::with_capacity_in(capacity, &self.0)
    }

    /// A map allocated from this arena with room for `capacity` entries.
    #[inline]
    pub fn map_with_capacity<K, V>(&self, capacity: usize) -> ArenaMap<'_, K, V> {
        AMap::with_capacity_and_hasher_in(capacity, FxBuildHasher, &self.0)
    }

    /// A table allocated from this arena with room for `capacity` entries.
    #[inline]
    pub fn table_with_capacity<T>(&self, capacity: usize) -> ArenaTable<'_, T> {
        HashTable::with_capacity_in(capacity, &self.0)
    }

    /// Frees every allocation at once and keeps the memory for reuse. Takes
    /// `&mut self`, so it cannot run while anything still borrows the arena.
    pub fn reset(&mut self) {
        self.0.reset()
    }
}

#[cfg(test)]
mod tests {
    use crate::arena::{Arena, ChunkedArenaVec};

    #[test]
    fn chunked_vec_starts_empty() {
        let arena = Arena::new();
        let v: ChunkedArenaVec<usize> = ChunkedArenaVec::new(&arena);
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
        assert_eq!(v.iter().count(), 0);
    }

    #[test]
    fn chunked_vec_keeps_push_order_across_chunks() {
        let arena = Arena::new();
        let mut v = ChunkedArenaVec::new(&arena);
        // 1,000 items take six chunks: 16, 32, 64, 128, 256, and 512.
        for i in 0..1000 {
            v.push(i);
        }
        assert_eq!(v.len(), 1000);
        assert!(!v.is_empty());
        let items: Vec<usize> = v.iter().copied().collect();
        assert_eq!(items, (0..1000).collect::<Vec<_>>());
        let by_ref: Vec<usize> = (&v).into_iter().copied().collect();
        assert_eq!(by_ref, items);
    }

    #[test]
    fn chunked_vec_keeps_items_in_place_when_it_grows() {
        let arena = Arena::new();
        let mut v = ChunkedArenaVec::new(&arena);
        for i in 0..1000 {
            v.push(i);
        }
        let before: Vec<*const usize> = v.iter().map(std::ptr::from_ref).collect();
        for i in 1000..5000 {
            v.push(i);
        }
        let after: Vec<*const usize> = v.iter().take(1000).map(std::ptr::from_ref).collect();
        assert_eq!(after, before);
    }

    #[test]
    fn chunked_vec_grows_after_clear() {
        let arena = Arena::new();
        let mut v = ChunkedArenaVec::new(&arena);
        for i in 0..100 {
            v.push(i);
        }
        v.clear();
        // The 200 pushes after the clear cross more than one chunk boundary.
        for i in 0..200 {
            v.push(i);
        }
        assert_eq!(v.len(), 200);
        let items: Vec<usize> = v.iter().copied().collect();
        assert_eq!(items, (0..200).collect::<Vec<_>>());
    }

    #[test]
    fn chunked_vec_clear_removes_every_item() {
        let arena = Arena::new();
        let mut v = ChunkedArenaVec::new(&arena);
        for i in 0..100 {
            v.push(i);
        }
        v.clear();
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
        assert_eq!(v.iter().count(), 0);
        v.push(7);
        v.push(8);
        assert_eq!(v.iter().copied().collect::<Vec<_>>(), vec![7, 8]);
    }
}
