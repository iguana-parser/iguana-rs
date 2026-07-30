use iguana_runtime::ids::TerminalId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::dfa::{Dfa, Nfa};
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
    /// A map from each nonterminal to the terminals in its follow set. Each
    /// terminal in the follow set is mapped to the follow restrictions (`!>>`)
    /// at the position where the nonterminal appears. For `S = A B !>> X C`,
    /// the follow set of `B` is `{C -> {X}}`: `C` appears after `B` only where
    /// `X` cannot appear. An empty restriction set means the terminal appears
    /// with no such condition. A terminal that follows the nonterminal at
    /// several positions keeps the restrictions shared by all of them.
    ///
    /// The restrictions are used by the LL(1) classification to rule out
    /// conflicts between overlapping terminals (`has_disjoint_alternatives`).
    follow_sets: FxHashMap<&'a Nonterminal, FxHashMap<Terminal, FxHashSet<Terminal>>>,
    ll1_nonterminals: FxHashSet<String>,
}

impl<'a> FirstFollowSets<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        let mut ff = FirstFollowSets {
            grammar,
            reachability: ReachabilityGraph::new(grammar),
            nullables: FxHashSet::default(),
            first_sets: FxHashMap::default(),
            follow_sets: FxHashMap::default(),
            ll1_nonterminals: FxHashSet::default(),
        };
        ff.calc_nullables();
        ff.calc_first_sets();
        ff.calc_follow_sets();
        ff.calc_ll1_nonterminals();
        ff
    }

    fn calc_ll1_nonterminals(&mut self) {
        let grammar = self.grammar;
        for nt in grammar.nonterminals() {
            if self.is_nonterminal_ll1(nt) {
                self.ll1_nonterminals.insert(nt.name.clone());
            }
        }
    }

    pub fn eof() -> Terminal {
        Terminal::new("EOF")
    }

    pub fn follow_set(&self, nt: &Nonterminal) -> impl Iterator<Item = &Terminal> {
        self.follow_sets[nt].keys()
    }

    /// Returns the prediction set for an alternative of a nonterminal: the
    /// terminals that can appear first wherever the alternative applies.
    /// These are FIRST(α) and, when α is nullable, the follow set of the
    /// nonterminal, since the alternative can derive nothing and the next
    /// terminal is then one that follows the nonterminal.
    ///
    /// The terminals taken from the follow set keep their restrictions as in
    /// `follow_sets`. A FIRST terminal does not have follow restrictions,
    /// because a `!>>` constrains what comes after a symbol, never how it
    /// starts.
    pub fn prediction_set(
        &self,
        nt: &Nonterminal,
        alt: &Alternative,
    ) -> FxHashMap<Terminal, FxHashSet<Terminal>> {
        let mut entries: FxHashMap<Terminal, FxHashSet<Terminal>> = self
            .first_set(alt)
            .into_iter()
            .map(|t| (t, FxHashSet::default()))
            .collect();
        if alt.symbols.iter().all(|s| self.is_nullable(s)) {
            for (t, restrictions) in &self.follow_sets[nt] {
                entries
                    .entry(t.clone())
                    .or_insert_with(|| restrictions.clone());
            }
        }
        entries
    }

    /// Returns whether one lookahead terminal distinguishes the alternatives
    /// of a nonterminal, i.e., no pair of prediction sets conflicts.
    ///
    /// A conflict means some input can start two alternatives at the same
    /// position, and both alternatives should be tried. The classical LL(1)
    /// condition, disjoint prediction sets, is enough in a two-phase parser:
    /// the scanner turns the input into a single token stream, so the next
    /// token is unique and two distinct terminals never compete for it.
    /// Iguana parses in a single phase and tries every interpretation of the
    /// input: at one position, two terminals can both match when a string of
    /// one is a prefix of a string of the other.
    ///
    /// The LL(1) classification in Iguana therefore checks prediction sets
    /// for prefix overlap, not only for shared terminals.
    ///
    /// Not checking for prefix sharing can manifest itself in two ways: the
    /// LL(1) path rejects an input that GLL accepts, or it silently returns
    /// one parse where GLL reports an ambiguity.
    ///
    /// For the grammar
    ///
    /// ```text
    /// S = "ab" "x"
    ///   | "a" "by"
    /// ```
    ///
    /// the prediction sets `{"ab"}` and `{"a"}` are disjoint, but parsing from
    /// `S`, for the input `aby`, longest match takes the longer `"ab"`, enters
    /// the first alternative, and fails at `y`, though the input parses as
    /// `"a" "by"`. A two-phase parser rejects `aby` earlier at the lexing
    /// phase: the scanner takes `ab` and cannot match `y` as it's not part of
    /// the language. In Iguana the grammar alone defines the language, and
    /// `aby` is a sentence. The main problem here is that `S` should not be
    /// classified as LL(1) for Iguana as it rejects a sentence that belongs
    /// to the accepted language.
    ///
    /// For the grammar
    ///
    /// ```text
    /// Expr = Int "." Id
    ///      | Float
    ///
    /// Int   = [0-9]+
    /// Float = [0-9]+ "." [fd]?
    /// Id    = [a-z]+
    /// ```
    ///
    /// the prediction sets `{Int}` and `{Float}` are disjoint, but the input
    /// `1.f` parses both ways: as a selector on an integer literal
    /// (`Int "." Id`) and as a single floating-point literal (`Float`). GLL
    /// reports the ambiguity; longest match commits to the longer `Float` and
    /// returns one parse, hiding it. This is the selector example of the Java
    /// grammar, where `1.f` is both a field access and a float literal.
    ///
    /// Checking prefix overlap alone would be too strict. Some overlapping
    /// pairs can never compete, because a follow restriction stands between
    /// them: a follow entry `C -> {X}` says that `C` appears only where `X`
    /// cannot, so when the terminal overlapping `C` is exactly `X`, no
    /// position admits both, and the pair is not a conflict. For the grammar
    ///
    /// ```text
    /// S = A B !>> X C
    ///
    /// B = X
    ///   | ε
    /// ```
    ///
    /// the prediction sets of `B` are `{X}` and the follow set of `B`, which
    /// holds `C -> {X}`. `X` predicts the first alternative and `C` predicts
    /// ε, but wherever `C` can follow `B`, `X` cannot appear, and wherever
    /// `X` can appear, `C` cannot follow: the two never predict at the same
    /// position, and `B` stays LL(1). Without this exception layout would
    /// never be LL(1): the follow set of the layout body holds
    /// `/ -> {WhiteSpace, Comment}`, and `/` prefix-overlaps Comment; the
    /// restriction is what makes the overlap harmless.
    pub fn has_disjoint_alternatives(&self, nt: &Nonterminal) -> bool {
        let alternatives = self.grammar.alternatives(nt);
        let prediction_sets: Vec<_> = alternatives
            .iter()
            .map(|alt| self.prediction_set(nt, alt))
            .collect();
        for i in 0..prediction_sets.len() {
            for j in i + 1..prediction_sets.len() {
                if self.prediction_sets_conflict(&prediction_sets[i], &prediction_sets[j]) {
                    return false;
                }
            }
        }
        true
    }

    /// Whether two prediction sets conflict: they share a terminal, or a
    /// terminal of one prefix-overlaps a terminal of the other and neither is
    /// among the other's restrictions. The identity test runs first because
    /// EOF has no language: only identity can catch two nullable alternatives
    /// meeting on end of input.
    fn prediction_sets_conflict(
        &self,
        a: &FxHashMap<Terminal, FxHashSet<Terminal>>,
        b: &FxHashMap<Terminal, FxHashSet<Terminal>>,
    ) -> bool {
        if a.keys().any(|t| b.contains_key(t)) {
            return true;
        }
        for (terminal_a, restrictions_a) in a {
            for (terminal_b, restrictions_b) in b {
                if !restrictions_a.contains(terminal_b)
                    && !restrictions_b.contains(terminal_a)
                    && self.prefix_overlap(terminal_a, terminal_b)
                {
                    return true;
                }
            }
        }
        false
    }

    /// True if the languages of `a` and `b` prefix-overlap in either direction:
    /// a string of one is a prefix of (or equal to) a string of the other.
    fn prefix_overlap(&self, a: &Terminal, b: &Terminal) -> bool {
        self.is_language_prefix(a, b) || self.is_language_prefix(b, a)
    }

    /// True if some string of `a`'s language is a prefix of (or equal to) some
    /// string of `b`'s language.
    fn is_language_prefix(&self, a: &Terminal, b: &Terminal) -> bool {
        match (self.terminal_dfa(a), self.terminal_dfa(b)) {
            (Some(da), Some(db)) => da.is_prefix_of(&db),
            // EOF and other sentinels have no lexical rule and no scanned
            // language, so they cannot prefix-overlap a real terminal.
            _ => false,
        }
    }

    /// The DFA of a terminal's language, with its excludes and excepts baked in,
    /// or `None` for a sentinel with no lexical rule. Built the same way the
    /// scanner builds terminal DFAs, so `\` and except restrictions carry into
    /// the compared languages. Built on demand: only the FIRST terminals of a
    /// candidate LL(1) nonterminal's alternatives are ever compared.
    fn terminal_dfa(&self, terminal: &Terminal) -> Option<Dfa> {
        let rule = self.grammar.lexical_rule(terminal)?;
        // The terminal id only labels accept states, which the prefix check does
        // not read, so a placeholder suffices.
        let placeholder = TerminalId(0);
        let nfa = if rule.except.is_empty() {
            Nfa::from_regex(&rule.regex, placeholder)
        } else {
            let excepts: Vec<_> = rule
                .except
                .iter()
                .map(|e| &self.grammar.except_terminal(e).1.regex)
                .collect();
            Nfa::with_excepts(&rule.regex, placeholder, &excepts)
        };
        Some(Dfa::from_nfa(&nfa))
    }

    /// A nonterminal is LL(1) if it and all transitively reachable nonterminals
    /// are LL(1), read from the `ll1_nonterminals` classification.
    pub fn is_ll1(&self, nt: &Nonterminal) -> bool {
        self.ll1_nonterminals.contains(&nt.name)
            && self
                .reachability
                .reachable(nt)
                .iter()
                .all(|referenced| self.ll1_nonterminals.contains(&referenced.name))
    }

    /// A nonterminal is LL(1) if no pair of its alternatives' prediction sets
    /// conflicts (`has_disjoint_alternatives`).
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
    /// again or return to the parent rule. One lookahead terminal can pick
    /// between them only when no input is valid for both:
    /// `FIRST(continuation)` and the non-recursive follow entries of Plus
    /// must not conflict, by the same identity and prefix conditions as
    /// `has_disjoint_alternatives`. The non-recursive follow excludes the
    /// contribution that the self-recursive rule would otherwise inject into
    /// `FOLLOW(Plus)`. When the sets conflict, Plus is not LL(1), and the
    /// parser uses GLL to explore both derivations.
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
        // The non-recursive follow of nt: what can follow nt per some rule
        // other than nt's own. Plus rules have the shape
        // `Plus_n = Plus_n continuation | base` and nt is referenced only at
        // position 0 of the recursive alternative, so skipping nt's own rule
        // entirely removes exactly the self-recursive contribution. Each
        // reference to nt elsewhere contributes:
        // - the FIRST set of the symbols after it (skipping longest-match
        //   symbols, which contribute nothing), under the `!>>` of the
        //   reference itself (e.g., `(Alpha|Digit)+ !>> Alpha !>> Digit`);
        // - the follow set of the rule containing the reference when the
        //   suffix is entirely nullable, with the reference's `!>>` added to
        //   each terminal's restrictions.
        let mut non_recursive_follow = FxHashMap::default();
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
                    let symbol_restrictions = self.follow_restrictions(symbol);
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
                    for t in local {
                        Self::insert_follow(&mut non_recursive_follow, t, &symbol_restrictions);
                    }
                    if all_nullable {
                        for (t, restrictions) in &self.follow_sets[rule] {
                            let mut combined = restrictions.clone();
                            combined.extend(symbol_restrictions.iter().cloned());
                            Self::insert_follow(&mut non_recursive_follow, t.clone(), &combined);
                        }
                    }
                }
            }
        }
        if first_continuation
            .iter()
            .any(|c| non_recursive_follow.contains_key(c))
        {
            return false;
        }
        // The loop matches the continuation greedily and stops only when the
        // continuation fails, so continuing and stopping compete like the two
        // alternatives of a rule: FIRST(continuation) predicts one more
        // iteration, the non-recursive follow predicts the stop. The same
        // conflict test as in `prediction_sets_conflict` applies: a shared
        // terminal or an unrestricted prefix overlap means one position can
        // demand both, and the greedy continue would then overrun the stop.
        // Classified LL(1), `S = "a"+ "ab"` would eat the second "a" of `aab`
        // that "ab" needs, and `S = "ab"+ "a" "b"` would eat a second "ab" of
        // `abab` where the only parse stops after one; the conflict test sends
        // both to GLL instead.
        !first_continuation.iter().any(|c| {
            non_recursive_follow
                .iter()
                .any(|(f, restrictions)| !restrictions.contains(c) && self.prefix_overlap(c, f))
        })
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
    /// unwrapped first, the same set `follow_restrictions` unwraps.
    fn excludes_following(&self, symbol: &Symbol, target: &FxHashSet<Terminal>) -> bool {
        match symbol {
            Symbol::FollowRestriction {
                restrictions,
                layout_aware: false,
                ..
            } => {
                let mut excluded = FxHashSet::default();
                for r in restrictions {
                    if let Definition::Terminal(t) = self.grammar.definition(r.resolve()) {
                        excluded.insert(t.clone());
                    }
                }
                target.iter().all(|t| excluded.contains(t))
            }
            // A `!>>>` restriction excludes its terminals after the layout,
            // not at the position immediately after the symbol, so its
            // terminals are not counted. The walk still recurses into the
            // wrapped symbol, which may carry a plain `!>>` of its own.
            Symbol::FollowRestriction {
                symbol,
                layout_aware: true,
                ..
            } => self.excludes_following(symbol, target),
            Symbol::Labeled { symbol, .. }
            | Symbol::Binding { symbol, .. }
            | Symbol::Except { symbol, .. }
            | Symbol::PrecedeRestriction { symbol, .. }
            | Symbol::Exclude { symbol, .. } => self.excludes_following(symbol, target),
            _ => false,
        }
    }

    /// The `!>>` terminals attached to a symbol reference, collected through
    /// the transparent wrappers (`Labeled`, `Binding`, `Except`,
    /// `PrecedeRestriction`, `Exclude`). A `!>>>` restriction contributes
    /// nothing: it forbids its terminals after the layout, not at the
    /// position immediately after the symbol.
    fn follow_restrictions(&self, symbol: &Symbol) -> FxHashSet<Terminal> {
        let mut restrictions = FxHashSet::default();
        let mut current = symbol;
        loop {
            match current {
                Symbol::FollowRestriction {
                    symbol,
                    restrictions: restricted,
                    layout_aware,
                } => {
                    if !layout_aware {
                        for r in restricted {
                            if let Definition::Terminal(t) = self.grammar.definition(r.resolve()) {
                                restrictions.insert(t.clone());
                            }
                        }
                    }
                    current = symbol;
                }
                Symbol::Labeled { symbol, .. }
                | Symbol::Binding { symbol, .. }
                | Symbol::Except { symbol, .. }
                | Symbol::PrecedeRestriction { symbol, .. }
                | Symbol::Exclude { symbol, .. } => current = symbol,
                _ => return restrictions,
            }
        }
    }

    /// Records in `follow_set` that `terminal` can appear as a follow of a
    /// symbol, together with the `!>>` restrictions at the position the
    /// symbol appears. Returns true if `follow_set` changed, which drives
    /// the fixpoint in `calc_follow_sets`.
    ///
    /// Three cases:
    /// - `terminal` is in its own `restrictions`: the position's `!>>` forbids
    ///   exactly this terminal, so it cannot appear there, and nothing is
    ///   recorded.
    /// - `terminal` is new: it enters with the position's restrictions.
    /// - `terminal` already has an entry from another position: the entry
    ///   keeps only the restrictions both positions impose, since the
    ///   classifier may only rely on a restriction that holds wherever the
    ///   terminal can appear.
    fn insert_follow(
        follow_set: &mut FxHashMap<Terminal, FxHashSet<Terminal>>,
        terminal: Terminal,
        restrictions: &FxHashSet<Terminal>,
    ) -> bool {
        if restrictions.contains(&terminal) {
            return false;
        }
        if let Some(existing) = follow_set.get_mut(&terminal) {
            let before = existing.len();
            existing.retain(|r| restrictions.contains(r));
            existing.len() != before
        } else {
            follow_set.insert(terminal, restrictions.clone());
            true
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

    /// FOLLOW(A) maps each terminal that can appear immediately after A in
    /// some sentential form to the follow restrictions (`!>>`) at the
    /// positions where it can appear, as described on `follow_sets`. Every
    /// nonterminal can be a start symbol, so EOF is in all follow sets.
    ///
    /// For each production A → α B β, the position of B contributes to
    /// FOLLOW(B):
    /// - each terminal of FIRST(β), under B's own `!>>` restrictions;
    /// - each terminal of FOLLOW(A) when β is nullable, under its
    ///   restrictions plus B's.
    fn calc_follow_sets(&mut self) {
        for nonterminal in self.grammar.nonterminals() {
            self.follow_sets.insert(nonterminal, FxHashMap::default());
        }

        for nonterminal in self.grammar.nonterminals() {
            self.follow_sets
                .get_mut(nonterminal)
                .unwrap()
                .insert(Self::eof(), FxHashSet::default());
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

                        let symbol_restrictions = self.follow_restrictions(symbol);

                        // Add FIRST(β) to FOLLOW(B) where β = symbols[i+1..]
                        let suffix = &symbols[i + 1..];
                        for s in suffix {
                            let firsts = self.first_of_symbol(s);
                            let follow_b = self.follow_sets.get_mut(nt_b).unwrap();
                            for t in firsts {
                                changed |= Self::insert_follow(follow_b, t, &symbol_restrictions);
                            }
                            if !self.is_nullable(s) {
                                break;
                            }
                        }

                        // If the entire suffix is nullable, add FOLLOW(A) to FOLLOW(B)
                        if suffix.iter().all(|s| self.is_nullable(s)) {
                            let follow_a: Vec<_> = self.follow_sets[nonterminal]
                                .iter()
                                .map(|(t, r)| (t.clone(), r.clone()))
                                .collect();
                            let follow_b = self.follow_sets.get_mut(nt_b).unwrap();
                            for (t, restrictions) in follow_a {
                                let mut combined = restrictions;
                                combined.extend(symbol_restrictions.iter().cloned());
                                changed |= Self::insert_follow(follow_b, t, &combined);
                            }
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
        assert!(follow_e.contains_key(&rparen));
        assert!(follow_e.contains_key(&eof));

        // FOLLOW(Ep) = FOLLOW(E) = { ")", EOF }
        let follow_ep = &ff.follow_sets[grammar.nonterminal("Ep").unwrap()];
        assert!(follow_ep.contains_key(&rparen));
        assert!(follow_ep.contains_key(&eof));

        // FOLLOW(T) = { "+", ")", EOF }
        let follow_t = &ff.follow_sets[grammar.nonterminal("T").unwrap()];
        assert!(follow_t.contains_key(&plus));
        assert!(follow_t.contains_key(&rparen));
        assert!(follow_t.contains_key(&eof));

        // FOLLOW(Tp) = FOLLOW(T) = { "+", ")", EOF }
        let follow_tp = &ff.follow_sets[grammar.nonterminal("Tp").unwrap()];
        assert!(follow_tp.contains_key(&plus));
        assert!(follow_tp.contains_key(&rparen));
        assert!(follow_tp.contains_key(&eof));

        // FOLLOW(F) = { "*", "+", ")", EOF }
        let follow_f = &ff.follow_sets[grammar.nonterminal("F").unwrap()];
        assert!(follow_f.contains_key(&star));
        assert!(follow_f.contains_key(&plus));
        assert!(follow_f.contains_key(&rparen));
        assert!(follow_f.contains_key(&eof));

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
        assert!(follow_a.contains_key(&tb));
        assert!(follow_a.contains_key(&tc));
        assert!(follow_a.contains_key(&td));

        // FOLLOW(B) = { "c", "d" }
        let follow_b = &ff.follow_sets[grammar.nonterminal("B").unwrap()];
        assert!(follow_b.contains_key(&tc));
        assert!(follow_b.contains_key(&td));

        // FOLLOW(C) = { "d" }
        let follow_c = &ff.follow_sets[grammar.nonterminal("C").unwrap()];
        assert!(follow_c.contains_key(&td));
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
        assert!(follow_s.contains_key(&eof));
        assert!(follow_s.contains_key(&td));

        // FOLLOW(B) = { "c" }
        let follow_b = &ff.follow_sets[grammar.nonterminal("B").unwrap()];
        assert!(follow_b.contains_key(&tc));
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
        assert!(follow_a.contains_key(&ta));

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
        assert!(follow_a.contains_key(&tb));

        // S has one alternative so disjoint trivially, but NOT LL(1)
        // because it references A which is not LL(1)
        assert!(ff.has_disjoint_alternatives(grammar.nonterminal("S").unwrap()));
        assert!(!ff.is_ll1(grammar.nonterminal("S").unwrap()));
        // A is NOT LL(1): prediction(A -> "b") = {"b"}, prediction(A -> e) = FOLLOW(A) contains {"b"}
        assert!(!ff.is_ll1(grammar.nonterminal("A").unwrap()));
    }

    // ---------------------------------------------------------------
    // Grammar 6: prefix conflict between FIRST and a follow terminal
    //
    //   S = O "a" "b"
    //   O = "ab" | ε
    //
    // The prediction sets of O are {"ab"} and FOLLOW(O) = {"a", EOF}:
    // disjoint as identities, but "a" is a prefix of "ab". On input `ab`
    // the dispatch matches "ab" and commits, though the parse needed
    // ε and then "a".
    // ---------------------------------------------------------------
    #[test]
    fn test_follow_prefix_conflict_is_not_ll1() {
        let grammar: Grammar = grammar_def!("opt_prefix",
            syntax: [
                syntax_rule!("S" => alternative!(id!("O"), lit!("a"), lit!("b"))),
                syntax_rule!("O" => priority_level!(
                    alternative!(lit!("ab")),
                    alternative!()
                ))
            ]
        )
        .try_into()
        .unwrap();
        let ff = FirstFollowSets::new(&grammar);
        assert!(!ff.is_ll1(grammar.nonterminal("O").unwrap()));
        assert!(!ff.is_ll1(grammar.nonterminal("S").unwrap()));
    }

    // The mirror direction: FIRST(O) = {"a"} and the follow terminal is
    // "ab". On input `ab` the dispatch matches "a" and commits, though the
    // parse needed ε and then "ab". Both directions must conflict.
    #[test]
    fn test_follow_prefix_conflict_mirror_is_not_ll1() {
        let grammar: Grammar = grammar_def!("opt_prefix_mirror",
            syntax: [
                syntax_rule!("S" => alternative!(id!("O"), lit!("ab"))),
                syntax_rule!("O" => priority_level!(
                    alternative!(lit!("a")),
                    alternative!()
                ))
            ]
        )
        .try_into()
        .unwrap();
        let ff = FirstFollowSets::new(&grammar);
        assert!(!ff.is_ll1(grammar.nonterminal("O").unwrap()));
    }

    // ---------------------------------------------------------------
    // Grammar 7: the same overlap, exempted by a follow restriction
    //
    //   S = O !>> AB A B
    //   O = AB | ε          AB = "ab"   A = "a"   B = "b"
    //
    // FOLLOW(O) = {A -> {AB}, EOF}: A is a prefix of AB, but A follows O
    // only where the `!>>` forbids AB, so no input matches both and O
    // stays LL(1). This is the layout shape in miniature.
    // ---------------------------------------------------------------
    fn restricted_opt_grammar_def() -> GrammarDef {
        use crate::grammar::regex::Regex;
        use crate::grammar::symbols::Identifier;
        use crate::lexical_rule;
        let restricted_o = Symbol::FollowRestriction {
            symbol: Box::new(id!("O")),
            restrictions: vec![Identifier {
                name: "AB".into(),
                definition: None,
            }],
            layout_aware: false,
        };
        grammar_def!("opt_prefix_restricted",
            syntax: [
                syntax_rule!("S" => alternative!(restricted_o, id!("A"), id!("B"))),
                syntax_rule!("O" => priority_level!(
                    alternative!(id!("AB")),
                    alternative!()
                )),
                syntax_rule!("T" => alternative!(id!("O"), id!("AB")))
            ],
            lexical: [
                lexical_rule!("AB" => Regex::literal("ab")),
                lexical_rule!("A" => Regex::literal("a")),
                lexical_rule!("B" => Regex::literal("b"))
            ]
        )
    }

    #[test]
    fn test_follow_restriction_exempts_prefix_overlap() {
        let mut def = restricted_opt_grammar_def();
        // Keep only S and O: the one position of O carries the `!>>`.
        def.syntax_rules.retain(|r| r.head.name != "T");
        let grammar: Grammar = def.try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);
        let o = grammar.nonterminal("O").unwrap();
        let ab = Terminal::new("AB");
        let a = Terminal::new("A");
        // The follow entry records the position's restriction.
        assert_eq!(ff.follow_sets[o][&a], [ab].into_iter().collect());
        assert!(ff.is_ll1(o));
    }

    // Grammar 7 plus `T = O AB`: the second position lets AB follow O with no
    // restriction, so O's one parse function is exposed there and the
    // exemption no longer applies anywhere.
    #[test]
    fn test_unrestricted_position_defeats_the_restriction() {
        let grammar: Grammar = restricted_opt_grammar_def().try_into().unwrap();
        let ff = FirstFollowSets::new(&grammar);
        let o = grammar.nonterminal("O").unwrap();
        // AB enters FOLLOW(O) through the unrestricted position in T, so the
        // identity check already fails: prediction(O -> AB) contains AB.
        assert!(ff.follow_sets[o].contains_key(&Terminal::new("AB")));
        assert!(!ff.is_ll1(o));
    }

    #[test]
    fn test_insert_follow_drops_a_self_restricted_terminal() {
        let mut entries = FxHashMap::default();
        let a = Terminal::new("A");
        let restrictions: FxHashSet<_> = [a.clone()].into_iter().collect();
        assert!(!FirstFollowSets::insert_follow(
            &mut entries,
            a.clone(),
            &restrictions
        ));
        assert!(entries.is_empty());
    }

    #[test]
    fn test_insert_follow_keeps_the_common_restrictions() {
        let mut entries = FxHashMap::default();
        let a = Terminal::new("A");
        let x = Terminal::new("X");
        let y = Terminal::new("Y");
        let position1: FxHashSet<_> = [x.clone(), y.clone()].into_iter().collect();
        assert!(FirstFollowSets::insert_follow(
            &mut entries,
            a.clone(),
            &position1
        ));
        assert_eq!(entries[&a], position1);
        // A second position restricting only X leaves {X}; a third with no
        // restrictions leaves the terminal unconditional.
        let position2: FxHashSet<_> = [x.clone()].into_iter().collect();
        assert!(FirstFollowSets::insert_follow(
            &mut entries,
            a.clone(),
            &position2
        ));
        assert_eq!(entries[&a], position2);
        assert!(FirstFollowSets::insert_follow(
            &mut entries,
            a.clone(),
            &FxHashSet::default()
        ));
        assert!(entries[&a].is_empty());
    }
}
