use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};

use crate::{
    dfa::{Dfa, Nfa},
    generator::{GenConfig, id::TerminalIds},
    grammar::{
        def::Grammar,
        symbols::{Definition, Terminal},
    },
};

pub fn generate(grammar: &Grammar, terminal_ids: &TerminalIds, config: &GenConfig) -> TokenStream {
    let grammar_name = &grammar.name;

    let imports = gen_imports(config);
    let memo_words = gen_memo_words_const(terminal_ids, config);
    let dfa_statics = gen_dfa_statics(grammar, terminal_ids);
    let scanner_struct = gen_scanner_struct(grammar_name, config);
    let scanner_impl = gen_scanner_imp(grammar, terminal_ids, config);
    let scanner_trait_impl = gen_scanner_trait_impl(grammar, terminal_ids, config);
    quote! {
        #imports
        #memo_words
        #dfa_statics
        #scanner_struct
        #scanner_impl
        #scanner_trait_impl
    }
}

fn gen_imports(config: &GenConfig) -> TokenStream {
    if config.match_memo {
        quote! {
            use iguana_runtime::{
                dfa::{Dfa, State},
                ids::TerminalId,
                input::Input,
                scanner::{Lookup, MatchMemo, Scanner},
            };
        }
    } else {
        quote! {
            use iguana_runtime::{
                dfa::{Dfa, State},
                ids::TerminalId,
                input::Input,
                scanner::Scanner,
            };
        }
    }
}

fn gen_memo_words_const(terminal_ids: &TerminalIds, config: &GenConfig) -> TokenStream {
    if !config.match_memo {
        return quote! {};
    }
    let words = (terminal_ids.len() + 2).div_ceil(64);
    let words_lit = Literal::usize_unsuffixed(words);
    quote! {
        const MATCH_MEMO_WORDS: usize = #words_lit;
    }
}

fn gen_scanner_struct(grammar_name: &str, config: &GenConfig) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    if config.match_memo {
        quote! {
            pub struct #name_ident<'i> {
                pub input: &'i Input,
                memo: MatchMemo<MATCH_MEMO_WORDS>,
            }
        }
    } else {
        quote! {
            pub struct #name_ident<'i> {
                pub input: &'i Input,
            }
        }
    }
}

fn gen_dfa_statics(grammar: &Grammar, terminal_ids: &TerminalIds) -> TokenStream {
    let statics: Vec<_> = terminal_ids
        .terminals()
        .enumerate()
        .map(|(id, terminal)| {
            let rule = grammar
                .lexical_rule(terminal)
                .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name));
            let nfa = Nfa::from_regex(&rule.regex, terminal_ids.get_id(terminal));
            let dfa = Dfa::from_nfa(&nfa);
            gen_dfa_static(id as u16, &dfa)
        })
        .collect();
    quote! {
        #(#statics)*
    }
}

