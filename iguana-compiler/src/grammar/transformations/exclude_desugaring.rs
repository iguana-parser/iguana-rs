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
// labels per nonterminal as the practical cap; the fallback if that is
// ever exceeded is i64). The target nonterminal grows an extra parameter
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
//     = [1 & e == 0] Id return 0                   #Id
//     | [2 & e == 0] Expr(0) "," Expr(0) return 1  #Comma
//     | [4 & e == 0] "(" Expr(2) ")" return 2      #Group
//
// What the desugaring does:
//
//   - Bit assignment. The desugaring numbers the labeled alternatives in
//     source order: #Id is 0, #Comma is 1, #Group is 2. Each number plays
//     two roles. It becomes the alternative's `return N`, and it is the
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
//   - Return value. Every alternative ends with `return N`, where N is
//     the alternative's number from the bit assignment above, or `-1`
//     for unlabeled alternatives. The parser uses the return value as
//     part of its parse-tree sharing key, so distinct alternatives at
//     the same input span produce distinct subtrees.
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
// be silently skipped if the order were reversed. When precedence
// desugaring also applies to a target nonterminal, the two passes
// share the same i32 return slot: precedence in the high 16 bits, the
// exclude label in the low 16 bits.

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
/// `Return(label_index)` to every alternative of a targeted nonterminal.
/// Unlabeled alts get no guard and return `-1`.
fn decorate_alt(alt: Alternative, labels: &[String]) -> Alternative {
    let label_index = alt
        .label
        .as_ref()
        .and_then(|label| labels.iter().position(|l| l == label).map(|p| p as u32));

    let mut symbols = Vec::with_capacity(alt.symbols.len() + 2);
    if let Some(bit) = label_index {
        symbols.push(exclude_guard(bit));
    }
    symbols.extend(alt.symbols);
    let return_value: i64 = label_index.map_or(-1, |bit| bit as i64);
    symbols.push(Symbol::Return(Expr::Int(return_value)));
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
        Symbol::Binding { name, symbol } => Symbol::Binding {
            name,
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
