use std::{
    io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use iguana_compiler::{
    generator::{GenConfig, GenConfigFile, generate_scaffold, generate_sources, generate_wasm},
    grammar::def::{Grammar, Phase},
    iggy::parse_grammar,
    utils::to_pascal_case,
    validation::render_errors,
};

mod viewer;

#[derive(Parser)]
#[command(name = "iguana", version, about = "A practical GLL parser generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new iggy grammar project
    ///
    /// Creates the directory and writes a starter .iggy grammar and a .gitignore
    New {
        /// Path of the new project directory
        ///
        /// Must not already exist
        path: PathBuf,
    },
    /// Generate a parser crate from an iggy grammar
    ///
    /// Generation knobs (ll1, match_memo, unsafe, bin_name, runtime_path) can
    /// be persisted in a gen.toml beside the grammar.
    /// Precedence is the built-in default, then gen.toml, then an explicit
    /// flag.
    Generate {
        /// Path to an iggy grammar file
        ///
        /// Defaults to the .iggy file in the output directory
        #[arg(short, long)]
        grammar: Option<PathBuf>,

        /// Output directory for generated parser (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Output timing information as JSON (for tool integration)
        #[arg(long)]
        json: bool,

        /// Enable LL(1) optimization in code generation
        ///
        /// True by default.
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        ll1: Option<bool>,

        /// Memoize scanner results (match_token and match_any) during parsing
        ///
        /// True by default.
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        match_memo: Option<bool>,

        /// When true, the generated parser runs in the unsafe mode (see Parser::UNSAFE
        /// in iguana-runtime).
        #[arg(long = "unsafe", value_name = "UNSAFE", num_args = 0..=1, default_missing_value = "true")]
        unsafe_mode: Option<bool>,

        /// Scaffold a CLI binary (Cargo.toml + src/main.rs)
        ///
        /// When false, the caller owns Cargo.toml and no CLI binary is emitted
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        cli: bool,

        /// Emit a wasm bundle into webview/
        ///
        /// A lib crate, a wasm-bindgen wrapper crate under wasm/, a
        /// manifest.json, and the web viewer. Serve it with iguana try. The
        /// bundle has no CLI and a fixed location, so --wasm conflicts with
        /// --cli and -o
        #[arg(long, conflicts_with_all = ["cli", "output"])]
        wasm: bool,

        /// Point the generated crate's iguana-runtime dependency at a local path
        ///
        /// Replaces the default git dependency. Applies to the standalone
        /// (--cli) and wasm shapes; use it to develop the runtime alongside a
        /// grammar or pin the bundle to a specific local checkout
        #[arg(long)]
        runtime_path: Option<PathBuf>,

        /// Name the parser binary
        ///
        /// Defaults to the crate name (the grammar name in snake_case). Use it
        /// when that name would clash with an existing command, like a Java
        /// grammar whose binary would otherwise be "java"
        #[arg(long, value_name = "NAME", conflicts_with = "wasm")]
        bin_name: Option<String>,

        /// Overwrite an existing Cargo.toml with the scaffold template
        ///
        /// Without it, an existing Cargo.toml is preserved so local edits
        /// (dependencies, license, versions) survive regeneration
        #[arg(long)]
        force: bool,

        /// Print the grammar after the given pipeline phases (comma-separated) to stderr
        ///
        /// Keeps generating afterward. Pass the flag with no value to list the
        /// available phases
        #[arg(long, value_delimiter = ',', num_args = 0..)]
        print_phase: Option<Vec<String>>,
    },
    /// Serve the wasm bundle in webview/ over HTTP
    ///
    /// Build the bundle first with iguana generate --wasm
    Try {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8000)]
        port: u16,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { path } => new_project(&path)?,
        Commands::Generate {
            grammar,
            output,
            json,
            ll1,
            match_memo,
            unsafe_mode,
            cli,
            wasm,
            runtime_path,
            bin_name,
            force,
            print_phase,
        } => {
            let dump_phases = match print_phase {
                None => Vec::new(),
                Some(tokens) if tokens.is_empty() => {
                    print!("{}", available_phases());
                    return Ok(());
                }
                Some(tokens) => match resolve_phases(&tokens) {
                    Ok(phases) => phases,
                    Err(token) => {
                        eprintln!("error: unknown phase `{token}`\n\n{}", available_phases());
                        std::process::exit(2);
                    }
                },
            };
            let resolved_path;
            let path = match grammar.as_deref() {
                Some(path) => path,
                None => {
                    resolved_path = find_iggy_file(&output)?;
                    &resolved_path
                }
            };
            // A wasm bundle always lands in webview/ (-o conflicts with --wasm);
            // the grammar was already resolved from the output dir above.
            let output = if wasm {
                PathBuf::from(viewer::WEBVIEW_DIR)
            } else {
                output
            };
            let source = std::fs::read_to_string(path)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
            let grammar_def = parse_grammar(&source)
                .map_err(|errors| io::Error::other(render_errors(&errors, path, &source)))?;
            // Layer the config: built-in defaults, then a gen.toml beside the
            // grammar, then the explicit CLI flags. A flag left unset (None)
            // falls through to the file, and a key absent from the file falls
            // through to the default.
            let file = GenConfigFile::load(path)?;
            let mut config = GenConfig {
                // A wasm bundle has no CLI, so --wasm forces the lib-only shape.
                cli: cli && !wasm,
                wasm,
                ..GenConfig::default()
            };
            config.apply_file(&file);
            if let Some(ll1) = ll1 {
                config.ll1_optimization = ll1;
            }
            if let Some(match_memo) = match_memo {
                config.match_memo = match_memo;
            }
            if let Some(unsafe_mode) = unsafe_mode {
                config.unsafe_mode = unsafe_mode;
            }
            let bin_name = bin_name.or(file.bin_name);
            let runtime_path = runtime_path.or(file.runtime_path);
            let grammar: Grammar = grammar_def
                .to_grammar(&dump_phases)
                .map_err(|errors: Vec<String>| std::io::Error::other(errors.join("\n")))?;
            // Absolute so the dependency resolves from both webview/ and
            // webview/wasm/; canonicalize also validates the path exists.
            let runtime_path = runtime_path.map(std::fs::canonicalize).transpose()?;
            // A binary name (from --bin-name or gen.toml) only takes effect when
            // the Cargo.toml is scaffolded, so flag the no-op when an existing one
            // is being preserved.
            if bin_name.is_some() && !force && output.join("Cargo.toml").exists() {
                eprintln!(
                    "Warning: the binary name is ignored because {} already has a Cargo.toml; pass --force or set [[bin]] there manually.",
                    output.display()
                );
            }
            generate_scaffold(
                &grammar,
                &output,
                config,
                runtime_path.as_deref(),
                bin_name.as_deref(),
                force,
            )?;
            let result = generate_sources(&grammar, &output, config)?;
            if config.wasm {
                generate_wasm(&grammar, &output, runtime_path.as_deref(), force)?;
                iguana_compiler::wasm_build::build(&output.join("wasm"))?;
                viewer::write_assets(&output)?;
            }
            if json {
                println!("{{\"total_duration_ms\":{}}}", result.total_duration_ms);
            } else {
                println!(
                    "Generated {} grammar in {} ms",
                    grammar.name, result.total_duration_ms
                );
            }
        }
        Commands::Try { port } => viewer::try_bundle(Path::new(viewer::WEBVIEW_DIR), port)?,
    }
    Ok(())
}

