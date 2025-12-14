use clap::{Parser, Subcommand};
use iguana::{
    generator::generate,
    grammar::{
        grammar::{Alternative, Grammar, GrammarDef, PriorityLevel, SyntaxRule},
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
    let grammar = &grammar2();
    generate(grammar)?;

    Ok(())
}

fn grammar1() -> Grammar {
    GrammarDef::builder()
        .name("Test2".to_string())
        .add_syntax_rule(
            SyntaxRule::builder()
                .head(Nonterminal::new("E"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::nonterminal("E"))
                                .add_symbol(Symbol::literal("+"))
                                .add_symbol(Symbol::nonterminal("E"))
                                .build(),
                        )
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::literal("a"))
                                .build(),
                        )
                        .build(),
                )
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
            SyntaxRule::builder()
                .head(Nonterminal::new("A"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::nonterminal("A"))
                                .add_symbol(Symbol::literal("a"))
                                .build(),
                        )
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::literal("a"))
                                .build(),
                        )
                        .build(),
                )
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
    // WS -> ' '*
    GrammarDef::builder()
        .name("Test2".to_string())
        .add_syntax_rule(
            SyntaxRule::builder()
                .head(Nonterminal::new("S"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::nonterminal("E"))
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .add_syntax_rule(
            SyntaxRule::builder()
                .head(Nonterminal::new("E"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::nonterminal("E"))
                                .add_symbol(Symbol::literal("+"))
                                .add_symbol(Symbol::nonterminal("E"))
                                .build(),
                        )
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::literal("a"))
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .add_lexical_rule(Terminal::identifier("WS"), Regex::star(Regex::Char(' ')))
        .add_layout_definition(Terminal::identifier("WS"))
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
            SyntaxRule::builder()
                .head(Nonterminal::new("S"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::nonterminal("S"))
                                .add_symbol(Symbol::nonterminal("S"))
                                .add_symbol(Symbol::nonterminal("S"))
                                .build(),
                        )
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::nonterminal("S"))
                                .add_symbol(Symbol::nonterminal("S"))
                                .build(),
                        )
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::literal("b"))
                                .build(),
                        )
                        .build(),
                )
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
// WS
//   = [ ]*
fn iggy() -> Grammar {
    GrammarDef::builder()
        .name("Iggy".to_string())
        .add_syntax_rule(
            SyntaxRule::builder()
                .head(Nonterminal::new("Grammar"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::literal("grammar"))
                                .add_symbol(Symbol::literal("WS"))
                                .add_symbol(Symbol::literal("Identifier"))
                                .build(),
                        )
                        .build(),
                )
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
        .add_lexical_rule(Terminal::identifier("WS"), Regex::star(Regex::Char(' ')))
        .start_symbol(Nonterminal::new("Grammar"))
        .build()
        .into()
}
