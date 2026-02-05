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
    /// Test-related commands
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
}

#[derive(Subcommand)]
enum TestCommands {
    /// Initialize a new grammar test
    Init {
        /// Name of the grammar (creates tests/<name>/)
        name: String,
    },
    /// Delete a grammar test
    #[command(alias = "rm")]
    Delete {
        /// Name of the grammar test to delete
        name: String,
    },
    /// Generate all test parsers
    GenerateAll,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate { grammar, output } => generate_parser(grammar.as_deref(), &output)?,
        Commands::Run => todo!(),
        Commands::Test { command } => match command {
            TestCommands::Init { name } => init_test(&name)?,
            TestCommands::Delete { name } => delete_test(&name)?,
            TestCommands::GenerateAll => generate_all_tests()?,
        },
    }
    Ok(())
}

fn init_test(name: &str) -> std::io::Result<()> {
    use std::io::Write;

    let test_dir = PathBuf::from("tests").join(name);
    let grammar_name = to_pascal_case(name);
    let grammar_file = test_dir.join(format!("{}.iggy", name));
    let cargo_toml = test_dir.join("Cargo.toml");
    let tests_rs = test_dir.join("tests.rs");

    // Create directory and grammar file if they don't exist
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)?;
        std::fs::write(&grammar_file, format!("grammar {grammar_name}\n"))?;
        println!("Created grammar: {}", grammar_file.display());
    }

    // Generate parser if not yet generated
    if !cargo_toml.exists() {
        print!("Generating parser... ");
        std::io::stdout().flush()?;
        generate_parser(Some(&grammar_file), &test_dir)?;
        println!("done");
    }

    // Add test infrastructure
    std::fs::create_dir_all(test_dir.join("parse_trees"))?;

    // Add [[test]] section to Cargo.toml if not present
    let cargo_content = std::fs::read_to_string(&cargo_toml)?;
    if !cargo_content.contains("[[test]]") {
        let updated = cargo_content.replace(
            "[features]",
            "[[test]]\nname = \"tests\"\npath = \"tests.rs\"\n\n[features]",
        );
        std::fs::write(&cargo_toml, updated)?;
    }

    // Create tests.rs if not present
    if !tests_rs.exists() {
        let tests_content = format!(
            r#"use {name}::{{parse, parse_tree::to_sexpr}};
use iguana_runtime::testing::{{check_golden_file, golden_path}};

fn check(start_nonterminal: &str, input: &str, test_name: &str) {{
    let tree = parse(input, start_nonterminal).expect("Parse failed");
    let actual = to_sexpr(tree.as_parse_tree_ref());
    check_golden_file(&actual, &golden_path(env!("CARGO_MANIFEST_DIR"), test_name));
}}

#[test]
fn test_example() {{
    // check("Start", "input", "example");
}}
"#
        );
        std::fs::write(&tests_rs, tests_content)?;
    }

    // Add to workspace members if workspace Cargo.toml exists
    let workspace_cargo = PathBuf::from("Cargo.toml");
    if workspace_cargo.exists() {
        let content = std::fs::read_to_string(&workspace_cargo)?;
        let member_entry = format!("tests/{name}");
        if content.contains("[workspace]") && !content.contains(&member_entry) {
            // Find members array and add the new member
            if let Some(members_start) = content.find("members = [") {
                let before_bracket = &content[..members_start + 11];
                let after_start = &content[members_start + 11..];
                if let Some(bracket_end) = after_start.find(']') {
                    let members_content = &after_start[..bracket_end];
                    let after_bracket = &after_start[bracket_end..];

                    // Add new member
                    let new_content = format!(
                        "{}\"{}\", {}{}",
                        before_bracket,
                        member_entry,
                        members_content.trim_start(),
                        after_bracket
                    );
                    std::fs::write(&workspace_cargo, new_content)?;
                }
            }
        }
    }

    println!();
    println!("To regenerate parser:  cargo run -p iguana -- generate --grammar {} --output {}", grammar_file.display(), test_dir.display());
    println!("To update golden files: REGENERATE=1 cargo test -p {name}");

    Ok(())
}

