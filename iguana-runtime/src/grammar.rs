use crate::ids::{NonterminalId, SlotId, TerminalId};
use crate::prediction::AlternativeSet;
use crate::scanner::TerminalSet;

/// The runtime's view of `iguana_compiler::grammar::symbols::Nonterminal`.
/// It holds only the information the runtime needs.
pub struct Nonterminal {
    pub name: &'static str,
    pub display_name: &'static str,
}

/// The runtime's view of `iguana_compiler::grammar::symbols::Terminal`.
/// It holds only the information the runtime needs.
pub struct Terminal {
    pub name: &'static str,
}

/// The runtime's view of a grammar slot, a position in an alternative. It
/// holds only the information the runtime needs.
pub struct Slot {
    pub display_name: &'static str,
    /// The number of symbols before the slot in its alternative.
    pub position: u16,
}

/// The runtime's view of `iguana_compiler::grammar::def::Grammar`. It holds
/// only the information the runtime needs. The generator implements the trait
/// once per grammar, and the runtime CLI, the wasm wrapper, and the parser
/// reach the grammar's tables and lookups through it.
pub trait Grammar {
    /// The set of predicted alternatives, a concrete `BitSet<W>`. `W` is the
    /// number of 64-bit words that fits the nonterminal with the most
    /// alternatives.
    type Alternatives: AlternativeSet;
    /// The name of the grammar.
    const NAME: &'static str;
    /// Every nonterminal, indexed by `NonterminalId`.
    const NONTERMINALS: &'static [Nonterminal];
    /// The names of the nonterminals the grammar text declares, in source
    /// order. These are the entry points.
    const DISPLAY_ORDER: &'static [&'static str];
    /// Every terminal, indexed by `TerminalId`. The last two entries are the
    /// synthetic epsilon and end-of-file terminals.
    const TERMINALS: &'static [Terminal];
    /// Every grammar slot, indexed by `SlotId`.
    const SLOTS: &'static [Slot];
    /// The first slot of each alternative of each nonterminal, indexed by
    /// `NonterminalId` and then by alternative index.
    const FIRST_SLOTS: &'static [&'static [SlotId]];
    /// The name of the layout nonterminal, or `None` when the grammar declares
    /// no layout.
    const LAYOUT_NAME: Option<&'static str>;
    /// The terminals reachable from the layout definition, or an empty slice
    /// when the grammar declares no layout.
    const LAYOUT_TERMINALS: &'static [TerminalId];
    /// One set per terminal holding just that terminal, indexed by
    /// `TerminalId`. A recorded failure refers to a static terminal set, and a
    /// failed terminal match has only the terminal's id in hand, so the error
    /// reporting path needs a way from the id to a set. This slice serves
    /// only that; the scanner never matches against these sets.
    const SINGLE_TERMINAL_SETS: &'static [TerminalSet];

    /// The id of the nonterminal with the given name.
    fn nonterminal_id(name: &str) -> Option<NonterminalId>;

    /// The id of the start wrapper for a nonterminal, which is the entry point
    /// for parsing from the nonterminal. For example, the id of the `StartA`
    /// wrapper for the nonterminal `A`. Derived nonterminals do not have start
    /// wrappers.
    fn start_nonterminal_id(name: &str) -> Option<NonterminalId> {
        Self::nonterminal_id(&format!("Start{name}"))
    }

    fn nonterminal_display_name(nonterminal_id: NonterminalId) -> &'static str {
        Self::NONTERMINALS[nonterminal_id.index()].display_name
    }

    fn terminal_name(terminal_id: TerminalId) -> &'static str {
        Self::TERMINALS[terminal_id.index()].name
    }

    /// The set holding just this terminal, which a failed match of it refers to.
    fn terminal_set(terminal_id: TerminalId) -> &'static TerminalSet {
        &Self::SINGLE_TERMINAL_SETS[terminal_id.index()]
    }

    /// Whether the terminal is a literal written in the grammar, such as
    /// `"else"`, rather than a named lexical definition. Literal terminal
    /// names retain their leading quote, so this test is exact.
    fn is_literal(terminal_id: TerminalId) -> bool {
        Self::terminal_name(terminal_id).starts_with('"')
    }

    fn slot_name(slot_id: SlotId) -> &'static str {
        Self::SLOTS[slot_id.index()].display_name
    }

    /// The id of the previous slot. To get the previous slot, we need to move
    /// the dot one symbol back in an alternative. For example, for
    /// `S = A . B C`, the previous slot is `S = . A B C`. Returns `None` when
    /// the dot is at position 0. The generator numbers the slots of an
    /// alternative consecutively in position order, so the earlier slot is the
    /// one with the next lower id.
    fn previous_slot(slot_id: SlotId) -> Option<SlotId> {
        (Self::SLOTS[slot_id.index()].position > 0).then(|| SlotId(slot_id.0 - 1))
    }

    /// The synthetic epsilon terminal.
    fn epsilon() -> TerminalId {
        TerminalId((Self::TERMINALS.len() - 2) as u16)
    }

    /// The number of terminals, excluding the synthetic epsilon and
    /// end-of-file terminals.
    fn terminal_count() -> u16 {
        (Self::TERMINALS.len() - 2) as u16
    }
}
