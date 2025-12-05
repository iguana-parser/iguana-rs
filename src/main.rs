use clap::{Parser, Subcommand};
use iguana::{
    generator::generate,
    grammar::{
        grammar::{Alternative, Grammar, GrammarDef},
        regex::Regex,
        symbols::{Nonterminal, Symbol, Terminal},
    },
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
    // let cli = Cli::parse();
    generate_parser()?;
    // match cli.command {
    //     Commands::Generate => generate_parser()?,
    //     Commands::Run => todo!(),
    // }
    Ok(())
}

fn generate_parser() -> std::io::Result<()> {
    let grammar = &iggy();
    generate(grammar)?;

    Ok(())
}

fn grammar1() -> Grammar {
    GrammarDef::builder()
        .name("Test2".to_string())
        .add_syntax_rule(
            Nonterminal::new("E"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .add_symbol(Symbol::Terminal(Terminal::literal("+")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_syntax_rule(
            Nonterminal::new("E"),
            Alternative::builder()
                .add_symbol(Symbol::Terminal(Terminal::literal("a")))
                .build(),
        )
        .start_symbol(Nonterminal::new("A"))
        .build()
        .into()
}

fn grammar() -> Grammar {
    // A ::= A 'a'
    // A ::= 'a'
    GrammarDef::builder()
        .name("Test2".to_string())
        .add_syntax_rule(
            Nonterminal::new("A"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("A")))
                .add_symbol(Symbol::Terminal(Terminal::literal("a")))
                .build(),
        )
        .add_syntax_rule(
            Nonterminal::new("A"),
            Alternative::builder()
                .add_symbol(Symbol::Terminal(Terminal::literal("a")))
                .build(),
        )
        .start_symbol(Nonterminal::new("A"))
        .build()
        .into()
}

fn grammar2() -> Grammar {
    // S -> E
    // E -> E '+' E
    // E -> 'a'
    GrammarDef::builder()
        .name("Test2".to_string())
        .add_syntax_rule(
            Nonterminal::new("S"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_syntax_rule(
            Nonterminal::new("E"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .add_symbol(Symbol::Terminal(Terminal::literal("+")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                .build(),
        )
        .add_syntax_rule(
            Nonterminal::new("E"),
            Alternative::builder()
                .add_symbol(Symbol::Terminal(Terminal::literal("a")))
                .build(),
        )
        .start_symbol(Nonterminal::new("S"))
        .build()
        .into()
}

fn grammar3() -> Grammar {
    // S : S S S
    //   | S S
    //   | b
    GrammarDef::builder()
        .name("Test2".to_string())
        .add_syntax_rule(
            Nonterminal::new("S"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .build(),
        )
        .add_syntax_rule(
            Nonterminal::new("S"),
            Alternative::builder()
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .add_symbol(Symbol::Nonterminal(Nonterminal::new("S")))
                .build(),
        )
        .add_syntax_rule(
            Nonterminal::new("S"),
            Alternative::builder()
                .add_symbol(Symbol::Terminal(Terminal::literal("b")))
                .build(),
        )
        .start_symbol(Nonterminal::new("S"))
        .build()
        .into()
}

// Grammar
//   = "grammar" Identifier
//   ;
// Identifier
//   = [a-zA-Z_][a-zA-Z_0-9]*
fn iggy() -> Grammar {
    GrammarDef::builder()
        .name("Iggy".to_string())
        .add_syntax_rule(
            Nonterminal::new("Grammar"),
            Alternative::builder()
                .add_symbol(Symbol::Terminal(Terminal::literal("grammar")))
                .add_symbol(Symbol::Terminal(Terminal::identifier("Identifier")))
                .build(),
        )
        .add_lexical_rule(
            Terminal::identifier("Identifier"),
            Regex::Seq(vec![
                Regex::Alt(vec![
                    Regex::CharRange {
                        start: 'a',
                        end: 'z',
                    },
                    Regex::CharRange {
                        start: 'A',
                        end: 'Z',
                    },
                    Regex::Char('_'),
                ]),
                Regex::Star(Box::new(Regex::Alt(vec![
                    Regex::CharRange {
                        start: 'a',
                        end: 'z',
                    },
                    Regex::CharRange {
                        start: 'A',
                        end: 'Z',
                    },
                    Regex::CharRange {
                        start: '0',
                        end: '9',
                    },
                    Regex::Char('_'),
                ]))),
            ]),
        )
        .start_symbol(Nonterminal::new("Grammar"))
        .build()
        .into()
}
