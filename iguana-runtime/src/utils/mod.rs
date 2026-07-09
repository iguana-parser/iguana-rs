pub mod chunked_vec;
pub mod inline_map;
pub mod inline_vec;

/// The allocator-aware `Vec`, re-exported so generated crates can name it
/// without a direct dependency on `allocator-api2`. Backs the parser's big
/// accumulators when placed in the arena: `AVec<T, &'arena Bump>`.
pub use allocator_api2::vec::Vec as AVec;
