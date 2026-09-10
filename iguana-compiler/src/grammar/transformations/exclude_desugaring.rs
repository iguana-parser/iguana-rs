// Exclude desugaring.
//
// In an iggy grammar, `Expr !Comma` reads as "match Expr, but admit no
// alternative labeled #Comma at the top". The user writes this when the
// surrounding context restricts which forms of an expression are valid
// at that position. The exclude desugaring transformation encodes the
// restriction into the grammar, by introducing a parameterized nonterminal
// that excludes the alternative at runtime based on the passed value.
//
// The implementation mechanism is a bitmask. Each call site names the
// excluded labels as a single i32 (with the sign bit unused, giving 31
// labels per nonterminal). The target nonterminal grows an extra parameter
// to receive that mask, and each labeled alternative gains a guard that
// checks its own bit. The alternative is admitted only when its bit is
// clear in the mask.
//
// Consider this grammar:
//
//   Expr
//     = Id                     #Id
//     | Expr "," Expr          #Comma
//     | "(" Expr !Comma ")"    #Group
//
// The parenthesized form forbids a comma at the top. After this pass:
//
//   Expr(e: i32)
//     = [1 & e == 0] Id return (0, 0)                   #Id
//     | [2 & e == 0] Expr(0) "," Expr(0) return (0, 1)  #Comma
//     | [4 & e == 0] "(" Expr(2) ")" return (0, 2)       #Group
//
// What the desugaring does:
//
//   - Bit assignment. The desugaring numbers the labeled alternatives in
//     source order: #Id is 0, #Comma is 1, #Group is 2. Each number plays
//     two roles. It becomes the return's label field, and it is the
//     position of the bit the guard tests. The bit values that appear in
//     the guards are therefore `1`, `2`, `4`, computed as `1 << N`.
//
//   - `e: i32` parameter. Every target nonterminal gets this parameter,
//     which holds the exclude mask. Each call site passes the union of
//     bit values for the labels it excludes: a bare `Expr` passes 0,
//     `Expr !Comma` passes 2, `Expr !Comma !Group` passes 6.
//
//   - Guard on each labeled alternative. `[1 & e == 0]`, `[2 & e == 0]`,
//     and `[4 & e == 0]` test whether this label's bit is clear in `e`.
//     The alternative is admitted only when its label is not in the
//     exclude set.
//
//   - Return value. Every alternative of an exclusion-targeted nonterminal
//     returns its own label, or NO_LABEL if unlabeled, alongside the other return
//     components. The precedence desugaring transformation preserves this label
//     for the left-operand exclusion check after the shared recursive call.
//     Ordinary calls enforce exclusions with the early guards above. Distinct
//     returned labels keep alternatives distinguishable in the parser's sharing key.
//
//   - Use-site rewriting. Every exclude reference becomes a call that
//     passes the mask, and every bare reference to a target nonterminal
//     becomes a call that passes 0. No bare reference to a target
//     survives.
//
// Tracing a call. Consider `Expr(2)`, the recursive call inside `#Group`
// after `"("`. The caller excludes #Comma, so `e = 2 = 0b010`. Each guard
// evaluates against that mask:
//
//   - `#Id`:    `1 & 2 == 0` → true   → admitted
//   - `#Comma`: `2 & 2 == 0` → false  → rejected
//   - `#Group`: `4 & 2 == 0` → true   → admitted
//
// Only `Expr(2)` rejects #Comma. The other recursive calls in the
// example use `Expr(0)`, where every guard passes and every alternative
// is admitted.
//
// Pipeline position. The exclude desugaring transformation runs after
// EBNF expansion and before precedence desugaring. EBNF wrappers are not
// recursed through, so an exclude reference nested inside one would
// be silently skipped if the order were reversed. Both passes always use the
// same return components, including rules that only use exclusions.

use super::return_value;
use rustc_hash::FxHashMap;

use crate::grammar::{
    def::{Alternative, PriorityLevel, SyntaxRule},
    symbols::{Cond, CondOp, Expr, ParamType, Parameter, Symbol},
    transformations::{transform_syntax_rule, visit_syntax_rule},
};

pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    // The set of nonterminals that are the *target* of some `!Label`
    // operator anywhere in the grammar, each paired with its labels in
    // the order they appear in the rule. These are the nonterminals
    // that need to be parameterized with `e: i32` and have their
    // alternatives decorated with guards.
    let mut targets: FxHashMap<String, Vec<String>> = FxHashMap::default();
    for rule in &syntax_rules {
        visit_syntax_rule(rule, &mut |symbol| {
            if let Symbol::Exclude { symbol, .. } = symbol {
                let name = symbol
                    .as_identifier()
                    .expect("Exclude symbol should wrap an Identifier")
                    .name
                    .clone();
                targets.entry(name).or_default();
            }
        });
    }

    if targets.is_empty() {
        return syntax_rules;
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

    syntax_rules
        .into_iter()
        .map(|rule| {
            let rule = if let Some(labels) = targets.get(&rule.head.name) {
                add_e_param_and_guards(rule, labels)
            } else {
                rule
            };
            transform_syntax_rule(rule, |symbol| rewrite_target_refs(symbol, &targets))
        })
        .collect()
}

fn add_e_param_and_guards(mut rule: SyntaxRule, labels: &[String]) -> SyntaxRule {
    rule.head.parameters.push(Parameter {
        name: "e".to_string(),
        ty: ParamType::I32,
    });
    rule.priority_levels = rule
        .priority_levels
        .into_iter()
        .map(|pl| PriorityLevel {
            alternatives: pl
                .alternatives
                .into_iter()
                .map(|alt| decorate_alt(alt, labels))
                .collect(),
            associativity: pl.associativity,
        })
        .collect();
    rule
}

/// Prepends `[(BIT_L & e) == 0]` to a labeled alternative and appends
/// a return with precedence zero and the alternative's label to every
/// alternative of a targeted nonterminal.
/// Unlabeled alternatives get no guard and return `(0, NO_LABEL)`.
fn decorate_alt(alt: Alternative, labels: &[String]) -> Alternative {
    let label_index = alt
        .label
        .as_ref()
        .and_then(|label| labels.iter().position(|l| l == label));

    let mut symbols = Vec::with_capacity(alt.symbols.len() + 2);
    if let Some(bit) = label_index {
        symbols.push(exclude_guard(bit as u32));
    }
    symbols.extend(alt.symbols);
    symbols.push(Symbol::Return(return_value::with_label(
        Expr::Int(0),
        label_index,
    )));
    Alternative {
        symbols,
        label: alt.label,
    }
}

fn exclude_guard(bit: u32) -> Symbol {
    let bit_value: i64 = 1 << bit;
    Symbol::Condition(Expr::Cond(Cond {
        left: Box::new(Expr::BitAnd(
            Box::new(Expr::Int(bit_value)),
            Box::new(Expr::Ref("e".to_string())),
        )),
        right: Box::new(Expr::Int(0)),
        op: CondOp::Eq,
    }))
}

fn rewrite_target_refs(symbol: Symbol, targets: &FxHashMap<String, Vec<String>>) -> Symbol {
    match symbol {
        Symbol::Exclude { symbol, labels } => {
            let id = symbol
                .as_identifier()
                .expect("Exclude symbol should wrap an Identifier")
                .clone();
            let target_labels = targets
                .get(&id.name)
                .expect("Exclude target should be in the targets map");
            let bitmask = labels.iter().fold(0i64, |acc, label| {
                let bit = target_labels
                    .iter()
                    .position(|l| l == label)
                    .unwrap_or_else(|| panic!("Label {label} not defined on {}", id.name));
                acc | (1i64 << bit)
            });
            Symbol::Call {
                name: id,
                arguments: vec![Expr::Int(bitmask)],
            }
        }
        Symbol::Identifier(id) if targets.contains_key(&id.name) => Symbol::Call {
            name: id,
            arguments: vec![Expr::Int(0)],
        },
        Symbol::Labeled { label, symbol } => Symbol::Labeled {
            label,
            symbol: Box::new(rewrite_target_refs(*symbol, targets)),
        },
        Symbol::Binding { pattern, symbol } => Symbol::Binding {
            pattern,
            symbol: Box::new(rewrite_target_refs(*symbol, targets)),
        },
        Symbol::Restricted {
            symbol,
            restrictions,
        } => Symbol::Restricted {
            symbol: Box::new(rewrite_target_refs(*symbol, targets)),
            restrictions,
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{alternative, exclude, id, lit, priority_level, syntax_rule};

    //   E = "a" #blocked
    //     | "b"
    //   S = E !blocked
    //
    // After exclusion desugaring:
    //
    //   E(e: i32)
    //     = [1 & e == 0] "a" return (0, 0)        #blocked
    //     | "b"              return (0, NO_LABEL)
    //   S = E(1)
    #[test]
    fn unlabeled_alternative_returns_no_label_before_precedence_desugaring() {
        let rules = transform(vec![
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
            alternatives[0].symbols.last().unwrap().to_string(),
            "return (0, 0)"
        );
        assert_eq!(
            alternatives[1].symbols.last().unwrap().to_string(),
            "return (0, NO_LABEL)"
        );
    }
}
