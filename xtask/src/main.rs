use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use clap::{Parser, Subcommand};
use iguana::{
    generator::{GenConfig, GenerateResult, generate_scaffold, generate_sources},
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
    /// Build iguana from this workspace and install it into `$CARGO_HOME/bin`
    Install,
    /// Install iguana, then launch the terrarium dev server
    Terrarium,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Bootstrap => bootstrap(),
        Commands::TestNew { name } => test_new(&name),
        Commands::TestRm { name } => test_rm(&name),
        Commands::TestGen { name } => test_gen(&name),
        Commands::TestGenAll => test_gen_all(),
        Commands::Install => install(),
        Commands::Terrarium => terrarium(),
    }
}

fn bootstrap() -> io::Result<()> {
    let iggy_dir = workspace_root().join("iggy");
    let grammar_file = iggy_dir.join("iggy.iggy");
    let result = regenerate(&grammar_file, &iggy_dir)?;
    println!("Generated iggy grammar in {} ms", result.total_duration_ms);
    Ok(())
}

fn test_new(name: &str) -> io::Result<()> {
    let test_dir = workspace_root().join("tests").join(name);
    let grammar_name = to_pascal_case(name);
    let grammar_file = test_dir.join(format!("{name}.iggy"));
    let cargo_toml = test_dir.join("Cargo.toml");
    let tests_rs = test_dir.join("tests.rs");

    if !test_dir.exists() {
        fs::create_dir_all(&test_dir)?;
        fs::write(&grammar_file, format!("grammar {grammar_name}\n"))?;
        println!("Created grammar: {}", grammar_file.display());
    }

    let result = regenerate(&grammar_file, &test_dir)?;
    println!(
        "Generated {name} grammar in {} ms",
        result.total_duration_ms
    );

    fs::create_dir_all(test_dir.join("parse_trees"))?;

    let cargo_content = fs::read_to_string(&cargo_toml)?;
    if !cargo_content.contains("[[test]]") {
        let updated = cargo_content.replace(
            "[features]",
            "[[test]]\nname = \"tests\"\npath = \"tests.rs\"\n\n[features]",
        );
        fs::write(&cargo_toml, updated)?;
    }

    if !tests_rs.exists() {
        let tests_content = format!(
            r#"// To regenerate parser:  cargo xtask test-gen {name}
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
            name = name
        );
        fs::write(&tests_rs, tests_content)?;
    }

    let workspace_cargo = workspace_root().join("Cargo.toml");
    let content = fs::read_to_string(&workspace_cargo)?;
    let member_entry = format!("\"tests/{name}\"");
    if !content.contains(&member_entry) {
        let new_content = content.replace(
            "    \"xtask\",\n",
            &format!("    \"xtask\",\n    {member_entry},\n"),
        );
        fs::write(&workspace_cargo, new_content)?;
    }

    println!();
    println!("To regenerate parser:  cargo xtask test-gen {name}");
    println!("To update golden files: REGENERATE=1 cargo test -p {name}");
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

    let workspace_cargo = workspace_root().join("Cargo.toml");
    let content = fs::read_to_string(&workspace_cargo)?;
    let member_entry = format!("    \"tests/{name}\",\n");
    if content.contains(&member_entry) {
        fs::write(&workspace_cargo, content.replace(&member_entry, ""))?;
    }
    Ok(())
}

fn test_gen(name: &str) -> io::Result<()> {
    let path = workspace_root().join("tests").join(name);
    let grammar_file = path.join(format!("{name}.iggy"));
    if !grammar_file.exists() {
        println!("Grammar file not found: {}", grammar_file.display());
        return Ok(());
    }

    let result = regenerate(&grammar_file, &path)?;
    println!(
        "Generated {name} grammar in {} ms",
        result.total_duration_ms
    );
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

fn regenerate(grammar_path: &Path, output: &Path) -> io::Result<GenerateResult> {
    let source = fs::read_to_string(grammar_path)?;
    let grammar_def = parse_grammar(&source).map_err(io::Error::other)?;
    let grammar: Grammar = grammar_def.try_into().map_err(|names: Vec<String>| {
        io::Error::other(format!("Unresolved identifiers: {}", names.join(", ")))
    })?;
    generate_scaffold(&grammar, output)?;
    let result = generate_sources(&grammar, output, GenConfig::default())?;
    format_sources(output)?;
    Ok(result)
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

fn install() -> io::Result<()> {
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
