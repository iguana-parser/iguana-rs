//! Scalar and tuple returns constructed by precedence and exclusion desugaring.
//!
//! Precedence comes first, followed by associativity and the exclusion label when
//! needed. Each rule returns the same components from all its alternatives.
//! The transformations choose these components and their binding names; the IR
//! represents them as ordinary scalar values, tuples, and references.

use crate::grammar::symbols::{BindingPattern, Expr, Symbol};

/// The components a precedence rule returns after its precedence value.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ReturnShape {
    pub associativity: bool,
    pub label: bool,
}

impl ReturnShape {
    pub fn value(self, precedence: Expr, associativity: Expr, label: Option<usize>) -> Expr {
        let mut values = vec![precedence];
        if self.associativity {
            values.push(associativity);
        }
        if self.label {
            values.push(label.map_or_else(no_label, |label| Expr::Int(label as i64)));
        }
        if values.len() == 1 {
            values.pop().unwrap()
        } else {
            Expr::Tuple(values)
        }
    }

    pub fn binding(self, prefix: &str) -> BindingPattern {
        let mut names = vec![format!("{prefix}_pr")];
        if self.associativity {
            names.push(format!("{prefix}_assoc"));
        }
        if self.label {
            names.push(format!("{prefix}_label"));
        }
        if names.len() == 1 {
            BindingPattern::Name(names.pop().unwrap())
        } else {
            BindingPattern::Tuple(names)
        }
    }
}

/// Constructs the pair returned by exclusion desugaring, including unlabeled alternatives.
pub fn with_label(precedence: Expr, label: Option<usize>) -> Expr {
    ReturnShape {
        associativity: false,
        label: true,
    }
    .value(precedence, Expr::Int(0), label)
}

/// Removes the return added by exclusion desugaring and reads this rule's label.
/// An unlabeled alternative also has an exclusion return, with `NO_LABEL`.
pub fn take_exclusion_return(symbols: &mut Vec<Symbol>) -> Option<usize> {
    let Some(Symbol::Return(Expr::Tuple(values))) = symbols.last() else {
        return None;
    };
    let label = match values.as_slice() {
        [_, Expr::Int(label)] => Some(*label as usize),
        [_, label] if *label == no_label() => None,
        _ => return None,
    };
    symbols.pop();
    label
}

/// The precedence returned when an indirect recursive end is absent.
pub fn undefined_precedence() -> Expr {
    Expr::DisplayInt {
        name: "UNDEFINED_PRECEDENCE".into(),
        value: -1,
    }
}

