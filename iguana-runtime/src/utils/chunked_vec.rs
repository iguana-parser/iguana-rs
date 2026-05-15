use std::mem::MaybeUninit;

/// An append-only sequence stored as a list of fixed-size heap chunks.
///
/// Growth never copies existing elements: when the top chunk fills, a new chunk
/// is allocated and linked in. Chunks remain allocated after pops, so subsequent
/// pushes reuse them without going back to the allocator.
pub struct ChunkedVec<T, const N: usize> {
    chunks: Vec<Box<[MaybeUninit<T>; N]>>,
    /// Index of the chunk the cursor sits in.
    current_chunk: usize,
    /// Next-write slot within the current chunk, in `0..=N`.
    offset: usize,
}

impl<T, const N: usize> ChunkedVec<T, N> {
    pub const fn new() -> Self {
        Self {
            chunks: Vec::new(),
            current_chunk: 0,
            offset: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.offset == N {
            self.current_chunk += 1;
            self.offset = 0;
        }
        if self.current_chunk == self.chunks.len() {
            self.chunks
                .push(Box::new([const { MaybeUninit::<T>::uninit() }; N]));
        }
        self.chunks[self.current_chunk][self.offset].write(value);
        self.offset += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.offset == 0 {
            if self.current_chunk == 0 {
                return None;
            }
            self.current_chunk -= 1;
            self.offset = N;
        }
        self.offset -= 1;
        // SAFETY: `offset` was previously written via `push`.
        Some(unsafe { self.chunks[self.current_chunk][self.offset].assume_init_read() })
    }

    pub fn len(&self) -> usize {
        self.current_chunk * N + self.offset
    }

    pub fn is_empty(&self) -> bool {
        self.current_chunk == 0 && self.offset == 0
    }
}

impl<T, const N: usize> Default for ChunkedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for ChunkedVec<T, N> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() {
            while self.pop().is_some() {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkedVec;

    #[test]
    fn empty() {
        let s: ChunkedVec<i32, 4> = ChunkedVec::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn push_pop_single_chunk() {
        let mut s: ChunkedVec<i32, 4> = ChunkedVec::new();
        s.push(1);
        s.push(2);
        s.push(3);
        assert_eq!(s.len(), 3);
        assert_eq!(s.pop(), Some(3));
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
        assert!(s.is_empty());
    }

    #[test]
    fn crosses_chunk_boundary() {
        let mut s: ChunkedVec<i32, 4> = ChunkedVec::new();
        for i in 0..10 {
            s.push(i);
        }
        assert_eq!(s.len(), 10);
        for i in (0..10).rev() {
            assert_eq!(s.pop(), Some(i));
        }
        assert!(s.is_empty());
    }

    #[test]
    fn keeps_chunks_after_pop() {
        let mut s: ChunkedVec<i32, 4> = ChunkedVec::new();
        for i in 0..12 {
            s.push(i);
        }
        let chunks_before = s.chunks.len();
        for _ in 0..12 {
            s.pop();
        }
        assert_eq!(s.chunks.len(), chunks_before);
        // Reuse without further allocation.
        for i in 0..12 {
            s.push(i);
        }
        assert_eq!(s.chunks.len(), chunks_before);
    }

    #[test]
    fn refills_after_partial_pop() {
        let mut s: ChunkedVec<i32, 4> = ChunkedVec::new();
        for i in 0..10 {
            s.push(i);
        }
        for _ in 0..6 {
            s.pop();
        }
        // 4 items left: chunk 0 full, chunk 1 has 0.
        assert_eq!(s.len(), 4);
        for i in 100..107 {
            s.push(i);
        }
        assert_eq!(s.len(), 11);
        for i in (100..107).rev() {
            assert_eq!(s.pop(), Some(i));
        }
        for i in (0..4).rev() {
            assert_eq!(s.pop(), Some(i));
        }
        assert!(s.is_empty());
    }

    #[test]
    fn drops_live_elements() {
        use std::rc::Rc;
        let r = Rc::new(());
        {
            let mut s: ChunkedVec<Rc<()>, 4> = ChunkedVec::new();
            for _ in 0..10 {
                s.push(r.clone());
            }
            assert_eq!(Rc::strong_count(&r), 11);
        }
        assert_eq!(Rc::strong_count(&r), 1);
    }
}
