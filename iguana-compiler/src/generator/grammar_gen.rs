use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;

use crate::generator::grammar_utils::{grammar_ident, prediction_ident};
use crate::generator::id::{NonterminalIds, SlotIds, TerminalIds};
use crate::generator::terminal_sets::{TerminalSet, TerminalSetKind};
use crate::grammar::def::Grammar;
use crate::grammar::first_chars::FirstChars;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::regex::CharRange;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::{Definition, Nonterminal};
use crate::utils::to_snake_case;
use iguana_runtime::ids::TerminalId;

/// Generates `grammar.rs`: the nonterminal id constants, the grammar type
/// with its `Grammar` implementation, the terminal sets the parser passes to
/// the scanner or refers to in failures, and the predictions of the
/// alternatives of the nonterminals. The statics are public because the
/// parser references only some of them, and a private unused static fails the
/// denied dead-code lint.
#[allow(clippy::too_many_arguments)]
pub fn generate<'a>(
    grammar: &'a Grammar,
    ff: &FirstFollowSets,
    first_chars: &FirstChars,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds<'a>,
    terminal_sets: &[TerminalSet],
    ll1_nonterminals: &FxHashSet<&Nonterminal>,
) -> TokenStream {
    let grammar_type = grammar_ident(&grammar.name);
    let grammar_name = &grammar.name;

    let terminal_set_statics = terminal_set_statics(grammar, terminal_ids, terminal_sets);
    let single_terminal_sets = single_terminal_sets(terminal_ids, terminal_sets);
    let prediction_statics =
        prediction_statics(grammar, ff, first_chars, terminal_ids, ll1_nonterminals);
    // A grammar parsed entirely by LL(1) functions does not have a
    // `Prediction` static, and the import would be unused.
    let prediction_import = if prediction_statics.is_empty() {
        quote! {}
    } else {
        quote! {
            use iguana_runtime::prediction::Prediction;
        }
    };
    // The width of `Grammar::Alternatives` in 64-bit words: enough for the
    // nonterminal with the most alternatives.
    let alternative_words = grammar
        .nonterminals()
        .map(|nonterminal| grammar.alternatives(nonterminal).len().div_ceil(64))
        .fold(1, usize::max);
    let alternative_words = Literal::usize_unsuffixed(alternative_words);

    let nonterminals = nonterminal_ids.nonterminals().map(|n| {
        let nonterminal_name = &n.name;
        let display_name = n.display_name();
        quote! {
            Nonterminal {
                name: #nonterminal_name,
                display_name: #display_name,
            }
        }
    });

    let layout_name = grammar
        .layout
        .as_ref()
        .and_then(|s| s.as_identifier())
        .map(|i| i.name.as_str());

    // The entry points, in `.iggy` source order: the derived nonterminals
    // (start wrappers, EBNF expansions, exclusion and precedence desugarings)
    // have no start wrapper and are left out, and the rest sort by declaration
    // position.
    let mut display_order: Vec<&Nonterminal> = nonterminal_ids
        .nonterminals()
        .filter(|n| !n.is_derived())
        .collect();
    display_order.sort_by_key(|n| grammar.source_index(&n.name));
    let display_order_names: Vec<&str> = display_order.iter().map(|n| n.name.as_str()).collect();

    let nonterminal_id_consts: Vec<_> = nonterminal_ids
        .nonterminals()
        .enumerate()
        .map(|(i, n)| {
            let const_name = format_ident!("{}", to_snake_case(&n.name).to_uppercase());
            let index = Literal::usize_unsuffixed(i);
            quote! { pub const #const_name: NonterminalId = NonterminalId(#index); }
        })
        .collect();

    let nonterminal_id_arms: Vec<_> = nonterminal_ids
        .nonterminals()
        .map(|n| {
            let name = &n.name;
            let const_name = format_ident!("{}", to_snake_case(name).to_uppercase());
            quote! { #name => Some(#const_name), }
        })
        .collect();

    let terminals: Vec<_> = terminal_ids
        .terminals()
        .map(|t| {
            let terminal_name = &t.name;
            quote! { Terminal { name: #terminal_name } }
        })
        .collect();

    let slots = slot_ids.slots().map(|s| {
        let display_name = slot_ids.display_name(&slot_ids.get_id(s));
        let position = Literal::usize_unsuffixed(s.pos());
        quote! {
            Slot { display_name: #display_name, position: #position }
        }
    });

    let first_slots = nonterminal_ids.nonterminals().map(|nonterminal| {
        let slots = grammar
            .alternatives(nonterminal)
            .iter()
            .map(|alternative| slot_ids.get_id(&Slot::new(nonterminal, alternative, 0)));
        quote! { &[#(#slots),*] }
    });

    let layout_name = layout_name
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! { None });
    let layout_terminals = layout_terminal_ids(grammar, terminal_ids);

    quote! {
        use iguana_runtime::{
            grammar::{Grammar, Nonterminal, Slot, Terminal},
            ids::{NonterminalId, SlotId, TerminalId},
            scanner::TerminalSet,
            utils::bit_set::BitSet,
        };
        #prediction_import

        #(#nonterminal_id_consts)*

        pub struct #grammar_type;

        impl Grammar for #grammar_type {
            type Alternatives = BitSet<#alternative_words>;

            const NAME: &'static str = #grammar_name;

            const NONTERMINALS: &'static [Nonterminal] = &[#(#nonterminals),*];

            const DISPLAY_ORDER: &'static [&'static str] = &[#(#display_order_names),*];

            const TERMINALS: &'static [Terminal] = &[
                #(#terminals,)*
                Terminal { name: "Epsilon" },
                Terminal { name: "EOF" },
            ];

            const SLOTS: &'static [Slot] = &[#(#slots),*];

            const FIRST_SLOTS: &'static [&'static [SlotId]] = &[#(#first_slots),*];

            const LAYOUT_NAME: Option<&'static str> = #layout_name;

            const LAYOUT_TERMINALS: &'static [TerminalId] = &[#(#layout_terminals),*];

            #[comment = "A failed terminal match refers to a static terminal set like every other
                         failure, so the error reporting path needs to reach a terminal set
                         from a terminal id. This slice, indexed by terminal id, serves only that."]
            const SINGLE_TERMINAL_SETS: &'static [TerminalSet] = &[#(#single_terminal_sets),*];

            fn nonterminal_id(name: &str) -> Option<NonterminalId> {
                match name {
                    #(#nonterminal_id_arms)*
                    _ => None,
                }
            }
        }

        #(#terminal_set_statics)*

        #(#prediction_statics)*
    }
}

/// The statics for the terminal sets, for example:
///
/// ```text
/// // Grammar { WS, LineComment, EOF }
/// pub static FOLLOW_SET_GRAMMAR: TerminalSet = TerminalSet {
///     id: 0,
///     terminals: &[TerminalId(8), TerminalId(10), TerminalId(39)],
/// };
/// ```
///
/// The parser passes these sets to the scanner for matching, and a recorded
/// failure refers to the set the parser expected. What each kind of set
/// contains and who uses it is documented on `TerminalSetKind`.
fn terminal_set_statics(
    grammar: &Grammar,
    terminal_ids: &TerminalIds,
    terminal_sets: &[TerminalSet],
) -> Vec<TokenStream> {
    terminal_sets
        .iter()
        .map(|set| {
            let name = format_ident!("{}", terminal_set_name(set, grammar));
            let comment = terminal_set_comment(set);
            let ids: Vec<_> = set
                .terminals
                .iter()
                .map(|t| terminal_ids.get_id(t))
                .collect();
            let set_id = Literal::usize_unsuffixed(set.id);
            quote! {
                #[comment = #comment]
                pub static #name: TerminalSet = TerminalSet { id: #set_id, terminals: &[#(#ids),*] };
            }
        })
        .collect()
}

/// `SINGLE_TERMINAL_SETS` is a slice of `TerminalSet`s, each holding a single
/// terminal: entry `i` holds the set for the terminal with id `i`. Failure
/// reporting only accepts a `TerminalSet`, and this slice is the way to reach a
/// terminal set from a single terminal id. These sets are not used for
/// matching by the scanner.
fn single_terminal_sets(
    terminal_ids: &TerminalIds,
    terminal_sets: &[TerminalSet],
) -> Vec<TokenStream> {
    let first_id = terminal_sets
        .iter()
        .map(|set| set.id + 1)
        .max()
        .unwrap_or(0);
    // Plus the synthetic epsilon and EOF terminals: this slice is indexed like
    // `TERMINALS`, so it must contain those two as well.
    (0..terminal_ids.len() + 2)
        .map(|index| {
            let set_id = Literal::usize_unsuffixed(first_id + index);
            let id = TerminalId(index as u16);
            quote! { TerminalSet { id: #set_id, terminals: &[#id] } }
        })
        .collect()
}

/// The ids of the terminals reachable from the grammar's layout definition,
/// sorted by id. Empty when the grammar declares no layout.
fn layout_terminal_ids(grammar: &Grammar, terminal_ids: &TerminalIds) -> Vec<TerminalId> {
    let Some(layout) = grammar.layout.as_ref().and_then(|s| s.as_identifier()) else {
        return vec![];
    };

    let mut pending = vec![layout.resolve()];
    let mut visited = FxHashSet::default();
    let mut terminals = vec![];
    while let Some(definition_id) = pending.pop() {
        if !visited.insert(definition_id) {
            continue;
        }
        match grammar.definition(definition_id) {
            Definition::Terminal(terminal) => {
                terminals.push(terminal_ids.get_id(terminal));
            }
            Definition::Nonterminal(nonterminal) => {
                for alternative in grammar.alternatives(nonterminal) {
                    for symbol in &alternative.symbols {
                        if let Some(identifier) = symbol.as_identifier() {
                            pending.push(identifier.resolve());
                        }
                    }
                }
            }
        }
    }
    terminals.sort_unstable_by_key(|terminal| terminal.0);
    terminals
}

/// The `Prediction` statics, one for each nonterminal the parser calls
/// through the GLL path: every nonterminal that is not parsed by an LL(1)
/// function, except the start wrappers, which nothing calls. For example:
///
/// ```text
/// // Rule
/// // 0: SyntaxRule
/// // 1: RegexRule
/// pub static PREDICTION_RULE: Prediction = {
///     // WS
///     const TERMINALS_1: &[(TerminalId, u16)] = &[(TerminalId(8), 0), …];
///     …
///     Prediction {
///         ascii: [&[], …, TERMINALS_1, …],
///         non_ascii: &[],
///         end_of_input: &[],
///     }
/// };
/// ```
///
/// The comment above a static names the nonterminal and lists its
/// alternatives by index, which is the index in the pairs. The pairs of the
/// nonterminal (`terminal_alternatives`) are split into lists by character:
///
/// - `ascii[c]` holds the pairs whose terminal can begin with `c`.
/// - `non_ascii` holds the pairs whose terminal can begin with a character
///   outside ASCII.
/// - `end_of_input` holds the pairs of the end-of-input terminal.
/// - A pair with a nullable terminal goes into every list.
///
/// Each distinct nonempty list is emitted once, as a constant named
/// `TERMINALS_n` and commented with its terminals, and the fields refer to
/// it. An empty list is written inline as `&[]`. The constants are local to
/// the initializer of the static, so their names do not collide across
/// statics.
fn prediction_statics(
    grammar: &Grammar,
    ff: &FirstFollowSets,
    first_chars: &FirstChars,
    terminal_ids: &TerminalIds,
    ll1_nonterminals: &FxHashSet<&Nonterminal>,
) -> Vec<TokenStream> {
    // The characters that can begin a match of each terminal, indexed by
    // terminal id. `None` stands for a nullable terminal, whose pairs go into
    // every list. The epsilon and end-of-input terminals come after the
    // grammar's terminals, and the end-of-input terminal matches at no
    // character.
    let mut terminal_first_chars: Vec<Option<&[CharRange]>> = terminal_ids
        .terminals()
        .map(|terminal| {
            (!first_chars.is_nullable(terminal)).then(|| first_chars.first_chars(terminal))
        })
        .collect();
    terminal_first_chars.push(None);
    terminal_first_chars.push(Some(&[]));
    let mut terminal_names: Vec<&str> = terminal_ids
        .terminals()
        .map(|terminal| terminal.name.as_str())
        .collect();
    terminal_names.extend(["Epsilon", "EOF"]);
    let eof = terminal_ids.eof_id();
    grammar
        .nonterminals()
        .filter(|nonterminal| {
            !ll1_nonterminals.contains(nonterminal) && !grammar.is_start(nonterminal)
        })
        .map(|nonterminal| {
            let pairs = terminal_alternatives(grammar, ff, terminal_ids, nonterminal);
            let ascii: Vec<_> = (0..128u8)
                .map(|c| {
                    let c = char::from(c);
                    pairs_starting_with(&pairs, &terminal_first_chars, |range| {
                        range.start <= c && c <= range.end
                    })
                })
                .collect();
            let non_ascii =
                pairs_starting_with(&pairs, &terminal_first_chars, |range| !range.end.is_ascii());
            let end_of_input: Vec<_> = pairs
                .iter()
                .copied()
                .filter(|&(terminal_id, _)| {
                    terminal_id == eof || terminal_first_chars[terminal_id.index()].is_none()
                })
                .collect();
            let mut lists: Vec<(Vec<(TerminalId, u16)>, Ident)> = vec![];
            let mut constants = vec![];
            let mut list_expr = |list: &Vec<(TerminalId, u16)>| -> TokenStream {
                if list.is_empty() {
                    return quote! { &[] };
                }
                if let Some((_, ident)) = lists.iter().find(|(existing, _)| existing == list) {
                    return quote! { #ident };
                }
                let ident = format_ident!("TERMINALS_{}", lists.len() + 1);
                let mut names: Vec<_> = list
                    .iter()
                    .map(|(terminal_id, _)| terminal_names[terminal_id.index()])
                    .collect();
                names.dedup();
                let comment = names.join(", ");
                let entries = list.iter().map(|(terminal_id, alternative)| {
                    let alternative = Literal::u16_unsuffixed(*alternative);
                    quote! { (#terminal_id, #alternative) }
                });
                constants.push(quote! {
                    #[comment = #comment]
                    const #ident: &[(TerminalId, u16)] = &[#(#entries),*];
                });
                lists.push((list.clone(), ident.clone()));
                quote! { #ident }
            };
            let ascii: Vec<_> = ascii.iter().map(&mut list_expr).collect();
            let non_ascii = list_expr(&non_ascii);
            let end_of_input = list_expr(&end_of_input);
            // The nonterminal, then one line for each alternative with its
            // index, so the comment names what each alternative index stands
            // for.
            let nonterminal_name = &nonterminal.name;
            let alternative_lines =
                grammar
                    .alternatives(nonterminal)
                    .iter()
                    .enumerate()
                    .map(|(index, alternative)| {
                        if alternative.symbols.is_empty() {
                            format!("{index}: ()")
                        } else {
                            format!("{index}: {alternative}")
                        }
                    });
            let name = prediction_ident(nonterminal);
            quote! {
                #[comment = #nonterminal_name]
                #(#[comment = #alternative_lines])*
                pub static #name: Prediction = {
                    #(#constants)*
                    Prediction {
                        ascii: [#(#ascii),*],
                        non_ascii: #non_ascii,
                        end_of_input: #end_of_input,
                    }
                };
            }
        })
        .collect()
}

/// The pairs whose terminal can begin with a character in one of the ranges
/// that `in_range` accepts. A pair with a nullable terminal is always kept,
/// because the terminal matches the empty string.
fn pairs_starting_with(
    pairs: &[(TerminalId, u16)],
    terminal_first_chars: &[Option<&[CharRange]>],
    in_range: impl Fn(&CharRange) -> bool,
) -> Vec<(TerminalId, u16)> {
    pairs
        .iter()
        .copied()
        .filter(|(terminal_id, _)| {
            terminal_first_chars[terminal_id.index()]
                .is_none_or(|ranges| ranges.iter().any(&in_range))
        })
        .collect()
}

/// The pairs of `nonterminal`: each terminal with each alternative it
/// predicts, sorted by terminal id and then by alternative. A terminal of
/// `FIRST(α)` predicts the alternative `α`, and a terminal of the FOLLOW set
/// of `nonterminal` predicts every nullable alternative.
fn terminal_alternatives(
    grammar: &Grammar,
    ff: &FirstFollowSets,
    terminal_ids: &TerminalIds,
    nonterminal: &Nonterminal,
) -> Vec<(TerminalId, u16)> {
    let mut pairs = vec![];
    for (index, alternative) in grammar.alternatives(nonterminal).iter().enumerate() {
        let index = index as u16;
        for terminal in ff.prediction_set(nonterminal, alternative).keys() {
            pairs.push((terminal_ids.get_id(terminal), index));
        }
    }
    pairs.sort_by_key(|&(terminal_id, alternative)| (terminal_id.index(), alternative));
    pairs.dedup();
    pairs
}

/// Identifier of the static for `set`, e.g. `FOLLOW_RESTRICTION_E_ALT0_POS1`.
fn terminal_set_name(set: &TerminalSet, grammar: &Grammar) -> String {
    let upper = |nonterminal: &Nonterminal| to_snake_case(&nonterminal.name).to_uppercase();
    match &set.kind {
        TerminalSetKind::Follow(nonterminal) => format!("FOLLOW_SET_{}", upper(nonterminal)),
        TerminalSetKind::First(nonterminal) => format!("FIRST_SET_{}", upper(nonterminal)),
        TerminalSetKind::FollowRestriction(slot) => format!(
            "FOLLOW_RESTRICTION_{}_ALT{}_POS{}",
            upper(slot.head()),
            slot.alternative_index(grammar),
            slot.pos()
        ),
        TerminalSetKind::LayoutAwareFollowRestriction(slot) => format!(
            "LAYOUT_AWARE_FOLLOW_RESTRICTION_{}_ALT{}_POS{}",
            upper(slot.head()),
            slot.alternative_index(grammar),
            slot.pos()
        ),
        TerminalSetKind::Except(slot) => format!(
            "EXCEPT_{}_ALT{}_POS{}",
            upper(slot.head()),
            slot.alternative_index(grammar),
            slot.pos()
        ),
    }
}

/// Comment line above the static for `set`: the grammar position, then the
/// terminals, e.g. `E : . E "+" E { "a" }`.
fn terminal_set_comment(set: &TerminalSet) -> String {
    let position = match &set.kind {
        TerminalSetKind::Follow(nonterminal) | TerminalSetKind::First(nonterminal) => {
            nonterminal.name.clone()
        }
        TerminalSetKind::FollowRestriction(slot) => format!("{} !>>", slot.name()),
        TerminalSetKind::LayoutAwareFollowRestriction(slot) => format!("{} !>>>", slot.name()),
        TerminalSetKind::Except(slot) => format!("{} \\", slot.name()),
    };
    let names: Vec<_> = set.terminals.iter().map(|t| t.name.clone()).collect();
    format!("{position} {{ {} }}", names.join(", "))
}
