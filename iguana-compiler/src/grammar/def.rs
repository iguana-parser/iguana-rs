use std::fmt::Display;
use std::str::FromStr;

use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    grammar::{
        regex::Regex,
        symbols::{
            Definition, DefinitionId, Expr, Identifier, Nonterminal, Restrictions, Symbol, Terminal,
        },
        transformations::{
            ebnf_to_bnf, exact_keyword_match, exclude_desugaring, layout_insertion,
            precedence_desugaring, transform_regex, transform_syntax_rule, visit_syntax_rule,
        },
    },
    priority_level,
    spans::GrammarSpans,
    validation::validate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Associativity {
    Left,
    Right,
    NonAssoc,
}

/// Controls how layout (whitespace/comments) is inserted between symbols in a syntax rule.
///
/// By default, the grammar's layout definition is inserted between consecutive symbols in a rule.
/// For character-level rules (e.g., `Id = Char+ !>> Char`), layout must be suppressed to avoid
/// inserting whitespace between individual characters. These character-level definitions correspond
/// to lexical definitions in scannerless parsers like Rascal or SDF. A custom layout can also be
/// specified per rule to use a different layout than the grammar default.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum LayoutStrategy {
    #[default]
    Default,
    None,
    Custom(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alternative {
    pub symbols: Vec<Symbol>,
    pub label: Option<String>,
}

impl Alternative {
    pub fn len(&self) -> usize {
        self.symbols.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn empty() -> Self {
        Self {
            symbols: vec![],
            label: None,
        }
    }

    pub fn display_name(&self, grammar: &Grammar) -> String {
        let symbols: Vec<String> = self
            .symbols
            .iter()
            .map(|s| s.display_name(grammar))
            .collect();
        let mut result = symbols.join(" ");
        if let Some(label) = &self.label {
            result = format!("{} #{}", result, label);
        }
        result
    }
}

impl Display for Alternative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbols = self.symbols.iter().join(" ");
        write!(f, "{}", symbols)?;
        if let Some(label) = &self.label {
            write!(f, " #{}", label)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SyntaxRule {
    pub head: Nonterminal,
    pub priority_levels: Vec<PriorityLevel>,
    pub layout: LayoutStrategy,
}

impl SyntaxRule {
    pub fn new(head: Nonterminal, priority_levels: Vec<PriorityLevel>) -> Self {
        Self {
            head,
            priority_levels,
            layout: LayoutStrategy::Default,
        }
    }

    /// Every alternative across all priority levels, in order.
    pub fn alternatives(&self) -> impl Iterator<Item = &Alternative> {
        self.priority_levels
            .iter()
            .flat_map(|level| &level.alternatives)
    }

    /// Whether this rule has an alternative labeled `label`.
    pub fn has_label(&self, label: &str) -> bool {
        self.alternatives()
            .any(|alternative| alternative.label.as_deref() == Some(label))
    }
}

#[derive(Debug)]
pub struct PriorityLevel {
    pub alternatives: Vec<Alternative>,
    pub associativity: Option<Associativity>,
}

impl PriorityLevel {
    pub fn new(alternatives: Vec<Alternative>) -> Self {
        Self {
            alternatives,
            associativity: None,
        }
    }

    pub fn with_associativity(
        alternatives: Vec<Alternative>,
        associativity: Option<Associativity>,
    ) -> Self {
        Self {
            alternatives,
            associativity,
        }
    }
}

impl From<Alternative> for PriorityLevel {
    fn from(alt: Alternative) -> Self {
        Self {
            alternatives: vec![alt],
            associativity: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LexicalRule {
    pub head: Terminal,
    pub regex: Regex,
    pub except: Vec<Identifier>,
    pub follow_restriction: Vec<Identifier>,
    pub precede_restriction: Option<Identifier>,
}

impl LexicalRule {
    pub fn new(head: Terminal, regex: Regex) -> Self {
        Self {
            head,
            regex,
            except: vec![],
            follow_restriction: vec![],
            precede_restriction: None,
        }
    }
}

impl Display for LexicalRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.head, self.regex)?;
        for except in &self.except {
            write!(f, " \\ {}", except)?;
        }
        for restriction in &self.follow_restriction {
            write!(f, " !>> {}", restriction)?;
        }
        if let Some(restriction) = &self.precede_restriction {
            write!(f, " !<< {}", restriction)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct GrammarDef {
    pub name: String,
    pub syntax_rules: Vec<SyntaxRule>,
    pub lexical_rules: Vec<LexicalRule>,
    // The layout nonterminal is used to define whitespace/comments that can
    // appear anywhere in the program.
    pub layout: Option<Symbol>,
    // The lexical rules marked with `@Identifier`.
    pub identifier_rules: Vec<Identifier>,
}

impl GrammarDef {
    /// A map from each rule name to its definition ID.
    pub fn symbol_table(&self) -> SymbolTable {
        create_symbol_table(&self.syntax_rules, &self.lexical_rules).1
    }

    pub fn resolve(self) -> GrammarDef {
        let symbol_table = self.symbol_table();
        let (syntax_rules, lexical_rules) =
            resolve_identifiers(self.syntax_rules, self.lexical_rules, &symbol_table);
        GrammarDef {
            name: self.name,
            syntax_rules,
            lexical_rules,
            layout: self.layout,
            identifier_rules: self.identifier_rules,
        }
    }

    pub fn for_each_symbol<'a>(&'a self, f: &mut impl FnMut(&'a Symbol)) {
        for rule in &self.syntax_rules {
            visit_syntax_rule(rule, f);
        }
    }

    pub fn for_each_identifier<'a>(&'a self, f: &mut impl FnMut(&'a Identifier)) {
        self.for_each_symbol(&mut |symbol| match symbol {
            Symbol::Identifier(id) | Symbol::Call { name: id, .. } => f(id),
            Symbol::Restricted { restrictions, .. } => restrictions.ids().for_each(&mut *f),
            _ => {}
        });
        for rule in &self.lexical_rules {
            visit_regex_identifiers(&rule.regex, f);
        }
    }

    pub fn to_grammar(self, dump: &[Phase]) -> Result<Grammar, Vec<String>> {
        let resolved = self.resolve();
        // A `GrammarDef` has no parse tree, so no error gets a span.
        let errors = validate(&resolved, &GrammarSpans::default());
        if !errors.is_empty() {
            return Err(errors.into_iter().map(|error| error.message).collect());
        }
        build_grammar(resolved, dump)
    }
}

impl TryFrom<GrammarDef> for Grammar {
    type Error = Vec<String>;

    fn try_from(grammar_def: GrammarDef) -> Result<Self, Self::Error> {
        grammar_def.to_grammar(&[])
    }
}

fn visit_regex_identifiers<'a>(regex: &'a Regex, f: &mut impl FnMut(&'a Identifier)) {
    match regex {
        Regex::Identifier(id) => f(id),
        Regex::Seq(rs) | Regex::Alt(rs) => {
            for r in rs {
                visit_regex_identifiers(r, f);
            }
        }
        Regex::Star(r) | Regex::Plus(r) | Regex::Opt(r) => {
            visit_regex_identifiers(r, f);
        }
        Regex::Char(_) | Regex::CharRange(_) | Regex::CharClass(_) | Regex::Epsilon => {}
    }
}

impl Display for SyntaxRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The layout annotation (@Layout / @NoLayout / @WithLayout) is a
        // grammar-level concern, so GrammarDef's Display emits it.
        writeln!(f, "{}", self.head)?;
        for (level_idx, level) in self.priority_levels.iter().enumerate() {
            for (alt_idx, alternative) in level.alternatives.iter().enumerate() {
                // `=` opens the first level, `>` each later level, `|` the rest.
                let prefix = match (level_idx, alt_idx) {
                    (0, 0) => "  = ",
                    (_, 0) => "  > ",
                    _ => "  | ",
                };
                write!(f, "{prefix}{}", alternative.symbols.iter().join(" "))?;
                if let Some(label) = &alternative.label {
                    write!(f, " #{}", label)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Display for GrammarDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "grammar {}\n", self.name)?;
        let layout_name = match &self.layout {
            Some(Symbol::Identifier(id)) => Some(id.name.as_str()),
            _ => None,
        };
        for rule in &self.syntax_rules {
            if layout_name == Some(rule.head.name.as_str()) {
                writeln!(f, "@Layout")?;
            } else {
                match &rule.layout {
                    LayoutStrategy::None => writeln!(f, "@NoLayout")?,
                    LayoutStrategy::Custom(id) => writeln!(f, "@WithLayout({})", id.name)?,
                    LayoutStrategy::Default => {}
                }
            }
            writeln!(f, "{}", rule)?;
        }
        for lexical_rule in &self.lexical_rules {
            let is_identifier_rule = self
                .identifier_rules
                .iter()
                .any(|id| id.name == lexical_rule.head.name);
            if layout_name == Some(lexical_rule.head.name.as_str()) {
                writeln!(f, "@Layout @Regex")?;
            } else if is_identifier_rule {
                writeln!(f, "@Identifier @Regex")?;
            } else {
                writeln!(f, "@Regex")?;
            }
            writeln!(f, "{} = {}", lexical_rule.head, lexical_rule.regex)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct SymbolTable {
    symbol_table: FxHashMap<String, DefinitionId>,
    next_id: u16,
}

impl SymbolTable {
    pub fn insert(&mut self, name: String) -> DefinitionId {
        let def_id = DefinitionId(self.next_id);
        self.next_id += 1;
        self.symbol_table.insert(name, def_id);
        def_id
    }
    pub fn get(&self, name: &str) -> Option<DefinitionId> {
        self.symbol_table.get(name).copied()
    }
}

/// Creates a map from names to definitions (terminal or nonterminal).
pub fn create_symbol_table(
    syntax_rules: &[SyntaxRule],
    lexical_rules: &[LexicalRule],
) -> (Vec<Definition>, SymbolTable) {
    let mut symbol_table = SymbolTable::default();
    let mut definitions = vec![];
    for lexical_rule in lexical_rules {
        let terminal = &lexical_rule.head;
        symbol_table.insert(terminal.name.clone());
        definitions.push(Definition::Terminal(terminal.clone()));
    }
    for syntax_rule in syntax_rules {
        symbol_table.insert(syntax_rule.head.name.clone());
        definitions.push(Definition::Nonterminal(syntax_rule.head.clone()));
    }
    (definitions, symbol_table)
}

/// Converts string literals (e.g., `"+"`) in syntax rules into terminal references
/// and generates corresponding lexical rules. Specifically:
/// 1. Converts each `Symbol::Literal` into a `Symbol::Identifier` referencing a terminal
/// 2. Creates a lexical rule that matches the literal string exactly
///
/// The name of the terminal is the same as the string literal.
fn add_lexical_rules_for_literals(
    syntax_rules: Vec<SyntaxRule>,
    lexical_rules: &mut Vec<LexicalRule>,
) -> Vec<SyntaxRule> {
    let mut transformed_syntax_rules = vec![];
    let mut added_terminals = FxHashSet::default();
    for rule in syntax_rules {
        let transformed = transform_syntax_rule(rule, |s| {
            add_lexical_rules(s, lexical_rules, &mut added_terminals)
        });
        transformed_syntax_rules.push(transformed);
    }
    transformed_syntax_rules
}

fn add_lexical_rules(
    symbol: Symbol,
    lexical_rules: &mut Vec<LexicalRule>,
    added_terminals: &mut FxHashSet<Terminal>,
) -> Symbol {
    match symbol {
        Symbol::Labeled { label, symbol } => {
            let transformed = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            Symbol::Labeled {
                label,
                symbol: Box::new(transformed),
            }
        }
        Symbol::Literal(name) => {
            let terminal = Terminal::literal(&name);
            let terminal_name = terminal.name.clone();
            if !added_terminals.contains(&terminal) {
                added_terminals.insert(terminal.clone());
                lexical_rules.push(LexicalRule::new(terminal, Regex::literal(&name)));
            }
            Symbol::Identifier(Identifier {
                name: terminal_name,
                definition: None,
            })
        }
        Symbol::Group(symbols) => {
            let transformed_symbols = symbols
                .into_iter()
                .map(|s| add_lexical_rules(s, lexical_rules, added_terminals))
                .collect();
            Symbol::Group(transformed_symbols)
        }
        Symbol::Opt(symbol) => {
            let transformed_symbol = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            Symbol::Opt(Box::new(transformed_symbol))
        }
        Symbol::Alt(symbols) => {
            let transformed_symbols = symbols
                .into_iter()
                .map(|s| add_lexical_rules(s, lexical_rules, added_terminals))
                .collect();
            Symbol::Alt(transformed_symbols)
        }
        Symbol::Star(symbol, sep) => {
            let transformed_symbol = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            match sep {
                Some(sep) => {
                    let transformed_sep = add_lexical_rules(*sep, lexical_rules, added_terminals);
                    Symbol::Star(
                        Box::new(transformed_symbol),
                        Some(Box::new(transformed_sep)),
                    )
                }
                None => Symbol::Star(Box::new(transformed_symbol), None),
            }
        }
        Symbol::Plus(symbol, sep) => {
            let transformed_symbol = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            match sep {
                Some(sep) => {
                    let transformed_sep = add_lexical_rules(*sep, lexical_rules, added_terminals);
                    Symbol::Plus(
                        Box::new(transformed_symbol),
                        Some(Box::new(transformed_sep)),
                    )
                }
                None => Symbol::Plus(Box::new(transformed_symbol), None),
            }
        }
        Symbol::Binding { name, symbol } => {
            let transformed = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            Symbol::Binding {
                name,
                symbol: Box::new(transformed),
            }
        }
        Symbol::Restricted {
            symbol,
            restrictions,
        } => {
            let transformed = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            Symbol::Restricted {
                symbol: Box::new(transformed),
                restrictions,
            }
        }
        _ => symbol,
    }
}

fn resolve_identifiers(
    syntax_rules: Vec<SyntaxRule>,
    lexical_rules: Vec<LexicalRule>,
    symbol_table: &SymbolTable,
) -> (Vec<SyntaxRule>, Vec<LexicalRule>) {
    let syntax_rules = syntax_rules
        .into_iter()
        .map(|rule| {
            let mut rule = transform_syntax_rule(rule, |s| resolve_identifier(s, symbol_table));
            rule.head.origin = rule
                .head
                .origin
                .map(|s| resolve_identifier(s, symbol_table));
            rule
        })
        .collect();
    let lexical_rules = lexical_rules
        .into_iter()
        .map(|mut rule| {
            rule.regex = transform_regex(rule.regex, &mut |regex| match regex {
                Regex::Identifier(id) => Regex::Identifier(Identifier {
                    definition: symbol_table.get(&id.name),
                    name: id.name,
                }),
                other => other,
            });
            rule
        })
        .collect();
    (syntax_rules, lexical_rules)
}

fn resolve_identifier(symbol: Symbol, symbol_table: &SymbolTable) -> Symbol {
    match symbol {
        Symbol::Labeled { label, symbol } => {
            let transformed = resolve_identifier(*symbol, symbol_table);
            Symbol::Labeled {
                label,
                symbol: Box::new(transformed),
            }
        }
        Symbol::Call { name, arguments } => Symbol::Call {
            name: Identifier {
                definition: symbol_table.get(&name.name),
                name: name.name,
            },
            arguments,
        },
        Symbol::Identifier(identifier) => Symbol::Identifier(Identifier {
            definition: symbol_table.get(&identifier.name),
            name: identifier.name,
        }),
        Symbol::Group(symbols) => {
            let resolved_symbols = symbols
                .into_iter()
                .map(|s| resolve_identifier(s, symbol_table))
                .collect();
            Symbol::Group(resolved_symbols)
        }
        Symbol::Opt(symbol) => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            Symbol::Opt(Box::new(resolved_symbol))
        }
        Symbol::Alt(symbols) => {
            let resolved_symbols = symbols
                .into_iter()
                .map(|s| resolve_identifier(s, symbol_table))
                .collect();
            Symbol::Alt(resolved_symbols)
        }
        Symbol::Star(symbol, sep) => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            match sep {
                Some(sep) => {
                    let transformed_sep = resolve_identifier(*sep, symbol_table);
                    Symbol::Star(Box::new(resolved_symbol), Some(Box::new(transformed_sep)))
                }
                None => Symbol::Star(Box::new(resolved_symbol), None),
            }
        }
        Symbol::Plus(symbol, sep) => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            match sep {
                Some(sep) => {
                    let transformed_sep = resolve_identifier(*sep, symbol_table);
                    Symbol::Plus(Box::new(resolved_symbol), Some(Box::new(transformed_sep)))
                }
                None => Symbol::Plus(Box::new(resolved_symbol), None),
            }
        }
        Symbol::Binding { name, symbol } => Symbol::Binding {
            name,
            symbol: Box::new(resolve_identifier(*symbol, symbol_table)),
        },
        Symbol::Restricted {
            symbol,
            restrictions,
        } => {
            let resolve = |ids: Vec<Identifier>| -> Vec<Identifier> {
                ids.into_iter()
                    .map(|id| Identifier {
                        definition: symbol_table.get(&id.name),
                        name: id.name,
                    })
                    .collect()
            };
            Symbol::Restricted {
                symbol: Box::new(resolve_identifier(*symbol, symbol_table)),
                restrictions: Restrictions {
                    precede: resolve(restrictions.precede),
                    excepts: resolve(restrictions.excepts),
                    follow: resolve(restrictions.follow),
                    layout_aware_follow: resolve(restrictions.layout_aware_follow),
                },
            }
        }
        Symbol::Exclude { symbol, labels } => Symbol::Exclude {
            symbol: Box::new(resolve_identifier(*symbol, symbol_table)),
            labels,
        },
        Symbol::Literal(_) | Symbol::Condition(_) | Symbol::Return(_) => symbol,
    }
}

/// Removes redundant structure such as single-symbol grouping (`(A)` → `A`).
/// A regex serves recognition (match or not), not structure capture, so
/// grouping carries no meaning.
fn simplify_regex(regex: &mut Regex) {
    match regex {
        Regex::Seq(parts) | Regex::Alt(parts) => {
            parts.iter_mut().for_each(simplify_regex);
            if parts.len() == 1 {
                *regex = parts.pop().unwrap();
            }
        }
        Regex::Star(inner) | Regex::Plus(inner) | Regex::Opt(inner) => simplify_regex(inner),
        _ => {}
    }
}

/// A lexical rule's restrictions: excepts (`\`), follow restrictions (`!>>`),
/// and a precede restriction (`!<<`).
#[derive(Default, Clone)]
struct LexicalRestrictions {
    except: Vec<Identifier>,
    follow_restriction: Vec<Identifier>,
    precede_restriction: Option<Identifier>,
}

impl LexicalRestrictions {
    fn of(rule: &LexicalRule) -> Self {
        Self {
            except: rule.except.clone(),
            follow_restriction: rule.follow_restriction.clone(),
            precede_restriction: rule.precede_restriction.clone(),
        }
    }

    fn is_empty(&self) -> bool {
        self.except.is_empty()
            && self.follow_restriction.is_empty()
            && self.precede_restriction.is_none()
    }

    /// Adds `other`'s restrictions, deduping excludes and follow restrictions by
    /// name. Two different precede restrictions can't share a rule, so they are
    /// an error.
    fn merge(&mut self, other: LexicalRestrictions, head: &Terminal, errors: &mut Vec<String>) {
        for restriction in other.except {
            if !self.except.iter().any(|e| e.name == restriction.name) {
                self.except.push(restriction);
            }
        }
        for restriction in other.follow_restriction {
            if !self
                .follow_restriction
                .iter()
                .any(|r| r.name == restriction.name)
            {
                self.follow_restriction.push(restriction);
            }
        }
        if let Some(precede) = other.precede_restriction {
            match &self.precede_restriction {
                Some(existing) if existing.name != precede.name => {
                    errors.push(format!(
                        "`{head}` stacks precede restrictions `!<< {existing}` and `!<< {precede}`; \
                         a rule can have only one"
                    ));
                }
                Some(_) => {}
                None => self.precede_restriction = Some(precede),
            }
        }
    }
}

impl Display for LexicalRestrictions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        parts.extend(self.except.iter().map(|e| format!("\\ {e}")));
        parts.extend(self.follow_restriction.iter().map(|r| format!("!>> {r}")));
        parts.extend(self.precede_restriction.iter().map(|r| format!("!<< {r}")));
        write!(f, "{}", parts.join(" "))
    }
}

/// Lexical-rule names in dependency order: a single-symbol rule's referenced
/// rule comes before it, so restrictions resolve in one forward pass. The
/// reference edges form a DAG; a reference to a non-lexical name has no edge,
/// and a cycle is left for `inline_regex` to report.
fn dependency_order(lexical_rules: &[LexicalRule]) -> Vec<String> {
    let names: FxHashSet<&str> = lexical_rules.iter().map(|r| r.head.name.as_str()).collect();
    let reference: FxHashMap<&str, &str> = lexical_rules
        .iter()
        .filter_map(|rule| match &rule.regex {
            Regex::Identifier(id) if names.contains(id.name.as_str()) => {
                Some((rule.head.name.as_str(), id.name.as_str()))
            }
            _ => None,
        })
        .collect();
    let mut order = Vec::new();
    let mut placed = FxHashSet::default();
    for rule in lexical_rules {
        place(&rule.head.name, &reference, &mut placed, &mut order);
    }
    order
}

/// Post-order placement: a rule's reference is pushed before the rule itself.
/// `placed` both dedups shared references and stops a cyclic chain.
fn place(
    name: &str,
    reference: &FxHashMap<&str, &str>,
    placed: &mut FxHashSet<String>,
    order: &mut Vec<String>,
) {
    if !placed.insert(name.to_string()) {
        return;
    }
    if let Some(&referenced) = reference.get(name) {
        place(referenced, reference, placed, order);
    }
    order.push(name.to_string());
}

/// Inlines `Regex::Identifier` references in lexical rules by substituting them
/// with the referenced rule's regex body. For example, given:
///   Digit = [0-9]
///   Digits = Digit+
/// After inlining, `Digits` becomes `[0-9]+`.
///
/// A referenced rule may itself carry restrictions (`Identifier = … \ Keyword`)
/// that inlining would drop. So before inlining, each rule whose whole body is a
/// single reference inherits its referenced rule's restrictions. A reference
/// inside a larger regex matches only a sub-span, so its restrictions can't be
/// hoisted: that is a grammar error rather than a silent drop.
fn inline_regex_refs(mut lexical_rules: Vec<LexicalRule>) -> Result<Vec<LexicalRule>, Vec<String>> {
    // Simplify bodies so a whole-body reference is a bare `Regex::Identifier`.
    for rule in &mut lexical_rules {
        simplify_regex(&mut rule.regex);
    }
    let rules: FxHashMap<String, &LexicalRule> = lexical_rules
        .iter()
        .map(|rule| (rule.head.name.clone(), rule))
        .collect();

    // Resolve each rule's restrictions in dependency order, so a single-symbol
    // rule's reference is already final when we reach it: the rule's
    // restrictions are its own plus its reference's.
    let mut errors = Vec::new();
    let mut resolved: FxHashMap<String, LexicalRestrictions> = FxHashMap::default();
    for name in dependency_order(&lexical_rules) {
        let rule = rules[name.as_str()];
        let mut restrictions = LexicalRestrictions::of(rule);
        if let Regex::Identifier(id) = &rule.regex {
            if let Some(referenced) = resolved.get(&id.name) {
                restrictions.merge(referenced.clone(), &rule.head, &mut errors);
            }
        }
        resolved.insert(name, restrictions);
    }

    // A reference inside a compound regex matches a sub-span, so the referenced
    // rule's restrictions can't carry over. Reject it instead of dropping them.
    if errors.is_empty() {
        for rule in &lexical_rules {
            if matches!(rule.regex, Regex::Identifier(_)) {
                continue;
            }
            let mut referenced: Vec<String> = Vec::new();
            visit_regex_identifiers(&rule.regex, &mut |id| {
                if !referenced.contains(&id.name) {
                    referenced.push(id.name.clone());
                }
            });
            for name in referenced {
                // A non-lexical reference has no entry; `inline_regex` reports it.
                let Some(restrictions) = resolved.get(&name) else {
                    continue;
                };
                if !restrictions.is_empty() {
                    errors.push(format!(
                        "`{}` references `{name}` inside a larger regex, but `{name}` carries \
                         restrictions ({restrictions}) that apply to its whole match; reference it \
                         as the only symbol of a rule, or inline its definition",
                        rule.head
                    ));
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Inline pass: flatten every reference's regex body, attaching each rule's
    // resolved restrictions.
    let regex_map: FxHashMap<String, Regex> = lexical_rules
        .iter()
        .map(|rule| (rule.head.name.clone(), rule.regex.clone()))
        .collect();
    let mut inlined_regexes: FxHashMap<String, Option<Regex>> = FxHashMap::default();
    Ok(lexical_rules
        .into_iter()
        .map(|rule| {
            let restrictions = resolved.remove(&rule.head.name).unwrap();
            LexicalRule {
                head: rule.head,
                regex: inline_regex(rule.regex, &regex_map, &mut inlined_regexes),
                except: restrictions.except,
                follow_restriction: restrictions.follow_restriction,
                precede_restriction: restrictions.precede_restriction,
            }
        })
        .collect())
}

/// Substitutes each `Regex::Identifier` with the referenced rule's regex body,
/// using `inlined_regexes` (`Option<Regex>` values) to track resolution state:
/// `None` means resolution is in progress on the current stack, `Some(regex)`
/// is the cached result. A name found mapped to `None` is a cyclic reference
/// (`A = B`, `B = A`), a grammar error.
// TODO: return a proper Result instead of panicking on errors (cyclic/undefined references).
fn inline_regex(
    regex: Regex,
    regex_map: &FxHashMap<String, Regex>,
    inlined_regexes: &mut FxHashMap<String, Option<Regex>>,
) -> Regex {
    match regex {
        Regex::Identifier(id) => match inlined_regexes.get(&id.name) {
            Some(Some(resolved)) => resolved.clone(),
            Some(None) => panic!(
                "Cyclic reference in @regex rules: '{}' references itself. \
                 Regex rules must be non-recursive.",
                id.name
            ),
            None => {
                let raw = regex_map
                    .get(&id.name)
                    .unwrap_or_else(|| panic!("Undefined @regex rule: '{}'", id.name));
                inlined_regexes.insert(id.name.clone(), None);
                let resolved = inline_regex(raw.clone(), regex_map, inlined_regexes);
                inlined_regexes.insert(id.name.clone(), Some(resolved.clone()));
                resolved
            }
        },
        Regex::Seq(rs) => Regex::Seq(
            rs.into_iter()
                .map(|r| inline_regex(r, regex_map, inlined_regexes))
                .collect(),
        ),
        Regex::Alt(rs) => Regex::Alt(
            rs.into_iter()
                .map(|r| inline_regex(r, regex_map, inlined_regexes))
                .collect(),
        ),
        Regex::Star(r) => Regex::Star(Box::new(inline_regex(*r, regex_map, inlined_regexes))),
        Regex::Plus(r) => Regex::Plus(Box::new(inline_regex(*r, regex_map, inlined_regexes))),
        Regex::Opt(r) => Regex::Opt(Box::new(inline_regex(*r, regex_map, inlined_regexes))),
        Regex::Char(_) | Regex::CharRange(_) | Regex::CharClass(_) | Regex::Epsilon => regex,
    }
}

/// A stage of the grammar transformation pipeline whose output can be dumped via
/// `iguana generate --print-phase`. Listed in the order the pipeline runs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Resolve,
    Keywords,
    Ebnf,
    Exclude,
    Precedence,
    Layout,
}

impl Phase {
    pub const ALL: [Phase; 6] = [
        Phase::Resolve,
        Phase::Keywords,
        Phase::Ebnf,
        Phase::Exclude,
        Phase::Precedence,
        Phase::Layout,
    ];

    /// The token accepted on the command line.
    pub fn token(self) -> &'static str {
        match self {
            Phase::Resolve => "resolve",
            Phase::Keywords => "keywords",
            Phase::Ebnf => "ebnf",
            Phase::Exclude => "exclude",
            Phase::Precedence => "precedence",
            Phase::Layout => "layout",
        }
    }

    /// What the phase shows, used in the dump header and the phase listing.
    pub fn description(self) -> &'static str {
        match self {
            Phase::Resolve => "identifier resolution",
            Phase::Keywords => "exact keyword matching",
            Phase::Ebnf => "EBNF-to-BNF expansion",
            Phase::Exclude => "exclude desugaring",
            Phase::Precedence => "precedence desugaring",
            Phase::Layout => "layout insertion and start wrapping",
        }
    }
}

impl FromStr for Phase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Phase::ALL
            .into_iter()
            .find(|phase| phase.token() == s)
            .ok_or_else(|| format!("unknown phase `{s}`"))
    }
}

/// Prints the grammar after a pipeline phase to stderr, when `dump` requests it.
/// Both the syntax rules (which the transformations rewrite) and the lexical
/// rules (which they leave alone) are shown, so each phase is the complete
/// grammar at that point. Generation continues afterward; this is a diagnostic
/// side effect.
fn dump_phase(phase: Phase, rules: &[SyntaxRule], lexical: &[LexicalRule], dump: &[Phase]) {
    if !dump.contains(&phase) {
        return;
    }
    eprintln!("===== after {} =====", phase.description());
    for rule in rules {
        eprintln!("{rule}");
    }
    for rule in lexical {
        eprintln!("{rule}");
    }
}

/// Runs the full transformation pipeline on a resolved GrammarDef: adds
/// literal terminals, inlines regex references, desugars EBNF/exclude/
/// precedence, inserts layout, and assembles the final Grammar.
fn build_grammar(grammar_def: GrammarDef, dump: &[Phase]) -> Result<Grammar, Vec<String>> {
    let mut lexical_rules = grammar_def.lexical_rules;
    let syntax_rules = grammar_def.syntax_rules;
    // Built before transformations run: precedence and exclude desugaring would
    // otherwise reposition rules and lose source order.
    let source_order: FxHashMap<String, u16> = syntax_rules
        .iter()
        .enumerate()
        .map(|(i, r)| (r.head.name.clone(), i as u16))
        .collect();
    let syntax_rules = add_lexical_rules_for_literals(syntax_rules, &mut lexical_rules);
    let (_, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
    let (syntax_rules, lexical_rules) =
        resolve_identifiers(syntax_rules, lexical_rules, &symbol_table);
    let lexical_rules = inline_regex_refs(lexical_rules)?;
    dump_phase(Phase::Resolve, &syntax_rules, &lexical_rules, dump);
    let (syntax_rules, lexical_rules) =
        exact_keyword_match::transform(syntax_rules, lexical_rules, &grammar_def.identifier_rules);
    dump_phase(Phase::Keywords, &syntax_rules, &lexical_rules, dump);
    let syntax_rules = ebnf_to_bnf::transform(syntax_rules);
    dump_phase(Phase::Ebnf, &syntax_rules, &lexical_rules, dump);
    let (_, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
    let (syntax_rules, lexical_rules) =
        resolve_identifiers(syntax_rules, lexical_rules, &symbol_table);
    let syntax_rules = exclude_desugaring::transform(syntax_rules);
    dump_phase(Phase::Exclude, &syntax_rules, &lexical_rules, dump);
    let (_, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
    let (syntax_rules, lexical_rules) =
        resolve_identifiers(syntax_rules, lexical_rules, &symbol_table);
    let syntax_rules = precedence_desugaring::transform(syntax_rules);
    dump_phase(Phase::Precedence, &syntax_rules, &lexical_rules, dump);
    // Create the final symbol table after all transformations. This must happen
    // after precedence desugaring because desugaring may add parameters to
    // nonterminals (e.g., E becomes E(p)), and the definitions must reflect that.
    let (definitions, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
    let lexical_rules: Vec<LexicalRule> = lexical_rules
        .into_iter()
        .map(|mut r| {
            for except in &mut r.except {
                except.definition = Some(
                    symbol_table
                        .get(&except.name)
                        .unwrap_or_else(|| panic!("Except terminal {} not found", except.name)),
                );
            }
            for restriction in &mut r.follow_restriction {
                restriction.definition =
                    Some(symbol_table.get(&restriction.name).unwrap_or_else(|| {
                        panic!("Follow restriction terminal {} not found", restriction.name)
                    }));
            }
            if let Some(restriction) = &mut r.precede_restriction {
                restriction.definition =
                    Some(symbol_table.get(&restriction.name).unwrap_or_else(|| {
                        panic!(
                            "Precede restriction terminal {} not found",
                            restriction.name
                        )
                    }));
            }
            r
        })
        .collect();

    let layout = grammar_def
        .layout
        .as_ref()
        .map(|layout| resolve_identifier(layout.clone(), &symbol_table));
    let mut syntax_rules = layout_insertion::transform(syntax_rules, layout.as_ref());

    // Every source nonterminal is an entry point and gets a start wrapper rule
    // (`StartX = Layout start:X Layout`, or `StartX = start:X` without layout),
    // so parsing always enters through a wrapper. Derived nonterminals (EBNF
    // expansion, desugaring helpers) and the layout rule are not entry points.
    let layout_name = layout
        .as_ref()
        .and_then(|l| l.as_identifier())
        .map(|id| id.name.clone());
    let is_entry = |r: &SyntaxRule| {
        source_order.contains_key(&r.head.name) && Some(&r.head.name) != layout_name.as_ref()
    };
    let start_rules: Vec<_> = syntax_rules
        .iter()
        .filter(|r| is_entry(r))
        .map(|r| add_start_rule(&r.head, layout.as_ref(), &symbol_table))
        .collect();
    let start_nonterminals: FxHashMap<String, String> = syntax_rules
        .iter()
        .filter(|r| is_entry(r))
        .map(|r| (r.head.name.clone(), format!("Start{}", r.head.name)))
        .collect();
    let start_wrapper_names: FxHashSet<String> = start_nonterminals.values().cloned().collect();
    syntax_rules.extend(start_rules);
    // Dumped after the start wrappers are added so the layout phase shows the
    // StartX rules too, not only layout woven into existing rules. A grammar
    // without layout weaves nothing in but still gets its wrappers here, so the
    // dump runs unconditionally.
    dump_phase(Phase::Layout, &syntax_rules, &lexical_rules, dump);

    let lexical_rules_map: IndexMap<Terminal, LexicalRule> = lexical_rules
        .into_iter()
        .map(|r| (r.head.clone(), r))
        .collect();
    let productions: IndexMap<Nonterminal, Vec<Alternative>> =
        syntax_rules
            .into_iter()
            .fold(IndexMap::new(), |mut acc, r| {
                let alternatives: Vec<Alternative> = r
                    .priority_levels
                    .into_iter()
                    .flat_map(|l| l.alternatives)
                    .collect();
                acc.entry(r.head).or_default().extend(alternatives);
                acc
            });
    Ok(Grammar {
        name: grammar_def.name,
        productions,
        lexical_rules: lexical_rules_map,
        definitions,
        layout,
        symbol_table,
        start_nonterminals,
        start_wrapper_names,
        source_order,
    })
}

fn add_start_rule(
    nt: &Nonterminal,
    layout_identifier: Option<&Symbol>,
    symbol_table: &SymbolTable,
) -> SyntaxRule {
    let nt_name = &nt.name;
    let def_id = symbol_table
        .get(nt_name)
        .unwrap_or_else(|| panic!("{} is not defined", nt_name));
    let name = format!("Start{}", nt_name);
    let identifier = Identifier {
        name: nt_name.into(),
        definition: Some(def_id),
    };
    let symbol = if !nt.parameters.is_empty() {
        let arguments = (0..nt.parameters.len()).map(|_| Expr::Int(0)).collect();
        Symbol::Call {
            name: identifier,
            arguments,
        }
    } else {
        Symbol::Identifier(identifier)
    };

    let start_symbol = Symbol::Labeled {
        label: "start".into(),
        symbol: Box::new(symbol.clone()),
    };
    let symbols = match layout_identifier {
        Some(layout) => vec![layout.clone(), start_symbol, layout.clone()],
        None => vec![start_symbol],
    };
    SyntaxRule {
        head: Nonterminal {
            name,
            origin: Some(symbol),
            parameters: vec![],
        },
        priority_levels: vec![priority_level!(Alternative {
            symbols,
            label: None
        })],
        layout: LayoutStrategy::Default,
    }
}

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    productions: IndexMap<Nonterminal, Vec<Alternative>>,
    lexical_rules: IndexMap<Terminal, LexicalRule>,
    definitions: Vec<Definition>,
    pub symbol_table: SymbolTable,
    pub layout: Option<Symbol>,
    start_nonterminals: FxHashMap<String, String>,
    start_wrapper_names: FxHashSet<String>,
    source_order: FxHashMap<String, u16>,
}

impl PartialEq for Grammar {
    fn eq(&self, other: &Self) -> bool {
        self.productions == other.productions && self.lexical_rules == other.lexical_rules
    }
}

impl Grammar {
    pub fn count_nonterminals(&self) -> usize {
        self.productions.len()
    }
    /// Returns `u16::MAX` for derived rules, so callers can sort directly without
    /// special-casing missing entries.
    pub fn source_index(&self, name: &str) -> u16 {
        self.source_order.get(name).copied().unwrap_or(u16::MAX)
    }
    pub fn nonterminals(&self) -> impl Iterator<Item = &'_ Nonterminal> {
        self.productions.keys()
    }
    pub fn nonterminal(&self, name: &str) -> Option<&Nonterminal> {
        self.productions.keys().find(|n| n.name == name)
    }
    pub fn is_start(&self, nonterminal: &Nonterminal) -> bool {
        self.start_wrapper_names.contains(&nonterminal.name)
    }
    /// Returns the associated start nonterminal for the given nonterminal, if it exists.
    pub fn start_nonterminal(&self, nonterminal: &Nonterminal) -> Option<&Nonterminal> {
        let wrapper_name = self.start_nonterminals.get(&nonterminal.name)?;
        self.nonterminal(wrapper_name)
    }
    pub fn alternatives(&self, nonterminal: &Nonterminal) -> &[Alternative] {
        self.productions.get(nonterminal).map_or(&[], |v| v)
    }
    pub fn terminals(&self) -> impl Iterator<Item = &'_ Terminal> {
        self.lexical_rules.keys()
    }
    pub fn terminal(&self, name: &str) -> Option<&Terminal> {
        self.lexical_rules.keys().find(|t| t.name == name)
    }
    pub fn lexical_rule(&self, terminal: &Terminal) -> Option<&LexicalRule> {
        self.lexical_rules.get(terminal)
    }
    /// The terminal an except operand refers to, with its lexical rule.
    /// Panics when the except is not a terminal or has restrictions of its
    /// own: an except contributes only its language, so restrictions on it
    /// have no meaning.
    pub fn except_terminal(&self, except: &Identifier) -> (&Terminal, &LexicalRule) {
        let Definition::Terminal(terminal) = self.definition(except.resolve()) else {
            panic!("Except {} must refer to a terminal", except.name);
        };
        let rule = self
            .lexical_rule(terminal)
            .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name));
        assert!(
            rule.except.is_empty()
                && rule.follow_restriction.is_empty()
                && rule.precede_restriction.is_none(),
            "Except {} has restrictions of its own; only plain terminals can be excluded",
            terminal.name
        );
        (terminal, rule)
    }
    pub fn definition(&self, definition_id: DefinitionId) -> &Definition {
        &self.definitions[definition_id.0 as usize]
    }
    pub fn is_terminal(&self, ident: &Identifier) -> bool {
        matches!(
            ident.definition.map(|d| self.definition(d)),
            Some(Definition::Terminal(_))
        )
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "grammar {}\n", self.name)?;
        for (head, alternatives) in &self.productions {
            writeln!(f, "{}", head)?;
            if let Some((first, rest)) = alternatives.split_first() {
                writeln!(f, "  = {}", first)?;
                for alternative in rest {
                    writeln!(f, "  | {}", alternative)?;
                }
            }
            writeln!(f)?;
        }
        for (_, rule) in &self.lexical_rules {
            writeln!(f, "{}", rule)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! alternative {
    ($($symbol:expr),* $(,)?) => {
        $crate::grammar::def::Alternative {
            symbols: vec![$($symbol),*],
            label: None,
        }
    };
    ($($symbol:expr),+ $(,)? ; #$label:ident) => {
        $crate::grammar::def::Alternative {
            symbols: vec![$($symbol),+],
            label: Some(stringify!($label).to_string()),
        }
    };
}

#[macro_export]
macro_rules! left {
    () => {
        $crate::grammar::def::Associativity::Left
    };
}

#[macro_export]
macro_rules! right {
    () => {
        $crate::grammar::def::Associativity::Right
    };
}

#[macro_export]
macro_rules! non_assoc {
    () => {
        $crate::grammar::def::Associativity::NonAssoc
    };
}

#[macro_export]
macro_rules! priority_level {
    ($assoc:expr; $($alt:expr),* $(,)?) => {
        $crate::grammar::def::PriorityLevel {
            alternatives: vec![$($alt),*],
            associativity: Some($assoc),
        }
    };
    ($($alt:expr),* $(,)?) => {
        $crate::grammar::def::PriorityLevel {
            alternatives: vec![$($alt),*],
            associativity: None,
        }
    };
}

#[macro_export]
macro_rules! syntax_rule {
    ($head:literal ( $( $pname:literal : $pty:ident ),* $(,)? ) => $($level:expr),* $(,)?) => {
        $crate::grammar::def::SyntaxRule {
            head: $crate::grammar::symbols::Nonterminal::with_params(
                $head,
                vec![$($crate::grammar::symbols::Parameter {
                    name: $pname.into(),
                    ty: $crate::grammar::symbols::ParamType::$pty,
                }),*],
            ),
            priority_levels: vec![$($level.into()),*],
            layout: $crate::grammar::def::LayoutStrategy::Default,
        }
    };
    ($head:literal => $($level:expr),* $(,)?) => {
        $crate::grammar::def::SyntaxRule {
            head: $crate::grammar::symbols::Nonterminal::new($head),
            priority_levels: vec![$($level.into()),*],
            layout: $crate::grammar::def::LayoutStrategy::Default,
        }
    };
}

#[macro_export]
macro_rules! lexical_rule {
    ($head:literal => $regex:expr) => {
        $crate::grammar::def::LexicalRule::new(
            $crate::grammar::symbols::Terminal::new($head),
            $regex,
        )
    };
}

#[macro_export]
macro_rules! grammar_def {
    (
        $name:literal,
        syntax: [$($syntax:expr),* $(,)?]
        $(, lexical: [$($lexical:expr),* $(,)?])?
        , layout: $layout:expr
        $(,)?
    ) => {
        $crate::grammar::def::GrammarDef {
            name: $name.to_string(),
            syntax_rules: vec![$($syntax),*],
            lexical_rules: vec![$($($lexical),*)?],
            layout: Some($layout),
            identifier_rules: vec![],
        }
    };
    (
        $name:literal,
        syntax: [$($syntax:expr),* $(,)?]
        $(, lexical: [$($lexical:expr),* $(,)?])?
        $(,)?
    ) => {
        $crate::grammar::def::GrammarDef {
            name: $name.to_string(),
            syntax_rules: vec![$($syntax),*],
            lexical_rules: vec![$($($lexical),*)?],
            layout: None,
            identifier_rules: vec![],
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::regex::Regex;

    fn id(name: &str) -> Identifier {
        Identifier {
            name: name.into(),
            definition: None,
        }
    }

    fn lexical(name: &str, regex: Regex) -> LexicalRule {
        LexicalRule::new(Terminal::new(name), regex)
    }

    fn reference(name: &str) -> Regex {
        Regex::Identifier(id(name))
    }

    fn names(identifiers: &[Identifier]) -> Vec<&str> {
        identifiers.iter().map(|i| i.name.as_str()).collect()
    }

    fn find<'a>(rules: &'a [LexicalRule], name: &str) -> &'a LexicalRule {
        rules.iter().find(|r| r.head.name == name).unwrap()
    }

    /// A whole-body reference inherits the referenced rule's `\`, `!>>`, and
    /// `!<<`, following the chain past intermediate references, while the body
    /// itself inlines to the chain's base regex.
    #[test]
    fn whole_body_reference_inherits_restrictions() {
        let letters = lexical("Letters", Regex::plus(Regex::range('a', 'z')));
        let mut identifier = lexical("Identifier", reference("Letters"));
        identifier.except = vec![id("Keyword")];
        identifier.follow_restriction = vec![id("Digit")];
        identifier.precede_restriction = Some(id("JavaLetter"));
        let mut type_id = lexical("TypeId", reference("Identifier"));
        type_id.except = vec![id("Var")];

        let rules = inline_regex_refs(vec![letters, identifier, type_id]).unwrap();

        let type_id = find(&rules, "TypeId");
        assert_eq!(names(&type_id.except), ["Var", "Keyword"]);
        assert_eq!(names(&type_id.follow_restriction), ["Digit"]);
        assert_eq!(
            type_id.precede_restriction.as_ref().unwrap().name,
            "JavaLetter"
        );
        assert_eq!(type_id.regex, Regex::plus(Regex::range('a', 'z')));
        // The referenced rule keeps exactly its own restrictions.
        let identifier = find(&rules, "Identifier");
        assert_eq!(names(&identifier.except), ["Keyword"]);
    }

    /// A restricted reference buried in a larger regex can't carry its
    /// restrictions to the whole rule, so it is an error rather than a drop.
    #[test]
    fn restricted_reference_in_a_sequence_is_rejected() {
        let mut identifier = lexical("Identifier", Regex::plus(Regex::range('a', 'z')));
        identifier.except = vec![id("Keyword")];
        let bar = lexical("Bar", Regex::plus(Regex::range('0', '9')));
        let foo = lexical(
            "Foo",
            Regex::seq(vec![reference("Identifier"), reference("Bar")]),
        );

        let errors = inline_regex_refs(vec![identifier, bar, foo]).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("`Foo` references `Identifier` inside a larger regex"));
    }

    /// Two different precede restrictions stacked along a whole-body chain
    /// can't share one rule.
    #[test]
    fn conflicting_precede_restrictions_are_rejected() {
        let letters = lexical("Letters", Regex::plus(Regex::range('a', 'z')));
        let mut identifier = lexical("Identifier", reference("Letters"));
        identifier.precede_restriction = Some(id("Bang"));
        let mut foo = lexical("Foo", reference("Identifier"));
        foo.precede_restriction = Some(id("Dot"));

        let errors = inline_regex_refs(vec![letters, identifier, foo]).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("stacks precede restrictions"));
    }
}
