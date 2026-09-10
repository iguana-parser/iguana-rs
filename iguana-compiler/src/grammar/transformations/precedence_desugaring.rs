// Precedence desugaring.
//
// Implements the desugaring algorithm described in "Operator Precedence for
// Data-Dependent Grammars":
// https://cdn.jsdelivr.net/gh/iguana-parser/papers@master/pepm16.pdf
//
// Consider the following grammar, where `>` defines priority groups:
//
//   E = "a"
//     > E "*" E
//     > E "+" E
//
// Without `>`, the input `a + a * a` would be ambiguous, with two derivations:
// `(a + a) * a` and `a + (a * a)`. Multiplication has higher precedence than
// addition in this grammar, so `a + (a * a)` is the intended one.
//
// The precedence desugaring transformation encodes this restriction with a
// parameter, return values, and conditions on recursive calls. Priority groups
// containing recursive alternatives get precedence levels starting at 1, from
// bottom to top. Alternatives in the same group share a level. For this grammar,
// addition has level 1 and multiplication has level 2. The resulting grammar is:
//
//   E(p: i32)
//     = "a"                                                  return 0
//     | [2 >= p] l_pr=E(p) [l_pr == 0 || l_pr >= 2] "*" E(2) return 2
//     | [1 >= p] l_pr=E(p) [l_pr == 0 || l_pr >= 1] "+" E(1) return 1
//
// `p` is the precedence level passed by the caller. In the addition
// alternative, the right end is called as `E(1)`. Multiplication is allowed
// in this call because its precondition is `2 >= 1`. The call `E(2)` on the
// right of multiplication excludes addition because `1 >= 2` is false.
// References to `E` that are not recursive ends become `E(0)`. For example,
// the `E` in `"(" E ")"` is unrestricted because neither end is recursive.
//
// The left recursive call passes `p` unchanged and returns a value, which is
// bound to `l_pr`. The parser rejects the grouping `(a + a) * a` because the
// addition `a + a` on the left returns 1. Multiplication's postcondition
// requires `l_pr == 0 || l_pr >= 2`, which fails for 1. The literal alternative
// returns zero, which is allowed by every precedence postcondition.
//
// Prefix and postfix alternatives have only one recursive end. A prefix
// such as `"-" E` passes its precedence level to `E`, but has no left
// recursive end on which to apply the left-end conditions shown above.
// A postfix such as `E "!"` applies those conditions to `E`, but has no
// right recursive end and returns zero.
//
// Left or right recursive ends can also be derived through intermediate
// nonterminals. For example:
//
//   E = "a"
//     > L "*" R
//     > L "+" R
//   L = E
//   R = E
//
// Reachability analysis establishes that `L` derives a left `E`-end and `R`
// derives a right `E`-end. The transformation adds a precedence parameter to
// each nonterminal. Both pass the argument to `E` and return its precedence.
// The desugared grammar is:
//
//   E(p: i32)
//     = "a"                                                  return 0
//     | [2 >= p] l_pr=L(p) [l_pr == 0 || l_pr >= 2] "*" R(2) return 2
//     | [1 >= p] l_pr=L(p) [l_pr == 0 || l_pr >= 1] "+" R(1) return 1
//
//   L(p: i32)
//     = l_pr=E(p) return l_pr
//
//   R(p: i32)
//     = r_pr=E(p) return r_pr
//
// Longer indirect chains work the same way: with `L = M` and `M = E`, both
// nonterminals pass the argument toward `E` and return the precedence to
// their caller.
//
// An intermediate nonterminal can have alternatives without the
// recursive end. For example, suppose `L` and `R` are defined as follows:
//
//   L = E | E "!"
//   R = E | E "!"
//
// Both alternatives of `L` have a left `E`-end, so the left end is always
// present. However, only the first alternative of `R` has a right `E`-end.
// In `R = E "!"`, the right end is `"!"`, and the call to `E` is
// unrestricted. Whether the right `E`-end is present depends on which
// alternative of `R` is parsed.
//
// The implementation uses the return value to distinguish these cases. The
// first alternative of `R` returns the precedence from `E`. The second
// returns an undefined precedence, written as `UNDEFINED_PRECEDENCE`,
// because it has no right `E`-end. The caller checks whether the returned
// precedence is undefined.
//
// The full translation is shown below:
//
//   E(p: i32)
//     = "a"                                                       return 0
//     | [2 >= p] l_pr=L(p) [l_pr == 0 || l_pr >= 2] "*" r_pr=R(2) return r_pr == UNDEFINED_PRECEDENCE ? 0 : 2
//     | [1 >= p] l_pr=L(p) [l_pr == 0 || l_pr >= 1] "+" r_pr=R(1) return r_pr == UNDEFINED_PRECEDENCE ? 0 : 1
//
//   L(p: i32)
//     = l_pr=E(p)     return l_pr
//     | l_pr=E(p) "!" return l_pr
//
//   R(p: i32)
//     = r_pr=E(p) return r_pr
//     | E(0) "!"  return UNDEFINED_PRECEDENCE
//
// The conditions on `l_pr` apply to both alternatives of `L`, since both derive
// a left `E`-end. The return expressions distinguish the two alternatives of
// `R`: `R = E` provides a right `E`-end, while `R = E "!"` does not. In the
// latter case, the enclosing alternative returns zero, as a postfix
// alternative does.
//
// The same intermediate nonterminal can also occur at both ends:
//
//   E = "a"
//     > L "*" L
//     > L "+" L
//   L = E
//
// In `L "*" L`, the first call to `L` is for the left `E`-end, and the
// second is for the right `E`-end. The transformation adds an `end` parameter
// alongside `p` to distinguish these uses. Its value selects the end to
// restrict and the end whose precedence is returned. Nonterminals used at
// only one end, such as the separate `L` and `R` above, omit this parameter.
//
// For `L = E`, either selection restricts the same call to `E`. If `L` also
// has the alternative `E "!"` discussed above, selecting the left end
// restricts that alternative's `E`. Selecting the right end leaves `E`
// unrestricted and returns an undefined precedence. The position of the call
// determines which end is selected; the alternative parsed by `L` determines
// whether that recursive `E`-end is present.
//
// Associativity can also be enforced through intermediate nonterminals.
// Consider a grammar with one operator, declared left associative:
//
//   E = "a"
//     > left L "+" R
//   L = E
//   R = E
//
// According to the `left` associativity, the input `a+a+a` should be parsed
// as `(a+a)+a`. `left` excludes `a+(a+a)` by preventing addition at the right
// end. Here, every call to `L` or `R` derives `E`, so both recursive ends are
// always present.
//
// Addition has precedence level 1. The transformation enforces left
// associativity by passing level 2 to `R`, which passes it to `E`. The
// resulting grammar is:
//
//   E(p: i32)
//     = "a"                                                  return 0
//     | [1 >= p] l_pr=L(p) [l_pr == 0 || l_pr >= 1] "+" R(2) return 1
//
//   L(p: i32)
//     = l_pr=E(p) return l_pr
//
//   R(p: i32)
//     = r_pr=E(p) return r_pr
//
// `R(2)` calls `E(2)`, where addition's precondition `1 >= p` fails.
// Another addition is allowed on the left because `L` passes `p`
// unchanged and the postcondition accepts precedence 1.
//
// An intermediate nonterminal may also define a literal alternative of its
// own, allowing the literal to appear in an expression without deriving `E`:
//
//   E = "a"
//     > L "*" R
//     > L "+" R
//   L = E | "b"
//   R = E | "b"
//
// Here, `"a"` is an alternative of `E`, but `"b"` is only an alternative of
// `L` and `R`. A call to either intermediate nonterminal can parse `"b"`
// without deriving a recursive `E`-end. The declared precedence restricts
// recursive `E`-ends, so it does not resolve every ambiguity in this grammar.
// In particular, `b + b * b` has both groupings. The inner `b + b` returns
// precedence zero because its `R` parses `"b"`, and zero is allowed on the
// left of multiplication.
//
// If `"b"` is instead an atom of `E`, with `L = E` and `R = E`, both
// recursive `E`-ends are always present. Precedence then selects only
// `b + (b * b)`.
//
// Literal alternatives in intermediate nonterminals also affect indirect
// associativity. Adding `"b"` to `L` and `R` in the left-associative grammar
// gives:
//
//   E = "a"
//     > left L "+" R
//   L = E | "b"
//   R = E | "b"
//
// The input `a+a+a` still has only the grouping `(a+a)+a`, because each
// addition derives both recursive `E`-ends. However, `b+b+b` has both
// groupings. In `b+(b+b)`, the outer addition has no left `E`-end, and the
// nested addition has neither a left nor a right `E`-end. These additions
// do not participate in the binary associativity restriction.
//
// Raising the precedence argument to 2, as in the preceding translation,
// would also exclude an addition whose right recursive `E`-end is absent.
// The associativity restriction must allow that case. The transformation
// therefore uses a separate associativity argument `a` and keeps the right
// precedence argument at 1. The argument `a = 1` excludes binary addition at
// level 1, while `a = 0` imposes no associativity restriction. The levels are
// the same precedence levels introduced above. The restriction applies only
// when the enclosing addition and the addition nested on its right each
// derive both a left and a right `E`-end. The choice between these
// translations depends on whether every alternative of `L` and `R` derives
// the corresponding `E`-end.
//
// Right and non-associative levels may also need to check the alternative
// returned by the left recursive call. The rule then returns the precedence
// level used for associativity checks alongside its returned precedence.
// The binding `(l_pr, l_assoc)` names these values separately. Every alternative
// of that rule returns a pair, including an atom returning `(0, 0)`. The right
// recursive call uses the corresponding names `(r_pr, r_assoc)`. When only the
// incoming argument `a` is needed, as in the left-associative example above,
// the returned precedence remains scalar.
//
// Precedence and exclusions. A nonterminal reference may also exclude an
// alternative:
//
//   E = "a"           #Lit
//     > E "*" E       #Mul
//     > E "+" E       #Add
//   S = E !Add
//
// In `S`, `E` cannot be derived through the alternative labeled `Add`.
// The exclusion transformation adds a parameter `e` for the excluded labels
// and guards the corresponding alternatives. The precedence transformation
// adds `p`, so a call can restrict both the precedence and the alternatives
// of `E`.
//
// The desugared grammar below shows returns as `(p, label)` pairs, where `p`
// is the returned precedence and `label` is the chosen alternative's label index.
// The indices for `Lit`, `Mul`, and `Add` are 0, 1, and 2. The input parameter
// `e` is an exclusion bitmask: the corresponding bits have values 1, 2, and 4.
// The binding `(l_pr, l_label)` names the precedence and label returned by
// the left recursive call. Conditions use these names directly.
//
//   E(p: i32, e: i32)
//     = [(1 & e) == 0] "a"                                                                   return (0, 0) #Lit
//     | [(2 & e) == 0] [2 >= p] (l_pr, l_label)=E(p, 0) [l_pr == 0 || l_pr >= 2] "*" E(2, 0) return (2, 1) #Mul
//     | [(4 & e) == 0] [1 >= p] (l_pr, l_label)=E(p, 0) [l_pr == 0 || l_pr >= 1] "+" E(1, 0) return (1, 2) #Add
//
//   S
//     = E(0, 4)
//
// The call `E(0, 4)` leaves precedence unrestricted and excludes `Add`.
// Multiplication returns `(2, 1)`: precedence 2 and the label index for `Mul`.
// Precedence conditions read the first component. An exclusion on a left
// recursive reference checks the second component after the call.
//
// An intermediate nonterminal forwards the precedence and associativity of the
// selected recursive end, but returns its own label. For example, `L = E`
// returns the precedence from `E`, but an exclusion on a reference to `L`
// applies to an alternative of `L`. Returning the label from `E` would make
// that exclusion check an alternative of the wrong rule.

use super::return_value;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::grammar::{
    def::{Alternative, Associativity, PriorityLevel, SyntaxRule},
    symbols::{
        Cond, CondOp, Expr, Identifier, Nonterminal, ParamType, Parameter, Restrictions, Symbol,
    },
    transformations::{transform_symbol, transform_syntax_rule},
};

/// Classifies an alternative by its left and right recursive ends.
/// Each recursive end derives a sequence with the rule's head at that end,
/// directly or through intermediate nonterminals.
/// Below, α, β, and γ are sequences of grammar symbols, possibly empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecursionKind {
    /// E = A α B, where A ⇒* E β and B ⇒* γ E.
    Binary,
    /// E = α B, where B ⇒* β E and the left end cannot derive a sequence starting with E.
    Prefix,
    /// E = A α, where A ⇒* E β and the right end cannot derive a sequence ending with E.
    Postfix,
    /// Neither end derives a sequence with E at that end, as in E = "a" or E = "(" E ")".
    NonRecursive,
}

/// A left or right end of an alternative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    Left,
    Right,
}

/// Indirect recursion from a nonterminal to a precedence head.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IndirectRecursion {
    head: String,
    /// True when this nonterminal is used as a left recursive end.
    left: bool,
    /// True when this nonterminal is used as a right recursive end.
    right: bool,
    /// True when the left recursive end is present in every derivation through
    /// this nonterminal.
    left_required: bool,
    /// True when the right recursive end is present in every derivation through
    /// this nonterminal.
    right_required: bool,
}

impl IndirectRecursion {
    fn new(head: &str) -> Self {
        Self {
            head: head.to_string(),
            left: false,
            right: false,
            left_required: false,
            right_required: false,
        }
    }

    /// Marks this intermediate nonterminal as occurring at the selected recursive end.
    fn mark_end(&mut self, end: End) {
        match end {
            End::Left => self.left = true,
            End::Right => self.right = true,
        }
    }

    /// True when this intermediate nonterminal occurs at the selected recursive end.
    /// For `E = L "+" R`, the record for `R` returns true for `End::Right`,
    /// even if `R = E | "b"` can parse an alternative without a right `E`-end.
    fn is_at_end(&self, end: End) -> bool {
        match end {
            End::Left => self.left,
            End::Right => self.right,
        }
    }

    fn is_at_both_ends(&self) -> bool {
        self.left && self.right
    }

    /// True when the recursive head is present at the selected end in every
    /// derivation through this intermediate nonterminal. For `R = E | "b"`,
    /// the record for `R` returns false for `End::Right`: `"b"` has no right
    /// `E`-end. For `R = E`, it returns true.
    fn is_always_present_at_end(&self, end: End) -> bool {
        match end {
            End::Left => self.left_required,
            End::Right => self.right_required,
        }
    }
}

/// Runs after EBNF expansion, which exposes the recursive ends needed for
/// precedence desugaring, and after exclusion desugaring, whose guards and
/// labels this transformation preserves.
pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Result<Vec<SyntaxRule>, Vec<String>> {
    let desugaring = PrecedenceDesugaring::new(&syntax_rules)?;
    Ok(desugaring.transform(syntax_rules))
}

struct PrecedenceDesugaring {
    /// Names of recursive rules that declare precedence or associativity.
    recursive_heads: Vec<String>,
    /// Direct left- and right-end relations between nonterminals.
    ends: Ends,
    /// Nonterminal names referenced with exclusions, such as `E` in `E !Add`.
    /// Exclusion desugaring gives these nonterminals an `e` parameter. Calls
    /// without an exclusion pass `0` for this argument.
    exclusion_targets: FxHashSet<String>,
    /// Maps each intermediate nonterminal's name to its `IndirectRecursion` record.
    indirect_recursion: FxHashMap<String, IndirectRecursion>,
    /// Maps each rule's name to one flag per priority group, in grammar order.
    /// True means the group uses an associativity parameter `a`. Its restriction
    /// applies only to binary alternatives whose two recursive ends both derive the head.
    associativity_arguments: FxHashMap<String, Vec<bool>>,
    /// Names of precedence heads that return an associativity level alongside precedence.
    /// Right and non-associative groups use this value to reject a binary alternative
    /// from the same group at the left recursive end.
    associativity_returns: FxHashSet<String>,
}

