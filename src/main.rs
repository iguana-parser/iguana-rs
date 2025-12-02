use std::{fs, io::Write, path::Path};

use clap::{Parser, Subcommand};
use iguana::{
    generator::{gen_cargo_toml_file, generate},
    grammar::symbols::{Grammar, Nonterminal, Seq, Symbol, Terminal},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate,
    Run,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate => generate_parser()?,
        Commands::Run => todo!(),
    }
    Ok(())
}

fn generate_parser() -> std::io::Result<()> {
    let name = "grammar-test";
    let base = Path::new("/Users/afroozeh/Workspace");
    let project_dir = base.join(name);
    if !project_dir.exists() {
        fs::create_dir(&project_dir)?;
    }
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let mut cargo_toml_file = fs::File::create(&cargo_toml_path)?;
    cargo_toml_file.write_all(gen_cargo_toml_file(name).as_bytes())?;
    cargo_toml_file.write_all(b"\n")?;
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir(&src_dir)?;
    }
    let lib_rs_path = src_dir.join("lib.rs");
    let mut lib_rs_file = fs::File::create(&lib_rs_path)?;
    lib_rs_file.write_all(generate(&grammar2()).as_bytes())?;
    lib_rs_file.write_all(b"\n")?;
    Ok(())
}

fn grammar1() -> Grammar {
    Grammar::builder()
        .name("Test2".to_string())
        .add_production(
            Nonterminal::new("E"),
            Seq::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .add_symbol(Symbol::Terminal(Terminal::new("+")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_production(
            Nonterminal::new("E"),
            Seq::builder()
                .add_symbol(Symbol::Terminal(Terminal::new("a")))
                .build(),
        )
        .start_symbol(Nonterminal::new("A"))
        .build()
}

fn grammar() -> Grammar {
    // A ::= A 'a'
    // A ::= 'a'
    Grammar::builder()
        .name("Test2".to_string())
        .add_production(
            Nonterminal::new("A"),
            Seq::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("A")))
                .add_symbol(Symbol::Terminal(Terminal::new("a")))
                .build(),
        )
        .add_production(
            Nonterminal::new("A"),
            Seq::builder()
                .add_symbol(Symbol::Terminal(Terminal::new("a")))
                .build(),
        )
        .start_symbol(Nonterminal::new("A"))
        .build()
}

fn grammar2() -> Grammar {
    // S -> E
    // E -> E '+' E
    // E -> 'a'
    Grammar::builder()
        .name("Test2".to_string())
        .add_production(
            Nonterminal::new("S"),
            Seq::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_production(
            Nonterminal::new("E"),
            Seq::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .add_symbol(Symbol::Terminal(Terminal::new("+")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_production(
            Nonterminal::new("E"),
            Seq::builder()
                .add_symbol(Symbol::Terminal(Terminal::new("a")))
                .build(),
        )
        .start_symbol(Nonterminal::new("S"))
        .build()
}

fn grammar3() -> Grammar {
    // S : S S S
    //   | S S
    //   | b
    Grammar::builder()
        .name("Test2".to_string())
        .add_production(
            Nonterminal::new("S"),
            Seq::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .build(),
        )
        .add_production(
            Nonterminal::new("S"),
            Seq::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .build(),
        )
        .add_production(
            Nonterminal::new("S"),
            Seq::builder()
                .add_symbol(Symbol::Terminal(Terminal::new("b")))
                .build(),
        )
        .start_symbol(Nonterminal::new("S"))
        .build()
}
