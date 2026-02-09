use crate::grammar::{
    def::{Alternative, Grammar},
    symbols::{self, Nonterminal},
};

/// Represents a grammar slot of the form `A : a B . c`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slot<'a> {
    pub head: &'a Nonterminal,
    pub alternative: &'a Alternative,
    pub pos: usize,
}

impl<'a> Slot<'a> {
    pub fn new(head: &'a Nonterminal, alternative: &'a Alternative, pos: usize) -> Self {
        Self {
            head,
            alternative,
            pos,
        }
    }

    /// Returns the next grammar slot by moving the dot to the next position.
    pub fn next(&self) -> Self {
        Slot::new(self.head, self.alternative, self.pos + 1)
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

    pub fn name(&self, _grammar: &'a Grammar) -> String {
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
