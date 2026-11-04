use std::collections::BTreeSet;

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

use crate::grammar::regex::Regex;

/// The literals of a regex that is an alternation of non-empty ASCII
/// literals. If `ignore_ascii_case` is true, the ASCII letters of every literal
/// match in both cases, and the literals are in lowercase.
pub struct LiteralSet {
    /// A sorted list of the literals, without duplicates.
    pub literals: Vec<Vec<u8>>,
    pub ignore_ascii_case: bool,
}

impl LiteralSet {
    /// Returns the literal set of `regex`, or `None` if `regex` is not a
    /// non-empty alternation of literals, which may be nested.
    ///
    /// A literal is a non-empty sequence of ASCII characters. A character is a
    /// character itself, a class with one character, or a class of one ASCII
    /// letter in both cases. All the letters of a literal set match in the same
    /// mode.
    pub fn from_regex(regex: &Regex) -> Option<LiteralSet> {
        let mut case = None;
        let mut literals = Vec::new();
        collect_literals(regex, &mut literals, &mut case)?;
        if literals.is_empty() || literals.iter().any(Vec::is_empty) {
            return None;
        }
        literals.sort();
        literals.dedup();
        Some(LiteralSet {
            literals,
            ignore_ascii_case: case == Some(Case::IgnoreAscii),
        })
    }
}

/// The parts of a `LiteralTrie` of `literals`, a sorted list in which the index
/// of a literal is its position: the characters of the alphabet, sorted, the
/// transitions, and the literal that each state accepts.
fn trie_parts(literals: &[Vec<u8>]) -> (Vec<u8>, Vec<u16>, Vec<Option<u16>>) {
    let alphabet: Vec<u8> = literals
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<u8>>()
        .into_iter()
        .collect();
    let mut transitions = vec![0; alphabet.len()];
    let mut accepts = vec![None];
    for (index, literal) in literals.iter().enumerate() {
        let mut state = 0;
        for c in literal {
            let slot = state * alphabet.len() + alphabet.binary_search(c).unwrap();
            if transitions[slot] == 0 {
                transitions[slot] = accepts.len() as u16;
                transitions.resize(transitions.len() + alphabet.len(), 0);
                accepts.push(None);
            }
            state = transitions[slot] as usize;
        }
        accepts[state] = Some(index as u16);
    }
    (alphabet, transitions, accepts)
}

/// The name of the generated `LiteralTrie`.
pub fn literal_trie_ident() -> Ident {
    format_ident!("LITERAL_TRIE")
}

