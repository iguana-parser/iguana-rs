use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use iguana::{
    alt, alternative,
    generator::generate,
    grammar::{def::Grammar, regex::Regex, symbols::Terminal},
    grammar_def, group, id, lexical_rule, lit, opt, plus, priority_level, star, syntax_rule,
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
            syntax_rule!("P" => alternative!(plus!(id!("S")))),
            syntax_rule!("S" => alternative!(id!("E"))),
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

#[allow(dead_code)]
fn grammar4() -> Grammar {
    // S : A+
    // A : 'a'
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(plus!(id!("A")))),
            syntax_rule!("A" => alternative!(lit!("a")))
        ]
    )
    .into()
}

#[allow(dead_code)]
fn grammar5() -> Grammar {
    // S : (A B C)+
    // A : 'a'
    // B: 'b'
    // C : 'c'
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(plus!(group!(id!("A"), id!("B"), id!("C"))))),
            syntax_rule!("A" => alternative!(lit!("a"))),
            syntax_rule!("B" => alternative!(lit!("b"))),
            syntax_rule!("C" => alternative!(lit!("c")))
        ]
    )
    .into()
}

fn star_grammar() -> Grammar {
    // S : A*
    // A : "a"
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(star!(id!("A")))),
            syntax_rule!("A" => alternative!(lit!("a")))
        ]
    )
    .into()
}

fn star_with_sep() -> Grammar {
    // S : {A ","}*
    // A : "a"
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(star!(id!("A"), lit!(",")))),
            syntax_rule!("A" => alternative!(lit!("a")))
        ]
    )
    .into()
}

fn plus_with_sep() -> Grammar {
    // S : {A ","}+
    // A : "a"
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(plus!(id!("A"), lit!(",")))),
            syntax_rule!("A" => alternative!(lit!("a")))
        ]
    )
    .into()
}

fn group() -> Grammar {
    // A: (B C D);
    // B: 'b'
    // C: 'c'
    // D: 'd'
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("A" => alternative!(group!(id!("B"), id!("C"), id!("D")))),
            syntax_rule!("B" => alternative!(lit!("b"))),
            syntax_rule!("C" => alternative!(lit!("c"))),
            syntax_rule!("D" => alternative!(lit!("d"))),
        ]
    )
    .into()
}

fn simple_alt() -> Grammar {
    // A: B (C | D);
    // B: 'b'
    // C: 'c'
    // D: 'd'
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("A" => alternative!(id!("B"), alt!(id!("C"), id!("D")))),
            syntax_rule!("B" => alternative!(lit!("b"))),
            syntax_rule!("C" => alternative!(lit!("c"))),
            syntax_rule!("D" => alternative!(lit!("d"))),
        ]
    )
    .into()
}

fn empty() -> Grammar {
    // A: ;
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("A" => alternative!()),
        ]
    )
    .into()
}

fn opt() -> Grammar {
    // S: A?;
    // A: "a";
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("S" => alternative!(opt!(id!("A")))),
            syntax_rule!("A" => alternative!(lit!("a")))
        ]
    )
    .into()
}

// Grammar
//   : "grammar" Identifier ";" Rule*
//   ;
// Rule
//   : Identifier ":" {PriorityLevel ">"}* ";"
//   ;
// PriorityLevel
//   : Alternative? ("|" Alternative)*
//   ;
// Alternative:
//   : Symbol*
//   ;
// Symbol
//   : Identifier
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
            // Grammar : "grammar" Identifier ";" Rule*
            syntax_rule!("Grammar" => alternative!(
                lit!("grammar"),
                id!("Identifier"),
                lit!(";"),
                star!(id!("Rule"))
            )),
            // Rule : Identifier ":" PriorityLevel? (">" PriorityLevel)* ";"
            syntax_rule!("Rule" => alternative!(
                id!("Identifier"),
                lit!(":"),
                star!(id!("PriorityLevel"), lit!(">")),
                lit!(";")
            )),
            // PriorityLevel : Alternative? ("|" Alternative)*
            syntax_rule!("PriorityLevel" => alternative!(
                opt!(id!("Alternative")),
                star!(group!(lit!("|"), id!("Alternative")))
            )),
            // Alternative : Symbol*
            syntax_rule!("Alternative" => alternative!(
                star!(id!("Symbol"))
            )),
            // Symbol : Identifier
            syntax_rule!("Symbol" => alternative!(
                id!("Identifier")
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