/// Exclusions use a 32-bit mask. Label indices 0 through 30 leave 31 for no label.
pub fn no_label() -> Expr {
    Expr::DisplayInt {
        name: "NO_LABEL".into(),
        value: 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{symbols::Symbol, transformations::visit_symbol};

    // Exclusion desugaring gives both alternatives a return:
    //
    //   E(e)
    //     = [1 & e == 0] "a" return (0, 0)
    //     | "b"             return (0, NO_LABEL)
    //
    // Precedence desugaring removes either return before rewriting the alternative.
    #[test]
    fn exclusion_returns_are_removed_from_labeled_and_unlabeled_alternatives() {
        for label in [Some(0), None] {
            let mut symbols = vec![Symbol::Return(with_label(Expr::Int(0), label))];
            assert_eq!(take_exclusion_return(&mut symbols), label);
            assert!(symbols.is_empty());
        }
        let mut symbols = vec![Symbol::Return(Expr::Int(0))];
        assert_eq!(take_exclusion_return(&mut symbols), None);
        assert_eq!(symbols, vec![Symbol::Return(Expr::Int(0))]);
    }

    // Original grammar:
    //
    //   E = "a"
    //     > left L "+" R
    //
    //   L
    //     = E
    //     | "b"
    //
    //   R
    //     = E
    //     | "b"
    //
    // Desugared grammar:
    //
    //   E(p: i32, a: i32)
    //     = "a"                                                                                                                                                                                                                                 return 0
    //     | l_pr=L(p, 0) [(l_pr == UNDEFINED_PRECEDENCE) || ((1 >= p) && ((l_pr == 0) || (l_pr >= 1)))] "+" r_pr=R(1, (l_pr == UNDEFINED_PRECEDENCE) ? 0 : 1) [((l_pr == UNDEFINED_PRECEDENCE) || (r_pr == UNDEFINED_PRECEDENCE)) || (a != 1)]  return (r_pr == UNDEFINED_PRECEDENCE) ? 0 : 1
    //
    //   L(p: i32, a: i32)
    //     = l_pr=E(p, a)  return l_pr
    //     | "b"           return UNDEFINED_PRECEDENCE
    //
    //   R(p: i32, a: i32)
    //     = r_pr=E(p, a)  return r_pr
    //     | "b"           return UNDEFINED_PRECEDENCE
    //
    // E, L, and R take (p, a), but return only precedence. E's atom returns 0.
    // The literal alternatives of L and R return UNDEFINED_PRECEDENCE. Both
    // intermediate nonterminals pass a to E without returning an associativity value.
    #[test]
    fn indirect_left_associativity_needs_an_argument_but_no_return_component() {
        use crate::grammar::def::Grammar;
        use crate::{alternative, grammar_def, id, left, lit, priority_level, syntax_rule};
        let grammar: Grammar = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"))),
                priority_level!(left!(); alternative!(id!("L"), lit!("+"), id!("R")))),
            syntax_rule!("L" => priority_level!(alternative!(id!("E")), alternative!(lit!("b")))),
            syntax_rule!("R" => priority_level!(alternative!(id!("E")), alternative!(lit!("b"))))
        ])
        .try_into()
        .unwrap();
        for name in ["E", "L", "R"] {
            let rule = grammar.nonterminal(name).unwrap();
            assert!(
                rule.parameters
                    .iter()
                    .any(|parameter| parameter.name == "a")
            );
            for alternative in grammar.alternatives(rule) {
                assert!(matches!(
                    alternative.symbols.last(),
                    Some(Symbol::Return(value)) if !matches!(value, Expr::Tuple(_))
                ));
                for symbol in &alternative.symbols {
                    visit_symbol(symbol, &mut |symbol| {
                        if let Symbol::Binding { pattern, .. } = symbol {
                            assert!(matches!(pattern, BindingPattern::Name(_)));
                        }
                    });
                }
            }
        }
    }

    #[test]
    fn printing_uses_the_declared_return_shape() {
        for (pr, assoc, label, printed) in [
            (Expr::Int(1), Expr::Int(0), None, "1"),
            (Expr::Int(1), Expr::Int(1), None, "(1, 1)"),
            (Expr::Int(1), Expr::Int(0), Some(0), "(1, 0)"),
            (Expr::Int(1), Expr::Int(0), Some(2), "(1, 2)"),
            (Expr::Int(1), Expr::Int(1), Some(2), "(1, 1, 2)"),
            (
                undefined_precedence(),
                Expr::Int(0),
                None,
                "UNDEFINED_PRECEDENCE",
            ),
            (Expr::Int(9000), Expr::Int(0), Some(100), "(9000, 100)"),
        ] {
            let shape = ReturnShape {
                associativity: assoc != Expr::Int(0),
                label: label.is_some(),
            };
            assert_eq!(shape.value(pr, assoc, label).to_string(), printed);
        }
        let shape = ReturnShape {
            associativity: true,
            label: true,
        };
        assert_eq!(
            shape.value(Expr::Int(0), Expr::Int(0), None).to_string(),
            "(0, 0, NO_LABEL)"
        );
        assert_eq!(shape.binding("l").to_string(), "(l_pr, l_assoc, l_label)");
        assert_eq!(no_label().to_string(), "NO_LABEL");
    }

    #[test]
    fn slot_display_uses_abstract_return_expressions() {
        use crate::grammar::{def::Grammar, slot::Slot};
        use crate::{alternative, grammar_def, id, left, lit, priority_level, syntax_rule};
        let grammar: Grammar = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"))),
                priority_level!(left!(); alternative!(id!("E"), lit!("<"), id!("E"))))
        ])
        .try_into()
        .unwrap();
        let e = grammar.nonterminal("E").unwrap();
        let slot = Slot::new(e, &grammar.alternatives(e)[1], 0);
        assert_eq!(
            slot.display_name(&grammar),
            "E : . [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] \"<\" E(2) return 1"
        );
        assert!(slot.name().ends_with("return 1"));
    }
    // Original grammar:
    //
    //   E = "a" #Atom
    //     | "b"
    //     > none L "+" L #Add
    //
    //   L
    //     = E
    //     | "x"
    //
    //   S
    //     = E !Add
    //
    // Desugared grammar:
    //
    //   E(p: i32, a: i32, e: i32)
    //     = [1 & e == 0] "a"                                                                                                                                                                                                                                                                                                                                                    return (0, 0, 0) #Atom
    //     | "b"                                                                                                                                                                                                                                                                                                                                                                 return (0, 0, NO_LABEL)
    //     | [2 & e == 0] (l_pr, l_assoc)=L(p, 0, 0) [(l_pr == UNDEFINED_PRECEDENCE) || ((1 >= p) && ((l_pr == 0) || (l_pr >= 1)))] "+" (r_pr, r_assoc)=L(1, 1, (l_pr == UNDEFINED_PRECEDENCE) ? 0 : 1) [((l_pr == UNDEFINED_PRECEDENCE) || (r_pr == UNDEFINED_PRECEDENCE)) || (a != 1)] [((l_pr == UNDEFINED_PRECEDENCE) || (r_pr == UNDEFINED_PRECEDENCE)) || (l_assoc != 1)]  return ((r_pr == UNDEFINED_PRECEDENCE) ? 0 : 1, ((l_pr == UNDEFINED_PRECEDENCE) || (r_pr == UNDEFINED_PRECEDENCE)) ? 0 : 1, 1) #Add
    //
    //   L(p: i32, end: i32, a: i32)
    //     = (v_pr, v_assoc, v_label)=E(p, a, 0)  return (v_pr, v_assoc)
    //     | "x"                                  return (UNDEFINED_PRECEDENCE, 0)
    //
    //   S
    //     = E(0, 0, 2)
    //
    // Every alternative of E returns precedence, associativity, and its own label.
    // The unlabeled literal uses NO_LABEL. L forwards precedence and associativity
    // from the selected E-end, with no label of its own. Its literal alternative
    // returns undefined precedence and associativity zero in the same pair shape.
    #[test]
    fn literal_and_unlabeled_alternatives_keep_the_rules_return_shape() {
        use crate::grammar::def::Grammar;
        use crate::{
            alternative, exclude, grammar_def, id, lit, non_assoc, priority_level, syntax_rule,
        };
        let grammar: Grammar = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"); #Atom), alternative!(lit!("b"))),
                priority_level!(non_assoc!(); alternative!(id!("L"), lit!("+"), id!("L"); #Add))),
            syntax_rule!("L" => priority_level!(alternative!(id!("E")), alternative!(lit!("x")))),
            syntax_rule!("S" => priority_level!(alternative!(exclude!(id!("E"), "Add"))))
        ])
        .try_into()
        .unwrap();
        let e = grammar.nonterminal("E").unwrap();
        let alternatives = grammar.alternatives(e);
        assert_eq!(
            alternatives[0].symbols.last().unwrap().to_string(),
            "return (0, 0, 0)"
        );
        assert_eq!(
            alternatives[1].symbols.last().unwrap().to_string(),
            "return (0, 0, NO_LABEL)"
        );
        let l = grammar.nonterminal("L").unwrap();
        let alternatives = grammar.alternatives(l);
        assert_eq!(
            alternatives[0].symbols[0].to_string(),
            "(v_pr, v_assoc, v_label)=E(p, a, 0)"
        );
        assert_eq!(
            alternatives[0].symbols.last().unwrap().to_string(),
            "return (v_pr, v_assoc)"
        );
        assert_eq!(
            alternatives[1].symbols.last().unwrap().to_string(),
            "return (UNDEFINED_PRECEDENCE, 0)"
        );
        for (rule, arity) in [(e, 3), (l, 2)] {
            for alternative in grammar.alternatives(rule) {
                assert!(matches!(alternative.symbols.last(),
                    Some(Symbol::Return(Expr::Tuple(values))) if values.len() == arity));
            }
        }
    }
}