impl PrecedenceDesugaring {
    fn new(syntax_rules: &[SyntaxRule]) -> Result<Self, Vec<String>> {
        let rules_with_precedence_or_associativity: Vec<String> = syntax_rules
            .iter()
            .filter(|rule| needs_desugaring(rule))
            .map(|rule| rule.head.name.clone())
            .collect();

        let rules_by_name: FxHashMap<&str, &SyntaxRule> = syntax_rules
            .iter()
            .map(|r| (r.head.name.as_str(), r))
            .collect();

        let ends = compute_ends(syntax_rules);
        let recursive_heads = compute_recursive_heads(syntax_rules, &ends);
        let mut indirect_recursion = compute_indirect_recursion(
            syntax_rules,
            &recursive_heads,
            &ends,
            &rules_with_precedence_or_associativity,
        )?;
        compute_always_present_ends(&mut indirect_recursion, &rules_by_name);
        let associativity_arguments = compute_associativity_arguments(
            syntax_rules,
            &recursive_heads,
            &indirect_recursion,
            &ends,
        );
        // Only right and non-associative levels check a returned associativity
        // value on the left. Left-associative levels use the incoming `a` argument.
        let associativity_returns = syntax_rules
            .iter()
            .filter(|rule| {
                rule.priority_levels
                    .iter()
                    .zip(&associativity_arguments[&rule.head.name])
                    .any(|(level, &use_a)| {
                        use_a
                            && matches!(
                                level.associativity,
                                Some(Associativity::Right | Associativity::NonAssoc)
                            )
                    })
            })
            .map(|rule| rule.head.name.clone())
            .collect();
        let exclusion_targets: FxHashSet<String> = syntax_rules
            .iter()
            .filter(|r| r.head.parameters.iter().any(|p| p.name == "e"))
            .map(|r| r.head.name.clone())
            .collect();

        Ok(Self {
            recursive_heads,
            ends,
            exclusion_targets,
            indirect_recursion,
            associativity_arguments,
            associativity_returns,
        })
    }

    fn transform(&self, syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
        syntax_rules
            .into_iter()
            .map(|rule| {
                let rule = if self.recursive_heads.contains(&rule.head.name) {
                    self.desugar_rule(rule)
                } else if let Some(recursion) = self.indirect_recursion.get(&rule.head.name) {
                    self.desugar_indirect_rule(rule, recursion)
                } else {
                    rule
                };
                self.update_unrestricted_references(rule)
            })
            .collect()
    }

