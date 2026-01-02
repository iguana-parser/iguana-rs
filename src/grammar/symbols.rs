use core::hash;
use std::{fmt::Display, hash::Hasher};

use itertools::Itertools;
use quote::quote;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone)]
pub enum Definition {
    Terminal(Terminal),
    Nonterminal(Nonterminal),
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Identifier(String),
    Literal(String),
    Group(Vec<Symbol>),
    Opt(Box<Symbol>),
    Alt(Vec<Symbol>),
    Star(Box<Symbol>),
    Plus(Box<Symbol>),
}

impl Symbol {
    pub fn literal(name: impl Into<String>) -> Self {
        Symbol::Literal(name.into())
    }
    pub fn identifier(name: impl Into<String>) -> Self {
        Symbol::Identifier(name.into())
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Literal(literal) => write!(f, "\"{literal}\""),
            Symbol::Identifier(name) => write!(f, "{name}"),
            Symbol::Group(symbols) => write!(f, "({})", symbols.iter().join(" ")),
            Symbol::Opt(opt) => write!(f, "{opt}?"),
            Symbol::Alt(symbols) => write!(f, "({})", symbols.iter().join(" | ")),
            Symbol::Star(symbol) => write!(f, "{symbol}*"),
            Symbol::Plus(symbol) => write!(f, "{symbol}*"),
        }
    }
}

/// A terminal represents a lexical definition.
/// A terminal can be a literal, representing string literals in the grammar,
/// e.g., `"+"`, `"if"`, `"while"`, or it can be Regex, which is referred to
/// by a name in the grammar, e.g., `identifier`, `number`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Terminal {
    pub name: String,
    pub kind: TerminalKind,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum TerminalKind {
    Literal,
    Regex,
}

impl Terminal {
    pub fn with_kind(name: impl Into<String>, kind: TerminalKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            TerminalKind::Literal => write!(f, "\"{}\"", self.name),
            TerminalKind::Regex => write!(f, "{}", self.name),
        }
    }
}

impl quote::ToTokens for TerminalKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = match self {
            Self::Literal => quote!(Literal),
            Self::Regex => quote!(Regex),
        };
        tokens.extend(quote!(TerminalKind::#variant));
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Type)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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

#[macro_export]
macro_rules! id {
    ($name:expr) => {
        $crate::grammar::symbols::Symbol::identifier($name)
    };
}

#[macro_export]
macro_rules! lit {
    ($name:literal) => {
        $crate::grammar::symbols::Symbol::literal($name)
    };
}

#[macro_export]
macro_rules! alt {
    ($($symbol:expr),* $(,)?) => {
        $crate::grammar::symbols::Symbol::Alt(vec![$($symbol),*])
    };
}

#[macro_export]
macro_rules! group {
    ($($symbol:expr),* $(,)?) => {
        $crate::grammar::symbols::Symbol::Group(vec![$($symbol),*])
    };
}

#[macro_export]
macro_rules! plus {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Plus(Box::new($symbol))
    };
}

#[macro_export]
macro_rules! star {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Star(Box::new($symbol))
    };
}

#[macro_export]
macro_rules! opt {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Opt(Box::new($symbol))
    };
}
