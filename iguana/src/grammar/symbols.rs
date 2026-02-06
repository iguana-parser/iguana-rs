use core::hash;
use std::{fmt::Display, hash::Hasher};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Definition {
    Terminal(Terminal),
    Nonterminal(Nonterminal),
}

impl Definition {
    pub fn name(&self) -> &str {
        match self {
            Definition::Terminal(terminal) => &terminal.name,
            Definition::Nonterminal(nonterminal) => &nonterminal.name,
        }
    }
    pub fn display_name(&self) -> String {
        match self {
            Definition::Terminal(terminal) => terminal.name.clone(),
            Definition::Nonterminal(nonterminal) => nonterminal.display_name(),
        }
    }
    pub fn as_nonterminal(&self) -> &Nonterminal {
        match self {
            Definition::Terminal(_) => panic!(),
            Definition::Nonterminal(nonterminal) => nonterminal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    Labeled { label: String, symbol: Box<Symbol> },
    Identifier(Identifier),
    Literal(String),
    Group(Vec<Symbol>),
    Opt(Box<Symbol>),
    Alt(Vec<Symbol>),
    Star(Box<Symbol>, Option<Box<Symbol>>), // symbol, separator
    Plus(Box<Symbol>, Option<Box<Symbol>>), // symbol, separator
}

impl Symbol {
    pub fn literal(name: impl Into<String>) -> Self {
        Symbol::Literal(name.into())
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            Symbol::Labeled { label, .. } => Some(label),
            _ => None,
        }
    }

    pub fn resolved_def(&self) -> DefinitionId {
        let ident = match self {
            Symbol::Labeled { symbol, .. } => symbol.as_identifier(),
            Symbol::Identifier(name) => name,
            _ => panic!("Expected identifier, got {:?}", self),
        };
        ident.definition.expect("Symbol should be resolved")
    }

    pub fn as_identifier(&self) -> &Identifier {
        match self {
            Symbol::Identifier(identifier) => identifier,
            _ => panic!("Expected identifier but found {}", self),
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Labeled { label, symbol } => write!(f, "{label}:{symbol}"),
            Symbol::Literal(literal) => write!(f, "\"{literal}\""),
            Symbol::Identifier(identifier) => write!(f, "{}", identifier.name),
            Symbol::Group(symbols) => write!(f, "({})", symbols.iter().join(" ")),
            Symbol::Opt(opt) => write!(f, "{opt}?"),
            Symbol::Alt(symbols) => write!(f, "({})", symbols.iter().join(" | ")),
            Symbol::Star(symbol, sep) => match sep {
                Some(sep) => write!(f, "{{{symbol} {sep}}}*"),
                None => write!(f, "{symbol}*"),
            },
            Symbol::Plus(symbol, sep) => match sep {
                Some(sep) => write!(f, "{{{symbol} {sep}}}+"),
                None => write!(f, "{symbol}+"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefinitionId(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
    pub definition: Option<DefinitionId>,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name)
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
#[derive(Debug, Clone)]
pub struct Nonterminal {
    pub name: String,
    pub origin: Option<Symbol>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: ParamType,
}

#[derive(Debug, Clone)]
pub enum ParamType {
    U16,
}

impl Nonterminal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            origin: None,
            parameters: vec![],
        }
    }

    pub fn with_params(name: impl Into<String>, parameters: Vec<Parameter>) -> Self {
        Self {
            name: name.into(),
            origin: None,
            parameters,
        }
    }

    pub fn with_origin(name: impl Into<String>, origin: Symbol) -> Self {
        Self {
            name: name.into(),
            origin: Some(origin),
            parameters: vec![],
        }
    }
    // For normal nonterminals, i.e., the ones that are defined by the user directly,
    // the display_name is the same the nonterminal name.
    // For other nonterminals that are generated during grammar transformations,
    // `display_name` shows a name that reflects the structure, rather than the unique,
    // synthetic name using for the code generation.
    // For example, for the rule S : A (B|C)+ C, the display name is (B|C)+, while the
    // name is S_Plus_0.
    pub fn display_name(&self) -> String {
        match &self.origin {
            Some(symbol) => symbol.to_string(),
            None => self.name.clone(),
        }
    }

    /// Returns true if the nonterminal was generated when converting from an EBNF Plus.
    pub fn is_plus(&self) -> bool {
        match &self.origin {
            Some(s) => matches!(s, Symbol::Plus(_, _)),
            None => false,
        }
    }

    /// Returns true if the nonterminal was generated when converting from an EBNF Star.
    pub fn is_star(&self) -> bool {
        match &self.origin {
            Some(s) => matches!(s, Symbol::Star(_, _)),
            None => false,
        }
    }

    /// Returns true if the nonterminal was generated when converting from an EBNF Group.
    pub fn is_group(&self) -> bool {
        match &self.origin {
            Some(s) => matches!(s, Symbol::Group(_)),
            None => false,
        }
    }

    /// Returns true if the nonterminal is derived, e.g., from the EBNF to BNF conversion.
    pub fn is_derived(&self) -> bool {
        self.origin.is_some()
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
macro_rules! labeled {
    ($label:literal, $symbol:expr) => {
        $crate::grammar::symbols::Symbol::Labeled {
            label: $label.into(),
            symbol: Box::new($symbol),
        }
    };
}

#[macro_export]
macro_rules! id {
    ($name:expr) => {
        $crate::grammar::symbols::Symbol::Identifier($crate::grammar::symbols::Identifier {
            name: $name.into(),
            definition: None,
        })
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
        $crate::grammar::symbols::Symbol::Plus(Box::new($symbol), None)
    };
    ($symbol:expr, $sep:expr) => {
        $crate::grammar::symbols::Symbol::Plus(Box::new($symbol), Some(Box::new($sep)))
    };
}

#[macro_export]
macro_rules! star {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Star(Box::new($symbol), None)
    };
    ($symbol:expr, $sep:expr) => {
        $crate::grammar::symbols::Symbol::Star(Box::new($symbol), Some(Box::new($sep)))
    };
}

#[macro_export]
macro_rules! opt {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Opt(Box::new($symbol))
    };
}
