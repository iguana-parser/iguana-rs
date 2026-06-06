use std::fmt::Display;

use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    alternative,
    grammar::{
        regex::Regex,
        symbols::{Definition, DefinitionId, Expr, Identifier, Nonterminal, Symbol, Terminal},
        transformations::{
            ebnf_to_bnf, exclude_desugaring, layout_insertion, precedence_desugaring,
            transform_regex, transform_syntax_rule, visit_syntax_rule,
        },
    },
    priority_level,
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
    pub start: bool,
}

impl SyntaxRule {
    pub fn new(head: Nonterminal, priority_levels: Vec<PriorityLevel>) -> Self {
        Self {
            head,
            priority_levels,
            layout: LayoutStrategy::Default,
            start: false,
        }
    }

    /// Every alternative across all priority levels, in order.
    pub fn alternatives(&self) -> impl Iterator<Item = &Alternative> {
        self.priority_levels
            .iter()
            .flat_map(|level| &level.alternatives)
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
    pub follow_restriction: Option<Identifier>,
    pub precede_restriction: Option<Identifier>,
}

impl LexicalRule {
    pub fn new(head: Terminal, regex: Regex) -> Self {
        Self {
            head,
            regex,
            except: vec![],
            follow_restriction: None,
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
        if let Some(restriction) = &self.follow_restriction {
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
}

impl GrammarDef {
    pub fn resolve(self) -> GrammarDef {
        let (_, symbol_table) = create_symbol_table(&self.syntax_rules, &self.lexical_rules);
        let (syntax_rules, lexical_rules) =
            resolve_identifiers(self.syntax_rules, self.lexical_rules, &symbol_table);
        GrammarDef {
            name: self.name,
            syntax_rules,
            lexical_rules,
            layout: self.layout,
        }
    }

    pub fn for_each_symbol(&self, f: &mut impl FnMut(&Symbol)) {
        for rule in &self.syntax_rules {
            visit_syntax_rule(rule, f);
        }
    }

    pub fn for_each_identifier(&self, f: &mut impl FnMut(&Identifier)) {
        self.for_each_symbol(&mut |symbol| match symbol {
            Symbol::Identifier(id) | Symbol::Call { name: id, .. } => f(id),
            Symbol::Except { except, .. } => except.iter().for_each(&mut *f),
            Symbol::FollowRestriction { restrictions, .. } => restrictions.iter().for_each(&mut *f),
            Symbol::PrecedeRestriction { restriction, .. } => f(restriction),
            _ => {}
        });
        for rule in &self.lexical_rules {
            visit_regex_identifiers(&rule.regex, f);
        }
    }

    pub fn unresolved_identifiers(&self) -> Vec<String> {
        let mut unresolved = Vec::new();
        self.for_each_identifier(&mut |id| {
            if id.definition.is_none() {
                unresolved.push(id.name.clone());
            }
        });
        unresolved
    }

    /// Returns labels in `Exclude` symbols (`A!label`) that don't match any
    /// alternative label on the referenced nonterminal.
    pub fn unresolved_labels(&self) -> Vec<String> {
        let rules_by_name: FxHashMap<&str, &SyntaxRule> = self
            .syntax_rules
            .iter()
            .map(|r| (r.head.name.as_str(), r))
            .collect();

        let mut unresolved = Vec::new();
        self.for_each_symbol(&mut |symbol| {
            if let Symbol::Exclude {
                symbol: inner,
                labels,
            } = symbol
            {
                if let Some(id) = inner.as_identifier() {
                    if let Some(rule) = rules_by_name.get(id.name.as_str()) {
                        let valid_labels: Vec<&str> = rule
                            .alternatives()
                            .filter_map(|alt| alt.label.as_deref())
                            .collect();

                        for label in labels {
                            if !valid_labels.contains(&label.as_str()) {
                                unresolved.push(label.clone());
                            }
                        }
                    }
                }
            }
        });
        unresolved
    }

    /// Returns names that are defined more than once across syntax and lexical rules.
    pub fn duplicate_definitions(&self) -> Vec<String> {
        let mut seen = FxHashSet::default();
        let mut duplicates = Vec::new();
        for rule in &self.lexical_rules {
            if !seen.insert(&rule.head.name) {
                duplicates.push(rule.head.name.clone());
            }
        }
        for rule in &self.syntax_rules {
            if !seen.insert(&rule.head.name) {
                duplicates.push(rule.head.name.clone());
            }
        }
        duplicates
    }
}

impl TryFrom<GrammarDef> for Grammar {
    type Error = Vec<String>;

    /// Resolves identifiers, checks for unresolved references, then runs the
    /// full transformation pipeline (EBNF-to-BNF, precedence desugaring,
    /// layout insertion, etc.).
    fn try_from(grammar_def: GrammarDef) -> Result<Self, Self::Error> {
        let resolved = grammar_def.resolve();
        let mut errors = resolved.duplicate_definitions();
        errors.extend(resolved.unresolved_identifiers());
        errors.extend(resolved.unresolved_labels());
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(build_grammar(resolved))
    }
}

fn visit_regex_identifiers(regex: &Regex, f: &mut impl FnMut(&Identifier)) {
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
        match &self.layout {
            LayoutStrategy::None => writeln!(f, "@NoLayout")?,
            LayoutStrategy::Custom(id) => writeln!(f, "@Layout({})", id.name)?,
            LayoutStrategy::Default => {}
        }
        writeln!(f, "{}", self.head)?;
        if let Some((first_level, rest_levels)) = self.priority_levels.split_first() {
            if let Some((first_alt, rest_alts)) = first_level.alternatives.split_first() {
                write!(f, "  = {}", first_alt.symbols.iter().join(" "))?;
                if let Some(label) = &first_alt.label {
                    write!(f, " #{}", label)?;
                }
                writeln!(f)?;
                for alternative in rest_alts {
                    write!(f, "  | {}", alternative.symbols.iter().join(" "))?;
                    if let Some(label) = &alternative.label {
                        write!(f, " #{}", label)?;
                    }
                    writeln!(f)?;
                }
            }
            for level in rest_levels.iter() {
                writeln!(f, "  >")?;
                if let Some((first_alt, rest_alts)) = level.alternatives.split_first() {
                    write!(f, "    {}", first_alt.symbols.iter().join(" "))?;
                    if let Some(label) = &first_alt.label {
                        write!(f, " #{}", label)?;
                    }
                    writeln!(f)?;
                    for alternative in rest_alts {
                        write!(f, "  | {}", alternative.symbols.iter().join(" "))?;
                        if let Some(label) = &alternative.label {
                            write!(f, " #{}", label)?;
                        }
                        writeln!(f)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Display for GrammarDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "grammar {}\n", self.name)?;
        for rule in &self.syntax_rules {
            writeln!(f, "{}", rule)?;
        }
        for lexical_rule in &self.lexical_rules {
            writeln!(f, "{} = {}", lexical_rule.head, lexical_rule.regex)?;
        }
        if let Some(layout) = &self.layout {
            writeln!(f, "\nlayout = {}", layout)?;
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
            let terminal_name = format!("\"{}\"", name);
            let terminal = Terminal::new(terminal_name.clone());
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
        Symbol::Except { symbol, except } => {
            let transformed = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            Symbol::Except {
                symbol: Box::new(transformed),
                except,
            }
        }
        Symbol::FollowRestriction {
            symbol,
            restrictions,
        } => {
            let transformed = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            Symbol::FollowRestriction {
                symbol: Box::new(transformed),
                restrictions,
            }
        }
        Symbol::PrecedeRestriction {
            symbol,
            restriction,
        } => {
            let transformed = add_lexical_rules(*symbol, lexical_rules, added_terminals);
            Symbol::PrecedeRestriction {
                symbol: Box::new(transformed),
                restriction,
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
        Symbol::Except { symbol, except } => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            let resolved_except = except
                .into_iter()
                .map(|e| Identifier {
                    definition: symbol_table.get(&e.name),
                    name: e.name,
                })
                .collect();
            Symbol::Except {
                symbol: Box::new(resolved_symbol),
                except: resolved_except,
            }
        }
        Symbol::FollowRestriction {
            symbol,
            restrictions,
        } => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            let resolved_restrictions = restrictions
                .into_iter()
                .map(|r| Identifier {
                    definition: symbol_table.get(&r.name),
                    name: r.name,
                })
                .collect();
            Symbol::FollowRestriction {
                symbol: Box::new(resolved_symbol),
                restrictions: resolved_restrictions,
            }
        }
        Symbol::PrecedeRestriction {
            symbol,
            restriction,
        } => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            let resolved_restriction = Identifier {
                definition: symbol_table.get(&restriction.name),
                name: restriction.name,
            };
            Symbol::PrecedeRestriction {
                symbol: Box::new(resolved_symbol),
                restriction: resolved_restriction,
            }
        }
        Symbol::Exclude { symbol, labels } => Symbol::Exclude {
            symbol: Box::new(resolve_identifier(*symbol, symbol_table)),
            labels,
        },
        Symbol::Literal(_) | Symbol::Condition(_) | Symbol::Return(_) => symbol,
    }
}

/// Inlines `Regex::Identifier` references in lexical rules by substituting them with the
/// referenced rule's regex body. For example, given:
///   Digit = [0-9]
///   Digits = Digit+
/// After inlining, `Digits` becomes `[0-9]+`.
///
/// Uses a single map (`inlined_regexes`) with `Option<Regex>` values to track resolution state:
/// - `None` → resolution is in progress (on the current recursion stack)
/// - `Some(regex)` → fully resolved and cached
///
/// If we encounter a name mapped to `None`, it means we have a cyclic reference
/// (e.g., `A = B`, `B = A`), which is a grammar error.
// TODO: return a proper Result instead of panicking on errors (cyclic/undefined references).
fn inline_regex_refs(lexical_rules: Vec<LexicalRule>) -> Vec<LexicalRule> {
    // Maps each lexical rule name to its original (uninlined) regex body.
    let regex_map: FxHashMap<String, Regex> = lexical_rules
        .iter()
        .map(|r| (r.head.name.clone(), r.regex.clone()))
        .collect();
    let mut inlined_regexes: FxHashMap<String, Option<Regex>> = FxHashMap::default();
    lexical_rules
        .into_iter()
        .map(|mut rule| {
            rule.regex = inline_regex(rule.regex, &regex_map, &mut inlined_regexes);
            rule
        })
        .collect()
}

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

/// Runs the full transformation pipeline on a resolved GrammarDef: adds
/// literal terminals, inlines regex references, desugars EBNF/exclude/
/// precedence, inserts layout, and assembles the final Grammar.
fn build_grammar(grammar_def: GrammarDef) -> Grammar {
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
    let lexical_rules = inline_regex_refs(lexical_rules);
    let syntax_rules = ebnf_to_bnf::transform(syntax_rules);
    let (_, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
    let (syntax_rules, lexical_rules) =
        resolve_identifiers(syntax_rules, lexical_rules, &symbol_table);
    let syntax_rules = exclude_desugaring::transform(syntax_rules);
    let (_, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
    let (syntax_rules, lexical_rules) =
        resolve_identifiers(syntax_rules, lexical_rules, &symbol_table);
    let syntax_rules = precedence_desugaring::transform(syntax_rules);
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
            if let Some(restriction) = &mut r.follow_restriction {
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

    let (syntax_rules, layout, start_nonterminals, start_wrapper_names) =
        if let Some(layout) = &grammar_def.layout {
            let resolved = resolve_identifier(layout.clone(), &symbol_table);
            let mut syntax_rules = layout_insertion::transform(syntax_rules, &resolved);
            let start_rules: Vec<_> = syntax_rules
                .iter()
                .filter(|r| r.start)
                .map(|r| add_start_rule(&r.head, &resolved, &symbol_table))
                .collect();
            let start_names: FxHashMap<String, String> = syntax_rules
                .iter()
                .filter(|r| r.start)
                .map(|r| (r.head.name.clone(), format!("Start{}", r.head.name)))
                .collect();
            let start_wrapper_names: FxHashSet<String> = start_names.values().cloned().collect();
            syntax_rules.extend(start_rules);
            (
                syntax_rules,
                Some(resolved),
                start_names,
                start_wrapper_names,
            )
        } else {
            (
                syntax_rules,
                None,
                FxHashMap::default(),
                FxHashSet::default(),
            )
        };

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
    Grammar {
        name: grammar_def.name,
        productions,
        lexical_rules: lexical_rules_map,
        definitions,
        layout,
        symbol_table,
        start_nonterminals,
        start_wrapper_names,
        source_order,
    }
}

fn add_start_rule(
    nt: &Nonterminal,
    layout_identifier: &Symbol,
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

    SyntaxRule {
        head: Nonterminal {
            name,
            origin: Some(symbol.clone()),
            parameters: vec![],
        },
        priority_levels: vec![priority_level!(alternative!(
            layout_identifier.clone(),
            Symbol::Labeled {
                label: "start".into(),
                symbol: Box::new(symbol)
            },
            layout_identifier.clone()
        ))],
        layout: LayoutStrategy::Default,
        start: false,
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
            start: false,
        }
    };
    ($head:literal => $($level:expr),* $(,)?) => {
        $crate::grammar::def::SyntaxRule {
            head: $crate::grammar::symbols::Nonterminal::new($head),
            priority_levels: vec![$($level.into()),*],
            layout: $crate::grammar::def::LayoutStrategy::Default,
            start: false,
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
        }
    };
}