fn delete_test(name: &str) -> std::io::Result<()> {
    let test_dir = PathBuf::from("tests").join(name);

    // Remove the directory
    if test_dir.exists() {
        std::fs::remove_dir_all(&test_dir)?;
        println!("Deleted: tests/{name}/");
    } else {
        println!("Test not found: tests/{name}/");
    }

    // Remove from workspace members
    let workspace_cargo = PathBuf::from("Cargo.toml");
    if workspace_cargo.exists() {
        let content = std::fs::read_to_string(&workspace_cargo)?;
        let member_entry = format!("\"tests/{name}\"");
        if content.contains(&member_entry) {
            // Remove the member entry (with trailing comma and space, or just the entry)
            let new_content = content
                .replace(&format!("{}, ", member_entry), "")
                .replace(&format!(", {}", member_entry), "")
                .replace(&member_entry, "");
            std::fs::write(&workspace_cargo, new_content)?;
        }
    }

    Ok(())
}

fn generate_all_tests() -> std::io::Result<()> {
    use std::io::Write;

    let tests_dir = PathBuf::from("tests");
    if !tests_dir.exists() {
        println!("No tests directory found");
        return Ok(());
    }

    for entry in std::fs::read_dir(&tests_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_string_lossy();
            let grammar_file = path.join(format!("{}.iggy", name));
            if grammar_file.exists() {
                print!("Generating {}... ", name);
                std::io::stdout().flush()?;
                generate_parser(Some(&grammar_file), &path)?;
                println!("done");
            }
        }
    }

    Ok(())
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
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
//   = "grammar" name:Identifier LayoutDef? SyntaxRule* RegexBlock?
//
// LayoutDef
//   = "layout" Identifier*
//
// SyntaxRule
//   = head:Identifier "=" {PriorityLevel ">"}*
//
// PriorityLevel
//   = { Alternative "|" }*
//
// Alternative
//   = Symbol* Label?
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
//   | label:Identifier ":" Symbol              @Labeled
//   | Identifier                               @Identifier
//
// RegexBlock
//   = regex "{" RegexRule* "}"
//
// RegexRule
//   = Identifier "=" body:{ Regex+ "|" }+
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
//   = neg:"!"? "[" ranges:RangeElement+ "]"
//
// RangeElement
//   = Range 
//   | RangeChar
//
// Range
//   = start:RangeChar "-" end:RangeChar
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
            // Grammar = "grammar" name:Identifier LayoutDef? SyntaxRule* RegexBlock?
            syntax_rule!("Grammar" => alternative!(
                lit!("grammar"),
                labeled!("name", id!("Identifier")),
                opt!(id!("LayoutDef")),
                star!(id!("SyntaxRule")),
                opt!(id!("RegexBlock"))
            )),
            // LayoutDef = "layout" Identifier*
            syntax_rule!("LayoutDef" => alternative!(
                lit!("layout"),
                star!(id!("Identifier"))
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
            // RegexRule = Identifier "=" body:{ Regex+ "|" }+
            syntax_rule!("RegexRule" => alternative!(
                id!("Identifier"),
                lit!("="),
                labeled!("body", plus!(plus!(id!("Regex")), lit!("|")))
            )),
            // PriorityLevel = { Alternative "|" }*
            syntax_rule!("PriorityLevel" => alternative!(
                star!(id!("Alternative"), lit!("|"))
            )),
            // Alternative = Symbol* Label?
            syntax_rule!("Alternative" => alternative!(
                star!(id!("Symbol")),
                opt!(id!("Label"))
            )),
            // Symbol
            //   = Symbol "*"                               @Star
            //   | Symbol "+"                               @Plus
            //   | Symbol "?"                               @Opt
            //   | "(" first:Symbol ("|" Symbol)+ ")"  @Alt
            //   | "\"" String "\""                         @Lit
            //   | "{" symbol:Symbol sep:Symbol "}" "*"     @StarSep
            //   | "{" symbol:Symbol sep:Symbol "}" "+"     @PlusSep
            //   | "(" Symbol+ ")"                          @Group
            //   | Identifier                               @Identifier
            syntax_rule!("Symbol" => priority_level!(
                alternative!(id!("Symbol"), lit!("*"), @"Star"),
                alternative!(id!("Symbol"), lit!("+"), @"Plus"),
                alternative!(id!("Symbol"), lit!("?"), @"Opt"),
                alternative!(lit!("("), labeled!("first", id!("Symbol")), plus!(group!(lit!("|"), id!("Symbol"))), lit!(")"), @"Alt"),
                alternative!(lit!("\""), id!("String"), lit!("\""), @"Lit"),
                alternative!(lit!("{"), labeled!("symbol", id!("Symbol")), labeled!("sep", id!("Symbol")), lit!("}"), lit!("*"), @"StarSep"),
                alternative!(lit!("{"), labeled!("symbol", id!("Symbol")), labeled!("sep", id!("Symbol")), lit!("}"), lit!("+"), @"PlusSep"),
                alternative!(lit!("("), plus!(id!("Symbol")), lit!(")"), @"Group"),
                alternative!(labeled!("label", id!("Identifier")), lit!(":"), id!("Symbol"), @"Labeled"),
                alternative!(id!("Identifier"), @"Identifier"),
            )),
            // Regex
            //   = Regex "+"                                 @Plus
            //   | Regex "*"                                 @Star
            //   | Regex "?"                                 @Opt
            //   | "(" first:Regex ("|" Regex)+ ")"     @Alt
            //   | "(" Regex+ ")"                            @Group
            //   | CharClass                                 @CharClass
            //   | "\"" Char "\""                            @Char
            syntax_rule!("Regex" => priority_level!(
                alternative!(id!("Regex"), lit!("+"), @"Plus"),
                alternative!(id!("Regex"), lit!("*"), @"Star"),
                alternative!(id!("Regex"), lit!("?"), @"Opt"),
                alternative!(lit!("("), labeled!("first", id!("Regex")), plus!(group!(lit!("|"), id!("Regex"))), lit!(")"), @"Alt"),
                alternative!(lit!("("), plus!(id!("Regex")), lit!(")"), @"Group"),
                alternative!(id!("CharClass"), @"CharClass"),
                alternative!(lit!("\""), id!("Char"), lit!("\""), @"Char")
            )),
            // CharClass = "!"? "[" RangeElement+ "]"
            syntax_rule!("CharClass" => alternative!(
                opt!(lit!("!")),
                lit!("["),
                plus!(id!("RangeElement")),
                lit!("]")
            )),
            // RangeElement = Range | RangeChar
            syntax_rule!("RangeElement" => priority_level!(
                alternative!(id!("Range")),
                alternative!(id!("RangeChar"))
            )),
            // Range = start:RangeChar "-" end:RangeChar
            syntax_rule!("Range" => alternative!(
                labeled!("start", id!("RangeChar")),
                lit!("-"),
                labeled!("end", id!("RangeChar"))
            )),
        ],
        lexical: [
            // Identifier = /[a-zA-Z_][a-zA-Z_0-9]*/
            lexical_rule!("Identifier" => r_seq!(
                cc!(['a'-'z', 'A'-'Z', '_'-'_']),
                r_star!(cc!(['a'-'z', 'A'-'Z', '_'-'_', '0'-'9']))
            )),
            // String = (("\\" [' " \\ t f r n]) | !['"\\])*
            lexical_rule!("String" => r_star!(r_alt!(
                r_seq!(c!('\\'), cc!(['\''-'\'', '"'-'"', '\\'-'\\', 't'-'t', 'f'-'f', 'r'-'r', 'n'-'n'])),
                cc!(!['\''-'\'', '"'-'"', '\\'-'\\'])
            ))),
            // RangeChar = ![\\ \- \[ \] \t \f \r \n \ ] | "\\" [\\ \- \[ \] t f r n \ ]
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
                    ' '-' ',
                ]),
                r_seq!(c!('\\'), cc!(['\\'-'\\', '-'-'-', '['-'[', ']'-']', 't'-'t', 'f'-'f', 'r'-'r', 'n'-'n', ' '-' ']))
            )),
            // Char = "\\" [' " \\ t f r n] | !['"\\]
            lexical_rule!("Char" => r_alt!(
                r_seq!(c!('\\'), cc!(['\''-'\'', '"'-'"', '\\'-'\\', 't'-'t', 'f'-'f', 'r'-'r', 'n'-'n'])),
                cc!(!['\''-'\'', '"'-'"', '\\'-'\\'])
            )),
            // Label = "@" [a-zA-Z_][a-zA-Z_0-9]*
            lexical_rule!("Label" => r_seq!(
                c!('@'),
                r_alt!(cc!(['a'-'z', 'A'-'Z']), c!('_')),
                r_star!(r_alt!(cc!(['a'-'z', 'A'-'Z', '0'-'9']), c!('_')))
            )),
            // WS = [ \n]*
            lexical_rule!("WS" => r_star!(r_alt!(c!(' '), c!('\n'))))
        ],
        layout: [
            Terminal::new("WS")
        ]
    )
}