    /// Parameter names in call order. Exclusion desugaring supplies `e`; this
    /// transformation inserts the precedence parameters before it.
    fn parameter_names(&self, nonterminal: &str) -> impl Iterator<Item = &'static str> {
        [
            Some("p"),
            self.indirect_recursion
                .get(nonterminal)
                .is_some_and(IndirectRecursion::is_at_both_ends)
                .then_some("end"),
            self.needs_associativity_parameter(nonterminal)
                .then_some("a"),
            self.exclusion_targets.contains(nonterminal).then_some("e"),
        ]
        .into_iter()
        .flatten()
    }

    fn add_parameters(&self, head: Nonterminal) -> Nonterminal {
        let mut parameters: Vec<_> = self
            .parameter_names(&head.name)
            .filter(|&name| name != "e")
            .map(|name| Parameter {
                name: name.to_string(),
                ty: ParamType::I32,
            })
            .collect();
        parameters.extend(head.parameters);
        Nonterminal { parameters, ..head }
    }

    /// Arguments for a call without precedence, associativity, or exclusion restrictions.
    fn unrestricted_arguments(&self, nonterminal: &str) -> Vec<Expr> {
        self.end_call_args(
            nonterminal,
            Expr::Int(0),
            Expr::Int(end_value(End::Left)),
            0,
            Expr::Int(0),
        )
    }

    fn needs_associativity_parameter(&self, nonterminal: &str) -> bool {
        let head = self
            .indirect_recursion
            .get(nonterminal)
            .map_or(nonterminal, |recursion| recursion.head.as_str());
        self.associativity_arguments
            .get(head)
            .is_some_and(|levels| levels.contains(&true))
    }

    fn return_shape(&self, nonterminal: &str) -> return_value::ReturnShape {
        let head = self
            .indirect_recursion
            .get(nonterminal)
            .map_or(nonterminal, |recursion| recursion.head.as_str());
        return_value::ReturnShape {
            associativity: self.associativity_returns.contains(head),
            label: self.exclusion_targets.contains(nonterminal),
        }
    }

    /// Desugars a single precedence head and its recursive alternatives.
    fn desugar_rule(&self, rule: SyntaxRule) -> SyntaxRule {
        let head_name = rule.head.name.clone();
        let return_shape = self.return_shape(&head_name);
        let precedences = assign_precedence(
            &rule.priority_levels,
            &head_name,
            &self.recursive_heads,
            &self.ends,
        );

        // The minimum prefix precedence determines which alternatives return
        // the minimum precedence along their right-recursive chain.
        let min_prefix_pr =
            self.min_prefix_precedence(&rule.priority_levels, &precedences, &head_name);

        let mut all_alternatives = Vec::new();

        for ((level, precedence), use_a) in rule
            .priority_levels
            .into_iter()
            .zip(precedences.iter())
            .zip(&self.associativity_arguments[&head_name])
        {
            let assoc = level.associativity;
            for alt in level.alternatives {
                // The rewrite works on the grammar symbols without exclusion guards
                // or returns. The desugared alternative keeps the guards and returns
                // its own label alongside precedence.
                let (guard_prefix, core, trailing_label) = split_alt_decorations(alt);
                let recursion = classify(&core, &head_name, &self.recursive_heads, &self.ends);
                let make_return = |precedence, associativity| {
                    Symbol::Return(return_shape.value(precedence, associativity, trailing_label))
                };
                let rewritten = match (recursion, precedence) {
                    (RecursionKind::Binary, Some(pr)) => {
                        self.rewrite_binary(core, *pr, assoc, *use_a, min_prefix_pr, make_return)
                    }
                    (RecursionKind::Prefix, Some(pr)) => {
                        self.rewrite_prefix(core, *pr, min_prefix_pr, make_return)
                    }
                    (RecursionKind::Postfix, Some(pr)) => {
                        self.rewrite_postfix(core, *pr, make_return)
                    }
                    _ => {
                        // Non-recursive alternatives return precedence zero and this rule's label.
                        let mut symbols = core.symbols;
                        symbols.push(make_return(Expr::Int(0), Expr::Int(0)));
                        Alternative {
                            symbols,
                            label: core.label,
                        }
                    }
                };
                let mut symbols = guard_prefix;
                symbols.extend(rewritten.symbols);
                all_alternatives.push(Alternative {
                    symbols,
                    label: rewritten.label,
                });
            }
        }

        let head = self.add_parameters(rule.head);

        SyntaxRule {
            head,
            priority_levels: vec![PriorityLevel::new(all_alternatives)],
            layout: rule.layout,
        }
    }

    /// Finds the minimum precedence among prefix alternatives, direct (`op E`) and
    /// indirect (`op X`, where X derives E at its right end). A binary alternative
    /// with an optional left recursive end also counts: when that end is absent,
    /// the alternative acts as a prefix. Returns `None` if no alternative can act
    /// as a prefix.
    ///
    /// For example, if an `=` alternative has an optional left end and lower
    /// precedence than `*` and `+`, both higher alternatives use the minimum return
    /// calculation. A nested `=` can act as a prefix. An enclosing left-end check
    /// must compare against that lower precedence. The return uses `min(r_pr, pr)`
    /// when the right end returns a nonzero, defined precedence, as for a
    /// declared prefix.
    fn min_prefix_precedence(
        &self,
        priority_levels: &[PriorityLevel],
        precedences: &[Option<i64>],
        head: &str,
    ) -> Option<i64> {
        priority_levels
            .iter()
            .zip(precedences.iter())
            .filter_map(|(level, prec)| {
                let pr = (*prec)?;
                let has_prefix = level.alternatives.iter().any(|alt| {
                    let recursion = classify(alt, head, &self.recursive_heads, &self.ends);
                    recursion == RecursionKind::Prefix
                        || (recursion == RecursionKind::Binary
                            && first_grammar_symbol(&alt.symbols)
                                .and_then(Symbol::as_identifier)
                                .and_then(|id| self.indirect_recursion.get(&id.name))
                                .is_some_and(|recursion| {
                                    !recursion.is_always_present_at_end(End::Left)
                                }))
                });
                has_prefix.then_some(pr)
            })
            .min()
    }

    /// Arguments for a recursive end. An intermediate nonterminal used at both
    /// ends has an `end` argument selecting the end that receives `p`. Other
    /// targets omit this argument.
    fn end_call_args(
        &self,
        target: &str,
        p_arg: Expr,
        end_arg: Expr,
        local_n: i64,
        a_arg: Expr,
    ) -> Vec<Expr> {
        self.parameter_names(target)
            .map(|name| match name {
                "p" => p_arg.clone(),
                "end" => end_arg.clone(),
                "a" => a_arg.clone(),
                "e" => Expr::Int(local_n),
                _ => unreachable!("unknown precedence parameter"),
            })
            .collect()
    }

    /// Creates the left binding symbol: `l_pr=E(p)` (or `l_pr=E(p, 0)` for an exclusion target).
    /// For example, a left recursive reference `E !Add` calls `E` with `e = 0`,
    /// even though the reference excludes addition. References with different
    /// exclusions can share this call without splitting the GSS by exclusion
    /// context. The call may return an addition; the caller rejects that result
    /// by checking its label with `make_local_exclusion_postcondition`.
    /// A `label` from the original symbol is preserved inside the binding.
    fn make_left_binding(&self, id: &Identifier, label: Option<&str>) -> Symbol {
        Symbol::Binding {
            pattern: self.return_shape(&id.name).binding("l"),
            symbol: Box::new(label_symbol(
                label,
                Symbol::Call {
                    name: id.clone(),
                    arguments: self.end_call_args(
                        &id.name,
                        Expr::Ref("p".to_string()),
                        Expr::Int(end_value(End::Left)),
                        0,
                        Expr::Int(0),
                    ),
                },
            )),
        }
    }

    /// Rewrites a binary alternative `E op E` at precedence level `pr` into a
    /// data-dependent form.
    ///
    /// In the schematic examples below, `l_pr` and `r_pr` denote extracted precedence
    /// fields and `return pr` means `return (pr, own_label)` when the rule has labels.
    /// The exact rewrite depends on associativity:
    ///
    /// - No associativity (default):
    ///   `[pr>=p] l_pr=E(p) [l_pr==0||l_pr>=pr] op E(pr) return pr`
    ///
    /// - Left-associative:
    ///   `[pr>=p] l_pr=E(p) [l_pr==0||l_pr>=pr] op E(pr+1) return pr`
    ///   (right end gets pr+1 to prevent right-recursive use at same level)
    ///
    /// - Right-associative:
    ///   `[pr>=p] l_pr=E(p) [l_pr==0||l_pr>=pr+1] op E(pr) return pr`
    ///   (postcondition uses pr+1 to prevent left-recursive use at same level)
    ///
    /// - Non-associative:
    ///   `[pr>=p] l_pr=E(p) [l_pr==0||l_pr>=pr+1] op E(pr+1) return pr`
    ///   (both restrictions)
    ///
    /// A level mixing binary and unary alternatives, or a binary alternative
    /// whose recursive end can be absent, uses a separate associativity argument
    /// instead of the pr+1 adjustment above. The argument `a` identifies
    /// the precedence level of a binary alternative excluded on the right. The
    /// return includes an associativity field when a corresponding check on the left
    /// is needed.
    /// These checks distinguish binary alternatives from unary alternatives at
    /// the same level. An alternative participates only when both recursive ends
    /// are present.
    ///
    /// For example:
    ///
    /// ```text
    ///   E = "a"
    ///     > left L "+" R
    ///   L = E | "b"
    ///   R = E | "b"
    /// ```
    ///
    /// Addition has level 1. An argument of `a = 0` allows addition, while `a = 1`
    /// excludes it when both of its recursive `E`-ends are present. The intermediate
    /// nonterminals pass `a` to `E`, alongside `p`.
    ///
    /// This left-associative rule checks the incoming `a` argument. No alternative
    /// checks an associativity value returned by the left call, so the returns
    /// contain only precedence. The initial call is `E(0, 0)`:
    ///
    /// ```text
    ///   E(p: i32, a: i32)
    ///     = "a"  return 0
    ///     | l_pr=L(p, 0) [l_pr == UNDEFINED_PRECEDENCE || (1 >= p && (l_pr == 0 || l_pr >= 1))] "+" r_pr=R(1, l_pr == UNDEFINED_PRECEDENCE ? 0 : 1) [l_pr == UNDEFINED_PRECEDENCE || r_pr == UNDEFINED_PRECEDENCE || a != 1]  return r_pr == UNDEFINED_PRECEDENCE ? 0 : 1
    ///
    ///   L(p: i32, a: i32)
    ///     = l_pr=E(p, a)  return l_pr
    ///     | "b"           return UNDEFINED_PRECEDENCE
    ///
    ///   R(p: i32, a: i32)
    ///     = r_pr=E(p, a)  return r_pr
    ///     | "b"           return UNDEFINED_PRECEDENCE
    /// ```
    ///
    /// The left call `L(p, 0)` allows another addition on the left. The argument
    /// passed to `R` depends on the result from `L`. If `L` derives a left
    /// `E`-end, `R` receives `a = 1` to exclude a binary addition on the right.
    /// If `L` parses `"b"`, `R` receives `a = 0` because the enclosing addition has
    /// no left `E`-end.
    ///
    /// The condition after `R` checks the enclosing addition against the `a`
    /// received from its caller. If either recursive `E`-end is absent, the
    /// condition succeeds regardless of `a`. Otherwise, `a != 1` requires
    /// the caller to allow this binary addition. The check waits for both calls
    /// to return because either call can parse `"b"` without deriving `E`.
    ///
    /// For right and non-associative levels, the left-end check compares the
    /// returned associativity value with `pr`. A binary alternative with both recursive
    /// ends present returns its own level in that field, even when its returned
    /// precedence reflects a lower-precedence prefix. An absent recursive end gives
    /// associativity zero.
    fn rewrite_binary(
        &self,
        alt: Alternative,
        pr: i64,
        assoc: Option<Associativity>,
        use_a: bool,
        min_prefix_pr: Option<i64>,
        make_return: impl Fn(Expr, Expr) -> Symbol,
    ) -> Alternative {
        let left = alt.symbols.first().unwrap();
        let right = alt.symbols.last().unwrap();
        let left_id = extract_identifier(left);
        let right_id = extract_identifier(right);
        let optional_left = self
            .indirect_recursion
            .get(&left_id.name)
            .is_some_and(|recursion| !recursion.is_always_present_at_end(End::Left));
        let optional_right = self
            .indirect_recursion
            .get(&right_id.name)
            .is_some_and(|recursion| !recursion.is_always_present_at_end(End::Right));
        let left_wrappers = end_wrappers(left);
        let right_wrappers = end_wrappers(right);
        let restrict_left = matches!(assoc, Some(Associativity::Right | Associativity::NonAssoc));
        let restrict_right = matches!(assoc, Some(Associativity::Left | Associativity::NonAssoc));
        let left_lower_bound = pr + i64::from(!use_a && restrict_left);
        let right_arg = pr + i64::from(!use_a && restrict_right);
        let use_min = min_prefix_pr.is_some_and(|min| pr > min);
        let absent = match (optional_left, optional_right) {
            (true, true) => Some(Expr::Or(
                Box::new(undefined_precedence_condition("l")),
                Box::new(undefined_precedence_condition("r")),
            )),
            (true, false) => Some(undefined_precedence_condition("l")),
            (false, true) => Some(undefined_precedence_condition("r")),
            (false, false) => None,
        };
        let mut deferred_checks = Vec::new();
        let mut symbols = Vec::new();
        if !optional_left {
            symbols.push(make_precondition(pr));
        }
        if use_a && restrict_right {
            let incoming = compare(Expr::Ref("a".to_string()), CondOp::Neq, Expr::Int(pr));
            if absent.is_some() {
                deferred_checks.push(incoming);
            } else {
                symbols.push(Symbol::Condition(incoming));
            }
        }
        symbols.push(Symbol::restricted(
            self.make_left_binding(left_id, left_wrappers.label.as_deref()),
            left_wrappers.restrictions,
        ));
        symbols.push(left_postcondition(pr, left_lower_bound, optional_left));
        let left_mask = extract_exclusion_mask(left);
        if self.exclusion_targets.contains(&left_id.name) && left_mask != 0 {
            symbols.push(make_local_exclusion_postcondition(left_mask, "l"));
        }
        if use_a && restrict_left {
            let left_associativity =
                compare(Expr::Ref("l_assoc".to_string()), CondOp::Neq, Expr::Int(pr));
            if absent.is_some() {
                deferred_checks.push(left_associativity);
            } else {
                symbols.push(Symbol::Condition(left_associativity));
            }
        }

        let right_a = if use_a && restrict_right {
            if optional_left {
                choose(
                    undefined_precedence_condition("l"),
                    Expr::Int(0),
                    Expr::Int(pr),
                )
            } else {
                Expr::Int(pr)
            }
        } else {
            Expr::Int(0)
        };
        let mut right_call = label_symbol(
            right_wrappers.label.as_deref(),
            Symbol::Call {
                name: right_id.clone(),
                arguments: self.end_call_args(
                    &right_id.name,
                    Expr::Int(right_arg),
                    Expr::Int(end_value(End::Right)),
                    extract_exclusion_mask(right),
                    right_a,
                ),
            },
        );
        if use_a || use_min || optional_right {
            right_call = Symbol::Binding {
                pattern: self.return_shape(&right_id.name).binding("r"),
                symbol: Box::new(right_call),
            };
        }
        let count = alt.symbols.len();
        symbols.extend(alt.symbols.into_iter().skip(1).take(count - 2));
        symbols.push(Symbol::restricted(right_call, right_wrappers.restrictions));

        // If either recursive end can be absent, the associativity checks wait
        // for both calls to return. An absent end skips these checks and gives
        // associativity zero.
        if let Some(absent) = &absent {
            for check in deferred_checks {
                symbols.push(Symbol::Condition(Expr::Or(
                    Box::new(absent.clone()),
                    Box::new(check),
                )));
            }
        }
        let associativity = if use_a && restrict_left {
            match absent {
                Some(absent) => choose(absent, Expr::Int(0), Expr::Int(pr)),
                None => Expr::Int(pr),
            }
        } else {
            Expr::Int(0)
        };
        symbols.push(make_return(
            right_precedence(pr, use_min, optional_right),
            associativity,
        ));
        Alternative {
            symbols,
            label: alt.label,
        }
    }

    /// Rewrites a prefix alternative `op E` at precedence level `pr` into:
    ///   op E(pr) return pr
    /// With a prefix at lower precedence, the return uses `min`:
    ///   op r_pr=E(pr) return r_pr == 0 ? pr : min(r_pr, pr)
    /// An absent right recursive end returns zero, as computed by `right_precedence`.
    /// These examples show unlabeled rules. A rule with labels also returns its own label.
    fn rewrite_prefix(
        &self,
        alt: Alternative,
        pr: i64,
        min_prefix_pr: Option<i64>,
        make_return: impl Fn(Expr, Expr) -> Symbol,
    ) -> Alternative {
        let mut symbols = Vec::new();
        let num_symbols = alt.symbols.len();

        let use_min = min_prefix_pr.is_some_and(|min_pr| pr > min_pr);

        let right_id = extract_identifier(alt.symbols.last().unwrap()).clone();
        let optional_right = self
            .indirect_recursion
            .get(&right_id.name)
            .is_some_and(|recursion| !recursion.is_always_present_at_end(End::Right));
        let right_local_n = extract_exclusion_mask(alt.symbols.last().unwrap());
        let right = end_wrappers(alt.symbols.last().unwrap());

        symbols.extend(alt.symbols.into_iter().take(num_symbols.saturating_sub(1)));

        let call = label_symbol(
            right.label.as_deref(),
            Symbol::Call {
                name: right_id.clone(),
                arguments: self.end_call_args(
                    &right_id.name,
                    Expr::Int(pr),
                    Expr::Int(end_value(End::Right)),
                    right_local_n,
                    Expr::Int(0),
                ),
            },
        );
        let call = if use_min || optional_right {
            Symbol::Binding {
                pattern: self.return_shape(&right_id.name).binding("r"),
                symbol: Box::new(call),
            }
        } else {
            call
        };
        symbols.push(Symbol::restricted(call, right.restrictions));
        symbols.push(make_return(
            right_precedence(pr, use_min, optional_right),
            Expr::Int(0),
        ));

        Alternative {
            symbols,
            label: alt.label,
        }
    }

    /// Rewrites a postfix alternative `E op` at precedence level `pr` into:
    ///   [pr>=p] l_pr=E(p) [l_pr==0||l_pr>=pr] op return 0
    /// If the left recursive end can be absent, `left_postcondition` defers both
    /// precedence checks until the call returns and skips them when the end is absent.
    /// A rule with labels also returns its own label.
    fn rewrite_postfix(
        &self,
        alt: Alternative,
        pr: i64,
        make_return: impl Fn(Expr, Expr) -> Symbol,
    ) -> Alternative {
        let left_id = extract_identifier(alt.symbols.first().unwrap()).clone();
        let optional_left = self
            .indirect_recursion
            .get(&left_id.name)
            .is_some_and(|recursion| !recursion.is_always_present_at_end(End::Left));
        let left_has_e = self.exclusion_targets.contains(&left_id.name);
        let left_local_n = extract_exclusion_mask(alt.symbols.first().unwrap());
        let left = end_wrappers(alt.symbols.first().unwrap());

        let mut symbols = Vec::new();

        if !optional_left {
            symbols.push(make_precondition(pr));
        }
        symbols.push(Symbol::restricted(
            self.make_left_binding(&left_id, left.label.as_deref()),
            left.restrictions.clone(),
        ));
        symbols.push(left_postcondition(pr, pr, optional_left));
        if left_has_e && left_local_n != 0 {
            symbols.push(make_local_exclusion_postcondition(left_local_n, "l"));
        }

        symbols.extend(alt.symbols.into_iter().skip(1));

        symbols.push(make_return(Expr::Int(0), Expr::Int(0)));

        Alternative {
            symbols,
            label: alt.label,
        }
    }

    /// Desugars an intermediate nonterminal at a left or right recursive end.
    /// The nonterminal passes `p` toward the head and returns the precedence and,
    /// when needed, associativity from that recursive end, together with its own label.
    /// If the nonterminal occurs at both ends, an additional `end` parameter selects
    /// whether the call restricts the left end (`0`) or right end (`1`).
    fn desugar_indirect_rule(&self, rule: SyntaxRule, recursion: &IndirectRecursion) -> SyntaxRule {
        let SyntaxRule {
            head: rule_head,
            priority_levels,
            layout,
        } = rule;
        let has_a = self.needs_associativity_parameter(&rule_head.name);
        let return_shape = self.return_shape(&rule_head.name);
        let mut alternatives = Vec::new();
        for alt in priority_levels
            .into_iter()
            .flat_map(|level| level.alternatives)
        {
            let (mut symbols, core, trailing_label) = split_alt_decorations(alt);
            let find_end = |end| {
                end_index(&core.symbols, end).filter(|&i| {
                    recursion.is_at_end(end)
                        && core.symbols[i].as_identifier().is_some_and(|id| {
                            id.name == recursion.head
                                || self.indirect_recursion.get(&id.name).is_some_and(|next| {
                                    next.head == recursion.head && next.is_at_end(end)
                                })
                        })
                })
            };
            let left = find_end(End::Left);
            let right = find_end(End::Right);
            let mut left_value = return_value::undefined_precedence();
            let mut right_value = return_value::undefined_precedence();
            let mut left_associativity = Expr::Int(0);
            let mut right_associativity = Expr::Int(0);
            for (i, symbol) in core.symbols.into_iter().enumerate() {
                let is_left = left == Some(i);
                let is_right = right == Some(i);
                if !is_left && !is_right {
                    symbols.push(symbol);
                    continue;
                }
                let id = extract_identifier(&symbol);
                let end = if is_left { End::Left } else { End::Right };
                let both = is_left && is_right;
                let binding = if both {
                    "v"
                } else if is_left {
                    "l"
                } else {
                    "r"
                };
                let p_arg = if both || !recursion.is_at_both_ends() {
                    Expr::Ref("p".to_string())
                } else {
                    precedence_for_end(end)
                };
                // A symbol at both ends is one call. Its end argument selects
                // the same end in the next intermediate nonterminal.
                let end_arg = if both {
                    Expr::Ref("end".to_string())
                } else {
                    Expr::Int(end_value(end))
                };
                let value = Expr::Ref(format!("{binding}_pr"));
                let associativity = if self.associativity_returns.contains(&recursion.head) {
                    Expr::Ref(format!("{binding}_assoc"))
                } else {
                    Expr::Int(0)
                };
                let a_arg = if !has_a {
                    Expr::Int(0)
                } else if both || !recursion.is_at_both_ends() {
                    Expr::Ref("a".to_string())
                } else {
                    choose(
                        end_selection_condition(end),
                        Expr::Ref("a".to_string()),
                        Expr::Int(0),
                    )
                };
                let local_n = if self.exclusion_targets.contains(&id.name) {
                    extract_exclusion_mask(&symbol)
                } else {
                    0
                };
                // Left calls pass `e = 0` and check exclusions on the returned label.
                // This also applies to a single call at both ends.
                symbols.push(self.make_indirect_end_binding(
                    &symbol,
                    binding,
                    p_arg,
                    end_arg,
                    a_arg,
                    if is_left { 0 } else { local_n },
                ));
                if is_left && local_n != 0 {
                    symbols.push(make_local_exclusion_postcondition(local_n, binding));
                }
                if is_left {
                    left_value = value.clone();
                    left_associativity = associativity.clone();
                }
                if is_right {
                    right_value = value;
                    right_associativity = associativity;
                }
            }
            symbols.push(Symbol::Return(return_shape.value(
                selected_end_value(recursion, left_value, right_value),
                selected_end_value(recursion, left_associativity, right_associativity),
                trailing_label,
            )));
            alternatives.push(Alternative {
                symbols,
                label: core.label,
            });
        }

        SyntaxRule {
            head: self.add_parameters(rule_head),
            priority_levels: vec![PriorityLevel::new(alternatives)],
            layout,
        }
    }

    fn make_indirect_end_binding(
        &self,
        original: &Symbol,
        binding: &str,
        p_arg: Expr,
        end_argument: Expr,
        a_arg: Expr,
        local_n: i64,
    ) -> Symbol {
        let id = extract_identifier(original);
        let wrappers = end_wrappers(original);
        let call = Symbol::Call {
            name: id.clone(),
            arguments: self.end_call_args(&id.name, p_arg, end_argument, local_n, a_arg),
        };
        Symbol::restricted(
            Symbol::Binding {
                pattern: self.return_shape(&id.name).binding(binding),
                symbol: Box::new(label_symbol(wrappers.label.as_deref(), call)),
            },
            wrappers.restrictions,
        )
    }

    /// Supplies unrestricted arguments to references in every rule, including
    /// references inside labels, bindings, and restrictions. For example, the
    /// reference in `"(" E ")"` becomes `E(0)` when E takes only precedence.
    /// An exclusion call such as `E(4)` keeps its mask after the added parameters.
    ///
    /// Bare references receive zero for each inserted parameter. Calls with missing
    /// arguments receive the same defaults before their supplied arguments.
    /// Calls already at full arity keep their arguments, including the restrictions
    /// inserted at recursive ends.
    fn update_unrestricted_references(&self, rule: SyntaxRule) -> SyntaxRule {
        transform_syntax_rule(rule, |symbol| {
            transform_symbol(symbol, &mut |symbol| match symbol {
                Symbol::Identifier(id)
                    if self.recursive_heads.contains(&id.name)
                        || self.indirect_recursion.contains_key(&id.name) =>
                {
                    let arguments = self.unrestricted_arguments(&id.name);
                    Symbol::Call {
                        name: Identifier {
                            name: id.name.clone(),
                            definition: id.definition,
                        },
                        arguments,
                    }
                }
                Symbol::Call { name, arguments }
                    if self.recursive_heads.contains(&name.name)
                        || self.indirect_recursion.contains_key(&name.name) =>
                {
                    let mut defaults = self.unrestricted_arguments(&name.name);
                    if arguments.len() < defaults.len() {
                        defaults.truncate(defaults.len() - arguments.len());
                        defaults.extend(arguments);
                        Symbol::Call {
                            name,
                            arguments: defaults,
                        }
                    } else {
                        Symbol::Call { name, arguments }
                    }
                }
                other => other,
            })
        })
    }
}

/// Discovers recursion back to each precedence rule's own head.
fn compute_recursive_heads(syntax_rules: &[SyntaxRule], ends: &Ends) -> Vec<String> {
    syntax_rules
        .iter()
        .filter(|rule| needs_desugaring(rule))
        .filter(|rule| has_recursive_path(rule, ends))
        .map(|rule| rule.head.name.clone())
        .collect()
}

/// Collects the indirect recursion information for each intermediate nonterminal.
/// For this grammar, `Lambda` and `Body` both derive sequences ending with `E`:
///
///   E = 'a' > Lambda
///   Lambda = 'fn' Body
///   Body = E
///
/// The result maps each intermediate nonterminal to its precedence head and records
/// whether it is used as a left or right recursive end. Here, `Lambda` is used as
/// a right recursive end of `E`, and `Body` is used at the right end of `Lambda`.
/// Both entries therefore have head `E` and only `right` set.
///
/// The walk follows each recursive end of an alternative:
///
/// - A direct reference to the head needs no entry for an intermediate nonterminal.
/// - An indirect reference starts a walk through the corresponding end symbols.
///   Each nonterminal on a path back to the head gets an entry.
/// - A rule with its own precedence or associativity stops the walk. Its
///   precedence levels are independent of the head's levels.
///
/// A nonterminal reached from two different heads is an error. A single `p`
/// parameter cannot represent the precedence levels of two different heads.
fn compute_indirect_recursion(
    syntax_rules: &[SyntaxRule],
    recursive_heads: &[String],
    ends: &Ends,
    rules_with_precedence_or_associativity: &[String],
) -> Result<FxHashMap<String, IndirectRecursion>, Vec<String>> {
    let priority: FxHashSet<&str> = rules_with_precedence_or_associativity
        .iter()
        .map(String::as_str)
        .collect();

    let mut indirect_recursion: FxHashMap<String, IndirectRecursion> = FxHashMap::default();
    let mut multi_head: FxHashSet<&str> = FxHashSet::default();
    for rule in syntax_rules {
        if !recursive_heads.contains(&rule.head.name) {
            continue;
        }
        let head = rule.head.name.as_str();
        for alt in rule.alternatives() {
            let recursion = classify(alt, head, recursive_heads, ends);
            let recursive_ends = [End::Left, End::Right].into_iter().filter(|&end| {
                matches!(
                    (recursion, end),
                    (RecursionKind::Binary, _)
                        | (RecursionKind::Prefix, End::Right)
                        | (RecursionKind::Postfix, End::Left)
                )
            });

            for end in recursive_ends {
                let mut stack: Vec<&str> =
                    non_head_nonterminal(end_symbol(&alt.symbols, end), head)
                        .into_iter()
                        .collect();
                let mut visited = FxHashSet::default();
                while let Some(node) = stack.pop() {
                    if !visited.insert(node)
                        || node == head
                        || priority.contains(node)
                        || !ends.reaches(node, head, end, recursive_heads)
                    {
                        continue;
                    }
                    let recursion = indirect_recursion
                        .entry(node.to_string())
                        .or_insert_with(|| IndirectRecursion::new(head));
                    if recursion.head != head {
                        multi_head.insert(node);
                    }
                    recursion.mark_end(end);
                    if let Some(steps) = ends.direct(end).get(node) {
                        stack.extend(steps.iter().map(String::as_str));
                    }
                }
            }
        }
    }

    if multi_head.is_empty() {
        return Ok(indirect_recursion);
    }
    let mut names: Vec<&str> = multi_head.into_iter().collect();
    names.sort_unstable();
    Err(names
        .into_iter()
        .map(|name| {
            format!(
                "indirect precedence cannot be enforced for `{name}` because it serves multiple precedence heads; give each precedence rule its own intermediate nonterminal"
            )
        })
        .collect())
}

