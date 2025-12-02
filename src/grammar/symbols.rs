use std::fmt::Display;

use indexmap::IndexMap;
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

#[derive(Debug, Clone)]
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

#[derive(Debug, TypedBuilder, Clone)]
#[builder(mutators(
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }
))]
pub struct Alternative {
    #[builder(via_mutators)]
    pub symbols: Vec<Symbol>,
    #[builder(default=None)]
    pub label: Option<String>,
}

impl Alternative {
    pub fn len(&self) -> usize {
        self.symbols.len()
    }
}

type Alternatives = Vec<Alternative>;

#[derive(Debug, TypedBuilder)]
#[builder(mutators(
    pub fn add_production(&mut self, nonterminal: Nonterminal, alternative: Alternative){
        self.productions.entry(nonterminal).or_default().push(alternative);
    }
))]
pub struct Grammar {
    pub name: String,
    pub start_symbol: Nonterminal,
    #[builder(via_mutators)]
    productions: IndexMap<Nonterminal, Alternatives>,
}

impl Grammar {
    pub fn count_nonterminals(&self) -> usize {
        self.productions.len()
    }
    pub fn nonterminals(&self) -> impl Iterator<Item = &'_ Nonterminal> {
        self.productions.keys()
    }
    pub fn alternatives(&self, nonterminal: &Nonterminal) -> Option<&Alternatives> {
        self.productions.get(nonterminal)
    }
    pub fn alternatives_len(&self, nonterminal: &Nonterminal) -> usize {
        self.productions
            .get(nonterminal)
            .map(|prod| prod.len())
            .unwrap_or_default()
    }
}