fn gen_dfa_static(id: u16, dfa: &Dfa) -> TokenStream {
    assert_eq!(
        dfa.start, 0,
        "subset construction always emits start state 0"
    );
    let const_name = format_ident!("DFA_{}", id);
    let states: Vec<TokenStream> = dfa
        .states
        .iter()
        .map(|state| {
            let transitions: Vec<TokenStream> = state
                .transitions
                .iter()
                .map(|(range, target)| {
                    let start = range.start;
                    let end = range.end;
                    let target_lit = Literal::u32_unsuffixed(*target as u32);
                    quote! { (#start, #end, #target_lit) }
                })
                .collect();
            let accept = match state.accept {
                Some(t) => {
                    let id_lit = Literal::u16_unsuffixed(t.0);
                    quote! { Some(TerminalId(#id_lit)) }
                }
                None => quote! { None },
            };
            quote! {
                State::new(&[#(#transitions),*], #accept)
            }
        })
        .collect();
    quote! {
        static #const_name: Dfa = Dfa::new(&[#(#states),*]);
    }
}

fn gen_scanner_imp(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    config: &GenConfig,
) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar.name, "Scanner"), Span::call_site());
    let match_terminals: Vec<_> = terminal_ids
        .terminals()
        .enumerate()
        .map(|(id, terminal)| gen_match_terminal_method(id as u16, terminal, grammar, terminal_ids))
        .collect();
    let new_body = if config.match_memo {
        quote! {
            let memo = MatchMemo::new(input.len() as usize);
            Self { input, memo }
        }
    } else {
        quote! {
            Self { input }
        }
    };
    quote! {
        impl<'i> #name_ident<'i> {
            pub fn new(input: &'i Input) -> Self {
                #new_body
            }
            #(#match_terminals)*
        }
    }
}

fn gen_scanner_trait_impl(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    config: &GenConfig,
) -> TokenStream {
    let match_token_method = gen_match_token(terminal_ids, config);
    let char_at_method = gen_char_at_method();
    let scanner_name = format_ident!("{}Scanner", grammar.name);
    quote! {
        impl Scanner for #scanner_name<'_> {
            #match_token_method
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

fn gen_match_token(terminal_ids: &TerminalIds, config: &GenConfig) -> TokenStream {
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

    let eof_id = Literal::u16_unsuffixed(terminal_ids.len() as u16 + 1);
    let dispatch = quote! {
        match terminal_id {
            #(#match_terminal_arms)*
            TerminalId(#eof_id) => {
                if input_index == self.input.len() { Some(input_index) } else { None }
            }
            _ => {
                unreachable!("Unknown token type: {terminal_id}");
            }
        }
    };
    let match_token = if match_terminal_arms.is_empty() {
        quote! {
            None
        }
    } else if config.match_memo {
        quote! {
            if let Some(lookup) = self.memo.get(terminal_id, input_index) {
                return match lookup {
                    Lookup::Match(end) => Some(end),
                    Lookup::Fail => None,
                };
            }
            let result = #dispatch;
            match result {
                Some(end) => self.memo.insert_match(terminal_id, input_index, end),
                None => self.memo.insert_fail(terminal_id, input_index),
            }
            result
        }
    } else {
        dispatch
    };
    quote! {
        fn match_token(&mut self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
            #match_token
        }
    }
}

fn gen_match_terminal_method(
    id: u16,
    terminal: &Terminal,
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
) -> TokenStream {
    let fn_name = format_ident!("match_terminal_{}", id);
    let dfa_name = format_ident!("DFA_{}", id);
    let rule = grammar
        .lexical_rule(terminal)
        .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name));

    let except_checks: Vec<_> = rule
        .except
        .iter()
        .map(|except| {
            let Definition::Terminal(except_terminal) = grammar.definition(except.resolve()) else {
                panic!("Except {} must refer to a terminal", except.name);
            };
            let except_id = terminal_ids.get_id(except_terminal);
            let except_fn = format_ident!("match_terminal_{}", except_id.index());
            quote! {
                .and_then(|end| {
                    if self.#except_fn(input_index) == Some(end) {
                        None
                    } else {
                        Some(end)
                    }
                })
            }
        })
        .collect();

    let follow_restriction_check = rule.follow_restriction.as_ref().map(|restriction| {
        let Definition::Terminal(restriction_terminal) = grammar.definition(restriction.resolve())
        else {
            panic!(
                "Follow restriction {} must refer to a terminal",
                restriction.name
            );
        };
        let restriction_id = terminal_ids.get_id(restriction_terminal);
        let restriction_fn = format_ident!("match_terminal_{}", restriction_id.index());
        quote! {
            .and_then(|end| {
                if self.#restriction_fn(end).is_some() {
                    None
                } else {
                    Some(end)
                }
            })
        }
    });

    let precede_restriction_check = rule.precede_restriction.as_ref().map(|restriction| {
        let Definition::Terminal(restriction_terminal) = grammar.definition(restriction.resolve())
        else {
            panic!(
                "Precede restriction {} must refer to a terminal",
                restriction.name
            );
        };
        let restriction_id = terminal_ids.get_id(restriction_terminal);
        let restriction_fn = format_ident!("match_terminal_{}", restriction_id.index());
        quote! {
            if input_index > 0 && self.#restriction_fn(input_index - 1).is_some() {
                return None;
            }
        }
    });

    let comment = rule.to_string();
    quote! {
        #[comment = #comment]
        pub fn #fn_name(&self, input_index: u32) -> Option<u32> {
            #precede_restriction_check
            self.scan(&#dfa_name, input_index)
            #(#except_checks)*
            #follow_restriction_check
        }
    }
}
