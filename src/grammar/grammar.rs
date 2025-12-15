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
    pub fn add_symbols(&mut self, symbols: Vec<Symbol>) {
        self.symbols.extend(symbols);
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

#[derive(Debug, TypedBuilder)]
#[builder(mutators(
    pub fn add_priority_level(&mut self, priority_level: PriorityLevel) {
        self.priority_levels.push(priority_level);
    }
    pub fn add_priority_levels(&mut self, priority_levels: Vec<PriorityLevel>) {
        self.priority_levels.extend(priority_levels);
    }
))]
pub struct SyntaxRule {
    pub head: Nonterminal,
    #[builder(via_mutators)]
    pub priority_levels: Vec<PriorityLevel>,
}

#[derive(Debug, TypedBuilder)]
#[builder(mutators(
    pub fn add_alternative(&mut self, alternative: Alternative) {
        self.alternatives.push(alternative);
    }
    pub fn add_alternatives(&mut self, alternatives: Vec<Alternative>) {
        self.alternatives.extend(alternatives);
    }
))]
pub struct PriorityLevel {
    #[builder(via_mutators)]
    pub alternatives: Vec<Alternative>,
}

#[derive(Debug)]
pub struct LexicalRule {
    pub head: Terminal,
    pub regex: Regex,
}

#[derive(Debug, TypedBuilder)]
#[builder(mutators(
    pub fn add_syntax_rule(&mut self, syntax_rule: SyntaxRule) {
        self.syntax_rules.push(syntax_rule);
    }
    pub fn add_lexical_rule(&mut self, head: Terminal, regex: Regex) {
        self.lexical_rules.push(LexicalRule { head, regex });
    }
    pub fn add_layout_definition(&mut self, terminal: Terminal) {
        self.layout_def.push(terminal);
    }
))]
pub struct GrammarDef {
    pub name: String,
    pub start_symbol: Nonterminal,
    #[builder(via_mutators)]
    pub syntax_rules: Vec<SyntaxRule>,
    #[builder(via_mutators)]
    pub lexical_rules: Vec<LexicalRule>,
    // Whitespace and comment nodes
    #[builder(via_mutators)]
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
        writeln!(f, "start: {}\n", self.start_symbol)?;
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

impl From<GrammarDef> for Grammar {
    fn from(grammar_def: GrammarDef) -> Self {
        let mut lexical_rules: IndexMap<Terminal, Regex> = grammar_def
            .lexical_rules
            .into_iter()
            .map(|r| (r.head, r.regex))
            .collect();
        for rule in &grammar_def.syntax_rules {
            for priority_level in &rule.priority_levels {
                for alternative in &priority_level.alternatives {
                    for symbol in &alternative.symbols {
                        match symbol {
                            Symbol::Terminal(terminal)
                                if terminal.kind == TerminalKind::Literal =>
                            {
                                if !lexical_rules.contains_key(terminal) {
                                    lexical_rules
                                        .insert(terminal.clone(), Regex::literal(&terminal.name));
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
        let productions: IndexMap<Nonterminal, Vec<Alternative>> = grammar_def
            .syntax_rules
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
            start_symbol: grammar_def.start_symbol,
            productions,
            lexical_rules,
            layout_defs: grammar_def.layout_def,
        }
    }
}

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    pub start_symbol: Nonterminal,
    productions: IndexMap<Nonterminal, Vec<Alternative>>,
    lexical_rules: IndexMap<Terminal, Regex>,
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
