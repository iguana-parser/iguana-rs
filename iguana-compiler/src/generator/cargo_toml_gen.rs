use std::path::Path;

use crate::{generator::GenConfig, grammar::def::Grammar, utils::to_snake_case};

/// The default `iguana-runtime` dependency line for a generated Cargo.toml:
/// the crates.io release matching the generator's own version. A published
/// version is immutable, so a parser keeps compiling against the runtime it
/// was generated for, offline and vendored builds work, and a change to an
/// unreleased runtime cannot reach a generated parser. Public because
/// xtask's workspace patching replaces this exact line.
pub fn pinned_runtime_dependency() -> String {
    format!("iguana-runtime = \"={}\"", env!("CARGO_PKG_VERSION"))
}

/// The `iguana-runtime` dependency line for a generated Cargo.toml: a local
/// path when `runtime_path` is set, otherwise the pinned crates.io release.
fn runtime_dependency(runtime_path: Option<&Path>) -> String {
    match runtime_path {
        Some(path) => format!("iguana-runtime = {{ path = \"{}\" }}", path.display()),
        None => pinned_runtime_dependency(),
    }
}

/// The `[features]` section for a shape that holds generated parser sources.
///
/// Generated code gates tracing and instrumentation on two features, so every
/// such shape declares them. `extra` holds the features a shape adds.
fn generated_source_features(extra: &[&str]) -> String {
    let mut section = String::from(
        "[features]\n\
         debug-trace = [\"iguana-runtime/debug-trace\"]\n\
         instrument = [\"iguana-runtime/instrument\"]",
    );
    for line in extra {
        section.push('\n');
        section.push_str(line);
    }
    section
}

/// Generate the contents of `Cargo.toml` for a parser crate.
///
/// `wasm` produces a standalone lib shape with only the wasm-safe deps
/// (`iguana-runtime`, `rustc-hash`, `serde_json`) and no `[[bin]]`, so the
/// crate compiles for `wasm32` as a dependency of the wrapper crate.
///
/// `cli` (without `wasm`) produces a full standalone-parser shape with CLI deps
/// and a `src/main.rs` binary.
///
/// With neither flag, the output is a minimal lib-only shape that assumes the
/// crate is a workspace member: deps come from `workspace = true`, the `[lib]`
/// target disables its empty test/doctest harnesses, and there is no `[[bin]]`.
///
/// A shape returns its manifest up to the feature section and the features it
/// adds. `generate` appends the section, so no shape can omit it.
pub fn generate(
    grammar: &Grammar,
    config: GenConfig,
    runtime_path: Option<&Path>,
    bin_name: Option<&str>,
) -> String {
    let name = to_snake_case(&grammar.name);
    let (body, extra_features) = if config.wasm {
        generate_wasm_lib(&name, runtime_path)
    } else if config.cli {
        generate_full(&name, runtime_path, bin_name)
    } else {
        generate_minimal(&name)
    };
    format!("{body}\n\n{}", generated_source_features(extra_features))
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

/// The wasm lib shape, which adds no features.
fn generate_wasm_lib(name: &str, runtime_path: Option<&Path>) -> (String, &'static [&'static str]) {
    let runtime = runtime_dependency(runtime_path);
    let body = format!(
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
    );
    (body.trim().to_owned(), &[])
}

/// The standalone CLI shape. Its dhat and pprof dependencies are optional and
/// named with `dep:`, so a build compiles them only under `dhat-heap` or
/// `profile`.
fn generate_full(
    name: &str,
    runtime_path: Option<&Path>,
    bin_name: Option<&str>,
) -> (String, &'static [&'static str]) {
    let runtime = runtime_dependency(runtime_path);
    // The [[bin]] is always explicit. It lets bin_name decouple the binary
    // name from the crate name (a grammar like Java avoids a binary called
    // "java" that shadows the JDK), and test = false keeps cargo test and
    // nextest from building and listing an empty test harness for the binary.
    let bin = bin_name.unwrap_or(name);
    let bin_section =
        format!("[[bin]]\nname = \"{bin}\"\npath = \"src/main.rs\"\ntest = false\n\n");
    let body = format!(
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
dhat = {{ version = "0.3", optional = true }}
pprof = {{ version = "0.15", features = ["flamegraph"], optional = true }}
    "#
    );
    (
        body.trim().to_owned(),
        &["dhat-heap = [\"dep:dhat\"]", "profile = [\"dep:pprof\"]"],
    )
}

/// The workspace-member shape, which adds no features.
fn generate_minimal(name: &str) -> (String, &'static [&'static str]) {
    let body = format!(
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
    "#
    );
    (body.trim().to_owned(), &[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iggy::parse_grammar;

    /// A generated standalone crate depends on the published runtime at the
    /// generator's own version. Nothing else in the workspace resolves that
    /// manifest, so this test is what catches a template regression.
    #[test]
    fn standalone_crate_pins_the_matching_runtime_release() {
        let expected = format!("iguana-runtime = \"={}\"", env!("CARGO_PKG_VERSION"));
        assert_eq!(pinned_runtime_dependency(), expected);
        assert!(manifest(GenConfig::default()).contains(&expected));
        assert!(
            manifest(GenConfig {
                wasm: true,
                cli: false,
                ..GenConfig::default()
            })
            .contains(&expected)
        );
        assert!(generate_wasm_wrapper(&demo_grammar(), None).contains(&expected));
    }

    fn demo_grammar() -> Grammar {
        parse_grammar("grammar Demo\n\nS = \"x\"\n")
            .expect("grammar should parse")
            .try_into()
            .expect("grammar should build")
    }

    fn manifest(config: GenConfig) -> String {
        generate(&demo_grammar(), config, None, None)
    }

    /// A shape added later takes its feature section from `generate`, so the
    /// two features generated code gates on cannot go missing.
    #[test]
    fn every_shape_that_holds_generated_sources_declares_their_features() {
        let common = generated_source_features(&[]);
        for config in [
            GenConfig {
                wasm: true,
                cli: false,
                ..GenConfig::default()
            },
            GenConfig::default(),
            GenConfig {
                cli: false,
                ..GenConfig::default()
            },
        ] {
            let manifest = manifest(config);
            assert!(manifest.contains(&common), "{manifest}");
        }
    }

    /// The CLI shape is the only one with profiling dependencies. `dep:` keeps
    /// them from becoming features that compile a dependency without switching
    /// anything on.
    #[test]
    fn the_cli_shape_adds_only_its_profiling_features() {
        let full = manifest(GenConfig::default());
        let expected =
            generated_source_features(&["dhat-heap = [\"dep:dhat\"]", "profile = [\"dep:pprof\"]"]);
        assert!(full.contains(&expected), "{full}");
        assert!(!full.contains("= [\"dhat\"]"), "{full}");
        assert!(!full.contains("= [\"pprof\"]"), "{full}");
    }
}
