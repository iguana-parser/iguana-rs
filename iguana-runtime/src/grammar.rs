use crate::ids::{NonterminalId, SlotId, TerminalId};

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
}

/// The runtime's view of `iguana_compiler::grammar::def::Grammar`. It holds
/// only the information the runtime needs. The generator implements the trait
/// once per grammar, and the generated CLI, the wasm wrapper, and the parser
/// reach the grammar's tables and lookups through it.
pub trait Grammar {
    /// Every nonterminal, indexed by `NonterminalId`.
    const NONTERMINALS: &'static [Nonterminal];
    /// The names of the nonterminals the grammar text declares, in source
    /// order.
    const DISPLAY_ORDER: &'static [&'static str];
    /// Every terminal, indexed by `TerminalId`. The last two entries are the
    /// synthetic epsilon and end-of-file terminals.
    const TERMINALS: &'static [Terminal];
    /// Every grammar slot, indexed by `SlotId`.
    const SLOTS: &'static [Slot];
    /// The name of the layout nonterminal, or `None` when the grammar declares
    /// no layout.
    const LAYOUT_NAME: Option<&'static str>;
    /// The terminals reachable from the layout definition, or an empty slice
    /// when the grammar declares no layout.
    const LAYOUT_TERMINALS: &'static [TerminalId];

    /// The id of the nonterminal with the given name.
    fn nonterminal_id(name: &str) -> Option<NonterminalId>;

    fn nonterminal_display_name(nonterminal_id: NonterminalId) -> &'static str {
        Self::NONTERMINALS[nonterminal_id.index()].display_name
    }

    fn terminal_name(terminal_id: TerminalId) -> &'static str {
        Self::TERMINALS[terminal_id.index()].name
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
