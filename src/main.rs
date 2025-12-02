use clap::{Parser, Subcommand};
use iguana::{
    generator::generate,
    grammar::symbols::{Alternative, Grammar, Nonterminal, Symbol, Terminal},
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
    let grammar = &grammar2();
    generate(grammar)?;

    Ok(())
}

fn grammar1() -> Grammar {
    Grammar::builder()
        .name("Test2".to_string())
        .add_production(
            Nonterminal::new("E"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .add_symbol(Symbol::Terminal(Terminal::new("+")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_production(
            Nonterminal::new("E"),
            Alternative::builder()
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
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("A")))
                .add_symbol(Symbol::Terminal(Terminal::new("a")))
                .build(),
        )
        .add_production(
            Nonterminal::new("A"),
            Alternative::builder()
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
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_production(
            Nonterminal::new("E"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .add_symbol(Symbol::Terminal(Terminal::new("+")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_production(
            Nonterminal::new("E"),
            Alternative::builder()
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
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .build(),
        )
        .add_production(
            Nonterminal::new("S"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .build(),
        )
        .add_production(
            Nonterminal::new("S"),
            Alternative::builder()
                .add_symbol(Symbol::Terminal(Terminal::new("b")))
                .build(),
        )
        .start_symbol(Nonterminal::new("S"))
        .build()
}
