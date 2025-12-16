use std::fmt::Display;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum Symbol {
    Terminal(Terminal),
    Nonterminal(Nonterminal),
    Group(Vec<Symbol>),
    Opt(Box<Symbol>),
    Alt(Vec<Symbol>),
    Star(Box<Symbol>),
    Plus(Box<Symbol>),
}

impl Symbol {
    pub fn literal(name: &str) -> Self {
        Symbol::Terminal(Terminal::literal(name))
    }
    pub fn nonterminal(name: &str) -> Self {
        Symbol::Nonterminal(Nonterminal::new(name))
    }
    pub fn terminal(name: &str) -> Self {
        Symbol::Terminal(Terminal::identifier(name))
    }
    pub fn plus(symbol: Symbol) -> Self {
        Symbol::Plus(Box::new(symbol))
    }
    pub fn star(symbol: Symbol) -> Self {
        Symbol::Star(Box::new(symbol))
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Terminal(terminal) => write!(f, "{terminal}"),
            Symbol::Nonterminal(nonterminal) => write!(f, "{nonterminal}"),
            Symbol::Group(symbols) => write!(f, "({})", symbols.iter().join(" ")),
            Symbol::Opt(opt) => write!(f, "{opt}?"),
            Symbol::Alt(symbols) => write!(f, "({})", symbols.iter().join(" | ")),
            Symbol::Star(symbol) => write!(f, "{symbol}*"),
            Symbol::Plus(symbol) => write!(f, "{symbol}*"),
        }
    }
}

/// A terminal represents a reference to a lexical rule.
/// In the grammar specification, there are two cases where terminals can appear:
/// - As identifiers in grammar rules, e.g., `identifier`
/// - As string literals, e.g., `"+"`, `"if"`, `"while"`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Terminal {
    pub name: String,
    pub kind: TerminalKind,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TerminalKind {
    Literal,
    Identifier,
}

impl Terminal {
    pub fn literal(lit: &str) -> Self {
        Self {
            name: lit.into(),
            kind: TerminalKind::Literal,
        }
    }

    pub fn identifier(name: &str) -> Self {
        Self {
            name: name.into(),
            kind: TerminalKind::Identifier,
        }
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            TerminalKind::Literal => write!(f, "\"{}\"", self.name),
            TerminalKind::Identifier => write!(f, "{}", self.name),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Nonterminal {
    pub name: String,
}

impl Nonterminal {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Display for Nonterminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct Opt {
    symbol: Symbol,
}

impl Display for Opt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?", self.symbol)
    }
}
