// Precedence Desugaring Algorithm
// ================================
// Based on Sections 3.2–3.5 of:
//   "Operator Precedence for Data-Dependent Grammars" (PEPM'16)
//   Ali Afroozeh, Anastasia Izmaylova
//
// Phase 1 — Identify desugared nonterminals
// ------------------------------------------
// Collect all nonterminals with multiple priority levels (`>`). Then for each,
// find its "recursive name" — the nonterminal at left/right ends of its
// alternatives:
// - Direct (Section 3.3): head name appears at ends (e.g., `E` in `E '+' E`)
// - Indirect (Section 3.5): a different desugared nonterminal appears at ends
//   (e.g., `E` in `E_except_comma`'s alternatives)
//
// Phase 2 — Desugar each rule
// ----------------------------
// Let `R` be the recursive name, `pr` the assigned precedence (bottom=1,
// increments at each `>`).
//
// Assign precedence (Section 3.3): levels with only non-recursive alternatives
// get no number.
//
// Rewrite alternatives (Section 3.3):
// - Binary `R op R`:  `[pr>=p] l=R(p) [l==0||l>=pr] op R(pr) {pr}`
//   with associativity adjusting thresholds (Section 3.4):
//   left gives right end `pr+1`, right gives postcondition `pr+1`,
//   non-assoc does both.
// - Prefix `op R`:    `op R(pr) {pr}`
// - Postfix `R op`:   `[pr>=p] l=R(p) [l==0||l>=pr] op {0}`
// - Non-recursive:    replace `R` refs with `R(0)`, return `{0}`
//
// Min trick (Section 3.3, deep case): when a prefix exists at lower precedence
// than a binary/prefix alternative, the higher alternative binds the right end:
// `r=R(pr) {r==0 ? pr : min(r, pr)}`. This propagates precedence upward
// through right-recursive chains.
//
// Phase 3 — Update external references
// --------------------------------------
// All references to desugared nonterminals become calls with argument 0:
// `E` → `E(0)`. This applies to ALL rules, including desugared ones (to handle
// cross-references like `E` referencing `E_except_comma`).
//
// Indirect case (Section 3.5)
// ----------------------------
// `desugar_rule` is agnostic about whether the recursive name equals the head
// name. For `E_except_comma` with recursive name `E`, the calls go to `E(p)` /
// `E(pr)` — the parameter is passed through to `E`, which does the enforcement.

use crate::grammar::{
    def::{Alternative, Associativity, PriorityLevel, SyntaxRule},
    symbols::{
        Cond, CondOp, DefinitionId, Expr, Identifier, Nonterminal, ParamType, Parameter, Symbol,
    },
    transformations::transform_syntax_rule,
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

pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    // First pass: identify which nonterminals need desugaring (multiple priority levels)
    // and find their recursive names. At this stage, classify uses exact name matching only,
    // which is sufficient because find_recursive_name checks each candidate name individually.
    let desugared_names: Vec<String> = syntax_rules
        .iter()
        .filter(|rule| needs_desugaring(rule))
        .map(|rule| rule.head.name.clone())
        .collect();

    // Map each desugared nonterminal to its recursive name.
    // Direct recursion: E → E. Indirect recursion: E_except_comma → E.
    let mut recursive_names: Vec<(String, String)> = Vec::new();
    for rule in &syntax_rules {
        if !needs_desugaring(rule) {
            continue;
        }
        if let Some(rec_name) = find_recursive_name(rule, &desugared_names) {
            recursive_names.push((rule.head.name.clone(), rec_name));
        }
    }

    let all_desugared: Vec<String> =
        recursive_names.iter().map(|(name, _)| name.clone()).collect();

    // Second pass: desugar rules and update external references.
    // When classifying alternatives, we need to recognize indirect recursion: if E and
    // E_except_comma both have recursive name E, then E_except_comma at a left/right end
    // of E's alternatives is an indirect reference to E. The recursive_names mapping
    // enables this lookup.
    syntax_rules
        .into_iter()
        .map(|rule| {
            let rule = if let Some((_, rec_name)) = recursive_names
                .iter()
                .find(|(name, _)| *name == rule.head.name)
            {
                desugar_rule(rule, rec_name, &recursive_names)
            } else {
                rule
            };
            update_external_references(rule, &all_desugared)
        })
        .collect()
}

