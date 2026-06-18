use std::{
    io,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use iguana::{
    alternative, bind, c, call, cond, cond_expr,
    generator::{GenConfig, generate_scaffold, generate_sources, generate_wasm},
    grammar::def::{Grammar, GrammarDef, Phase},
    grammar_def, id,
    iggy::parse_grammar,
    lexical_rule, lit, min, opt, priority_level, r_star, ret, syntax_rule, ternary,
    utils::to_pascal_case,
};

#[derive(Parser)]
#[command(name = "iguana", version, about = "A GLL-based parser generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new iggy project in the given directory
    New {
        /// Path of the new project directory. Must not already exist.
        path: PathBuf,
    },
    /// Generate a parser crate from an iggy grammar
    Generate {
        /// Path to an iggy grammar file. Defaults to the .iggy file in the output directory.
        #[arg(short, long)]
        grammar: Option<PathBuf>,

        /// Output directory for generated parser (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Output timing information as JSON (for tool integration)
        #[arg(long)]
        json: bool,

        /// Enable LL(1) optimization in code generation
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        ll1: bool,

        /// Memoize scanner results (match_token and match_any) during parsing
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        match_memo: bool,

        /// Scaffold a CLI binary (Cargo.toml + src/main.rs). When false, the
        /// caller owns Cargo.toml and no CLI binary is emitted.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        cli: bool,

        /// Emit a wasm bundle into webview/: a lib crate, a wasm-bindgen wrapper
        /// crate under wasm/, a manifest.json, and the web viewer. Serve it with
        /// `iguana try`. The bundle has no CLI and a fixed location, so --wasm
        /// conflicts with --cli and -o.
        #[arg(long, conflicts_with_all = ["cli", "output"])]
        wasm: bool,

        /// Point the generated crate's `iguana-runtime` dependency at this local
        /// path instead of the default git dependency. Applies to the standalone
        /// (--cli) and wasm shapes; use it to develop the runtime alongside a
        /// grammar or pin the bundle to a specific local checkout.
        #[arg(long)]
        runtime_path: Option<PathBuf>,

        /// Overwrite scaffolded files (Cargo.toml, src/main.rs) even if they
        /// already exist. Without this, the scaffold step is skipped on
        /// regeneration so local edits are preserved.
        #[arg(long)]
        force: bool,

        /// Print the grammar after the given pipeline phases (comma-separated)
        /// to stderr, then keep generating. Pass the flag with no value to list
        /// the available phases.
        #[arg(long, value_delimiter = ',', num_args = 0..)]
        print_phase: Option<Vec<String>>,
    },
    /// Serve the wasm bundle in webview/ over HTTP and open it in the web
    /// viewer. Build the bundle first with `iguana generate --wasm`.
    Try {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8000)]
        port: u16,
    },
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { path } => new_project(&path)?,
        Commands::Generate {
            grammar,
            output,
            json,
            ll1,
            match_memo,
            cli,
            wasm,
            runtime_path,
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
                PathBuf::from(iguana::viewer::WEBVIEW_DIR)
            } else {
                output
            };
            let source = std::fs::read_to_string(path)?;
            let grammar_def = parse_grammar(&source).map_err(std::io::Error::other)?;
            let config = GenConfig {
                ll1_optimization: ll1,
                match_memo,
                // A wasm bundle has no CLI, so --wasm forces the lib-only shape.
                cli: cli && !wasm,
                wasm,
            };
            let grammar: Grammar =
                grammar_def
                    .to_grammar(&dump_phases)
                    .map_err(|names: Vec<String>| {
                        std::io::Error::other(format!(
                            "Unresolved identifiers: {}",
                            names.join(", ")
                        ))
                    })?;
            // Absolute so the dependency resolves from both webview/ and
            // webview/wasm/; canonicalize also validates the path exists.
            let runtime_path = runtime_path.map(std::fs::canonicalize).transpose()?;
            generate_scaffold(&grammar, &output, config, runtime_path.as_deref(), force)?;
            let result = generate_sources(&grammar, &output, config)?;
            if config.wasm {
                generate_wasm(&grammar, &output, runtime_path.as_deref(), force)?;
                iguana::wasm_build::build(&output.join("wasm"))?;
                iguana::viewer::write_assets(&output)?;
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
        Commands::Try { port } => {
            iguana::viewer::try_bundle(Path::new(iguana::viewer::WEBVIEW_DIR), port)?
        }
    }
    Ok(())
}

fn new_project(path: &Path) -> io::Result<()> {
    use io::Write;

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
    std::fs::write(&grammar_file, format!("grammar {grammar_name}\n"))?;

    std::fs::write(path.join(".gitignore"), "/target\n")?;

    print!("Generating parser... ");
    io::stdout().flush()?;
    generate_parser(Some(&grammar_file), path)?;
    println!("done");

    println!("Created iggy project at {}", path.display());
    Ok(())
}

