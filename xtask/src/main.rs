use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use clap::{Parser, Subcommand};
use iguana_compiler::{
    generator::{GenConfig, GenerateResult, generate_scaffold, generate_sources, generate_wasm},
    grammar::def::Grammar,
    iggy::parse_grammar,
    utils::to_pascal_case,
};

#[derive(Parser)]
#[command(name = "xtask", about = "Iguana dev commands")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Regenerate the iggy bootstrap parser from iggy/iggy.iggy
    Bootstrap,
    /// Scaffold a new grammar test under tests/<name>/
    TestNew {
        /// Name of the test
        name: String,
    },
    /// Remove a grammar test
    TestRm {
        /// Name of the test to remove
        name: String,
    },
    /// Regenerate the parser for a single test
    TestGen {
        /// Name of the test to regenerate
        name: String,
    },
    /// Regenerate all test parsers
    TestGenAll,
    /// Run the workspace test suite: the cargo tests (nextest if available)
    /// plus the grammar tests (each parser binary checked against its
    /// expected .sexpr output)
    Test {
        /// Rewrite the grammar tests' expected output instead of checking it;
        /// skips the cargo tests
        #[arg(long)]
        regen: bool,
        /// Extra arguments forwarded to nextest / cargo test
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build iguana from this workspace and install it into `$CARGO_HOME/bin`
    Install,
    /// Install iguana, then launch the terrarium dev server
    Terrarium,
    /// Generate the iggy wasm bundle and build it with wasm-pack
    Wasm,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Bootstrap => bootstrap(),
        Commands::TestNew { name } => test_new(&name),
        Commands::TestRm { name } => test_rm(&name),
        Commands::TestGen { name } => test_gen(&name),
        Commands::TestGenAll => test_gen_all(),
        Commands::Test { regen, args } => test(regen, &args),
        Commands::Install => install(),
        Commands::Terrarium => terrarium(),
        Commands::Wasm => wasm(),
    }
}

fn bootstrap() -> io::Result<()> {
    let iggy_dir = workspace_root().join("iggy");
    let grammar_file = iggy_dir.join("iggy.iggy");
    let config = GenConfig {
        cli: true,
        ..GenConfig::default()
    };
    let (result, _) = regenerate_with(&grammar_file, &iggy_dir, config, None, true)?;
    patch_iggy_cargo_toml(&iggy_dir.join("Cargo.toml"))?;
    println!("Generated iggy grammar in {} ms", result.total_duration_ms);
    Ok(())
}

/// Adapt a generated `cli=true` Cargo.toml to workspace membership: the git
/// dependency on iguana-runtime becomes a workspace dependency, and the
/// per-crate `[profile.release]` block is removed (cargo ignores profiles on
/// non-root members and warns). Returns the patched text.
///
/// Each substitution asserts it actually fired; if a `cargo_toml_gen` template
/// change breaks a pattern, the caller fails loudly instead of silently leaving
/// a broken Cargo.toml.
fn patch_workspace_cargo_toml(original: &str) -> io::Result<String> {
    let replaced = original.replace(
        "iguana-runtime = { git = \"https://github.com/iguana-parser/iguana-rs\" }",
        "iguana-runtime.workspace = true",
    );
    if replaced == original {
        return Err(io::Error::other(
            "Cargo.toml: `iguana-runtime = { git = ... }` pattern not found; \
             cargo_toml_gen template may have changed and this patch needs updating",
        ));
    }

    let without_profile = replaced.replace("\n[profile.release]\ndebug = true\n", "\n");
    if without_profile == replaced {
        return Err(io::Error::other(
            "Cargo.toml: `[profile.release]` block not found; \
             cargo_toml_gen template may have changed and this patch needs updating",
        ));
    }

    // Opt the workspace-member parser into `[workspace.lints]`, so its generated
    // sources are built with warnings denied and a generator regression that
    // reintroduces one fails `cargo xtask test`. The standalone end-user crate
    // is never patched, so its build only warns.
    Ok(format!(
        "{}\n[lints]\nworkspace = true\n",
        without_profile.trim_end()
    ))
}

