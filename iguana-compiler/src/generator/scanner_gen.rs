use indexmap::IndexMap;
use std::collections::BTreeSet;

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

use crate::{
    dfa::{Dfa, dfa::State},
    generator::{
        GenConfig,
        grammar_utils::scanner_ident,
        id::TerminalIds,
        literal_set::{LiteralSet, gen_literal_set, gen_literal_trie, literal_trie_ident},
        terminal_sets::TerminalSet,
    },
    grammar::{
        def::Grammar,
        first_chars::FirstChars,
        regex::{CharRange, Regex, merge_ranges},
        symbols::{Definition, Identifier, Terminal},
    },
    ids::TerminalId,
};

pub struct ScannerGen<'a> {
    grammar: &'a Grammar,
    terminal_ids: &'a TerminalIds,
    first_chars: &'a FirstChars,
    terminal_sets: &'a [TerminalSet<'a>],
    match_any_count: usize,
    config: GenConfig,
    /// The scan id of each terminal, indexed by terminal id. The scan
    /// recognizes the terminal's regex without its excepts, and terminals with
    /// the same regex share a scan.
    scan_ids: Vec<usize>,
    /// The scans, indexed by scan id.
    scans: Vec<Scan>,
    /// The sorted literals of `LITERAL_TRIE`.
    trie_literals: Vec<Vec<u8>>,
}

/// How a scan matches its regex.
enum Scan {
    /// The regex is one literal.
    Literal {
        literal: Vec<u8>,
        ignore_ascii_case: bool,
    },
    /// The regex is a set of literals, looked up in a trie.
    Trie(LiteralSet),
    Dfa(Dfa),
}

impl Scan {
    fn new(regex: &Regex) -> Scan {
        match LiteralSet::from_regex(regex) {
            Some(mut set) if set.literals.len() == 1 => Scan::Literal {
                literal: set.literals.pop().unwrap(),
                ignore_ascii_case: set.ignore_ascii_case,
            },
            Some(set) => Scan::Trie(set),
            None => Scan::Dfa(Dfa::from_regex(regex)),
        }
    }
}