/// Emits the `LiteralTrie` of `literals`, a sorted list in which the index of
/// a literal is its position, with a comment that lists the literals by index.
pub fn gen_literal_trie(literals: &[Vec<u8>]) -> TokenStream {
    let name = literal_trie_ident();
    let index_comments = literals.iter().enumerate().map(|(index, literal)| {
        let comment = format!(
            "{index}: {}",
            String::from_utf8_lossy(literal).escape_debug()
        );
        quote! { #[comment = #comment] }
    });
    let (alphabet, transitions, accepts) = trie_parts(literals);
    let max_len = Literal::u32_unsuffixed(max_len(literals));
    let alphabet = Literal::byte_string(&alphabet);
    let transitions = transitions.into_iter().map(Literal::u16_unsuffixed);
    let accepts = accepts.iter().map(|accept| match accept {
        Some(index) => {
            let index = Literal::u16_unsuffixed(*index);
            quote! { Some(#index) }
        }
        None => quote! { None },
    });
    quote! {
        #(#index_comments)*
        static #name: LiteralTrie = LiteralTrie {
            max_len: #max_len,
            alphabet: Alphabet::from_chars(#alphabet),
            transitions: &[#(#transitions),*],
            accepts: &[#(#accepts),*],
        };
    }
}

fn max_len(literals: &[Vec<u8>]) -> u32 {
    literals
        .iter()
        .map(|literal| literal.len() as u32)
        .max()
        .unwrap()
}

/// Emits `static #name: BitSet<W>`, the set of the indices of the literals of
/// `set` in `trie_literals`, the sorted literals of their trie. `W` is the
/// number of 64-bit words that hold an index for every literal of the trie.
pub fn gen_literal_set(name: &Ident, set: &LiteralSet, trie_literals: &[Vec<u8>]) -> TokenStream {
    let width = Literal::usize_unsuffixed(trie_literals.len().div_ceil(64));
    let members = literal_set_members(set, trie_literals)
        .into_iter()
        .map(Literal::usize_unsuffixed);
    quote! {
        static #name: BitSet<#width> = BitSet::from_members(&[#(#members),*]);
    }
}

/// The indices of the literals of `set` in `trie_literals`, in increasing
/// order.
fn literal_set_members(set: &LiteralSet, trie_literals: &[Vec<u8>]) -> Vec<usize> {
    set.literals
        .iter()
        .map(|literal| trie_literals.binary_search(literal).unwrap())
        .collect()
}

/// How the letters of a regex match: exactly, or in both cases. Characters
/// other than letters match the same in both modes.
#[derive(Clone, Copy, PartialEq)]
enum Case {
    Exact,
    IgnoreAscii,
}

/// Records that a letter of the regex requires `required` in `case`, and
/// returns `None` if the regex already requires the other mode.
fn require(case: &mut Option<Case>, required: Case) -> Option<()> {
    if case.is_some_and(|case| case != required) {
        return None;
    }
    *case = Some(required);
    Some(())
}

/// Adds the literals of `regex`, an alternation of literals, to `literals`.
/// Returns `None` if `regex` has another shape. `case` holds the mode the
/// letters seen so far require.
fn collect_literals(
    regex: &Regex,
    literals: &mut Vec<Vec<u8>>,
    case: &mut Option<Case>,
) -> Option<()> {
    match regex {
        Regex::Alt(items) => {
            for item in items {
                collect_literals(item, literals, case)?;
            }
        }
        Regex::Seq(items) => {
            let literal = items
                .iter()
                .map(|item| literal_char(item, case))
                .collect::<Option<_>>()?;
            literals.push(literal);
        }
        item => literals.push(vec![literal_char(item, case)?]),
    }
    Some(())
}

/// The character that `regex` contributes to a literal, in lowercase if
/// `regex` is an ASCII letter in both cases, or `None` if `regex` is not a
/// character of a literal.
fn literal_char(regex: &Regex, case: &mut Option<Case>) -> Option<u8> {
    match regex {
        Regex::Char(c) => exact_char(*c, case),
        Regex::CharRange(range) if range.start == range.end => exact_char(range.start, case),
        Regex::CharClass(class) if !class.negated => {
            let mut chars = Vec::new();
            for range in &class.ranges {
                for c in range.start..=range.end {
                    if !chars.contains(&c) {
                        if chars.len() == 2 {
                            return None;
                        }
                        chars.push(c);
                    }
                }
            }
            match chars.as_slice() {
                [c] => exact_char(*c, case),
                [a, b] if a.is_ascii_alphabetic() && a.eq_ignore_ascii_case(b) && a != b => {
                    require(case, Case::IgnoreAscii)?;
                    Some(a.to_ascii_lowercase() as u8)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// The character `c` as a character of a literal that matches only itself.
fn exact_char(c: char, case: &mut Option<Case>) -> Option<u8> {
    if !c.is_ascii() {
        return None;
    }
    if c.is_ascii_alphabetic() {
        require(case, Case::Exact)?;
    }
    Some(c as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::regex::{CharClass, CharRange};
    use iguana_runtime::{
        char_set::Alphabet, input::Input, scanner::LiteralTrie, utils::bit_set::BitSet,
    };

    fn literal(text: &str) -> Regex {
        Regex::Seq(text.chars().map(Regex::Char).collect())
    }

    fn ignore_case_literal(text: &str) -> Regex {
        let class = |c: char| {
            Regex::CharClass(CharClass {
                ranges: [c.to_ascii_lowercase(), c.to_ascii_uppercase()]
                    .map(|c| CharRange { start: c, end: c })
                    .to_vec(),
                negated: false,
            })
        };
        Regex::Seq(text.chars().map(class).collect())
    }

    fn literals_of(regex: &Regex) -> Option<(Vec<String>, bool)> {
        LiteralSet::from_regex(regex).map(|set| {
            let literals = set
                .literals
                .into_iter()
                .map(|literal| String::from_utf8(literal).unwrap())
                .collect();
            (literals, set.ignore_ascii_case)
        })
    }

    #[test]
    fn an_alternation_of_literals_has_its_literals() {
        let regex = Regex::Alt(vec![literal("while"), literal("for"), literal("for")]);
        let literals = ["for", "while"].map(String::from).to_vec();
        assert_eq!(literals_of(&regex), Some((literals, false)));
    }

    #[test]
    fn case_pairs_match_in_both_cases() {
        let regex = Regex::Alt(vec![ignore_case_literal("select"), literal("_")]);
        let literals = ["_", "select"].map(String::from).to_vec();
        assert_eq!(literals_of(&regex), Some((literals, true)));
    }

    #[test]
    fn a_nested_alternation_has_the_literals_of_its_alternatives() {
        let inner = Regex::Alt(vec![literal("b"), literal("c")]);
        let regex = Regex::Alt(vec![literal("a"), inner]);
        let literals = ["a", "b", "c"].map(String::from).to_vec();
        assert_eq!(literals_of(&regex), Some((literals, false)));
    }

    /// The `LiteralTrie` that `gen_literal_trie` emits for `literals`.
    fn literal_trie(literals: &[Vec<u8>]) -> LiteralTrie {
        let (alphabet, transitions, accepts) = trie_parts(literals);
        LiteralTrie {
            max_len: max_len(literals),
            alphabet: Alphabet::from_chars(&alphabet),
            transitions: transitions.leak(),
            accepts: accepts.leak(),
        }
    }

    #[test]
    fn a_trie_finds_the_literals_of_its_sets() {
        let mut regexes: Vec<Regex> = (0..300)
            .map(|i| ignore_case_literal(&format!("w{i}x")))
            .collect();
        regexes.extend(["time", "timestamp", "starts", "status"].map(ignore_case_literal));
        let large = LiteralSet::from_regex(&Regex::Alt(regexes)).unwrap();
        let small = LiteralSet::from_regex(&ignore_case_literal("time")).unwrap();
        let trie_literals = large.literals.clone();
        let trie = literal_trie(&trie_literals);
        let bit_set = |set: &LiteralSet| -> BitSet<5> {
            BitSet::from_members(&literal_set_members(set, &trie_literals))
        };
        let (large, small) = (bit_set(&large), bit_set(&small));
        let input = Input::from("TimeStamp w17X status stats".to_string());
        let in_set = |set: &BitSet<5>, start, end| match trie
            .index_of_ignore_ascii_case(&input, start, end)
        {
            Some(index) => set.contains(index),
            None => false,
        };
        assert_eq!(trie.index_of(&input, 0, 9), None);
        assert!(in_set(&large, 0, 9));
        assert!(in_set(&large, 0, 4));
        assert!(in_set(&small, 0, 4));
        assert!(!in_set(&small, 0, 9));
        assert_eq!(trie.index_of_ignore_ascii_case(&input, 0, 5), None);
        assert!(in_set(&large, 10, 14));
        assert!(in_set(&large, 15, 21));
        assert_eq!(trie.index_of_ignore_ascii_case(&input, 22, 27), None);
        assert_eq!(
            trie.longest_match_ignore_ascii_case(&input, 0, &large),
            Some(9)
        );
        assert_eq!(
            trie.longest_match_ignore_ascii_case(&input, 0, &small),
            Some(4)
        );
        assert_eq!(
            trie.longest_match_ignore_ascii_case(&input, 22, &large),
            None
        );
        assert_eq!(trie.longest_match(&input, 0, &large), None);
    }

    #[test]
    fn other_languages_do_not_have_literals() {
        let digits = Regex::CharClass(CharClass {
            ranges: vec![CharRange {
                start: '0',
                end: '9',
            }],
            negated: false,
        });
        let mixed = Regex::Alt(vec![ignore_case_literal("select"), literal("from")]);
        let combinations = Regex::Seq(vec![
            Regex::Alt(vec![literal("a"), literal("b")]),
            literal("c"),
        ]);
        let empty_string = Regex::Opt(Box::new(literal("a")));
        let empty_language = Regex::Alt(vec![]);
        let plus = Regex::Plus(Box::new(literal("a")));
        for regex in [
            combinations,
            plus,
            digits,
            mixed,
            empty_string,
            empty_language,
            literal("é"),
        ] {
            assert!(LiteralSet::from_regex(&regex).is_none());
        }
    }
}
