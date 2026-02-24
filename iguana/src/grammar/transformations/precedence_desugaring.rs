use crate::grammar::{
    def::{Alternative, PriorityLevel, SyntaxRule},
    symbols::{
        Cond, CondOp, DefinitionId, Expr, Identifier, Nonterminal, ParamType, Parameter, Symbol,
    },
    transformations::transform_rule,
};

/// Classifies how an alternative relates to its head nonterminal
/// in terms of recursion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Recursion {
    /// Both left- and right-recursive: E op E
    Binary,
    /// Only left-recursive: E op
    Left,
    /// Only right-recursive: op E
    Right,
    /// Not recursive: 'a', '(' E ')'
    None,
}

/// Desugars operator precedence annotations into data-dependent grammar constructs.
///
/// For each nonterminal with multiple priority levels, this transformation:
///
/// 1. Classifies each alternative as binary, left-recursive, right-recursive,
///    or non-recursive based on whether it starts/ends with the head nonterminal.
///
/// 2. Assigns precedence numbers in reverse order (bottom = 1, each `>` boundary
///    increments), skipping levels that contain only non-recursive alternatives.
///
/// 3. Adds a parameter `p` to the nonterminal: `E` becomes `E(p)`.
///
/// 4. Rewrites each alternative:
///    - Binary `E op E` at level pr becomes:
///      `[pr>=p] l=E(p) [l==0||l>=pr] op E(pr) {pr}`
///    - Non-recursive alternatives get E references replaced with E(0)
///      and a return value of {0} appended.
///
/// 5. Updates all external references to the desugared nonterminal:
///    `E` becomes `E(0)` in other rules.
///
/// Example:
///
/// Input (Rascal convention — atoms first, then operators tightest to loosest):
///   E = 'a' | '(' E ')'
///     > E '*' E
///     > E '+' E | E '-' E
///
/// Numbering (reverse, bottom = 1, skip non-recursive levels):
///   '+', '-': level 1
///   '*':      level 2
///   atoms:    no number
///
/// Output:
///   E(p) 
///     = [2>=p] l=E(p) [l==0||l>=2] '*' E(2)    {2}
///     | [1>=p] l=E(p) [l==0||l>=1] '+' E(1)    {1}
///     | [1>=p] l=E(p) [l==0||l>=1] '-' E(1)    {1}
///     | 'a'                                    {0}
///     | '(' E(0) ')'                           {0}
pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    // First pass: identify which nonterminals will be desugared
    let desugared_names: Vec<String> = syntax_rules
        .iter()
        .filter(|rule| needs_desugaring(rule))
        .map(|rule| rule.head.name.clone())
        .collect();

    // Second pass: desugar rules and update external references
    syntax_rules
        .into_iter()
        .map(|rule| {
            if desugared_names.contains(&rule.head.name) {
                desugar_rule(rule)
            } else {
                update_external_references(rule, &desugared_names)
            }
        })
        .collect()
}

/// A rule needs desugaring if it has more than one priority level.
fn needs_desugaring(rule: &SyntaxRule) -> bool {
    rule.priority_levels.len() > 1
}

/// Classifies an alternative's recursion type relative to the head nonterminal.
fn classify(alternative: &Alternative, head_name: &str) -> Recursion {
    let is_left = is_reference_to(alternative.symbols.first(), head_name);
    let is_right = is_reference_to(alternative.symbols.last(), head_name);
    match (is_left, is_right) {
        (true, true) => Recursion::Binary,
        (true, false) => Recursion::Left,
        (false, true) => Recursion::Right,
        (false, false) => Recursion::None,
    }
}

/// Checks if a symbol is an identifier reference to the given nonterminal name.
fn is_reference_to(symbol: Option<&Symbol>, name: &str) -> bool {
    match symbol {
        Some(Symbol::Identifier(id)) => id.name == name,
        _ => false,
    }
}

/// Assigns precedence numbers to priority levels in reverse order.
/// Bottom level = 1, each `>` boundary increments.
/// Levels containing only non-recursive alternatives get `None`.
fn assign_precedence(
    priority_levels: &[PriorityLevel],
    head_name: &str,
) -> Vec<Option<i64>> {
    let mut result = vec![Option::<i64>::None; priority_levels.len()];
    let mut next_precedence: i64 = 1;

    // Iterate in reverse (bottom to top)
    for i in (0..priority_levels.len()).rev() {
        let has_recursive = priority_levels[i]
            .alternatives
            .iter()
            .any(|alt| classify(alt, head_name) != Recursion::None);
        if has_recursive {
            result[i] = Some(next_precedence);
            next_precedence += 1;
        }
    }

    result
}

fn desugar_rule(rule: SyntaxRule) -> SyntaxRule {
    let head_name = rule.head.name.clone();
    let precedences = assign_precedence(&rule.priority_levels, &head_name);

    // Find the resolved DefinitionId for the head nonterminal from any reference in the
    // alternatives. Identifiers are already resolved at this point in the pipeline.
    let head_def = find_definition_id(&rule.priority_levels, &head_name)
        .expect("desugared nonterminal should have at least one self-reference");

    let mut all_alternatives = Vec::new();

    for (level, precedence) in rule.priority_levels.into_iter().zip(precedences.iter()) {
        for alt in level.alternatives {
            let recursion = classify(&alt, &head_name);
            let rewritten = match (recursion, precedence) {
                (Recursion::Binary, Some(pr)) => rewrite_binary(&head_name, head_def, alt, *pr),
                (Recursion::None, _) => rewrite_non_recursive(&head_name, alt),
                _ => alt, // Left/Right-only: pass through for now (first iteration)
            };
            all_alternatives.push(rewritten);
        }
    }

    let head = Nonterminal {
        name: head_name,
        origin: rule.head.origin,
        parameters: vec![Parameter {
            name: "p".to_string(),
            ty: ParamType::I32,
        }],
    };

    SyntaxRule::new(head, vec![PriorityLevel::new(all_alternatives)])
}