/// Adapt iggy's regenerated Cargo.toml to its workspace membership, plus its
/// MIT/Apache license metadata.
fn patch_iggy_cargo_toml(path: &Path) -> io::Result<()> {
    let original = fs::read_to_string(path)?;
    let patched = patch_workspace_cargo_toml(&original)?;

    let with_license = patched.replace(
        "edition = \"2024\"\n",
        "edition = \"2024\"\nlicense = \"MIT OR Apache-2.0\"\n",
    );
    if with_license == patched {
        return Err(io::Error::other(
            "iggy Cargo.toml: `edition = \"2024\"` line not found; \
             cargo_toml_gen template may have changed and this patch needs updating",
        ));
    }

    fs::write(path, with_license)
}

/// Adapt a test grammar's regenerated Cargo.toml to workspace membership, and
/// disable the lib's empty test/doctest harnesses: a grammar is tested through
/// its golden files (run by the subprocess runner), not through unit tests.
fn patch_test_cargo_toml(path: &Path) -> io::Result<()> {
    let original = fs::read_to_string(path)?;
    let patched = patch_workspace_cargo_toml(&original)?;

    let with_lib_flags = patched.replace(
        "[lib]\npath = \"src/lib.rs\"\n",
        "[lib]\npath = \"src/lib.rs\"\ntest = false\ndoctest = false\n",
    );
    if with_lib_flags == patched {
        return Err(io::Error::other(
            "test Cargo.toml: `[lib]` block not found; \
             cargo_toml_gen template may have changed and this patch needs updating",
        ));
    }

    fs::write(path, with_lib_flags)
}

fn test_new(name: &str) -> io::Result<()> {
    let test_dir = workspace_root().join("tests").join(name);
    let grammar_name = to_pascal_case(name);
    let grammar_file = test_dir.join(format!("{name}.iggy"));

    fs::create_dir_all(&test_dir)?;
    if !grammar_file.exists() {
        fs::write(&grammar_file, format!("grammar {grammar_name}\n"))?;
        println!("Created grammar: {}", grammar_file.display());
    }

    println!();
    println!(
        "Edit {} to define your grammar, then run:",
        grammar_file.display()
    );
    println!("    cargo xtask test-gen {name}");
    Ok(())
}

fn test_rm(name: &str) -> io::Result<()> {
    let test_dir = workspace_root().join("tests").join(name);

    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
        println!("Deleted: tests/{name}/");
    } else {
        println!("Test not found: tests/{name}/");
    }

    remove_workspace_member(name)?;
    Ok(())
}

fn test_gen(name: &str) -> io::Result<()> {
    let path = workspace_root().join("tests").join(name);
    let grammar_file = path.join(format!("{name}.iggy"));
    if !grammar_file.exists() {
        return Err(io::Error::other(format!(
            "Grammar file not found: {}\nRun `cargo xtask test-new {name}` first.",
            grammar_file.display()
        )));
    }

    let (result, _) = regenerate(&grammar_file, &path)?;
    patch_test_cargo_toml(&path.join("Cargo.toml"))?;
    println!(
        "Generated {name} grammar in {} ms",
        result.total_duration_ms
    );

    add_workspace_member(name)?;
    Ok(())
}

fn add_workspace_member(name: &str) -> io::Result<()> {
    let workspace_cargo = workspace_root().join("Cargo.toml");
    let content = fs::read_to_string(&workspace_cargo)?;
    let member_entry = format!("    \"tests/{name}\",\n");
    if content.contains(&member_entry) {
        return Ok(());
    }
    let new_content = content.replace(
        "    \"xtask\",\n",
        &format!("    \"xtask\",\n{member_entry}"),
    );
    fs::write(&workspace_cargo, new_content)?;
    println!("Added tests/{name} to workspace members");
    Ok(())
}

