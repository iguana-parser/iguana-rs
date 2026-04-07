//! Lightweight instrumentation for measuring the size distribution of inline
//! collections (`InlineSet`, `InlineVec`, `InlineMap`, ...) at drop time.
//!
//! All instrumentation is gated behind the `instrument` Cargo feature. When the
//! feature is off, `record` and `dump` are no-op stubs and there is zero
//! runtime cost.
//!
//! Usage:
//! - Types under instrumentation implement `Drop` (gated on the feature) and
//!   call `record(name, self.len())`.
//! - Generated parsers call `dump()` after parsing to print histograms.

#[cfg(feature = "instrument")]
mod imp {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    thread_local! {
        static STATS: RefCell<BTreeMap<&'static str, Vec<usize>>> =
            RefCell::new(BTreeMap::new());
    }

    pub fn record(name: &'static str, len: usize) {
        STATS.with(|s| s.borrow_mut().entry(name).or_default().push(len));
    }

    pub fn reset() {
        STATS.with(|s| s.borrow_mut().clear());
    }

    pub fn dump() {
        STATS.with(|s| {
            let stats = s.borrow();
            if stats.is_empty() {
                eprintln!("[instrument] no samples recorded");
                return;
            }
            for (name, lens) in stats.iter() {
                let n = lens.len();
                let max = lens.iter().copied().max().unwrap_or(0);
                let sum: usize = lens.iter().sum();
                let avg = sum as f64 / n.max(1) as f64;
                let mut hist = [0usize; 8];
                for &l in lens {
                    let b = match l {
                        0 => 0,
                        1 => 1,
                        2 => 2,
                        3..=4 => 3,
                        5..=8 => 4,
                        9..=16 => 5,
                        17..=32 => 6,
                        _ => 7,
                    };
                    hist[b] += 1;
                }
                eprintln!("[instrument] {name}: n={n} max={max} avg={avg:.2}");
                eprintln!(
                    "  size  0:{} 1:{} 2:{} 3-4:{} 5-8:{} 9-16:{} 17-32:{} 33+:{}",
                    hist[0], hist[1], hist[2], hist[3], hist[4], hist[5], hist[6], hist[7]
                );
            }
        });
    }
}

#[cfg(not(feature = "instrument"))]
mod imp {
    #[inline(always)]
    pub fn record(_name: &'static str, _len: usize) {}
    #[inline(always)]
    pub fn reset() {}
    #[inline(always)]
    pub fn dump() {}
}

pub use imp::{dump, record, reset};
