use std::fmt::Display;

use typed_builder::TypedBuilder;

#[derive(Debug, Clone)]
pub enum Symbol {
    Terminal(Terminal),
    Nonterminal(Nonterminal),
    Seq(Seq),
    Opt(Box<Opt>),
    Alt(Seq),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Terminal(terminal) => write!(f, "{terminal}"),
            Symbol::Nonterminal(nonterminal) => write!(f, "{nonterminal}"),
            Symbol::Seq(seq) => write!(f, "{seq}"),
            Symbol::Opt(opt) => write!(f, "{opt}"),
            Symbol::Alt(_) => todo!(),
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

#[derive(Debug, TypedBuilder, Clone)]
#[builder(mutators(
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }
))]
pub struct Seq {
    #[builder(via_mutators)]
    pub symbols: Vec<Symbol>,
}

impl Seq {
    pub fn len(&self) -> usize {
        self.symbols.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Display for Seq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seq_to_string = self
            .symbols
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{seq_to_string}")
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
