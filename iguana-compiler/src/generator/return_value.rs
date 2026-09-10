//! Tuple returns in the Rust generator's single i32 return slot.
//!
//! Scalars use the slot directly. Pairs use 16 bits per component; triples use
//! 14, 13, and 5 bits. The first component is signed and the remaining components
//! are unsigned. Components are packed from most to least significant bits.
//! The five-bit component fits exclusion label indices 0 through 30 and
//! `NO_LABEL` (31). Packing and extraction treat it as an unsigned integer.
//! These limits belong to this backend; tuple expressions and bindings in the
//! grammar IR describe only values and positions.
//!
//! Destructuring extracts the components when a call returns. Each component
//! becomes an ordinary environment binding; reading a name needs no extraction.

use crate::grammar::{
    def::Grammar,
    symbols::{BindingPattern, Expr, Symbol},
    transformations::visit_symbol,
};
use std::io;

fn widths(arity: usize) -> Result<&'static [u32], String> {
    match arity {
        2 => Ok(&[16, 16]),
        3 => Ok(&[14, 13, 5]),
        _ => Err("tuple returns must have two or three components in the Rust generator".into()),
    }
}

/// Packs tuple components into the runtime return slot.
pub(super) fn pack(values: &[Expr]) -> Expr {
    let widths = widths(values.len()).expect("tuple arity was validated");
    values
        .iter()
        .zip(widths)
        .enumerate()
        .fold(Expr::Int(0), |packed, (index, (value, _))| {
            let shift: u32 = widths[index + 1..].iter().sum();
            let value = match value {
                Expr::DisplayInt { value, .. } => Expr::Int(*value),
                value => value.clone(),
            };
            let shifted = match value {
                Expr::Int(value) => Expr::Int(value << shift),
                value if shift == 0 => value,
                value => Expr::Shl(Box::new(value), Box::new(Expr::Int(shift as i64))),
            };
            match (packed, shifted) {
                (Expr::Int(left), Expr::Int(right)) => Expr::Int(left | right),
                (Expr::Int(0), right) => right,
                (left, Expr::Int(0)) => left,
                (left, right) => Expr::BitOr(Box::new(left), Box::new(right)),
            }
        })
}

