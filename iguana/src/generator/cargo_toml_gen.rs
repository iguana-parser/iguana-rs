use crate::{generator::utils::to_snake_case, grammar::def::Grammar};

pub fn generate(grammar: &Grammar) -> String {
    let grammar_name = &grammar.name;
    format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"

[dependencies]
iguana-runtime = {{ path = "/Users/afroozeh/Workspace/iguana-rs/iguana-runtime" }}
clap = {{ version = "4.5.51", features = ["derive"] }}
dot = {{ git = "https://github.com/przygienda/dot-rust.git", rev = "fed06f613a9d72bfde711a12791f96a777b2371e" }}
log = "0.4"
rustc-hash = "2.1.1"
serde_json = "1.0"
dhat = "0.3"
phf = {{ version = "0.11", features = ["macros"] }}
pprof = {{ version = "0.14", features = ["flamegraph"], optional = true }}

[lints]
workspace = true

[features]
dhat-heap = []
debug-trace = ["iguana-runtime/debug-trace"]
profile = ["pprof"]
    "#,
        to_snake_case(grammar_name)
    )
    .trim().to_owned()
}
