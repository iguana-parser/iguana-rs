use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{generator::grammar_utils::parser_ident, grammar::def::Grammar, utils::to_snake_case};

/// Generates `main.rs`: a stub that hands the process to the runtime's
/// command-line interface with the crate's parser type.
pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", to_snake_case(&grammar.name));
    let parser = parser_ident(&grammar.name);
    quote! {
        use std::process::ExitCode;

        use iguana_runtime::cli;
        use #grammar_name::#parser;

        fn main() -> ExitCode {
            cli::main::<#parser>()
        }
    }
}
