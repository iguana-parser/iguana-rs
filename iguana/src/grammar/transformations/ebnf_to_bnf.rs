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

    fn next_group(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Group_{}", parent_name, self.group);
        self.group += 1;
        name
    }

    fn next_opt(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Opt_{}", parent_name, self.opt);
        self.opt += 1;
        name
    }

    fn next_alt(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Alt_{}", parent_name, self.alt);
        self.alt += 1;
        name
    }

    fn next_star(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Star_{}", parent_name, self.star);
        self.star += 1;
        name
    }

    fn next_plus(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Plus_{}", parent_name, self.plus);
        self.plus += 1;
        name
    }
}

pub fn transform(syntax_rules: Vec<SyntaxRule>) -> (Vec<SyntaxRule>, FxHashMap<Symbol, Symbol>) {
    let mut counters = Counters::new();
    let mut new_rules = vec![];
    // A map from a symbol to an identifier that refers to the EBNF definition
    // that is introduced as the result of EBNF to BNF rewriting.
    let mut ebnf_symbols: FxHashMap<Symbol, Symbol> = FxHashMap::default();
    let mut transformed_rules: Vec<_> = syntax_rules
        .into_iter()
        .map(|rule| {
            let name = rule.head.name.clone();
            let layout = rule.layout.clone();
            transform_syntax_rule(rule, |s| {
                rewrite_ebnf_symbol(
                    s,
                    &name,
                    &layout,
                    &mut counters,
                    &mut new_rules,
                    &mut ebnf_symbols,
                )
            })
        })
        .collect();
    transformed_rules.extend(new_rules);
    (transformed_rules, ebnf_symbols)
}

fn rewrite_ebnf_symbol(
    symbol: Symbol,
    parent_name: &str,
    layout: &LayoutStrategy,
    counters: &mut Counters,
    new_rules: &mut Vec<SyntaxRule>,
    ebnf_symbols: &mut FxHashMap<Symbol, Symbol>,
) -> Symbol {
    if let Some(s) = ebnf_symbols.get(&symbol) {
        return s.clone();
    }
    let origin = symbol.clone();
    let res = match symbol {
        // Preserve label, transform inner symbol
        Symbol::Labeled { label, symbol } => {
            let transformed = rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
            Symbol::Labeled {
                label,
                symbol: Box::new(transformed),
            }
        }
        // Transform (A B C) into: S_Group0 ::= A B C
        Symbol::Group(symbols) => {
            let name = counters.next_group(parent_name);
            let transformed_symbols: Vec<_> = symbols
                .into_iter()
                .map(|s| rewrite_ebnf_symbol(s, parent_name, layout, counters, new_rules, ebnf_symbols))
                .collect();
            let head = Nonterminal::with_origin(&name, origin.clone());
            let new_rule = SyntaxRule {
                head,
                priority_levels: vec![priority_level!(Alternative {
                    symbols: transformed_symbols,
                    label: None,
                })],
                layout: layout.clone(),
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name,
                definition: None,
            })
        }
        // Transform A? into: S_Opt0 ::= A | ε
        Symbol::Opt(symbol) => {
            let name = counters.next_opt(parent_name);
            let transformed_symbol =
                rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, origin.clone()),
                priority_levels: vec![priority_level!(
                    alternative!(transformed_symbol),
                    Alternative::empty()
                )],
                layout: layout.clone(),
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name,
                definition: None,
            })
        }
        // Transform (A | B | C) into: S_Alt0 ::= A | B | C
        Symbol::Alt(symbols) => {
            let name = counters.next_alt(parent_name);
            let transformed_symbols: Vec<_> = symbols
                .into_iter()
                .map(|s| rewrite_ebnf_symbol(s, parent_name, layout, counters, new_rules, ebnf_symbols))
                .collect();
            let alternatives: Vec<_> = transformed_symbols
                .into_iter()
                .map(|s| alternative!(s))
                .collect();
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, origin.clone()),
                priority_levels: vec![PriorityLevel::new(alternatives)],
                layout: layout.clone(),
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name,
                definition: None,
            })
        }
        // Transform A* into: A_Star ::= (A+)?
        // This allows a more uniform parse-tree construction.
        Symbol::Star(symbol, sep) => {
            let new_symbol = match sep {
                Some(sep) => opt!(plus!(*symbol, *sep)),
                None => opt!(plus!(*symbol)),
            };
            let transformed_symbol = rewrite_ebnf_symbol(
                new_symbol.clone(),
                parent_name,
                layout,
                counters,
                new_rules,
                ebnf_symbols,
            );
            let name = counters.next_star(parent_name);
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, origin.clone()),
                priority_levels: vec![priority_level!(alternative!(transformed_symbol),)],
                layout: layout.clone(),
            };
            new_rules.push(new_rule);
            Symbol::Identifier(Identifier {
                name: name.clone(),
                definition: None,
            })
        }
        // Transform A+ into: S_Plus0 ::= S_Plus0 A | A (left-recursive)
        Symbol::Plus(symbol, sep) => {
            let name = counters.next_plus(parent_name);
            let transformed_symbol =
                rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
            let new_symbol = Symbol::Identifier(Identifier {
                name: name.clone(),
                definition: None,
            });
            let new_rule = match sep {
                Some(sep) => {
                    let transformed_sep =
                        rewrite_ebnf_symbol(*sep, parent_name, layout, counters, new_rules, ebnf_symbols);
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
                    }
                }
                None => SyntaxRule {
                    head: Nonterminal::with_origin(&name, origin.clone()),
                    priority_levels: vec![priority_level!(
                        alternative!(new_symbol.clone(), transformed_symbol.clone()),
                        alternative!(transformed_symbol)
                    )],
                    layout: layout.clone(),
                },
            };
            new_rules.push(new_rule);
            new_symbol
        }
        Symbol::Binding { name, symbol } => {
            let transformed = rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
            Symbol::Binding {
                name,
                symbol: Box::new(transformed),
            }
        }
        Symbol::Except { symbol, except } => {
            let transformed = rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
            Symbol::Except {
                symbol: Box::new(transformed),
                except,
            }
        }
        Symbol::FollowRestriction {
            symbol,
            restriction,
        } => {
            let transformed = rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
            Symbol::FollowRestriction {
                symbol: Box::new(transformed),
                restriction,
            }
        }
        Symbol::PrecedeRestriction {
            symbol,
            restriction,
        } => {
            let transformed = rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
            Symbol::PrecedeRestriction {
                symbol: Box::new(transformed),
                restriction,
            }
        }
        Symbol::Exclude { symbol, labels } => {
            let transformed = rewrite_ebnf_symbol(*symbol, parent_name, layout, counters, new_rules, ebnf_symbols);
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
    ebnf_symbols.insert(origin, res.clone());
    res
}

#[cfg(test)]
mod tests {
    use super::transform;
    use crate::{
        alt, alternative,
        grammar::def::{SymbolTable, create_symbol_table},
        grammar_def, group, id, opt, plus, priority_level, star, syntax_rule,
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
