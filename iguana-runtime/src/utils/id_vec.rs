use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use allocator_api2::{
    alloc::{Allocator, Global},
    vec::Vec,
};

use crate::ids::Id;

/// A vector of values addressed by an `Id` type, such as `SPPFNodeId`, instead
/// of `usize`. Index 0 holds a placeholder because an `Id` starts at 1.
#[derive(Debug)]
pub struct IdVec<I, T, A: Allocator = Global> {
    items: Vec<T, A>,
    _id: PhantomData<I>,
}

/// The placeholder an `IdVec` stores at index 0. It is never accessed
/// through an id, since ids start at 1.
pub trait Placeholder {
    fn placeholder() -> Self;
}

impl<I: Id, T: Placeholder> IdVec<I, T> {
    /// An empty store on the global allocator.
    pub fn new() -> Self {
        Self::with_capacity_in(0, Global)
    }
}

impl<I: Id, T: Placeholder> Default for IdVec<I, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Id, T: Placeholder, A: Allocator> IdVec<I, T, A> {
    /// An empty store with room for `capacity` values, allocated from `alloc`.
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Self {
        let mut items = Vec::with_capacity_in(capacity + 1, alloc);
        items.push(T::placeholder());
        Self {
            items,
            _id: PhantomData,
        }
    }
}

impl<I: Id, T, A: Allocator> IdVec<I, T, A> {
    /// Stores `value` and returns its id.
    #[inline]
    pub fn push(&mut self, value: T) -> I {
        let id = I::from_index(self.items.len());
        self.items.push(value);
        id
    }

    /// The value `id` addresses, or `None` if no stored value has that id.
    pub fn get(&self, id: I) -> Option<&T> {
        self.items.get(id.index())
    }

    /// The value `id` addresses, or `None` if no stored value has that id.
    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.items.get_mut(id.index())
    }

    /// Removes every stored value and keeps the placeholder, so the next
    /// `push` gets id 1 again.
    pub fn clear(&mut self) {
        self.items.truncate(1);
    }

    /// The number of stored values, not counting the placeholder.
    pub fn len(&self) -> usize {
        self.items.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The stored values with their ids, without the placeholder, in the order
    /// they were pushed.
    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> {
        self.items
            .iter()
            .enumerate()
            .skip(1)
            .map(|(index, value)| (I::from_index(index), value))
    }

    /// The stored values, without the placeholder, in the order they were
    /// pushed.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.items[1..].iter()
    }
}

impl<I: Id, T, A: Allocator> Index<I> for IdVec<I, T, A> {
    type Output = T;

    #[inline]
    fn index(&self, id: I) -> &T {
        &self.items[id.index()]
    }
}

impl<I: Id, T, A: Allocator> IndexMut<I> for IdVec<I, T, A> {
    #[inline]
    fn index_mut(&mut self, id: I) -> &mut T {
        &mut self.items[id.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::SPPFNodeId;

    impl Placeholder for &str {
        fn placeholder() -> Self {
            "placeholder"
        }
    }

    #[test]
    fn ids_start_at_one_and_address_their_values() {
        let mut store: IdVec<SPPFNodeId, &str> = IdVec::new();
        let a = store.push("a");
        let b = store.push("b");
        assert_eq!(a.index(), 1);
        assert_eq!(b.index(), 2);
        assert_eq!(store[a], "a");
        assert_eq!(store[b], "b");
        assert_eq!(store.len(), 2);
        assert_eq!(
            store.iter().collect::<std::vec::Vec<_>>(),
            [(a, &"a"), (b, &"b")]
        );
        assert_eq!(store.values().collect::<std::vec::Vec<_>>(), [&"a", &"b"]);
    }

    #[test]
    fn an_optional_id_is_the_size_of_the_id() {
        assert_eq!(size_of::<Option<SPPFNodeId>>(), size_of::<SPPFNodeId>());
    }
}
