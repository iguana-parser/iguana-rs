use allocator_api2::vec::{IntoIter as AVecIntoIter, Vec as AVec};
use bumpalo::Bump;
use hashbrown::{HashMap as AMap, hash_map};
use rustc_hash::FxBuildHasher;

/// The arena that holds a parse tree and the parser's internal collections.
/// Everything allocated here is freed at once when the arena is reset or
/// dropped.
#[derive(Debug, Default)]
pub struct Arena(Bump);

/// A `Vec` allocated from an [`Arena`]. Backs the parser's large accumulators
/// (descriptors, GSS and SPPF nodes) and the spilled buffers of `InlineVec`.
pub type ArenaVec<'arena, T> = AVec<T, &'arena Bump>;

/// The owning iterator of an [`ArenaVec`].
pub type ArenaVecIntoIter<'arena, T> = AVecIntoIter<T, &'arena Bump>;

/// An Fx-hashed `HashMap` allocated from an [`Arena`]. Backs the spilled
/// tables of `InlineMap`.
///
/// `ArenaMap` is hashbrown's map rather than `FxHashMap`, because `std`'s
/// `HashMap` has no allocator parameter and so cannot live in an arena. The
/// hasher is the same one `FxHashMap` uses.
pub type ArenaMap<'arena, K, V> = AMap<K, V, FxBuildHasher, &'arena Bump>;

/// The borrowing iterator of an [`ArenaMap`].
pub type ArenaMapIter<'a, K, V> = hash_map::Iter<'a, K, V>;

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

    /// A map allocated from this arena with room for `capacity` entries.
    #[inline]
    pub fn map_with_capacity<K, V>(&self, capacity: usize) -> ArenaMap<'_, K, V> {
        AMap::with_capacity_and_hasher_in(capacity, FxBuildHasher, &self.0)
    }

    /// Frees every allocation at once and keeps the memory for reuse. Takes
    /// `&mut self`, so it cannot run while anything still borrows the arena.
    pub fn reset(&mut self) {
        self.0.reset()
    }
}
