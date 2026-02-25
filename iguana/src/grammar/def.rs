use std::fmt::Display;

use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    alternative,
    grammar::{
        regex::Regex,
        symbols::{Definition, DefinitionId, Expr, Identifier, Nonterminal, Symbol, Terminal},
        transformations::{ebnf_to_bnf, layout_insertion, precedence_desugaring, transform_rule},
    },
    lexical_rule, priority_level,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Associativity {
    Left,
    Right,
    NonAssoc,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alternative {
    pub symbols: Vec<Symbol>,
    pub label: Option<String>,
    pub associativity: Option<Associativity>,
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
            associativity: None,
        }
    }

    pub fn display_name(&self, grammar: &Grammar) -> String {
        let symbols: Vec<String> = self
            .symbols
            .iter()
            .map(|s| s.display_name(grammar))
            .collect();
        let mut result = symbols.join(" ");
        if let Some(assoc) = &self.associativity {
            let assoc_str = match assoc {
                Associativity::Left => "left",
                Associativity::Right => "right",
                Associativity::NonAssoc => "non_assoc",
            };
            result = format!("{} {}", result, assoc_str);
        }
        if let Some(label) = &self.label {
            result = format!("{} @{}", result, label);
        }
        result
    }
}

impl Display for Alternative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbols = self.symbols.iter().join(" ");
        write!(f, "{}", symbols)?;
        if let Some(assoc) = &self.associativity {
            let assoc_str = match assoc {
                Associativity::Left => "left",
                Associativity::Right => "right",
                Associativity::NonAssoc => "non_assoc",
            };
            write!(f, " {}", assoc_str)?;
        }
        if let Some(label) = &self.label {
            write!(f, " @{}", label)?;
        }
        Ok(())
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