/// Determines whether each recursive end is present in every derivation through
/// its intermediate nonterminal. The greatest fixed point starts with all recorded
/// ends as required and removes an end if any alternative can lack it. The fixed
/// point also handles pass-through cycles whose productive exits all derive
/// the recursive end.
fn compute_always_present_ends(
    indirect: &mut FxHashMap<String, IndirectRecursion>,
    rules: &FxHashMap<&str, &SyntaxRule>,
) {
    for end in [End::Left, End::Right] {
        let mut required: FxHashSet<String> = indirect
            .iter()
            .filter(|(_, recursion)| recursion.is_at_end(end))
            .map(|(name, _)| name.clone())
            .collect();
        loop {
            let remove: Vec<_> = required
                .iter()
                .filter(|name| {
                    let head = &indirect[*name].head;
                    rules[name.as_str()].alternatives().any(|alt| {
                        !end_symbol(&alt.symbols, end)
                            .and_then(Symbol::as_identifier)
                            .is_some_and(|id| {
                                &id.name == head
                                    || (required.contains(&id.name)
                                        && indirect[&id.name].head == *head)
                            })
                    })
                })
                .cloned()
                .collect();
            if remove.is_empty() {
                break;
            }
            for name in remove {
                required.remove(&name);
            }
        }
        for (name, recursion) in indirect.iter_mut() {
            match end {
                End::Left => recursion.left_required = required.contains(name),
                End::Right => recursion.right_required = required.contains(name),
            }
        }
    }
}

/// Identifies the levels that need a separate associativity argument. The pr+1
/// adjustment applies when all recursive alternatives in a level are binary
/// and both ends are always present. A level mixing binary and unary alternatives,
/// or containing a binary alternative whose recursive end can be absent, uses `a`
/// to restrict only alternatives with both recursive ends present.
fn compute_associativity_arguments(
    rules: &[SyntaxRule],
    recursive_heads: &[String],
    indirect: &FxHashMap<String, IndirectRecursion>,
    ends: &Ends,
) -> FxHashMap<String, Vec<bool>> {
    rules
        .iter()
        .map(|rule| {
            let levels = rule
                .priority_levels
                .iter()
                .map(|level| {
                    if level.associativity.is_none() {
                        return false;
                    }
                    let mut binary = false;
                    let mut unary = false;
                    for alt in &level.alternatives {
                        match classify(alt, &rule.head.name, recursive_heads, ends) {
                            RecursionKind::Binary => {
                                binary = true;
                                unary |= [End::Left, End::Right].into_iter().any(|end| {
                                    end_symbol(&alt.symbols, end)
                                        .and_then(Symbol::as_identifier)
                                        .and_then(|id| indirect.get(&id.name))
                                        .is_some_and(|recursion| {
                                            !recursion.is_always_present_at_end(end)
                                        })
                                });
                            }
                            RecursionKind::NonRecursive => {}
                            _ => unary = true,
                        }
                    }
                    binary && unary
                })
                .collect();
            (rule.head.name.clone(), levels)
        })
        .collect()
}

/// Finds a recursive path through either end, before precedence boundaries
/// are applied. A rule with declared precedence remains an independent head
/// even if its only recursive path passes through another precedence head.
/// Classification later leaves references to that other head unrestricted.
fn has_recursive_path(rule: &SyntaxRule, ends: &Ends) -> bool {
    has_direct_recursive_end(rule, &rule.head.name)
        || rule.alternatives().any(|alt| {
            recurses_indirectly(alt, &rule.head.name, ends, End::Left)
                || recurses_indirectly(alt, &rule.head.name, ends, End::Right)
        })
}

/// True when an alternative directly references the given nonterminal at either end.
/// Indirect recursion is discovered separately through the end graph.
fn has_direct_recursive_end(rule: &SyntaxRule, name: &str) -> bool {
    rule.alternatives().any(|alt| {
        is_reference_to(first_grammar_symbol(&alt.symbols), name)
            || is_reference_to(last_grammar_symbol(&alt.symbols), name)
    })
}

/// A rule needs desugaring if it declares precedence (more than one priority
/// level) or associativity.
fn needs_desugaring(rule: &SyntaxRule) -> bool {
    rule.priority_levels.len() > 1
        || rule
            .priority_levels
            .iter()
            .any(|level| level.associativity.is_some())
}

/// True if the selected end recurses into `head` through another nonterminal,
/// like `Lambda` deriving a sequence ending with `Expression`. A reference to `head`
/// itself is direct recursion and returns false here.
fn recurses_indirectly(alternative: &Alternative, head: &str, ends: &Ends, end: End) -> bool {
    non_head_nonterminal(end_symbol(&alternative.symbols, end), head)
        .is_some_and(|name| ends.reaches(name, head, end, &[]))
}

/// The nonterminal at this symbol position, unless it is `head` itself.
fn non_head_nonterminal<'a>(symbol: Option<&'a Symbol>, head: &str) -> Option<&'a str> {
    symbol
        .and_then(Symbol::as_identifier)
        .map(|id| id.name.as_str())
        .filter(|&name| name != head)
}

/// Classifies direct and indirect recursion back to the same head.
///
/// Classification uses the first and last grammar symbols, ignoring conditions
/// and returns added by exclusion desugaring. A nullable symbol before `E`
/// does not make `E` a left end: `"-"? E "+" E` is a prefix alternative.
/// The same rule applies to intermediate nonterminals.
/// In `L = E Opt`, `E` is not a right end even if `Opt` is nullable. When
/// `Opt` derives non-empty input, a right-end restriction on `E` would be
/// incorrect. Classification does not depend on what `Opt` derives at runtime.
fn classify(
    alternative: &Alternative,
    head: &str,
    recursive_heads: &[String],
    ends: &Ends,
) -> RecursionKind {
    let recurses = |end| {
        is_reference_to(end_symbol(&alternative.symbols, end), head)
            || has_indirect_end(alternative, head, recursive_heads, ends, end)
    };
    let left = recurses(End::Left);
    let right = recurses(End::Right);
    if left && right && grammar_symbol_count(alternative) == 1 {
        // An alternative such as `E > Ternary`, `Ternary = E "?" E ":" E`, is
        // one call. It remains unrestricted; the binary rewrite requires
        // distinct left and right ends.
        return RecursionKind::NonRecursive;
    }
    match (left, right) {
        (true, true) => RecursionKind::Binary,
        (false, true) => RecursionKind::Prefix,
        (true, false) => RecursionKind::Postfix,
        (false, false) => RecursionKind::NonRecursive,
    }
}

/// Indirect recursion used for alternative classification must not cross into a
/// different desugared precedence nonterminal. That target defines its own
/// precedence levels and cannot interpret this rule's precedence number.
fn has_indirect_end(
    alternative: &Alternative,
    head: &str,
    recursive_heads: &[String],
    ends: &Ends,
    end: End,
) -> bool {
    non_head_nonterminal(end_symbol(&alternative.symbols, end), head)
        .is_some_and(|name| ends.reaches(name, head, end, recursive_heads))
}

fn grammar_symbol_count(alternative: &Alternative) -> usize {
    alternative
        .symbols
        .iter()
        .filter(|symbol| !is_decoration(symbol))
        .count()
}

fn is_decoration(symbol: &Symbol) -> bool {
    matches!(symbol, Symbol::Condition(_) | Symbol::Return(_))
}

fn first_grammar_symbol(symbols: &[Symbol]) -> Option<&Symbol> {
    symbols.iter().find(|s| !is_decoration(s))
}

fn last_grammar_symbol(symbols: &[Symbol]) -> Option<&Symbol> {
    symbols.iter().rev().find(|s| !is_decoration(s))
}

fn end_symbol(symbols: &[Symbol], end: End) -> Option<&Symbol> {
    match end {
        End::Left => first_grammar_symbol(symbols),
        End::Right => last_grammar_symbol(symbols),
    }
}

/// Exact head reference, including labeled, restricted, and exclusion calls.
fn is_reference_to(symbol: Option<&Symbol>, head: &str) -> bool {
    symbol
        .and_then(Symbol::as_identifier)
        .is_some_and(|id| id.name == head)
}

/// The nonterminals at the first and last grammar-symbol positions of each
/// alternative. For `A = B C`, the left relation contains `A -> B` and the
/// right relation contains `A -> C`. Reachability follows these direct edges.
struct Ends {
    direct_left: FxHashMap<String, FxHashSet<String>>,
    direct_right: FxHashMap<String, FxHashSet<String>>,
}

impl Ends {
    /// True when a nonempty path from `from` reaches `head` at the selected end.
    /// A precedence head in `boundaries` stops the walk unless it is the target.
    /// Discovery passes no boundaries; alternative classification passes the
    /// precedence heads whose levels must remain independent.
    fn reaches(&self, from: &str, head: &str, end: End, boundaries: &[String]) -> bool {
        let is_boundary = |name: &str| boundaries.iter().any(|boundary| boundary == name);
        if from != head && is_boundary(from) {
            return false;
        }
        let relation = self.direct(end);
        let mut stack: Vec<&str> = relation
            .get(from)
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect();
        let mut visited = FxHashSet::default();
        while let Some(node) = stack.pop() {
            if node == head {
                return true;
            }
            if !visited.insert(node) || is_boundary(node) {
                continue;
            }
            if let Some(next) = relation.get(node) {
                stack.extend(next.iter().map(String::as_str));
            }
        }
        false
    }

    /// The direct relation for the selected end.
    fn direct(&self, end: End) -> &FxHashMap<String, FxHashSet<String>> {
        match end {
            End::Left => &self.direct_left,
            End::Right => &self.direct_right,
        }
    }
}

/// Builds the left- and right-end relations from the first and last grammar
/// symbols. A nullable symbol still occupies its end position: whether it derives
/// empty is known only while parsing.
fn compute_ends(syntax_rules: &[SyntaxRule]) -> Ends {
    let mut direct_left: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();
    let mut direct_right: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();
    for rule in syntax_rules {
        for alt in rule.alternatives() {
            for (end, relation) in [
                (End::Left, &mut direct_left),
                (End::Right, &mut direct_right),
            ] {
                if let Some(id) = end_symbol(&alt.symbols, end).and_then(Symbol::as_identifier) {
                    relation
                        .entry(rule.head.name.clone())
                        .or_default()
                        .insert(id.name.clone());
                }
            }
        }
    }
    Ends {
        direct_left,
        direct_right,
    }
}

/// The position of the first or last grammar symbol, excluding decorations.
fn end_index(symbols: &[Symbol], end: End) -> Option<usize> {
    match end {
        End::Left => symbols.iter().position(|s| !is_decoration(s)),
        End::Right => symbols.iter().rposition(|s| !is_decoration(s)),
    }
}

/// Assigns precedence numbers to priority levels in reverse order.
/// Bottom level = 1, each `>` between recursive priority groups increments the level.
/// Levels whose alternatives all lack recursion (direct and indirect) get `None`.
fn assign_precedence(
    priority_levels: &[PriorityLevel],
    head: &str,
    recursive_heads: &[String],
    ends: &Ends,
) -> Vec<Option<i64>> {
    let mut result = vec![Option::<i64>::None; priority_levels.len()];
    let mut next_precedence: i64 = 1;

    // Precedence increases from bottom to top.
    for i in (0..priority_levels.len()).rev() {
        let has_recursive = priority_levels[i]
            .alternatives
            .iter()
            .any(|alt| classify(alt, head, recursive_heads, ends) != RecursionKind::NonRecursive);
        if has_recursive {
            result[i] = Some(next_precedence);
            next_precedence += 1;
        }
    }

    result
}

/// Separates the conditions and return added by exclusion desugaring from
/// the grammar symbols. The precedence rewrite preserves the current rule's
/// label when it constructs the new return.
fn split_alt_decorations(alt: Alternative) -> (Vec<Symbol>, Alternative, Option<usize>) {
    let mut symbols = alt.symbols;
    let trailing_label = return_value::take_exclusion_return(&mut symbols);

    let leading_count = symbols
        .iter()
        .position(|s| !matches!(s, Symbol::Condition(_)))
        .unwrap_or(symbols.len());
    let rest = symbols.split_off(leading_count);
    let prefix = symbols;

    (
        prefix,
        Alternative {
            symbols: rest,
            label: alt.label,
        },
        trailing_label,
    )
}

fn compare(left: Expr, op: CondOp, right: Expr) -> Expr {
    Expr::Cond(Cond {
        left: Box::new(left),
        right: Box::new(right),
        op,
    })
}

/// Builds a condition testing whether a bound return's precedence is undefined.
fn undefined_precedence_condition(binding: &str) -> Expr {
    compare(
        Expr::Ref(format!("{binding}_pr")),
        CondOp::Eq,
        return_value::undefined_precedence(),
    )
}

fn choose(cond: Expr, then: Expr, otherwise: Expr) -> Expr {
    Expr::Ternary {
        cond: Box::new(cond),
        then: Box::new(then),
        r#else: Box::new(otherwise),
    }
}

/// Computes the head's return from the value returned by its right end.
/// An absent right end gives a precedence return of zero.
///
/// With a lower-precedence prefix alternative, `use_min` makes the return
/// `r_pr == 0 ? pr : min(r_pr, pr)`, where `r_pr` is the precedence returned
/// by the right end. This returns the lowest precedence along the chain of right
/// recursive ends, allowing an enclosing left-end check to restrict the
/// nested prefix. Otherwise, a present right end gives return precedence `pr`.
fn right_precedence(pr: i64, use_min: bool, optional: bool) -> Expr {
    let r_pr = Expr::Ref("r_pr".to_string());
    let precedence_value = if use_min {
        Expr::Ternary {
            cond: Box::new(Expr::Cond(Cond {
                left: Box::new(r_pr.clone()),
                right: Box::new(Expr::Int(0)),
                op: CondOp::Eq,
            })),
            then: Box::new(Expr::Int(pr)),
            r#else: Box::new(Expr::Min(Box::new(r_pr), Box::new(Expr::Int(pr)))),
        }
    } else {
        Expr::Int(pr)
    };
    if optional {
        choose(
            undefined_precedence_condition("r"),
            Expr::Int(0),
            precedence_value,
        )
    } else {
        precedence_value
    }
}

