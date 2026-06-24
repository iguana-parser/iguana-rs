use rustc_hash::{FxHashMap, FxHashSet};

use crate::grammar::{
    def::{Alternative, Grammar},
    reachability::ReachabilityGraph,
    symbols::{Definition, Nonterminal, Symbol, Terminal},
};

pub struct FirstFollowSets<'a> {
    grammar: &'a Grammar,
    reachability: ReachabilityGraph<'a>,
    nullables: FxHashSet<&'a Nonterminal>,
    first_sets: FxHashMap<&'a Nonterminal, FxHashSet<Terminal>>,
    follow_sets: FxHashMap<&'a Nonterminal, FxHashSet<Terminal>>,
}

impl<'a> FirstFollowSets<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        let mut ff = FirstFollowSets {
            grammar,
            reachability: ReachabilityGraph::new(grammar),
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

    pub fn follow_set(&self, nt: &Nonterminal) -> impl Iterator<Item = &Terminal> {
        self.follow_sets[nt].iter()
    }

    /// Returns the prediction set (director set) for an alternative of a nonterminal.
    ///
    /// The prediction set is:
    /// - FIRST(α) if α is not nullable
    /// - FIRST(α) ∪ FOLLOW(A) if α is nullable
    pub fn prediction_set(&self, nt: &Nonterminal, alt: &Alternative) -> FxHashSet<Terminal> {
        let mut set = self.first_set(alt);
        if alt.symbols.iter().all(|s| self.is_nullable(s)) {
            set.extend(self.follow_sets[nt].iter().cloned());
        }
        set
    }

    /// Returns true if the prediction sets of all alternatives of a nonterminal
    /// are pairwise disjoint. This is necessary but not sufficient for LL(1).
    pub fn has_disjoint_alternatives(&self, nt: &Nonterminal) -> bool {
        let alternatives = self.grammar.alternatives(nt);
        let prediction_sets: Vec<_> = alternatives
            .iter()
            .map(|alt| self.prediction_set(nt, alt))
            .collect();
        for i in 0..prediction_sets.len() {
            for j in i + 1..prediction_sets.len() {
                if !prediction_sets[i].is_disjoint(&prediction_sets[j]) {
                    return false;
                }
            }
        }
        true
    }

    /// A nonterminal is LL(1) if it and all transitively reachable
    /// nonterminals pass `is_nonterminal_ll1`.
    pub fn is_ll1(&self, nt: &Nonterminal) -> bool {
        self.is_nonterminal_ll1(nt)
            && self
                .reachability
                .reachable(nt)
                .iter()
                .all(|referenced| self.is_nonterminal_ll1(referenced))
    }

    /// A nonterminal is LL(1) if its alternatives have disjoint prediction
    /// sets.
    ///
    /// A parameterized nonterminal is never LL(1). It threads data-dependent
    /// arguments and guards its alternatives with conditions that single-token
    /// lookahead cannot evaluate, so it always parses through the
    /// descriptor-based GLL path, whatever its prediction sets look like.
    ///
    /// Plus is a special case: EBNF desugaring produces left-recursive
    /// rules (e.g., `A+ desugars into APlus = APlus A | A`) whose alternatives
    /// always overlap. The left recursion is an artifact of the desugaring;
    /// what the parser actually decides each iteration is whether to keep
    /// looping or stop, by trying to match the *continuation*: the symbols in
    /// the recursive alternative that come after the leading self-reference.
    ///
    /// For `Plus = Plus A | A`, the continuation is `[A]` and
    /// `FIRST(continuation) = FIRST(A)`. `{A ","}+` desugars to
    /// `Plus = Plus "," A | A`, so the continuation is `["," A]` and
    /// `FIRST(continuation) = {","}` (the separator is not nullable, so the
    /// walk stops there).
    ///
    /// A continuation may also contain a *longest-match* symbol: a nullable
    /// nonterminal whose rule body is followed by a restriction that excludes
    /// its own FIRST set, giving it the same longest-match behavior a regex
    /// repetition would have natively. At any position past such a symbol, no
    /// token in its FIRST can remain as a possible lookahead. Layout is one
    /// instance, e.g.,
    /// `Layout = (WhiteSpace | Comment)* !>> WhiteSpace !>> Comment`.
    ///
    /// At each iteration, the loop decides whether to match the continuation
    /// again or return to the parent rule. Single-token lookahead can pick
    /// between them only when no character is valid for both:
    /// `FIRST(continuation)` and the non-recursive FOLLOW of Plus must be
    /// disjoint. The non-recursive FOLLOW excludes the contribution that the
    /// self-recursive rule would otherwise inject into `FOLLOW(Plus)`. When
    /// the sets overlap, Plus is not LL(1), and the parser uses GLL to
    /// explore both derivations.
    fn is_nonterminal_ll1(&self, nt: &Nonterminal) -> bool {
        if !nt.parameters.is_empty() {
            return false;
        }
        if self.has_disjoint_alternatives(nt) {
            return true;
        }
        let Some(Symbol::Plus(element, separator)) = &nt.origin else {
            return false;
        };
        // The LL(1) Plus loop only matches the symbol but ignores the restrictions.
        // If a Plus inner symbol has restrictions, the Plus is not classified as LL(1),
        // and will be parsed using GLL.
        if element.has_restriction() || separator.as_deref().is_some_and(Symbol::has_restriction) {
            return false;
        }
        let continuation = &self.grammar.alternatives(nt)[0].symbols[1..];
        // FIRST set of the continuation. Walk symbols left to right:
        // - contribute each symbol's FIRST set to the running set;
        // - stop after the first non-nullable symbol;
        // - skip a longest-match symbol's FIRST set, since no token in
        //   it can appear immediately after the symbol, so it is not
        //   part of the actual lookahead at the position past it.
        let mut first_continuation = FxHashSet::default();
        for s in continuation {
            if !self.is_longest_match(s) {
                self.collect_first_of_symbol(s, &mut first_continuation);
            }
            if !self.is_nullable(s) {
                break;
            }
        }
        // Non-recursive FOLLOW(nt): the terminals that can immediately
        // follow nt per some rule other than nt's own. Plus rules have
        // the shape `Plus_n = Plus_n continuation | base` and nt is
        // referenced only at position 0 of the recursive alternative, so
        // skipping nt's own rule entirely removes exactly the self-
        // recursive contribution. For each reference to nt elsewhere:
        // - take FIRST of the symbols after it (skipping longest-match
        //   symbols, which contribute nothing);
        // - if the suffix is entirely nullable, fall back to FOLLOW of
        //   the rule containing the reference;
        // - subtract any `!>>` on the reference itself (e.g.,
        //   `(Alpha|Digit)+ !>> Alpha !>> Digit`), which forbids those
        //   terminals from following nt at this site.
        let mut non_recursive_follow = FxHashSet::default();
        for rule in self.grammar.nonterminals() {
            if rule.name == nt.name {
                continue;
            }
            for alternative in self.grammar.alternatives(rule) {
                for (i, symbol) in alternative.symbols.iter().enumerate() {
                    let Some(nt_b) = self.symbol_nonterminal(symbol) else {
                        continue;
                    };
                    if nt_b.name != nt.name {
                        continue;
                    }
                    let mut local = FxHashSet::default();
                    let mut all_nullable = true;
                    for s in &alternative.symbols[i + 1..] {
                        if !self.is_longest_match(s) {
                            self.collect_first_of_symbol(s, &mut local);
                        }
                        if !self.is_nullable(s) {
                            all_nullable = false;
                            break;
                        }
                    }
                    if all_nullable {
                        local.extend(self.follow_sets[rule].iter().cloned());
                    }
                    self.remove_restricted_terminals(&mut local, symbol);
                    non_recursive_follow.extend(local);
                }
            }
        }
        non_recursive_follow.is_disjoint(&first_continuation)
    }

    /// A symbol is a *longest match* when it is a nullable nonterminal whose
    /// rule guarantees, structurally, that no token in its FIRST set can
    /// immediately follow it. The shape is a repetition body followed by a
    /// restriction excluding that body's FIRST, e.g.,
    /// `Layout = (WhiteSpace | Comment)* !>> WhiteSpace !>> Comment` where
    /// FIRST(Layout) = {WhiteSpace, Comment} and the restriction excludes
    /// exactly those. Detection is structural rather than via computed
    /// FOLLOW: the restriction registers against the inner symbol wrapped by
    /// the `!>>` (the EBNF body), not against the enclosing nonterminal, so
    /// the standard FOLLOW algorithm does not filter FOLLOW(nt) directly.
    fn is_longest_match(&self, symbol: &Symbol) -> bool {
        if !self.is_nullable(symbol) {
            return false;
        }
        let Some(nt) = self.symbol_nonterminal(symbol) else {
            return false;
        };
        let Some(nt_first) = self.first_sets.get(nt) else {
            return false;
        };
        let alternatives = self.grammar.alternatives(nt);
        if alternatives.is_empty() {
            return false;
        }
        alternatives.iter().all(|alt| {
            alt.symbols
                .last()
                .is_some_and(|last| self.excludes_following(last, nt_first))
        })
    }

    /// True if `symbol` has a follow restriction whose excluded terminals
    /// cover every terminal in `target`, i.e., none of those terminals can
    /// immediately follow `symbol` at this position. Transparent wrappers
    /// (`Labeled`, `Binding`, `Except`, `PrecedeRestriction`, `Exclude`) are
    /// unwrapped first, matching the set handled by
    /// `remove_restricted_terminals`.
    fn excludes_following(&self, symbol: &Symbol, target: &FxHashSet<Terminal>) -> bool {
        match symbol {
            Symbol::FollowRestriction { restrictions, .. } => {
                let mut excluded = FxHashSet::default();
                for r in restrictions {
                    if let Definition::Terminal(t) = self.grammar.definition(r.resolve()) {
                        excluded.insert(t.clone());
                    }
                }
                target.iter().all(|t| excluded.contains(t))
            }
            Symbol::Labeled { symbol, .. }
            | Symbol::Binding { symbol, .. }
            | Symbol::Except { symbol, .. }
            | Symbol::PrecedeRestriction { symbol, .. }
            | Symbol::Exclude { symbol, .. } => self.excludes_following(symbol, target),
            _ => false,
        }
    }

    /// Remove from `set` every terminal that appears in any
    /// `FollowRestriction` reached by unwrapping the symbol's transparent
    /// wrappers.
    fn remove_restricted_terminals(&self, set: &mut FxHashSet<Terminal>, symbol: &Symbol) {
        match symbol {
            Symbol::FollowRestriction {
                symbol: inner,
                restrictions,
            } => {
                for r in restrictions {
                    if let Definition::Terminal(t) = self.grammar.definition(r.resolve()) {
                        set.remove(t);
                    }
                }
                self.remove_restricted_terminals(set, inner);
            }
            Symbol::Labeled { symbol, .. }
            | Symbol::Binding { symbol, .. }
            | Symbol::Except { symbol, .. }
            | Symbol::PrecedeRestriction { symbol, .. }
            | Symbol::Exclude { symbol, .. } => {
                self.remove_restricted_terminals(set, symbol);
            }
            _ => {}
        }
    }

    /// Returns true if every symbol of an alternative is nullable.
    pub fn is_alt_nullable(&self, alt: &Alternative) -> bool {
        alt.symbols.iter().all(|s| self.is_nullable(s))
    }

    /// Returns true if the nonterminal has any nullable alternative.
    pub fn is_nonterminal_nullable(&self, nt: &Nonterminal) -> bool {
        self.nullables.contains(nt)
    }

    /// Returns the FIRST set of an alternative. Walks symbols left to right,
    /// collecting FIRST of each symbol, stopping at the first non-nullable.
    pub fn first_set(&self, alt: &Alternative) -> FxHashSet<Terminal> {
        let mut set = FxHashSet::default();
        for symbol in &alt.symbols {
            let firsts = self.first_of_symbol(symbol);
            set.extend(firsts);
            if !self.is_nullable(symbol) {
                break;
            }
        }
        set
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
                    if alternative.symbols.iter().all(|s| self.is_nullable(s)) {
                        self.nullables.insert(nonterminal);
                        changed = true;
                        break;
                    }
                }
            }
        }
    }

    fn is_nullable(&self, s: &Symbol) -> bool {
        match s {
            Symbol::Labeled { symbol, .. } | Symbol::Binding { symbol, .. } => {
                self.is_nullable(symbol)
            }
            Symbol::Identifier(id) => {
                let def_id = id.resolve();
                match self.grammar.definition(def_id) {
                    Definition::Terminal(terminal) => self
                        .grammar
                        .lexical_rule(terminal)
                        .is_some_and(|rule| rule.regex.is_nullable()),
                    Definition::Nonterminal(nt) => self.nullables.contains(nt),
                }
            }
            Symbol::Literal(_) => false,
            Symbol::Group(symbols) => symbols.iter().all(|s| self.is_nullable(s)),
            Symbol::Opt(_) | Symbol::Star(_, _) => true,
            Symbol::Alt(symbols) => symbols.iter().any(|s| self.is_nullable(s)),
            Symbol::Plus(symbol, _) => self.is_nullable(symbol),
            Symbol::Except { symbol, .. }
            | Symbol::FollowRestriction { symbol, .. }
            | Symbol::PrecedeRestriction { symbol, .. }
            | Symbol::Exclude { symbol, .. } => self.is_nullable(symbol),
            // Conditions and returns don't consume input, so they are nullable.
            Symbol::Condition(_) | Symbol::Return(_) => true,
            Symbol::Call { name, .. } => {
                let def_id = name.resolve();
                match self.grammar.definition(def_id) {
                    Definition::Terminal(_) => false,
                    Definition::Nonterminal(nt) => self.nullables.contains(nt),
                }
            }
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
                        if !self.is_nullable(symbol) {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn first_of_symbol(&self, symbol: &Symbol) -> FxHashSet<Terminal> {
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
                    if !self.is_nullable(s) {
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
            Symbol::Call { name, .. } => self.symbol_nonterminal(&Symbol::Identifier(name.clone())),
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

        // Every nonterminal can be a start symbol, so EOF is in all FOLLOW sets
        for nonterminal in self.grammar.nonterminals() {
            self.follow_sets
                .get_mut(nonterminal)
                .unwrap()
                .insert(Self::eof());
        }

        let restrictions = self.collect_follow_restrictions();

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

                        let follow_restrictions = restrictions.get(nt_b);

                        // Add FIRST(β) to FOLLOW(B) where β = symbols[i+1..]
                        let suffix = &symbols[i + 1..];
                        for s in suffix {
                            let firsts = self.first_of_symbol(s);
                            let follow_b = self.follow_sets.get_mut(nt_b).unwrap();
                            let old_len = follow_b.len();
                            for t in firsts {
                                if follow_restrictions.is_none_or(|r| !r.contains(&t)) {
                                    follow_b.insert(t);
                                }
                            }
                            changed |= follow_b.len() > old_len;
                            if !self.is_nullable(s) {
                                break;
                            }
                        }

                        // If the entire suffix is nullable, add FOLLOW(A) to FOLLOW(B)
                        if suffix.iter().all(|s| self.is_nullable(s)) {
                            let follow_a: Vec<_> =
                                self.follow_sets[nonterminal].iter().cloned().collect();
                            let follow_b = self.follow_sets.get_mut(nt_b).unwrap();
                            let old_len = follow_b.len();
                            for t in follow_a {
                                if follow_restrictions.is_none_or(|r| !r.contains(&t)) {
                                    follow_b.insert(t);
                                }
                            }
                            changed |= follow_b.len() > old_len;
                        }
                    }
                }
            }
        }
    }

    /// Collects follow restrictions from the grammar. For each `A !>> B`,
    /// maps A's nonterminal to the set of restriction terminals.
    fn collect_follow_restrictions(&self) -> FxHashMap<&'a Nonterminal, FxHashSet<Terminal>> {
        let mut result: FxHashMap<&'a Nonterminal, FxHashSet<Terminal>> = FxHashMap::default();
        for nonterminal in self.grammar.nonterminals() {
            for alternative in self.grammar.alternatives(nonterminal) {
                for symbol in &alternative.symbols {
                    if let Symbol::FollowRestriction {
                        symbol: inner,
                        restrictions,
                    } = symbol
                    {
                        if let Some(nt) = self.symbol_nonterminal(inner) {
                            for restriction in restrictions {
                                let def = self.grammar.definition(restriction.resolve());
                                if let Definition::Terminal(t) = def {
                                    result.entry(nt).or_default().insert(t.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::def::GrammarDef;
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
    fn expression_grammar() -> GrammarDef {
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
        )
    }

    #[test]
    fn test_expression_grammar() {
        let grammar: Grammar = expression_grammar().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("Ep").unwrap()));
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("Tp").unwrap()));
        assert!(!ff.is_nonterminal_nullable(grammar.nonterminal("E").unwrap()));
        assert!(!ff.is_nonterminal_nullable(grammar.nonterminal("T").unwrap()));
        assert!(!ff.is_nonterminal_nullable(grammar.nonterminal("F").unwrap()));

        // FIRST sets
        let lparen = Terminal::new("\"(\"");
        let rparen = Terminal::new("\")\"");
        let plus = Terminal::new("\"+\"");
        let star = Terminal::new("\"*\"");
        let id_terminal = Terminal::new("\"id\"");
        let layout = Terminal::new("Layout");
        let eof = FirstFollowSets::eof();

        let first_e = &ff.first_sets[grammar.nonterminal("E").unwrap()];
        assert!(first_e.contains(&lparen));
        assert!(first_e.contains(&id_terminal));
        assert!(!first_e.contains(&layout));
        assert!(!first_e.contains(&plus));
        assert!(!first_e.contains(&star));
        assert!(!first_e.contains(&rparen));

        let first_ep = &ff.first_sets[grammar.nonterminal("Ep").unwrap()];
        assert!(first_ep.contains(&plus));
        assert_eq!(first_ep.len(), 1);

        let first_f = &ff.first_sets[grammar.nonterminal("F").unwrap()];
        assert!(first_f.contains(&lparen));
        assert!(first_f.contains(&id_terminal));
        assert_eq!(first_f.len(), 2);

        let first_tp = &ff.first_sets[grammar.nonterminal("Tp").unwrap()];
        assert!(first_tp.contains(&star));
        assert_eq!(first_tp.len(), 1);

        // FOLLOW sets
        // FOLLOW(E) = { ")", EOF }
        let follow_e = &ff.follow_sets[grammar.nonterminal("E").unwrap()];
        assert!(follow_e.contains(&rparen));
        assert!(follow_e.contains(&eof));

        // FOLLOW(Ep) = FOLLOW(E) = { ")", EOF }
        let follow_ep = &ff.follow_sets[grammar.nonterminal("Ep").unwrap()];
        assert!(follow_ep.contains(&rparen));
        assert!(follow_ep.contains(&eof));

        // FOLLOW(T) = { "+", ")", EOF }
        let follow_t = &ff.follow_sets[grammar.nonterminal("T").unwrap()];
        assert!(follow_t.contains(&plus));
        assert!(follow_t.contains(&rparen));
        assert!(follow_t.contains(&eof));

        // FOLLOW(Tp) = FOLLOW(T) = { "+", ")", EOF }
        let follow_tp = &ff.follow_sets[grammar.nonterminal("Tp").unwrap()];
        assert!(follow_tp.contains(&plus));
        assert!(follow_tp.contains(&rparen));
        assert!(follow_tp.contains(&eof));

        // FOLLOW(F) = { "*", "+", ")", EOF }
        let follow_f = &ff.follow_sets[grammar.nonterminal("F").unwrap()];
        assert!(follow_f.contains(&star));
        assert!(follow_f.contains(&plus));
        assert!(follow_f.contains(&rparen));
        assert!(follow_f.contains(&eof));

        // LL(1): this grammar is LL(1)
        assert!(ff.is_ll1(grammar.nonterminal("E").unwrap()));
        assert!(ff.is_ll1(grammar.nonterminal("Ep").unwrap()));
        assert!(ff.is_ll1(grammar.nonterminal("T").unwrap()));
        assert!(ff.is_ll1(grammar.nonterminal("Tp").unwrap()));
        assert!(ff.is_ll1(grammar.nonterminal("F").unwrap()));
    }

    #[test]
    fn test_parameterized_nonterminal_is_not_ll1() {
        use crate::grammar::symbols::{ParamType, Parameter};

        let grammar: Grammar = expression_grammar().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);

        // F = "(" E ")" | "id" has disjoint prediction sets, so it is LL(1).
        let f = grammar.nonterminal("F").unwrap();
        assert!(ff.is_nonterminal_ll1(f));

        // The same alternatives behind a parameter are not LL(1): a parameter
        // carries data-dependent conditions the LL(1) path cannot evaluate, so
        // disjoint prediction sets no longer suffice.
        let mut parameterized = f.clone();
        parameterized.parameters.push(Parameter {
            name: "p".to_string(),
            ty: ParamType::I32,
        });
        assert!(!ff.is_nonterminal_ll1(&parameterized));
    }

    // ---------------------------------------------------------------
    // Grammar 2: Multiple nullable prefixes
    //
    //   S = A B C "d"
    //   A = "a" | ε
    //   B = "b" | ε
    //   C = "c" | ε
    // ---------------------------------------------------------------
    fn nullable_prefix_grammar() -> GrammarDef {
        grammar_def!("nullable",
            syntax: [
                syntax_rule!("S" => alternative!(id!("A"), id!("B"), id!("C"), lit!("d"))),
                syntax_rule!("A" => priority_level!(alternative!(lit!("a")), alternative!())),
                syntax_rule!("B" => priority_level!(alternative!(lit!("b")), alternative!())),
                syntax_rule!("C" => priority_level!(alternative!(lit!("c")), alternative!()))
            ]
        )
    }

    #[test]
    fn test_nullable_prefix_grammar() {
        let grammar: Grammar = nullable_prefix_grammar().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("A").unwrap()));
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("B").unwrap()));
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("C").unwrap()));
        assert!(!ff.is_nonterminal_nullable(grammar.nonterminal("S").unwrap()));

        // FIRST sets
        let ta = Terminal::new("\"a\"");
        let tb = Terminal::new("\"b\"");
        let tc = Terminal::new("\"c\"");
        let td = Terminal::new("\"d\"");

        let first_s = &ff.first_sets[grammar.nonterminal("S").unwrap()];
        assert!(first_s.contains(&ta));
        assert!(first_s.contains(&tb));
        assert!(first_s.contains(&tc));
        assert!(first_s.contains(&td));

        let first_a = &ff.first_sets[grammar.nonterminal("A").unwrap()];
        assert!(first_a.contains(&ta));
        assert_eq!(first_a.len(), 1);

        let first_b = &ff.first_sets[grammar.nonterminal("B").unwrap()];
        assert!(first_b.contains(&tb));
        assert_eq!(first_b.len(), 1);

        let first_c = &ff.first_sets[grammar.nonterminal("C").unwrap()];
        assert!(first_c.contains(&tc));
        assert_eq!(first_c.len(), 1);

        // FOLLOW sets
        // FOLLOW(A) = { "b", "c", "d" }
        let follow_a = &ff.follow_sets[grammar.nonterminal("A").unwrap()];
        assert!(follow_a.contains(&tb));
        assert!(follow_a.contains(&tc));
        assert!(follow_a.contains(&td));

        // FOLLOW(B) = { "c", "d" }
        let follow_b = &ff.follow_sets[grammar.nonterminal("B").unwrap()];
        assert!(follow_b.contains(&tc));
        assert!(follow_b.contains(&td));

        // FOLLOW(C) = { "d" }
        let follow_c = &ff.follow_sets[grammar.nonterminal("C").unwrap()];
        assert!(follow_c.contains(&td));
    }

    // ---------------------------------------------------------------
    // Grammar 3: Mutually recursive FIRST sets
    //
    //   S = A B "c"
    //   A = "a" | ε
    //   B = S "d" | ε
    // ---------------------------------------------------------------
    fn recursive_first_grammar() -> GrammarDef {
        grammar_def!("recursive",
            syntax: [
                syntax_rule!("S" => alternative!(id!("A"), id!("B"), lit!("c"))),
                syntax_rule!("A" => priority_level!(alternative!(lit!("a")), alternative!())),
                syntax_rule!("B" => priority_level!(
                    alternative!(id!("S"), lit!("d")),
                    alternative!()
                ))
            ]
        )
    }

    #[test]
    fn test_recursive_first_grammar() {
        let grammar: Grammar = recursive_first_grammar().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("A").unwrap()));
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("B").unwrap()));
        assert!(!ff.is_nonterminal_nullable(grammar.nonterminal("S").unwrap()));

        // FIRST sets
        let ta = Terminal::new("\"a\"");
        let tc = Terminal::new("\"c\"");
        let td = Terminal::new("\"d\"");
        let eof = FirstFollowSets::eof();

        let first_s = &ff.first_sets[grammar.nonterminal("S").unwrap()];
        assert!(first_s.contains(&ta));
        assert!(first_s.contains(&tc));
        assert!(!first_s.contains(&td));

        let first_a = &ff.first_sets[grammar.nonterminal("A").unwrap()];
        assert!(first_a.contains(&ta));
        assert_eq!(first_a.len(), 1);

        let first_b = &ff.first_sets[grammar.nonterminal("B").unwrap()];
        assert!(first_b.contains(&ta));
        assert!(first_b.contains(&tc));
        assert!(!first_b.contains(&td));

        // FOLLOW sets
        // FOLLOW(S) = { EOF, "d" }
        let follow_s = &ff.follow_sets[grammar.nonterminal("S").unwrap()];
        assert!(follow_s.contains(&eof));
        assert!(follow_s.contains(&td));

        // FOLLOW(B) = { "c" }
        let follow_b = &ff.follow_sets[grammar.nonterminal("B").unwrap()];
        assert!(follow_b.contains(&tc));
    }

    // ---------------------------------------------------------------
    // Grammar 4: FIRST/FOLLOW conflict with right-recursive nullable
    // (Appel, "Modern Compiler Implementation", Chapter 3)
    //
    //   S = A "a" | "b"
    //   A = "a" A | ε
    //
    // A is nullable and right-recursive. FIRST(A → "a" A) = {"a"}.
    // Prediction set of A → ε = FOLLOW(A) = {"a"}.
    // Conflict on "a" → not LL(1).
    // ---------------------------------------------------------------
    fn appel_conflict_grammar() -> GrammarDef {
        grammar_def!("appel",
            syntax: [
                syntax_rule!("S" => priority_level!(
                    alternative!(id!("A"), lit!("a")),
                    alternative!(lit!("b"))
                )),
                syntax_rule!("A" => priority_level!(
                    alternative!(lit!("a"), id!("A")),
                    alternative!()
                ))
            ]
        )
    }

    #[test]
    fn test_appel_conflict_grammar() {
        let grammar: Grammar = appel_conflict_grammar().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("A").unwrap()));
        assert!(!ff.is_nonterminal_nullable(grammar.nonterminal("S").unwrap()));

        // FIRST sets
        let ta = Terminal::new("\"a\"");
        let tb = Terminal::new("\"b\"");

        let first_s = &ff.first_sets[grammar.nonterminal("S").unwrap()];
        assert!(first_s.contains(&ta));
        assert!(first_s.contains(&tb));

        let first_a = &ff.first_sets[grammar.nonterminal("A").unwrap()];
        assert!(first_a.contains(&ta));

        // FOLLOW sets
        // FOLLOW(A) = { "a" }, from S -> A "a"
        let follow_a = &ff.follow_sets[grammar.nonterminal("A").unwrap()];
        assert!(follow_a.contains(&ta));

        // S has disjoint alternatives ({"a"} vs {"b"}) but is NOT LL(1)
        // because it references A which is not LL(1)
        assert!(ff.has_disjoint_alternatives(grammar.nonterminal("S").unwrap()));
        assert!(!ff.is_ll1(grammar.nonterminal("S").unwrap()));
        // A is NOT LL(1): prediction(A -> "a" A) = {"a"}, prediction(A -> e) = FOLLOW(A) = {"a"}
        assert!(!ff.is_ll1(grammar.nonterminal("A").unwrap()));
    }

    // ---------------------------------------------------------------
    // Grammar 5: Simple FIRST/FOLLOW conflict with nullable
    // (Grune & Jacobs, "Parsing Techniques")
    //
    //   S = A "b"
    //   A = "b" | ε
    //
    // FIRST(A → "b") = {"b"}.
    // Prediction set of A → ε = FOLLOW(A) = {"b"}.
    // Conflict on "b" → not LL(1).
    // ---------------------------------------------------------------
    fn grune_conflict_grammar() -> GrammarDef {
        grammar_def!("grune",
            syntax: [
                syntax_rule!("S" => alternative!(id!("A"), lit!("b"))),
                syntax_rule!("A" => priority_level!(
                    alternative!(lit!("b")),
                    alternative!()
                ))
            ]
        )
    }

    #[test]
    fn test_grune_conflict_grammar() {
        let grammar: Grammar = grune_conflict_grammar().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);

        // Nullables
        assert!(ff.is_nonterminal_nullable(grammar.nonterminal("A").unwrap()));
        assert!(!ff.is_nonterminal_nullable(grammar.nonterminal("S").unwrap()));

        // FIRST sets
        let tb = Terminal::new("\"b\"");

        let first_s = &ff.first_sets[grammar.nonterminal("S").unwrap()];
        assert!(first_s.contains(&tb));

        let first_a = &ff.first_sets[grammar.nonterminal("A").unwrap()];
        assert!(first_a.contains(&tb));

        // FOLLOW sets
        // FOLLOW(A) = { Layout, "b" }, from S -> A Layout "b"
        let follow_a = &ff.follow_sets[grammar.nonterminal("A").unwrap()];
        assert!(follow_a.contains(&tb));

        // S has one alternative so disjoint trivially, but NOT LL(1)
        // because it references A which is not LL(1)
        assert!(ff.has_disjoint_alternatives(grammar.nonterminal("S").unwrap()));
        assert!(!ff.is_ll1(grammar.nonterminal("S").unwrap()));
        // A is NOT LL(1): prediction(A -> "b") = {"b"}, prediction(A -> e) = FOLLOW(A) contains {"b"}
        assert!(!ff.is_ll1(grammar.nonterminal("A").unwrap()));
    }
}
