use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    generator::id::{CharClassIds, TerminalIds, collect_char_classes},
    grammar::{
        def::Grammar,
        regex::{CharClass, CharRange, Regex},
        symbols::Terminal,
    },
};

pub fn generate(grammar: &Grammar, terminal_ids: &TerminalIds) -> TokenStream {
    let grammar_name = &grammar.name;

    // Collect all character classes from lexical rules
    let mut char_class_ids = CharClassIds::default();
    for terminal in grammar.terminals() {
        if let Some(regex) = grammar.lexical_rules(terminal) {
            collect_char_classes(regex, &mut char_class_ids);
        }
    }

    let imports = gen_imports();
    let char_class_consts = gen_char_class_consts(&char_class_ids);
    let scanner_struct = gen_scanner_struct(grammar_name);
    let scanner_impl = gen_scanner_imp(grammar, terminal_ids, &char_class_ids);
    let scanner_trait_impl = gen_scanner_trait_impl(grammar, terminal_ids, &char_class_ids);
    quote! {
        #imports
        #char_class_consts
        #scanner_struct
        #scanner_impl
        #scanner_trait_impl
    }
}

fn gen_imports() -> TokenStream {
    quote! {
        use iguana::{
            ids::TerminalId,
            input::Input,
            scanner::Scanner,
            sppf::{Span, TerminalNode},
        };
    }
}

fn gen_scanner_struct(grammar_name: &str) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    quote! {
        pub struct #name_ident<'i> {
            pub input: &'i Input,
        }
    }
}

fn gen_char_class_consts(char_class_ids: &CharClassIds) -> TokenStream {
    let consts: Vec<_> = char_class_ids
        .ids()
        .map(|id| {
            let char_class = char_class_ids.get(id);
            let const_name = format_ident!("CHAR_CLASS_{}", id.index());
            let len = char_class.ranges.len();
            let range_tuples: Vec<_> = char_class
                .ranges
                .iter()
                .map(|r| {
                    let start = r.start;
                    let end = r.end;
                    quote! { (#start, #end) }
                })
                .collect();
            quote! {
                const #const_name: [(char, char); #len] = [#(#range_tuples),*];
            }
        })
        .collect();
    quote! {
        #(#consts)*
    }
}

fn gen_scanner_imp(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    char_class_ids: &CharClassIds,
) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar.name, "Scanner"), Span::call_site());
    let match_terminals: Vec<_> = terminal_ids
        .terminals()
        .enumerate()
        .map(|(id, terminal)| {
            gen_match_terminal_method(id as u16, terminal, char_class_ids, grammar)
        })
        .collect();
    quote! {
        impl<'i> #name_ident<'i> {
            pub fn new(input: &'i Input) -> Self {
                Self { input }
            }
            #(#match_terminals)*
        }
    }
}

fn gen_scanner_trait_impl(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    _char_class_ids: &CharClassIds,
) -> TokenStream {
    let match_token_method = gen_match_token(terminal_ids);
    let char_at_method = gen_char_at_method();
    let match_leading_layout_method = gen_match_layout_method(grammar, terminal_ids, false);
    let match_trailing_layout_method = gen_match_layout_method(grammar, terminal_ids, true);
    let scanner_name = format_ident!("{}Scanner", grammar.name);
    quote! {
        impl Scanner for #scanner_name<'_> {
            #match_token_method
            #char_at_method
            #match_leading_layout_method
            #match_trailing_layout_method
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

fn gen_match_token(terminal_ids: &TerminalIds) -> TokenStream {
    let match_terminal_arms: Vec<_> = terminal_ids
        .ids()
        .map(|id| {
            let fn_name = format_ident!("match_terminal_{}", id.index() as u16);
            quote! {
                #id => {
                    self.#fn_name(input_index)
                }
            }
        })
        .collect();

    let match_token = if match_terminal_arms.is_empty() {
        quote! {
            None
        }
    } else {
        quote! {
            let res = match terminal_id {
                #(#match_terminal_arms)*
                _ => {
                    unreachable!("Unknown token type: {terminal_id}");
                }
            };
            #[comment = "Regular expressions should match at least 1 character."]
            res.filter(|next_index| *next_index > input_index)
        }
    };
    quote! {
        fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
            #match_token
        }
    }
}

