use crate::{grammar::def::Grammar, utils::to_snake_case};

/// Generate the contents of `Cargo.toml` for a parser crate.
///
/// `cli=true` produces a full standalone-parser shape with CLI deps
/// (clap/dot/dhat/pprof) and a `src/main.rs` binary.
///
/// `cli=false` produces a minimal lib-only shape that assumes the crate is
/// a workspace member: deps come from `workspace = true`, the `[lib]` target
/// disables its empty test/doctest harnesses, and there is no `[[bin]]`.
pub fn generate(grammar: &Grammar, cli: bool) -> String {
    let name = to_snake_case(&grammar.name);
    if cli {
        generate_full(&name)
    } else {
        generate_minimal(&name)
    }
}

fn generate_full(name: &str) -> String {
    format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"

[dependencies]
iguana-runtime = {{ git = "https://github.com/iguana-parser/iguana-rs" }}
clap = {{ version = "4.5.51", features = ["derive"] }}
dot = {{ git = "https://github.com/przygienda/dot-rust.git", rev = "fed06f613a9d72bfde711a12791f96a777b2371e" }}
log = "0.4"
rustc-hash = "2.1.1"
serde_json = "1.0"
dhat = "0.3"
pprof = {{ version = "0.14", features = ["flamegraph"], optional = true }}

[features]
dhat-heap = []
debug-trace = ["iguana-runtime/debug-trace"]
profile = ["pprof"]
instrument = ["iguana-runtime/instrument"]

[profile.release]
debug = true
    "#
    )
    .trim()
    .to_owned()
}

fn generate_minimal(name: &str) -> String {
    format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"
test = false
doctest = false

[dependencies]
iguana-runtime.workspace = true
rustc-hash.workspace = true
serde_json.workspace = true

[features]
debug-trace = ["iguana-runtime/debug-trace"]
instrument = ["iguana-runtime/instrument"]
    "#
    )
    .trim()
    .to_owned()
}