/// Creates the precedence postcondition for the left recursive end.
///
/// In `L "+" R`, with `L = E | "b"`, the alternative parsed by `L`
/// determines whether the addition's left-end precedence checks apply.
/// If `L` parses `"b"`, there is no left `E`-end to restrict. The addition
/// skips both `pr >= p` and the comparison with the precedence returned by `L`.
///
/// When `optional` is true, the rewrite omits the separate precondition.
/// This postcondition combines both checks after the call to `L`, where its
/// return value is available. An undefined precedence allows the parse to
/// continue; a defined precedence requires both checks to succeed.
fn left_postcondition(pr: i64, lower_bound: i64, optional: bool) -> Symbol {
    let post = make_postcondition(lower_bound);
    if optional {
        Symbol::Condition(Expr::Or(
            Box::new(undefined_precedence_condition("l")),
            Box::new(Expr::And(
                Box::new(compare(
                    Expr::Int(pr),
                    CondOp::Geq,
                    Expr::Ref("p".to_string()),
                )),
                Box::new(post),
            )),
        ))
    } else {
        Symbol::Condition(post)
    }
}

/// Reads the first call argument as an exclusion mask, or returns zero if it
/// is not an integer. Exclusion desugaring puts the mask in this position for
/// nonterminals with an `e` parameter. A user argument in the same position
/// has no exclusion meaning unless the nonterminal has that parameter.
fn extract_exclusion_mask(symbol: &Symbol) -> i64 {
    if let Some(Expr::Int(bitmask)) = symbol.call_arguments().first() {
        *bitmask
    } else {
        0
    }
}

/// The identifier at a recursive end, seen through any label or restriction
/// wrapper. Panics if the end has no identifier.
fn extract_identifier(symbol: &Symbol) -> &Identifier {
    symbol
        .as_identifier()
        .unwrap_or_else(|| panic!("expected an identifier at recursive end, got {symbol:?}"))
}

/// The field label and restrictions of a recursive-end symbol.
/// `label` is the outermost field label, if any.
struct EndWrappers {
    restrictions: Restrictions,
    label: Option<String>,
}

fn end_wrappers(symbol: &Symbol) -> EndWrappers {
    let mut restrictions = Restrictions::default();
    let mut label = None;
    let mut current = symbol;
    loop {
        match current {
            Symbol::Labeled { label: l, symbol } => {
                label.get_or_insert_with(|| l.clone());
                current = symbol;
            }
            Symbol::Restricted {
                symbol,
                restrictions: r,
            } => {
                restrictions = r.clone();
                current = symbol;
            }
            _ => break,
        }
    }
    EndWrappers {
        restrictions,
        label,
    }
}

fn label_symbol(label: Option<&str>, core: Symbol) -> Symbol {
    match label {
        Some(label) => Symbol::Labeled {
            label: label.to_string(),
            symbol: Box::new(core),
        },
        None => core,
    }
}

/// Creates the precedence precondition `[pr >= p]`.
fn make_precondition(pr: i64) -> Symbol {
    Symbol::Condition(Expr::Cond(Cond {
        left: Box::new(Expr::Int(pr)),
        right: Box::new(Expr::Ref("p".to_string())),
        op: CondOp::Geq,
    }))
}

/// Admits the returned result if its label is `NO_LABEL` or its bit is clear
/// in the local exclusion mask. This check reads only the label field.
fn make_local_exclusion_postcondition(local_n: i64, binding: &str) -> Symbol {
    let l_label = || Expr::Ref(format!("{binding}_label"));
    Symbol::Condition(Expr::Or(
        Box::new(Expr::Cond(Cond {
            left: Box::new(l_label()),
            right: Box::new(return_value::no_label()),
            op: CondOp::Eq,
        })),
        Box::new(Expr::Cond(Cond {
            left: Box::new(Expr::BitAnd(
                Box::new(Expr::Shr(Box::new(Expr::Int(local_n)), Box::new(l_label()))),
                Box::new(Expr::Int(1)),
            )),
            right: Box::new(Expr::Int(0)),
            op: CondOp::Eq,
        })),
    ))
}

/// Checks `l_pr == 0 || l_pr >= pr`.
fn make_postcondition(pr: i64) -> Expr {
    let l_pr = Expr::Ref("l_pr".to_string());
    Expr::Or(
        Box::new(Expr::Cond(Cond {
            left: Box::new(l_pr.clone()),
            right: Box::new(Expr::Int(0)),
            op: CondOp::Eq,
        })),
        Box::new(Expr::Cond(Cond {
            left: Box::new(l_pr),
            right: Box::new(Expr::Int(pr)),
            op: CondOp::Geq,
        })),
    )
}

fn selected_end_value(recursion: &IndirectRecursion, left: Expr, right: Expr) -> Expr {
    if recursion.is_at_both_ends() && left != right {
        choose(end_selection_condition(End::Left), left, right)
    } else if recursion.left {
        left
    } else {
        right
    }
}

/// Encodes the selected end in the generated argument: left is 0, right is 1.
fn end_value(end: End) -> i64 {
    match end {
        End::Left => 0,
        End::Right => 1,
    }
}

/// Builds a condition testing whether the `end` argument selects this end.
fn end_selection_condition(end: End) -> Expr {
    Expr::Cond(Cond {
        left: Box::new(Expr::Ref("end".to_string())),
        right: Box::new(Expr::Int(end_value(end))),
        op: CondOp::Eq,
    })
}

