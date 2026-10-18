use crate::grammar::{
    def::{Alternative, Grammar},
    symbols::{self, Definition, Nonterminal},
};

/// Represents a grammar slot of the form `A : a B . c`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slot<'a> {
    head: &'a Nonterminal,
    alternative: &'a Alternative,
    pos: usize,
}

impl<'a> Slot<'a> {
    pub fn new(head: &'a Nonterminal, alternative: &'a Alternative, pos: usize) -> Self {
        Self {
            head,
            alternative,
            pos,
        }
    }

    pub fn head(&self) -> &'a Nonterminal {
        self.head
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn alternative(&self) -> &'a Alternative {
        self.alternative
    }

    /// The index of the slot's alternative among the head's alternatives.
    pub fn alternative_index(&self, grammar: &Grammar) -> usize {
        grammar
            .alternatives(self.head)
            .iter()
            .position(|alternative| std::ptr::eq(alternative, self.alternative))
            .expect("the slot's alternative belongs to its head")
    }

    /// Returns the next grammar slot by moving the dot to the next position.
    pub fn next(&self) -> Self {
        Slot::new(self.head, self.alternative, self.pos + 1)
    }

    /// Returns true if the slot is the first symbol in the alternative.
    /// If there are only condition symbols before the slot position, it returns true.
    pub fn is_first(&self) -> bool {
        self.alternative.symbols[..self.pos]
            .iter()
            .all(|s| !s.is_parse_tree_symbol())
    }

    /// The nonterminal before the slot, or `None` when the slot is not a return
    /// slot.
    pub fn called_nonterminal(&self, grammar: &'a Grammar) -> Option<&'a Nonterminal> {
        let pos = self.pos.checked_sub(1)?;
        let called = self.alternative.symbols[pos].as_identifier()?;
        match grammar.definition(called.resolve()) {
            Definition::Nonterminal(called) => Some(called),
            Definition::Terminal(_) => None,
        }
    }

    /// Whether the slot is a return slot, the slot after a nonterminal.
    pub fn is_return_slot(&self, grammar: &Grammar) -> bool {
        self.called_nonterminal(grammar).is_some()
    }

    pub fn symbol(&self) -> Option<&symbols::Symbol> {
        Self::symbol_at(self.alternative, self.pos)
    }

    fn symbol_at(alternative: &Alternative, pos: usize) -> Option<&symbols::Symbol> {
        alternative.symbols.get(pos)
    }

    pub fn display_name(&self, grammar: &'a Grammar) -> String {
        self.name_to_string(|n| n.display_name(), |s| s.display_name(grammar))
    }

    pub fn name(&self) -> String {
        self.name_to_string(|n| n.to_string(), |s| s.to_string())
    }

    fn name_to_string(
        &self,
        nt_name: impl Fn(&Nonterminal) -> String,
        symbol_name: impl Fn(&symbols::Symbol) -> String,
    ) -> String {
        let mut result = String::new();
        result.push_str(&nt_name(self.head));
        result.push_str(" : ");
        for i in 0..self.alternative.symbols.len() {
            if i == self.pos {
                result.push_str(". ");
            }
            let symbol = Self::symbol_at(self.alternative, i).unwrap();
            result.push_str(&symbol_name(symbol));
            if i < self.alternative.len() - 1 {
                result.push(' ');
            }
        }
        if self.alternative.symbols.len() == self.pos {
            result.push('.');
        }
        result
    }
}
