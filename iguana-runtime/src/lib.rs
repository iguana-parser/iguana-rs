pub mod arena;
pub mod cli;
pub mod descriptor;
pub mod dfa;
pub mod env;
pub mod gss;
pub mod ids;
pub mod input;
pub mod instrument;
pub mod parse_tree;
pub mod parser;
pub mod result;
pub mod scanner;
pub mod sppf;
pub mod trace;
pub mod utils;
pub mod visualization;

// Re-export the record macro from trace module
pub use trace::*;

// web-time's Instant reads the browser Performance clock on wasm32, where
// std::time::Instant::now() compiles but panics. On native targets it is
// std::time::Instant.
pub use web_time::Instant;
