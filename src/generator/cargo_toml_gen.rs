use crate::{generator::utils::to_first_lowercase, grammar::def::Grammar};

pub fn generate(grammar: &Grammar) -> String {
    let grammar_name = &grammar.name;
    format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[profile.release]
debug = true

[lib]
path = "src/lib.rs"

[dependencies]
iguana = {{ path = "/Users/afroozeh/Workspace/iguana-rs" }}
dot = {{ git = "https://github.com/przygienda/dot-rust.git", rev = "fed06f613a9d72bfde711a12791f96a777b2371e" }}
log = "0.4.28"
rustc-hash = "2.1.1"
dhat = "0.3"
clap = {{ version = "4", features = ["derive"] }}
serde_json = "1.0"
phf = {{ version = "0.11", features = ["macros"] }}

[features]
dhat-heap = []
debug-trace = ["iguana/debug-trace"]
    "#,
        to_first_lowercase(grammar_name)
    )
    .trim().to_owned()
}
