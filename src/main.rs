use std::path::{Path, PathBuf};

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
    Generate {
        #[arg(short, long)]
        output: PathBuf,
    },
    Run,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate { output } => generate_parser(&output)?,
        Commands::Run => todo!(),
    }
    Ok(())
}

fn generate_parser(output: &Path) -> std::io::Result<()> {
    let grammar = iggy().into();
    generate(&grammar, output)?;
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
//   : "grammar" Identifier Rule+
//   ;
// Rule
//   : Identifier ":" Identifier+ ";"
//   ;
// regex Identifier
//   : [a-zA-Z_][a-zA-Z_0-9]*
//   ;
// WS
//   : [ \n]*
//   ;
fn iggy() -> GrammarDef {
    GrammarDef::builder()
        .name("Iggy".to_string())
        // Grammar : "grammar" Identifier Rule+
        .add_syntax_rule(
            SyntaxRule::builder()
                .head(Nonterminal::new("Grammar"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::literal("grammar"))
                                .add_symbol(Symbol::terminal("Identifier"))
                                .add_symbol(Symbol::plus(Symbol::nonterminal("Rule")))
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        // Rule : Identifier ":" Identifier+
        .add_syntax_rule(
            SyntaxRule::builder()
                .head(Nonterminal::new("Rule"))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::terminal("Identifier"))
                                .add_symbol(Symbol::literal(":"))
                                .add_symbol(Symbol::plus(Symbol::terminal("Identifier")))
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        // regex Identifier : [a-zA-Z_][a-zA-Z_0-9]*
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
        // WS : [ ]*
        .add_lexical_rule(
            Terminal::identifier("WS"),
            Regex::star(Regex::Alt(vec![Regex::Char(' '), Regex::Char('\n')])),
        )
        .add_layout_definition(Terminal::identifier("WS"))
        .start_symbol(Nonterminal::new("Grammar"))
        .build()
}
