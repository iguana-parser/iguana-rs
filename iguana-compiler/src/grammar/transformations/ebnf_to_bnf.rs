use rustc_hash::FxHashMap;

use crate::{
    alternative,
    grammar::{
        def::{Alternative, LayoutStrategy, PriorityLevel, SyntaxRule},
        symbols::{Identifier, Nonterminal, Symbol},
        transformations::transform_syntax_rule,
    },
    opt, plus, priority_level,
};

struct Counters {
    group: u16,
    opt: u16,
    alt: u16,
    star: u16,
    plus: u16,
}

impl Counters {
    fn new() -> Self {
        Self {
            group: 0,
            opt: 0,
            alt: 0,
            star: 0,
            plus: 0,
        }
    }

    fn next_group(&mut self) -> String {
        let name = format!("Group_{}", self.group);
        self.group += 1;
        name
    }

    fn next_opt(&mut self) -> String {
        let name = format!("Opt_{}", self.opt);
        self.opt += 1;
        name
    }

    fn next_alt(&mut self) -> String {
        let name = format!("Alt_{}", self.alt);
        self.alt += 1;
        name
    }

    fn next_star(&mut self) -> String {
        let name = format!("Star_{}", self.star);
        self.star += 1;
        name
    }

    fn next_plus(&mut self) -> String {
        let name = format!("Plus_{}", self.plus);
        self.plus += 1;
        name
    }
}

pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    let mut counters = Counters::new();
    let mut new_rules = vec![];
    // Maps (original EBNF symbol, parent layout) to the identifier of the synthetic
    // nonterminal introduced by EBNF-to-BNF rewriting. Layout is part of the key
    // because the synthetic rule inherits the parent's layout, and reusing a rule
    // generated under one layout for a parent with a different layout (e.g.,
    // @NoLayout vs default) would cause incorrect layout insertion downstream.
    let mut ebnf_symbols: FxHashMap<(Symbol, LayoutStrategy), Symbol> = FxHashMap::default();
    let mut transformed_rules: Vec<_> = syntax_rules
        .into_iter()
        .map(|rule| {
            let layout = rule.layout.clone();
            transform_syntax_rule(rule, |s| {
                rewrite_ebnf_symbol(s, &layout, &mut counters, &mut new_rules, &mut ebnf_symbols)
            })
        })
        .collect();
    transformed_rules.extend(new_rules);
    transformed_rules
}

