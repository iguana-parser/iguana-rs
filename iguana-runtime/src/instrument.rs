//! Lightweight instrumentation for parser diagnostics.
//!
//! All instrumentation is gated behind the `instrument` Cargo feature. When
//! the feature is off, the `Stats` type does not exist and none of the
//! related code paths (counter increments, walker, etc.) are compiled.
//!
//! Usage from a generated parser:
//! ```ignore
//! #[cfg(feature = "instrument")]
//! let stats = parser.record_stats();
//! #[cfg(feature = "instrument")]
//! eprintln!("{stats}");
//! ```

#[cfg(feature = "instrument")]
pub use imp::Stats;

#[cfg(feature = "instrument")]
mod imp {
    use std::collections::BTreeMap;
    use std::fmt;

    use serde::Serialize;

    #[derive(Debug, Default, Clone, Serialize)]
    pub struct Stats {
        pub descriptors_count: usize,
        pub gss_nodes_count: usize,
        pub gss_edges_count: usize,
        pub nonterminal_nodes_count: usize,
        pub intermediate_nodes_count: usize,
        pub terminal_nodes_count: usize,
        pub ambiguous_nodes_count: usize,

        pub histograms: BTreeMap<&'static str, Vec<usize>>,
    }

    impl Stats {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn count_all_sppf_nodes(&self) -> usize {
            self.nonterminal_nodes_count
                + self.intermediate_nodes_count
                + self.terminal_nodes_count
        }

        pub fn record(&mut self, name: &'static str, len: usize) {
            self.histograms.entry(name).or_default().push(len);
        }
    }

    impl fmt::Display for Stats {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "[stats] counters:")?;
            writeln!(f, "  descriptors:         {}", self.descriptors_count)?;
            writeln!(f, "  gss_nodes:           {}", self.gss_nodes_count)?;
            writeln!(f, "  gss_edges:           {}", self.gss_edges_count)?;
            writeln!(f, "  nonterminal_nodes:   {}", self.nonterminal_nodes_count)?;
            writeln!(f, "  intermediate_nodes:  {}", self.intermediate_nodes_count)?;
            writeln!(f, "  terminal_nodes:      {}", self.terminal_nodes_count)?;
            writeln!(f, "  ambiguous_nodes:     {}", self.ambiguous_nodes_count)?;
            writeln!(f, "  sppf_nodes (total):  {}", self.count_all_sppf_nodes())?;

            if !self.histograms.is_empty() {
                writeln!(f)?;
                writeln!(f, "[stats] size histograms:")?;
                const BUCKETS: [&str; 8] = ["0", "1", "2", "3-4", "5-8", "9-16", "17-32", "33+"];
                const BAR_WIDTH: usize = 40;
                for (name, lens) in self.histograms.iter() {
                    let n = lens.len();
                    let max_val = lens.iter().copied().max().unwrap_or(0);
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
                    let bucket_max = hist.iter().copied().max().unwrap_or(1).max(1);
                    writeln!(f)?;
                    writeln!(f, "  {name}")?;
                    writeln!(f, "    n={n}  max={max_val}  avg={avg:.2}")?;
                    for (label, &count) in BUCKETS.iter().zip(hist.iter()) {
                        let bar_len = (count * BAR_WIDTH) / bucket_max;
                        let bar: String = "█".repeat(bar_len);
                        writeln!(f, "    {label:>5} | {bar:<width$} {count}", width = BAR_WIDTH)?;
                    }
                }
            }
            Ok(())
        }
    }
}
