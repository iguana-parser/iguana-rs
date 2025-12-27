use crate::grammar::{
    grammar::{Alternative, GrammarDef, PriorityLevel, SyntaxRule},
    symbols::{Nonterminal, NonterminalNodeKind, Symbol},
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
        let name = format!("{}_Group{}", parent_name, self.group);
        self.group += 1;
        name
    }

    fn next_opt(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Opt{}", parent_name, self.opt);
        self.opt += 1;
        name
    }

    fn next_alt(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Alt{}", parent_name, self.alt);
        self.alt += 1;
        name
    }

    fn next_star(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Star{}", parent_name, self.star);
        self.star += 1;
        name
    }

    fn next_plus(&mut self, parent_name: &str) -> String {
        let name = format!("{}_Plus{}", parent_name, self.plus);
        self.plus += 1;
        name
    }
}

pub fn ebnf_to_bnf(grammar: GrammarDef) -> GrammarDef {
    let mut counters = Counters::new();
    let mut new_rules = vec![];
    let mut transformed_rules: Vec<_> = grammar
        .syntax_rules
        .into_iter()
        .map(|rule| {
            let parent_name = &rule.head.name;
            let new_priority_levels: Vec<_> = rule
                .priority_levels
                .into_iter()
                .map(|pl| {
                    let new_alternatives: Vec<_> = pl
                        .alternatives
                        .into_iter()
                        .map(|alt| {
                            let new_symbols: Vec<_> = alt
                                .symbols
                                .into_iter()
                                .map(|s| {
                                    transform_symbol(s, parent_name, &mut counters, &mut new_rules)
                                })
                                .collect();
                            Alternative::builder().add_symbols(new_symbols).build()
                        })
                        .collect();
                    PriorityLevel::builder()
                        .add_alternatives(new_alternatives)
                        .build()
                })
                .collect();
            SyntaxRule::builder()
                .head(rule.head)
                .add_priority_levels(new_priority_levels)
                .build()
        })
        .collect();
    transformed_rules.extend(new_rules);
    GrammarDef {
        name: grammar.name,
        start_symbol: grammar.start_symbol,
        syntax_rules: transformed_rules,
        lexical_rules: grammar.lexical_rules,
        layout_def: grammar.layout_def,
    }
}

