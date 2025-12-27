use core::hash;
use std::{fmt::Display, hash::Hasher};

use itertools::Itertools;
use quote::quote;

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
pub enum NonterminalNodeKind {
    Simple,
    Star,
    Plus,
    Opt,
    Group,
    Alt,
}

impl quote::ToTokens for NonterminalNodeKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = match self {
            Self::Simple => quote!(Simple),
            Self::Star => quote!(Star),
            Self::Plus => quote!(Plus),
            Self::Opt => quote!(Opt),
            Self::Group => quote!(Group),
            Self::Alt => quote!(Alt),
        };
        tokens.extend(quote!(NonterminalNodeKind::#variant));
    }
}

/// The `name` uniquely identifies the nonterminal in the grammar.
/// `kind` has information on how this nonterminal was derived (e.g., from EBNF transformations).
#[derive(Debug, Clone)]
pub struct Nonterminal {
    pub name: String,
    pub kind: NonterminalNodeKind,
}

impl Nonterminal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: NonterminalNodeKind::Simple,
        }
    }

    pub fn with_kind(name: impl Into<String>, kind: NonterminalNodeKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }
}

impl hash::Hash for Nonterminal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Nonterminal {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Nonterminal {}

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
