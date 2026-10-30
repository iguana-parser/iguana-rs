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
    let dfa_statics = gen_dfa_statics(grammar, terminal_ids);
    let first_chars_statics =
        gen_first_chars_statics(terminal_ids, first_chars, terminal_sets, match_any_count);
    let match_any_masks_static =
        gen_match_any_masks_static(terminal_ids, terminal_sets, match_any_count, config);
    let first_char_masks_static = gen_first_char_masks_static(terminal_ids, first_chars, config);
    let scanner_struct = gen_scanner_struct(grammar_name, config);
    let scanner_impl = gen_scanner_imp(grammar, terminal_ids, match_any_count, config);
    let scanner_trait_impl = gen_scanner_trait_impl(grammar, terminal_ids, config);
    quote! {
        #imports
        #memo_words
        #dfa_statics
        #first_chars_statics
        #match_any_masks_static
        #first_char_masks_static
        #scanner_struct
        #scanner_impl
        #scanner_trait_impl
    }
}

fn gen_imports(config: &GenConfig) -> TokenStream {
    let mut scanner_imports = vec![quote! { Scanner }, quote! { TerminalSet }];
    let (arena_import, prediction_import) = if config.match_memo {
        scanner_imports.push(quote! { Lookup });
        scanner_imports.push(quote! { MatchMemo });
        (
            quote! { arena::Arena, },
            quote! { prediction::{AlternativeSet, Prediction}, },
        )
    } else {
        (quote! {}, quote! {})
    };
    quote! {
        use iguana_runtime::{
            #arena_import
            char_set::{CharRange, CharSet},
            dfa::{Dfa, State},
            ids::TerminalId,
            input::Input,
            #prediction_import
            scanner::{#(#scanner_imports),*},
        };
    }
}

fn gen_memo_words_const(terminal_ids: &TerminalIds, config: &GenConfig) -> TokenStream {
    if !config.match_memo {
        return quote! {};
    }
    let words_lit = Literal::usize_unsuffixed(memo_words(terminal_ids));
    quote! {
        const MATCH_MEMO_WORDS: usize = #words_lit;
    }
}

/// The number of `u64` words in a bitset indexed by terminal id. The synthetic
/// epsilon and end-of-input terminals come after the grammar's terminals.
fn memo_words(terminal_ids: &TerminalIds) -> usize {
    (terminal_ids.len() + 2).div_ceil(64)
}

/// Emits `MATCH_ANY_MASKS`, which holds for each `match_any` set the set's
/// terminals as a bitset indexed by terminal id. `match_any` tests a set's mask
/// against the memo's bitsets at a position.
fn gen_match_any_masks_static(
    terminal_ids: &TerminalIds,
    terminal_sets: &[TerminalSet],
    match_any_count: usize,
    config: &GenConfig,
) -> TokenStream {
    if !config.match_memo {
        return quote! {};
    }
    let mut masks = vec![vec![0u64; memo_words(terminal_ids)]; match_any_count];
    for set in terminal_sets.iter().filter(|set| set.id < match_any_count) {
        for terminal in &set.terminals {
            let i = terminal_ids.get_id(terminal).index();
            masks[set.id][i / 64] |= 1u64 << (i % 64);
        }
    }
    let set_count = Literal::usize_unsuffixed(match_any_count);
    let masks = masks.iter().map(|mask| {
        let words = mask.iter().map(|&word| Literal::u64_unsuffixed(word));
        quote! { [#(#words),*] }
    });
    quote! {
        static MATCH_ANY_MASKS: [[u64; MATCH_MEMO_WORDS]; #set_count] = [#(#masks),*];
    }
}

fn gen_scanner_struct(grammar_name: &str, config: &GenConfig) -> TokenStream {
    let name_ident = scanner_ident(grammar_name);
    if config.match_memo {
        quote! {
            pub struct #name_ident<'i, 'arena> {
                pub input: &'i Input,
                memo: MatchMemo<'arena, MATCH_MEMO_WORDS>,
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
    let terminals = terminal_first_chars(terminal_ids, first_chars);
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
        #[comment = "Indexed by terminal id. `TERMINAL_FIRST_CHARS[t]` holds the characters that can begin a
                     match of the terminal `t`. A nullable terminal can match at every position, so its entry
                     holds every character. `match_token` tests the character at a position against it before it
                     matches the terminal."]
        static TERMINAL_FIRST_CHARS: [CharSet; #terminal_count] = [#(#terminals),*];
        static MATCH_ANY_FIRST_CHARS: [CharSet; #set_count] = [#(#sets),*];
    }
}

/// The characters that can begin a match of each terminal, indexed by terminal
/// id, as sorted and disjoint ranges.
fn terminal_first_chars(
    terminal_ids: &TerminalIds,
    first_chars: &FirstChars,
) -> Vec<Vec<CharRange>> {
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
    terminals
}

/// Emits `FIRST_CHAR_MASKS`, indexed by ASCII character. `FIRST_CHAR_MASKS[c]`
/// is a bitset indexed by terminal id, and the bit of a terminal is set if the
/// terminal can begin with the character `c`. `match_any` clears the bits of the
/// other terminals before it matches the untried terminals of a set. The
/// synthetic epsilon terminal does not have a bit, because no set contains it.
fn gen_first_char_masks_static(
    terminal_ids: &TerminalIds,
    first_chars: &FirstChars,
    config: &GenConfig,
) -> TokenStream {
    if !config.match_memo {
        return quote! {};
    }
    let terminals = terminal_first_chars(terminal_ids, first_chars);
    let epsilon = terminal_ids.len();
    let masks = (0..128u32).map(|c| {
        let mut mask = vec![0u64; memo_words(terminal_ids)];
        for (i, ranges) in terminals.iter().enumerate() {
            if i != epsilon
                && ranges
                    .iter()
                    .any(|range| range.start as u32 <= c && c <= range.end as u32)
            {
                mask[i / 64] |= 1u64 << (i % 64);
            }
        }
        let words = mask.into_iter().map(Literal::u64_unsuffixed);
        quote! { [#(#words),*] }
    });
    quote! {
        #[comment = "Indexed by ASCII character. `FIRST_CHAR_MASKS[c]` is a bitset indexed by terminal id, and the
                     bit of a terminal is set if the terminal can begin with the character `c`. It holds the same
                     information as `TERMINAL_FIRST_CHARS`, inverted. `match_any` clears the bits of the terminals
                     of a set that cannot begin with the character at a position, so `match_any_untried` visits
                     only the terminals that can match there."]
        static FIRST_CHAR_MASKS: [[u64; MATCH_MEMO_WORDS]; 128] = [#(#masks),*];
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
                Self { input, memo }
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
    let match_token_and_memoize_method = gen_match_token_and_memoize(terminal_ids, config);
    let match_exact_method = gen_match_exact_method(grammar, terminal_ids);
    quote! {
        impl #impl_generics #name_ident #ty_generics {
            pub fn new(#new_params) -> Self {
                #new_body
            }
            #(#match_terminals)*
            #match_any_method
            #match_token_and_memoize_method
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

/// Emits `match_any`, which returns true if any terminal of a set matches at a
/// position.
///
/// With the memo, `match_any` answers from the memo of `match_token` where it
/// can. The mask of the set in `MATCH_ANY_MASKS` has a bit for each terminal of
/// the set, and `match_any` combines it with the `tried` and `matched` bitsets
/// of the position in two steps:
///
/// - `mask & matched` holds the terminals of the set that already matched at
///   the position. If it is not empty, the answer is true, whichever terminal
///   matched.
/// - `mask & !tried` holds the terminals of the set that were not tried at the
///   position yet. At an ASCII character, `FIRST_CHAR_MASKS` also clears the
///   terminals that cannot begin with it. If the result is empty, no terminal of
///   the set can match, and the answer is false. Otherwise `match_any_untried`
///   matches only these terminals through `match_token`, which records each
///   result in the memo.
///
/// A terminal that was tried and did not match is in neither result, so the two
/// steps skip it. Without the memo, `match_any` matches the terminals of the set
/// one by one.
fn gen_match_any_method(match_any_count: usize, config: &GenConfig) -> TokenStream {
    // Both versions, with and without the memo, start with the set's
    // first-character test. Without the memo, each terminal of the set then gets
    // its own first-character test, which skips the call to `match_token` for a
    // terminal that cannot start with `c`. With the memo, `FIRST_CHAR_MASKS`
    // clears such a terminal at an ASCII character, and `match_token` rejects it
    // at any other character. As in `match_token`, the tests pass at the end of
    // the input, where `char_at` is `None`. `match_any` is not inlined, for the
    // same reason as `match_token`.
    let first_char_test = quote! {
        let c = self.input.char_at(input_index);
        if c.is_some_and(|c| !MATCH_ANY_FIRST_CHARS[set.id].contains(c)) {
            return false;
        }
    };
    let scan = quote! {
        set.terminals.iter().any(|id| {
            c.is_none_or(|c| TERMINAL_FIRST_CHARS[id.index()].contains(c))
                && self.match_token(*id, input_index).is_some()
        })
    };
    if config.match_memo {
        let match_any_count = Literal::usize_unsuffixed(match_any_count);
        quote! {
            #[comment = "Returns true if any terminal in `set` matches at `input_index`. The memo of
                         `match_token` answers for the terminals of `set` already tried at `input_index`, and
                         only the other terminals are matched."]
            #[inline(never)]
            pub fn match_any(&mut self, set: &TerminalSet, input_index: u32) -> bool {
                debug_assert!(
                    set.id < #match_any_count,
                    "terminal set {} does not have a match_any id",
                    set.id,
                );
                #first_char_test
                let mask = &MATCH_ANY_MASKS[set.id];
                let (tried, matched) = self.memo.bits(input_index);
                if mask.iter().zip(matched).any(|(mask, matched)| mask & matched != 0) {
                    return true;
                }
                let mut untried = *mask;
                for (word, tried) in untried.iter_mut().zip(tried) {
                    *word &= !tried;
                }
                #[comment = "A terminal that cannot begin with an ASCII character `c` does not match, and its
                             bit is cleared without matching it."]
                if let Some(c) = c.filter(char::is_ascii) {
                    for (word, first) in untried.iter_mut().zip(&FIRST_CHAR_MASKS[c as usize]) {
                        *word &= first;
                    }
                }
                if untried.iter().all(|&word| word == 0) {
                    return false;
                }
                self.match_any_untried(untried, input_index)
            }
            #[comment = "Matches the terminals in `untried`, a bitset indexed by terminal id, at `input_index`
                         until one matches. `match_any` calls this method for the terminals of a set that were
                         not tried at `input_index` yet."]
            #[comment = "Not inlined into `match_any`. The parser calls `match_any` very often, and many calls
                         return after the first-character test or the bitset tests. This method needs its
                         arguments again after each call to `match_token`, and the compiler handles that with
                         extra instructions at the start and end of the function. Kept separate, those
                         instructions run only when a terminal of the set was not tried yet."]
            #[inline(never)]
            fn match_any_untried(&mut self, untried: [u64; MATCH_MEMO_WORDS], input_index: u32) -> bool {
                for (word, &bits) in untried.iter().enumerate() {
                    let mut bits = bits;
                    while bits != 0 {
                        let id = TerminalId((word * 64) as u16 + bits.trailing_zeros() as u16);
                        bits &= bits - 1;
                        if self.match_token(id, input_index).is_some() {
                            return true;
                        }
                    }
                }
                false
            }
        }
    } else {
        quote! {
            #[inline(never)]
            pub fn match_any(&mut self, set: &TerminalSet, input_index: u32) -> bool {
                #first_char_test
                #scan
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
    let predict_method = gen_predict_method(config);
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
            #predict_method
            #char_at_method
        }
    }
}

/// Emits `predict` when the scanner has a memo, in place of the default method
/// of `Scanner`. Like `match_any`, it answers from the `tried` and `matched`
/// bitsets of the position where it can, and only an untried terminal is
/// matched. Without the memo, the scanner uses the default method.
fn gen_predict_method(config: &GenConfig) -> TokenStream {
    if !config.match_memo {
        return quote! {};
    }
    quote! {
        #[comment = "The alternatives that `prediction` predicts at `input_index`: those predicted by a
                     terminal that matches there. The memo of `match_token` answers for the terminals already
                     tried at `input_index`, and only the other terminals are matched. A call to `match_token`
                     changes only the bits of its own terminal, so the copies of the bitsets stay valid."]
        #[inline]
        fn predict<A: AlternativeSet>(&mut self, prediction: &Prediction, input_index: u32) -> A {
            let (tried, matched) = {
                let (tried, matched) = self.memo.bits(input_index);
                (*tried, *matched)
            };
            let mut alternatives = A::default();
            for &(terminal, alternative) in prediction.terminals_at(self.input.char_at(input_index)) {
                let alternative = usize::from(alternative);
                #[comment = "An alternative already predicted does not need its terminal matched again."]
                if alternatives.contains(alternative) {
                    continue;
                }
                let (word, bit) = (terminal.index() / 64, 1u64 << (terminal.index() % 64));
                let matches = if matched[word] & bit != 0 {
                    true
                } else if tried[word] & bit != 0 {
                    false
                } else {
                    self.match_token(terminal, input_index).is_some()
                };
                if matches {
                    alternatives.insert(alternative);
                }
            }
            alternatives
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

/// The `match` on `terminal_id` that runs the DFA of the terminal, or tests
/// for the end of the input for EOF.
fn gen_token_dispatch(terminal_ids: &TerminalIds) -> TokenStream {
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
    quote! {
        match terminal_id {
            #(#match_terminal_arms)*
            TerminalId(#eof_id) => {
                if input_index == self.input.len() { Some(input_index) } else { None }
            }
            _ => {
                unreachable!("Unknown token type: {terminal_id}");
            }
        }
    }
}

/// Emits `match_token_and_memoize` when the scanner has a memo and the grammar
/// has terminals.
fn gen_match_token_and_memoize(terminal_ids: &TerminalIds, config: &GenConfig) -> TokenStream {
    if terminal_ids.len() == 0 || !config.match_memo {
        return quote! {};
    }
    let dispatch = gen_token_dispatch(terminal_ids);
    quote! {
        #[comment = "Matches `terminal_id` at `input_index` with its DFA and records the result in the memo.
                     `match_token` looks up the memo and calls this method on a miss."]
        #[comment = "Not inlined into `match_token`. The parser calls `match_token` very often, and most
                     calls return after the first-character test or a memo hit. This method needs its
                     arguments again after the DFA returns, for the memo insert, and the compiler handles
                     that with extra instructions at the start and end of the function. Kept separate,
                     those instructions run only on a memo miss instead of on every call."]
        #[inline(never)]
        fn match_token_and_memoize(&mut self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
            let result = #dispatch;
            match result {
                Some(end) => self.memo.insert_match(terminal_id, input_index, end),
                None => self.memo.insert_fail(terminal_id, input_index),
            }
            result
        }
    }
}

fn gen_match_token(terminal_ids: &TerminalIds, config: &GenConfig) -> TokenStream {
    let match_token = if terminal_ids.len() == 0 {
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
            self.match_token_and_memoize(terminal_id, input_index)
        }
    } else {
        gen_token_dispatch(terminal_ids)
    };
    // `match_token` is not inlined, so its many callers share one copy of the
    // first-character test and the memo lookup.
    quote! {
        #[inline(never)]
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