fn remove_workspace_member(name: &str) -> io::Result<()> {
    let workspace_cargo = workspace_root().join("Cargo.toml");
    let content = fs::read_to_string(&workspace_cargo)?;
    let member_entry = format!("    \"tests/{name}\",\n");
    if content.contains(&member_entry) {
        fs::write(&workspace_cargo, content.replace(&member_entry, ""))?;
    }
    Ok(())
}

fn test_gen_all() -> io::Result<()> {
    let tests_dir = workspace_root().join("tests");
    if !tests_dir.exists() {
        println!("No tests directory found");
        return Ok(());
    }

    for entry in fs::read_dir(&tests_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_string_lossy();
            let grammar_file = path.join(format!("{name}.iggy"));
            if grammar_file.exists() {
                test_gen(&name)?;
            }
        }
    }
    Ok(())
}

fn regenerate(grammar_path: &Path, output: &Path) -> io::Result<(GenerateResult, Vec<String>)> {
    // Test crates build a CLI binary (cli=true) so the golden-file runner can
    // shell out to them. force=true keeps src/main.rs current with the
    // generator; patch_test_cargo_toml then adapts the standalone Cargo.toml
    // the scaffold emits to workspace membership.
    let config = GenConfig {
        cli: true,
        ..GenConfig::default()
    };
    regenerate_with(grammar_path, output, config, None, true)
}

fn regenerate_with(
    grammar_path: &Path,
    output: &Path,
    config: GenConfig,
    runtime_path: Option<&Path>,
    force: bool,
) -> io::Result<(GenerateResult, Vec<String>)> {
    let source = fs::read_to_string(grammar_path)?;
    let grammar_def = parse_grammar(&source).map_err(io::Error::other)?;
    let starts: Vec<String> = grammar_def
        .syntax_rules
        .iter()
        .filter(|r| r.start)
        .map(|r| r.head.name.clone())
        .collect();
    let grammar: Grammar = grammar_def
        .try_into()
        .map_err(|errors: Vec<String>| io::Error::other(errors.join("\n")))?;
    generate_scaffold(&grammar, output, config, runtime_path, None, force)?;
    let result = generate_sources(&grammar, output, config)?;
    format_sources(output)?;
    if config.wasm {
        generate_wasm(&grammar, output, runtime_path, force)?;
        format_sources(&output.join("wasm"))?;
    }
    Ok((result, starts))
}

/// Format every `.rs` file in `<crate_dir>/src/` with rustfmt. One invocation
/// for the whole crate avoids the project-mode race between parallel rustfmt
/// processes, and skipping cargo means no workspace traversal.
fn format_sources(crate_dir: &Path) -> io::Result<()> {
    let src_dir = crate_dir.join("src");
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&src_dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|x| x == "rs") {
            files.push(path);
        }
    }
    if files.is_empty() {
        return Ok(());
    }
    let status = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg("--quiet")
        .args(&files)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "rustfmt failed in {}",
            src_dir.display()
        )));
    }
    Ok(())
}

