use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::{
    dfa::{Dfa, Nfa},
    generator::{
        GenConfig, grammar_utils::scanner_ident, id::TerminalIds, terminal_sets::TerminalSet,
    },
    grammar::{
        def::Grammar,
        first_chars::FirstChars,
        regex::{CharRange, Regex, merge_ranges},
        symbols::{Definition, Terminal},
    },
};

pub fn generate(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    first_chars: &FirstChars,
    terminal_sets: &[TerminalSet],
    match_any_count: usize,
    config: &GenConfig,
) -> TokenStream {
    let grammar_name = &grammar.name;

    let imports = gen_imports(config);
    let memo_words = gen_memo_words_const(terminal_ids, config);
    let match_any_words = gen_match_any_words_const(match_any_count, config);
    let dfa_statics = gen_dfa_statics(grammar, terminal_ids);
    let first_chars_statics =
        gen_first_chars_statics(terminal_ids, first_chars, terminal_sets, match_any_count);
    let scanner_struct = gen_scanner_struct(grammar_name, config);
    let scanner_impl = gen_scanner_imp(grammar, terminal_ids, match_any_count, config);
    let scanner_trait_impl = gen_scanner_trait_impl(grammar, terminal_ids, config);
    quote! {
        #imports
        #memo_words
        #match_any_words
        #dfa_statics
        #first_chars_statics
        #scanner_struct
        #scanner_impl
        #scanner_trait_impl
    }
}