fn precedence_for_end(end: End) -> Expr {
    Expr::Ternary {
        cond: Box::new(end_selection_condition(end)),
        then: Box::new(Expr::Ref("p".to_string())),
        r#else: Box::new(Expr::Int(0)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        alternative, bind, call, cond, eq, exclude, follow, ge, grammar::def::Grammar,
        grammar::regex::Regex, grammar_def, id, labeled, left, lexical_rule, lit, min, ne,
        non_assoc, opt, or, priority_level, ret, right, syntax_rule, ternary, tuple,
    };

    use super::return_value;
    use crate::grammar::symbols::{Expr, Symbol};

    // Grammar comments show one alternative per line. Every alternative of a
    // rule has the same return shape. Bindings name the returned components as
    // l_pr, l_assoc, and l_label, with matching right and shared-call names.
    // Generated start rules and literal regex definitions are omitted.
    fn precedence_ref(binding: &str) -> Expr {
        Expr::Ref(format!("{binding}_pr"))
    }

    // A missing indirect recursive end returns undefined precedence; an atom returns zero.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > E "+" E
    //     > "pre" Body
    //
    //   Body
    //     = E
    //     | "b"
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                           return 0
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" r_pr=E(2) return (r_pr == 0) ? 2 : min(r_pr, 2)
    //     | "pre" r_pr=Body(1)                                            return (r_pr == UNDEFINED_PRECEDENCE) ? 0 : 1
    //
    //   Body(p: i32)
    //     = r_pr=E(p) return r_pr
    //     | "b"       return UNDEFINED_PRECEDENCE
    //
    #[test]
    fn absent_end_is_distinct_from_an_atom() {
        let input = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"))),
                priority_level!(alternative!(id!("E"), lit!("+"), id!("E"))),
                priority_level!(alternative!(lit!("pre"), id!("Body")))),
            syntax_rule!("Body" => priority_level!(alternative!(id!("E")), alternative!(lit!("b"))))
        ]);
        let grammar: Grammar = input.try_into().unwrap();
        let body = grammar.nonterminal("Body").unwrap();
        let e = grammar.nonterminal("E").unwrap();
        assert_eq!(
            grammar.alternatives(body)[1].symbols.last(),
            Some(&ret!(expr return_value::undefined_precedence()))
        );
        assert_eq!(grammar.alternatives(e)[0].symbols.last(), Some(&ret!(0)));
        // The prefix must inspect Body's result even though it has no lower
        // prefix precedence to propagate.
        assert!(matches!(
            grammar.alternatives(e)[2].symbols[1],
            Symbol::Binding { .. }
        ));
    }

    // A reference to a different precedence head remains one unrestricted call.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left L "+" L
    //     |      F
    //
    //   L
    //     = E
    //
    //   F
    //     = "b"
    //     > left F "*" F
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                            return 0
    //     | [1 >= p] l_pr=L(p, 0) [(l_pr == 0) || (l_pr >= 1)] "+" L(2, 1) return 1
    //     | F(0)                                                           return 0
    //
    //   L(p: i32, end: i32)
    //     = v_pr=E(p) return v_pr
    //
    //   F(p: i32)
    //     = "b"                                                      return 0
    //     | [1 >= p] l_pr=F(p) [(l_pr == 0) || (l_pr >= 1)] "*" F(2) return 1
    //
    #[test]
    fn foreign_head_reference_stays_one_call() {
        let input = grammar_def!("Test", syntax: [
            syntax_rule!("E" =>
                priority_level!(alternative!(lit!("a"))),
                priority_level!(left!(); alternative!(id!("L"), lit!("+"), id!("L")), alternative!(id!("F")))),
            syntax_rule!("L" => priority_level!(alternative!(id!("E")))),
            syntax_rule!("F" =>
                priority_level!(alternative!(lit!("b"))),
                priority_level!(left!(); alternative!(id!("F"), lit!("*"), id!("F"))))
        ]);
        let grammar: Grammar = input.try_into().unwrap();
        let e = grammar.nonterminal("E").unwrap();
        let symbols = &grammar.alternatives(e)[2].symbols;
        assert_eq!(symbols.len(), 2);
        assert!(matches!(&symbols[0], Symbol::Call { name, arguments }
            if name.name == "F" && arguments == &[Expr::Int(0)]));
        assert_eq!(symbols[1], ret!(0));
    }

    // A level mixing binary and postfix alternatives uses a separate associativity argument.
    //
    // Original grammar:
    //
    //   E
    //     = left E "+" E
    //     |      E "!"
    //     |      "a"
    //
    // Desugared grammar:
    //
    //   E(p: i32, a: i32)
    //     = [1 >= p] [a != 1] l_pr=E(p, 0) [(l_pr == 0) || (l_pr >= 1)] "+" r_pr=E(1, 1) return 1
    //     | [1 >= p] l_pr=E(p, 0) [(l_pr == 0) || (l_pr >= 1)] "!"                       return 0
    //     | "a"                                                                          return 0
    //
    #[test]
    fn mixed_associativity_uses_a_separate_argument() {
        let input = grammar_def!("Test", syntax: [
            syntax_rule!("E" => priority_level!(left!();
                alternative!(id!("E"), lit!("+"), id!("E")),
                alternative!(id!("E"), lit!("!")),
                alternative!(lit!("a"))))
        ]);
        let grammar: Grammar = input.try_into().unwrap();
        let e = grammar.nonterminal("E").unwrap();
        assert_eq!(
            e.parameters
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["p", "a"]
        );
        assert!(
            grammar.alternatives(e)[0]
                .symbols
                .iter()
                .any(|symbol| matches!(symbol,
            Symbol::Binding { pattern, symbol } if pattern.names()[0] == "r_pr"
                && matches!(symbol.as_ref(), Symbol::Call { arguments, .. }
                    if arguments == &[Expr::Int(1), Expr::Int(1)])))
        );
    }

    // Associativity uses the precedence numbering, including levels that need no `a`.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > none E "+" E
    //     |      E "!"
    //     > ...
    //     > none E "+" E
    //     |      E "!"
    //     > left E "*" E
    //
    // There are twelve levels mixing binary and postfix alternatives, numbered
    // 13 down to 2. Multiplication is level 1 and uses the pr+1 adjustment.
    //
    // Desugared grammar (the two lines using k repeat for k = 13 down to 2):
    //
    //   E(p: i32, a: i32)
    //     = "a"                                                                                                           return (0, 0)
    //     | [k >= p] [a != k] (l_pr, l_assoc)=E(p, 0) [l_pr == 0 || l_pr >= k] [l_assoc != k] "+" (r_pr, r_assoc)=E(k, k) return (k, k)
    //     | [k >= p] (l_pr, l_assoc)=E(p, 0) [l_pr == 0 || l_pr >= k] "!"                                                 return (0, 0)
    //     | [1 >= p] (l_pr, l_assoc)=E(p, 0) [l_pr == 0 || l_pr >= 1] "*" E(2, 0)                                         return (1, 0)
    //
    #[test]
    fn associativity_uses_precedence_levels_beyond_eleven() {
        let mut rule = syntax_rule!("E" => priority_level!(alternative!(lit!("a"))));
        rule.priority_levels.extend((0..12).map(|_| {
            priority_level!(non_assoc!();
                alternative!(id!("E"), lit!("+"), id!("E")),
                alternative!(id!("E"), lit!("!")))
        }));
        rule.priority_levels.push(priority_level!(left!();
            alternative!(id!("E"), lit!("*"), id!("E"))));

        let actual = super::transform(vec![rule]).unwrap();
        let mut alternatives = vec![alternative!(lit!("a"), ret!(expr tuple!(0, 0)))];
        for k in (2..=13).rev() {
            alternatives.push(alternative!(
                cond!(ge!(k, "p")),
                cond!(ne!("a", k)),
                bind!(("l_pr", "l_assoc"), call!("E", "p", 0)),
                cond!(or!(
                    eq!(precedence_ref("l"), 0),
                    ge!(precedence_ref("l"), k)
                )),
                cond!(ne!(Expr::Ref("l_assoc".to_string()), k)),
                lit!("+"),
                bind!(("r_pr", "r_assoc"), call!("E", k, k)),
                ret!(expr tuple!(k, k))
            ));
            alternatives.push(alternative!(
                cond!(ge!(k, "p")),
                bind!(("l_pr", "l_assoc"), call!("E", "p", 0)),
                cond!(or!(
                    eq!(precedence_ref("l"), 0),
                    ge!(precedence_ref("l"), k)
                )),
                lit!("!"),
                ret!(expr tuple!(0, 0))
            ));
        }
        alternatives.push(alternative!(
            cond!(1 >= "p"),
            bind!(("l_pr", "l_assoc"), call!("E", "p", 0)),
            cond!(or!(
                eq!(precedence_ref("l"), 0),
                ge!(precedence_ref("l"), 1)
            )),
            lit!("*"),
            call!("E", 2, 0),
            ret!(expr tuple!(1, 0))
        ));
        let mut expected = syntax_rule!("E"("p": I32, "a": I32) => priority_level!());
        expected.priority_levels[0].alternatives = alternatives;
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].head, expected.head);
        assert_eq!(actual[0].priority_levels.len(), 1);
        assert_eq!(
            actual[0].priority_levels[0].alternatives,
            expected.priority_levels[0].alternatives
        );
    }

    // A user argument on a rule without exclusions does not create a label check.
    //
    // Original grammar:
    //
    //   E
    //     = left E(7) "+" E
    //     |      "-" E
    //     |      "a"
    //
    // Desugared grammar:
    //
    //   E(p: i32, a: i32)
    //     = [1 >= p] [a != 1] l_pr=E(p, 0) [(l_pr == 0) || (l_pr >= 1)] "+" r_pr=E(1, 1) return 1
    //     | "-" E(1, 0)                                                                  return 1
    //     | "a"                                                                          return 0
    //
    #[test]
    fn binary_does_not_treat_a_user_argument_as_an_exclusion() {
        let rules = vec![syntax_rule!("E" => priority_level!(left!();
            alternative!(call!("E", 7), lit!("+"), id!("E")),
            alternative!(lit!("-"), id!("E")),
            alternative!(lit!("a"))))];
        let transformed = super::transform(rules).unwrap();
        let binary = transformed[0].alternatives().next().unwrap();
        let conditions: Vec<_> = binary
            .symbols
            .iter()
            .filter_map(|symbol| match symbol {
                Symbol::Condition(condition) => Some(condition.to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(
            conditions,
            ["1 >= p", "a != 1", "(l_pr == 0) || (l_pr >= 1)"]
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
    //   E(p: i32, e: i32)
    //     = [1 & e == 0] "a"                                                         return (0, 0) #atom
    //     | [2 & e == 0] [1 >= p] l_pr=L(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(1, 0) return (1, 1) #binary
    //
    //   L(p: i32)
    //     = (l_pr, l_label)=E(p, 0) [(l_label == NO_LABEL) || ((1 >> (l_label)) & 1 == 0)] return l_pr
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
    //   E(p: i32, e: i32)
    //     = [1 & e == 0] "a"                                                            return (0, 0) #atom
    //     | [2 & e == 0] [1 >= p] l_pr=L(p, 0) [(l_pr == 0) || (l_pr >= 1)] "+" L(1, 1) return (1, 1) #binary
    //
    //   L(p: i32, end: i32)
    //     = (v_pr, v_label)=E(p, 0) [(v_label == NO_LABEL) || ((1 >> (v_label)) & 1 == 0)] return v_pr
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
                    if arguments == &[Expr::Ref("p".to_string()), Expr::Int(0)])));
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

    // Alternatives in the same priority group receive the same precedence level.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > E "*" E
    //     > E "+" E
    //     | E "-" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "*" E(2) return 2
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(1) return 1
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "-" E(1) return 1
    //
    #[test]
    fn test_desugaring() {
        let input = grammar_def!("Test",
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
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("*"),
                        call!("E", 2),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("+"),
                        call!("E", 1),
                        ret!(1),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("-"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // Prefix and postfix alternatives restrict only their respective recursive ends.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > E "!"
    //     > "-" E
    //     > E "*" E
    //     > E "+" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [4 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 4)] "!"      return 0
    //     | "-" E(3)                                                 return 3
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "*" E(2) return 2
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(1) return 1
    //
    #[test]
    fn test_prefix_postfix_desugaring() {
        let input = grammar_def!("Test",
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
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(4 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 4),
                        )),
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
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("*"),
                        call!("E", 2),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("+"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // Left associativity excludes an addition at the right recursive end.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left E "+" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(2) return 1
    //
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

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("+"),
                        call!("E", 2),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // Right associativity excludes another semicolon at the left recursive end.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > right E ";" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] ";" E(1) return 1
    //
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

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!(";"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // Non-associativity excludes another comparison at either recursive end.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > none E "<" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "<" E(2) return 1
    //
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

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("<"),
                        call!("E", 2),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // An associativity declaration triggers desugaring even without a priority boundary.
    //
    // Original grammar:
    //
    //   E
    //     = left E "+" E
    //     |      "a"
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(2) return 1
    //     | "a"                                                      return 0
    //
    #[test]
    fn test_single_level_assoc() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(left!();
                        alternative!(id!("E"), lit!("+"), id!("E")),
                        alternative!(lit!("a"))
                    )
                ),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("+"),
                        call!("E", 2),
                        ret!(1),
                    ),
                    alternative!(lit!("a"), ret!(0)),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // Each priority group applies its own associativity restriction.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left E "*" E
    //     |      E "/" E
    //     > left E "+" E
    //     |      E "-" E
    //     > none E "<" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [3 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 3)] "*" E(4) return 3
    //     | [3 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 3)] "/" E(4) return 3
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" E(3) return 2
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "-" E(3) return 2
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "<" E(2) return 1
    //
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

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(3 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 3),
                        )),
                        lit!("*"),
                        call!("E", 4),
                        ret!(3),
                    ),
                    alternative!(
                        cond!(3 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 3),
                        )),
                        lit!("/"),
                        call!("E", 4),
                        ret!(3),
                    ),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("+"),
                        call!("E", 3),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("-"),
                        call!("E", 3),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("<"),
                        call!("E", 2),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // The return from addition retains a lower precedence from a nested prefix.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > E "+" E
    //     > "if" E "then" E "else" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                           return 0
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" r_pr=E(2) return (r_pr == 0) ? 2 : min(r_pr, 2)
    //     | "if" E(0) "then" E(0) "else" E(1)                             return 1
    //
    #[test]
    fn test_min_precedence_deep_case() {
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
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("+"),
                        bind!("r_pr", call!("E", 2)),
                        ret!(expr ternary!(eq!(precedence_ref("r"), 0), 2, min!(precedence_ref("r"), 2))),
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // A prefix above the binary level does not require a minimum precedence return.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > "-" E
    //     > E "+" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | "-" E(2)                                                 return 2
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(1) return 1
    //
    #[test]
    fn test_prefix_above_binary_returns_own_precedence() {
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

        // Prefix '-' at level 2 has higher precedence than binary '+' at level 1.
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
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("+"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // The minimum return restricts nested prefixes across several precedence levels.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left E "*" E
    //     > left E "+" E
    //     > "-" E
    //     > "if" E "then" E "else" E
    //     > right E ";" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                           return 0
    //     | [5 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 5)] "*" r_pr=E(6) return (r_pr == 0) ? 5 : min(r_pr, 5)
    //     | [4 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 4)] "+" r_pr=E(5) return (r_pr == 0) ? 4 : min(r_pr, 4)
    //     | "-" r_pr=E(3)                                                 return (r_pr == 0) ? 3 : min(r_pr, 3)
    //     | "if" E(0) "then" E(0) "else" E(2)                             return 2
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] ";" E(1)      return 1
    //
    #[test]
    fn test_min_precedence_multiple_operators() {
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
                    // '*' at level 5 uses `min` because 5 > 2.
                    alternative!(
                        cond!(5 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 5),
                        )),
                        lit!("*"),
                        bind!("r_pr", call!("E", 6)),
                        ret!(expr ternary!(eq!(precedence_ref("r"), 0), 5, min!(precedence_ref("r"), 5))),
                    ),
                    // '+' at level 4 uses `min` because 4 > 2.
                    alternative!(
                        cond!(4 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 4),
                        )),
                        lit!("+"),
                        bind!("r_pr", call!("E", 5)),
                        ret!(expr ternary!(eq!(precedence_ref("r"), 0), 4, min!(precedence_ref("r"), 4))),
                    ),
                    // '-' at level 3 uses `min` because 3 > 2.
                    alternative!(
                        lit!("-"),
                        bind!("r_pr", call!("E", 3)),
                        ret!(expr ternary!(eq!(precedence_ref("r"), 0), 3, min!(precedence_ref("r"), 3))),
                    ),
                    // 'if-then-else' returns its own level, 2.
                    alternative!(
                        lit!("if"),
                        call!("E", 0),
                        lit!("then"),
                        call!("E", 0),
                        lit!("else"),
                        call!("E", 2),
                        ret!(2),
                    ),
                    // ';' returns its own level, 1.
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!(";"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // An indirect path stops at a rule with its own precedence levels.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > F "*" "b"
    //     > E "+" E
    //
    //   F
    //     = "a"
    //     > E "+" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | F(0) "*" "b"                                             return 0
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(1) return 1
    //
    //   F(p: i32)
    //     = "a"           return 0
    //     | E(0) "+" E(0) return 0
    //
    #[test]
    fn test_precedence_heads_have_independent_levels() {
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

        let ends = super::compute_ends(&input.syntax_rules);
        let heads = super::compute_recursive_heads(&input.syntax_rules, &ends);
        assert_eq!(heads, ["E", "F"]);
        assert!(ends.reaches("F", "E", super::End::Left, &[]));
        assert!(!ends.reaches("F", "E", super::End::Left, &heads));
        assert!(
            input.syntax_rules[1]
                .alternatives()
                .all(
                    |alternative| super::classify(alternative, "F", &heads, &ends)
                        == super::RecursionKind::NonRecursive
                )
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        call!("F", 0),
                        lit!("*"),
                        lit!("b"),
                        ret!(0),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("+"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
                syntax_rule!("F"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        call!("E", 0),
                        lit!("+"),
                        call!("E", 0),
                        ret!(0),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
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
    //   E(p: i32, e: i32)
    //     = [1 & e == 0] "a"                                                                                                                       return (0, 0) #A
    //     | [2 & e == 0] [3 >= p] (l_pr, l_label)=E(p, 0) [(l_pr == 0) || (l_pr >= 3)] [(l_label == NO_LABEL) || ((8 >> (l_label)) & 1 == 0)] "++" return (0, 1) #C
    //     | [4 & e == 0] [2 >= p] (l_pr, l_label)=E(p, 0) [(l_pr == 0) || (l_pr >= 2)] "+" E(2, 0)                                                 return (2, 2) #D
    //     | [8 & e == 0] [1 >= p] (l_pr, l_label)=E(p, 0) [(l_pr == 0) || (l_pr >= 1)] "-" E(1, 0)                                                 return (1, 3) #B
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

        fn exclusion_guard(bit: i64) -> Symbol {
            cond!(Expr::Cond(Cond {
                left: Box::new(Expr::BitAnd(
                    Box::new(Expr::Int(bit)),
                    Box::new(Expr::Ref("e".to_string())),
                )),
                right: Box::new(Expr::Int(0)),
                op: CondOp::Eq,
            }))
        }

        // `E(arg0, local_n)`. Recursive call that passes a local exclusion
        // bitmask, not the caller's `e`. `local_n = 0` for non-excluded
        // recursive references.
        fn e_call(arg0: Expr, local_n: i64) -> Symbol {
            Symbol::Call {
                name: Identifier {
                    name: "E".to_string(),
                    definition: None,
                },
                arguments: vec![arg0, Expr::Int(local_n)],
            }
        }

        // Local-exclusion postcondition for the left recursive end:
        // `[l_label == NO_LABEL || ((local_n >> l_label) & 1) == 0]`.
        fn local_exclusion_postcondition(local_n: i64) -> Symbol {
            let l_label = || Expr::Ref("l_label".to_string());
            cond!(Expr::Or(
                Box::new(Expr::Cond(Cond {
                    left: Box::new(l_label()),
                    right: Box::new(return_value::no_label()),
                    op: CondOp::Eq,
                })),
                Box::new(Expr::Cond(Cond {
                    left: Box::new(Expr::BitAnd(
                        Box::new(Expr::Shr(Box::new(Expr::Int(local_n)), Box::new(l_label()))),
                        Box::new(Expr::Int(1)),
                    )),
                    right: Box::new(Expr::Int(0)),
                    op: CondOp::Eq,
                })),
            ))
        }

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32, "e": I32) => priority_level!(
                    alternative!(exclusion_guard(1), lit!("a"), ret!(expr tuple!(0, 0)); #A),
                    // #C: postfix at pr=3, left has local exclusion BIT_B=8.
                    alternative!(
                        exclusion_guard(2),
                        cond!(3 >= "p"),
                        bind!(("l_pr", "l_label"), e_call(Expr::Ref("p".to_string()), 0)),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 3),
                        )),
                        local_exclusion_postcondition(8),
                        lit!("++"),
                        ret!(expr tuple!(0, 1));
                        #C
                    ),
                    alternative!(
                        exclusion_guard(4),
                        cond!(2 >= "p"),
                        bind!(("l_pr", "l_label"), e_call(Expr::Ref("p".to_string()), 0)),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("+"),
                        e_call(Expr::Int(2), 0),
                        ret!(expr tuple!(2, 2));
                        #D
                    ),
                    alternative!(
                        exclusion_guard(8),
                        cond!(1 >= "p"),
                        bind!(("l_pr", "l_label"), e_call(Expr::Ref("p".to_string()), 0)),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("-"),
                        e_call(Expr::Int(1), 0),
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

    // Reachability follows the last symbol of each alternative through intermediate
    // nonterminals.
    //
    // Original grammar:
    //
    //   Expression
    //     = "a"
    //     > Lambda
    //
    //   Lambda
    //     = LambdaParameters "->" LambdaBody
    //
    //   LambdaBody
    //     = Expression
    //     | Block
    //
    //   LambdaParameters
    //     = Identifier
    //
    // Right-end reachability:
    //
    //   Lambda -> LambdaBody -> Expression
    //   Treating LambdaBody as a precedence boundary stops this path.
    //   LambdaParameters does not reach Expression at the right end.
    //
    // Left-end reachability:
    //
    //   Lambda -> LambdaParameters
    //
    // This test performs analysis only; it does not desugar the rules.
    //
    #[test]
    fn test_right_ends() {
        let rules = vec![
            syntax_rule!("Expression" =>
                priority_level!(alternative!(lit!("a"))),
                priority_level!(alternative!(id!("Lambda")))
            ),
            syntax_rule!("Lambda" =>
                priority_level!(alternative!(id!("LambdaParameters"), lit!("->"), id!("LambdaBody")))
            ),
            syntax_rule!("LambdaBody" =>
                priority_level!(
                    alternative!(id!("Expression")),
                    alternative!(id!("Block"))
                )
            ),
            syntax_rule!("LambdaParameters" =>
                priority_level!(alternative!(id!("Identifier")))
            ),
        ];

        let ends = super::compute_ends(&rules);
        use super::End::Right;

        // The right-end path is Lambda -> LambdaBody -> Expression.
        assert!(ends.reaches("Lambda", "Expression", Right, &[]));
        assert!(ends.reaches("LambdaBody", "Expression", Right, &[]));
        assert!(!ends.reaches("Lambda", "Expression", Right, &["LambdaBody".to_string()]));
        // LambdaParameters is a left end of Lambda, not a right end of anything here.
        assert!(!ends.reaches("LambdaParameters", "Expression", Right, &[]));
        assert!(ends.reaches("Lambda", "LambdaParameters", super::End::Left, &[]));
    }

    // The lambda alternative is indirectly recursive; the binary alternative is directly
    // recursive.
    //
    // Original grammar:
    //
    //   Expression
    //     = "a"
    //     > Expression "+" Expression
    //     > Lambda
    //
    //   Lambda
    //     = LambdaParameters "->" LambdaBody
    //
    //   LambdaBody
    //     = Expression
    //     | Block
    //
    //   LambdaParameters
    //     = Identifier
    //
    // Indirect right recursion into Expression:
    //
    //   "a": false
    //   Expression "+" Expression: false (direct recursion)
    //   Lambda: true
    //
    // This test performs analysis only; it does not desugar the rules.
    //
    #[test]
    fn test_recurses_indirectly() {
        let rules = vec![
            syntax_rule!("Expression" =>
                priority_level!(alternative!(lit!("a"))),
                priority_level!(alternative!(id!("Expression"), lit!("+"), id!("Expression"))),
                priority_level!(alternative!(id!("Lambda")))
            ),
            syntax_rule!("Lambda" =>
                priority_level!(alternative!(id!("LambdaParameters"), lit!("->"), id!("LambdaBody")))
            ),
            syntax_rule!("LambdaBody" =>
                priority_level!(
                    alternative!(id!("Expression")),
                    alternative!(id!("Block"))
                )
            ),
            syntax_rule!("LambdaParameters" =>
                priority_level!(alternative!(id!("Identifier")))
            ),
        ];

        let ends = super::compute_ends(&rules);

        let expression = rules
            .iter()
            .find(|rule| rule.head.name == "Expression")
            .unwrap();
        let alternatives: Vec<_> = expression.alternatives().collect();
        let right = |alt| super::recurses_indirectly(alt, "Expression", &ends, super::End::Right);

        assert!(!right(alternatives[0])); // 'a': no recursion
        assert!(!right(alternatives[1])); // Expression '+' Expression: direct recursion
        assert!(right(alternatives[2])); // Lambda derives a sequence ending with Expression.
    }

    // Classification distinguishes an atom, a direct binary alternative, and an indirect
    // prefix.
    //
    // Original grammar:
    //
    //   Expression
    //     = "a"
    //     > Expression "+" Expression
    //     > Lambda
    //
    //   Lambda
    //     = LambdaParameters "->" LambdaBody
    //
    //   LambdaBody
    //     = Expression
    //     | Block
    //
    //   LambdaParameters
    //     = Identifier
    //
    // Classification:
    //
    //   "a": NonRecursive
    //   Expression "+" Expression: Binary
    //   Lambda: Prefix
    //
    // This test performs analysis only; it does not desugar the rules.
    //
    #[test]
    fn test_classify() {
        use super::RecursionKind;

        let rules = vec![
            syntax_rule!("Expression" =>
                priority_level!(alternative!(lit!("a"))),
                priority_level!(alternative!(id!("Expression"), lit!("+"), id!("Expression"))),
                priority_level!(alternative!(id!("Lambda")))
            ),
            syntax_rule!("Lambda" =>
                priority_level!(alternative!(id!("LambdaParameters"), lit!("->"), id!("LambdaBody")))
            ),
            syntax_rule!("LambdaBody" =>
                priority_level!(
                    alternative!(id!("Expression")),
                    alternative!(id!("Block"))
                )
            ),
            syntax_rule!("LambdaParameters" =>
                priority_level!(alternative!(id!("Identifier")))
            ),
        ];

        let ends = super::compute_ends(&rules);

        let expression = rules
            .iter()
            .find(|rule| rule.head.name == "Expression")
            .unwrap();
        let alternatives: Vec<_> = expression.alternatives().collect();

        let kind = |alt| super::classify(alt, "Expression", &[], &ends);

        assert_eq!(kind(alternatives[0]), RecursionKind::NonRecursive);
        assert_eq!(kind(alternatives[1]), RecursionKind::Binary);
        assert_eq!(kind(alternatives[2]), RecursionKind::Prefix);
    }

    // Binary classification allows any combination of direct and indirect recursive ends.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     | E "+" R
    //     | L "-" E
    //     | L "*" R
    //
    //   L
    //     = E
    //
    //   R
    //     = E
    //
    // Classification:
    //
    //   "a": NonRecursive
    //   E "+" R: Binary
    //   L "-" E: Binary
    //   L "*" R: Binary
    //
    // This test performs analysis only; it does not desugar the rules.
    //
    #[test]
    fn test_classify_mixed_and_indirect_binary() {
        use super::RecursionKind;

        let rules = vec![
            syntax_rule!("E" => priority_level!(
                alternative!(lit!("a")),
                alternative!(id!("E"), lit!("+"), id!("R")),
                alternative!(id!("L"), lit!("-"), id!("E")),
                alternative!(id!("L"), lit!("*"), id!("R"))
            )),
            syntax_rule!("L" => priority_level!(alternative!(id!("E")))),
            syntax_rule!("R" => priority_level!(alternative!(id!("E")))),
        ];

        let ends = super::compute_ends(&rules);
        let e = &rules[0];
        let alternatives: Vec<_> = e.alternatives().collect();
        let kind = |alt| super::classify(alt, "E", &[], &ends);

        assert_eq!(kind(alternatives[0]), RecursionKind::NonRecursive);
        assert_eq!(kind(alternatives[1]), RecursionKind::Binary);
        assert_eq!(kind(alternatives[2]), RecursionKind::Binary);
        assert_eq!(kind(alternatives[3]), RecursionKind::Binary);
    }

    // A precedence head is discovered even when neither recursive end names it directly.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > L "*" R
    //
    //   L
    //     = E
    //
    //   R
    //     = E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [1 >= p] l_pr=L(p) [(l_pr == 0) || (l_pr >= 1)] "*" R(1) return 1
    //
    //   L(p: i32)
    //     = l_pr=E(p) return l_pr
    //
    //   R(p: i32)
    //     = r_pr=E(p) return r_pr
    //
    #[test]
    fn test_indirect_only_binary_desugaring() {
        use crate::grammar::symbols::Expr;

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("L"), lit!("*"), id!("R")))
                ),
                syntax_rule!("L" => priority_level!(alternative!(id!("E")))),
                syntax_rule!("R" => priority_level!(alternative!(id!("E")))),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("L", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("*"),
                        call!("R", 1),
                        ret!(1),
                    ),
                )),
                syntax_rule!("L"("p": I32) => priority_level!(alternative!(
                    bind!("l_pr", call!("E", "p")),
                    ret!(expr Expr::Ref("l_pr".to_string())),
                ))),
                syntax_rule!("R"("p": I32) => priority_level!(alternative!(
                    bind!("r_pr", call!("E", "p")),
                    ret!(expr Expr::Ref("r_pr".to_string())),
                ))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // Each intermediate nonterminal passes precedence to the recursive head.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > E "+" R
    //     > L "-" E
    //
    //   L
    //     = E
    //
    //   R
    //     = E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" R(2) return 2
    //     | [1 >= p] l_pr=L(p) [(l_pr == 0) || (l_pr >= 1)] "-" E(1) return 1
    //
    //   L(p: i32)
    //     = l_pr=E(p) return l_pr
    //
    //   R(p: i32)
    //     = r_pr=E(p) return r_pr
    //
    #[test]
    fn test_mixed_binary_desugaring() {
        use crate::grammar::symbols::Expr;

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("E"), lit!("+"), id!("R"))),
                    priority_level!(alternative!(id!("L"), lit!("-"), id!("E")))
                ),
                syntax_rule!("L" => priority_level!(alternative!(id!("E")))),
                syntax_rule!("R" => priority_level!(alternative!(id!("E")))),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("+"),
                        call!("R", 2),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("L", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("-"),
                        call!("E", 1),
                        ret!(1),
                    ),
                )),
                syntax_rule!("L"("p": I32) => priority_level!(alternative!(
                    bind!("l_pr", call!("E", "p")),
                    ret!(expr Expr::Ref("l_pr".to_string())),
                ))),
                syntax_rule!("R"("p": I32) => priority_level!(alternative!(
                    bind!("r_pr", call!("E", "p")),
                    ret!(expr Expr::Ref("r_pr".to_string())),
                ))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // The end argument selects which of two distinct recursive calls receives the restriction.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left L "*" L
    //
    //   L
    //     = E "q" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                            return 0
    //     | [1 >= p] l_pr=L(p, 0) [(l_pr == 0) || (l_pr >= 1)] "*" L(2, 1) return 1
    //
    //   L(p: i32, end: i32)
    //     = l_pr=E((end == 0) ? p : 0) "q" r_pr=E((end == 1) ? p : 0) return (end == 0) ? (l_pr) : (r_pr)
    //
    #[test]
    fn test_shared_indirect_end_propagates_both_ends() {
        use crate::grammar::symbols::Expr;

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("L"), lit!("*"), id!("L")))
                ),
                syntax_rule!("L" => priority_level!(alternative!(
                    id!("E"), lit!("q"), id!("E")
                ))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let e = actual.nonterminal("E").unwrap();
        let binary = &actual.alternatives(e)[1];
        assert_eq!(
            binary.symbols[1].call_arguments(),
            &[Expr::Ref("p".to_string()), Expr::Int(0)]
        );
        assert_eq!(
            binary.symbols[4].call_arguments(),
            &[Expr::Int(2), Expr::Int(1)]
        );

        let l = actual.nonterminal("L").unwrap();
        assert_eq!(
            l.parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            vec!["p", "end"]
        );
        let desugared = &actual.alternatives(l)[0];
        assert_eq!(
            desugared.symbols[0].call_arguments(),
            &[ternary!(
                eq!(Expr::Ref("end".to_string()), 0),
                Expr::Ref("p".to_string()),
                0
            )]
        );
        assert_eq!(
            desugared.symbols[2].call_arguments(),
            &[ternary!(
                eq!(Expr::Ref("end".to_string()), 1),
                Expr::Ref("p".to_string()),
                0
            )]
        );
        assert_eq!(
            desugared.symbols[3],
            ret!(expr Expr::Ternary {
                cond: Box::new(eq!(Expr::Ref("end".to_string()), 0)),
                then: Box::new(precedence_ref("l")),
                r#else: Box::new(precedence_ref("r")),
            })
        );
    }

    // A different precedence head keeps its own levels and receives an unrestricted call.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left E "-" F
    //     > left E "+" F
    //
    //   F
    //     = "b"
    //     > left F "*" F
    //     |      E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "-" F(0) return 0
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" F(0) return 0
    //
    //   F(p: i32)
    //     = "b"                                                      return 0
    //     | [1 >= p] l_pr=F(p) [(l_pr == 0) || (l_pr >= 1)] "*" F(2) return 1
    //     | E(0)                                                     return 0
    //
    #[test]
    fn test_indirect_binary_stops_at_another_precedence_head() {
        use crate::grammar::symbols::{Expr, Symbol};

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("E"), lit!("-"), id!("F"))),
                    priority_level!(left!(); alternative!(id!("E"), lit!("+"), id!("F")))
                ),
                syntax_rule!("F" =>
                    priority_level!(alternative!(lit!("b"))),
                    priority_level!(left!();
                        alternative!(id!("F"), lit!("*"), id!("F")),
                        alternative!(id!("E"))
                    )
                ),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let e = actual.nonterminal("E").unwrap();
        for alternative in &actual.alternatives(e)[1..] {
            let f_call = alternative
                .symbols
                .iter()
                .find_map(|symbol| match symbol {
                    Symbol::Call { name, arguments } if name.name == "F" => Some(arguments),
                    _ => None,
                })
                .expect("E alternative should retain its call to F");
            assert_eq!(f_call, &[Expr::Int(0)]);
        }
    }

    // An intermediate nonterminal forwards the head's precedence without its label.
    //
    // Original grammar:
    //
    //   S
    //     = E !B
    //
    //   E
    //     = "a" #A
    //     > left L "+" E #B
    //
    //   L
    //     = E
    //
    // Desugared grammar:
    //
    //   S
    //     = E(0, 2)
    //
    //   E(p: i32, e: i32)
    //     = [1 & e == 0] "a"                                                         return (0, 0) #A
    //     | [2 & e == 0] [1 >= p] l_pr=L(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(2, 0) return (1, 1) #B
    //
    //   L(p: i32)
    //     = (l_pr, l_label)=E(p, 0) return l_pr
    //
    #[test]
    fn test_indirect_binary_preserves_precedence_from_exclude_head() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("S" => priority_level!(alternative!(exclude!(id!("E"), "B")))),
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"); #A)),
                    priority_level!(left!(); alternative!(id!("L"), lit!("+"), id!("E"); #B))
                ),
                syntax_rule!("L" => priority_level!(alternative!(id!("E")))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let e = actual.nonterminal("E").unwrap();
        let binary = &actual.alternatives(e)[1];
        assert!(
            binary.symbols.contains(&cond!(or!(
                eq!(precedence_ref("l"), 0),
                ge!(precedence_ref("l"), 1)
            ))),
            "L returns the precedence field with its own label"
        );
        let l = actual.nonterminal("L").unwrap();
        assert_eq!(
            actual.alternatives(l)[0].symbols.last(),
            Some(&ret!(expr precedence_ref("l")))
        );
    }

    // A single-symbol alternative remains one unrestricted call even when both ends recurse.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left E "+" E
    //     > Ternary
    //
    //   Ternary
    //     = E "?" E ":" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [1 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 1)] "+" E(2) return 1
    //     | Ternary                                                  return 0
    //
    //   Ternary
    //     = E(0) "?" E(0) ":" E(0)
    //
    #[test]
    fn test_single_symbol_alternative_is_not_duplicated() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("E"), lit!("+"), id!("E"))),
                    priority_level!(alternative!(id!("Ternary")))
                ),
                syntax_rule!("Ternary" => priority_level!(alternative!(
                    id!("E"), lit!("?"), id!("E"), lit!(":"), id!("E")
                ))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let e = actual.nonterminal("E").unwrap();
        let single_symbol = &actual.alternatives(e)[2];
        assert_eq!(
            single_symbol
                .symbols
                .iter()
                .filter(|symbol| symbol
                    .as_identifier()
                    .is_some_and(|id| id.name == "Ternary"))
                .count(),
            1
        );
        assert!(actual.nonterminal("Ternary").unwrap().parameters.is_empty());
    }

    // A nullable syntax symbol before E prevents E from being the left recursive end.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left L "*" R
    //
    //   L
    //     = "x"? E
    //
    //   R
    //     = E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"        return 0
    //     | L "*" R(1) return 1
    //
    //   L
    //     = Opt_0 E(0)
    //
    //   R(p: i32)
    //     = r_pr=E(p) return r_pr
    //
    //   Opt_0
    //     = "x"
    //     | ()
    //
    #[test]
    fn test_nullable_prefix_does_not_expose_indirect_left_end() {
        use crate::grammar::symbols::{Expr, Symbol};

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("L"), lit!("*"), id!("R")))
                ),
                syntax_rule!("L" => priority_level!(alternative!(opt!(lit!("x")), id!("E")))),
                syntax_rule!("R" => priority_level!(alternative!(id!("E")))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let l = actual.nonterminal("L").unwrap();
        assert!(l.parameters.is_empty());
        assert!(actual.alternatives(l)[0].symbols.iter().any(|symbol| {
            matches!(symbol, Symbol::Call { name, arguments }
                if name.name == "E" && arguments == &[Expr::Int(0)])
        }));
    }

    // A nullable lexical symbol before E also occupies the left end.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left L "*" R
    //
    //   L
    //     = D E
    //
    //   R
    //     = E
    //
    //   @Regex
    //   D = "0"*
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"        return 0
    //     | L "*" R(1) return 1
    //
    //   L
    //     = D E(0)
    //
    //   R(p: i32)
    //     = r_pr=E(p) return r_pr
    //
    //   D = "0"*
    //
    #[test]
    fn test_nullable_lexical_prefix_does_not_expose_indirect_left_end() {
        use crate::grammar::symbols::{Expr, Symbol};

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("L"), lit!("*"), id!("R")))
                ),
                syntax_rule!("L" => priority_level!(alternative!(id!("D"), id!("E")))),
                syntax_rule!("R" => priority_level!(alternative!(id!("E")))),
            ],
            lexical: [lexical_rule!("D" => Regex::Star(Box::new(Regex::Char('0'))))]
        );

        let actual: Grammar = input.try_into().unwrap();
        let l = actual.nonterminal("L").unwrap();
        assert!(l.parameters.is_empty());
        assert!(actual.alternatives(l)[0].symbols.iter().any(|symbol| {
            matches!(symbol, Symbol::Call { name, arguments }
                if name.name == "E" && arguments == &[Expr::Int(0)])
        }));
    }

    // An intermediate alternative leading to a different precedence head leaves it
    // unrestricted.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left L "*" R
    //
    //   L
    //     = E
    //     | F
    //
    //   R
    //     = E
    //
    //   F
    //     = "b"
    //     > left F "-" F
    //
    // Desugared grammar:
    //
    //   E(p: i32, a: i32)
    //     = "a"                                                                                                                                                                                             return 0
    //     | l_pr=L(p, 0) [(l_pr == UNDEFINED_PRECEDENCE) || ((1 >= p) && ((l_pr == 0) || (l_pr >= 1)))] "*" r_pr=R(1, (l_pr == UNDEFINED_PRECEDENCE) ? 0 : 1) [(l_pr == UNDEFINED_PRECEDENCE) || (a != 1)]  return 1
    //
    //   L(p: i32, a: i32)
    //     = l_pr=E(p, a)  return l_pr
    //     | F(0)          return UNDEFINED_PRECEDENCE
    //
    //   R(p: i32, a: i32)
    //     = r_pr=E(p, a) return r_pr
    //
    //   F(p: i32)
    //     = "b"                                                       return 0
    //     | [1 >= p] l_pr=F(p) [(l_pr == 0) || (l_pr >= 1)] "-" F(2)  return 1
    //
    #[test]
    fn test_indirect_end_restricts_only_its_own_head() {
        use crate::grammar::symbols::{Expr, Symbol};

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("L"), lit!("*"), id!("R")))
                ),
                syntax_rule!("L" =>
                    priority_level!(alternative!(id!("E")), alternative!(id!("F")))
                ),
                syntax_rule!("R" => priority_level!(alternative!(id!("E")))),
                syntax_rule!("F" =>
                    priority_level!(alternative!(lit!("b"))),
                    priority_level!(left!(); alternative!(id!("F"), lit!("-"), id!("F")))
                ),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let l = actual.nonterminal("L").unwrap();
        assert_eq!(l.parameters.len(), 2);
        let alternatives = actual.alternatives(l);
        assert!(matches!(
            &alternatives[0].symbols[0],
            Symbol::Binding { symbol, .. }
                if matches!(symbol.as_ref(), Symbol::Call { name, arguments }
                    if name.name == "E" && arguments == &[Expr::Ref("p".to_string()), Expr::Ref("a".to_string())])
        ));
        assert!(alternatives[1].symbols.iter().any(|symbol| matches!(
            symbol,
            Symbol::Call { name, arguments }
                if name.name == "F" && arguments == &[Expr::Int(0)]
        )));
    }

    // A leading optional symbol makes the head alternative prefix, even when it derives empty.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left "-"? E "+" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                 return 0
    //     | Opt_0 E(0) "+" E(1) return 1
    //
    //   Opt_0
    //     = "-"
    //     | ()
    //
    #[test]
    fn test_nullable_symbol_does_not_expose_a_recursive_end() {
        use crate::grammar::symbols::{Expr, Symbol};

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(opt!(lit!("-")), id!("E"), lit!("+"), id!("E")))
                ),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let e = actual.nonterminal("E").unwrap();
        let prefix = &actual.alternatives(e)[1];
        assert!(matches!(prefix.symbols[0], Symbol::Identifier(_)));
        let calls: Vec<_> = prefix
            .symbols
            .iter()
            .filter_map(|symbol| match symbol {
                Symbol::Call { name, arguments } => Some((name.name.as_str(), arguments.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(
            calls,
            vec![("E", vec![Expr::Int(0)]), ("E", vec![Expr::Int(1)])]
        );
    }

    // An intermediate nonterminal cannot serve two different precedence heads.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left E "+" M
    //
    //   F
    //     = "b"
    //     > left F "-" M
    //
    //   M
    //     = E
    //     | F
    //
    // No desugared grammar is produced:
    //
    //   indirect precedence cannot be enforced for `M` because it serves multiple precedence heads; give each precedence rule its own intermediate nonterminal
    //
    #[test]
    fn test_multi_head_indirect_end_is_rejected() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("E"), lit!("+"), id!("M")))
                ),
                syntax_rule!("F" =>
                    priority_level!(alternative!(lit!("b"))),
                    priority_level!(left!(); alternative!(id!("F"), lit!("-"), id!("M")))
                ),
                syntax_rule!("M" => priority_level!(alternative!(id!("E")), alternative!(id!("F")))),
            ]
        );

        let errors = Grammar::try_from(input).unwrap_err();
        assert_eq!(
            errors,
            vec![
                "indirect precedence cannot be enforced for `M` because it serves multiple precedence heads; give each precedence rule its own intermediate nonterminal"
            ]
        );
    }

    // An intermediate nonterminal can accept end, associativity, and exclusion arguments
    // together.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left L "*" L
    //
    //   L
    //     = E #plain
    //     | "(" L ")" #paren
    //
    //   C
    //     = L !paren
    //   D
    //     = x=selected:L !paren
    //
    // Desugared grammar:
    //
    //   E(p: i32, a: i32)
    //     = "a"                                                                                                                                                                                                                                                                   return 0
    //     | (l_pr, l_label)=L(p, 0, 0, 0) [(l_pr == UNDEFINED_PRECEDENCE) || ((1 >= p) && ((l_pr == 0) || (l_pr >= 1)))] "*" (r_pr, r_label)=L(1, 1, (l_pr == UNDEFINED_PRECEDENCE) ? 0 : 1, 0) [((l_pr == UNDEFINED_PRECEDENCE) || (r_pr == UNDEFINED_PRECEDENCE)) || (a != 1)]  return (r_pr == UNDEFINED_PRECEDENCE) ? 0 : 1
    //
    //   L(p: i32, end: i32, a: i32, e: i32)
    //     = [1 & e == 0] v_pr=E(p, a)           return (v_pr, 0) #plain
    //     | [2 & e == 0] "(" L(0, 0, 0, 0) ")"  return (UNDEFINED_PRECEDENCE, 1) #paren
    //
    //   C
    //     = L(0, 0, 0, 2)
    //   D
    //     = x=selected:L(0, 0, 0, 2)
    //
    #[test]
    fn test_dual_indirect_end_can_also_be_exclude_target() {
        use crate::grammar::symbols::{Expr, Symbol};

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!(); alternative!(id!("L"), lit!("*"), id!("L")))
                ),
                syntax_rule!("L" => priority_level!(
                    alternative!(id!("E"); #plain),
                    alternative!(lit!("("), id!("L"), lit!(")"); #paren)
                )),
                syntax_rule!("C" => priority_level!(alternative!(exclude!(id!("L"), "paren")))),
                syntax_rule!("D" => priority_level!(alternative!(
                    bind!("x", labeled!("selected", exclude!(id!("L"), "paren")))
                ))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let d = actual.nonterminal("D").unwrap();
        assert_eq!(
            actual.alternatives(d)[0].symbols[0].to_string(),
            "x=selected:L(0, 0, 0, 2)"
        );
        let l = actual.nonterminal("L").unwrap();
        assert_eq!(
            l.parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            vec!["p", "end", "a", "e"]
        );
        let e = actual.nonterminal("E").unwrap();
        let binary = &actual.alternatives(e)[1];
        let left_arguments = match binary
            .symbols
            .iter()
            .find(|s| matches!(s, Symbol::Binding { pattern, .. } if pattern.names()[0] == "l_pr"))
            .unwrap()
        {
            Symbol::Binding { symbol, .. } => match symbol.as_ref() {
                Symbol::Call { arguments, .. } => arguments,
                other => panic!("expected call, got {other:?}"),
            },
            other => panic!("expected binding, got {other:?}"),
        };
        assert_eq!(
            left_arguments,
            &[
                Expr::Ref("p".to_string()),
                Expr::Int(0),
                Expr::Int(0),
                Expr::Int(0)
            ]
        );
    }

    // A field label on an indirect recursive reference remains inside the generated binding.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > L "!"
    //
    //   L
    //     = lhs:E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                 return 0
    //     | [1 >= p] l_pr=L(p) [(l_pr == 0) || (l_pr >= 1)] "!" return 0
    //
    //   L(p: i32)
    //     = l_pr=lhs:E(p) return l_pr
    //
    #[test]
    fn test_indirect_end_preserves_label() {
        use crate::grammar::symbols::Symbol;

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("L"), lit!("!")))
                ),
                syntax_rule!("L" => priority_level!(alternative!(labeled!("lhs", id!("E"))))),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let l = actual.nonterminal("L").unwrap();
        assert!(matches!(
            &actual.alternatives(l)[0].symbols[0],
            Symbol::Binding { symbol, .. }
                if matches!(symbol.as_ref(), Symbol::Labeled { label, .. } if label == "lhs")
        ));
    }

    // Right and non-associativity restrict the selected ends of a shared intermediate
    // nonterminal.
    //
    // First case:
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > right L "@" L
    //
    //   L
    //     = E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                            return 0
    //     | [1 >= p] l_pr=L(p, 0) [(l_pr == 0) || (l_pr >= 2)] "@" L(1, 1) return 1
    //
    //   L(p: i32, end: i32)
    //     = v_pr=E(p) return v_pr
    //
    // Second case:
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > none L "@" L
    //
    //   L
    //     = E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                            return 0
    //     | [1 >= p] l_pr=L(p, 0) [(l_pr == 0) || (l_pr >= 2)] "@" L(2, 1) return 1
    //
    //   L(p: i32, end: i32)
    //     = v_pr=E(p) return v_pr
    //
    #[test]
    fn test_indirect_right_and_non_associativity_arguments() {
        use crate::grammar::symbols::Expr;

        for (associativity, right_arg, postcondition) in [
            (
                right!(),
                1,
                cond!(or!(
                    eq!(precedence_ref("l"), 0),
                    ge!(precedence_ref("l"), 2),
                )),
            ),
            (
                non_assoc!(),
                2,
                cond!(or!(
                    eq!(precedence_ref("l"), 0),
                    ge!(precedence_ref("l"), 2),
                )),
            ),
        ] {
            let input = grammar_def!("Test",
                syntax: [
                    syntax_rule!("E" =>
                        priority_level!(alternative!(lit!("a"))),
                        priority_level!(associativity; alternative!(id!("L"), lit!("@"), id!("L")))
                    ),
                    syntax_rule!("L" => priority_level!(alternative!(id!("E")))),
                ]
            );
            let actual: Grammar = input.try_into().unwrap();
            let e = actual.nonterminal("E").unwrap();
            let binary = &actual.alternatives(e)[1];
            assert_eq!(
                binary.symbols[4].call_arguments(),
                &[Expr::Int(right_arg), Expr::Int(1)]
            );
            assert!(binary.symbols.contains(&postcondition));
        }
    }

    // An indirect prefix passes the restriction to the head and returns its precedence.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > E "+" E
    //     > Lambda
    //
    //   Lambda
    //     = "fn" Body
    //
    //   Body
    //     = E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                           return 0
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" r_pr=E(2) return (r_pr == 0) ? 2 : min(r_pr, 2)
    //     | Lambda(1)                                                     return 1
    //
    //   Lambda(p: i32)
    //     = "fn" r_pr=Body(p) return r_pr
    //
    //   Body(p: i32)
    //     = r_pr=E(p) return r_pr
    //
    #[test]
    fn test_indirect_prefix_desugaring() {
        use crate::grammar::symbols::Expr;

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("E"), lit!("+"), id!("E"))),
                    priority_level!(alternative!(id!("Lambda")))
                ),
                syntax_rule!("Lambda" =>
                    priority_level!(alternative!(lit!("fn"), id!("Body")))
                ),
                syntax_rule!("Body" =>
                    priority_level!(alternative!(id!("E")))
                ),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("+"),
                        bind!("r_pr", call!("E", 2)),
                        ret!(expr ternary!(eq!(precedence_ref("r"), 0), 2, min!(precedence_ref("r"), 2))),
                    ),
                    alternative!(call!("Lambda", 1), ret!(1)),
                )),
                syntax_rule!("Lambda"("p": I32) => priority_level!(
                    alternative!(
                        lit!("fn"),
                        bind!("r_pr", call!("Body", "p")),
                        ret!(expr Expr::Ref("r_pr".to_string())),
                    ),
                )),
                syntax_rule!("Body"("p": I32) => priority_level!(
                    alternative!(
                        bind!("r_pr", call!("E", "p")),
                        ret!(expr Expr::Ref("r_pr".to_string())),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // An indirect postfix passes the left-end restriction through its intermediate
    // nonterminals.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > E "+" E
    //     > Postfix
    //
    //   Postfix
    //     = Body "!"
    //
    //   Body
    //     = E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                      return 0
    //     | [2 >= p] l_pr=E(p) [(l_pr == 0) || (l_pr >= 2)] "+" E(2) return 2
    //     | [1 >= p] l_pr=Postfix(p) [(l_pr == 0) || (l_pr >= 1)]    return 0
    //
    //   Postfix(p: i32)
    //     = l_pr=Body(p) "!" return l_pr
    //
    //   Body(p: i32)
    //     = l_pr=E(p) return l_pr
    //
    #[test]
    fn test_indirect_postfix_desugaring() {
        use crate::grammar::symbols::Expr;

        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(alternative!(id!("E"), lit!("+"), id!("E"))),
                    priority_level!(alternative!(id!("Postfix")))
                ),
                syntax_rule!("Postfix" =>
                    priority_level!(alternative!(id!("Body"), lit!("!")))
                ),
                syntax_rule!("Body" =>
                    priority_level!(alternative!(id!("E")))
                ),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l_pr", call!("E", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 2),
                        )),
                        lit!("+"),
                        call!("E", 2),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", call!("Postfix", "p")),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        ret!(0),
                    ),
                )),
                syntax_rule!("Postfix"("p": I32) => priority_level!(
                    alternative!(
                        bind!("l_pr", call!("Body", "p")),
                        lit!("!"),
                        ret!(expr Expr::Ref("l_pr".to_string())),
                    ),
                )),
                syntax_rule!("Body"("p": I32) => priority_level!(
                    alternative!(
                        bind!("l_pr", call!("E", "p")),
                        ret!(expr Expr::Ref("l_pr".to_string())),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // Field labels at both recursive ends survive the rewrite.
    //
    // Original grammar:
    //
    //   E
    //     = "a"
    //     > left lhs:E "+" rhs:E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = "a"                                                              return 0
    //     | [1 >= p] l_pr=lhs:E(p) [(l_pr == 0) || (l_pr >= 1)] "+" rhs:E(2) return 1
    //
    #[test]
    fn test_labeled_recursive_ends() {
        let input = grammar_def!("Test",
            syntax: [
                syntax_rule!("E" =>
                    priority_level!(alternative!(lit!("a"))),
                    priority_level!(left!();
                        alternative!(labeled!("lhs", id!("E")), lit!("+"), labeled!("rhs", id!("E")))
                    )
                ),
            ]
        );

        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l_pr", labeled!("lhs", call!("E", "p"))),
                        cond!(or!(
                            eq!(precedence_ref("l"), 0),
                            ge!(precedence_ref("l"), 1),
                        )),
                        lit!("+"),
                        labeled!("rhs", call!("E", 2)),
                        ret!(1),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    // A follow restriction wraps the generated binding at the left recursive end.
    //
    // Original grammar:
    //
    //   E
    //     = Num
    //     > (E !>> ";") "+" E
    //
    // Desugared grammar:
    //
    //   E(p: i32)
    //     = Num                                                                return 0
    //     | [1 >= p] (l_pr=E(p) !>> ";") [(l_pr == 0) || (l_pr >= 1)] "+" E(1) return 1
    //
    #[test]
    fn test_restriction_preserved_at_left_end() {
        use crate::grammar::symbols::Symbol;

        let rules = vec![syntax_rule!("E" =>
            priority_level!(alternative!(id!("Num"))),
            priority_level!(alternative!(follow!(id!("E"), ";"), lit!("+"), id!("E")))
        )];

        let transformed = super::transform(rules).unwrap();
        let e = transformed.iter().find(|r| r.head.name == "E").unwrap();
        let alternatives: Vec<_> = e.alternatives().collect();
        // alt 0 is `Num return 0`; alt 1 is the recursive `+`. Its symbols are
        // [precondition, restricted left binding, ...].
        let left_end = &alternatives[1].symbols[1];
        match left_end {
            Symbol::Restricted {
                symbol,
                restrictions,
            } => {
                assert_eq!(restrictions.follow.len(), 1);
                assert!(
                    matches!(symbol.as_ref(), Symbol::Binding { pattern, .. } if pattern.names()[0] == "l_pr"),
                    "restriction should wrap the `l` binding, got {symbol:?}",
                );
            }
            other => panic!("expected the left end to keep its `!>>`, got {other:?}"),
        }
    }
}