fn test(regen: bool, extra: &[String]) -> io::Result<()> {
    // --regen rewrites the grammar tests' expected output; the cargo tests have
    // nothing to regenerate, so skip them.
    if regen {
        return run_grammar_tests(true);
    }

    let root = workspace_root();
    let nextest_available = Command::new("cargo")
        .args(["nextest", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let mut cmd = Command::new("cargo");
    cmd.current_dir(root);
    if nextest_available {
        cmd.args(["nextest", "run", "--workspace"]);
    } else {
        cmd.args(["test", "--workspace"]);
    }
    cmd.args(extra);

    let status = cmd.status()?;
    if !status.success() {
        return Err(io::Error::other("test run failed"));
    }

    // The grammar tests run through each parser binary, not the cargo test
    // binary, so run them as a second step.
    run_grammar_tests(false)
}

/// Runs every grammar's tests: its input/expected-output file pairs. Builds the
/// workspace binaries, then for each `tests/<grammar>/tests/<Start>/` directory
/// runs that grammar's parser with the truthful render flags: `--check-sexpr` to
/// compare against the expected `.sexpr` output, or `--regenerate-sexpr` to
/// rewrite it. The directory name is the start nonterminal, so no per-grammar
/// configuration is needed. Runs concurrently; the run fails if any parser does.
fn run_grammar_tests(regenerate: bool) -> io::Result<()> {
    use std::io::IsTerminal;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    let sexpr_flag = if regenerate {
        "--regenerate-sexpr"
    } else {
        "--check-sexpr"
    };
    let root = workspace_root();
    let status = Command::new("cargo")
        .current_dir(root)
        .args(["build", "--workspace", "--quiet"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other("cargo build failed"));
    }

    // Collect one job per start-nonterminal directory, with its case count.
    let mut jobs: Vec<(String, String, PathBuf, usize)> = Vec::new();
    for entry in fs::read_dir(root.join("tests"))? {
        let grammar_dir = entry?.path();
        let grammar = grammar_dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let cases_root = grammar_dir.join("tests");
        if !cases_root.is_dir() {
            continue;
        }
        for start_entry in fs::read_dir(&cases_root)? {
            let start_dir = start_entry?.path();
            if !start_dir.is_dir() {
                continue;
            }
            let start = start_dir.file_name().unwrap().to_string_lossy().to_string();
            let cases = fs::read_dir(&start_dir)?
                .filter_map(Result::ok)
                .filter(|e| e.path().extension().is_some_and(|x| x == "txt"))
                .count();
            jobs.push((grammar.clone(), start, start_dir, cases));
        }
    }
    jobs.sort();

    let total = jobs.len();
    let idx_width = total.to_string().len();
    let label_width = jobs
        .iter()
        .map(|(g, s, _, _)| g.len() + 1 + s.len())
        .max()
        .unwrap_or(0);
    let (green, red, reset) = if io::stdout().is_terminal() {
        ("\x1b[32m", "\x1b[31m", "\x1b[0m")
    } else {
        ("", "", "")
    };

    let bin_dir = root.join("target/debug");
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(String, bool, String)>> = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let start_time = Instant::now();

    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some((grammar, start, start_dir, cases)) = jobs.get(i) else {
                        break;
                    };
                    let bin = bin_dir.join(grammar);
                    let output = Command::new(&bin)
                        .args(["--dir", start_dir.to_str().unwrap()])
                        .args(["--ext", "txt", "-n", start, sexpr_flag])
                        .args(["--show-layout", "--show-empty", "--show-wrappers"])
                        .output();
                    let (ok, text) = match output {
                        Ok(o) => (
                            o.status.success(),
                            format!(
                                "{}{}",
                                String::from_utf8_lossy(&o.stdout),
                                String::from_utf8_lossy(&o.stderr)
                            ),
                        ),
                        Err(e) => (false, format!("failed to run {}: {e}", bin.display())),
                    };
                    let label = format!("{grammar}/{start}");
                    let (word, code) = if ok { ("PASS", green) } else { ("FAIL", red) };
                    // Print and record under one lock so lines never interleave
                    // and the counter follows completion order.
                    let mut done = results.lock().unwrap();
                    let n = done.len() + 1;
                    println!(
                        "  {code}{word}{reset}  ({n:>idx_width$}/{total})  {label:<label_width$}  {cases:>2} cases"
                    );
                    done.push((label, ok, text));
                }
            });
        }
    });

    let elapsed = start_time.elapsed().as_secs_f64();
    let mut results = results.into_inner().unwrap();
    results.sort();
    let failures: Vec<&(String, bool, String)> = results.iter().filter(|(_, ok, _)| !ok).collect();
    if !failures.is_empty() {
        println!();
        for (label, _, text) in &failures {
            println!("{red}--- {label} ---{reset}");
            for line in text.lines() {
                println!("  {line}");
            }
        }
    }
    let passed = results.len() - failures.len();
    let verb = if regenerate { "regenerated" } else { "passed" };
    let code = if failures.is_empty() { green } else { red };
    println!();
    println!("{code}Grammar tests: {passed}/{total} {verb} in {elapsed:.2}s{reset}");
    if !failures.is_empty() {
        return Err(io::Error::other(format!(
            "{} grammar test(s) failed",
            failures.len()
        )));
    }
    Ok(())
}

