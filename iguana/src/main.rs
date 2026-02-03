use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use iguana::{
    alt, alternative, c, cc,
    generator::generate,
    grammar::{
        def::{Grammar, GrammarDef},
        symbols::Terminal,
    },
    grammar_def, group, id,
    iggy::parse_grammar,
    labeled, lexical_rule, lit, opt, plus, priority_level, r_alt, r_seq, r_star, star, syntax_rule,
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
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate { grammar, output } => generate_parser(grammar.as_deref(), &output)?,
        Commands::Run => todo!(),
    }
    Ok(())
}

fn generate_parser(grammar_path: Option<&Path>, output: &Path) -> std::io::Result<()> {
    let grammar = match grammar_path {
        Some(path) => {
            let source = std::fs::read_to_string(path)?;
            parse_grammar(&source).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        }
        None => iggy(),
    };
    generate(&grammar.into(), output)?;
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

fn expression_grammar() -> Grammar {
    // E
    //  = E '*' E
    //  | E '+' E
    //  | 'a'
    grammar_def!("Test2",
        syntax: [
            syntax_rule!("E" => priority_level!(
                alternative!(id!("E"), lit!("*"), id!("E"), @"Mul"),
                alternative!(id!("E"), lit!("+"), id!("E"), @"Add"),
                alternative!(lit!("a"), @"Lit")
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
//   = "grammar" name:Identifier SyntaxRule* RegexBlock?
//
// SyntaxRule
//   = head:Identifier "=" {PriorityLevel ">"}*
//
// PriorityLevel
//   = { Alternative "|" }*
//
// Alternative
//   = Symbol*
//
// Symbol
//   = Symbol "*"                               @Star
//   | Symbol "+"                               @Plus
//   | Symbol "?"                               @Opt
//   | "(" first:Symbol rest:("|" Symbol)+ ")"  @Alt
//   | "\"" String "\""                         @Lit
//   | "{" symbol:Symbol sep:Symbol "}" "*"     @StarSep
//   | "{" symbol:Symbol sep:Symbol "}" "+"     @PlusSep
//   | "(" Symbol+ ")"                          @Group
//   | Identifier                               @Identifier
//
// RegexBlock
//   = regex "{" RegexRule* "}"
//
// RegexRule
//   = Identifier "=" { Regex+ "|" }+
//
// Regex
//   = Regex "+"                                 @Plus
//   | Regex "*"                                 @Star
//   | Regex "?"                                 @Opt
//   | "(" first:Regex rest:("|" Regex)+ ")"     @Alt
//   | "(" Regex+ ")"                            @Group
//   | CharClass                                 @CharClass
//   | "\"" Char "\""                            @Char
//
// CharClass
//   = "!"? "[" (Range | RangeChar)+ "]"
//
// Range
//   = RangeChar "-" RangeChar
//
// regex {
//   RangeChar
//     = ![\\ \- \[ \] \t \f \r \n \ ]
//     | "\\" [\\ \- \[ \] t f r n \ ]
//
//   Char
//     = "\\" [' " \\ t f r n]
//     | !['"\\]
//
//   String = (("\\" [' " \\ t f r n]) | !['"\\])*
//   Identifier = [a-zA-Z_][a-zA-Z_0-9]*
//   WS = [\ \n]*
// }
//
fn iggy() -> GrammarDef {
    grammar_def!("Iggy",
        syntax: [
            // Grammar = "grammar" name:Identifier SyntaxRule* RegexBlock?
            syntax_rule!("Grammar" => alternative!(
                lit!("grammar"),
                labeled!("name", id!("Identifier")),
                star!(id!("SyntaxRule")),
                opt!(id!("RegexBlock"))
            )),
            // SyntaxRule = head:Identifier "=" {PriorityLevel ">"}*
            syntax_rule!("SyntaxRule" => alternative!(
                labeled!("head", id!("Identifier")),
                lit!("="),
                star!(id!("PriorityLevel"), lit!(">"))
            )),
            // RegexBlock = "regex" "{" RegexRule* "}"
            syntax_rule!("RegexBlock" => alternative!(
                lit!("regex"),
                lit!("{"),
                star!(id!("RegexRule")),
                lit!("}")
            )),
            // RegexRule = Identifier "=" { Regex+ "|" }+
            syntax_rule!("RegexRule" => alternative!(
                id!("Identifier"),
                lit!("="),
                plus!(plus!(id!("Regex")), lit!("|"))
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
            //   = Symbol "*"                               @Star
            //   | Symbol "+"                               @Plus
            //   | Symbol "?"                               @Opt
            //   | "(" first:Symbol rest:("|" Symbol)+ ")"  @Alt
            //   | "\"" String "\""                         @Lit
            //   | "{" symbol:Symbol sep:Symbol "}" "*"     @StarSep
            //   | "{" symbol:Symbol sep:Symbol "}" "+"     @PlusSep
            //   | "(" Symbol+ ")"                          @Group
            //   | Identifier                               @Identifier
            syntax_rule!("Symbol" => priority_level!(
                alternative!(id!("Symbol"), lit!("*"), @"Star"),
                alternative!(id!("Symbol"), lit!("+"), @"Plus"),
                alternative!(id!("Symbol"), lit!("?"), @"Opt"),
                alternative!(lit!("("), labeled!("first", id!("Symbol")), labeled!("rest", plus!(group!(lit!("|"), id!("Symbol")))), lit!(")"), @"Alt"),
                alternative!(lit!("\""), id!("String"), lit!("\""), @"Lit"),
                alternative!(lit!("{"), labeled!("symbol", id!("Symbol")), labeled!("sep", id!("Symbol")), lit!("}"), lit!("*"), @"StarSep"),
                alternative!(lit!("{"), labeled!("symbol", id!("Symbol")), labeled!("sep", id!("Symbol")), lit!("}"), lit!("+"), @"PlusSep"),
                alternative!(lit!("("), plus!(id!("Symbol")), lit!(")"), @"Group"),
                alternative!(id!("Identifier"), @"Identifier"),
            )),
            // Regex
            //   = Regex "+"                                 @Plus
            //   | Regex "*"                                 @Star
            //   | Regex "?"                                 @Opt
            //   | "(" first:Regex rest:("|" Regex)+ ")"     @Alt
            //   | "(" Regex+ ")"                            @Group
            //   | CharClass                                 @CharClass
            //   | "\"" Char "\""                            @Char
            syntax_rule!("Regex" => priority_level!(
                alternative!(id!("Regex"), lit!("+"), @"Plus"),
                alternative!(id!("Regex"), lit!("*"), @"Star"),
                alternative!(id!("Regex"), lit!("?"), @"Opt"),
                alternative!(lit!("("), labeled!("first", id!("Regex")), labeled!("rest", plus!(group!(lit!("|"), id!("Regex")))), lit!(")"), @"Alt"),
                alternative!(lit!("("), plus!(id!("Regex")), lit!(")"), @"Group"),
                alternative!(id!("CharClass"), @"CharClass"),
                alternative!(lit!("\""), id!("Char"), lit!("\""), @"Char")
            )),
            // CharClass = "!"? "[" (Range | RangeChar)+ "]"
            syntax_rule!("CharClass" => alternative!(
                opt!(lit!("!")),
                lit!("["),
                plus!(alt!(id!("Range"), id!("RangeChar"))),
                lit!("]")
            )),
            // Range = RangeChar "-" RangeChar
            syntax_rule!("Range" => alternative!(
                id!("RangeChar"),
                lit!("-"),
                id!("RangeChar")
            )),
        ],
        lexical: [
            // Identifier = /[a-zA-Z_][a-zA-Z_0-9]*/
            lexical_rule!("Identifier" => r_seq!(
                r_alt!(cc!(['a'-'z', 'A'-'Z']), c!('_')),
                r_star!(r_alt!(cc!(['a'-'z', 'A'-'Z', '0'-'9']), c!('_')))
            )),
            // String = (("\\" [' " \\ t f r n]) | !['"\\])*
            lexical_rule!("String" => r_star!(r_alt!(
                r_seq!(c!('\\'), cc!(['\''-'\'', '"'-'"', '\\'-'\\', 't'-'t', 'f'-'f', 'r'-'r', 'n'-'n'])),
                cc!(!['\''-'\'', '"'-'"', '\\'-'\\'])
            ))),
            // RangeChar = ![\\ \- \[ \] \t \f \r \n] | "\\" [\\ \- \[ \] t f r n]
            lexical_rule!("RangeChar" => r_alt!(
                cc!(![
                    '\\'-'\\',
                    '-'-'-',
                    '['-'[',
                    ']'-']',
                    '\t'-'\t',
                    '\x0c'-'\x0c',
                    '\r'-'\r',
                    '\n'-'\n',
                ]),
                r_seq!(c!('\\'), cc!(['\\'-'\\', '-'-'-', '['-'[', ']'-']', 't'-'t', 'f'-'f', 'r'-'r', 'n'-'n']))
            )),
            // Char = "\\" [' " \\ t f r n] | !['"\\]
            lexical_rule!("Char" => r_alt!(
                r_seq!(c!('\\'), cc!(['\''-'\'', '"'-'"', '\\'-'\\', 't'-'t', 'f'-'f', 'r'-'r', 'n'-'n'])),
                cc!(!['\''-'\'', '"'-'"', '\\'-'\\'])
            )),
            // WS = [ \n]*
            lexical_rule!("WS" => r_star!(r_alt!(c!(' '), c!('\n'))))
        ],
        layout: [
            Terminal::new("WS")
        ]
    )
}
