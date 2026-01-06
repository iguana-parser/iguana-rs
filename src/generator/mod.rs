use std::{fs, io::Write, path::Path};

use proc_macro2::TokenStream;

use crate::{
    generator::{
        id::{NonterminalIds, SlotIds, TerminalIds},
        utils::rustfmt,
    },
    grammar::def::Grammar,
};

mod cargo_toml_gen;
mod id;
mod lib_gen;
mod main_gen;
mod parse_tree_gen;
mod parser_gen;
mod scanner_gen;
mod types_gen;
mod utils;

enum FileFormat {
    Rust,
    Toml,
}

pub fn generate(grammar: &Grammar, output_dir: &Path) -> std::io::Result<()> {
    let mut nonterminal_ids = NonterminalIds::default();
    for nonterminal in grammar.nonterminals() {
        nonterminal_ids.insert(nonterminal.clone());
    }
    let mut terminal_ids = TerminalIds::default();
    for terminal in grammar.terminals() {
        terminal_ids.insert(terminal.clone());
    }
    let mut slot_ids = SlotIds::new(grammar);

    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    write_file(
        cargo_toml_gen::generate(grammar),
        &output_dir.join("Cargo.toml"),
        FileFormat::Toml,
    )?;

    let src_dir = output_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir(&src_dir)?;
    }
    write_file(
        to_string(lib_gen::generate()),
        &src_dir.join("lib.rs"),
        FileFormat::Rust,
    )?;
    let parser_code = parser_gen::generate(
        grammar,
        &mut nonterminal_ids,
        &mut slot_ids,
        &mut terminal_ids,
    );
    write_file(
        to_string(parser_code),
        &src_dir.join("parser.rs"),
        FileFormat::Rust,
    )?;

    let scanner_code = scanner_gen::generate(grammar, &terminal_ids);
    write_file(
        to_string(scanner_code),
        &src_dir.join("scanner.rs"),
        FileFormat::Rust,
    )?;

    let parse_tree_code =
        parse_tree_gen::generate(grammar, &nonterminal_ids, &terminal_ids, &slot_ids);
    write_file(
        to_string(parse_tree_code),
        &src_dir.join("parse_tree.rs"),
        FileFormat::Rust,
    )?;

    let types_code = types_gen::generate();
    write_file(
        to_string(types_code),
        &src_dir.join("types.rs"),
        FileFormat::Rust,
    )?;

    let main_code = main_gen::generate(grammar);
    write_file(
        to_string(main_code),
        &src_dir.join("main.rs"),
        FileFormat::Rust,
    )?;

    Ok(())
}

fn write_file(content: impl AsRef<str>, path: &Path, format: FileFormat) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    let formatted = match format {
        FileFormat::Rust => rustfmt(content.as_ref()),
        _ => content.as_ref().into(),
    };
    file.write_all(formatted.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn to_string(tokens: TokenStream) -> String {
    let syntax = syn::parse_file(&tokens.to_string()).unwrap_or_else(|e| {
        panic!("Parse error at {:?}: {}", e.span().start(), e);
    });
    prettyplease::unparse(&syntax)
}