fn transform_symbol(
    symbol: Symbol,
    parent_name: &str,
    counters: &mut Counters,
    new_rules: &mut Vec<SyntaxRule>,
) -> Symbol {
    match symbol {
        // Transform (A B C) into: S_Group0 ::= A B C
        Symbol::Group(symbols) => {
            let name = counters.next_group(parent_name);
            let new_rule = SyntaxRule::builder()
                .head(Nonterminal::with_kind(&name, NonterminalNodeKind::Group))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(Alternative::builder().add_symbols(symbols).build())
                        .build(),
                )
                .build();
            new_rules.push(new_rule);
            Symbol::Nonterminal(Nonterminal::new(name))
        }
        // Transform A? into: S_Opt0 ::= A | ε
        Symbol::Opt(symbol) => {
            let name = counters.next_opt(parent_name);
            let transformed_symbol = transform_symbol(*symbol, parent_name, counters, new_rules);
            let new_rule = SyntaxRule::builder()
                .head(Nonterminal::with_kind(&name, NonterminalNodeKind::Opt))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(transformed_symbol)
                                .build(),
                        )
                        .add_alternative(Alternative::builder().build())
                        .build(),
                )
                .build();
            new_rules.push(new_rule);
            Symbol::Nonterminal(Nonterminal::new(name))
        }
        // Transform (A | B | C) into: S_Alt0 ::= A | B | C
        Symbol::Alt(symbols) => {
            let name = counters.next_alt(parent_name);
            let transformed_symbols: Vec<_> = symbols
                .into_iter()
                .map(|s| transform_symbol(s, parent_name, counters, new_rules))
                .collect();
            let alternatives: Vec<_> = transformed_symbols
                .into_iter()
                .map(|s| Alternative::builder().add_symbol(s).build())
                .collect();
            let new_rule = SyntaxRule::builder()
                .head(Nonterminal::with_kind(&name, NonterminalNodeKind::Alt))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternatives(alternatives)
                        .build(),
                )
                .build();
            new_rules.push(new_rule);
            Symbol::Nonterminal(Nonterminal::new(name))
        }
        // Transform A* into: S_Star0 ::= S_Star0 A | ε (left-recursive)
        Symbol::Star(symbol) => {
            let name = counters.next_star(parent_name);
            let transformed_symbol = transform_symbol(*symbol, parent_name, counters, new_rules);
            let new_rule = SyntaxRule::builder()
                .head(Nonterminal::with_kind(&name, NonterminalNodeKind::Star))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::Nonterminal(Nonterminal::new(&name)))
                                .add_symbol(transformed_symbol)
                                .build(),
                        )
                        .add_alternative(Alternative::builder().build())
                        .build(),
                )
                .build();
            new_rules.push(new_rule);
            Symbol::Nonterminal(Nonterminal::new(name))
        }
        // Transform A+ into: S_Plus0 ::= S_Plus0 A | A (left-recursive)
        Symbol::Plus(symbol) => {
            let name = counters.next_plus(parent_name);
            let transformed_symbol = transform_symbol(*symbol, parent_name, counters, new_rules);
            let new_rule = SyntaxRule::builder()
                .head(Nonterminal::with_kind(&name, NonterminalNodeKind::Plus))
                .add_priority_level(
                    PriorityLevel::builder()
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(Symbol::Nonterminal(Nonterminal::new(&name)))
                                .add_symbol(transformed_symbol.clone())
                                .build(),
                        )
                        .add_alternative(
                            Alternative::builder()
                                .add_symbol(transformed_symbol)
                                .build(),
                        )
                        .build(),
                )
                .build();
            new_rules.push(new_rule);
            Symbol::Nonterminal(Nonterminal::new(name))
        }
        _ => symbol,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_group_transformation() {
        // S ::= A (B C D) E
        let grammar = GrammarDef::builder()
            .name("TestGrammar".to_string())
            .start_symbol(Nonterminal::new("S"))
            .add_syntax_rule(
                SyntaxRule::builder()
                    .head(Nonterminal::new("S"))
                    .add_priority_level(
                        PriorityLevel::builder()
                            .add_alternative(
                                Alternative::builder()
                                    .add_symbol(Symbol::nonterminal("A"))
                                    .add_symbol(Symbol::Group(vec![
                                        Symbol::nonterminal("B"),
                                        Symbol::nonterminal("C"),
                                        Symbol::nonterminal("D"),
                                    ]))
                                    .add_symbol(Symbol::nonterminal("E"))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        println!("Original grammar:\n{}\n", grammar);

        let transformed = ebnf_to_bnf(grammar);

        println!("Transformed grammar:\n{}", transformed);
    }

    #[test]
    fn test_opt_transformation() {
        // S ::= A B? C
        let grammar = GrammarDef::builder()
            .name("TestGrammar".to_string())
            .start_symbol(Nonterminal::new("S"))
            .add_syntax_rule(
                SyntaxRule::builder()
                    .head(Nonterminal::new("S"))
                    .add_priority_level(
                        PriorityLevel::builder()
                            .add_alternative(
                                Alternative::builder()
                                    .add_symbol(Symbol::nonterminal("A"))
                                    .add_symbol(Symbol::Opt(Box::new(Symbol::nonterminal("B"))))
                                    .add_symbol(Symbol::nonterminal("C"))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        println!("Original grammar:\n{}\n", grammar);

        let transformed = ebnf_to_bnf(grammar);

        println!("Transformed grammar:\n{}", transformed);
    }

    #[test]
    fn test_alt_transformation() {
        // S ::= A (B | C | D) E
        let grammar = GrammarDef::builder()
            .name("TestGrammar".to_string())
            .start_symbol(Nonterminal::new("S"))
            .add_syntax_rule(
                SyntaxRule::builder()
                    .head(Nonterminal::new("S"))
                    .add_priority_level(
                        PriorityLevel::builder()
                            .add_alternative(
                                Alternative::builder()
                                    .add_symbol(Symbol::nonterminal("A"))
                                    .add_symbol(Symbol::Alt(vec![
                                        Symbol::nonterminal("B"),
                                        Symbol::nonterminal("C"),
                                        Symbol::nonterminal("D"),
                                    ]))
                                    .add_symbol(Symbol::nonterminal("E"))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        println!("Original grammar:\n{}\n", grammar);

        let transformed = ebnf_to_bnf(grammar);

        println!("Transformed grammar:\n{}", transformed);
    }

    #[test]
    fn test_star_transformation() {
        // S ::= A B* C
        let grammar = GrammarDef::builder()
            .name("TestGrammar".to_string())
            .start_symbol(Nonterminal::new("S"))
            .add_syntax_rule(
                SyntaxRule::builder()
                    .head(Nonterminal::new("S"))
                    .add_priority_level(
                        PriorityLevel::builder()
                            .add_alternative(
                                Alternative::builder()
                                    .add_symbol(Symbol::nonterminal("A"))
                                    .add_symbol(Symbol::Star(Box::new(Symbol::nonterminal("B"))))
                                    .add_symbol(Symbol::nonterminal("C"))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        println!("Original grammar:\n{}\n", grammar);

        let transformed = ebnf_to_bnf(grammar);

        println!("Transformed grammar:\n{}", transformed);
    }

    #[test]
    fn test_plus_transformation() {
        // S ::= A B+ C
        let grammar = GrammarDef::builder()
            .name("TestGrammar".to_string())
            .start_symbol(Nonterminal::new("S"))
            .add_syntax_rule(
                SyntaxRule::builder()
                    .head(Nonterminal::new("S"))
                    .add_priority_level(
                        PriorityLevel::builder()
                            .add_alternative(
                                Alternative::builder()
                                    .add_symbol(Symbol::nonterminal("A"))
                                    .add_symbol(Symbol::Plus(Box::new(Symbol::nonterminal("B"))))
                                    .add_symbol(Symbol::nonterminal("C"))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        println!("Original grammar:\n{}\n", grammar);

        let transformed = ebnf_to_bnf(grammar);

        println!("Transformed grammar:\n{}", transformed);
    }

    #[test]
    fn test_combined_transformations() {
        // S ::= A (B | C)* D+ E?
        let grammar = GrammarDef::builder()
            .name("TestGrammar".to_string())
            .start_symbol(Nonterminal::new("S"))
            .add_syntax_rule(
                SyntaxRule::builder()
                    .head(Nonterminal::new("S"))
                    .add_priority_level(
                        PriorityLevel::builder()
                            .add_alternative(
                                Alternative::builder()
                                    .add_symbol(Symbol::nonterminal("A"))
                                    .add_symbol(Symbol::Star(Box::new(Symbol::Alt(vec![
                                        Symbol::nonterminal("B"),
                                        Symbol::nonterminal("C"),
                                    ]))))
                                    .add_symbol(Symbol::Plus(Box::new(Symbol::nonterminal("D"))))
                                    .add_symbol(Symbol::Opt(Box::new(Symbol::nonterminal("E"))))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        println!("Original grammar:\n{}\n", grammar);

        let transformed = ebnf_to_bnf(grammar);

        println!("Transformed grammar:\n{}", transformed);
    }

    #[test]
    fn test_deeply_nested_symbols() {
        // S ::= A ( B | ( C | D)+)?
        // This tests 3 levels of nesting:
        // - Opt containing Alt containing Plus containing Alt
        let grammar = GrammarDef::builder()
            .name("TestGrammar".to_string())
            .start_symbol(Nonterminal::new("S"))
            .add_syntax_rule(
                SyntaxRule::builder()
                    .head(Nonterminal::new("S"))
                    .add_priority_level(
                        PriorityLevel::builder()
                            .add_alternative(
                                Alternative::builder()
                                    .add_symbol(Symbol::nonterminal("A"))
                                    .add_symbol(Symbol::Opt(Box::new(Symbol::Alt(vec![
                                        Symbol::nonterminal("B"),
                                        Symbol::Plus(Box::new(Symbol::Alt(vec![
                                            Symbol::nonterminal("C"),
                                            Symbol::nonterminal("D"),
                                        ]))),
                                    ]))))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        println!("Original grammar:\n{}\n", grammar);

        let transformed = ebnf_to_bnf(grammar);

        println!("Transformed grammar:\n{}", transformed);

        // Expected transformation:
        // S      ::= A S_Opt0
        // S_Opt0 ::= S_Alt0 | ε
        // S_Alt0 ::= B | S_Plus0
        // S_Plus0 ::= S_Plus0 S_Alt1 | S_Alt1
        // S_Alt1 ::= C | D
    }
}