impl<'a> ScannerGen<'a> {
    pub fn new(
        grammar: &'a Grammar,
        terminal_ids: &'a TerminalIds,
        first_chars: &'a FirstChars,
        terminal_sets: &'a [TerminalSet<'a>],
        match_any_count: usize,
        config: GenConfig,
    ) -> Self {
        let mut scans: IndexMap<&Regex, Scan> = IndexMap::new();
        let scan_ids = terminal_ids
            .terminals()
            .map(|terminal| {
                let regex = &grammar
                    .lexical_rule(terminal)
                    .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name))
                    .regex;
                let entry = scans.entry(regex);
                let scan_id = entry.index();
                entry.or_insert_with(|| Scan::new(regex));
                scan_id
            })
            .collect();
        let mut trie_literals = BTreeSet::new();
        for scan in scans.values() {
            if let Scan::Trie(set) = scan {
                trie_literals.extend(set.literals.iter().cloned());
            }
        }
        // The state ids and literal indices of the trie are `u16`, and the trie
        // has at most one state per character of its literals, plus the start
        // state. If the literals have more characters than a `u16` can count,
        // the literal sets use DFAs instead.
        if trie_literals.iter().map(Vec::len).sum::<usize>() >= u16::MAX as usize {
            for (regex, scan) in scans.iter_mut() {
                if let Scan::Trie(_) = scan {
                    *scan = Scan::Dfa(Dfa::from_regex(regex));
                }
            }
            trie_literals.clear();
        }
        ScannerGen {
            grammar,
            terminal_ids,
            first_chars,
            terminal_sets,
            match_any_count,
            config,
            scan_ids,
            scans: scans.into_values().collect(),
            trie_literals: trie_literals.into_iter().collect(),
        }
    }

    pub fn generate(&self) -> TokenStream {
        let imports = self.gen_imports();
        let memo_words = self.gen_memo_words_const();
        let first_chars_statics = self.gen_first_chars_statics();
        let match_any_masks_static = self.gen_match_any_masks_static();
        let first_char_masks_static = self.gen_first_char_masks_static();
        let scanner_struct = self.gen_scanner_struct();
        let literal_sets = self.gen_literal_sets();
        let scanner_impl = self.gen_scanner_imp();
        let scanner_trait_impl = self.gen_scanner_trait_impl();
        quote! {
            #imports
            #memo_words
            #first_chars_statics
            #match_any_masks_static
            #first_char_masks_static
            #literal_sets
            #scanner_struct
            #scanner_impl
            #scanner_trait_impl
        }
    }

    fn gen_imports(&self) -> TokenStream {
        let literal_sets = self.has_literal_sets();
        let mut scanner_imports = vec![quote! { Scanner }, quote! { TerminalSet }];
        if literal_sets {
            scanner_imports.push(quote! { LiteralTrie });
        }
        let (arena_import, prediction_import) = if self.config.match_memo {
            scanner_imports.push(quote! { Lookup });
            scanner_imports.push(quote! { MatchMemo });
            (
                quote! { arena::Arena, },
                quote! { prediction::{AlternativeSet, Prediction}, },
            )
        } else {
            (quote! {}, quote! {})
        };
        let bit_set_import = literal_sets.then(|| quote! { utils::bit_set::BitSet, });
        let mut char_set_imports = vec![quote! { CharRange }, quote! { CharSet }];
        if literal_sets {
            char_set_imports.insert(0, quote! { Alphabet });
        }
        quote! {
            use iguana_runtime::{
                #arena_import
                char_set::{#(#char_set_imports),*},
                ids::TerminalId,
                input::Input,
                #prediction_import
                scanner::{#(#scanner_imports),*},
                #bit_set_import
            };
        }
    }

    fn gen_memo_words_const(&self) -> TokenStream {
        if !self.config.match_memo {
            return quote! {};
        }
        let words_lit = Literal::usize_unsuffixed(self.memo_words());
        quote! {
            const MATCH_MEMO_WORDS: usize = #words_lit;
        }
    }

    /// The number of `u64` words in a bitset indexed by terminal id. The synthetic
    /// epsilon and end-of-input terminals come after the grammar's terminals.
    fn memo_words(&self) -> usize {
        (self.terminal_ids.len() + 2).div_ceil(64)
    }

    /// Emits `MATCH_ANY_MASKS`, which holds for each `match_any` set the set's
    /// terminals as a bitset indexed by terminal id. `match_any` tests a set's mask
    /// against the memo's bitsets at a position.
    fn gen_match_any_masks_static(&self) -> TokenStream {
        if !self.config.match_memo {
            return quote! {};
        }
        let mut masks = vec![vec![0u64; self.memo_words()]; self.match_any_count];
        for set in self
            .terminal_sets
            .iter()
            .filter(|set| set.id < self.match_any_count)
        {
            for terminal in &set.terminals {
                let i = self.terminal_ids.get_id(terminal).index();
                masks[set.id][i / 64] |= 1u64 << (i % 64);
            }
        }
        let set_count = Literal::usize_unsuffixed(self.match_any_count);
        let masks = masks.iter().map(|mask| {
            let words = mask.iter().map(|&word| Literal::u64_unsuffixed(word));
            quote! { [#(#words),*] }
        });
        quote! {
            static MATCH_ANY_MASKS: [[u64; MATCH_MEMO_WORDS]; #set_count] = [#(#masks),*];
        }
    }

    fn gen_scanner_struct(&self) -> TokenStream {
        let name_ident = scanner_ident(&self.grammar.name);
        if self.config.match_memo {
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

    /// The id of the scan of `terminal_id`, and the scan.
    fn scan(&self, terminal_id: usize) -> (usize, &Scan) {
        let scan_id = self.scan_ids[terminal_id];
        (scan_id, &self.scans[scan_id])
    }

    fn gen_longest_match(&self, terminal_id: usize) -> TokenStream {
        let (scan_id, scan) = self.scan(terminal_id);
        match scan {
            Scan::Literal {
                literal,
                ignore_ascii_case,
            } => gen_match_literal(literal, *ignore_ascii_case, &quote! { input_index }),
            Scan::Trie(set) => {
                let trie = literal_trie_ident();
                let longest_match = trie_method("longest_match", set.ignore_ascii_case);
                let set = literal_set_ident(scan_id);
                quote! { #trie.#longest_match(self.input, input_index, &#set) }
            }
            Scan::Dfa(_) => {
                let scan_fn = format_ident!("scan_dfa_{}", scan_id);
                quote! { self.#scan_fn(input_index, self.input.len()) }
            }
        }
    }

    fn gen_exact_match(
        &self,
        terminal_id: usize,
        start: TokenStream,
        end: TokenStream,
    ) -> TokenStream {
        let (scan_id, scan) = self.scan(terminal_id);
        match scan {
            Scan::Literal {
                literal,
                ignore_ascii_case,
            } => {
                let match_literal = gen_match_literal(literal, *ignore_ascii_case, &start);
                quote! { #match_literal == Some(#end) }
            }
            Scan::Trie(set) => {
                let index_of = trie_method("index_of", set.ignore_ascii_case);
                let set = literal_set_ident(scan_id);
                let trie = literal_trie_ident();
                quote! {
                    match #trie.#index_of(self.input, #start, #end) {
                        Some(index) => #set.contains(index),
                        None => false,
                    }
                }
            }
            Scan::Dfa(_) => {
                let scan_fn = format_ident!("scan_dfa_{}", scan_id);
                quote! { self.#scan_fn(#start, #end) == Some(#end) }
            }
        }
    }

    fn has_literal_sets(&self) -> bool {
        !self.trie_literals.is_empty()
    }

    fn gen_literal_sets(&self) -> TokenStream {
        let trie = self
            .has_literal_sets()
            .then(|| gen_literal_trie(&self.trie_literals));
        let sets = self
            .scans
            .iter()
            .enumerate()
            .filter_map(|(scan_id, scan)| match scan {
                Scan::Trie(set) => Some(gen_literal_set(
                    &literal_set_ident(scan_id),
                    set,
                    &self.trie_literals,
                )),
                _ => None,
            });
        quote! {
            #trie
            #(#sets)*
        }
    }

    fn gen_scan_dfa_methods(&self) -> Vec<TokenStream> {
        self.scans
            .iter()
            .enumerate()
            .filter_map(|(scan_id, scan)| match scan {
                Scan::Dfa(dfa) => Some(gen_scan_dfa_method(scan_id, dfa)),
                _ => None,
            })
            .collect()
    }

    fn syntax_except_ids(&self) -> Vec<TerminalId> {
        let mut except_ids = Vec::new();
        for nonterminal in self.grammar.nonterminals() {
            for alternative in self.grammar.alternatives(nonterminal) {
                for symbol in &alternative.symbols {
                    for e in &symbol.restrictions().excepts {
                        let (terminal, _) = self.grammar.except_terminal(e);
                        let id = self.terminal_ids.get_id(terminal);
                        if !except_ids.contains(&id) {
                            except_ids.push(id);
                        }
                    }
                }
            }
        }
        except_ids
    }

    /// Emits `TERMINAL_FIRST_CHARS`, the characters that can begin a match of
    /// each terminal, indexed by terminal id, and `MATCH_ANY_FIRST_CHARS`, the
    /// characters that can begin a match of a terminal of each `match_any` set,
    /// indexed by set id.
    ///
    /// A nullable terminal matches at every position, so its entry is the full
    /// set, which contains every character, and the test always passes.
    fn gen_first_chars_statics(&self) -> TokenStream {
        let terminals = self.terminal_first_chars();
        let mut sets = vec![vec![]; self.match_any_count];
        for set in self
            .terminal_sets
            .iter()
            .filter(|set| set.id < self.match_any_count)
        {
            let ranges = set
                .terminals
                .iter()
                .flat_map(|terminal| &terminals[self.terminal_ids.get_id(terminal).index()])
                .copied()
                .collect();
            sets[set.id] = merge_ranges(ranges);
        }
        let terminal_count = Literal::usize_unsuffixed(terminals.len());
        let set_count = Literal::usize_unsuffixed(self.match_any_count);
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
    fn terminal_first_chars(&self) -> Vec<Vec<CharRange>> {
        let all = vec![CharRange {
            start: '\0',
            end: char::MAX,
        }];
        let mut terminals: Vec<Vec<CharRange>> = self
            .terminal_ids
            .terminals()
            .map(|terminal| {
                if self.first_chars.is_nullable(terminal) {
                    all.clone()
                } else {
                    self.first_chars.first_chars(terminal).to_vec()
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
    fn gen_first_char_masks_static(&self) -> TokenStream {
        if !self.config.match_memo {
            return quote! {};
        }
        let terminals = self.terminal_first_chars();
        let epsilon = self.terminal_ids.len();
        let masks = (0..128u32).map(|c| {
            let mut mask = vec![0u64; self.memo_words()];
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

    fn gen_scanner_imp(&self) -> TokenStream {
        let name_ident = scanner_ident(&self.grammar.name);
        let match_terminals: Vec<_> = self
            .terminal_ids
            .terminals()
            .enumerate()
            .map(|(id, terminal)| self.gen_match_terminal_method(id, terminal))
            .collect();
        let scan_methods = self.gen_scan_dfa_methods();
        let (impl_generics, ty_generics, new_params, new_body) = if self.config.match_memo {
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
        let match_any_method = self.gen_match_any_method();
        let match_token_and_memoize_method = self.gen_match_token_and_memoize();
        let match_exact_method = self.gen_match_exact_method();
        quote! {
            impl #impl_generics #name_ident #ty_generics {
                pub fn new(#new_params) -> Self {
                    #new_body
                }
                #(#match_terminals)*
                #(#scan_methods)*
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
    fn gen_match_exact_method(&self) -> TokenStream {
        let except_ids = self.syntax_except_ids();
        if except_ids.is_empty() {
            return quote! {};
        }
        let arms: Vec<_> = except_ids
            .iter()
            .map(|id| {
                let exact_match =
                    self.gen_exact_match(id.index(), quote! { start }, quote! { end });
                quote! {
                    #id => #exact_match,
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
    fn gen_match_any_method(&self) -> TokenStream {
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
        if self.config.match_memo {
            let match_any_count = Literal::usize_unsuffixed(self.match_any_count);
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

    fn gen_scanner_trait_impl(&self) -> TokenStream {
        let match_token_method = self.gen_match_token();
        let predict_method = self.gen_predict_method();
        let char_at_method = gen_char_at_method();
        let scanner_name = scanner_ident(&self.grammar.name);
        let scanner_ty = if self.config.match_memo {
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
    fn gen_predict_method(&self) -> TokenStream {
        if !self.config.match_memo {
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

    /// The `match` on `terminal_id` that matches the terminal, or tests for the
    /// end of the input for EOF.
    fn gen_token_dispatch(&self) -> TokenStream {
        let match_terminal_arms: Vec<_> = self
            .terminal_ids
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
        let eof_id = Literal::u16_unsuffixed(self.terminal_ids.eof_id().0);
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
    fn gen_match_token_and_memoize(&self) -> TokenStream {
        if self.terminal_ids.len() == 0 || !self.config.match_memo {
            return quote! {};
        }
        let dispatch = self.gen_token_dispatch();
        quote! {
            #[comment = "Matches `terminal_id` at `input_index` and records the result in the memo.
                         `match_token` looks up the memo and calls this method on a miss."]
            #[comment = "Not inlined into `match_token`. The parser calls `match_token` very often, and most
                         calls return after the first-character test or a memo hit. This method needs its
                         arguments again after the match returns, for the memo insert, and the compiler handles
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

    fn gen_match_token(&self) -> TokenStream {
        let match_token = if self.terminal_ids.len() == 0 {
            quote! {
                None
            }
        } else if self.config.match_memo {
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
            self.gen_token_dispatch()
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

    fn gen_match_terminal_method(&self, id: usize, terminal: &Terminal) -> TokenStream {
        let fn_name = format_ident!("match_terminal_{}", id);
        let longest_match = self.gen_longest_match(id);
        let rule = self
            .grammar
            .lexical_rule(terminal)
            .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name));

        // One check per except, which rejects the match if the except matches
        // exactly the matched span. The excepts that are sets of literals with the
        // same case mode share one walk of the trie, and each of them tests the
        // index of the literal.
        let mut except_checks = Vec::new();
        let mut set_checks: [Vec<Ident>; 2] = Default::default();
        for except in &rule.except {
            let (except_terminal, _) = self.grammar.except_terminal(except);
            let except_id = self.terminal_ids.get_id(except_terminal).index();
            if let (scan_id, Scan::Trie(set)) = self.scan(except_id) {
                set_checks[set.ignore_ascii_case as usize].push(literal_set_ident(scan_id));
            } else {
                let exact_match =
                    self.gen_exact_match(except_id, quote! { input_index }, quote! { end });
                except_checks.push(quote! {
                    if #exact_match {
                        return None;
                    }
                });
            }
        }
        for (ignore_ascii_case, sets) in [false, true].into_iter().zip(set_checks) {
            if sets.is_empty() {
                continue;
            }
            let trie = literal_trie_ident();
            let index_of = trie_method("index_of", ignore_ascii_case);
            let contains = quote! { (#(#sets.contains(index))||*) };
            except_checks.push(quote! {
                if let Some(index) = #trie.#index_of(self.input, input_index, end)
                    && #contains
                {
                    return None;
                }
            });
        }

        // One check per follow restriction, which rejects the match if the
        // restriction terminal matches at the end position.
        let follow_restriction_checks: Vec<_> = rule
            .follow_restriction
            .iter()
            .map(|restriction| {
                let restriction_fn = self.restriction_fn(restriction, "Follow");
                quote! {
                    if self.#restriction_fn(end).is_some() {
                        return None;
                    }
                }
            })
            .collect();

        let precede_restriction_check = rule.precede_restriction.as_ref().map(|restriction| {
            let restriction_fn = self.restriction_fn(restriction, "Precede");
            quote! {
                if input_index > 0 && self.#restriction_fn(input_index - 1).is_some() {
                    return None;
                }
            }
        });

        let comment = rule.to_string();
        let body = if except_checks.is_empty() && follow_restriction_checks.is_empty() {
            longest_match
        } else {
            quote! {
                let end = #longest_match?;
                #(#except_checks)*
                #(#follow_restriction_checks)*
                Some(end)
            }
        };
        quote! {
            #[comment = #comment]
            pub fn #fn_name(&self, input_index: u32) -> Option<u32> {
                #precede_restriction_check
                #body
            }
        }
    }

    fn restriction_fn(&self, restriction: &Identifier, kind: &str) -> Ident {
        let Definition::Terminal(terminal) = self.grammar.definition(restriction.resolve()) else {
            panic!(
                "{kind} restriction {} must refer to a terminal",
                restriction.name
            );
        };
        let restriction_id = self.terminal_ids.get_id(terminal);
        format_ident!("match_terminal_{}", restriction_id.index())
    }
}

fn gen_match_literal(literal: &[u8], ignore_ascii_case: bool, start: &TokenStream) -> TokenStream {
    let literal = Literal::byte_string(literal);
    if ignore_ascii_case {
        quote! { self.input.match_literal_ignore_ascii_case(#start, #literal) }
    } else {
        quote! { self.input.match_literal(#start, #literal) }
    }
}

/// The name of the `LiteralTrie` method `name`, or of its variant that ignores
/// ASCII case.
fn trie_method(name: &str, ignore_ascii_case: bool) -> Ident {
    if ignore_ascii_case {
        format_ident!("{}_ignore_ascii_case", name)
    } else {
        format_ident!("{}", name)
    }
}

fn literal_set_ident(scan_id: usize) -> Ident {
    format_ident!("LITERAL_SET_{}", scan_id)
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

fn gen_char_at_method() -> TokenStream {
    quote! {
        fn char_at(&self, i: u32) -> Option<char> {
            self.input.char_at(i)
        }
    }
}

/// Emits `scan_dfa_<id>`, which runs the DFA from `start` and returns the end
/// of the longest prefix of `[start, end)` that the DFA accepts, or `None` if
/// the DFA does not accept any prefix.
///
/// `match_terminal_<id>` passes the length of the input as `end`, which makes
/// the scan an unbounded longest match. An except check and `match_exact` pass
/// the end of a span to check whether the terminal matches exactly that span.
/// The scan does not read past `end`, so it returns `Some(end)` if the terminal
/// matches the whole span. An unbounded scan could return a longer match and
/// miss the exact one.
fn gen_scan_dfa_method(id: usize, dfa: &Dfa) -> TokenStream {
    assert_eq!(
        dfa.start, 0,
        "subset construction always emits start state 0"
    );
    let fn_name = format_ident!("scan_dfa_{}", id);
    // A DFA without accept states, or without transitions, gets a body
    // without the loop. In such a DFA, the variables of the loop would be
    // unused or never reassigned, which the compiler warns about.
    if dfa.states.iter().all(|state| state.accept.is_none()) {
        return quote! {
            fn #fn_name(&self, _start: u32, _end: u32) -> Option<u32> {
                None
            }
        };
    }
    if dfa.states.iter().all(|state| state.transitions.is_empty()) {
        // The start state is the only state, and it accepts the empty string.
        return quote! {
            fn #fn_name(&self, start: u32, _end: u32) -> Option<u32> {
                Some(start)
            }
        };
    }
    let arms = dfa.states.iter().enumerate().map(|(index, state)| {
        let index = Literal::u32_unsuffixed(index as u32);
        let record_accept = state
            .accept
            .map(|_| quote! { last_accept = Some(position); });
        let transition = gen_dfa_transition(state);
        quote! {
            #index => {
                #record_accept
                #transition
            }
        }
    });
    // If every state accepts, every arm assigns `last_accept` before the loop
    // can end, and the compiler warns that the initial value is never read.
    let last_accept_init = if dfa.states.iter().all(|state| state.accept.is_some()) {
        quote! {}
    } else {
        quote! { = None }
    };
    // Inlined into its callers: `match_terminal_<id>`, and the exact-match
    // checks of the terminals used as excepts.
    quote! {
        #[inline(always)]
        fn #fn_name(&self, start: u32, end: u32) -> Option<u32> {
            let mut state: u32 = 0;
            let mut position = start;
            let mut last_accept #last_accept_init;
            loop {
                match state {
                    #(#arms)*
                    _ => unreachable!(),
                }
                position += 1;
            }
            last_accept
        }
    }
}

/// Emits the code that computes the next state from `state` and the character
/// at `position`. The code breaks out of the loop at `end`, at the end of the
/// input, and at a character without a transition from `state`.
fn gen_dfa_transition(state: &State) -> TokenStream {
    if state.transitions.is_empty() {
        return quote! { break; };
    }
    let mut targets: IndexMap<usize, Vec<TokenStream>> = IndexMap::new();
    for (range, target) in &state.transitions {
        let (start, end) = (range.start, range.end);
        let pattern = if start == end {
            quote! { #start }
        } else {
            quote! { #start..=#end }
        };
        targets.entry(*target).or_default().push(pattern);
    }
    let arms = targets.iter().map(|(target, patterns)| {
        let target = Literal::u32_unsuffixed(*target as u32);
        quote! { #(#patterns)|* => #target, }
    });
    // If the transitions of `state` cover the whole Unicode range, the `match`
    // on the character is exhaustive without the fallback arm, and the
    // compiler would warn that the fallback arm is unreachable.
    let ranges: Vec<CharRange> = state.transitions.iter().map(|(range, _)| *range).collect();
    let fallback = if covers_every_char(&ranges) {
        quote! {}
    } else {
        quote! { _ => break, }
    };
    quote! {
        if position == end {
            break;
        }
        let Some(ch) = self.input.char_at(position) else {
            break;
        };
        state = match ch {
            #(#arms)*
            #fallback
        };
    }
}

/// True if `ranges`, which are sorted and disjoint, cover the whole Unicode
/// range of `char`: U+0000 to U+10FFFF without the surrogates U+D800 to
/// U+DFFF.
fn covers_every_char(ranges: &[CharRange]) -> bool {
    let mut next = Some('\0');
    for range in ranges {
        match next {
            Some(c) if range.start <= c => {
                next = match range.end as u32 + 1 {
                    // The surrogates are not chars.
                    0xD800 => Some('\u{E000}'),
                    after => char::from_u32(after),
                };
            }
            _ => return false,
        }
    }
    next.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_range(start: char, end: char) -> CharRange {
        CharRange { start, end }
    }

    #[test]
    fn ranges_around_the_surrogates_cover_every_char() {
        let ranges = [
            char_range('\0', 'a'),
            char_range('b', '\u{D7FF}'),
            char_range('\u{E000}', char::MAX),
        ];
        assert!(covers_every_char(&ranges));
    }

    #[test]
    fn ranges_with_a_gap_do_not_cover_every_char() {
        let ranges = [char_range('\0', 'a'), char_range('c', char::MAX)];
        assert!(!covers_every_char(&ranges));
        assert!(!covers_every_char(&[char_range('\u{1}', char::MAX)]));
        assert!(!covers_every_char(&[char_range('\0', 'z')]));
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
