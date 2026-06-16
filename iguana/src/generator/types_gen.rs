use proc_macro2::TokenStream;
use quote::quote;

pub fn generate() -> TokenStream {
    let nonterminal_struct = gen_nonterminal_struct();
    let terminal_struct = gen_terminal_struct();
    let slot_struct = gen_slot_struct();
    quote! {
        #[comment = "Lightweight grammar metadata types for the CLI and parser runtime."]

        #nonterminal_struct
        #terminal_struct
        #slot_struct
    }
}

fn gen_nonterminal_struct() -> TokenStream {
    quote! {
        pub struct Nonterminal {
            pub name: &'static str,
            pub display: &'static str,
            #[comment = "Whether this nonterminal was introduced by a grammar transformation (e.g., EBNF
                         desugaring, start symbol wrapping, or exclude desugaring) rather than being
                         explicitly defined by the user."]
            pub derived: bool,
        }
    }
}

fn gen_terminal_struct() -> TokenStream {
    quote! {
        pub struct Terminal {
            pub name: &'static str,
        }
    }
}

fn gen_slot_struct() -> TokenStream {
    quote! {
        pub struct Slot {
            pub display_name: &'static str,
        }
    }
}