fn rewrite_ebnf_symbol(
    symbol: Symbol,
    layout: &LayoutStrategy,
    counters: &mut Counters,
    new_rules: &mut Vec<SyntaxRule>,
    ebnf_symbols: &mut FxHashMap<(Symbol, LayoutStrategy), Symbol>,
) -> Symbol {
    if let Some(s) = ebnf_symbols.get(&(symbol.clone(), layout.clone())) {
        return s.clone();
    }
    let origin = symbol.clone();
    let res = match symbol {
        // Preserve label, transform inner symbol
        Symbol::Labeled { label, symbol } => {
            let transformed =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            Symbol::Labeled {
                label,
                symbol: Box::new(transformed),
            }
        }
        // Transform (A B C) into: Group_n ::= A B C
        Symbol::Group(symbols) => {
            let name = counters.next_group();
            let transformed_symbols: Vec<_> = symbols
                .into_iter()
                .map(|s| rewrite_ebnf_symbol(s, layout, counters, new_rules, ebnf_symbols))
                .collect();
            let head = Nonterminal::with_origin(&name, origin.clone());
            let new_rule = SyntaxRule {
                head,
                priority_levels: vec![priority_level!(Alternative {
                    symbols: transformed_symbols,
                    label: None,
                })],
                layout: layout.clone(),
                start: false,
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name,
                definition: None,
            })
        }
        // Transform A? into: Opt_n ::= A | ε
        Symbol::Opt(symbol) => {
            let name = counters.next_opt();
            let transformed_symbol =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, origin.clone()),
                priority_levels: vec![priority_level!(
                    alternative!(transformed_symbol),
                    Alternative::empty()
                )],
                layout: layout.clone(),
                start: false,
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name,
                definition: None,
            })
        }
        // Transform (A | B | C) into: Alt_n ::= A | B | C
        Symbol::Alt(symbols) => {
            let name = counters.next_alt();
            let transformed_symbols: Vec<_> = symbols
                .into_iter()
                .map(|s| rewrite_ebnf_symbol(s, layout, counters, new_rules, ebnf_symbols))
                .collect();
            let alternatives: Vec<_> = transformed_symbols
                .into_iter()
                .map(|s| alternative!(s))
                .collect();
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, origin.clone()),
                priority_levels: vec![PriorityLevel::new(alternatives)],
                layout: layout.clone(),
                start: false,
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name,
                definition: None,
            })
        }
        // Transform A* into: Star_n ::= (A+)?
        // This allows a more uniform parse-tree construction.
        Symbol::Star(symbol, sep) => {
            let new_symbol = match sep {
                Some(sep) => opt!(plus!(*symbol, *sep)),
                None => opt!(plus!(*symbol)),
            };
            let transformed_symbol = rewrite_ebnf_symbol(
                new_symbol.clone(),
                layout,
                counters,
                new_rules,
                ebnf_symbols,
            );
            let name = counters.next_star();
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, origin.clone()),
                priority_levels: vec![priority_level!(alternative!(transformed_symbol),)],
                layout: layout.clone(),
                start: false,
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name: name.clone(),
                definition: None,
            })
        }
        // Transform A+ into: Plus_n ::= Plus_n A | A (left-recursive)
        Symbol::Plus(symbol, sep) => {
            let name = counters.next_plus();
            let transformed_symbol =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            let new_symbol = Symbol::Identifier(Identifier {
                name: name.clone(),
                definition: None,
            });
            let new_rule = match sep {
                Some(sep) => {
                    let transformed_sep =
                        rewrite_ebnf_symbol(*sep, layout, counters, new_rules, ebnf_symbols);
                    SyntaxRule {
                        head: Nonterminal::with_origin(&name, origin.clone()),
                        priority_levels: vec![priority_level!(
                            alternative!(
                                new_symbol.clone(),
                                transformed_sep,
                                transformed_symbol.clone()
                            ),
                            alternative!(transformed_symbol)
                        )],
                        layout: layout.clone(),
                        start: false,
                    }
                }
                None => SyntaxRule {
                    head: Nonterminal::with_origin(&name, origin.clone()),
                    priority_levels: vec![priority_level!(
                        alternative!(new_symbol.clone(), transformed_symbol.clone()),
                        alternative!(transformed_symbol)
                    )],
                    layout: layout.clone(),
                    start: false,
                },
            };
            new_rules.push(new_rule);
            new_symbol
        }
        Symbol::Binding { name, symbol } => {
            let transformed =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            Symbol::Binding {
                name,
                symbol: Box::new(transformed),
            }
        }
        Symbol::Except { symbol, except } => {
            let transformed =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            Symbol::Except {
                symbol: Box::new(transformed),
                except,
            }
        }
        Symbol::FollowRestriction {
            symbol,
            restrictions,
        } => {
            let transformed =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            Symbol::FollowRestriction {
                symbol: Box::new(transformed),
                restrictions,
            }
        }
        Symbol::PrecedeRestriction {
            symbol,
            restriction,
        } => {
            let transformed =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            Symbol::PrecedeRestriction {
                symbol: Box::new(transformed),
                restriction,
            }
        }
        Symbol::Exclude { symbol, labels } => {
            let transformed =
                rewrite_ebnf_symbol(*symbol, layout, counters, new_rules, ebnf_symbols);
            Symbol::Exclude {
                symbol: Box::new(transformed),
                labels,
            }
        }
        Symbol::Identifier(_)
        | Symbol::Literal(_)
        | Symbol::Call { .. }
        | Symbol::Condition(_)
        | Symbol::Return(_) => symbol,
    };
    ebnf_symbols.insert((origin, layout.clone()), res.clone());
    res
}