fn gen_match_terminal_method(
    id: u16,
    terminal: &Terminal,
    char_class_ids: &CharClassIds,
    grammar: &Grammar,
) -> TokenStream {
    let fn_name = format_ident!("match_terminal_{}", id);
    let terminal_name = &terminal.name;
    let regex = grammar
        .lexical_rules(terminal)
        .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name));
    let match_regex = match_regex(regex, char_class_ids);

    quote! {
        #[comment= #terminal_name]
        pub fn #fn_name(&self, input_index: u32) -> Option<u32> {
            let i = input_index;
            #match_regex
        }
    }
}

fn gen_match_layout_method(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    trailing: bool,
) -> TokenStream {
    let layout_def_ids: Vec<_> = grammar
        .layout_defs
        .iter()
        .map(|t| terminal_ids.get_id(t))
        .collect();
    let method_name = if trailing {
        Ident::new("match_trailing_layout", Span::call_site())
    } else {
        Ident::new("match_leading_layout", Span::call_site())
    };
    let trailing_layout_check = if trailing {
        quote! {
            #[comment = "// If the last matched character is a newline, do not match further"]
            if let Some(last_matched_char) = self.input.char_at(next_index - 1)
                && last_matched_char == '\n'
            {
                break;
            }
        }
    } else {
        quote! {}
    };
    quote! {
        fn #method_name(&self, input_index: u32) -> (u32, Vec<TerminalNode>) {
            let mut i = input_index;
            let mut layout_nodes = vec![];
            while let Some((next_index, terminal_id)) = self.match_any(&vec![#(#layout_def_ids),*], i) {
                layout_nodes.push(TerminalNode::new(terminal_id, Span::new(i, next_index)));
                i = next_index;
                #trailing_layout_check
            }
            (i, layout_nodes)
        }
    }
}

fn match_regex(regex: &Regex, char_class_ids: &CharClassIds) -> TokenStream {
    match regex {
        Regex::Char(c) => match_char(*c),
        Regex::CharRange(range) => match_char_range(*range),
        Regex::CharClass(cc) => match_char_class(cc, char_class_ids),
        Regex::Seq(rs) => match_seq(rs, char_class_ids),
        Regex::Alt(rs) => match_alt(rs, char_class_ids),
        Regex::Star(r) => match_star(r, char_class_ids),
        _ => todo!(),
    }
}

fn match_char(c: char) -> TokenStream {
    quote! {
        self.match_char(i, #c)
    }
}

fn match_char_range(range: CharRange) -> TokenStream {
    let start = range.start;
    let end = range.end;
    quote! {
        self.match_char_range(i, #start, #end)
    }
}

fn match_char_class(cc: &CharClass, char_class_ids: &CharClassIds) -> TokenStream {
    let id = char_class_ids
        .get_id(cc)
        .expect("CharClass should have been collected");
    let char_class_name = format_ident!("CHAR_CLASS_{}", id.index());
    let negated = cc.negated;
    quote! {
        self.match_char_class(i, &#char_class_name, #negated)
    }
}

fn match_seq(rs: &[Regex], char_class_ids: &CharClassIds) -> TokenStream {
    if let Some((first, rest)) = rs.split_first() {
        let match_first = match_regex(first, char_class_ids);
        let rest: Vec<_> = rest
            .iter()
            .map(|r| {
                let match_r = match_regex(r, char_class_ids);
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

fn match_alt(rs: &[Regex], char_class_ids: &CharClassIds) -> TokenStream {
    if let Some((first, rest)) = rs.split_first() {
        let match_first = match_regex(first, char_class_ids);
        let rest: Vec<_> = rest
            .iter()
            .map(|r| {
                let match_r = match_regex(r, char_class_ids);
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

fn match_star(r: &Regex, char_class_ids: &CharClassIds) -> TokenStream {
    let match_r = match_regex(r, char_class_ids);
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
            id::CharClassIds,
            scanner_gen::{match_alt, match_char, match_char_class, match_char_range, match_star},
            utils::rustfmt,
        },
        grammar::regex::{CharClass, CharRange, Regex}, scanner::Range,
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
        let code = format(match_char_range(CharRange {
            start: 'a',
            end: 'z',
        }));
        assert_snapshot!(code);
    }

    #[test]
    fn test_match_alt() {
        let char_class_ids = CharClassIds::default();
        let code = format(match_alt(
            &[Regex::Char('a'), Regex::Char('b')],
            &char_class_ids,
        ));
        assert_snapshot!(code);
    }

    #[test]
    fn test_match_star() {
        let char_class_ids = CharClassIds::default();
        let code = format(match_star(&Regex::Char('a'), &char_class_ids));
        assert_snapshot!(code);
    }
}