/// Finds the nonterminal name that appears at recursive (left/right) ends of a rule's
/// alternatives. For directly recursive rules this is the head name itself. For indirectly
/// recursive rules this is a different desugared nonterminal.
/// Returns `None` if no recursive ends are found.
fn find_recursive_name(rule: &SyntaxRule, desugared_names: &[String]) -> Option<String> {
    // Check direct recursion first
    if has_recursive_ends(rule, &rule.head.name) {
        return Some(rule.head.name.clone());
    }
    // Check indirect recursion: does any desugared nonterminal appear at left/right ends?
    for name in desugared_names {
        if has_recursive_ends(rule, name) {
            return Some(name.clone());
        }
    }
    None
}

/// Returns true if any alternative in the rule has the given nonterminal at a left or right end.
/// Uses exact name matching only (no indirect lookup), since this runs before the full
/// recursive_names mapping is built. This is correct because find_recursive_name tries
/// each desugared name individually.
fn has_recursive_ends(rule: &SyntaxRule, name: &str) -> bool {
    rule.priority_levels.iter().any(|pl| {
        pl.alternatives
            .iter()
            .any(|alt| classify(alt, name, &[]) != RecursiveEnds::None)
    })
}

/// A rule needs desugaring if it has more than one priority level.
fn needs_desugaring(rule: &SyntaxRule) -> bool {
    rule.priority_levels.len() > 1
}

/// Classifies an alternative's recursion type relative to the recursive nonterminal.
/// `recursive_names` maps each desugared nonterminal to its recursive name, enabling
/// recognition of indirect recursion (e.g., E_except_comma as a reference to E).
fn classify(
    alternative: &Alternative,
    recursive_name: &str,
    recursive_names: &[(String, String)],
) -> RecursiveEnds {
    let is_left = is_reference_to(alternative.symbols.first(), recursive_name, recursive_names);
    let is_right = is_reference_to(alternative.symbols.last(), recursive_name, recursive_names);
    match (is_left, is_right) {
        (true, true) => RecursiveEnds::Binary,
        (true, false) => RecursiveEnds::Left,
        (false, true) => RecursiveEnds::Right,
        (false, false) => RecursiveEnds::None,
    }
}

/// Checks if a symbol is an identifier reference to the given recursive name,
/// either directly or indirectly (via a nonterminal that shares the same recursive name).
fn is_reference_to(
    symbol: Option<&Symbol>,
    recursive_name: &str,
    recursive_names: &[(String, String)],
) -> bool {
    match symbol {
        Some(Symbol::Identifier(id)) => {
            id.name == recursive_name
                || recursive_names
                    .iter()
                    .any(|(name, rec)| name == &id.name && rec == recursive_name)
        }
        _ => false,
    }
}

/// Assigns precedence numbers to priority levels in reverse order.
/// Bottom level = 1, each `>` boundary increments.
/// Levels containing only non-recursive alternatives get `None`.
fn assign_precedence(
    priority_levels: &[PriorityLevel],
    recursive_name: &str,
    recursive_names: &[(String, String)],
) -> Vec<Option<i64>> {
    let mut result = vec![Option::<i64>::None; priority_levels.len()];
    let mut next_precedence: i64 = 1;

    // Iterate in reverse (bottom to top)
    for i in (0..priority_levels.len()).rev() {
        let has_recursive = priority_levels[i]
            .alternatives
            .iter()
            .any(|alt| classify(alt, recursive_name, recursive_names) != RecursiveEnds::None);
        if has_recursive {
            result[i] = Some(next_precedence);
            next_precedence += 1;
        }
    }

    result
}

