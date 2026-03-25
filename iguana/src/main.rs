use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use iguana::{
    alternative, bind, c, call, cond, cond_expr,
    generator::{GenConfig, generate},
    grammar::def::{Grammar, GrammarDef},
    grammar::symbols::Terminal,
    grammar_def, id,
    iggy::parse_grammar,
    lexical_rule, lit, min, opt, priority_level, r_star, ret, syntax_rule, ternary,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new iggy project
    Init {
        /// Grammar name
        name: String,

        /// Directory to initialize (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    Generate {
        /// Path to an iggy grammar file. If not provided, uses the built-in iggy grammar.
        #[arg(short, long)]
        grammar: Option<PathBuf>,

        /// Output directory for generated parser (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Output timing information as JSON (for tool integration)
        #[arg(long)]
        json: bool,
    },
    Run,
    /// Test-related commands
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
}

#[derive(Subcommand)]
enum TestCommands {
    /// Initialize a new grammar test
    Init {
        /// Name of the grammar (creates tests/<name>/)
        name: String,
    },
    /// Delete a grammar test
    #[command(alias = "rm")]
    Delete {
        /// Name of the grammar test to delete
        name: String,
    },
    /// Regenerate the parser for a single test
    #[command(alias = "gen")]
    Generate {
        /// Name of the grammar test to regenerate
        name: String,
    },
    /// Generate all test parsers
    GenerateAll,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name, output } => init_project(&output, &name)?,
        Commands::Generate {
            grammar,
            output,
            json,
        } => {
            let resolved_path;
            let path = match grammar.as_deref() {
                Some(path) => path,
                None => {
                    resolved_path = find_iggy_file(&output)?;
                    &resolved_path
                }
            };
            let source = std::fs::read_to_string(path)?;
            let grammar_def = parse_grammar(&source).map_err(std::io::Error::other)?;
            let result = generate(&grammar_def.into(), &output, GenConfig::default())?;
            if json {
                println!("{{\"total_duration_ms\":{}}}", result.total_duration_ms);
            } else {
                println!("Generated in {}ms", result.total_duration_ms);
            }
        }
        Commands::Run => todo!(),
        Commands::Test { command } => match command {
            TestCommands::Init { name } => init_test(&name)?,
            TestCommands::Delete { name } => delete_test(&name)?,
            TestCommands::Generate { name } => generate_test(&name)?,
            TestCommands::GenerateAll => generate_all_tests()?,
        },
    }
    Ok(())
}

fn init_project(output: &Path, name: &str) -> std::io::Result<()> {
    use std::io::Write;
    let grammar_name = to_pascal_case(name);
    let grammar_file = output.join(format!("{}.iggy", name));

    // Create directory if needed
    if !output.exists() {
        std::fs::create_dir_all(output)?;
        println!("Created directory: {}", output.display());
    }

    // Create grammar file if it doesn't exist
    if !grammar_file.exists() {
        std::fs::write(&grammar_file, format!("grammar {grammar_name}\n"))?;
        println!("Created grammar: {}", grammar_file.display());
    }

    // Generate parser (creates Cargo.toml and source files)
    let cargo_toml = output.join("Cargo.toml");
    if !cargo_toml.exists() {
        print!("Generating parser... ");
        std::io::stdout().flush()?;
        generate_parser(Some(&grammar_file), output)?;
        println!("done");
    }

    println!("Initialized iggy project at {}", output.display());
    Ok(())
}

