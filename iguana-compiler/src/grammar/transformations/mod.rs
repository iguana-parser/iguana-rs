use crate::grammar::{
    def::{Alternative, PriorityLevel, SyntaxRule},
    regex::Regex,
    symbols::{Nonterminal, Symbol},
};

pub mod ebnf_to_bnf;
pub mod exact_keyword_match;
pub mod exclude_desugaring;
pub mod layout_insertion;
pub mod precedence_desugaring;

/// Transforms a syntax rule by applying `f` to each individual symbol in every alternative.
pub fn transform_syntax_rule<F>(rule: SyntaxRule, mut transform_symbol: F) -> SyntaxRule
where
    F: FnMut(Symbol) -> Symbol,
{
    let name = rule.head.name;
    let new_priority_levels: Vec<_> = rule
        .priority_levels
        .into_iter()
        .map(|priority_level| {
            let new_alternatives: Vec<_> = priority_level
                .alternatives
                .into_iter()
                .map(|alternative| {
                    let new_symbols: Vec<_> = alternative
                        .symbols
                        .into_iter()
                        .map(&mut transform_symbol)
                        .collect();
                    Alternative {
                        symbols: new_symbols,
                        label: alternative.label,
                    }
                })
                .collect();
            PriorityLevel::with_associativity(new_alternatives, priority_level.associativity)
        })
        .collect();
    let head = Nonterminal {
        name,
        origin: rule.head.origin,
        parameters: rule.head.parameters,
    };
    SyntaxRule {
        head,
        priority_levels: new_priority_levels,
        layout: rule.layout,
    }
}

/// Transforms a syntax rule by applying `f` to the entire symbol list of each alternative.
/// Unlike `transform_syntax_rule`, this gives `f` access to the full list, allowing
/// insertions or reorderings (e.g., interleaving layout symbols).
pub fn transform_rule_by_symbols<F>(rule: SyntaxRule, mut transform_symbol: F) -> SyntaxRule
where
    F: FnMut(Vec<Symbol>) -> Vec<Symbol>,
{
    let layout = rule.layout;
    let new_priority_levels: Vec<_> = rule
        .priority_levels
        .into_iter()
        .map(|priority_level| {
            let new_alternatives: Vec<_> = priority_level
                .alternatives
                .into_iter()
                .map(|alternative| {
                    let new_symbols = transform_symbol(alternative.symbols);
                    Alternative {
                        symbols: new_symbols,
                        label: alternative.label,
                    }
                })
                .collect();
            PriorityLevel::with_associativity(new_alternatives, priority_level.associativity)
        })
        .collect();
    SyntaxRule {
        head: rule.head,
        priority_levels: new_priority_levels,
        layout,
    }
}

/// Applies a transformation to each node in a symbol tree (bottom-up):
/// children are transformed first, then `f` runs on the rebuilt node.
pub fn transform_symbol<F>(symbol: Symbol, f: &mut F) -> Symbol
where
    F: FnMut(Symbol) -> Symbol,
{
    let symbol = match symbol {
        Symbol::Labeled { label, symbol } => Symbol::Labeled {
            label,
            symbol: Box::new(transform_symbol(*symbol, f)),
        },
        Symbol::Binding { name, symbol } => Symbol::Binding {
            name,
            symbol: Box::new(transform_symbol(*symbol, f)),
        },
        Symbol::Restricted {
            symbol,
            restrictions,
        } => Symbol::Restricted {
            symbol: Box::new(transform_symbol(*symbol, f)),
            restrictions,
        },
        Symbol::Exclude { symbol, labels } => Symbol::Exclude {
            symbol: Box::new(transform_symbol(*symbol, f)),
            labels,
        },
        Symbol::Opt(symbol) => Symbol::Opt(Box::new(transform_symbol(*symbol, f))),
        Symbol::Group(symbols) => Symbol::Group(
            symbols
                .into_iter()
                .map(|symbol| transform_symbol(symbol, f))
                .collect(),
        ),
        Symbol::Alt(symbols) => Symbol::Alt(
            symbols
                .into_iter()
                .map(|symbol| transform_symbol(symbol, f))
                .collect(),
        ),
        Symbol::Star(symbol, sep) => Symbol::Star(
            Box::new(transform_symbol(*symbol, f)),
            sep.map(|sep| Box::new(transform_symbol(*sep, f))),
        ),
        Symbol::Plus(symbol, sep) => Symbol::Plus(
            Box::new(transform_symbol(*symbol, f)),
            sep.map(|sep| Box::new(transform_symbol(*sep, f))),
        ),
        leaf @ (Symbol::Identifier(_)
        | Symbol::Literal(_)
        | Symbol::Call { .. }
        | Symbol::Condition(_)
        | Symbol::Return(_)) => leaf,
    };
    f(symbol)
}

/// Visits each symbol in a syntax rule, recursively descending into nested symbols.
pub fn visit_syntax_rule<'a>(rule: &'a SyntaxRule, f: &mut impl FnMut(&'a Symbol)) {
    for alt in rule.alternatives() {
        for symbol in &alt.symbols {
            visit_symbol(symbol, f);
        }
    }
}

fn visit_symbol<'a>(symbol: &'a Symbol, f: &mut impl FnMut(&'a Symbol)) {
    f(symbol);
    match symbol {
        Symbol::Labeled { symbol, .. }
        | Symbol::Binding { symbol, .. }
        | Symbol::Restricted { symbol, .. }
        | Symbol::Exclude { symbol, .. }
        | Symbol::Opt(symbol) => visit_symbol(symbol, f),
        Symbol::Group(symbols) | Symbol::Alt(symbols) => {
            for s in symbols {
                visit_symbol(s, f);
            }
        }
        Symbol::Star(symbol, sep) | Symbol::Plus(symbol, sep) => {
            visit_symbol(symbol, f);
            if let Some(sep) = sep {
                visit_symbol(sep, f);
            }
        }
        Symbol::Identifier(_)
        | Symbol::Literal(_)
        | Symbol::Call { .. }
        | Symbol::Condition(_)
        | Symbol::Return(_) => {}
    }
}

/// Applies a transformation function to each node in a regex tree (top-down).
/// The function `f` is applied first, then the result is recursively traversed.
pub fn transform_regex<F>(regex: Regex, f: &mut F) -> Regex
where
    F: FnMut(Regex) -> Regex,
{
    let regex = f(regex);
    match regex {
        Regex::Seq(rs) => Regex::Seq(rs.into_iter().map(|r| transform_regex(r, f)).collect()),
        Regex::Alt(rs) => Regex::Alt(rs.into_iter().map(|r| transform_regex(r, f)).collect()),
        Regex::Star(r) => Regex::Star(Box::new(transform_regex(*r, f))),
        Regex::Plus(r) => Regex::Plus(Box::new(transform_regex(*r, f))),
        Regex::Opt(r) => Regex::Opt(Box::new(transform_regex(*r, f))),
        leaf => leaf,
    }
}
