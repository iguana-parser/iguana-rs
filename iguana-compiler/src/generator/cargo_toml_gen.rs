use std::path::Path;

use crate::{generator::GenConfig, grammar::def::Grammar, utils::to_snake_case};

/// The default `iguana-runtime` dependency line for a generated Cargo.toml:
/// a git dependency pinned to the generator's own version, so a parser and
/// its runtime move in lockstep and a generated parser never builds against
/// a runtime it was not generated for. Public because xtask's workspace
/// patching replaces this exact line.
pub fn git_runtime_dependency() -> String {
    format!(
        "iguana-runtime = {{ git = \"https://github.com/iguana-parser/iguana-rs\", version = \"={}\" }}",
        env!("CARGO_PKG_VERSION")
    )
}

/// The `iguana-runtime` dependency line for a generated Cargo.toml: a local
/// path when `runtime_path` is set, otherwise the pinned git dependency.
fn runtime_dependency(runtime_path: Option<&Path>) -> String {
    match runtime_path {
        Some(path) => format!("iguana-runtime = {{ path = \"{}\" }}", path.display()),
        None => git_runtime_dependency(),
    }
}

/// Generate the contents of `Cargo.toml` for a parser crate.
///
/// `wasm` produces a standalone lib shape with only the wasm-safe deps
/// (`iguana-runtime`, `rustc-hash`, `serde_json`) and no `[[bin]]`, so the
/// crate compiles for `wasm32` as a dependency of the wrapper crate.
///
/// `cli` (without `wasm`) produces a full standalone-parser shape with CLI deps
/// (clap/dhat/pprof) and a `src/main.rs` binary.
///
/// With neither flag, the output is a minimal lib-only shape that assumes the
/// crate is a workspace member: deps come from `workspace = true`, the `[lib]`
/// target disables its empty test/doctest harnesses, and there is no `[[bin]]`.
pub fn generate(
    grammar: &Grammar,
    config: GenConfig,
    runtime_path: Option<&Path>,
    bin_name: Option<&str>,
) -> String {
    let name = to_snake_case(&grammar.name);
    if config.wasm {
        generate_wasm_lib(&name, runtime_path)
    } else if config.cli {
        generate_full(&name, runtime_path, bin_name)
    } else {
        generate_minimal(&name)
    }
}

/// Generate the `Cargo.toml` for the `wasm-bindgen` wrapper crate that lives at
/// `wasm/`. It is a `cdylib` depending on the parser crate by path, plus the
/// runtime and the serialization deps the wrapper itself uses.
pub fn generate_wasm_wrapper(grammar: &Grammar, runtime_path: Option<&Path>) -> String {
    let name = to_snake_case(&grammar.name);
    let runtime = runtime_dependency(runtime_path);
    format!(
        r#"
# A self-contained workspace, so the bundle builds when dropped into a repo
# that is itself a cargo workspace.
[workspace]

[package]
name = "{name}-wasm"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde_json = "1.0"
{runtime}
{name} = {{ path = ".." }}
    "#
    )
    .trim()
    .to_owned()
}

fn generate_wasm_lib(name: &str, runtime_path: Option<&Path>) -> String {
    let runtime = runtime_dependency(runtime_path);
    format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"

[dependencies]
{runtime}
rustc-hash = "2.1.1"
serde_json = "1.0"
    "#
    )
    .trim()
    .to_owned()
}

fn generate_full(name: &str, runtime_path: Option<&Path>, bin_name: Option<&str>) -> String {
    let runtime = runtime_dependency(runtime_path);
    // The [[bin]] is always explicit. It lets bin_name decouple the binary
    // name from the crate name (a grammar like Java avoids a binary called
    // "java" that shadows the JDK), and test = false keeps cargo test and
    // nextest from building and listing an empty test harness for the binary.
    let bin = bin_name.unwrap_or(name);
    let bin_section =
        format!("[[bin]]\nname = \"{bin}\"\npath = \"src/main.rs\"\ntest = false\n\n");
    format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"

{bin_section}[dependencies]
{runtime}
clap = {{ version = "4.5.51", features = ["derive"] }}
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
