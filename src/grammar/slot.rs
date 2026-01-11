use crate::grammar::{
    def::{Alternative, Grammar},
    symbols::{Definition, DefinitionId, Nonterminal},
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

    pub fn symbol_def(&self) -> Option<DefinitionId> {
        Self::symbol_def_at_pos(self.alternative, self.pos)
    }

    fn symbol_def_at_pos(alternative: &Alternative, pos: usize) -> Option<DefinitionId> {
        let symbol = alternative.symbols.get(pos)?;
        Some(symbol.resolved_def())
    }

    pub fn display_name(&self, grammar: &'a Grammar) -> String {
        self.name_to_string(grammar, |n| n.display_name(), |d| d.display_name())
    }

    pub fn name(&self, grammar: &'a Grammar) -> String {
        self.name_to_string(grammar, |n| n.name.clone(), |d| d.name().into())
    }

    fn name_to_string(
        &self,
        grammar: &'a Grammar,
        nt_name: fn(&Nonterminal) -> String,
        def_name: fn(&Definition) -> String,
    ) -> String {
        let mut result = String::new();
        result.push_str(&nt_name(self.head));
        result.push_str(" : ");
        for i in 0..self.alternative.symbols.len() {
            if i == self.pos {
                result.push_str(". ");
            }
            let def_id = Self::symbol_def_at_pos(self.alternative, i).unwrap();
            let def = grammar.definition(def_id);
            result.push_str(&def_name(def));
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
