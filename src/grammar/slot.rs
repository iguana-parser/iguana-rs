use crate::grammar::{
    def::{Alternative, Grammar},
    symbols::{Definition, Nonterminal, Symbol},
};

/// Represents a grammar slot of the form `A : a B . c`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slot<'a> {
    pub head: &'a Nonterminal,
    pub alternative: &'a Alternative,
    pub pos: usize,
    pub symbol_def: Option<&'a Definition>,
}

impl<'a> Slot<'a> {
    pub fn new(
        head: &'a Nonterminal,
        alternative: &'a Alternative,
        pos: usize,
        grammar: &'a Grammar,
    ) -> Self {
        Self {
            head,
            alternative,
            pos,
            symbol_def: Self::symbol_def(alternative, pos, grammar),
        }
    }

    /// Returns the next grammar slot by moving the dot to the next position.
    pub fn next(&self, grammar: &'a Grammar) -> Self {
        Slot::new(self.head, self.alternative, self.pos + 1, grammar)
    }

    fn symbol_def(
        alternative: &'a Alternative,
        pos: usize,
        grammar: &'a Grammar,
    ) -> Option<&'a Definition> {
        let symbol = alternative.symbols.get(pos)?;
        let name = match symbol {
            Symbol::Identifier(name) => name,
            _ => panic!("At this point we expect only identifiers in the grammar"),
        };
        Some(
            grammar
                .definition(name)
                .unwrap_or_else(|| panic!("{name} is not defined")),
        )
    }

    pub fn display_name(&self, grammar: &'a Grammar) -> String {
        let mut result = String::new();
        result.push_str(&self.head.display_name());
        result.push_str(" : ");
        for i in 0..self.alternative.symbols.len() {
            if i == self.pos {
                result.push_str(". ");
            }
            let def = Self::symbol_def(self.alternative, i, grammar).unwrap();
            result.push_str(&def.display_name());
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