#[cfg(test)]
mod tests {
    use super::transform;
    use crate::{
        alt, alternative, grammar_def, group, id, opt, plus, priority_level, star, syntax_rule,
    };

    #[test]
    fn test_single_group_transformation() {
        // S ::= A (B C D) E
        let grammar = grammar_def!("TestGrammar",
            syntax: [
                syntax_rule!("S" => priority_level!(alternative!(
                    id!("A"),
                    group!(id!("B"), id!("C"), id!("D")),
                    id!("E")
                )))
        ]);

        println!("Original grammar:\n{}\n", grammar);

        let transformed = transform(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);
    }

    #[test]
    fn test_opt_transformation() {
        // S ::= A B? C
        let grammar = grammar_def!("TestGrammar",
            syntax: [
                syntax_rule!("S" => priority_level!(alternative!(
                    id!("A"),
                    opt!(id!("B")),
                    id!("C")
                )))
        ]);

        println!("Original grammar:\n{}\n", grammar);

        let transformed = transform(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);
    }

    #[test]
    fn test_alt_transformation() {
        // S ::= A (B | C | D) E
        let grammar = grammar_def!("TestGrammar",
            syntax: [
                syntax_rule!("S" => priority_level!(alternative!(
                    id!("A"),
                    alt!(id!("B"), id!("C"), id!("D")),
                    id!("E")
                )))
        ]);

        println!("Original grammar:\n{}\n", grammar);

        let transformed = transform(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);
    }

    #[test]
    fn test_star_transformation() {
        // S ::= A B* C
        let grammar = grammar_def!("TestGrammar",
            syntax: [
                syntax_rule!("S" => priority_level!(alternative!(
                    id!("A"),
                    star!(id!("B")),
                    id!("C")
                )))
        ]);

        println!("Original grammar:\n{}\n", grammar);

        let transformed = transform(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);
    }

    #[test]
    fn test_plus_transformation() {
        // S ::= A B+ C
        let grammar = grammar_def!("TestGrammar",
            syntax: [
                syntax_rule!("S" => priority_level!(alternative!(
                    id!("A"),
                    plus!(id!("B")),
                    id!("C")
                )))
        ]);

        println!("Original grammar:\n{}\n", grammar);

        let transformed = transform(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);
    }

    #[test]
    fn test_combined_transformations() {
        // S ::= A (B | C)* D+ E?
        let grammar = grammar_def!("TestGrammar",
            syntax: [
                syntax_rule!("S" => priority_level!(alternative!(
                    id!("A"),
                    star!(alt!(id!("B"), id!("C"),)),
                    plus!(id!("D")),
                    opt!(id!("E"))
                )))
        ]);

        println!("Original grammar:\n{}\n", grammar);

        let transformed = transform(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);
    }

    #[test]
    fn test_deeply_nested_symbols() {
        // S ::= A ( B | ( C | D)+)?
        // This tests 3 levels of nesting:
        // - Opt containing Alt containing Plus containing Alt
        let grammar = grammar_def!("TestGrammar",
            syntax: [
                syntax_rule!("S" =>
                    priority_level!(alternative!(
                        id!("A"),
                        opt!(alt!(id!("B"), plus!(alt!(id!("C"), id!("D"))))),
                    )))
        ]);

        println!("Original grammar:\n{}\n", grammar);

        let transformed = transform(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);

        // Expected transformation:
        // S      ::= A S_Opt0
        // S_Opt0 ::= S_Alt0 | ε
        // S_Alt0 ::= B | S_Plus0
        // S_Plus0 ::= S_Plus0 S_Alt1 | S_Alt1
        // S_Alt1 ::= C | D
    }
}
