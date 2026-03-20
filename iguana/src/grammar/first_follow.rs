use rustc_hash::{FxHashMap, FxHashSet};

use crate::grammar::{
    def::Grammar,
    symbols::{Definition, Nonterminal, Symbol, Terminal},
};

pub struct FirstFollowSets<'a> {
    grammar: &'a Grammar,
    nullables: FxHashSet<&'a Nonterminal>,
    first_sets: FxHashMap<&'a Nonterminal, FxHashSet<Terminal>>,
    follow_sets: FxHashMap<&'a Nonterminal, FxHashSet<Terminal>>,
}

impl<'a> FirstFollowSets<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        let mut ff = FirstFollowSets {
            grammar,
            nullables: FxHashSet::default(),
            first_sets: FxHashMap::default(),
            follow_sets: FxHashMap::default(),
        };
        ff.calc_nullables();
        ff.calc_first_sets();
        ff.calc_follow_sets();
        ff
    }

    pub fn eof() -> Terminal {
        Terminal::new("EOF")
    }

    pub fn is_nullable(&self, nt: &Nonterminal) -> bool {
        self.nullables.contains(nt)
    }

    pub fn first_set(&self, nt: &Nonterminal) -> impl Iterator<Item = &Terminal> {
        self.first_sets[nt].iter()
    }

    pub fn follow_set(&self, nt: &Nonterminal) -> impl Iterator<Item = &Terminal> {
        self.follow_sets[nt].iter()
    }

    // -- Nullables --

    fn calc_nullables(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            for nonterminal in self.grammar.nonterminals() {
                if self.nullables.contains(nonterminal) {
                    continue;
                }
                for alternative in self.grammar.alternatives(nonterminal) {
                    if alternative
                        .symbols
                        .iter()
                        .all(|s| self.is_symbol_nullable(s))
                    {
                        self.nullables.insert(nonterminal);
                        changed = true;
                        break;
                    }
                }
            }
        }
    }

    fn is_symbol_nullable(&self, s: &Symbol) -> bool {
        match s {
            Symbol::Labeled { symbol, .. } | Symbol::Binding { symbol, .. } => {
                self.is_symbol_nullable(symbol)
            }
            Symbol::Identifier(id) => {
                let def_id = id.resolve();
                match self.grammar.definition(def_id) {
                    Definition::Terminal(terminal) => self
                        .grammar
                        .lexical_rule(terminal)
                        .map_or(false, |rule| rule.regex.is_nullable()),
                    Definition::Nonterminal(nt) => self.nullables.contains(nt),
                }
            }
            Symbol::Literal(_) => false,
            Symbol::Group(symbols) => symbols.iter().all(|s| self.is_symbol_nullable(s)),
            Symbol::Opt(_) | Symbol::Star(_, _) => true,
            Symbol::Alt(symbols) => symbols.iter().any(|s| self.is_symbol_nullable(s)),
            Symbol::Plus(symbol, _) => self.is_symbol_nullable(symbol),
            Symbol::Except { symbol, .. }
            | Symbol::FollowRestriction { symbol, .. }
            | Symbol::PrecedeRestriction { symbol, .. }
            | Symbol::Exclude { symbol, .. } => self.is_symbol_nullable(symbol),
            Symbol::Call { .. } | Symbol::Condition(_) | Symbol::Return(_) => false,
        }
    }

    // -- FIRST sets --

    fn calc_first_sets(&mut self) {
        for nonterminal in self.grammar.nonterminals() {
            self.first_sets.insert(nonterminal, FxHashSet::default());
        }

        let mut changed = true;
        while changed {
            changed = false;
            for nonterminal in self.grammar.nonterminals() {
                for alternative in self.grammar.alternatives(nonterminal) {
                    for symbol in &alternative.symbols {
                        let firsts = self.first_of_symbol(symbol);
                        let target_set = self.first_sets.get_mut(nonterminal).unwrap();
                        let old_len = target_set.len();
                        target_set.extend(firsts);
                        changed |= target_set.len() > old_len;
                        if !self.is_symbol_nullable(symbol) {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn first_of_symbol(&self, symbol: &Symbol) -> FxHashSet<Terminal> {
        let mut result = FxHashSet::default();
        self.collect_first_of_symbol(symbol, &mut result);
        result
    }

    fn collect_first_of_symbol(&self, symbol: &Symbol, result: &mut FxHashSet<Terminal>) {
        match symbol {
            Symbol::Identifier(id) => {
                let def_id = id.resolve();
                match self.grammar.definition(def_id) {
                    Definition::Terminal(terminal) => {
                        result.insert(terminal.clone());
                    }
                    Definition::Nonterminal(nt) => {
                        if let Some(set) = self.first_sets.get(nt) {
                            result.extend(set.iter().cloned());
                        }
                    }
                }
            }
            Symbol::Literal(lit) => {
                if let Some(terminal) = self.grammar.terminals().find(|t| t.name == *lit) {
                    result.insert(terminal.clone());
                }
            }
            Symbol::Labeled { symbol, .. } | Symbol::Binding { symbol, .. } => {
                self.collect_first_of_symbol(symbol, result);
            }
            Symbol::Except { symbol, .. }
            | Symbol::FollowRestriction { symbol, .. }
            | Symbol::PrecedeRestriction { symbol, .. }
            | Symbol::Exclude { symbol, .. } => {
                self.collect_first_of_symbol(symbol, result);
            }
            Symbol::Group(symbols) => {
                for s in symbols {
                    self.collect_first_of_symbol(s, result);
                    if !self.is_symbol_nullable(s) {
                        break;
                    }
                }
            }
            Symbol::Alt(symbols) => {
                for s in symbols {
                    self.collect_first_of_symbol(s, result);
                }
            }
            Symbol::Opt(symbol) | Symbol::Star(symbol, _) | Symbol::Plus(symbol, _) => {
                self.collect_first_of_symbol(symbol, result);
            }
            Symbol::Call { name, .. } => {
                self.collect_first_of_symbol(&Symbol::Identifier(name.clone()), result);
            }
            Symbol::Condition(_) | Symbol::Return(_) => {}
        }
    }

    // -- FOLLOW sets --

    /// Extracts the nonterminal that a symbol refers to, if any.
    fn symbol_nonterminal(&self, symbol: &Symbol) -> Option<&'a Nonterminal> {
        match symbol {
            Symbol::Identifier(id) => {
                let def_id = id.resolve();
                match self.grammar.definition(def_id) {
                    Definition::Nonterminal(nt) => Some(nt),
                    Definition::Terminal(_) => None,
                }
            }
            Symbol::Labeled { symbol, .. } | Symbol::Binding { symbol, .. } => {
                self.symbol_nonterminal(symbol)
            }
            Symbol::Except { symbol, .. }
            | Symbol::FollowRestriction { symbol, .. }
            | Symbol::PrecedeRestriction { symbol, .. }
            | Symbol::Exclude { symbol, .. } => self.symbol_nonterminal(symbol),
            Symbol::Call { name, .. } => {
                self.symbol_nonterminal(&Symbol::Identifier(name.clone()))
            }
            _ => None,
        }
    }

    /// FOLLOW(A) is the set of terminals that can appear immediately after A
    /// in some sentential form. For start nonterminals, FOLLOW includes EOF.
    ///
    /// For each production A → α B β:
    /// - FIRST(β) \ {ε} is added to FOLLOW(B)
    /// - If β is nullable, FOLLOW(A) is added to FOLLOW(B)
    fn calc_follow_sets(&mut self) {
        for nonterminal in self.grammar.nonterminals() {
            self.follow_sets.insert(nonterminal, FxHashSet::default());
        }

        // FOLLOW(start) includes EOF
        for nonterminal in self.grammar.nonterminals() {
            if nonterminal.name.starts_with("Start") {
                self.follow_sets
                    .get_mut(nonterminal)
                    .unwrap()
                    .insert(Self::eof());
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for nonterminal in self.grammar.nonterminals() {
                for alternative in self.grammar.alternatives(nonterminal) {
                    let symbols = &alternative.symbols;
                    for (i, symbol) in symbols.iter().enumerate() {
                        let Some(nt_b) = self.symbol_nonterminal(symbol) else {
                            continue;
                        };

                        // Add FIRST(β) to FOLLOW(B) where β = symbols[i+1..]
                        let suffix = &symbols[i + 1..];
                        for s in suffix {
                            let firsts = self.first_of_symbol(s);
                            let follow_b = self.follow_sets.get_mut(nt_b).unwrap();
                            let old_len = follow_b.len();
                            follow_b.extend(firsts);
                            changed |= follow_b.len() > old_len;
                            if !self.is_symbol_nullable(s) {
                                break;
                            }
                        }

                        // If the entire suffix is nullable, add FOLLOW(A) to FOLLOW(B)
                        if suffix.iter().all(|s| self.is_symbol_nullable(s)) {
                            let follow_a: Vec<_> =
                                self.follow_sets[nonterminal].iter().cloned().collect();
                            let follow_b = self.follow_sets.get_mut(nt_b).unwrap();
                            let old_len = follow_b.len();
                            follow_b.extend(follow_a);
                            changed |= follow_b.len() > old_len;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{alternative, grammar_def, id, lit, priority_level, syntax_rule};

    // ---------------------------------------------------------------
    // Grammar 1: Classic expression grammar (Dragon Book, Example 4.17)
    //
    //   E  = T Ep
    //   Ep = "+" T Ep | ε
    //   T  = F Tp
    //   Tp = "*" F Tp | ε
    //   F  = "(" E ")" | "id"
    // ---------------------------------------------------------------

    fn expression_grammar() -> Grammar {
        grammar_def!("expr",
            syntax: [
                syntax_rule!("E" => alternative!(id!("T"), id!("Ep"))),
                syntax_rule!("Ep" => priority_level!(
                    alternative!(lit!("+"), id!("T"), id!("Ep")),
                    alternative!()
                )),
                syntax_rule!("T" => alternative!(id!("F"), id!("Tp"))),
                syntax_rule!("Tp" => priority_level!(
                    alternative!(lit!("*"), id!("F"), id!("Tp")),
                    alternative!()
                )),
                syntax_rule!("F" => priority_level!(
                    alternative!(lit!("("), id!("E"), lit!(")")),
                    alternative!(lit!("id"))
                ))
            ]
        ).into()
    }

    #[test]
    fn test_expression_grammar() {
        let grammar = expression_grammar();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nullable(grammar.nonterminal("Ep").unwrap()));
        assert!(ff.is_nullable(grammar.nonterminal("Tp").unwrap()));
        assert!(!ff.is_nullable(grammar.nonterminal("E").unwrap()));
        assert!(!ff.is_nullable(grammar.nonterminal("T").unwrap()));
        assert!(!ff.is_nullable(grammar.nonterminal("F").unwrap()));

        // FIRST sets
        let lparen = Terminal::new("\"(\"");
        let rparen = Terminal::new("\")\"");
        let plus = Terminal::new("\"+\"");
        let star = Terminal::new("\"*\"");
        let id_terminal = Terminal::new("\"id\"");
        let layout = Terminal::new("Layout");
        let eof = FirstFollowSets::eof();

        let first_e: FxHashSet<_> = ff.first_set(grammar.nonterminal("E").unwrap()).cloned().collect();
        assert!(first_e.contains(&lparen));
        assert!(first_e.contains(&id_terminal));
        assert!(!first_e.contains(&layout));
        assert!(!first_e.contains(&plus));
        assert!(!first_e.contains(&star));
        assert!(!first_e.contains(&rparen));

        let first_ep: FxHashSet<_> = ff.first_set(grammar.nonterminal("Ep").unwrap()).cloned().collect();
        assert!(first_ep.contains(&plus));
        assert_eq!(first_ep.len(), 1);

        let first_f: FxHashSet<_> = ff.first_set(grammar.nonterminal("F").unwrap()).cloned().collect();
        assert!(first_f.contains(&lparen));
        assert!(first_f.contains(&id_terminal));
        assert_eq!(first_f.len(), 2);

        let first_tp: FxHashSet<_> = ff.first_set(grammar.nonterminal("Tp").unwrap()).cloned().collect();
        assert!(first_tp.contains(&star));
        assert_eq!(first_tp.len(), 1);

        // FOLLOW sets
        // FOLLOW(E) = { ")", EOF }
        let follow_e: FxHashSet<_> = ff.follow_set(grammar.nonterminal("E").unwrap()).cloned().collect();
        assert!(follow_e.contains(&rparen));
        assert!(follow_e.contains(&eof));

        // FOLLOW(Ep) = FOLLOW(E) = { ")", EOF }
        let follow_ep: FxHashSet<_> = ff.follow_set(grammar.nonterminal("Ep").unwrap()).cloned().collect();
        assert!(follow_ep.contains(&rparen));
        assert!(follow_ep.contains(&eof));

        // FOLLOW(T) = { "+", ")", EOF }
        let follow_t: FxHashSet<_> = ff.follow_set(grammar.nonterminal("T").unwrap()).cloned().collect();
        assert!(follow_t.contains(&plus));
        assert!(follow_t.contains(&rparen));
        assert!(follow_t.contains(&eof));

        // FOLLOW(Tp) = FOLLOW(T) = { "+", ")", EOF }
        let follow_tp: FxHashSet<_> = ff.follow_set(grammar.nonterminal("Tp").unwrap()).cloned().collect();
        assert!(follow_tp.contains(&plus));
        assert!(follow_tp.contains(&rparen));
        assert!(follow_tp.contains(&eof));

        // FOLLOW(F) = { "*", "+", ")", EOF }
        let follow_f: FxHashSet<_> = ff.follow_set(grammar.nonterminal("F").unwrap()).cloned().collect();
        assert!(follow_f.contains(&star));
        assert!(follow_f.contains(&plus));
        assert!(follow_f.contains(&rparen));
        assert!(follow_f.contains(&eof));
    }

    // ---------------------------------------------------------------
    // Grammar 2: Multiple nullable prefixes
    //
    //   S = A B C "d"
    //   A = "a" | ε
    //   B = "b" | ε
    //   C = "c" | ε
    // ---------------------------------------------------------------

    fn nullable_prefix_grammar() -> Grammar {
        grammar_def!("nullable",
            syntax: [
                syntax_rule!("S" => alternative!(id!("A"), id!("B"), id!("C"), lit!("d"))),
                syntax_rule!("A" => priority_level!(alternative!(lit!("a")), alternative!())),
                syntax_rule!("B" => priority_level!(alternative!(lit!("b")), alternative!())),
                syntax_rule!("C" => priority_level!(alternative!(lit!("c")), alternative!()))
            ]
        ).into()
    }

    #[test]
    fn test_nullable_prefix_grammar() {
        let grammar = nullable_prefix_grammar();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nullable(grammar.nonterminal("A").unwrap()));
        assert!(ff.is_nullable(grammar.nonterminal("B").unwrap()));
        assert!(ff.is_nullable(grammar.nonterminal("C").unwrap()));
        assert!(!ff.is_nullable(grammar.nonterminal("S").unwrap()));

        // FIRST sets
        let ta = Terminal::new("\"a\"");
        let tb = Terminal::new("\"b\"");
        let tc = Terminal::new("\"c\"");
        let td = Terminal::new("\"d\"");
        let layout = Terminal::new("Layout");

        let first_s: FxHashSet<_> = ff.first_set(grammar.nonterminal("S").unwrap()).cloned().collect();
        assert!(first_s.contains(&ta));
        assert!(first_s.contains(&tb));
        assert!(first_s.contains(&tc));
        assert!(first_s.contains(&td));
        assert!(first_s.contains(&layout));

        let first_a: FxHashSet<_> = ff.first_set(grammar.nonterminal("A").unwrap()).cloned().collect();
        assert!(first_a.contains(&ta));
        assert_eq!(first_a.len(), 1);

        let first_b: FxHashSet<_> = ff.first_set(grammar.nonterminal("B").unwrap()).cloned().collect();
        assert!(first_b.contains(&tb));
        assert_eq!(first_b.len(), 1);

        let first_c: FxHashSet<_> = ff.first_set(grammar.nonterminal("C").unwrap()).cloned().collect();
        assert!(first_c.contains(&tc));
        assert_eq!(first_c.len(), 1);

        // FOLLOW sets
        // FOLLOW(A) = { Layout, "b", "c", "d" }
        let follow_a: FxHashSet<_> = ff.follow_set(grammar.nonterminal("A").unwrap()).cloned().collect();
        assert!(follow_a.contains(&layout));
        assert!(follow_a.contains(&tb));
        assert!(follow_a.contains(&tc));
        assert!(follow_a.contains(&td));

        // FOLLOW(B) = { Layout, "c", "d" }
        let follow_b: FxHashSet<_> = ff.follow_set(grammar.nonterminal("B").unwrap()).cloned().collect();
        assert!(follow_b.contains(&layout));
        assert!(follow_b.contains(&tc));
        assert!(follow_b.contains(&td));

        // FOLLOW(C) = { Layout, "d" }
        let follow_c: FxHashSet<_> = ff.follow_set(grammar.nonterminal("C").unwrap()).cloned().collect();
        assert!(follow_c.contains(&layout));
        assert!(follow_c.contains(&td));
    }

    // ---------------------------------------------------------------
    // Grammar 3: Mutually recursive FIRST sets
    //
    //   S = A B "c"
    //   A = "a" | ε
    //   B = S "d" | ε
    // ---------------------------------------------------------------

    fn recursive_first_grammar() -> Grammar {
        grammar_def!("recursive",
            syntax: [
                syntax_rule!("S" => alternative!(id!("A"), id!("B"), lit!("c"))),
                syntax_rule!("A" => priority_level!(alternative!(lit!("a")), alternative!())),
                syntax_rule!("B" => priority_level!(
                    alternative!(id!("S"), lit!("d")),
                    alternative!()
                ))
            ]
        ).into()
    }

    #[test]
    fn test_recursive_first_grammar() {
        let grammar = recursive_first_grammar();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nullable(grammar.nonterminal("A").unwrap()));
        assert!(ff.is_nullable(grammar.nonterminal("B").unwrap()));
        assert!(!ff.is_nullable(grammar.nonterminal("S").unwrap()));

        // FIRST sets
        let ta = Terminal::new("\"a\"");
        let tc = Terminal::new("\"c\"");
        let td = Terminal::new("\"d\"");
        let layout = Terminal::new("Layout");
        let eof = FirstFollowSets::eof();

        let first_s: FxHashSet<_> = ff.first_set(grammar.nonterminal("S").unwrap()).cloned().collect();
        assert!(first_s.contains(&ta));
        assert!(first_s.contains(&tc));
        assert!(first_s.contains(&layout));
        assert!(!first_s.contains(&td));

        let first_a: FxHashSet<_> = ff.first_set(grammar.nonterminal("A").unwrap()).cloned().collect();
        assert!(first_a.contains(&ta));
        assert_eq!(first_a.len(), 1);

        let first_b: FxHashSet<_> = ff.first_set(grammar.nonterminal("B").unwrap()).cloned().collect();
        assert!(first_b.contains(&ta));
        assert!(first_b.contains(&tc));
        assert!(first_b.contains(&layout));
        assert!(!first_b.contains(&td));

        // FOLLOW sets
        // FOLLOW(S) = { EOF, Layout, "d" }
        let follow_s: FxHashSet<_> = ff.follow_set(grammar.nonterminal("S").unwrap()).cloned().collect();
        assert!(follow_s.contains(&eof));
        assert!(follow_s.contains(&layout));
        assert!(follow_s.contains(&td));

        // FOLLOW(B) = { Layout, "c" }
        let follow_b: FxHashSet<_> = ff.follow_set(grammar.nonterminal("B").unwrap()).cloned().collect();
        assert!(follow_b.contains(&layout));
        assert!(follow_b.contains(&tc));
    }
}