fn gen_imports(config: &GenConfig) -> TokenStream {
    let mut scanner_imports = vec![quote! { Scanner }, quote! { TerminalSet }];
    let arena_import = if config.match_memo {
        scanner_imports.push(quote! { Lookup });
        scanner_imports.push(quote! { MatchMemo });
        scanner_imports.push(quote! { MatchAnyMemo });
        quote! { arena::Arena, }
    } else {
        quote! {}
    };
    quote! {
        use iguana_runtime::{
            #arena_import
            char_set::{CharRange, CharSet},
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
/// The memo packs one bit per set id at each input position, so
/// `match_any_count` distinct sets need `ceil(match_any_count / 64)` words,
/// at least one so the array is never zero-length. The scanner sizes its
/// table as `MatchAnyMemo<MATCH_ANY_SET_WORDS>`.
fn gen_match_any_words_const(match_any_count: usize, config: &GenConfig) -> TokenStream {
    if !config.match_memo {
        return quote! {};
    }
    let words = match_any_count.div_ceil(64).max(1);
    let words_lit = Literal::usize_unsuffixed(words);
    quote! {
        const MATCH_ANY_SET_WORDS: usize = #words_lit;
    }
}

fn gen_scanner_struct(grammar_name: &str, config: &GenConfig) -> TokenStream {
    let name_ident = scanner_ident(grammar_name);
    if config.match_memo {
        quote! {
            pub struct #name_ident<'i, 'arena> {
                pub input: &'i Input,
                vec_arena: &'arena Arena,
                memo: MatchMemo<'arena, MATCH_MEMO_WORDS>,
                match_any_memo: MatchAnyMemo<'arena, MATCH_ANY_SET_WORDS>,
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

/// Emits `TERMINAL_FIRST_CHARS`, the characters that can begin a match of
/// each terminal, indexed by terminal id, and `MATCH_ANY_FIRST_CHARS`, the
/// characters that can begin a match of a terminal of each `match_any` set,
/// indexed by set id.
///
/// A nullable terminal matches at every position, so its entry is the full
/// set, which contains every character, and the test always passes.
fn gen_first_chars_statics(
    terminal_ids: &TerminalIds,
    first_chars: &FirstChars,
    terminal_sets: &[TerminalSet],
    match_any_count: usize,
) -> TokenStream {
    let all = vec![CharRange {
        start: '\0',
        end: char::MAX,
    }];
    let mut terminals: Vec<Vec<CharRange>> = terminal_ids
        .terminals()
        .map(|terminal| {
            if first_chars.is_nullable(terminal) {
                all.clone()
            } else {
                first_chars.first_chars(terminal).to_vec()
            }
        })
        .collect();
    // The synthetic epsilon and end-of-input terminals come after the grammar's
    // terminals. The scanner never asks for epsilon. If it did, the full set
    // would let the call reach the dispatch, which panics. The end-of-input
    // terminal matches no character, and its entry is the empty set.
    terminals.push(all);
    terminals.push(vec![]);
    let mut sets = vec![vec![]; match_any_count];
    for set in terminal_sets.iter().filter(|set| set.id < match_any_count) {
        let ranges = set
            .terminals
            .iter()
            .flat_map(|terminal| &terminals[terminal_ids.get_id(terminal).index()])
            .copied()
            .collect();
        sets[set.id] = merge_ranges(ranges);
    }
    let terminal_count = Literal::usize_unsuffixed(terminals.len());
    let set_count = Literal::usize_unsuffixed(match_any_count);
    let terminals = terminals.iter().map(|ranges| gen_char_set(ranges));
    let sets = sets.iter().map(|ranges| gen_char_set(ranges));
    quote! {
        static TERMINAL_FIRST_CHARS: [CharSet; #terminal_count] = [#(#terminals),*];
        static MATCH_ANY_FIRST_CHARS: [CharSet; #set_count] = [#(#sets),*];
    }
}

/// Emits the `CharSet` of `ranges`, which are sorted and disjoint. The ASCII
/// characters become the bitmask, and the other characters become ranges.
fn gen_char_set(ranges: &[CharRange]) -> TokenStream {
    let mut ascii: u128 = 0;
    let mut non_ascii = Vec::new();
    for range in ranges {
        let (start, end) = (range.start as u32, range.end as u32);
        for c in start..=end.min(127) {
            ascii |= 1 << c;
        }
        if end >= 128 {
            let start = range.start.max('\u{80}');
            let end = range.end;
            non_ascii.push(quote! { CharRange { start: #start, end: #end } });
        }
    }
    let ascii: Literal = format!("{ascii:#x}").parse().unwrap();
    quote! { CharSet::new(#ascii, &[#(#non_ascii),*]) }
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
    match_any_count: usize,
    config: &GenConfig,
) -> TokenStream {
    let name_ident = scanner_ident(&grammar.name);
    let match_terminals: Vec<_> = terminal_ids
        .terminals()
        .enumerate()
        .map(|(id, terminal)| gen_match_terminal_method(id as u16, terminal, grammar, terminal_ids))
        .collect();
    let (impl_generics, ty_generics, new_params, new_body) = if config.match_memo {
        (
            quote! { <'i, 'arena> },
            quote! { <'i, 'arena> },
            quote! { input: &'i Input, vec_arena: &'arena Arena },
            quote! {
                let memo = MatchMemo::new(input.len() as usize, vec_arena);
                let match_any_memo = MatchAnyMemo::new(input.len() as usize, vec_arena);
                Self { input, vec_arena, memo, match_any_memo }
            },
        )
    } else {
        (
            quote! { <'i> },
            quote! { <'i> },
            quote! { input: &'i Input },
            quote! { Self { input } },
        )
    };
    let match_any_method = gen_match_any_method(match_any_count, config);
    let match_exact_method = gen_match_exact_method(grammar, terminal_ids);
    quote! {
        impl #impl_generics #name_ident #ty_generics {
            pub fn new(#new_params) -> Self {
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
                for e in &symbol.restrictions().excepts {
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

fn gen_match_any_method(match_any_count: usize, config: &GenConfig) -> TokenStream {
    // Both versions, with and without the memo, start with the set's
    // first-character test. As in `match_token`, the test passes at the end of
    // the input, where `char_at` is `None`.
    if config.match_memo {
        let match_any_count = Literal::usize_unsuffixed(match_any_count);
        quote! {
            #[comment = "Whether any terminal in `set` matches at `input_index`, cached by the set's memo id. The
                         first query of a set at a position scans it; later queries return the cached bit."]
            pub fn match_any(&mut self, set: &TerminalSet, input_index: u32) -> bool {
                debug_assert!(
                    set.id < #match_any_count,
                    "terminal set {} does not have a match_any memo id",
                    set.id,
                );
                if self
                    .input
                    .char_at(input_index)
                    .is_some_and(|c| !MATCH_ANY_FIRST_CHARS[set.id].contains(c))
                {
                    return false;
                }
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
                if self
                    .input
                    .char_at(input_index)
                    .is_some_and(|c| !MATCH_ANY_FIRST_CHARS[set.id].contains(c))
                {
                    return false;
                }
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
    let scanner_name = scanner_ident(&grammar.name);
    let scanner_ty = if config.match_memo {
        quote! { #scanner_name<'_, '_> }
    } else {
        quote! { #scanner_name<'_> }
    };
    quote! {
        impl Scanner for #scanner_ty {
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

    let eof_id = Literal::u16_unsuffixed(terminal_ids.eof_id().0);
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
            let arena = self.vec_arena;
            match result {
                Some(end) => self.memo.insert_match(terminal_id, input_index, end, arena),
                None => self.memo.insert_fail(terminal_id, input_index),
            }
            result
        }
    } else {
        dispatch
    };
    quote! {
        fn match_token(&mut self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
            // `char_at` is `None` at the end of the input, where the test
            // passes and the scan decides.
            if self
                .input
                .char_at(input_index)
                .is_some_and(|c| !TERMINAL_FIRST_CHARS[terminal_id.index()].contains(c))
            {
                return None;
            }
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

    // One check per follow restriction; chaining `.filter` rejects the match
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
                .filter(|&end| self.#restriction_fn(end).is_none())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn char_range(start: char, end: char) -> CharRange {
        CharRange { start, end }
    }

    #[test]
    fn gen_char_set_splits_a_range_at_the_end_of_ascii() {
        let ascii: Literal = "0xffffffff000000000000000000000000".parse().unwrap();
        let (start, end) = ('\u{80}', '\u{100}');
        let expected = quote! {
            CharSet::new(#ascii, &[CharRange { start: #start, end: #end }])
        };
        assert_eq!(
            gen_char_set(&[char_range('`', '\u{100}')]).to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn gen_char_set_of_ascii_ranges_has_no_ranges_outside_ascii() {
        let ascii: Literal = "0x3ff000000000000".parse().unwrap();
        let expected = quote! { CharSet::new(#ascii, &[]) };
        assert_eq!(
            gen_char_set(&[char_range('0', '9')]).to_string(),
            expected.to_string()
        );
    }
}
