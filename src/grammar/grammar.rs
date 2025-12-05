use std::fmt::Display;

use indexmap::IndexMap;
use itertools::Itertools;
use typed_builder::TypedBuilder;

use crate::grammar::{
    regex::Regex,
    symbols::{Nonterminal, Symbol, Terminal, TerminalKind},
};

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
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub body: Alternative,
}

#[derive(Debug)]
pub struct LexicalRule {
    pub head: Terminal,
    pub regex: Regex,
}

#[derive(Debug, TypedBuilder)]
#[builder(mutators(
    pub fn add_syntax_rule(&mut self, head: Nonterminal, body: Alternative) {
        self.syntax_rules.push(SyntaxRule { head, body });
    }
    pub fn add_lexical_rule(&mut self, head: Terminal, regex: Regex) {
        self.lexical_rules.push(LexicalRule { head, regex });
    }
))]
pub struct GrammarDef {
    pub name: String,
    pub start_symbol: Nonterminal,
    #[builder(via_mutators)]
    syntax_rules: Vec<SyntaxRule>,
    #[builder(via_mutators)]
    lexical_rules: Vec<LexicalRule>,
}

impl From<GrammarDef> for Grammar {
    fn from(grammar_def: GrammarDef) -> Self {
        let mut lexical_rules: IndexMap<Terminal, Regex> = grammar_def
            .lexical_rules
            .into_iter()
            .map(|r| (r.head, r.regex))
            .collect();
        for SyntaxRule { head: _, body } in &grammar_def.syntax_rules {
            for symbol in &body.symbols {
                match symbol {
                    Symbol::Terminal(terminal) if terminal.kind == TerminalKind::Literal => {
                        if !lexical_rules.contains_key(terminal) {
                            lexical_rules.insert(terminal.clone(), Regex::literal(&terminal.name));
                        }
                    }
                    _ => (),
                }
            }
        }
        let productions: IndexMap<Nonterminal, Vec<Alternative>> = grammar_def
            .syntax_rules
            .into_iter()
            .fold(IndexMap::new(), |mut acc, r| {
                acc.entry(r.head).or_default().push(r.body);
                acc
            });
        Self {
            name: grammar_def.name,
            start_symbol: grammar_def.start_symbol,
            productions,
            lexical_rules,
        }
    }
}

type Alternatives = Vec<Alternative>;

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    pub start_symbol: Nonterminal,
    productions: IndexMap<Nonterminal, Alternatives>,
    lexical_rules: IndexMap<Terminal, Regex>,
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
    pub fn terminals(&self) -> impl Iterator<Item = &'_ Terminal> {
        self.lexical_rules.keys()
    }
    pub fn lexical_rules(&self, terminal: &Terminal) -> Option<&Regex> {
        self.lexical_rules.get(terminal)
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
