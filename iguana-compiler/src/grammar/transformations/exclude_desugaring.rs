// Exclude desugaring.
//
// In an iggy grammar, `Expr !Comma` reads as "match Expr, but admit no
// alternative labeled #Comma at the top". The user writes this when the
// surrounding context restricts which forms of an expression are valid
// at that position. The exclude desugaring transformation encodes the
// restriction into the grammar: every alternative of `Expr` returns its
// label, and a condition after the call rejects an excluded label. Because
// the check follows the call, calls to `Expr` at a position with the same
// arguments share one GSS node, whatever they exclude.
//
// The implementation mechanism is a bitmask. The desugaring numbers the
// labeled alternatives of each target, and an excluding reference names its
// excluded labels as a single i32 (with the sign bit unused, giving 31
// labels per nonterminal). The condition after the call tests the bit of the
// returned label in that mask.
//
// Consider this grammar:
//
//   Expr
//     = Id                     #Id
//     | Expr "," Expr          #Comma
//     | "(" Expr !Comma ")"    #Group
//
// The parenthesized form forbids a comma at the top. After this
// transformation:
//
//   Expr
//     = Id return (0, 0)                                                                                #Id
//     | Expr "," Expr return (0, 1)                                                                     #Comma
//     | "(" (x_pr, x_label)=Expr [(x_label == NO_LABEL) || ((2 >> x_label) & 1 == 0)] ")" return (0, 2) #Group
//
// What the desugaring does:
//
//   - Bit assignment. The desugaring numbers the labeled alternatives in
//     source order: #Id is 0, #Comma is 1, #Group is 2. Each number plays
//     two roles. It becomes the return's label field, and it is the
//     position of the label's bit in a mask, `1 << N`.
//
//   - Return value. Every alternative of an exclusion target returns its
//     own label, or NO_LABEL if unlabeled, as the last return component.
//     Distinct returned labels keep alternatives distinguishable in the
//     parser's sharing key.
//
//   - Label check. Every excluding reference is bound to its returned
//     tuple and followed by a condition on the bound label.
//     The condition admits the result if the label is NO_LABEL or its bit
//     is clear in the mask: `Expr !Comma` has the mask 2, and
//     `Expr !Comma !Group` has the mask 6.
//
// Tracing the check. The call to `Expr` inside #Group binds `x_label` to the
// label of the alternative the call derived. For #Id, `(2 >> 0) & 1` is 0
// and the result is admitted. For #Comma, `(2 >> 1) & 1` is 1 and the
// result is rejected.
//
// Exclusions and precedence. An exclusion target may also have precedence
// levels:
//
//   E = "a"           #Lit
//     > E "*" E       #Mul
//     > E "+" E       #Add
//   S = E !Add
//
// In `S`, `E` cannot be derived through the alternative labeled `Add`.
// The desugared grammar below shows returns as `(p, label)` pairs, where `p`
// is the returned precedence and `label` is the chosen alternative's label index.
// The indices for `Lit`, `Mul`, and `Add` are 0, 1, and 2, so `E !Add` has
// the mask 4. The binding `(l_pr, l_label)` names the precedence and label
// returned by the left recursive call. Conditions use these names directly.
//
//   E(p: i32)
//     = "a"                                                             return (0, 0) #Lit
//     | [2 >= p] (l_pr, l_label)=E(p) [l_pr == 0 || l_pr >= 2] "*" E(2) return (2, 1) #Mul
//     | [1 >= p] (l_pr, l_label)=E(p) [l_pr == 0 || l_pr >= 1] "+" E(1) return (1, 2) #Add
//
//   S
//     = (x_pr, x_label)=E(0) [(x_label == NO_LABEL) || ((4 >> x_label) & 1 == 0)]
//
// The call in `S` leaves precedence unrestricted, and the condition after it
// rejects a returned `Add`. Multiplication returns `(2, 1)`: precedence 2 and
// the label index for `Mul`. Precedence conditions read the first component,
// and exclusion checks read the second.
//
// Pipeline position. The exclude desugaring transformation runs in two parts
// around the precedence desugaring transformation. The first part, exclusion
// labels, numbers the labels and adds the returns. It leaves each excluding
// reference in place. The second part, exclusion checks, binds each excluding
// reference to its returned tuple and adds the condition after it.
//
// The second part runs after the precedence desugaring, because the
// precedence desugaring decides the final form of a call. It binds a
// recursive end to names such as `(l_pr, l_label)`, which its own conditions
// read, and it decides whether the returned tuple has an associativity
// component. The label check reuses such a binding. The second part also
// runs after EBNF expansion. EBNF wrappers are not recursed through, so an
// exclusion nested inside one would be silently skipped if the order were
// reversed.

