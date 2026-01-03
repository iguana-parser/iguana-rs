use proc_macro2::TokenStream;
use quote::quote;

pub fn generate() -> TokenStream {
    quote! {
        pub mod parser;
        pub mod parse_tree;
        pub mod scanner;
        pub mod types;
    }
}