fn find_iggy_file(directory: &Path) -> std::io::Result<PathBuf> {
    let iggy_files: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "iggy"))
        .collect();

    match iggy_files.len() {
        0 => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No .iggy file found in {}", directory.display()),
        )),
        1 => Ok(iggy_files[0].path()),
        n => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Found {n} .iggy files in {}. Expected exactly one.",
                directory.display()
            ),
        )),
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
    writeln!(message, "  {:<11}{}", "all", "every phase").unwrap();
    message
}

fn generate_parser(grammar_path: Option<&Path>, output: &Path) -> io::Result<()> {
    let grammar = load_grammar(grammar_path, output)?;
    let config = GenConfig::default();
    generate_scaffold(&grammar, output, config, None, false)?;
    generate_sources(&grammar, output, config)?;
    Ok(())
}

fn load_grammar(grammar_path: Option<&Path>, output: &Path) -> io::Result<Grammar> {
    let resolved_path;
    let path = match grammar_path {
        Some(path) => path,
        None => {
            resolved_path = find_iggy_file(output)?;
            &resolved_path
        }
    };
    let source = std::fs::read_to_string(path)?;
    let grammar_def = parse_grammar(&source).map_err(io::Error::other)?;
    grammar_def.try_into().map_err(|names: Vec<String>| {
        io::Error::other(format!("Unresolved identifiers: {}", names.join(", ")))
    })
}

#[allow(dead_code)]
fn grammar3() -> Grammar {
    // S : S S S | S S | b
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => priority_level!(
                alternative!(id!("S"), id!("S"), id!("S")),
                alternative!(id!("S"), id!("S")),
                alternative!(lit!("b"))
            ))
        ]
    )
    .try_into()
    .unwrap()
}

#[allow(dead_code)]
fn ambiguous_grammar() -> Grammar {
    // S: A? | ;
    // A: "a";
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => priority_level!(alternative!(opt!(id!("A"))), alternative!())),
            syntax_rule!("A" => alternative!(lit!("a")))
        ]
    )
    .try_into()
    .unwrap()
}

fn nonterminal_parameters() -> GrammarDef {
    // S = A(0)
    // A(p)
    //  = [p == 0] "a"
    //  | [p == 1] "b"
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(call!("A", 0))),
            syntax_rule!("A"("p": I32) => priority_level!(
                alternative!(cond!("p" == 0), lit!("a")),
                alternative!(cond!("p" == 1), lit!("b")),
            )),
        ]
    )
}

// S = E(0)
// E(p)
//   = [p == 0] E(0) "+" E(1)
//   | "a"
fn conditions() -> GrammarDef {
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(call!("E", 0))),
            syntax_rule!("E"("p": I32) => priority_level!(
                alternative!(cond!("p" == 0), call!("E", 0), lit!("+"), call!("E", 1)),
                alternative!(lit!("a")),
            )),
        ]
    )
}

// S = E(0)
// E(p)
//   = [2>=p] l=E(p) [l==0||l>=2] '*' E(3) return 2
//   | [1>=p] l=E(p) [l==0||l>=1] '+' E(2) return 1
//   | 'a' return 0
fn return_values() -> GrammarDef {
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(call!("E", 0))),
            syntax_rule!("E"("p": I32) => priority_level!(
                alternative!(
                    cond!(2 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 2)),
                    lit!("*"),
                    call!("E", 3),
                    ret!(2),
                ),
                alternative!(
                    cond!(1 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 1)),
                    lit!("+"),
                    call!("E", 2),
                    ret!(1),
                ),
                alternative!(lit!("a"), ret!(0)),
            )),
        ]
    )
}

// S = E(0)
// E(p)
//   = [2>=p] l=E(p) [l==0||l>=2] '*' E(3) return 2
//   | [2>=p] l=E(p) [l==0||l>=2] '/' E(3) return 2
//   | [1>=p] l=E(p) [l==0||l>=1] '+' E(2) return 1
//   | [1>=p] l=E(p) [l==0||l>=1] '-' E(2) return 1
//   | 'a' return 0
fn binary_expressions_with_multiple_precedence_levels() -> GrammarDef {
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(call!("E", 0))),
            syntax_rule!("E"("p": I32) => priority_level!(
                alternative!(
                    cond!(2 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 2)),
                    lit!("*"),
                    call!("E", 3),
                    ret!(2),
                ),
                alternative!(
                    cond!(2 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 2)),
                    lit!("/"),
                    call!("E", 3),
                    ret!(2),
                ),
                alternative!(
                    cond!(1 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 1)),
                    lit!("+"),
                    call!("E", 2),
                    ret!(1),
                ),
                alternative!(
                    cond!(1 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 1)),
                    lit!("-"),
                    call!("E", 2),
                    ret!(1),
                ),
                alternative!(lit!("a"), ret!(0)),
            )),
        ]
    )
}