fn init_test(name: &str) -> std::io::Result<()> {
    use std::io::Write;

    let test_dir = PathBuf::from("tests").join(name);
    let grammar_name = to_pascal_case(name);
    let grammar_file = test_dir.join(format!("{}.iggy", name));
    let cargo_toml = test_dir.join("Cargo.toml");
    let tests_rs = test_dir.join("tests.rs");

    // Create directory and grammar file if they don't exist
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)?;
        std::fs::write(&grammar_file, format!("grammar {grammar_name}\n"))?;
        println!("Created grammar: {}", grammar_file.display());
    }

    // Generate parser if not yet generated
    if !cargo_toml.exists() {
        print!("Generating parser... ");
        std::io::stdout().flush()?;
        generate_parser(Some(&grammar_file), &test_dir)?;
        println!("done");
    }

    // Add test infrastructure
    std::fs::create_dir_all(test_dir.join("parse_trees"))?;

    // Add [[test]] section to Cargo.toml if not present
    let cargo_content = std::fs::read_to_string(&cargo_toml)?;
    if !cargo_content.contains("[[test]]") {
        let updated = cargo_content.replace(
            "[features]",
            "[[test]]\nname = \"tests\"\npath = \"tests.rs\"\n\n[features]",
        );
        std::fs::write(&cargo_toml, updated)?;
    }

    // Create tests.rs if not present
    if !tests_rs.exists() {
        let tests_content = format!(
            r#"// To regenerate parser:  cargo run -p iguana -- generate --grammar {grammar_file} --output {test_dir}
// To update golden files: REGENERATE=1 cargo test -p {name}

use {name}::{{parse, parse_tree::to_sexpr}};
use iguana_runtime::{{input::Input, testing::{{check_golden_file, golden_path}}}};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {{
    let input = Input::from(input);
    let tree = parse(&input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}}

#[test]
fn test_example() {{
    // check("Start", "input", "example");
}}
"#,
            grammar_file = grammar_file.display(),
            test_dir = test_dir.display(),
            name = name
        );
        std::fs::write(&tests_rs, tests_content)?;
    }

    // Add to workspace members if workspace Cargo.toml exists
    let workspace_cargo = PathBuf::from("Cargo.toml");
    if workspace_cargo.exists() {
        let content = std::fs::read_to_string(&workspace_cargo)?;
        let member_entry = format!("tests/{name}");
        if content.contains("[workspace]") && !content.contains(&member_entry) {
            // Find members array and add the new member
            if let Some(members_start) = content.find("members = [") {
                let before_bracket = &content[..members_start + 11];
                let after_start = &content[members_start + 11..];
                if let Some(bracket_end) = after_start.find(']') {
                    let members_content = &after_start[..bracket_end];
                    let after_bracket = &after_start[bracket_end..];

                    // Add new member
                    let new_content = format!(
                        "{}\"{}\", {}{}",
                        before_bracket,
                        member_entry,
                        members_content.trim_start(),
                        after_bracket
                    );
                    std::fs::write(&workspace_cargo, new_content)?;
                }
            }
        }
    }

    println!();
    println!(
        "To regenerate parser:  cargo run -p iguana -- generate --grammar {} --output {}",
        grammar_file.display(),
        test_dir.display()
    );
    println!("To update golden files: REGENERATE=1 cargo test -p {name}");

    Ok(())
}

fn delete_test(name: &str) -> std::io::Result<()> {
    let test_dir = PathBuf::from("tests").join(name);

    // Remove the directory
    if test_dir.exists() {
        std::fs::remove_dir_all(&test_dir)?;
        println!("Deleted: tests/{name}/");
    } else {
        println!("Test not found: tests/{name}/");
    }

    // Remove from workspace members
    let workspace_cargo = PathBuf::from("Cargo.toml");
    if workspace_cargo.exists() {
        let content = std::fs::read_to_string(&workspace_cargo)?;
        let member_entry = format!("\"tests/{name}\"");
        if content.contains(&member_entry) {
            // Remove the member entry (with trailing comma and space, or just the entry)
            let new_content = content
                .replace(&format!("{}, ", member_entry), "")
                .replace(&format!(", {}", member_entry), "")
                .replace(&member_entry, "");
            std::fs::write(&workspace_cargo, new_content)?;
        }
    }

    Ok(())
}

fn generate_test(name: &str) -> std::io::Result<()> {
    use std::io::Write;

    let path = PathBuf::from("tests").join(name);
    let grammar_file = path.join(format!("{}.iggy", name));
    if !grammar_file.exists() {
        println!("Grammar file not found: {}", grammar_file.display());
        return Ok(());
    }

    print!("Generating {}... ", name);
    std::io::stdout().flush()?;
    generate_parser(Some(&grammar_file), &path)?;

    // Re-add [[test]] section if tests.rs exists
    let tests_rs = path.join("tests.rs");
    if tests_rs.exists() {
        let cargo_toml = path.join("Cargo.toml");
        let cargo_content = std::fs::read_to_string(&cargo_toml)?;
        if !cargo_content.contains("[[test]]") {
            let updated = cargo_content.replace(
                "[features]",
                "[[test]]\nname = \"tests\"\npath = \"tests.rs\"\n\n[features]",
            );
            std::fs::write(&cargo_toml, updated)?;
        }
    }

    println!("done");
    Ok(())
}

fn generate_all_tests() -> std::io::Result<()> {
    let tests_dir = PathBuf::from("tests");
    if !tests_dir.exists() {
        println!("No tests directory found");
        return Ok(());
    }

    for entry in std::fs::read_dir(&tests_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_string_lossy();
            let grammar_file = path.join(format!("{}.iggy", name));
            if grammar_file.exists() {
                generate_test(&name)?;
            }
        }
    }

    Ok(())
}

fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
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

fn generate_parser(grammar_path: Option<&Path>, output: &Path) -> std::io::Result<()> {
    let resolved_path;
    let path = match grammar_path {
        Some(path) => path,
        None => {
            resolved_path = find_iggy_file(output)?;
            &resolved_path
        }
    };
    let source = std::fs::read_to_string(path)?;
    let grammar = parse_grammar(&source).map_err(std::io::Error::other)?;
    generate(&grammar.into(), output, GenConfig::default())?;
    Ok(())
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
    .into()
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
    .into()
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
        layout: [
            Terminal::new("WS")
        ],
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
        layout: [
            Terminal::new("WS")
        ],
    )
}
