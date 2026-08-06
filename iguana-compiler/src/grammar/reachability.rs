use rustc_hash::{FxHashMap, FxHashSet};

use crate::grammar::{
    def::Grammar,
    symbols::{Definition, Nonterminal, Symbol},
};

pub struct ReachabilityGraph<'a> {
    /// For each nonterminal, the set of all nonterminals transitively reachable from it.
    reachable: FxHashMap<&'a Nonterminal, FxHashSet<&'a Nonterminal>>,
}

impl<'a> ReachabilityGraph<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        let mut reachable = FxHashMap::default();
        for nt in grammar.nonterminals() {
            let mut visited = FxHashSet::default();
            Self::dfs(grammar, nt, &mut visited);
            reachable.insert(nt, visited);
        }
        ReachabilityGraph { reachable }
    }

    fn dfs(grammar: &'a Grammar, nt: &'a Nonterminal, visited: &mut FxHashSet<&'a Nonterminal>) {
        for alt in grammar.alternatives(nt) {
            for symbol in &alt.symbols {
                Self::visit_symbol(grammar, symbol, visited);
            }
        }
    }

    fn visit_symbol(
        grammar: &'a Grammar,
        symbol: &Symbol,
        visited: &mut FxHashSet<&'a Nonterminal>,
    ) {
        match symbol {
            Symbol::Identifier(id) => {
                let def_id = id.resolve();
                if let Definition::Nonterminal(nt) = grammar.definition(def_id) {
                    if visited.insert(nt) {
                        Self::dfs(grammar, nt, visited);
                    }
                }
            }
            Symbol::Labeled { symbol, .. } | Symbol::Binding { symbol, .. } => {
                Self::visit_symbol(grammar, symbol, visited);
            }
            Symbol::Restricted { symbol, .. } | Symbol::Exclude { symbol, .. } => {
                Self::visit_symbol(grammar, symbol, visited);
            }
            Symbol::Group(symbols) | Symbol::Alt(symbols) => {
                for s in symbols {
                    Self::visit_symbol(grammar, s, visited);
                }
            }
            Symbol::Opt(symbol) | Symbol::Star(symbol, _) | Symbol::Plus(symbol, _) => {
                Self::visit_symbol(grammar, symbol, visited);
            }
            Symbol::Call { name, .. } => {
                Self::visit_symbol(grammar, &Symbol::Identifier(name.clone()), visited);
            }
            Symbol::Literal(_) | Symbol::Condition(_) | Symbol::Return(_) => {}
        }
    }

    pub fn reachable(&self, nt: &Nonterminal) -> &FxHashSet<&'a Nonterminal> {
        &self.reachable[nt]
    }
}
