use crate::ids::TerminalId;
use crate::utils::bit_set::BitSet;

/// `Prediction` keeps the mapping from characters to the terminals that can
/// predict an alternative of a nonterminal. An alternative `α` is predicted by
/// the terminals of `FIRST(α)`, and, if `α` is nullable, by the terminals of
/// the FOLLOW set of the nonterminal.
///
/// The `Prediction` for `Rule = SyntaxRule | RegexRule` looks like the
/// following:
///
/// ```text
/// // Rule
/// // 0: SyntaxRule
/// // 1: RegexRule
/// pub static PREDICTION_RULE: Prediction = Prediction {
///     ascii: [
///         &[],                                    // '\0'
///         …
///         &[(WS, 0), (WS, 1)],                    // '\t'
///         &[(WS, 0), (WS, 1)],                    // '\n'
///         …
///         &[(WS, 0), (WS, 1)],                    // ' '
///         …
///         &[(LineComment, 0), (LineComment, 1)],  // '/'
///         …
///         &[("@NoLayout", 0), ("@Layout", 0), ("@Layout", 1),
///           ("@Identifier", 1), ("@Regex", 1)],   // '@'
///         &[(Identifier, 0)],                     // 'A'
///         …
///         &[(Identifier, 0)],                     // 'z'
///         …
///     ],
///     non_ascii: &[],
///     end_of_input: &[],
/// };
/// ```
///
/// The `ascii` table maps each character to a list of pairs of a terminal that
/// can begin with the character and an alternative the terminal predicts.
/// `Scanner::predict` first selects the list by the character at the
/// position, which does not need scanning, and then matches only the
/// terminals of that list.
///
/// `non_ascii` is the list for every character outside ASCII, and
/// `end_of_input` is the list at the end of the input, which holds the EOF
/// terminal and the nullable terminals.
///
/// The example is for illustration only. The generated code writes terminal
/// ids in place of the names. It declares each distinct list once, as a
/// constant, and the entries refer to it.
#[derive(Debug)]
pub struct Prediction {
    /// The pairs to try at each ASCII character, indexed by code point.
    pub ascii: [&'static [(TerminalId, u16)]; 128],
    /// The pairs to try at a character outside ASCII.
    pub non_ascii: &'static [(TerminalId, u16)],
    /// The pairs to try at the end of the input.
    pub end_of_input: &'static [(TerminalId, u16)],
}

impl Prediction {
    /// The pairs to try at a position whose character is `c`: the pairs whose
    /// terminal can begin with `c`. `c` is `None` at the end of the input.
    pub fn terminals_at(&self, c: Option<char>) -> &'static [(TerminalId, u16)] {
        match c {
            Some(c) if c.is_ascii() => self.ascii[c as usize],
            Some(_) => self.non_ascii,
            None => self.end_of_input,
        }
    }

    /// The terminals of all pairs, in ascending id order. This method is only
    /// called for a failed call, when no alternative of the nonterminal is
    /// predicted at its position.
    pub fn terminals(&self) -> Vec<TerminalId> {
        let mut terminals: Vec<_> = self
            .ascii
            .iter()
            .chain([&self.non_ascii, &self.end_of_input])
            .flat_map(|pairs| pairs.iter().map(|&(terminal, _)| terminal))
            .collect();
        terminals.sort_unstable_by_key(|terminal| terminal.0);
        terminals.dedup();
        terminals
    }
}

/// Represents a set of alternatives (by index) returned by `Scanner::predict`.
/// The set is a `BitSet<W>`, and the width, `W`, is the number of 64-bit
/// words, chosen for each grammar at generation time. The generic runtime
/// code, however, cannot write the type `BitSet<W>`, because the width
/// would have to come from the grammar, as in `BitSet<{ G::WORDS }>`, and
/// stable Rust does not allow that as a const generic argument. Each grammar
/// therefore declares its set type as `Grammar::Alternatives`, and the
/// runtime code uses it through this trait.
pub trait AlternativeSet: Copy + Default {
    fn contains(&self, alternative: usize) -> bool;
    fn insert(&mut self, alternative: usize);
    fn is_empty(&self) -> bool;
    /// The alternatives from the last to the first.
    fn iter_descending(&self) -> impl Iterator<Item = usize>;
}

impl<const W: usize> AlternativeSet for BitSet<W> {
    fn contains(&self, alternative: usize) -> bool {
        BitSet::contains(self, alternative)
    }

    fn insert(&mut self, alternative: usize) {
        BitSet::insert(self, alternative);
    }

    fn is_empty(&self) -> bool {
        BitSet::is_empty(self)
    }

    fn iter_descending(&self) -> impl Iterator<Item = usize> {
        BitSet::iter_descending(self)
    }
}
