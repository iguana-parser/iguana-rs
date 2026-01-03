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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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

impl quote::ToTokens for Symbol {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ts = match self {
            Symbol::Identifier(s) => quote! { Symbol::Identifier(#s.to_string()) },
            Symbol::Literal(s) => quote! { Symbol::Literal(#s.to_string()) },
            Symbol::Group(syms) => quote! { Symbol::Group(vec![#(#syms),*]) },
            Symbol::Alt(syms) => quote! { Symbol::Alt(vec![#(#syms),*]) },
            Symbol::Opt(s) => quote! { Symbol::Opt(Box::new(#s)) },
            Symbol::Star(s) => quote! { Symbol::Star(Box::new(#s)) },
            Symbol::Plus(s) => quote! { Symbol::Plus(Box::new(#s)) },
        };
        tokens.extend(ts);
    }
}

/// A terminal represents a lexical definition.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Terminal {
    pub name: String,
}

impl Terminal {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// The `name` uniquely identifies the nonterminal in the grammar.
/// Origin tracks how the nonterminal is created, e.g., from EBNF to BNF conversion.
/// If origin is None, it's not a derived nonterminal
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Nonterminal {
    pub name: String,
    pub origin: Option<Symbol>,
}

impl Nonterminal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            origin: None,
        }
    }

    pub fn with_origin(name: impl Into<String>, origin: Symbol) -> Self {
        Self {
            name: name.into(),
            origin: Some(origin),
        }
    }

    pub fn is_derived(&self) -> bool {
        self.origin.is_some()
    }

    pub fn display_name(&self) -> String {
        match &self.origin {
            Some(symbol) => symbol.to_string(),
            None => self.name.clone(),
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