/// Generates extraction of a tuple component from a Rust value expression.
/// The first component uses an arithmetic shift; the others are masked.
pub(super) fn gen_component(
    value: proc_macro2::TokenStream,
    index: usize,
    arity: usize,
) -> proc_macro2::TokenStream {
    use quote::quote;
    let widths = widths(arity).expect("tuple arity was validated");
    let shift: u32 = widths[index + 1..].iter().sum();
    let mask = proc_macro2::Literal::i32_unsuffixed((1_i32 << widths[index]) - 1);
    let shifted = if shift == 0 {
        value
    } else {
        let shift = proc_macro2::Literal::u32_unsuffixed(shift);
        quote! { (#value) >> #shift }
    };
    if index == 0 {
        shifted
    } else {
        quote! { (#shifted) & #mask }
    }
}

/// Checks tuple arities, bindings to explicit returns, and constant component bounds
/// before writing generated files.
pub(super) fn validate(grammar: &Grammar) -> io::Result<()> {
    for head in grammar.nonterminals() {
        for alternative in grammar.alternatives(head) {
            for symbol in &alternative.symbols {
                let mut result = Ok(());
                visit_symbol(symbol, &mut |symbol| {
                    if result.is_err() {
                        return;
                    }
                    result = match symbol {
                        Symbol::Return(expr) | Symbol::Condition(expr) => validate_expr(expr),
                        Symbol::Call { arguments, .. } => {
                            arguments.iter().try_for_each(validate_expr)
                        }
                        Symbol::Binding {
                            pattern: BindingPattern::Tuple(names),
                            symbol,
                        } => widths(names.len())
                            .and_then(|_| validate_binding(names.len(), symbol, grammar)),
                        _ => Ok(()),
                    };
                });
                result.map_err(|message| io::Error::other(format!("`{}` {message}", head.name)))?;
            }
        }
    }
    Ok(())
}

/// Checks each callee alternative whose return arity is explicit. A forwarded
/// reference has no arity information here and remains unchecked.
fn validate_binding(arity: usize, symbol: &Symbol, grammar: &Grammar) -> Result<(), String> {
    let Some(head) = symbol
        .as_identifier()
        .and_then(|id| grammar.nonterminal(&id.name))
    else {
        return Ok(());
    };
    for alternative in grammar.alternatives(head) {
        let value = match alternative.symbols.last() {
            Some(Symbol::Return(value)) => value,
            _ => &Expr::Int(0),
        };
        check_return_arity(value, arity).map_err(|returned| {
            let component = if returned == 1 { "component" } else { "components" };
            format!(
                "calls `{}` with a {arity}-component binding, but an alternative returns {returned} {component}",
                head.name
            )
        })?;
    }
    Ok(())
}

fn check_return_arity(value: &Expr, expected: usize) -> Result<(), usize> {
    let arity = match value {
        Expr::Tuple(values) => values.len(),
        Expr::Ref(_) => return Ok(()),
        Expr::Ternary { then, r#else, .. } => {
            check_return_arity(then, expected)?;
            return check_return_arity(r#else, expected);
        }
        _ => 1,
    };
    if arity == expected {
        Ok(())
    } else {
        Err(arity)
    }
}

fn validate_expr(expr: &Expr) -> Result<(), String> {
    let mut result = Ok(());
    expr.clone().transform(&mut |expr| {
        if result.is_ok() {
            if let Expr::Tuple(values) = &expr {
                result = widths(values.len()).and_then(|widths| {
                    values.iter().zip(widths).enumerate().try_for_each(
                        |(index, (value, &width))| {
                            let (min, max) = if index == 0 {
                                (-(1_i64 << (width - 1)), (1_i64 << (width - 1)) - 1)
                            } else {
                                (0, (1_i64 << width) - 1)
                            };
                            validate_component(value, min, max).map_err(|_| {
                                format!(
                                    "tuple component {} must be between {min} and {max}",
                                    index + 1
                                )
                            })
                        },
                    )
                });
            }
        }
        expr
    });
    result
}

fn validate_component(expr: &Expr, min: i64, max: i64) -> Result<(), ()> {
    match expr {
        Expr::Int(value) | Expr::DisplayInt { value, .. } if !(min..=max).contains(value) => {
            Err(())
        }
        Expr::Tuple(_) => Err(()),
        Expr::Ternary { then, r#else, .. } | Expr::Min(then, r#else) => {
            validate_component(then, min, max)?;
            validate_component(r#else, min, max)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A tuple binding must use the layout of the callee's return:
    //
    //   S = (x, y)=A return x
    //   A = "a" return (1, 2, 3)
    //
    // This grammar is rejected: A returns three components and S binds two.
    #[test]
    fn tuple_binding_must_match_the_callees_explicit_returns() {
        use crate::{alternative, grammar_def, id, lit, priority_level, ret, syntax_rule, tuple};
        for (value, arity, valid) in [
            (tuple!(1, 2), 2, true),
            (tuple!(1, 2, 3), 2, false),
            (tuple!(1, 2), 3, false),
            (tuple!(1, 2, 3), 3, true),
            (Expr::Int(1), 2, false),
        ] {
            let binding = Symbol::Binding {
                pattern: BindingPattern::Tuple(
                    ["x", "y", "z"][..arity]
                        .iter()
                        .map(|name| (*name).into())
                        .collect(),
                ),
                symbol: Box::new(id!("A")),
            };
            let grammar: Grammar = grammar_def!("Test", syntax: [
                syntax_rule!("S" => priority_level!(alternative!(
                    binding, ret!(expr Expr::Ref("x".into()))))),
                syntax_rule!("A" => priority_level!(alternative!(lit!("a"), ret!(expr value))))
            ])
            .try_into()
            .unwrap();
            let result = validate(&grammar);
            assert_eq!(result.is_ok(), valid, "{result:?}");
            if !valid {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains(&format!("calls `A` with a {arity}-component binding"))
                );
            }
        }
    }

    #[test]
    fn tuple_constants_pack_at_storage_boundaries() {
        for (values, packed) in [
            (vec![-32768, 65535], -2147418113),
            (vec![32767, 0], 2147418112),
            (vec![-1, 31], -65505),
            (vec![-8192, 8191, 31], -2147221505),
            (vec![8191, 8191, 30], 2147483646),
            (vec![2, 3, 5], 524389),
        ] {
            let exprs: Vec<_> = values.iter().copied().map(Expr::Int).collect();
            assert_eq!(pack(&exprs), Expr::Int(packed));
            assert!(validate_expr(&Expr::Tuple(exprs)).is_ok());
        }
    }

    #[test]
    fn tuple_limits_depend_on_position_and_arity() {
        for values in [
            vec![32768, 0],
            vec![0, -1],
            vec![0, 65536],
            vec![8192, 0, 0],
            vec![0, 8192, 0],
            vec![0, 0, 32],
        ] {
            assert!(
                validate_expr(&Expr::Tuple(values.into_iter().map(Expr::Int).collect())).is_err()
            );
        }
        assert!(validate_expr(&Expr::Tuple(vec![Expr::Int(0); 4])).is_err());
    }
}
