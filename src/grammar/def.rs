use std::fmt::Display;

use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::FxHashMap;

use crate::grammar::{
    regex::Regex,
    symbols::{Definition, DefinitionId, Identifier, Nonterminal, Symbol, Terminal},
    transformations::{ebnf_to_bnf, transform_rule},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alternative {
    pub symbols: Vec<Symbol>,
    pub label: Option<String>,
}

impl Alternative {
    pub fn new(symbols: Vec<Symbol>) -> Self {
        Self {
            symbols,
            label: None,
        }
    }
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
}

impl Display for Alternative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.symbols.iter().join(" "))
    }
}

#[derive(Debug)]
pub struct SyntaxRule {
    pub head: Nonterminal,
    pub priority_levels: Vec<PriorityLevel>,
}

impl SyntaxRule {
    pub fn new(head: Nonterminal, priority_levels: Vec<PriorityLevel>) -> Self {
        Self {
            head,
            priority_levels,
        }
    }
}

#[derive(Debug)]
pub struct PriorityLevel {
    pub alternatives: Vec<Alternative>,
}

impl PriorityLevel {
    pub fn new(alternatives: Vec<Alternative>) -> Self {
        Self { alternatives }
    }
}

impl From<Alternative> for PriorityLevel {
    fn from(alt: Alternative) -> Self {
        Self {
            alternatives: vec![alt],
        }
    }
}

#[derive(Debug)]
pub struct LexicalRule {
    pub head: Terminal,
    pub regex: Regex,
}

pub struct GrammarDef {
    pub name: String,
    pub syntax_rules: Vec<SyntaxRule>,
    pub lexical_rules: Vec<LexicalRule>,
    // Whitespace and comment nodes
    pub layout_def: Vec<Terminal>,
}

impl Display for SyntaxRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.head)?;
        if let Some((first_level, rest_levels)) = self.priority_levels.split_first() {
            if let Some((first_alt, rest_alts)) = first_level.alternatives.split_first() {
                write!(f, "  : {}", first_alt.symbols.iter().join(" "))?;
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
                if rest_levels.is_empty() {
                    writeln!(f, "  ;")?;
                }
            }
            for (level_idx, level) in rest_levels.iter().enumerate() {
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
                    if level_idx == rest_levels.len() - 1 {
                        writeln!(f, "  ;")?;
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
            writeln!(f, "{}: {}", lexical_rule.head, lexical_rule.regex)?;
        }
        if !self.layout_def.is_empty() {
            writeln!(f, "\nlayout: {}", self.layout_def.iter().join(", "))?;
        }
        Ok(())
    }
}

/// Creates a map from names to definitions (terminal or nonterminal).
fn create_symbol_table<'a>(
    syntax_rules: &[SyntaxRule],
    terminals: impl Iterator<Item = &'a Terminal>,
    definitions: &mut Vec<Definition>,
) -> FxHashMap<String, DefinitionId> {
    let mut symbol_table = FxHashMap::default();
    for terminal in terminals {
        symbol_table.insert(
            terminal.name.clone(),
            DefinitionId(definitions.len() as u16),
        );
        definitions.push(Definition::Terminal(terminal.clone()));
    }
    for syntax_rule in syntax_rules {
        symbol_table.insert(
            syntax_rule.head.name.clone(),
            DefinitionId(definitions.len() as u16),
        );
        definitions.push(Definition::Nonterminal(syntax_rule.head.clone()));
    }
    symbol_table
}

/// Converts string literals (e.g., `"+"`) in syntax rules into terminal references
/// and generates corresponding lexical rules. Specifically:
/// 1. Converts each `Symbol::Literal` into a `Symbol::Identifier` referencing a terminal
/// 2. Creates a lexical rule that matches the literal string exactly
///
/// The name of the terminal is the same as the string literal.
fn add_lexical_rules_for_literals(
    syntax_rules: Vec<SyntaxRule>,
    lexical_rules: &mut IndexMap<Terminal, Regex>,
) -> Vec<SyntaxRule> {
    let mut rules = vec![];
    for rule in syntax_rules {
        let transformed = transform_rule(rule, |s| {
            if let Symbol::Literal(name) = s {
                let terminal_name = format!("\"{}\"", name);
                let terminal = Terminal::new(terminal_name.clone());
                if !lexical_rules.contains_key(&terminal) {
                    lexical_rules.insert(terminal, Regex::literal(&name));
                }
                Symbol::identifier(terminal_name)
            } else {
                s
            }
        });
        rules.push(transformed);
    }
    rules
}

fn resolve_identifiers(
    syntax_rules: Vec<SyntaxRule>,
    symbol_table: &FxHashMap<String, DefinitionId>,
) -> Vec<SyntaxRule> {
    let mut rules = vec![];
    for rule in syntax_rules {
        let transformed = transform_rule(rule, |s| resolve_identifier(s, symbol_table));
        rules.push(transformed);
    }
    rules
}

