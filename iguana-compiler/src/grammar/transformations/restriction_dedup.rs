use itertools::Itertools;

use crate::grammar::{
    def::SyntaxRule,
    symbols::{Restrictions, Symbol},
    transformations::{transform_symbol, transform_syntax_rule},
};

/// Removes the repeated operands of each restriction kind, keeping the first
/// occurrence. For example, `A !>> B !>> B` becomes `A !>> B`. Operands of
/// different kinds are kept: `A \ B !>> B` does not change.
pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    syntax_rules
        .into_iter()
        .map(|rule| transform_syntax_rule(rule, |symbol| transform_symbol(symbol, &mut dedup)))
        .collect()
}

fn dedup(symbol: Symbol) -> Symbol {
    match symbol {
        Symbol::Restricted {
            symbol,
            restrictions,
        } => Symbol::Restricted {
            symbol,
            restrictions: Restrictions {
                precede: restrictions.precede.into_iter().unique().collect(),
                excepts: restrictions.excepts.into_iter().unique().collect(),
                required: restrictions.required.into_iter().unique().collect(),
                follow: restrictions.follow.into_iter().unique().collect(),
                layout_aware_follow: restrictions
                    .layout_aware_follow
                    .into_iter()
                    .unique()
                    .collect(),
            },
        },
        symbol => symbol,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{id, restriction_ids};

    fn dedup_restrictions(restrictions: Restrictions) -> Restrictions {
        dedup(Symbol::restricted(id!("A"), restrictions))
            .restrictions()
            .clone()
    }

    #[test]
    fn a_repeated_operand_is_removed() {
        let restrictions = dedup_restrictions(Restrictions {
            follow: restriction_ids!("B", "C", "B"),
            ..Default::default()
        });
        assert_eq!(restrictions.follow, restriction_ids!("B", "C"));
    }

    #[test]
    fn the_same_operand_in_two_kinds_is_kept() {
        let restrictions = dedup_restrictions(Restrictions {
            excepts: restriction_ids!("B"),
            follow: restriction_ids!("B"),
            ..Default::default()
        });
        assert_eq!(restrictions.excepts, restriction_ids!("B"));
        assert_eq!(restrictions.follow, restriction_ids!("B"));
    }
}
