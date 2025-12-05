use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use crate::{
    generator::id::TerminalIds,
    grammar::{grammar::Grammar, regex::Regex},
};

pub fn generate(grammar: &Grammar, terminal_ids: &TerminalIds) -> TokenStream {
    let grammar_name = &grammar.name;
    let imports = gen_imports();
    let scanner_struct = gen_scanner_struct(grammar_name);
    let scanner_impl = gen_scanner_impl(grammar_name, terminal_ids, grammar);
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

fn gen_scanner_impl(name: &str, terminal_ids: &TerminalIds, grammar: &Grammar) -> TokenStream {
    let match_tokens_method = gen_match_token(terminal_ids, grammar);
    let char_at_method = gen_char_at_method();
    let scanner_name = format_ident!("{name}Scanner");
    quote! {
        impl Scanner for #scanner_name<'_> {
            #match_tokens_method
            #char_at_method
        }
    }
}

fn gen_char_at_method() -> TokenStream {
    quote! {
        fn char_at(&self, i: u32) -> Option<char> {
            self.input.char_at(i)
        }
    }
}

fn gen_match_token(terminal_ids: &TerminalIds, grammar: &Grammar) -> TokenStream {
    let mut match_terminal_cases = vec![];
    for (id, terminal) in terminal_ids.terminals().enumerate() {
        let id = id as u16;
        let regex = grammar
            .lexical_rules(terminal)
            .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name));
        let match_regex = match_regex(regex);
        let terminal_name = &terminal.name;
        match_terminal_cases.push(quote! {
            #[comment= #terminal_name]
            TerminalId(#id) => {
                let i = input_index;
                #match_regex
            }
        });
    }

    quote! {
        fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
            match terminal_id {
                #(#match_terminal_cases)*
                _ => {
                    unreachable!("Unknown token type: {terminal_id}");
                }
            }
        }
    }
}

fn match_regex(regex: &Regex) -> TokenStream {
    match regex {
        Regex::Char(c) => match_char(*c),
        Regex::CharRange { start, end } => match_char_range(*start, *end),
        Regex::Seq(rs) => match_seq(rs),
        Regex::Alt(rs) => match_alt(rs),
        Regex::Star(r) => match_star(r),
        _ => todo!(),
    }
}

fn match_char(c: char) -> TokenStream {
    quote! {
        self.match_char(i, #c)
    }
}

fn match_char_range(start: char, end: char) -> TokenStream {
    quote! {
        self.match_char_range(i, #start, #end)
    }
}

fn match_seq(rs: &[Regex]) -> TokenStream {
    if let Some((first, rest)) = rs.split_first() {
        let match_first = match_regex(first);
        let rest: Vec<_> = rest
            .iter()
            .map(|r| {
                let match_r = match_regex(r);
                quote! {
                    .and_then(|i| { #match_r })
                }
            })
            .collect();
        quote! {
            #match_first
            #(#rest)*
        }
    } else {
        // there should be at least one seq
        unreachable!()
    }
}

fn match_alt(rs: &[Regex]) -> TokenStream {
    if let Some((first, rest)) = rs.split_first() {
        let match_first = match_regex(first);
        let rest: Vec<_> = rest
            .iter()
            .map(|r| {
                let match_r = match_regex(r);
                quote! {
                    .or_else(|| { #match_r })
                }
            })
            .collect();
        quote! {
            #match_first
            #(#rest)*
        }
    } else {
        unreachable!()
    }
}

fn match_star(r: &Regex) -> TokenStream {
    let match_r = match_regex(r);
    quote! {
        let mut j = i;
        while let Some(k) = (|i| { #match_r })(j) {
            j = k;
        }
        Some(j)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{
        generator::{
            scanner_gen::{match_alt, match_char, match_char_range, match_star},
            utils::rustfmt,
        },
        grammar::regex::Regex,
    };

    fn format(token_stream: TokenStream) -> String {
        let code = quote! {
            fn test_match(i: u32) -> Option<u32> {
                #token_stream
            }
        };
        rustfmt(&code.to_string())
    }

    #[test]
    fn test_match_char() {
        let code = format(match_char('a'));
        assert_snapshot!(code);
    }

    #[test]
    fn test_match_char_range() {
        let code = format(match_char_range('a', 'z'));
        assert_snapshot!(code);
    }

    #[test]
    fn test_match_alt() {
        let code = format(match_alt(&[Regex::Char('a'), Regex::Char('b')]));
        assert_snapshot!(code);
    }

    #[test]
    fn test_match_star() {
        let code = format(match_star(&Regex::Char('a')));
        assert_snapshot!(code);
    }
}
