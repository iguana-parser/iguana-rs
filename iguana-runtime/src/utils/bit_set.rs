/// A set of integers, stored as `W` words of 64 bits.
/// Integer `i` is in word `i / 64`, at bit `i % 64`.
/// It can hold the integers in `[0, 64 * W)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitSet<const W: usize>([u64; W]);

impl<const W: usize> Default for BitSet<W> {
    fn default() -> Self {
        Self([0; W])
    }
}

impl<const W: usize> BitSet<W> {
    pub fn contains(&self, i: usize) -> bool {
        self.0[i / 64] & (1 << (i % 64)) != 0
    }

    pub fn insert(&mut self, i: usize) {
        self.0[i / 64] |= 1 << (i % 64);
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&word| word == 0)
    }

    /// The members from the largest to the smallest.
    pub fn iter_descending(&self) -> impl Iterator<Item = usize> {
        let words = self.0;
        (0..W).rev().flat_map(move |index| {
            let mut remaining = words[index];
            std::iter::from_fn(move || {
                if remaining == 0 {
                    return None;
                }
                let bit = u64::BITS - 1 - remaining.leading_zeros();
                remaining &= !(1 << bit);
                Some(index * 64 + bit as usize)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_iterate_from_the_largest_across_words() {
        let mut set = BitSet::<2>::default();
        assert!(set.is_empty());
        for i in [0, 3, 64, 127] {
            set.insert(i);
        }
        assert!(set.contains(64) && !set.contains(65));
        assert_eq!(set.iter_descending().collect::<Vec<_>>(), [127, 64, 3, 0]);
    }
}
