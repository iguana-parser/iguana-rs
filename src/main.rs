use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use iguana::{
    alternative,
    generator::generate,
    grammar::{def::Grammar, regex::Regex, symbols::Terminal},
    grammar_def, id, lexical_rule, lit, plus, priority_level, syntax_rule,
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
    let grammar = iggy();
    generate(&grammar, output)?;
    Ok(())
}

#[allow(dead_code)]
fn grammar1() -> Grammar {
    // E ::= E '+' E | 'a'
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("E" => priority_level!(
                alternative!(id!("E"), lit!("+"), id!("E")),
                alternative!(lit!("a"))
            ))
        ]
    )
    .into()
}

#[allow(dead_code)]
fn test_grammar() -> Grammar {
    // A ::= A 'a' | 'a'
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("A" => priority_level!(
                alternative!(id!("A"), lit!("a")),
                alternative!(lit!("a"))
            ))
        ]
    )
    .into()
}

#[allow(dead_code)]
fn grammar2() -> Grammar {
    // P -> S+
    // S -> E ";"
    // E -> E '+' E | 'a'
    // WS -> ' '*
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("P" => priority_level!(
                alternative!(plus!(id!("S")))
            )),
            syntax_rule!("S" => priority_level!(
                alternative!(id!("E"))
            )),
            syntax_rule!("E" => priority_level!(
                alternative!(id!("E"), lit!("+"), id!("E")),
                alternative!(lit!("a"))
            ))
        ],
        lexical: [
            lexical_rule!("WS" => Regex::star(Regex::Char(' ')))
        ],
        layout: [
            Terminal::new("WS")
        ]
    )
    .into()
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

// Grammar
//   : "grammar" Identifier ";" Rule*
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
fn iggy() -> Grammar {
    grammar_def!("Iggy",
        syntax: [
            // Grammar : "grammar" Identifier ";" Rule+
            syntax_rule!("Grammar" => priority_level!(
                alternative!(
                    lit!("grammar"),
                    id!("Identifier"),
                    lit!(";"),
                    plus!(id!("Rule"))
                )
            )),
            // Rule : Identifier ":" Identifier+ ";"
            syntax_rule!("Rule" => priority_level!(
                alternative!(
                    id!("Identifier"),
                    lit!(":"),
                    plus!(id!("Identifier")),
                    lit!(";")
                )
            ))
        ],
        lexical: [
            // regex Identifier : [a-zA-Z_][a-zA-Z_0-9]*
            lexical_rule!("Identifier" => Regex::Seq(vec![
                    Regex::Alt(vec![
                        Regex::CharRange { start: 'a', end: 'z' },
                        Regex::CharRange { start: 'A', end: 'Z' },
                        Regex::Char('_'),
                    ]),
                    Regex::Star(Box::new(Regex::Alt(vec![
                        Regex::CharRange { start: 'a', end: 'z' },
                        Regex::CharRange { start: 'A', end: 'Z' },
                        Regex::CharRange { start: '0', end: '9' },
                        Regex::Char('_'),
                    ]))),
                ])),
            // WS : [ \n]*
            lexical_rule!("WS" => Regex::star(Regex::Alt(vec![Regex::Char(' '), Regex::Char('\n')])))
        ],
        layout: [
            Terminal::new("WS")
        ]
    )
    .into()
}