use super::return_value::{self, ReturnShape};
use rustc_hash::FxHashMap;

use crate::grammar::{
    def::{Alternative, PriorityLevel, SyntaxRule},
    symbols::{BindingPattern, Cond, CondOp, Expr, Restrictions, Symbol},
    transformations::{transform_rule_by_symbols, visit_syntax_rule},
};

/// A map from each exclusion target, such as `E` in `E !Add`, to its labels
/// in bit order.
pub type ExclusionTargets = FxHashMap<String, Vec<String>>;

/// Numbers the labels of each exclusion target and appends a return with its
/// label to each of its alternatives. Excluding references stay in place for
/// `check_returned_labels`.
pub fn transform(syntax_rules: Vec<SyntaxRule>) -> (Vec<SyntaxRule>, ExclusionTargets) {
    let mut targets = ExclusionTargets::default();
    for rule in &syntax_rules {
        visit_syntax_rule(rule, &mut |symbol| {
            if let Symbol::Exclude { symbol, .. } = symbol {
                let Symbol::Identifier(id) = symbol.as_ref() else {
                    panic!("an exclusion applies only to a nonterminal reference, got {symbol}");
                };
                targets.entry(id.name.clone()).or_default();
            }
        });
    }

    if targets.is_empty() {
        return (syntax_rules, targets);
    }

    for rule in &syntax_rules {
        let Some(labels) = targets.get_mut(&rule.head.name) else {
            continue;
        };
        for alt in rule.alternatives() {
            if let Some(label) = &alt.label
                && !labels.contains(label)
            {
                assert!(
                    labels.len() < 31,
                    "exclude-targeted nonterminal {} has more than 31 labels",
                    rule.head.name
                );
                labels.push(label.clone());
            }
        }
    }

    let syntax_rules = syntax_rules
        .into_iter()
        .map(|rule| match targets.get(&rule.head.name) {
            Some(labels) => add_label_returns(rule, labels),
            None => rule,
        })
        .collect();
    (syntax_rules, targets)
}

fn add_label_returns(mut rule: SyntaxRule, labels: &[String]) -> SyntaxRule {
    rule.priority_levels = rule
        .priority_levels
        .into_iter()
        .map(|pl| PriorityLevel {
            alternatives: pl
                .alternatives
                .into_iter()
                .map(|alt| add_label_return(alt, labels))
                .collect(),
            associativity: pl.associativity,
        })
        .collect();
    rule
}

