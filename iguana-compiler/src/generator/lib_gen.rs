use std::{fs, io, path::Path};

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    generator::grammar_utils::{grammar_ident, parser_ident},
    grammar::def::Grammar,
};

/// Generates `lib.rs`: the module list and the crate-root re-exports. The
/// root imports nothing else, so a nonterminal may carry any name the runtime
/// uses, such as `Input` or `Parser`, without shadowing it here.
///
/// `highlighting_rules` is the file with the language's highlighting rules.
pub fn generate(
    grammar: &Grammar,
    src_dir: &Path,
    highlighting_rules: Option<&Path>,
) -> io::Result<TokenStream> {
    let grammar_type = grammar_ident(&grammar.name);
    let parser = parser_ident(&grammar.name);
    let highlighting = highlighting_module(src_dir, highlighting_rules)?;

    Ok(quote! {
        pub mod grammar;
        #highlighting
        pub mod parser;
        pub mod parse_tree;
        pub mod scanner;

        pub use grammar::#grammar_type;
        pub use iguana_runtime::grammar::Grammar;
        pub use iguana_runtime::result::{ParseError, ParseSuccess};
        pub use parser::#parser;
    })
}

/// The module `highlighting`, if the language has highlighting rules: the
/// crate's own `src/highlighting.rs`, or, in a crate generated outside the
/// grammar's crate such as a WebAssembly bundle, the grammar's
/// `src/highlighting.rs` by its absolute path.
fn highlighting_module(
    src_dir: &Path,
    highlighting_rules: Option<&Path>,
) -> io::Result<TokenStream> {
    let Some(rules) = highlighting_rules else {
        return Ok(TokenStream::new());
    };
    if src_dir.join("highlighting.rs").is_file() {
        return Ok(quote! { pub mod highlighting; });
    }
    let path = fs::canonicalize(rules)?.to_string_lossy().into_owned();
    Ok(quote! {
        #[path = #path]
        pub mod highlighting;
    })
}