/// Rebuild the web viewer bundle (web-viewer/dist) with vite. The iguana binary
/// embeds that directory via include_dir!, so rebuilding it from source here
/// keeps `iguana try` in sync with the viewer. This needs npm on PATH and the
/// viewer dependencies installed (`npm install` in the repo root); if either is
/// missing the build errors rather than falling back to a stale bundle.
fn viewer() -> io::Result<()> {
    println!("Building the web viewer...");
    let status = Command::new("npm")
        .current_dir(workspace_root())
        .args(["run", "build", "--workspace", "web-viewer"])
        .status()
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => io::Error::other(
                "npm not found. Install Node.js (which provides npm), \
                 then run `npm install` in the repo root.",
            ),
            _ => e,
        })?;
    if !status.success() {
        return Err(io::Error::other(
            "npm run build failed; run `npm install` in the repo root if the \
             viewer dependencies are missing.",
        ));
    }
    Ok(())
}

fn install() -> io::Result<()> {
    viewer()?;
    let root = workspace_root();
    println!("Building iguana (release)...");
    let status = Command::new("cargo")
        .current_dir(root)
        .args(["build", "--release", "-p", "iguana"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other("cargo build failed"));
    }

    let built = root.join("target/release/iguana");
    let dest_dir = cargo_bin_dir();
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join("iguana");
    fs::copy(&built, &dest)?;
    println!("Installed: {}", dest.display());
    Ok(())
}

fn terrarium() -> io::Result<()> {
    install()?;

    let terrarium_dir = workspace_root().join("terrarium");
    if !terrarium_dir.join("node_modules").exists() {
        println!("Installing npm dependencies...");
        let status = Command::new("npm")
            .current_dir(&terrarium_dir)
            .arg("install")
            .status()?;
        if !status.success() {
            return Err(io::Error::other("npm install failed"));
        }
    }

    println!("Launching terrarium dev server...");
    let status = Command::new("npm")
        .current_dir(&terrarium_dir)
        .args(["run", "tauri", "dev"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other("npm run tauri dev failed"));
    }
    Ok(())
}

/// Generate the iggy wasm bundle into `target/wasm/iggy` and build it with
/// wasm-pack.
fn wasm() -> io::Result<()> {
    let root = workspace_root();
    let grammar_file = root.join("iggy").join("iggy.iggy");
    let output = root.join("target").join("wasm").join("iggy");

    let config = GenConfig {
        wasm: true,
        cli: false,
        ..GenConfig::default()
    };
    // Build the bundle against the local runtime checkout, not the git dep.
    let runtime = root.join("iguana-runtime");
    let (result, _) = regenerate_with(
        &grammar_file,
        &output,
        config,
        Some(runtime.as_path()),
        true,
    )?;
    println!(
        "Generated iggy wasm bundle in {} ms",
        result.total_duration_ms
    );

    let wasm_dir = output.join("wasm");
    iguana_compiler::wasm_build::build(&wasm_dir)?;
    println!("Wasm package ready at {}", wasm_dir.join("pkg").display());
    Ok(())
}

fn cargo_bin_dir() -> PathBuf {
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        return PathBuf::from(cargo_home).join("bin");
    }
    let home = std::env::var("HOME").expect("HOME must be set");
    PathBuf::from(home).join(".cargo").join("bin")
}

fn workspace_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask must live one level below the workspace root")
            .to_path_buf()
    })
}