/// Appends a return with precedence zero and the alternative's label.
/// An unlabeled alternative returns `(0, NO_LABEL)`.
fn add_label_return(alt: Alternative, labels: &[String]) -> Alternative {
    let label_index = alt
        .label
        .as_ref()
        .and_then(|label| labels.iter().position(|l| l == label));
    let mut symbols = alt.symbols;
    symbols.push(Symbol::Return(return_value::with_label(
        Expr::Int(0),
        label_index,
    )));
    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Binds each excluding reference, such as `E !Add` in `S = E !Add`, to its
/// returned tuple, followed by a check of the returned label:
///
/// ```text
///   S = (x_pr, x_label)=E(0) [(x_label == NO_LABEL) || ((4 >> x_label) & 1 == 0)]
/// ```
///
/// A call already bound to a tuple, such as the left end
/// `(l_pr, l_label)=E(p)`, keeps its binding, and the check reads its label.
/// Other references are bound with the prefix `x`. A reference bound to a
/// single name is an error, because its label has no name to check. Runs
/// after precedence desugaring, which decides the arguments, the bindings, and
/// the tuple shape of each call.
pub fn check_returned_labels(
    syntax_rules: Vec<SyntaxRule>,
    targets: &ExclusionTargets,
) -> Result<Vec<SyntaxRule>, Vec<String>> {
    if targets.is_empty() {
        return Ok(syntax_rules);
    }
    let shapes: FxHashMap<String, ReturnShape> = syntax_rules
        .iter()
        .filter(|rule| targets.contains_key(&rule.head.name))
        .map(|rule| (rule.head.name.clone(), return_shape(rule)))
        .collect();
    let mut errors = vec![];
    let syntax_rules = syntax_rules
        .into_iter()
        .map(|rule| {
            transform_rule_by_symbols(rule, |symbols| {
                symbols
                    .into_iter()
                    .flat_map(|symbol| {
                        check_returned_label(symbol, targets, &shapes).unwrap_or_else(|error| {
                            errors.push(error);
                            vec![]
                        })
                    })
                    .collect()
            })
        })
        .collect();
    if errors.is_empty() {
        Ok(syntax_rules)
    } else {
        Err(errors)
    }
}

/// The return shape of an exclusion target, read from the return of its
/// first alternative. Every alternative of a rule returns the same
/// components, and the label is the last of them.
fn return_shape(rule: &SyntaxRule) -> ReturnShape {
    let first = rule.alternatives().next().unwrap();
    let Some(Symbol::Return(Expr::Tuple(values))) = first.symbols.last() else {
        panic!(
            "exclusion target {} does not return a label",
            rule.head.name
        );
    };
    ReturnShape {
        associativity: values.len() == 3,
        label: true,
    }
}

/// The symbols for `symbol`: the symbol itself if it does not exclude
/// anything, or the call bound to its returned tuple followed by a check of
/// its returned label.
fn check_returned_label(
    symbol: Symbol,
    targets: &ExclusionTargets,
    shapes: &FxHashMap<String, ReturnShape>,
) -> Result<Vec<Symbol>, String> {
    if !excludes(&symbol) {
        return Ok(vec![symbol]);
    }
    let reference = Reference::from(symbol);
    let excluded = reference.excluded.as_ref().unwrap();
    let target = reference
        .core
        .as_identifier()
        .expect("an exclusion applies only to a nonterminal reference")
        .name
        .clone();
    let labels = &targets[&target];
    let mask = excluded.iter().fold(0i64, |mask, label| {
        let bit = labels
            .iter()
            .position(|l| l == label)
            .unwrap_or_else(|| panic!("Label {label} not defined on {target}"));
        mask | (1i64 << bit)
    });
    let pattern = match reference.pattern {
        Some(BindingPattern::Tuple(names)) => names,
        Some(BindingPattern::Name(name)) => {
            return Err(format!(
                "the excluding reference to `{target}` is bound to the single name `{name}`, which leaves no name for its label"
            ));
        }
        None => shapes[&target].binding("x").names().to_vec(),
    };
    // The label is the last component of the returned tuple.
    let check = exclusion_postcondition(mask, pattern.last().unwrap());
    let reference = Reference {
        pattern: Some(BindingPattern::Tuple(pattern)),
        excluded: None,
        ..reference
    };
    Ok(vec![reference.into_symbol(), check])
}

/// Whether `symbol` is an exclusion, possibly inside restrictions, a binding,
/// or a field label.
fn excludes(symbol: &Symbol) -> bool {
    match symbol {
        Symbol::Exclude { .. } => true,
        Symbol::Restricted { symbol, .. }
        | Symbol::Binding { symbol, .. }
        | Symbol::Labeled { symbol, .. } => excludes(symbol),
        _ => false,
    }
}

/// A nonterminal reference taken apart into its wrappers. `from` accepts the
/// wrappers in any order. `into_symbol` rebuilds them in the order precedence
/// desugaring builds a call: the restrictions outermost, then the binding, the
/// field label, the exclusion, and the call.
struct Reference {
    restrictions: Restrictions,
    pattern: Option<BindingPattern>,
    label: Option<String>,
    excluded: Option<Vec<String>>,
    core: Symbol,
}

impl Reference {
    fn from(mut symbol: Symbol) -> Self {
        let mut restrictions = Restrictions::default();
        let mut pattern = None;
        let mut label = None;
        let mut excluded = None;
        loop {
            symbol = match symbol {
                Symbol::Restricted {
                    symbol,
                    restrictions: r,
                } => {
                    restrictions = r;
                    *symbol
                }
                Symbol::Binding { pattern: p, symbol } => {
                    pattern = Some(p);
                    *symbol
                }
                Symbol::Labeled { label: l, symbol } => {
                    label = Some(l);
                    *symbol
                }
                Symbol::Exclude { symbol, labels } => {
                    excluded = Some(labels);
                    *symbol
                }
                core => {
                    return Self {
                        restrictions,
                        pattern,
                        label,
                        excluded,
                        core,
                    };
                }
            }
        }
    }

    fn into_symbol(self) -> Symbol {
        let mut symbol = self.core;
        if let Some(labels) = self.excluded {
            symbol = Symbol::Exclude {
                symbol: Box::new(symbol),
                labels,
            };
        }
        if let Some(label) = self.label {
            symbol = Symbol::Labeled {
                label,
                symbol: Box::new(symbol),
            };
        }
        if let Some(pattern) = self.pattern {
            symbol = Symbol::Binding {
                pattern,
                symbol: Box::new(symbol),
            };
        }
        Symbol::restricted(symbol, self.restrictions)
    }
}

/// Admits the returned result if the bound `label` is `NO_LABEL` or its bit
/// is clear in `mask`.
fn exclusion_postcondition(mask: i64, label: &str) -> Symbol {
    let label = || Expr::Ref(label.to_string());
    Symbol::Condition(Expr::Or(
        Box::new(Expr::Cond(Cond {
            left: Box::new(label()),
            right: Box::new(return_value::no_label()),
            op: CondOp::Eq,
        })),
        Box::new(Expr::Cond(Cond {
            left: Box::new(Expr::BitAnd(
                Box::new(Expr::Shr(Box::new(Expr::Int(mask)), Box::new(label()))),
                Box::new(Expr::Int(1)),
            )),
            right: Box::new(Expr::Int(0)),
            op: CondOp::Eq,
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::def::Grammar;
    use crate::{
        alternative, bind, cond, eq, exclude, follow, ge, grammar_def, id, labeled, lit, or,
        priority_level, ret, syntax_rule, tuple,
    };

    use itertools::Itertools;

    // Grammar comments show one alternative per line. Every alternative of a
    // rule has the same return shape. Bindings name the returned components as
    // l_pr, l_assoc, and l_label, with matching right and shared-call names.
    // Generated start rules and literal regex definitions are omitted.
    fn precedence_ref(binding: &str) -> Expr {
        Expr::Ref(format!("{binding}_pr"))
    }

    // The alternatives of `name`, each printed as its symbols separated by spaces.
    fn printed_alternatives(grammar: &Grammar, name: &str) -> Vec<String> {
        let rule = grammar.nonterminal(name).unwrap();
        grammar
            .alternatives(rule)
            .iter()
            .map(|alt| alt.symbols.iter().join(" "))
            .collect()
    }

    // An excluding reference bound to a single name has no name for its label.
    // iggy cannot write a binding, so only the test macros build one.
    #[test]
    fn an_excluding_reference_bound_to_a_single_name_is_an_error() {
        let errors = Grammar::try_from(grammar_def!("Test", syntax: [
            syntax_rule!("E" => priority_level!(alternative!(lit!("a"); #blocked))),
            syntax_rule!("S" => priority_level!(alternative!(
                bind!("x", exclude!(id!("E"), "blocked"))
            )))
        ]))
        .unwrap_err();
        assert_eq!(
            errors,
            [
                "the excluding reference to `E` is bound to the single name `x`, which leaves no name for its label"
            ]
        );
    }

    // iggy puts restrictions around an exclusion, so only the test macros build
    // an exclusion around restrictions.
    #[test]
    #[should_panic(expected = "an exclusion applies only to a nonterminal reference")]
    fn an_exclusion_around_restrictions_panics() {
        transform(vec![
            syntax_rule!("E" => priority_level!(alternative!(lit!("a"); #blocked))),
            syntax_rule!("S" => priority_level!(alternative!(
                exclude!(follow!(id!("E"), "Z"), "blocked")
            ))),
        ]);
    }

    //   E = "a" #blocked
    //     | "b"
    //   S = E !blocked
    //
    // After the first part of exclusion desugaring:
    //
    //   E
    //     = "a" return (0, 0)        #blocked
    //     | "b" return (0, NO_LABEL)
    //   S = E !blocked
    #[test]
    fn unlabeled_alternative_returns_no_label_before_precedence_desugaring() {
        let (rules, targets) = transform(vec![
            syntax_rule!("E" => priority_level!(
                alternative!(lit!("a"); #blocked),
                alternative!(lit!("b"))
            )),
            syntax_rule!("S" => priority_level!(
                alternative!(exclude!(id!("E"), "blocked"))
            )),
        ]);
        let alternatives = &rules[0].priority_levels[0].alternatives;
        assert_eq!(
            alternatives[0]
                .symbols
                .iter()
                .map(Symbol::to_string)
                .collect::<Vec<_>>(),
            ["\"a\"", "return (0, 0)"]
        );
        assert_eq!(
            alternatives[1].symbols.last().unwrap().to_string(),
            "return (0, NO_LABEL)"
        );
        assert!(rules[0].head.parameters.is_empty());
        assert_eq!(
            rules[1].priority_levels[0].alternatives[0].symbols[0].to_string(),
            "E !blocked"
        );
        assert_eq!(targets["E"], ["blocked"]);
    }

    // In a grammar without precedence, an excluding reference is bound to its
    // returned tuple without becoming a call. This is the example in the
    // module comment.
    //
    // Original grammar:
    //
    //   Expr
    //     = Id                     #Id
    //     | Expr "," Expr          #Comma
    //     | "(" Expr !Comma ")"    #Group
    //
    // Desugared grammar:
    //
    //   Expr
    //     = Id return (0, 0)                                                                                #Id
    //     | Expr "," Expr return (0, 1)                                                                     #Comma
    //     | "(" (x_pr, x_label)=Expr [(x_label == NO_LABEL) || ((2 >> x_label) & 1 == 0)] ")" return (0, 2) #Group
    #[test]
    fn exclusion_without_precedence_binds_the_reference() {
        let source = "grammar G\nExpr = Id #Id | Expr \",\" Expr #Comma | \"(\" Expr !Comma \")\" #Group\n@Regex\nId = [a-z]+\n";
        let grammar: Grammar = crate::iggy::parse_grammar(source)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            printed_alternatives(&grammar, "Expr"),
            [
                "Id return (0, 0)",
                "Expr \",\" Expr return (0, 1)",
                "\"(\" (x_pr, x_label)=Expr [(x_label == NO_LABEL) || ((2 >> x_label) & 1 == 0)] \")\" return (0, 2)",
            ]
        );
    }

    // An exclusion inside an intermediate nonterminal checks the label after the shared call.
    //
    // First case:
    //
    // Original grammar:
    //
    //   E
    //     = "a" #atom
    //     > L "+" E #binary
    //
    //   L
    //     = E !atom
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return (0, 0) #atom
    //     | [1 >= p] l_pr=L(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(1) return (1, 1) #binary
    //
    //   L(p: i32)
    //     = (l_pr, l_label)=E(p) [(l_label == NO_LABEL) || ((1 >> l_label) & 1 == 0)] return l_pr
    //
    // Second case:
    //
    // Original grammar:
    //
    //   E
    //     = "a" #atom
    //     > L "+" L #binary
    //
    //   L
    //     = E !atom
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                            return (0, 0) #atom
    //     | [1 >= p] l_pr=L(p, 0) [(l_pr == 0) || (l_pr >= 1)] "+" L(1, 1) return (1, 1) #binary
    //
    //   L(p: i32, end: i32)
    //     = (v_pr, v_label)=E(p) [(v_label == NO_LABEL) || ((1 >> v_label) & 1 == 0)] return v_pr
    //
    #[test]
    fn indirect_left_exclusion_uses_a_shared_call_and_returned_label() {
        for both_ends in [false, true] {
            let right = if both_ends { id!("L") } else { id!("E") };
            let input = grammar_def!("Test", syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"); #atom)),
                    priority_level!(alternative!(id!("L"), lit!("+"), right; #binary))),
                syntax_rule!("L" => priority_level!(alternative!(exclude!(id!("E"), "atom"))))
            ]);
            let grammar: Grammar = input.try_into().unwrap();
            let l = grammar.nonterminal("L").unwrap();
            let symbols = &grammar.alternatives(l)[0].symbols;
            let binding = if both_ends { "v" } else { "l" };
            assert!(matches!(&symbols[0], Symbol::Binding { pattern, symbol }
                if pattern.names()[0] == format!("{binding}_pr") && matches!(symbol.as_ref(), Symbol::Call { arguments, .. }
                    if arguments == &[Expr::Ref("p".to_string())])));
            assert_eq!(
                symbols[1],
                cond!(or!(
                    eq!(
                        Expr::Ref(format!("{binding}_label")),
                        return_value::no_label()
                    ),
                    eq!(
                        Expr::BitAnd(
                            Box::new(Expr::Shr(
                                Box::new(Expr::Int(1)),
                                Box::new(Expr::Ref(format!("{binding}_label")))
                            )),
                            Box::new(Expr::Int(1))
                        ),
                        0
                    ),
                ))
            );
        }
    }

    // Every call with an exclusion checks the returned label.
    //
    // Original grammar:
    //
    //   E
    //     = "a"         #Lit
    //     > "-" E !Add  #Neg
    //     > E "+" E !Neg #Add
    //
    //   S
    //     = E !Add
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a" return (0, 0) #Lit
    //     | "-" (x_pr, x_label)=E(2) [(x_label == NO_LABEL) || ((4 >> x_label) & 1 == 0)] return (2, 1) #Neg
    //     | [1 >= p] (l_pr, l_label)=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" (x_pr, x_label)=E(1) [(x_label == NO_LABEL) || ((2 >> x_label) & 1 == 0)] return (1, 2) #Add
    //
    //   S
    //     = (x_pr, x_label)=E(0) [(x_label == NO_LABEL) || ((4 >> x_label) & 1 == 0)]
    #[test]
    fn exclusions_at_right_ends_and_other_rules_check_the_returned_label() {
        let grammar: Grammar = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"); #Lit)),
                priority_level!(alternative!(lit!("-"), exclude!(id!("E"), "Add"); #Neg)),
                priority_level!(alternative!(id!("E"), lit!("+"), exclude!(id!("E"), "Neg"); #Add))),
            syntax_rule!("S" => priority_level!(alternative!(exclude!(id!("E"), "Add"))))
        ])
        .try_into()
        .unwrap();
        assert_eq!(
            printed_alternatives(&grammar, "E"),
            [
                "\"a\" return (0, 0)",
                "\"-\" (x_pr, x_label)=E(2) [(x_label == NO_LABEL) || ((4 >> x_label) & 1 == 0)] return (2, 1)",
                "[1 >= p] (l_pr, l_label)=E(p) [(l_pr == 0) || (l_pr >= 1)] \"+\" (x_pr, x_label)=E(1) [(x_label == NO_LABEL) || ((2 >> x_label) & 1 == 0)] return (1, 2)",
            ]
        );
        assert_eq!(
            printed_alternatives(&grammar, "S"),
            ["(x_pr, x_label)=E(0) [(x_label == NO_LABEL) || ((4 >> x_label) & 1 == 0)]"]
        );
    }

    // An exclusion at a right end that is already bound checks the label of that binding.
    //
    // Original grammar:
    //
    //   E
    //     = "a"           #Lit
    //     > E "+" E !Neg  #Add
    //     > "-" E         #Neg
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a" return (0, 0) #Lit
    //     | [2 >= p] (l_pr, l_label)=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" (r_pr, r_label)=E(2) [(r_label == NO_LABEL) || ((4 >> r_label) & 1 == 0)] return ((r_pr == 0) ? 2 : min(r_pr, 2), 1) #Add
    //     | "-" E(1) return (1, 2) #Neg
    //
    // The prefix `#Neg` has a lower precedence than `#Add`, so the right end of
    // `#Add` is bound as `r` to compute the returned precedence with `min`.
    #[test]
    fn exclusion_at_a_bound_right_end_checks_its_label() {
        let grammar: Grammar = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"); #Lit)),
                priority_level!(alternative!(id!("E"), lit!("+"), exclude!(id!("E"), "Neg"); #Add)),
                priority_level!(alternative!(lit!("-"), id!("E"); #Neg)))
        ])
        .try_into()
        .unwrap();
        assert_eq!(
            printed_alternatives(&grammar, "E"),
            [
                "\"a\" return (0, 0)",
                "[2 >= p] (l_pr, l_label)=E(p) [(l_pr == 0) || (l_pr >= 2)] \"+\" (r_pr, r_label)=E(2) [(r_label == NO_LABEL) || ((4 >> r_label) & 1 == 0)] return ((r_pr == 0) ? 2 : min(r_pr, 2), 1)",
                "\"-\" E(1) return (1, 2)",
            ]
        );
    }

    // A call bound for its exclusion check keeps its field label and restrictions.
    //
    // Original grammar:
    //
    //   E
    //     = "a"        #Lit
    //     > E "+" E    #Add
    //
    //   S
    //     = x:E !Add !>> Z
    //
    //   Z
    //     = "z"
    //
    // Desugared `S`:
    //
    //   S
    //     = (x_pr, x_label)=x:E(0) !>> Z [(x_label == NO_LABEL) || ((2 >> x_label) & 1 == 0)]
    #[test]
    fn exclusion_check_keeps_the_field_label_and_restrictions() {
        let grammar: Grammar = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"); #Lit)),
                priority_level!(alternative!(id!("E"), lit!("+"), id!("E"); #Add))),
            syntax_rule!("S" => priority_level!(alternative!(
                labeled!("x", follow!(exclude!(id!("E"), "Add"), "Z"))
            ))),
            syntax_rule!("Z" => priority_level!(alternative!(lit!("z"))))
        ])
        .try_into()
        .unwrap();
        assert_eq!(
            printed_alternatives(&grammar, "S"),
            ["(x_pr, x_label)=x:E(0) !>> Z [(x_label == NO_LABEL) || ((2 >> x_label) & 1 == 0)]"]
        );
    }

    // A left-end exclusion checks the returned label separately from the precedence.
    //
    // Original grammar:
    //
    //   E
    //     = "a" #A
    //     > E !B "++" #C
    //     > E "+" E #D
    //     > E "-" E #B
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                                                                                     return (0, 0) #A
    //     | [3 >= p] (l_pr, l_label)=E(p) [(l_label == NO_LABEL) || ((8 >> l_label) & 1 == 0)] [(l_pr == 0) || (l_pr >= 3)] "++"    return (0, 1) #C
    //     | [2 >= p] (l_pr, l_label)=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" E(2)                                                     return (2, 2) #D
    //     | [1 >= p] (l_pr, l_label)=E(p) [(l_pr == 0) || (l_pr >= 1)] "-" E(1)                                                     return (1, 3) #B
    //
    #[test]
    fn test_exclusions_at_left_recursive_ends() {
        use crate::grammar::symbols::{Cond, CondOp, Expr, Identifier, Symbol};

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(
                        alternative!(lit!("a"); #A)
                    ),
                    priority_level!(
                        alternative!(exclude!(id!("E"), "B"), lit!("++"); #C)
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("+"), id!("E"); #D)
                    ),
                    priority_level!(
                        alternative!(id!("E"), lit!("-"), id!("E"); #B)
                    )
                ),
            ]
        );

        // `E(arg0)`.
        fn e_call(arg0: Expr) -> Symbol {
            Symbol::Call {
                name: Identifier {
                    name: "E".to_string(),
                    definition: None,
                },
                arguments: vec![arg0],
            }
        }

        // Exclusion postcondition for the left recursive end:
        // `[l_label == NO_LABEL || ((mask >> l_label) & 1) == 0]`.
        fn left_end_postcondition(mask: i64) -> Symbol {
            let l_label = || Expr::Ref("l_label".to_string());
            cond!(Expr::Or(
                Box::new(Expr::Cond(Cond {
                    left: Box::new(l_label()),
                    right: Box::new(return_value::no_label()),
                    op: CondOp::Eq,
                })),
                Box::new(Expr::Cond(Cond {
                    left: Box::new(Expr::BitAnd(
                        Box::new(Expr::Shr(Box::new(Expr::Int(mask)), Box::new(l_label()))),
                        Box::new(Expr::Int(1)),
                    )),
                    right: Box::new(Expr::Int(0)),
                    op: CondOp::Eq,
                })),
            ))
        }

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(expr tuple!(0, 0)); #A),
                    // #C: postfix at pr=3, left excludes B (bit 8).
                    alternative!(
                        cond!(3 >= "p"),
                        bind!(("l_pr", "l_label"), e_call(Expr::Ref("p".to_string()))),
                        left_end_postcondition(8),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 3),
                        )),
                        lit!("++"),
                        ret!(expr tuple!(0, 1));
                        #C
                    ),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!(("l_pr", "l_label"), e_call(Expr::Ref("p".to_string()))),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("+"),
                        e_call(Expr::Int(2)),
                        ret!(expr tuple!(2, 2));
                        #D
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!(("l_pr", "l_label"), e_call(Expr::Ref("p".to_string()))),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("-"),
                        e_call(Expr::Int(1)),
                        ret!(expr tuple!(1, 3));
                        #B
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }
}
