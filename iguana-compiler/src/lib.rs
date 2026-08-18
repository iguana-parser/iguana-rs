// Generator-specific modules
pub mod comments;
pub mod dfa;
pub mod generator;
pub mod grammar;
pub mod iggy;
pub mod spans;
pub mod utils;
pub mod validation;
pub mod wasm_build;

// Re-export runtime modules for convenience
pub use iguana_runtime::{
    cli, descriptor, gss, ids, input, parse_tree, parser, record, scanner, sppf, trace,
    visualization,
};