/// Desugars a single rule. `recursive_name` is the nonterminal that appears at
/// left/right recursive ends — for direct recursion this equals the head name,
/// for indirect recursion (e.g., exclude-derived rules) it's a different nonterminal.
fn desugar_rule(
    rule: SyntaxRule,
    recursive_name: &str,
    recursive_names: &[(String, String)],
) -> SyntaxRule {
    let head_name = rule.head.name.clone();
    let precedences = assign_precedence(&rule.priority_levels, recursive_name, recursive_names);

    // Find the resolved DefinitionId for the recursive nonterminal from any reference in the
    // alternatives. Identifiers are already resolved at this point in the pipeline.
    let head_def = find_definition_id(&rule.priority_levels, recursive_name)
        .expect("desugared nonterminal should have at least one recursive reference");

    // Find the minimum precedence among prefix (Right-only) alternatives.
    // This determines which alternatives need the min trick.
    let min_prefix_pr = min_prefix_precedence(
        &rule.priority_levels,
        &precedences,
        recursive_name,
        recursive_names,
    );

    let mut all_alternatives = Vec::new();

    for (level, precedence) in rule.priority_levels.into_iter().zip(precedences.iter()) {
        let assoc = level.associativity;
        for alt in level.alternatives {
            let recursion = classify(&alt, recursive_name, recursive_names);
            let rewritten = match (recursion, precedence) {
                (RecursiveEnds::Binary, Some(pr)) => {
                    rewrite_binary(recursive_name, head_def, alt, *pr, assoc, min_prefix_pr)
                }
                (RecursiveEnds::Right, Some(pr)) => {
                    rewrite_prefix(recursive_name, head_def, alt, *pr, min_prefix_pr)
                }
                (RecursiveEnds::Left, Some(pr)) => {
                    rewrite_postfix(recursive_name, head_def, alt, *pr)
                }
                (RecursiveEnds::None, _) => rewrite_non_recursive(recursive_name, alt),
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

    SyntaxRule {
        head,
        priority_levels: vec![PriorityLevel::new(all_alternatives)],
        layout: rule.layout,
    }
}

/// Finds the minimum precedence among prefix (Right-only recursive) alternatives.
/// Returns `None` if there are no prefix alternatives.
fn min_prefix_precedence(
    priority_levels: &[PriorityLevel],
    precedences: &[Option<i64>],
    recursive_name: &str,
    recursive_names: &[(String, String)],
) -> Option<i64> {
    priority_levels
        .iter()
        .zip(precedences.iter())
        .filter_map(|(level, prec)| {
            let pr = (*prec)?;
            let has_prefix = level
                .alternatives
                .iter()
                .any(|alt| classify(alt, recursive_name, recursive_names) == RecursiveEnds::Right);
            has_prefix.then_some(pr)
        })
        .min()
}

/// Creates the return expression for the min trick:
///   `{r == 0 ? pr : min(r, pr)}`
fn make_min_return(pr: i64) -> Symbol {
    Symbol::Return(Expr::Ternary {
        cond: Box::new(Expr::Cond(Cond {
            left: Box::new(Expr::Ref("r".to_string())),
            right: Box::new(Expr::Int(0)),
            op: CondOp::Eq,
        })),
        then: Box::new(Expr::Int(pr)),
        r#else: Box::new(Expr::Min(
            Box::new(Expr::Ref("r".to_string())),
            Box::new(Expr::Int(pr)),
        )),
    })
}

/// Creates a right-end binding: `r=E(arg)`
fn make_right_binding(head_name: &str, head_def: DefinitionId, arg: i64) -> Symbol {
    Symbol::Binding {
        name: "r".to_string(),
        symbol: Box::new(Symbol::Call {
            name: Identifier {
                name: head_name.to_string(),
                definition: Some(head_def),
            },
            arguments: vec![Expr::Int(arg)],
        }),
    }
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
fn rewrite_binary(head_name: &str, head_def: DefinitionId, alt: Alternative, pr: i64, assoc: Option<Associativity>, min_prefix_pr: Option<i64>) -> Alternative {
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

    // The min trick is needed when there is a prefix alternative at a lower
    // precedence level than this binary alternative.
    let use_min = min_prefix_pr.is_some_and(|min_pr| pr > min_pr);

    let mut symbols = Vec::new();

    symbols.push(make_precondition(pr));
    symbols.push(make_left_binding(head_name, head_def));
    symbols.push(make_postcondition(postcond_threshold));

    // Middle symbols (everything except the first and last, which are the recursive E references)
    let num_symbols = alt.symbols.len();
    for symbol in alt.symbols.into_iter().skip(1).take(num_symbols.saturating_sub(2)) {
        symbols.push(symbol);
    }

    if use_min {
        // r=E(right_arg)
        symbols.push(make_right_binding(head_name, head_def, right_arg));
        // {r==0 ? pr : min(r, pr)}
        symbols.push(make_min_return(pr));
    } else {
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
    }

    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Rewrites a prefix alternative `op E` at precedence level `pr` into:
///   op E(pr) {pr}
/// Or with the min trick (when a prefix exists at lower precedence):
///   op r=E(pr) {r==0 ? pr : min(r, pr)}
fn rewrite_prefix(head_name: &str, head_def: DefinitionId, alt: Alternative, pr: i64, min_prefix_pr: Option<i64>) -> Alternative {
    let mut symbols = Vec::new();
    let num_symbols = alt.symbols.len();

    // The min trick is needed when there is a prefix alternative at a lower
    // precedence level than this prefix alternative.
    let use_min = min_prefix_pr.is_some_and(|min_pr| pr > min_pr);

    // All symbols except the last (right-end E), with any E references replaced by E(0)
    for symbol in alt.symbols.into_iter().take(num_symbols.saturating_sub(1)) {
        symbols.push(replace_head_ref(symbol, head_name));
    }

    if use_min {
        // r=E(pr)
        symbols.push(make_right_binding(head_name, head_def, pr));
        // {r==0 ? pr : min(r, pr)}
        symbols.push(make_min_return(pr));
    } else {
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
    }

    Alternative {
        symbols,
        label: alt.label,
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
    }
}

/// Updates references to desugared nonterminals in non-desugared rules:
/// `E` becomes `E(0)`.
fn update_external_references(rule: SyntaxRule, desugared_names: &[String]) -> SyntaxRule {
    transform_syntax_rule(rule, |symbol| {
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
        alternative, bind, call, cond, cond_expr,
        grammar::def::{Grammar, GrammarDef},
        grammar_def, id, left, lit, min, non_assoc, priority_level, ret, right, syntax_rule,
        ternary,
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
                    priority_level!(left!(); alternative!(id!("E"), lit!("+"), id!("E")))
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
                    priority_level!(right!(); alternative!(id!("E"), lit!(";"), id!("E")))
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
                    priority_level!(non_assoc!(); alternative!(id!("E"), lit!("<"), id!("E")))
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
                    priority_level!(left!();
                        alternative!(id!("E"), lit!("*"), id!("E")),
                        alternative!(id!("E"), lit!("/"), id!("E"))
                    ),
                    priority_level!(left!();
                        alternative!(id!("E"), lit!("+"), id!("E")),
                        alternative!(id!("E"), lit!("-"), id!("E"))
                    ),
                    priority_level!(non_assoc!();
                        alternative!(id!("E"), lit!("<"), id!("E"))
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

    /// Deep case (min trick): prefix at lower precedence than binary.
    ///   E = 'a'
    ///     > E '+' E
    ///     > 'if' E 'then' E 'else' E
    ///
    /// Precedences: 'if-then-else': 1 (prefix), '+': 2 (binary)
    /// The '+' alternative needs the min trick because there's a prefix at level 1.
    /// The 'if-then-else' does NOT need the min trick (no prefix below it).
    ///
    /// Expected:
    ///   E(p)
    ///     = 'a' {0}
    ///     | [2>=p] l=E(p) [l==0||l>=2] '+' r=E(2) {r==0 ? 2 : min(r,2)}
    ///     | 'if' E(0) 'then' E(0) 'else' E(1) {1}
    #[test]
    fn test_min_trick_deep_case() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E"))
                    ),
                    priority_level!(
                        alternative!(lit!("if"), id!("E"), lit!("then"), id!("E"), lit!("else"), id!("E"))
                    )
                ),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("+"),
                        bind!("r", call!("E", 2)),
                        ret!(expr ternary!(cond_expr!("r" == 0), 2, min!("r", 2))),
                    ),
                    alternative!(
                        lit!("if"),
                        call!("E", 0),
                        lit!("then"),
                        call!("E", 0),
                        lit!("else"),
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

    /// Min trick with prefix at higher precedence: no min trick needed.
    ///   E = 'a' > '-' E > E '+' E
    ///
    /// Precedences (bottom=1): '+': 1, '-': 2 (prefix above binary)
    /// No min trick because the prefix is ABOVE the binary.
    #[test]
    fn test_no_min_trick_prefix_above() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(
                        alternative!(lit!("-"), id!("E"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E"))
                    )
                ),
            ]
        );

        // No min trick: prefix '-' at level 2 is above binary '+' at level 1
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        lit!("-"),
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
        );

        let actual: Grammar = input.into();
        let expected: Grammar = expected.into();
        assert_eq!(actual, expected);
    }

    /// Min trick with multiple operators (closer to Figure 4 from PEPM'16).
    ///   E = 'a'
    ///     > E '*' E  left
    ///     > E '+' E  left
    ///     > '-' E
    ///     > 'if' E 'then' E 'else' E
    ///     > E ';' E  right
    ///
    /// Precedences: ';':1, 'if':2, '-':3, '+':4 left, '*':5 left
    /// min_prefix_pr = 2 (the 'if-then-else')
    /// Min trick applies to: '*'(5>2), '+'(4>2), '-'(3>2)
    /// No min trick for: 'if'(2=2), ';'(1<2)
    #[test]
    fn test_min_trick_multiple_operators() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!();
                        alternative!(id!("E"), lit!("*"), id!("E"))
                    ),
                    priority_level!(left!();
                        alternative!(id!("E"), lit!("+"), id!("E"))
                    ),
                    priority_level!(
                        alternative!(lit!("-"), id!("E"))
                    ),
                    priority_level!(
                        alternative!(lit!("if"), id!("E"), lit!("then"), id!("E"), lit!("else"), id!("E"))
                    ),
                    priority_level!(right!();
                        alternative!(id!("E"), lit!(";"), id!("E"))
                    )
                ),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    // '*' left at level 5: min trick (5 > 2)
                    alternative!(
                        cond!(5 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 5)),
                        lit!("*"),
                        bind!("r", call!("E", 6)),
                        ret!(expr ternary!(cond_expr!("r" == 0), 5, min!("r", 5))),
                    ),
                    // '+' left at level 4: min trick (4 > 2)
                    alternative!(
                        cond!(4 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 4)),
                        lit!("+"),
                        bind!("r", call!("E", 5)),
                        ret!(expr ternary!(cond_expr!("r" == 0), 4, min!("r", 4))),
                    ),
                    // '-' prefix at level 3: min trick (3 > 2)
                    alternative!(
                        lit!("-"),
                        bind!("r", call!("E", 3)),
                        ret!(expr ternary!(cond_expr!("r" == 0), 3, min!("r", 3))),
                    ),
                    // 'if-then-else' prefix at level 2: NO min trick (2 == 2, not > 2)
                    alternative!(
                        lit!("if"),
                        call!("E", 0),
                        lit!("then"),
                        call!("E", 0),
                        lit!("else"),
                        call!("E", 2),
                        ret!(2),
                    ),
                    // ';' right-assoc at level 1: NO min trick (1 < 2)
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

    /// Indirect recursion: F is a filtered copy of E (e.g., from exclude desugaring).
    /// F's alternatives reference E at their ends, making F indirectly recursive with
    /// recursive name E. When E has an alternative with F at the left end, classify
    /// must recognize F as an indirect reference to E.
    ///
    ///   E = 'a' > F '*' 'b' > E '+' E
    ///   F = 'a' > E '+' E
    ///
    /// F is indirectly recursive (recursive name = E). E has F at the left end of
    /// `F '*' 'b'`, which should be classified as Left (postfix).
    ///
    /// Precedences for E: '+': 1, '*': 2
    /// Precedences for F: '+': 1
    #[test]
    fn test_indirect_recursion() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(
                        alternative!(lit!("a"))
                    ),
                    priority_level!(
                        alternative!(id!("F"), lit!("*"), lit!("b"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E"))
                    )
                ),
                syntax_rule!("F" =>
                    priority_level!(
                        alternative!(lit!("a"))
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E"))
                    )
                ),
            ]
        );

        // E(p):
        //   'a' {0}
        //   [2>=p] l=E(p) [l==0||l>=2] '*' 'b' {0}    -- F recognized as left-recursive
        //   [1>=p] l=E(p) [l==0||l>=1] '+' E(1) {1}
        //
        // F(p):
        //   'a' {0}
        //   [1>=p] l=E(p) [l==0||l>=1] '+' E(1) {1}
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("*"),
                        lit!("b"),
                        ret!(0),
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
                syntax_rule!("F"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
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
        );

        let actual: Grammar = input.into();
        let expected: Grammar = expected.into();
        assert_eq!(actual, expected);
    }
}
