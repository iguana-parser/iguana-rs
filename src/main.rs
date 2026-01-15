use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use iguana::{
    alt, alternative, c, cc,
    generator::generate,
    grammar::{def::Grammar, symbols::Terminal},
    grammar_def, group, id, lexical_rule, lit, opt, plus, priority_level, r_alt, r_seq, r_star,
    star, syntax_rule,
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
            lexical_rule!("WS" => r_star!(c!(' ')))
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

// grammar Iggy
//
// Grammar
//   = "grammar" Identifier Rule*
//
// Rule
//   = Identifier "=" {PriorityLevel ">"}*
//   | Identifier "=" "/" { Regex+ "|" }+ "/"
//
// PriorityLevel
//   = { Alternative "|" }*
//
// Alternative
//   = Symbol*
//
// Symbol
//   = Symbol "*"
//   | Symbol "+"
//   | "(" Symbol "|" Symbol ")"
//   | "\"" String "\""
//   | "{" Symbol Symbol "}" "*"
//   | "{" Symbol Symbol "}" "+"
//   | "(" Symbol+ ")"
//   | Identifier
//
// Regex
//   = Regex+
//   | Regex*
//   | Regex?
//   | "(" { Regex+ "|" }* ")"
//   | CharClass
//   | Char
//
// CharClass
//   = "!"? "[" (CharRange | Char)+ "]"
//
// CharRange
//   = Char "-" Char
//
// Char
//   = /!['-[]*+?/]/
//
// String
//  = /!["]*/
//
// Identifier
//   = /[a-zA-Z_][a-zA-Z_0-9]*/
//
// WS
//   = /[ \n]*/
//
fn iggy() -> Grammar {
    grammar_def!("Iggy",
        syntax: [
            // Grammar = "grammar" Identifier Rule*
            syntax_rule!("Grammar" => alternative!(
                lit!("grammar"),
                id!("Identifier"),
                star!(id!("Rule"))
            )),
            // Rule = Identifier "=" {PriorityLevel ">"}*
            //      | Identifier "=" "/" { Regex+ "|" }+ "/"
            syntax_rule!("Rule" => priority_level!(
                alternative!(id!("Identifier"), lit!("="), star!(id!("PriorityLevel"), lit!(">"))),
                alternative!(id!("Identifier"), lit!("="), lit!("/"), plus!(plus!(id!("Regex")), lit!("|")), lit!("/"))
            )),
            // PriorityLevel = { Alternative "|" }*
            syntax_rule!("PriorityLevel" => alternative!(
                star!(id!("Alternative"), lit!("|"))
            )),
            // Alternative = Symbol*
            syntax_rule!("Alternative" => alternative!(
                star!(id!("Symbol"))
            )),
            // Symbol
            //   = Symbol "*"
            //   | Symbol "+"
            //   | "(" Symbol "|" Symbol ")"
            //   | "\"" String "\""
            //   | "{" Symbol Symbol "}" "*"
            //   | "{" Symbol Symbol "}" "+"
            //   | "(" Symbol+ ")"
            //   | Identifier
            syntax_rule!("Symbol" => priority_level!(
                alternative!(id!("Symbol"), lit!("*")),
                alternative!(id!("Symbol"), lit!("+")),
                alternative!(lit!("("), id!("Symbol"), lit!("|"), id!("Symbol"), lit!(")")),
                alternative!(lit!("\""), id!("String"), lit!("\"")),
                alternative!(lit!("{"), id!("Symbol"), id!("Symbol"), lit!("}"), lit!("*")),
                alternative!(lit!("{"), id!("Symbol"), id!("Symbol"), lit!("}"), lit!("+")),
                alternative!(lit!("("), star!(id!("Symbol")), lit!(")")),
                alternative!(id!("Identifier")),
            )),
            // Regex
            //   = Regex+
            //   | Regex*
            //   | Regex?
            //   | "(" { Regex+ "|" }* ")"
            //   | CharClass
            //   | Char
            syntax_rule!("Regex" => priority_level!(
                alternative!(id!("Regex"), lit!("+")),
                alternative!(id!("Regex"), lit!("*")),
                alternative!(id!("Regex"), lit!("?")),
                alternative!(lit!("("), star!(plus!(id!("Regex")), lit!("|")), lit!(")")),
                alternative!(id!("CharClass")),
                alternative!(id!("Char"))
            )),
            // CharClass = "!"? "[" (CharRange | Char)+ "]"
            syntax_rule!("CharClass" => alternative!(
                opt!(lit!("!")),
                lit!("["),
                plus!(alt!(id!("CharRange"), id!("Char"))),
                lit!("]")
            )),
            // CharRange = Char "-" Char
            syntax_rule!("CharRange" => alternative!(
                id!("Char"),
                lit!("-"),
                id!("Char")
            )),
        ],
        lexical: [
            // Identifier = /[a-zA-Z_][a-zA-Z_0-9]*/
            lexical_rule!("Identifier" => r_seq!(
                r_alt!(cc!(['a'-'z', 'A'-'Z']), c!('_')),
                r_star!(r_alt!(cc!(['a'-'z', 'A'-'Z', '0'-'9']), c!('_')))
            )),
            // String = /!["]*/
            lexical_rule!("String" => r_star!(cc!(!['"'-'"']))),
            // Char = /!['-[]*+?/]/
            lexical_rule!("Char" => cc!(![
                '\''-'\'', 
                '-'-'-', 
                '['-'[', 
                ']'-']',
                '*'-'*',
                '+'-'+',
                '?'-'?',
                '/'-'/', 
            ])),
            // WS = /[ \n]*/
            lexical_rule!("WS" => r_star!(r_alt!(c!(' '), c!('\n'))))
        ],
        layout: [
            Terminal::new("WS")
        ]
    )
    .into()
}
