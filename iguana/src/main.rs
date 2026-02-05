use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use iguana::{
    alternative,
    generator::generate,
    grammar::def::Grammar,
    grammar_def, id,
    iggy::parse_grammar,
    lit, opt, priority_level, syntax_rule,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        /// Path to an iggy grammar file. If not provided, uses the built-in iggy grammar.
        #[arg(short, long)]
        grammar: Option<PathBuf>,

        /// Output directory for generated parser
        #[arg(short, long)]
        output: PathBuf,
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
    /// Generate all test parsers
    GenerateAll,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate { grammar, output } => generate_parser(grammar.as_deref(), &output)?,
        Commands::Run => todo!(),
        Commands::Test { command } => match command {
            TestCommands::Init { name } => init_test(&name)?,
            TestCommands::Delete { name } => delete_test(&name)?,
            TestCommands::GenerateAll => generate_all_tests()?,
        },
    }
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
use iguana_runtime::testing::{{check_golden_file, golden_path}};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {{
    let tree = parse(input, start_nonterminal).expect("Parse failed");
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

fn generate_all_tests() -> std::io::Result<()> {
    use std::io::Write;

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
            }
        }
    }

    Ok(())
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

fn generate_parser(grammar_path: Option<&Path>, output: &Path) -> std::io::Result<()> {
    let grammar = match grammar_path {
        Some(path) => {
            let source = std::fs::read_to_string(path)?;
            parse_grammar(&source).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        }
        None => panic!(),
    };
    generate(&grammar.into(), output)?;
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
