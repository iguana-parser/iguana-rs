use crate::{
    alternative,
    grammar::{
        def::{Alternative, PriorityLevel, SyntaxRule},
        symbols::{Nonterminal, Symbol},
        transformations::transform_rule,
    },
    priority_level,
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

pub fn ebnf_to_bnf(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    let mut counters = Counters::new();
    let mut new_rules = vec![];
    let mut transformed_rules: Vec<_> = syntax_rules
        .into_iter()
        .map(|rule| {
            let name = rule.head.name.clone();
            transform_rule(rule, |s| {
                rewrite_ebnf_symbol(s, &name, &mut counters, &mut new_rules)
            })
        })
        .collect();
    transformed_rules.extend(new_rules);
    transformed_rules
}

fn rewrite_ebnf_symbol(
    symbol: Symbol,
    parent_name: &str,
    counters: &mut Counters,
    new_rules: &mut Vec<SyntaxRule>,
) -> Symbol {
    let def = symbol.clone();
    match symbol {
        // Transform (A B C) into: S_Group0 ::= A B C
        Symbol::Group(symbols) => {
            let name = counters.next_group(parent_name);
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, def),
                priority_levels: vec![priority_level!(Alternative::new(symbols))],
            };
            new_rules.push(new_rule);
            Symbol::identifier(name)
        }
        // Transform A? into: S_Opt0 ::= A | ε
        Symbol::Opt(symbol) => {
            let name = counters.next_opt(parent_name);
            let transformed_symbol = rewrite_ebnf_symbol(*symbol, parent_name, counters, new_rules);
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, def),
                priority_levels: vec![priority_level!(
                    alternative!(transformed_symbol),
                    Alternative::empty()
                )],
            };
            new_rules.push(new_rule);
            Symbol::identifier(name)
        }
        // Transform (A | B | C) into: S_Alt0 ::= A | B | C
        Symbol::Alt(symbols) => {
            let name = counters.next_alt(parent_name);
            let transformed_symbols: Vec<_> = symbols
                .into_iter()
                .map(|s| rewrite_ebnf_symbol(s, parent_name, counters, new_rules))
                .collect();
            let alternatives: Vec<_> = transformed_symbols
                .into_iter()
                .map(|s| alternative!(s))
                .collect();
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, def),
                priority_levels: vec![PriorityLevel::new(alternatives)],
            };
            new_rules.push(new_rule);
            Symbol::identifier(name)
        }
        // Transform A* into: S_Star0 ::= S_Star0 A | ε (left-recursive)
        Symbol::Star(symbol) => {
            let name = counters.next_star(parent_name);
            let transformed_symbol = rewrite_ebnf_symbol(*symbol, parent_name, counters, new_rules);
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, def),
                priority_levels: vec![priority_level!(
                    alternative!(Symbol::identifier(name.clone()), transformed_symbol),
                    Alternative::empty()
                )],
            };
            new_rules.push(new_rule);
            Symbol::identifier(name)
        }
        // Transform A+ into: S_Plus0 ::= S_Plus0 A | A (left-recursive)
        Symbol::Plus(symbol) => {
            let name = counters.next_plus(parent_name);
            let transformed_symbol = rewrite_ebnf_symbol(*symbol, parent_name, counters, new_rules);
            let new_rule = SyntaxRule {
                head: Nonterminal::with_origin(&name, def),
                priority_levels: vec![priority_level!(
                    alternative!(Symbol::identifier(name.clone()), transformed_symbol.clone()),
                    alternative!(transformed_symbol)
                )],
            };
            new_rules.push(new_rule);
            Symbol::identifier(name)
        }
        _ => symbol,
    }
}

#[cfg(test)]
mod tests {
    use super::ebnf_to_bnf;
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

        let transformed = ebnf_to_bnf(grammar.syntax_rules);

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

        let transformed = ebnf_to_bnf(grammar.syntax_rules);

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

        let transformed = ebnf_to_bnf(grammar.syntax_rules);

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

        let transformed = ebnf_to_bnf(grammar.syntax_rules);

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

        let transformed = ebnf_to_bnf(grammar.syntax_rules);

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

        let transformed = ebnf_to_bnf(grammar.syntax_rules);

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

        let transformed = ebnf_to_bnf(grammar.syntax_rules);

        println!("Transformed grammar:\n{:?}", transformed);

        // Expected transformation:
        // S      ::= A S_Opt0
        // S_Opt0 ::= S_Alt0 | ε
        // S_Alt0 ::= B | S_Plus0
        // S_Plus0 ::= S_Plus0 S_Alt1 | S_Alt1
        // S_Alt1 ::= C | D
    }
}