fn new_project(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("destination `{}` already exists", path.display()),
        ));
    }

    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("could not derive a project name from `{}`", path.display()),
            )
        })?
        .to_owned();
    let grammar_name = to_pascal_case(&name);

    std::fs::create_dir_all(path)?;

    let grammar_file = path.join(format!("{name}.iggy"));
    std::fs::write(&grammar_file, starter_grammar(&grammar_name))?;

    std::fs::write(path.join(".gitignore"), "/target\n")?;

    println!("Created iggy grammar project at {}", path.display());
    println!("Run `iguana generate` to generate the parser from {name}.iggy");
    Ok(())
}

/// The grammar a new project starts from. One rule is enough for the crate to
/// generate, compile, and parse, and a placeholder keeps lexical rules and
/// layout out of a user's first minute.
fn starter_grammar(grammar_name: &str) -> String {
    format!("grammar {grammar_name}\n\nS = \"hello\"\n")
}

fn find_iggy_file(directory: &Path) -> std::io::Result<PathBuf> {
    let iggy_files: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "iggy"))
        .collect();

    match iggy_files.len() {
        0 => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("No .iggy file found in {}", directory.display()),
        )),
        1 => Ok(iggy_files[0].path()),
        n => Err(io::Error::other(format!(
            "Found {n} .iggy files in {}. Expected exactly one.",
            directory.display()
        ))),
    }
}

/// Resolves the `--print-phase` tokens into pipeline phases. `all` expands to
/// every phase; duplicates collapse and the result is in pipeline order,
/// independent of the order the tokens were given. Returns the first unknown
/// token on failure.
fn resolve_phases(tokens: &[String]) -> Result<Vec<Phase>, &str> {
    let mut requested = Vec::new();
    for token in tokens {
        if token == "all" {
            requested.extend(Phase::ALL);
        } else {
            requested.push(token.parse().map_err(|_| token.as_str())?);
        }
    }
    Ok(Phase::ALL
        .into_iter()
        .filter(|phase| requested.contains(phase))
        .collect())
}

/// A human-readable listing of the `--print-phase` tokens and what each prints.
fn available_phases() -> String {
    use std::fmt::Write;
    let mut message = String::from("Available phases:\n");
    for phase in Phase::ALL {
        writeln!(message, "  {:<11}{}", phase.token(), phase.description()).unwrap();
    }
    writeln!(message, "  {:<11}every phase", "all").unwrap();
    message
}