// S = E(0)
// E(p)
//   = [2>=p] l=E(p) [l==0||l>=2] '*' E(3) return 2
//   | [1>=p] l=E(p) [l==0||l>=1] '+' E(2) return 1
//   | [3>=p] '-' E(3) return 3
//   | 'a' return 0
fn unary_expression() -> GrammarDef {
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(call!("E", 0))),
            syntax_rule!("E"("p": I32) => priority_level!(
                alternative!(
                    cond!(2 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 2)),
                    lit!("*"),
                    call!("E", 3),
                    ret!(2),
                ),
                alternative!(
                    cond!(1 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 1)),
                    lit!("+"),
                    call!("E", 2),
                    ret!(1),
                ),
                alternative!(
                    cond!(3 >= "p"),
                    lit!("-"),
                    call!("E", 3),
                    ret!(3),
                ),
                alternative!(lit!("a"), ret!(0)),
            )),
        ]
    )
}

// E(p)
//   = [2>=p] l=E(p) [l==0||l>=2] '+' r=E(3) return r==0 ? 2 : min(r,2)
//   | 'if' E(0) 'then' E(0) 'else' E(1) return 1
//   | 'a' return 0
fn deep_unary_case() -> GrammarDef {
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(call!("E", 0))),
            syntax_rule!("E"("p": I32) => priority_level!(
                alternative!(
                    cond!(2 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 2)),
                    lit!("+"),
                    bind!("r", call!("E", 3)),
                    ret!(expr ternary!(cond_expr!("r" == 0), 2, min!("r", 2))),
                ),
                alternative!(
                    lit!("if"),
                    call!("E", 0),
                    lit!("then"),
                    call!("E", 0),
                    lit!("else"),
                    call!("E", 1),
                    ret!(1),
                ),
                alternative!(lit!("a"), ret!(0)),
            )),
        ],
        lexical: [
            lexical_rule!("WS" => r_star!(c!(' ')))
        ],
        layout: id!("WS"),
    )
}

// E(p)
//  = [7>=p] l=E(p) [l==0||l>=7] '.' 'f' return 0
//  | [6>=p] l=E(p) [l==0||l>=6] r=E(7) return r==0 ? 6 : min(r,6)
//  | '-' r=E(5) return r==0 ? 5 : min(r,5)
//  | [4>=p] l=E(p) [l==0||l>=4] '*' r=E(5) return r==0 ? 4 : min(r,4)
//  | [3>=p] l=E(p) [l==0||l>=3] '+' r=E(4) return r==0 ? 3 : min(r,3)
//  | [3>=p] l=E(p) [l==0||l>=3] '-' r=E(4) return r==0 ? 3 : min(r,3)
//  | 'if' E(0) 'then' E(2) return 2
//  | [1>=p] l=E(p) [l==0||l>=2] ';' r=E(1) return 1
//  | '(' E(0) ')' return 0
//  | 'a' return 0
fn full_pepm16_example() -> GrammarDef {
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(call!("E", 0))),
            syntax_rule!("E"("p": I32) => priority_level!(
                alternative!(
                    cond!(7 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 7)),
                    lit!("."),
                    lit!("f"),
                    ret!(0),
                ),
                alternative!(
                    cond!(6 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 6)),
                    bind!("r", call!("E", 7)),
                    ret!(expr ternary!(cond_expr!("r" == 0), 6, min!("r", 6))),
                ),
                alternative!(
                    lit!("-"),
                    bind!("r", call!("E", 5)),
                    ret!(expr ternary!(cond_expr!("r" == 0), 5, min!("r", 5))),
                ),
                alternative!(
                    cond!(4 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 4)),
                    lit!("*"),
                    bind!("r", call!("E", 5)),
                    ret!(expr ternary!(cond_expr!("r" == 0), 4, min!("r", 4))),
                ),
                alternative!(
                    cond!(3 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 3)),
                    lit!("+"),
                    bind!("r", call!("E", 4)),
                    ret!(expr ternary!(cond_expr!("r" == 0), 3, min!("r", 3))),
                ),
                alternative!(
                    cond!(3 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 3)),
                    lit!("-"),
                    bind!("r", call!("E", 4)),
                    ret!(expr ternary!(cond_expr!("r" == 0), 3, min!("r", 3))),
                ),
                alternative!(
                    lit!("if"),
                    call!("E", 0),
                    lit!("then"),
                    call!("E", 2),
                    ret!(2),
                ),
                alternative!(
                    cond!(1 >= "p"),
                    bind!("l", call!("E", ref "p")),
                    cond!(("l" == 0) || ("l" >= 2)),
                    lit!(";"),
                    bind!("r", call!("E", 1)),
                    ret!(1),
                ),
                alternative!(
                    lit!("("),
                    call!("E", 0),
                    lit!(")"),
                    ret!(0),
                ),
                alternative!(lit!("a"), ret!(0)),
            )),
        ],
        lexical: [
            lexical_rule!("WS" => r_star!(c!(' ')))
        ],
        layout: id!("WS"),
    )
}
