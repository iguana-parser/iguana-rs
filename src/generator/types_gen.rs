use proc_macro2::TokenStream;
use quote::quote;

pub fn generate() -> TokenStream {
    let ebnf_kind_enum = gen_ebnf_kind_enum();
    let nonterminal_struct = gen_nonterminal_struct();
    let nonterminal_impl = gen_nonterminal_impl();
    let terminal_struct = gen_terminal_struct();
    let slot_struct = gen_slot_struct();
    quote! {
        #ebnf_kind_enum
        #nonterminal_struct
        #nonterminal_impl
        #terminal_struct
        #slot_struct
    }
}

fn gen_ebnf_kind_enum() -> TokenStream {
    quote! {
        pub enum EbnfKind {
            Star,
            Plus,
            Opt,
            Group,
            Alt,
        }
    }
}

fn gen_nonterminal_struct() -> TokenStream {
    quote! {
        pub struct Nonterminal {
            pub name: &'static str,
            pub display: &'static str,
            pub kind: Option<EbnfKind>,
        }
    }
}

fn gen_nonterminal_impl() -> TokenStream {
    quote! {
        impl Nonterminal {
            pub fn is_ebnf(&self) -> bool {
                self.kind.is_some()
            }
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