#[derive(Debug)]
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
        if !self.layout_def.is_empty() {
            writeln!(f, "\nlayout = {}", self.layout_def.iter().join(", "))?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct SymbolTable {
    symbol_table: FxHashMap<String, DefinitionId>,
}

impl SymbolTable {
    pub fn insert(&mut self, name: String) -> DefinitionId {
        let def_id = DefinitionId(self.symbol_table.len() as u16);
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
        let transformed = transform_rule(rule, |s| {
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
                lexical_rules.push(LexicalRule {
                    head: terminal,
                    regex: Regex::literal(&name),
                });
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
        _ => symbol,
    }
}

fn resolve_identifiers(
    syntax_rules: Vec<SyntaxRule>,
    symbol_table: &SymbolTable,
) -> Vec<SyntaxRule> {
    let mut rules = vec![];
    for rule in syntax_rules {
        let transformed = transform_rule(rule, |s| resolve_identifier(s, symbol_table));
        rules.push(transformed);
    }
    rules
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
        Symbol::Call { name, arguments } => {
            if let Some(definition_id) = symbol_table.get(&name.name) {
                Symbol::Call {
                    name: Identifier {
                        name: name.name,
                        definition: Some(definition_id),
                    },
                    arguments,
                }
            } else {
                panic!("Definition {} not found", &name.name)
            }
        }
        Symbol::Identifier(identifier) => {
            if let Some(definition_id) = symbol_table.get(&identifier.name) {
                Symbol::Identifier(Identifier {
                    name: identifier.name,
                    definition: Some(definition_id),
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
            let resolved_except = if let Some(definition_id) = symbol_table.get(&except.name) {
                Identifier {
                    name: except.name,
                    definition: Some(definition_id),
                }
            } else {
                panic!("Definition {} not found", &except.name)
            };
            Symbol::Except {
                symbol: Box::new(resolved_symbol),
                except: resolved_except,
            }
        }
        Symbol::Literal(_) | Symbol::Condition(_) | Symbol::Return(_) => symbol,
    }
}

impl From<GrammarDef> for Grammar {
    fn from(grammar_def: GrammarDef) -> Self {
        let mut lexical_rules = grammar_def.lexical_rules;
        let syntax_rules = grammar_def.syntax_rules;
        let syntax_rules = add_lexical_rules_for_literals(syntax_rules, &mut lexical_rules);
        let (_, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
        let syntax_rules = resolve_identifiers(syntax_rules, &symbol_table);
        let (syntax_rules, mut ebnf_symbols) = ebnf_to_bnf::transform(syntax_rules);
        let (_, symbol_table) = create_symbol_table(&syntax_rules, &lexical_rules);
        let syntax_rules = resolve_identifiers(syntax_rules, &symbol_table);
        let syntax_rules = precedence_desugaring::transform(syntax_rules);
        // Create the final symbol table after all transformations. This must happen
        // after precedence desugaring because desugaring may add parameters to
        // nonterminals (e.g., E becomes E(p)), and the definitions must reflect that.
        let (mut definitions, mut symbol_table) =
            create_symbol_table(&syntax_rules, &lexical_rules);
        ebnf_symbols = ebnf_symbols
            .into_iter()
            .map(|(k, v)| (k, resolve_identifier(v, &symbol_table)))
            .collect();
        let mut lexical_rules_map: IndexMap<Terminal, Regex> = lexical_rules
            .into_iter()
            .map(|r| (r.head, r.regex))
            .collect();
        let layout_rule = layout_rule(&grammar_def.layout_def, &lexical_rules_map);
        let def_id = symbol_table.insert("Layout".into());
        let layout_identifier = Symbol::Identifier(Identifier {
            name: "Layout".into(),
            definition: Some(def_id),
        });
        let mut syntax_rules = layout_insertion::transform(syntax_rules, layout_identifier.clone());
        let layout_terminal = layout_rule.head;
        definitions.push(Definition::Terminal(layout_terminal.clone()));
        lexical_rules_map.insert(layout_terminal, layout_rule.regex);

        let start_rules: Vec<_> = syntax_rules
            .iter()
            .filter(|r| !r.head.is_derived())
            .map(|r| add_start_rule(&r.head, &layout_identifier, &symbol_table))
            .collect();
        syntax_rules.extend(start_rules);
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
            symbol_table,
        }
    }
}

// TODO: for now we only support regex layouts
fn layout_rule(
    layout_def: &[Terminal],
    lexical_rules_map: &IndexMap<Terminal, Regex>,
) -> LexicalRule {
    let layout_regex = match layout_def {
        [] => Regex::Epsilon,
        [single] => lexical_rules_map.get(single).unwrap().clone(),
        multiple => Regex::Alt(
            multiple
                .iter()
                .map(|def| {
                    lexical_rules_map
                        .get(def)
                        .expect("Layout should be defined")
                        .clone()
                })
                .collect(),
        ),
    };
    lexical_rule!("Layout" => layout_regex)
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
            origin: None,
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
    }
}

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    productions: IndexMap<Nonterminal, Vec<Alternative>>,
    lexical_rules: IndexMap<Terminal, Regex>,
    definitions: Vec<Definition>,
    ebnf_symbols: FxHashMap<Symbol, Symbol>,
    pub symbol_table: SymbolTable,
    pub layout_defs: Vec<Terminal>,
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
                if let Some((last, rest)) = rest.split_last() {
                    for alternative in rest {
                        writeln!(f, "  | {}", alternative)?;
                    }
                    writeln!(f, "  | {}", last)?;
                }
            }
            writeln!(f, "  ;\n")?;
        }
        for (name, regex) in &self.lexical_rules {
            writeln!(f, "{}: {}", name, regex)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! alternative {
    ($($symbol:expr),+; $assoc:expr) => {
        $crate::grammar::def::Alternative {
            symbols: vec![$($symbol),*],
            label: None,
            associativity: Some($assoc),
        }
    };
    ($($symbol:expr),* $(,)?) => {
        $crate::grammar::def::Alternative {
            symbols: vec![$($symbol),*],
            label: None,
            associativity: None,
        }
    };
}

#[macro_export]
macro_rules! left {
    () => { $crate::grammar::def::Associativity::Left };
}

#[macro_export]
macro_rules! right {
    () => { $crate::grammar::def::Associativity::Right };
}

#[macro_export]
macro_rules! non_assoc {
    () => { $crate::grammar::def::Associativity::NonAssoc };
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
        }
    };
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