/// Finds the resolved DefinitionId for a nonterminal by scanning its alternatives
/// for a self-reference.
fn find_definition_id(
    priority_levels: &[PriorityLevel],
    head_name: &str,
) -> Option<DefinitionId> {
    for level in priority_levels {
        for alt in &level.alternatives {
            for symbol in &alt.symbols {
                if let Symbol::Identifier(id) = symbol && id.name == head_name {
                    return id.definition;
                }
            }
        }
    }
    None
}

/// Rewrites a binary alternative `E op E` at precedence level `pr` into:
///   [pr>=p] l=E(p) [l==0||l>=pr] op E(pr) {pr}
fn rewrite_binary(head_name: &str, head_def: DefinitionId, alt: Alternative, pr: i64) -> Alternative {
    let mut symbols = Vec::new();

    // [pr >= p]
    symbols.push(Symbol::Condition(Expr::Cond(Cond {
        left: Box::new(Expr::Int(pr)),
        right: Box::new(Expr::Ref("p".to_string())),
        op: CondOp::Geq,
    })));

    // l=E(p)
    symbols.push(Symbol::Binding {
        name: "l".to_string(),
        symbol: Box::new(Symbol::Call {
            name: Identifier {
                name: head_name.to_string(),
                definition: Some(head_def),
            },
            arguments: vec![Expr::Ref("p".to_string())],
        }),
    });

    // [l==0 || l>=pr]
    symbols.push(Symbol::Condition(Expr::Or(
        Box::new(Expr::Cond(Cond {
            left: Box::new(Expr::Ref("l".to_string())),
            right: Box::new(Expr::Int(0)),
            op: CondOp::Eq,
        })),
        Box::new(Expr::Cond(Cond {
            left: Box::new(Expr::Ref("l".to_string())),
            right: Box::new(Expr::Int(pr)),
            op: CondOp::Geq,
        })),
    )));

    // Middle symbols (everything except the first and last, which are the recursive E references)
    let num_symbols = alt.symbols.len();
    for symbol in alt.symbols.into_iter().skip(1).take(num_symbols.saturating_sub(2)) {
        symbols.push(symbol);
    }

    // E(pr)
    symbols.push(Symbol::Call {
        name: Identifier {
            name: head_name.to_string(),
            definition: Some(head_def),
        },
        arguments: vec![Expr::Int(pr)],
    });

    // {pr}
    symbols.push(Symbol::Return(Expr::Int(pr)));

    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Rewrites a non-recursive alternative: replaces any E references with E(0)
/// and appends {0}.
fn rewrite_non_recursive(head_name: &str, alt: Alternative, ) -> Alternative {
    let mut symbols: Vec<Symbol> = alt
        .symbols
        .into_iter()
        .map(|symbol| match &symbol {
            Symbol::Identifier(id) if id.name == head_name => Symbol::Call {
                name: Identifier {
                    name: id.name.clone(),
                    definition: id.definition,
                },
                arguments: vec![Expr::Int(0)],
            },
            _ => symbol,
        })
        .collect();

    symbols.push(Symbol::Return(Expr::Int(0)));

    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Updates references to desugared nonterminals in non-desugared rules:
/// `E` becomes `E(0)`.
fn update_external_references(rule: SyntaxRule, desugared_names: &[String]) -> SyntaxRule {
    transform_rule(rule, |symbol| {
        if let Symbol::Identifier(id) = &symbol && desugared_names.contains(&id.name) {
            return Symbol::Call {
                name: Identifier {
                    name: id.name.clone(),
                    definition: id.definition,
                },
                arguments: vec![Expr::Int(0)],
            };
        }
        symbol
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        alternative, bind, call, cond,
        grammar::def::{Grammar, GrammarDef},
        grammar_def, id, lit, priority_level, ret, syntax_rule,
    };

    /// Input grammar with priority levels (before desugaring):
    ///   E
    ///     = 'a'
    ///     > E '*' E
    ///     > E '+' E
    ///     | E '-' E
    fn input_grammar() -> GrammarDef {
        grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(
                        alternative!(lit!("a"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("*"), id!("E"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E")),
                        alternative!(id!("E"), lit!("-"), id!("E"))
                    )
                ),
            ]
        )
    }

    /// Expected grammar after desugaring (hand-written, no associativity):
    ///   E(p)
    ///     = 'a' return 0
    ///     | [2>=p] l=E(p) [l==0||l>=2] '*' E(2) return 2
    ///     | [1>=p] l=E(p) [l==0||l>=1] '+' E(1) return 1
    ///     | [1>=p] l=E(p) [l==0||l>=1] '-' E(1) return 1
    fn expected_grammar() -> GrammarDef {
        grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("*"),
                        call!("E", 2),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 1)),
                        lit!("+"),
                        call!("E", 1),
                        ret!(1),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 1)),
                        lit!("-"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
            ]
        )
    }

    #[test]
    fn test_desugaring() {
        let actual: Grammar = input_grammar().into();
        let expected: Grammar = expected_grammar().into();
        assert_eq!(actual, expected);
    }
}
