use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{generator::id::TerminalIds, grammar::symbols::Grammar};

pub fn generate(grammar: &Grammar, terminal_ids: &TerminalIds) -> TokenStream {
    let grammar_name = &grammar.name;
    let imports = gen_imports();
    let scanner_struct = gen_scanner_struct(grammar_name);
    let scanner_impl = gen_scanner_impl(grammar_name, terminal_ids);
    quote! {
        #imports
        #scanner_struct
        #scanner_impl
    }
}

fn gen_imports() -> TokenStream {
    quote! {
        use iguana::{input::Input, parser::TerminalId, scanner::Scanner};
    }
}

fn gen_scanner_struct(grammar_name: &str) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    quote! {
        pub struct #name_ident<'i> {
            pub input: &'i Input,
        }

        impl<'i> #name_ident<'i> {
            pub fn new(input: &'i Input) -> Self {
                Self { input }
            }
        }
    }
}

fn gen_scanner_impl(name: &str, terminal_ids: &TerminalIds) -> TokenStream {
    let match_tokens_method = gen_match_token(terminal_ids);
    let name_ident = syn::Ident::new(&format!("{}{}", name, "Scanner"), Span::call_site());
    quote! {
        impl Scanner for #name_ident<'_> {
            #match_tokens_method
        }
    }
}

fn gen_match_token(terminal_ids: &TerminalIds) -> TokenStream {
    let mut match_terminal_id_quotes = vec![];
    for (id, terminal_name) in terminal_ids.terminals().enumerate() {
        let ch = terminal_name.chars().next().unwrap();
        let id = id as u16;
        match_terminal_id_quotes.push(quote! {
            #[comment= #terminal_name]
            TerminalId(#id) => {
                if let Some(c) = self.input.char_at(input_index) && c == #ch {
                    Some(input_index + 1)
                } else {
                    None
                }
            }
        });
    }
    quote! {
        fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
            match terminal_id {
                #(#match_terminal_id_quotes)*
                _ => {
                    unreachable!("Unknown token type: {terminal_id}");
                }
            }
        }
    }
}
