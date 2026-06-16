use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};

use crate::{
    dfa::{Dfa, Nfa},
    generator::{GenConfig, id::TerminalIds, terminal_sets::SetIds},
    grammar::{
        def::Grammar,
        regex::Regex,
        symbols::{Definition, Symbol, Terminal},
    },
};

pub fn generate(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    match_any_sets: &SetIds,
    config: &GenConfig,
) -> TokenStream {
    let grammar_name = &grammar.name;

    let imports = gen_imports(config);
    let memo_words = gen_memo_words_const(terminal_ids, config);
    let match_any_words = gen_match_any_words_const(match_any_sets, config);
    let dfa_statics = gen_dfa_statics(grammar, terminal_ids);
    let scanner_struct = gen_scanner_struct(grammar_name, config);
    let scanner_impl = gen_scanner_imp(grammar, terminal_ids, config);
    let scanner_trait_impl = gen_scanner_trait_impl(grammar, terminal_ids, config);
    quote! {
        #imports
        #memo_words
        #match_any_words
        #dfa_statics
        #scanner_struct
        #scanner_impl
        #scanner_trait_impl
    }
}

fn gen_imports(config: &GenConfig) -> TokenStream {
    let mut scanner_imports = vec![quote! { Scanner }, quote! { TerminalSet }];
    if config.match_memo {
        scanner_imports.push(quote! { Lookup });
        scanner_imports.push(quote! { MatchMemo });
        scanner_imports.push(quote! { MatchAnyMemo });
    }
    quote! {
        use iguana_runtime::{
            dfa::{Dfa, State},
            ids::TerminalId,
            input::Input,
            scanner::{#(#scanner_imports),*},
        };
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

/// Emits `MATCH_ANY_SET_WORDS`, the number of `u64` words in each of the
/// `match_any` memo's bitsets.
///
/// The memo packs one bit per set id at each input position, so `count`
/// distinct sets need `ceil(count / 64)` words, at least one so the array is
/// never zero-length. The scanner sizes its table as
/// `MatchAnyMemo<MATCH_ANY_SET_WORDS>`.
fn gen_match_any_words_const(match_any_sets: &SetIds, config: &GenConfig) -> TokenStream {
    if !config.match_memo {
        return quote! {};
    }
    let words = match_any_sets.count().div_ceil(64).max(1);
    let words_lit = Literal::usize_unsuffixed(words);
    quote! {
        const MATCH_ANY_SET_WORDS: usize = #words_lit;
    }
}

fn gen_scanner_struct(grammar_name: &str, config: &GenConfig) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    if config.match_memo {
        quote! {
            pub struct #name_ident<'i> {
                pub input: &'i Input,
                memo: MatchMemo<MATCH_MEMO_WORDS>,
                match_any_memo: MatchAnyMemo<MATCH_ANY_SET_WORDS>,
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
            let terminal_id = terminal_ids.get_id(terminal);
            let nfa = if rule.except.is_empty() {
                Nfa::from_regex(&rule.regex, terminal_id)
            } else {
                let excepts: Vec<&Regex> = rule
                    .except
                    .iter()
                    .map(|except| {
                        let (_, except_rule) = grammar.except_terminal(except);
                        &except_rule.regex
                    })
                    .collect();
                Nfa::with_excepts(&rule.regex, terminal_id, &excepts)
            };
            gen_dfa_static(id as u16, &Dfa::from_nfa(&nfa))
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
            let constructor = if state.excluded {
                quote! { State::new_excluded }
            } else {
                quote! { State::new }
            };
            quote! {
                #constructor(&[#(#transitions),*], #accept)
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
            let match_any_memo = MatchAnyMemo::new(input.len() as usize);
            Self { input, memo, match_any_memo }
        }
    } else {
        quote! {
            Self { input }
        }
    };
    let match_any_method = gen_match_any_method(config);
    let match_exact_method = gen_match_exact_method(grammar, terminal_ids);
    quote! {
        impl<'i> #name_ident<'i> {
            pub fn new(input: &'i Input) -> Self {
                #new_body
            }
            #(#match_terminals)*
            #match_any_method
            #match_exact_method
        }
    }
}

/// Emits `match_exact`, the dispatcher behind syntax-level excepts
/// (`Id = Name \ Keyword` in a syntax rule): whether `terminal_id` matches
/// exactly the span a symbol matched. Only terminals used as syntax-level
/// excepts get an arm; grammars without such excepts get no method. The walk
/// mirrors the parser generator's: excepts sit at top-level alternative
/// positions after desugaring.
fn gen_match_exact_method(grammar: &Grammar, terminal_ids: &TerminalIds) -> TokenStream {
    let mut except_ids = Vec::new();
    for nonterminal in grammar.nonterminals() {
        for alternative in grammar.alternatives(nonterminal) {
            for symbol in &alternative.symbols {
                let Symbol::Except { except, .. } = symbol else {
                    continue;
                };
                for e in except {
                    let (terminal, _) = grammar.except_terminal(e);
                    let id = terminal_ids.get_id(terminal);
                    if !except_ids.contains(&id) {
                        except_ids.push(id);
                    }
                }
            }
        }
    }
    if except_ids.is_empty() {
        return quote! {};
    }
    let arms: Vec<_> = except_ids
        .iter()
        .map(|id| {
            let dfa_name = format_ident!("DFA_{}", id.index());
            quote! {
                #id => self.scan_exact(&#dfa_name, start, end),
            }
        })
        .collect();
    quote! {
        #[comment = "Whether `terminal_id` matches exactly the span `[start, end)`. Dispatches only the
                     terminals used as syntax-level excepts."]
        pub fn match_exact(&self, terminal_id: TerminalId, start: u32, end: u32) -> bool {
            match terminal_id {
                #(#arms)*
                _ => unreachable!("match_exact called for {terminal_id}, which is not an except"),
            }
        }
    }
}

fn gen_match_any_method(config: &GenConfig) -> TokenStream {
    if config.match_memo {
        quote! {
            #[comment = "Whether any terminal in `set` matches at `input_index`, cached by the set's memo id. The
                         first query of a set at a position scans it; later queries return the cached bit."]
            pub fn match_any(&mut self, set: &TerminalSet, input_index: u32) -> bool {
                if let Some(matched) = self.match_any_memo.get(set.id, input_index) {
                    return matched;
                }
                let matched = set
                    .terminals
                    .iter()
                    .any(|id| self.match_token(*id, input_index).is_some());
                self.match_any_memo.insert(set.id, input_index, matched);
                matched
            }
        }
    } else {
        quote! {
            pub fn match_any(&mut self, set: &TerminalSet, input_index: u32) -> bool {
                set.terminals
                    .iter()
                    .any(|id| self.match_token(*id, input_index).is_some())
            }
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

    // One check per follow restriction; chaining `.and_then` rejects the match
    // if any restriction terminal matches at the end position.
    let follow_restriction_checks: Vec<_> = rule
        .follow_restriction
        .iter()
        .map(|restriction| {
            let Definition::Terminal(restriction_terminal) =
                grammar.definition(restriction.resolve())
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
        })
        .collect();

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
            #(#follow_restriction_checks)*
        }
    }
}
