use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    generator::grammar_utils::{grammar_ident, parser_ident},
    grammar::def::Grammar,
};

/// Generates `lib.rs`: the module list and the crate-root re-exports. The
/// root imports nothing else, so a nonterminal may carry any name the runtime
/// uses, such as `Input` or `Parser`, without shadowing it here.
pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_type = grammar_ident(&grammar.name);
    let parser = parser_ident(&grammar.name);

    quote! {
        pub mod grammar;
        pub mod parser;
        pub mod parse_tree;
        pub mod scanner;

        pub use grammar::#grammar_type;
        pub use iguana_runtime::grammar::Grammar;
        pub use iguana_runtime::result::{ParseError, ParseSuccess};
        pub use parser::#parser;
    }
}
