use crate::grammar::{
    def::{Alternative, Associativity, PriorityLevel, SyntaxRule},
    symbols::{
        Cond, CondOp, DefinitionId, Expr, Identifier, Nonterminal, ParamType, Parameter, Symbol,
    },
    transformations::transform_rule,
};

/// Which ends of an alternative are recursive (i.e., reference the head nonterminal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecursiveEnds {
    /// Both ends: E op E
    Binary,
    /// Left end only: E op
    Left,
    /// Right end only: op E
    Right,
    /// Neither end: 'a', '(' E ')'
    None,
}

/// Desugars operator precedence and associativity annotations into
/// data-dependent grammar constructs.
///
/// For each nonterminal with multiple priority levels, this transformation:
///
/// 1. Classifies each alternative by which ends are recursive (reference the
///    head nonterminal): binary (both), prefix (right only), postfix (left
///    only), or non-recursive (neither).
///
/// 2. Assigns precedence numbers in reverse order (bottom = 1, each `>`
///    boundary increments), skipping levels that contain only non-recursive
///    alternatives.
///
/// 3. Adds a parameter `p` to the nonterminal: `E` becomes `E(p)`.
///
/// 4. Rewrites each alternative depending on its recursion type and
///    associativity. Let `pr` be the assigned precedence level.
///
///    **Binary `E op E`** — the rewrite depends on associativity:
///
///    - No associativity (default):
///      `[pr>=p] l=E(p) [l==0||l>=pr] op E(pr) {pr}`
///
///    - Left-associative:
///      `[pr>=p] l=E(p) [l==0||l>=pr] op E(pr+1) {pr}`
///      The right operand gets `pr+1`, preventing right-recursive nesting
///      at the same level.
///
///    - Right-associative:
///      `[pr>=p] l=E(p) [l==0||l>=pr+1] op E(pr) {pr}`
///      The postcondition uses `pr+1`, preventing left-recursive nesting
///      at the same level.
///
///    - Non-associative:
///      `[pr>=p] l=E(p) [l==0||l>=pr+1] op E(pr+1) {pr}`
///      Both restrictions apply, preventing any nesting at the same level.
///
///    **Prefix `op E`**: `op E(pr) {pr}`
///
///    **Postfix `E op`**: `[pr>=p] l=E(p) [l==0||l>=pr] op {0}`
///
///    **Non-recursive**: E references replaced with E(0), return {0}.
///
/// 5. Updates all external references to the desugared nonterminal:
///    `E` becomes `E(0)` in other rules.
///
/// ## Example: precedence with left associativity
///
/// Input:
///   E = 'a'
///     > E '*' E  left
///     | E '/' E  left
///     > E '+' E  left
///     | E '-' E  left
///     > E '<' E  non_assoc
///
/// Numbering (reverse, bottom = 1):
///   '<': 1, '+'/'-': 2, '*'/'/': 3, atoms: none
///
/// Output:
///   E(p)
///     = 'a'                                      {0}
///     | [3>=p] l=E(p) [l==0||l>=3] '*' E(4)     {3}
///     | [3>=p] l=E(p) [l==0||l>=3] '/' E(4)     {3}
///     | [2>=p] l=E(p) [l==0||l>=2] '+' E(3)     {2}
///     | [2>=p] l=E(p) [l==0||l>=2] '-' E(3)     {2}
///     | [1>=p] l=E(p) [l==0||l>=2] '<' E(2)     {1}
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
fn classify(alternative: &Alternative, head_name: &str) -> RecursiveEnds {
    let is_left = is_reference_to(alternative.symbols.first(), head_name);
    let is_right = is_reference_to(alternative.symbols.last(), head_name);
    match (is_left, is_right) {
        (true, true) => RecursiveEnds::Binary,
        (true, false) => RecursiveEnds::Left,
        (false, true) => RecursiveEnds::Right,
        (false, false) => RecursiveEnds::None,
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
            .any(|alt| classify(alt, head_name) != RecursiveEnds::None);
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
                (RecursiveEnds::Binary, Some(pr)) => {
                    rewrite_binary(&head_name, head_def, alt, *pr)
                }
                (RecursiveEnds::Right, Some(pr)) => rewrite_prefix(&head_name, head_def, alt, *pr),
                (RecursiveEnds::Left, Some(pr)) => rewrite_postfix(&head_name, head_def, alt, *pr),
                (RecursiveEnds::None, _) => rewrite_non_recursive(&head_name, alt),
                _ => alt,
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

/// Creates the precondition symbol: [pr >= p]
fn make_precondition(pr: i64) -> Symbol {
    Symbol::Condition(Expr::Cond(Cond {
        left: Box::new(Expr::Int(pr)),
        right: Box::new(Expr::Ref("p".to_string())),
        op: CondOp::Geq,
    }))
}

/// Creates the left binding symbol: l=E(p)
fn make_left_binding(head_name: &str, head_def: DefinitionId) -> Symbol {
    Symbol::Binding {
        name: "l".to_string(),
        symbol: Box::new(Symbol::Call {
            name: Identifier {
                name: head_name.to_string(),
                definition: Some(head_def),
            },
            arguments: vec![Expr::Ref("p".to_string())],
        }),
    }
}

/// Creates the postcondition symbol: [l==0 || l>=pr]
fn make_postcondition(pr: i64) -> Symbol {
    Symbol::Condition(Expr::Or(
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
    ))
}

/// Replaces a reference to the head nonterminal with a call passing 0.
fn replace_head_ref(symbol: Symbol, head_name: &str) -> Symbol {
    match symbol {
        Symbol::Identifier(id) if id.name == head_name => Symbol::Call {
            name: Identifier {
                name: id.name.clone(),
                definition: id.definition,
            },
            arguments: vec![Expr::Int(0)],
        },
        Symbol::Labeled { label, symbol } => Symbol::Labeled {
            label,
            symbol: Box::new(replace_head_ref(*symbol, head_name)),
        },
        _ => symbol,
    }
}

/// Rewrites a binary alternative `E op E` at precedence level `pr` into a
/// data-dependent form. The exact rewrite depends on associativity:
///
/// - No associativity (default):
///   `[pr>=p] l=E(p) [l==0||l>=pr] op E(pr) {pr}`
///
/// - Left-associative:
///   `[pr>=p] l=E(p) [l==0||l>=pr] op E(pr+1) {pr}`
///   (right end gets pr+1 to prevent right-recursive use at same level)
///
/// - Right-associative:
///   `[pr>=p] l=E(p) [l==0||l>=pr+1] op E(pr) {pr}`
///   (postcondition uses pr+1 to prevent left-recursive use at same level)
///
/// - Non-associative:
///   `[pr>=p] l=E(p) [l==0||l>=pr+1] op E(pr+1) {pr}`
///   (both restrictions)
fn rewrite_binary(head_name: &str, head_def: DefinitionId, alt: Alternative, pr: i64) -> Alternative {
    let assoc = alt.associativity;

    // Postcondition threshold: pr+1 for right-assoc and non-assoc, pr otherwise
    let postcond_threshold = match assoc {
        Some(Associativity::Right | Associativity::NonAssoc) => pr + 1,
        _ => pr,
    };

    // Right-end argument: pr+1 for left-assoc and non-assoc, pr otherwise
    let right_arg = match assoc {
        Some(Associativity::Left | Associativity::NonAssoc) => pr + 1,
        _ => pr,
    };

    let mut symbols = Vec::new();

    symbols.push(make_precondition(pr));
    symbols.push(make_left_binding(head_name, head_def));
    symbols.push(make_postcondition(postcond_threshold));

    // Middle symbols (everything except the first and last, which are the recursive E references)
    let num_symbols = alt.symbols.len();
    for symbol in alt.symbols.into_iter().skip(1).take(num_symbols.saturating_sub(2)) {
        symbols.push(symbol);
    }

    // E(right_arg)
    symbols.push(Symbol::Call {
        name: Identifier {
            name: head_name.to_string(),
            definition: Some(head_def),
        },
        arguments: vec![Expr::Int(right_arg)],
    });

    // {pr}
    symbols.push(Symbol::Return(Expr::Int(pr)));

    Alternative {
        symbols,
        label: alt.label,
        associativity: None,
    }
}

/// Rewrites a prefix alternative `op E` at precedence level `pr` into:
///   op E(pr) {pr}
fn rewrite_prefix(head_name: &str, head_def: DefinitionId, alt: Alternative, pr: i64) -> Alternative {
    let mut symbols = Vec::new();
    let num_symbols = alt.symbols.len();

    // All symbols except the last (right-end E), with any E references replaced by E(0)
    for symbol in alt.symbols.into_iter().take(num_symbols.saturating_sub(1)) {
        symbols.push(replace_head_ref(symbol, head_name));
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
        associativity: None,
    }
}

/// Rewrites a postfix alternative `E op` at precedence level `pr` into:
///   [pr>=p] l=E(p) [l==0||l>=pr] op {0}
fn rewrite_postfix(head_name: &str, head_def: DefinitionId, alt: Alternative, pr: i64) -> Alternative {
    let mut symbols = Vec::new();

    symbols.push(make_precondition(pr));
    symbols.push(make_left_binding(head_name, head_def));
    symbols.push(make_postcondition(pr));

    // All symbols except the first (left-end E), with any E references replaced by E(0)
    for symbol in alt.symbols.into_iter().skip(1) {
        symbols.push(replace_head_ref(symbol, head_name));
    }

    // {0}
    symbols.push(Symbol::Return(Expr::Int(0)));

    Alternative {
        symbols,
        label: alt.label,
        associativity: None,
    }
}

/// Rewrites a non-recursive alternative: replaces any E references with E(0)
/// and appends {0}.
fn rewrite_non_recursive(head_name: &str, alt: Alternative) -> Alternative {
    let mut symbols: Vec<Symbol> = alt
        .symbols
        .into_iter()
        .map(|symbol| replace_head_ref(symbol, head_name))
        .collect();

    symbols.push(Symbol::Return(Expr::Int(0)));

    Alternative {
        symbols,
        label: alt.label,
        associativity: None,
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
        grammar_def, id, left, lit, non_assoc, priority_level, ret, right, syntax_rule,
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

    /// Input grammar with prefix and postfix operators:
    ///   E = 'a' > E '!' > '-' E > E '*' E > E '+' E
    ///
    /// Precedences (bottom=1):
    ///   '+': 1, '*': 2, '-': 3, '!': 4
    fn prefix_postfix_input() -> GrammarDef {
        grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(
                        alternative!(lit!("a"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("!"))
                    ),
                    priority_level!(
                        alternative!(lit!("-"), id!("E"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("*"), id!("E"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E"))
                    )
                ),
            ]
        )
    }

    /// Expected grammar after desugaring:
    ///   E(p)
    ///     = 'a' {0}
    ///     | [4>=p] l=E(p) [l==0||l>=4] '!' {0}
    ///     | '-' E(3) {3}
    ///     | [2>=p] l=E(p) [l==0||l>=2] '*' E(2) {2}
    ///     | [1>=p] l=E(p) [l==0||l>=1] '+' E(1) {1}
    fn prefix_postfix_expected() -> GrammarDef {
        grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(4 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 4)),
                        lit!("!"),
                        ret!(0),
                    ),
                    alternative!(
                        lit!("-"),
                        call!("E", 3),
                        ret!(3),
                    ),
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
                )),
            ]
        )
    }

    #[test]
    fn test_prefix_postfix_desugaring() {
        let actual: Grammar = prefix_postfix_input().into();
        let expected: Grammar = prefix_postfix_expected().into();
        assert_eq!(actual, expected);
    }

    /// E = 'a' > E '+' E  left
    #[test]
    fn test_left_assoc() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("E"), lit!("+"), id!("E"); left!()))
                ),
            ]
        );

        // E(p) = 'a' {0} | [1>=p] l=E(p) [l==0||l>=1] '+' E(2) {1}
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 1)),
                        lit!("+"),
                        call!("E", 2),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.into();
        let expected: Grammar = expected.into();
        assert_eq!(actual, expected);
    }

    /// E = 'a' > E ';' E  right
    #[test]
    fn test_right_assoc() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("E"), lit!(";"), id!("E"); right!()))
                ),
            ]
        );

        // E(p) = 'a' {0} | [1>=p] l=E(p) [l==0||l>=2] ';' E(1) {1}
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!(";"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.into();
        let expected: Grammar = expected.into();
        assert_eq!(actual, expected);
    }

    /// E = 'a' > E '<' E  non_assoc
    #[test]
    fn test_non_assoc() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("E"), lit!("<"), id!("E"); non_assoc!()))
                ),
            ]
        );

        // E(p) = 'a' {0} | [1>=p] l=E(p) [l==0||l>=2] '<' E(2) {1}
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("<"),
                        call!("E", 2),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.into();
        let expected: Grammar = expected.into();
        assert_eq!(actual, expected);
    }

    /// E = 'a'
    ///   > E '*' E  left
    ///   | E '/' E  left
    ///   > E '+' E  left
    ///   | E '-' E  left
    ///   > E '<' E  non_assoc
    #[test]
    fn test_mixed_assoc() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(
                        alternative!(id!("E"), lit!("*"), id!("E"); left!()),
                        alternative!(id!("E"), lit!("/"), id!("E"); left!())
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E"); left!()),
                        alternative!(id!("E"), lit!("-"), id!("E"); left!())
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("<"), id!("E"); non_assoc!())
                    )
                ),
            ]
        );

        // E(p)
        //   = 'a' {0}
        //   | [3>=p] l=E(p) [l==0||l>=3] '*' E(4) {3}
        //   | [3>=p] l=E(p) [l==0||l>=3] '/' E(4) {3}
        //   | [2>=p] l=E(p) [l==0||l>=2] '+' E(3) {2}
        //   | [2>=p] l=E(p) [l==0||l>=2] '-' E(3) {2}
        //   | [1>=p] l=E(p) [l==0||l>=2] '<' E(2) {1}
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(3 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 3)),
                        lit!("*"),
                        call!("E", 4),
                        ret!(3),
                    ),
                    alternative!(
                        cond!(3 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 3)),
                        lit!("/"),
                        call!("E", 4),
                        ret!(3),
                    ),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("+"),
                        call!("E", 3),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("-"),
                        call!("E", 3),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("<"),
                        call!("E", 2),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.into();
        let expected: Grammar = expected.into();
        assert_eq!(actual, expected);
    }
}