fn resolve_identifier(symbol: Symbol, symbol_table: &FxHashMap<String, DefinitionId>) -> Symbol {
    match symbol {
        Symbol::Identifier(identifier) => {
            if let Some(definition_id) = symbol_table.get(&identifier.name) {
                Symbol::Identifier(Identifier {
                    name: identifier.name,
                    definition: Some(*definition_id),
                })
            } else {
                panic!("Definition {} not found", &identifier.name)
            }
        }
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
        Symbol::Star(symbol) => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            Symbol::Star(Box::new(resolved_symbol))
        }
        Symbol::Plus(symbol) => {
            let resolved_symbol = resolve_identifier(*symbol, symbol_table);
            Symbol::Plus(Box::new(resolved_symbol))
        }
        _ => symbol,
    }
}

impl From<GrammarDef> for Grammar {
    fn from(grammar_def: GrammarDef) -> Self {
        let lexical_rules = grammar_def.lexical_rules;
        let syntax_rules = grammar_def.syntax_rules;
        let mut lexical_rules_map: IndexMap<Terminal, Regex> = lexical_rules
            .into_iter()
            .map(|r| (r.head, r.regex))
            .collect();
        let syntax_rules = add_lexical_rules_for_literals(syntax_rules, &mut lexical_rules_map);
        let mut definitions = vec![];
        let symbol_table =
            create_symbol_table(&syntax_rules, lexical_rules_map.keys(), &mut definitions);
        let syntax_rules = resolve_identifiers(syntax_rules, &symbol_table);
        let (syntax_rules, ebnf_symbols) = ebnf_to_bnf::ebnf_to_bnf(syntax_rules);
        let symbol_table =
            create_symbol_table(&syntax_rules, lexical_rules_map.keys(), &mut definitions);
        let syntax_rules = resolve_identifiers(syntax_rules, &symbol_table);
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
        Self {
            name: grammar_def.name,
            productions,
            lexical_rules: lexical_rules_map,
            definitions,
            ebnf_symbols,
            layout_defs: grammar_def.layout_def,
        }
    }
}

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    productions: IndexMap<Nonterminal, Vec<Alternative>>,
    lexical_rules: IndexMap<Terminal, Regex>,
    definitions: Vec<Definition>,
    ebnf_symbols: FxHashMap<Symbol, Symbol>,
    pub layout_defs: Vec<Terminal>,
}

impl Grammar {
    pub fn count_nonterminals(&self) -> usize {
        self.productions.len()
    }
    pub fn nonterminals(&self) -> impl Iterator<Item = &'_ Nonterminal> {
        self.productions.keys()
    }
    pub fn alternatives(&self, nonterminal: &Nonterminal) -> &[Alternative] {
        self.productions.get(nonterminal).map_or(&[], |v| v)
    }
    pub fn terminals(&self) -> impl Iterator<Item = &'_ Terminal> {
        self.lexical_rules.keys()
    }
    pub fn lexical_rules(&self, terminal: &Terminal) -> Option<&Regex> {
        self.lexical_rules.get(terminal)
    }
    pub fn definition(&self, definition_id: DefinitionId) -> &Definition {
        &self.definitions[definition_id.0 as usize]
    }
    pub fn ebnf_symbol(&self, symbol: &Symbol) -> Option<&Symbol> {
        self.ebnf_symbols.get(symbol)
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "grammar {}\n", self.name)?;
        for (head, alternatives) in &self.productions {
            writeln!(f, "{}", head)?;
            if let Some((first, rest)) = alternatives.split_first() {
                writeln!(f, "  : {}", first)?;
                for alternative in rest {
                    writeln!(f, "  | {}", alternative)?;
                }
            }
        }
        for (name, regex) in &self.lexical_rules {
            writeln!(f, "{}: {}", name, regex)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! alternative {
    ($($symbol:expr),* $(,)?, @$label:literal) => {
        $crate::grammar::Alternative {
            symbols: vec![$($symbol),*],
            label: Some($label.to_string()),
        }
    };
    ($($symbol:expr),* $(,)?) => {
        $crate::grammar::def::Alternative {
            symbols: vec![$($symbol),*],
            label: None,
        }
    };
}

#[macro_export]
macro_rules! priority_level {
    ($($alt:expr),* $(,)?) => {
        $crate::grammar::def::PriorityLevel {
            alternatives: vec![$($alt),*],
        }
    };
}

#[macro_export]
macro_rules! syntax_rule {
    ($head:literal => $($level:expr),* $(,)?) => {
        $crate::grammar::def::SyntaxRule {
            head: $crate::grammar::symbols::Nonterminal::new($head),
            priority_levels: vec![$($level.into()),*],
        }
    };
}

#[macro_export]
macro_rules! lexical_rule {
    ($head:literal => $regex:expr) => {
        $crate::grammar::def::LexicalRule {
            head: $crate::grammar::symbols::Terminal::new($head),
            regex: $regex,
        }
    };
}

#[macro_export]
macro_rules! grammar_def {
    (
        $name:literal,
        syntax: [$($syntax:expr),* $(,)?]
        $(, lexical: [$($lexical:expr),* $(,)?])?
        $(, layout: [$($layout:expr),* $(,)?])?
        $(,)?
    ) => {
        $crate::grammar::def::GrammarDef {
            name: $name.to_string(),
            syntax_rules: vec![$($syntax),*],
            lexical_rules: vec![$($($lexical),*)?],
            layout_def: vec![$($($layout),*)?],
        }
    };
}
