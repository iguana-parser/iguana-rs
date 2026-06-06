// Precedence desugaring.
//
// Consider this grammar, where `>` separates priority groups (top to
// bottom: tightest binding to loosest):
//
//   E = "a"           #Lit
//     > E "*" E       #Mul
//     > E "+" E       #Add
//
// Without further machinery, the input `a + a * a` would have two parses:
// `(a + a) * a` and `a + (a * a)`. The intent of `>` is that the operator
// named earlier binds tighter, so only the second parse should be
// admitted. This pass rewrites the grammar so the parser enforces that.
//
// After this pass:
//
//   E(p: i32)
//     = "a" return 0                                            #Lit
//     | [2 >= p] l=E(2) [l == 0 || l >= 2] "*" E(2) return 2    #Mul
//     | [1 >= p] l=E(1) [l == 0 || l >= 1] "+" E(1) return 1    #Add
//
// Each alternative now carries its priority as guards on the call into
// `E`. A call `E(p)` admits an alternative whose precedence is at least
// `p`; the recursive sub-expressions inside that alternative are called
// with the alternative's own precedence. The literal alternative returns
// 0, a sentinel that the post-condition `[l == 0 || l >= pr]` always
// admits.
//
// What the desugaring does:
//
//   - `p: i32` parameter. Appended to every nonterminal with priority
//     levels. Outer call sites pass 0 (no restriction, admits every
//     alternative). Recursive sub-expressions inside an alternative pass
//     the alternative's own precedence.
//
//   - Precedence assignment. Priority groups are numbered bottom-up,
//     starting at 1. Groups whose alternatives are all non-recursive get
//     no number: `"a"` has none, `E "+" E` has pr=1, `E "*" E` has
//     pr=2.
//
//   - Pre-condition. Each recursive alternative starts with `[pr >= p]`:
//     the caller's threshold must be at most the alternative's own
//     precedence.
//
//   - Left-side post-condition. The left recursive call binds to a
//     variable `l`, followed by `[l == 0 || l >= pr]`. This ensures the
//     left subtree was parsed at high-enough precedence, or was a
//     non-recursive leaf returning 0.
//
//   - Return value. Each alternative ends with `return pr`, or
//     `return 0` for non-recursive alternatives.
//
//   - Associativity adjustments. Left associativity raises the right
//     recursive call's argument by one (`E(pr+1)`). Right associativity
//     raises the post-condition threshold by one
//     (`[l == 0 || l >= pr+1]`). Non-associativity does both.
//
//   - Min trick. When a prefix alternative exists at lower precedence
//     than a binary or prefix alternative, the right recursive call
//     binds to `r` and the return becomes `r == 0 ? pr : min(r, pr)`.
//     This propagates the inner precedence up through right-recursive
//     chains.
//
//   - Use-site rewriting. Every bare reference to a desugared
//     nonterminal becomes a call with 0 as the precedence argument.
//
// Indirect operator precedence. An operator's operand is not always the
// priority nonterminal itself; it can sit behind one or more other
// nonterminals. Consider a lambda added to `E`:
//
//   E = "a"
//     > E "+" E
//     > Lambda
//   Lambda = "fn" Body
//   Body = E
//
// The `Lambda` alternative is a prefix operator whose operand is `E`, but `E`
// is reached only through `Lambda` and `Body`. Without further work the body
// is parsed with no precedence restriction, so `fn a + a` has both the parse
// `fn (a + a)` and `(fn a) + a`, reintroducing the ambiguity that precedence
// is meant to remove.
//
// The desugaring threads the precedence parameter along that path. `Lambda`
// and `Body` each gain a `p` parameter and pass it to the nonterminal at
// their operand end: the `Lambda` alternative calls `Lambda(1)`, `Lambda`
// calls `Body(p)`, and `Body` calls `E(p)`, so the restriction reaches `E`
// two nonterminals away. An indirect postfix `X op` is the mirror image,
// threading from the left end. A nonterminal that has its own priority levels
// is left alone, since it manages its own precedence.
//
// Composition with exclude desugaring. When exclude desugaring has
// already applied to a nonterminal, it carries an `e: i32` parameter and
// references like `E(mask)` already pass an exclude mask. This pass
// prepends the precedence argument: bare `E` becomes `E(0, 0)`;
// `E(mask)` becomes `E(0, mask)`. The precedence return value and the
// label index pack into a single i32, precedence in the high half and
// label in the low half, so guards like `[l >= pr]` become
// `[l >> 16 >= pr]`.
//
// Recursive ends are handled asymmetrically. The right recursive end of
// a binary or prefix alternative passes the local exclusion bitmask as
// `e` (so `op E !Prefix` rewrites the right call as `E(pr, BIT_Prefix)`).
// The left recursive end of a binary or postfix alternative passes
// `e = 0`, regardless of any local exclusion at this position, to keep
// the GSS shared across exclusion contexts; the local exclusion is
// enforced instead as a postcondition on the returned label. See
// `docs/operator_precedence_desugaring.md` sections 8 and 9 for the
// full algorithm and rationale.
//
// Pipeline position. The precedence desugaring transformation runs after
// EBNF expansion. EBNF wrappers are not recursed through, so a head
// reference nested inside one would not get rewritten to a call.
//
// Reference: Operator Precedence for Data-Dependent Grammars, Afroozeh
// and Izmaylova, PEPM'16, sections 3.2–3.5.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::grammar::{
    def::{Alternative, Associativity, PriorityLevel, SyntaxRule},
    symbols::{Cond, CondOp, Expr, Identifier, Nonterminal, ParamType, Parameter, Symbol},
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

/// How an alternative recurses into the head: directly, or indirectly through
/// another nonterminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecursionKind {
    /// E op E
    Binary,
    /// op E
    Prefix,
    /// E op
    Postfix,
    /// op X, where X derives E at its right end (X =>* αE)
    IndirectPrefix,
    /// X op, where X derives E at its left end (X =>* Eα)
    IndirectPostfix,
    /// Neither end recurses: 'a', '(' E ')'
    NonRecursive,
}

/// A left or right end of an alternative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

pub fn transform(syntax_rules: Vec<SyntaxRule>) -> Vec<SyntaxRule> {
    // Rules with `>` separators (multiple priority levels). Candidates for
    // desugaring; each one we'll try to find a recursive name for.
    let rules_with_priority_levels: Vec<String> = syntax_rules
        .iter()
        .filter(|rule| needs_desugaring(rule))
        .map(|rule| rule.head.name.clone())
        .collect();

    let rules_by_name: FxHashMap<&str, &SyntaxRule> = syntax_rules
        .iter()
        .map(|r| (r.head.name.as_str(), r))
        .collect();

    let recursive_names = compute_recursive_names(&syntax_rules, &rules_with_priority_levels);

    let ends = compute_ends(&syntax_rules);
    let precedences_by_rule = compute_precedences(&recursive_names, &rules_by_name, &ends);

    // Nonterminals that already carry `e` after `exclude_desugaring`. Recursive-end calls
    // emitted into the desugared body must pass `e` along when the target has it;
    // calls to non-exclude targets stay one-arg.
    let names_with_e: FxHashSet<String> = syntax_rules
        .iter()
        .filter(|r| r.head.parameters.iter().any(|p| p.name == "e"))
        .map(|r| r.head.name.clone())
        .collect();

    let intermediates = compute_intermediates(
        &syntax_rules,
        &recursive_names,
        &ends,
        &rules_with_priority_levels,
    );

    // Nonterminals that gain a `p` parameter; their call sites pass `0` as the
    // precedence argument.
    let desugared_names: Vec<String> = recursive_names
        .iter()
        .map(|(name, _)| name.clone())
        .chain(intermediates.keys().cloned())
        .collect();

    // `rules_by_name` borrow ends after this line (last use was building
    // `precedences_by_rule`), so the move below is fine under NLL.
    syntax_rules
        .into_iter()
        .map(|rule| {
            let rule = if let Some((_, rec_name)) = recursive_names
                .iter()
                .find(|(name, _)| *name == rule.head.name)
            {
                let precedences = precedences_by_rule[&rule.head.name].clone();
                desugar_rule(
                    rule,
                    rec_name,
                    &recursive_names,
                    precedences,
                    &names_with_e,
                    &ends,
                )
            } else if let Some((head, side)) = intermediates.get(&rule.head.name).cloned() {
                thread_intermediate(rule, &head, side, &names_with_e, &ends)
            } else {
                rule
            };
            update_external_references(rule, &desugared_names, &names_with_e)
        })
        .collect()
}

/// For each rule with priority levels, find the nonterminal that appears at
/// recursive (left/right) ends of its alternatives. See `find_recursive_name`.
fn compute_recursive_names(
    syntax_rules: &[SyntaxRule],
    rules_with_priority_levels: &[String],
) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for rule in syntax_rules {
        if !needs_desugaring(rule) {
            continue;
        }
        if let Some(rec_name) = find_recursive_name(rule, rules_with_priority_levels) {
            result.push((rule.head.name.clone(), rec_name));
        }
    }
    result
}

/// Materialize the precedence vector for each rule that gets desugared. Same
/// shape as `assign_precedence`'s output (`Some(i)` for a priority level with
/// recursive alternatives, `None` for a non-recursive level), indexed by level
/// position.
fn compute_precedences(
    recursive_names: &[(String, String)],
    rules_by_name: &FxHashMap<&str, &SyntaxRule>,
    ends: &Ends,
) -> FxHashMap<String, Vec<Option<i64>>> {
    let mut result = FxHashMap::default();
    for (rule_name, rec_name) in recursive_names {
        let rule = rules_by_name[rule_name.as_str()];
        result.insert(
            rule_name.clone(),
            assign_precedence(
                &rule.priority_levels,
                rule_name,
                rec_name,
                recursive_names,
                ends,
            ),
        );
    }
    result
}

/// A map from each intermediate nonterminal to a `(head, side)` pair. `head` is
/// the precedence nonterminal whose `p` value this nonterminal must pass along.
/// `side` says where that head sits: `Right` for a prefix operator (head at the
/// right end), `Left` for a postfix operator (head at the left end).
///
/// In an indirect prefix `op X`, the nonterminal `X` derives the head at its
/// right end, through one or more nonterminals; in an indirect postfix `X op`, `X`
/// derives the head at its left end. Each nonterminal on that path carries the
/// precedence parameter until it reaches the head, the operand the precedence
/// restricts, so each one is recorded here and later threaded. Consider:
///
///   E = 'a'
///     > Lambda
///   Lambda
///     = 'fn' Body
///   Body
///     = E
///
/// `Lambda` and `Body` lie between the `> Lambda` alternative and `E`, so both
/// pass `E`'s precedence along from its right end: `Lambda` and `Body` map to
/// `(E, Right)`.
///
/// To collect the intermediates, follow the rightmost symbols (for a prefix) or
/// leftmost symbols (for a postfix) from `X` toward the head, one step at a time.
/// Every nonterminal reached before the head is an intermediate. The walk stops at
/// the head, and at any other precedence nonterminal along the way: one that has
/// its own precedence does not pass this head's `p` along.
fn compute_intermediates(
    syntax_rules: &[SyntaxRule],
    recursive_names: &[(String, String)],
    ends: &Ends,
    rules_with_priority_levels: &[String],
) -> FxHashMap<String, (String, Side)> {
    let priority: FxHashSet<&str> = rules_with_priority_levels
        .iter()
        .map(String::as_str)
        .collect();

    let mut intermediates = FxHashMap::default();
    for rule in syntax_rules {
        let Some((_, recursive_name)) = recursive_names
            .iter()
            .find(|(name, _)| *name == rule.head.name)
        else {
            continue;
        };
        let head = rule.head.name.as_str();
        for alt in rule.alternatives() {
            let side = match classify(alt, head, recursive_name, recursive_names, ends) {
                RecursionKind::IndirectPrefix => Side::Right,
                RecursionKind::IndirectPostfix => Side::Left,
                _ => continue,
            };
            let Some(end) = end_symbol(&alt.symbols, side).and_then(Symbol::as_identifier) else {
                continue;
            };

            // Walk the `side` ends from the indirect end toward `head`, stopping
            // there. Every nonterminal in between is an intermediate.
            let mut stack = vec![end.name.as_str()];
            let mut visited = FxHashSet::default();
            while let Some(node) = stack.pop() {
                if !visited.insert(node) {
                    continue;
                }
                if node == head || priority.contains(node) || !ends.reaches(node, head, side) {
                    continue;
                }
                let entry = (head.to_string(), side);
                if let Some(previous) = intermediates.insert(node.to_string(), entry) {
                    debug_assert_eq!(previous, (head.to_string(), side), "{node} threaded twice");
                }
                if let Some(steps) = ends.direct(side).get(node) {
                    stack.extend(steps.iter().map(String::as_str));
                }
            }
        }
    }
    intermediates
}

/// Finds the nonterminal name that appears at recursive (left/right) ends of a
/// rule's alternatives. Direct recursion (head name appears at ends) takes
/// precedence, then indirect recursion (some other rule with priority levels
/// appears at ends). Returns `None` if no recursive ends are found.
fn find_recursive_name(rule: &SyntaxRule, rules_with_priority_levels: &[String]) -> Option<String> {
    if has_recursive_ends(rule, &rule.head.name) {
        return Some(rule.head.name.clone());
    }
    for name in rules_with_priority_levels {
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
    rule.alternatives()
        .any(|alt| classify_ends(alt, name, &[]) != RecursiveEnds::None)
}

/// A rule needs desugaring if it has more than one priority level.
fn needs_desugaring(rule: &SyntaxRule) -> bool {
    rule.priority_levels.len() > 1
}

/// Classifies an alternative's recursion type relative to the recursive nonterminal.
/// `recursive_names` maps each desugared nonterminal to its recursive name, enabling
/// recognition of indirect recursion (e.g., a sibling cascade as a reference to E).
/// Skips leading `Condition` and trailing `Return` symbols so decorations injected
/// by `exclude_desugaring` don't shadow the actual recursive ends.
fn classify_ends(
    alternative: &Alternative,
    recursive_name: &str,
    recursive_names: &[(String, String)],
) -> RecursiveEnds {
    let first = first_grammar_symbol(&alternative.symbols);
    let last = last_grammar_symbol(&alternative.symbols);
    let is_left = is_reference_to(first, recursive_name, recursive_names);
    let is_right = is_reference_to(last, recursive_name, recursive_names);
    match (is_left, is_right) {
        (true, true) => RecursiveEnds::Binary,
        (true, false) => RecursiveEnds::Left,
        (false, true) => RecursiveEnds::Right,
        (false, false) => RecursiveEnds::None,
    }
}

/// True if the alternative's `side` end recurses into `head` indirectly: the end
/// symbol is a nonterminal other than `head` that reaches `head` on that side,
/// like `Lambda` reaching `Expression` at the right. An end that is `head` itself
/// is direct recursion, which `classify_ends` reports.
fn recurses_indirectly(alternative: &Alternative, head: &str, ends: &Ends, side: Side) -> bool {
    non_head_nonterminal(end_symbol(&alternative.symbols, side), head)
        .is_some_and(|name| ends.reaches(name, head, side))
}

/// The nonterminal at this symbol position, unless it is `head` itself.
fn non_head_nonterminal<'a>(symbol: Option<&'a Symbol>, head: &str) -> Option<&'a str> {
    symbol
        .and_then(Symbol::as_identifier)
        .map(|id| id.name.as_str())
        .filter(|&name| name != head)
}

/// Combines an alternative's direct and indirect recursion. The direct
/// classification dominates; a direct-non-recursive alternative whose right end
/// recurses indirectly is `IndirectPrefix`, and whose left end does so is
/// `IndirectPostfix`. Recursing indirectly at both ends (indirect binary) is not
/// handled and leaves the alternative non-recursive.
fn combine_recursion(
    direct: RecursiveEnds,
    indirect_left: bool,
    indirect_right: bool,
) -> RecursionKind {
    use RecursionKind as C;
    match direct {
        RecursiveEnds::Binary => C::Binary,
        RecursiveEnds::Right => C::Prefix,
        RecursiveEnds::Left => C::Postfix,
        RecursiveEnds::None if indirect_right && !indirect_left => C::IndirectPrefix,
        RecursiveEnds::None if indirect_left && !indirect_right => C::IndirectPostfix,
        RecursiveEnds::None => C::NonRecursive,
    }
}

/// The recursion kind of an alternative, combining direct recursion (keyed on
/// `recursive_name`) with indirect recursion at either end (keyed on `head`).
fn classify(
    alternative: &Alternative,
    head: &str,
    recursive_name: &str,
    recursive_names: &[(String, String)],
    ends: &Ends,
) -> RecursionKind {
    let direct = classify_ends(alternative, recursive_name, recursive_names);
    let indirect_left = recurses_indirectly(alternative, head, ends, Side::Left);
    let indirect_right = recurses_indirectly(alternative, head, ends, Side::Right);
    combine_recursion(direct, indirect_left, indirect_right)
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

/// The alternative's grammar symbol at `side`: the first for `Left`, the last for
/// `Right`.
fn end_symbol(symbols: &[Symbol], side: Side) -> Option<&Symbol> {
    match side {
        Side::Left => first_grammar_symbol(symbols),
        Side::Right => last_grammar_symbol(symbols),
    }
}

/// The index of the alternative's grammar symbol at `side`.
fn end_index(symbols: &[Symbol], side: Side) -> Option<usize> {
    match side {
        Side::Left => symbols.iter().position(|s| !is_decoration(s)),
        Side::Right => symbols.iter().rposition(|s| !is_decoration(s)),
    }
}

/// Checks if a symbol is an identifier reference to the given recursive name,
/// either directly or indirectly (via a nonterminal that shares the same recursive name).
/// Accepts both `Symbol::Identifier` and `Symbol::Call` (the latter is what
/// `exclude_desugaring` leaves behind for exclude-targeted nonterminals).
fn is_reference_to(
    symbol: Option<&Symbol>,
    recursive_name: &str,
    recursive_names: &[(String, String)],
) -> bool {
    let name = match symbol {
        Some(Symbol::Identifier(id)) => &id.name,
        Some(Symbol::Call { name, .. }) => &name.name,
        _ => return false,
    };
    name == recursive_name
        || recursive_names
            .iter()
            .any(|(n, rec)| n == name && rec == recursive_name)
}

/// Left and right ends of the grammar's nonterminals: the first/last symbol of
/// each alternative (`direct_left`/`direct_right`), and their transitive closures
/// (`left`/`right`).
///
/// For A = B C, B = D E, C = F G: `direct_right[A]` is `{C}` and `right[A]` is
/// `{C, G}` (C is A's rightmost symbol, G is C's); the left side mirrors this with
/// `direct_left[A] = {B}` and `left[A] = {B, D}`.
struct Ends {
    direct_left: FxHashMap<String, FxHashSet<String>>,
    direct_right: FxHashMap<String, FxHashSet<String>>,
    left: FxHashMap<String, FxHashSet<String>>,
    right: FxHashMap<String, FxHashSet<String>>,
}

impl Ends {
    /// True if `head` can appear at the `side` end of a string derived from
    /// `from`: the rightmost symbol for `Right` (`from =>* α head`), the leftmost
    /// for `Left` (`from =>* head α`).
    fn reaches(&self, from: &str, head: &str, side: Side) -> bool {
        self.transitive(side)
            .get(from)
            .is_some_and(|set| set.contains(head))
    }

    /// The direct (single-step) `side` ends.
    fn direct(&self, side: Side) -> &FxHashMap<String, FxHashSet<String>> {
        match side {
            Side::Left => &self.direct_left,
            Side::Right => &self.direct_right,
        }
    }

    fn transitive(&self, side: Side) -> &FxHashMap<String, FxHashSet<String>> {
        match side {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }
}

/// Builds the left- and right-end relations over all nonterminals: the first and
/// last symbol of each alternative, and their transitive closures.
fn compute_ends(syntax_rules: &[SyntaxRule]) -> Ends {
    let mut direct_left: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();
    let mut direct_right: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();
    for rule in syntax_rules {
        for alt in rule.alternatives() {
            if let Some(id) = first_grammar_symbol(&alt.symbols).and_then(Symbol::as_identifier) {
                direct_left
                    .entry(rule.head.name.clone())
                    .or_default()
                    .insert(id.name.clone());
            }
            if let Some(id) = last_grammar_symbol(&alt.symbols).and_then(Symbol::as_identifier) {
                direct_right
                    .entry(rule.head.name.clone())
                    .or_default()
                    .insert(id.name.clone());
            }
        }
    }
    Ends {
        left: transitive_closure(&direct_left),
        right: transitive_closure(&direct_right),
        direct_left,
        direct_right,
    }
}

/// Transitive (one-or-more-step) closure of a single-step ends relation.
/// A node appears in its own reachable set only when it sits on a cycle.
fn transitive_closure(
    ends: &FxHashMap<String, FxHashSet<String>>,
) -> FxHashMap<String, FxHashSet<String>> {
    let mut result = FxHashMap::default();
    for start in ends.keys() {
        let mut reached = FxHashSet::default();
        let mut stack: Vec<&str> = ends[start].iter().map(String::as_str).collect();
        while let Some(node) = stack.pop() {
            if reached.insert(node.to_string())
                && let Some(node_ends) = ends.get(node)
            {
                stack.extend(node_ends.iter().map(String::as_str));
            }
        }
        result.insert(start.clone(), reached);
    }
    result
}

/// Assigns precedence numbers to priority levels in reverse order.
/// Bottom level = 1, each `>` boundary increments.
/// Levels whose alternatives all lack recursion (direct and indirect) get `None`.
fn assign_precedence(
    priority_levels: &[PriorityLevel],
    head: &str,
    recursive_name: &str,
    recursive_names: &[(String, String)],
    ends: &Ends,
) -> Vec<Option<i64>> {
    let mut result = vec![Option::<i64>::None; priority_levels.len()];
    let mut next_precedence: i64 = 1;

    // Iterate in reverse (bottom to top)
    for i in (0..priority_levels.len()).rev() {
        let has_recursive = priority_levels[i].alternatives.iter().any(|alt| {
            classify(alt, head, recursive_name, recursive_names, ends)
                != RecursionKind::NonRecursive
        });
        if has_recursive {
            result[i] = Some(next_precedence);
            next_precedence += 1;
        }
    }

    result
}

/// Desugars a single rule. `recursive_name` is the nonterminal that appears at
/// left/right recursive ends. For direct recursion this equals the head name;
/// for indirect recursion it is a different nonterminal.
fn desugar_rule(
    rule: SyntaxRule,
    recursive_name: &str,
    recursive_names: &[(String, String)],
    precedences: Vec<Option<i64>>,
    names_with_e: &FxHashSet<String>,
    ends: &Ends,
) -> SyntaxRule {
    let head_name = rule.head.name.clone();
    let head_has_e = names_with_e.contains(&head_name);

    // Find the minimum precedence among prefix alternatives (direct and
    // indirect). This determines which alternatives need the min trick.
    let min_prefix_pr = min_prefix_precedence(
        &rule.priority_levels,
        &precedences,
        &head_name,
        recursive_name,
        recursive_names,
        ends,
    );

    let mut all_alternatives = Vec::new();

    for (level, precedence) in rule.priority_levels.into_iter().zip(precedences.iter()) {
        let assoc = level.associativity;
        for alt in level.alternatives {
            // `exclude_desugaring` may have prepended `[(BIT_L & e) == 0]` guards
            // and appended `Return(label_index)` to alternatives of exclude-targeted
            // nonterminals. Strip both so `classify` and the rewrite helpers see the
            // actual recursive ends, then prepend the guards back to the desugared
            // form and combine the label index with our own precedence return.
            let (guard_prefix, core, trailing_label) = split_alt_decorations(alt);
            let recursion = classify(&core, &head_name, recursive_name, recursive_names, ends);
            let rewritten = match (recursion, precedence) {
                (RecursionKind::Binary, Some(pr)) => rewrite_binary(
                    core,
                    *pr,
                    assoc,
                    min_prefix_pr,
                    head_has_e,
                    trailing_label,
                    names_with_e,
                ),
                (RecursionKind::Prefix | RecursionKind::IndirectPrefix, Some(pr)) => {
                    rewrite_prefix(
                        recursive_name,
                        core,
                        *pr,
                        min_prefix_pr,
                        head_has_e,
                        trailing_label,
                        names_with_e,
                    )
                }
                (RecursionKind::Postfix | RecursionKind::IndirectPostfix, Some(pr)) => {
                    rewrite_postfix(
                        recursive_name,
                        core,
                        *pr,
                        head_has_e,
                        trailing_label,
                        names_with_e,
                    )
                }
                (RecursionKind::NonRecursive, _) => {
                    rewrite_non_recursive(recursive_name, core, head_has_e, trailing_label)
                }
                _ => {
                    // No rewrite applied (e.g., recursive ends found but no precedence
                    // assigned to this level). Reattach any trailing Return verbatim.
                    let mut symbols = core.symbols;
                    if let Some(label_idx) = trailing_label {
                        symbols.push(Symbol::Return(Expr::Int(label_idx)));
                    }
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

    // Prepend `p` to whatever parameters the head already carries (`e` from
    // `exclude_desugaring`, if any). Order is `[p, e]`.
    let mut parameters = Vec::with_capacity(1 + rule.head.parameters.len());
    parameters.push(Parameter {
        name: "p".to_string(),
        ty: ParamType::I32,
    });
    parameters.extend(rule.head.parameters);

    let head = Nonterminal {
        name: head_name,
        origin: rule.head.origin,
        parameters,
    };

    SyntaxRule {
        head,
        priority_levels: vec![PriorityLevel::new(all_alternatives)],
        layout: rule.layout,
        start: rule.start,
    }
}

/// Peels the decorations added by `exclude_desugaring` off an alternative:
/// the leading `Symbol::Condition` guards (returned as a prefix to re-prepend
/// later), and the trailing `Return(Int(label_index))` (returned as a bare
/// `i64` so the rewrite functions can pack it together with their own
/// precedence return value). Returns the stripped alternative core in between.
fn split_alt_decorations(alt: Alternative) -> (Vec<Symbol>, Alternative, Option<i64>) {
    let mut symbols = alt.symbols;

    let trailing_label =
        matches!(symbols.last(), Some(Symbol::Return(Expr::Int(_)))).then(|| match symbols.pop() {
            Some(Symbol::Return(Expr::Int(v))) => v,
            _ => unreachable!(),
        });

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

/// Packs a precedence value with a label index into a single i32:
/// `(pr << 16) | (label & 0xFFFF)` when the head carries `e`, else just `pr`.
/// The label occupies the low 16 bits; the precedence occupies the high half.
/// Constant-folds when `pr_expr` is an integer literal.
fn pack_return(pr_expr: Expr, label_idx: Option<i64>, head_has_e: bool) -> Expr {
    if !head_has_e {
        return pr_expr;
    }
    let label_low16: i64 = label_idx.unwrap_or(-1) & 0xFFFF;
    match pr_expr {
        Expr::Int(pr) => Expr::Int((pr << 16) | label_low16),
        other => Expr::BitOr(
            Box::new(Expr::Shl(Box::new(other), Box::new(Expr::Int(16)))),
            Box::new(Expr::Int(label_low16)),
        ),
    }
}

/// `value >> 16` when the source of `value` carries `e` (its return is packed);
/// otherwise the value itself.
fn unpack_pr_value(value: Expr, target_has_e: bool) -> Expr {
    if target_has_e {
        Expr::Shr(Box::new(value), Box::new(Expr::Int(16)))
    } else {
        value
    }
}

/// Finds the minimum precedence among prefix alternatives, direct (`op E`) and
/// indirect (`op X`, where X derives E at its right end). Returns `None` if there
/// are none.
fn min_prefix_precedence(
    priority_levels: &[PriorityLevel],
    precedences: &[Option<i64>],
    head: &str,
    recursive_name: &str,
    recursive_names: &[(String, String)],
    ends: &Ends,
) -> Option<i64> {
    priority_levels
        .iter()
        .zip(precedences.iter())
        .filter_map(|(level, prec)| {
            let pr = (*prec)?;
            let has_prefix = level.alternatives.iter().any(|alt| {
                matches!(
                    classify(alt, head, recursive_name, recursive_names, ends),
                    RecursionKind::Prefix | RecursionKind::IndirectPrefix
                )
            });
            has_prefix.then_some(pr)
        })
        .min()
}

/// Creates the return expression for the min trick:
///   `return r == 0 ? pr : min(r, pr)`
/// When the right recursive call's target carries `e`, `r` holds a packed
/// `(pr << 16) | label` and the comparison and min operate on its precedence
/// half via `r >> 16`. When the head carries `e`, the final return value is
/// packed with this alternative's `label_idx`.
fn make_min_return(pr: i64, label_idx: Option<i64>, head_has_e: bool, right_has_e: bool) -> Symbol {
    let r_pr = unpack_pr_value(Expr::Ref("r".to_string()), right_has_e);
    let precedence_value = Expr::Ternary {
        cond: Box::new(Expr::Cond(Cond {
            left: Box::new(r_pr.clone()),
            right: Box::new(Expr::Int(0)),
            op: CondOp::Eq,
        })),
        then: Box::new(Expr::Int(pr)),
        r#else: Box::new(Expr::Min(Box::new(r_pr), Box::new(Expr::Int(pr)))),
    };
    Symbol::Return(pack_return(precedence_value, label_idx, head_has_e))
}

/// Creates a right-end binding: `r=E(arg)` (or `r=E(arg, local_n)` if
/// `target_has_e`). The `local_n` bitmask comes from any `Symbol::Exclude` at
/// this call site that `exclude_desugaring` rewrote into `Call(E, [n])`.
fn make_right_binding(id: &Identifier, arg: i64, target_has_e: bool, local_n: i64) -> Symbol {
    Symbol::Binding {
        name: "r".to_string(),
        symbol: Box::new(Symbol::Call {
            name: id.clone(),
            arguments: recursive_call_args(Expr::Int(arg), target_has_e, local_n),
        }),
    }
}

/// Builds the argument list for a recursive call into a desugared nonterminal:
/// the precedence value, plus the local exclusion bitmask when the target
/// carries `e`. The caller's `e` is not propagated; only the local-site
/// exclusion is. Propagating the caller's `e` would split the GSS by
/// exclusion context and defeat sharing.
fn recursive_call_args(p_arg: Expr, target_has_e: bool, local_n: i64) -> Vec<Expr> {
    if target_has_e {
        vec![p_arg, Expr::Int(local_n)]
    } else {
        vec![p_arg]
    }
}

/// Pulls the local exclusion bitmask out of a recursive-end symbol.
/// `exclude_desugaring` rewrites `E!X` into `Call(E, [BIT_X])` and bare `E`
/// into `Call(E, [0])` for targeted `E`, so the bitmask sits at `arguments[0]`.
/// Returns `0` for symbols that don't carry a bitmask (e.g. non-targeted
/// identifiers).
fn extract_local_bitmask(symbol: &Symbol) -> i64 {
    if let Symbol::Call { arguments, .. } = symbol
        && let Some(Expr::Int(bitmask)) = arguments.first()
    {
        *bitmask
    } else {
        0
    }
}

/// Extracts the Identifier from a symbol at a recursive end. Accepts both
/// `Symbol::Identifier` and `Symbol::Call` (the latter shows up after
/// `exclude_desugaring` for exclude-targeted nonterminals).
fn extract_identifier(symbol: &Symbol) -> &Identifier {
    match symbol {
        Symbol::Identifier(id) => id,
        Symbol::Call { name, .. } => name,
        _ => panic!("expected Identifier or Call at recursive end, got {symbol:?}"),
    }
}

/// Creates the precondition symbol: [pr >= p]
fn make_precondition(pr: i64) -> Symbol {
    Symbol::Condition(Expr::Cond(Cond {
        left: Box::new(Expr::Int(pr)),
        right: Box::new(Expr::Ref("p".to_string())),
        op: CondOp::Geq,
    }))
}

/// Creates the left binding symbol: `l=E(p)` (or `l=E(p, 0)` if `target_has_e`).
/// The caller's `e` is not propagated to the left recursive call; that would
/// split the GSS by exclusion context and defeat sharing. The local exclusion
/// at this position is enforced via `make_local_exclusion_postcondition`
/// instead.
fn make_left_binding(id: &Identifier, target_has_e: bool) -> Symbol {
    Symbol::Binding {
        name: "l".to_string(),
        symbol: Box::new(Symbol::Call {
            name: id.clone(),
            arguments: recursive_call_args(Expr::Ref("p".to_string()), target_has_e, 0),
        }),
    }
}

/// Builds a post-condition that filters left recursive matches whose returned
/// label is in the local exclusion set, while admitting matches that returned
/// the `-1` sentinel (unlabeled or not-in-table alternatives). The condition
/// is `[(l & 0xFFFF) == 0xFFFF || ((local_n >> (l & 0xFFFF)) & 1) == 0]`: the
/// label half of `l` is either the sentinel, or its bit is clear in
/// `local_n`.
fn make_local_exclusion_postcondition(local_n: i64) -> Symbol {
    let l_label = || {
        Expr::BitAnd(
            Box::new(Expr::Ref("l".to_string())),
            Box::new(Expr::Int(0xFFFF)),
        )
    };
    Symbol::Condition(Expr::Or(
        Box::new(Expr::Cond(Cond {
            left: Box::new(l_label()),
            right: Box::new(Expr::Int(0xFFFF)),
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

/// Creates the postcondition symbol: `[l == 0 || l >= pr]`. When the left
/// binding's target carries `e`, `l` holds a packed `(pr << 16) | label` and
/// both comparisons operate on its precedence half via `l >> 16`.
fn make_postcondition(pr: i64, left_has_e: bool) -> Symbol {
    let l_pr = unpack_pr_value(Expr::Ref("l".to_string()), left_has_e);
    Symbol::Condition(Expr::Or(
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
    ))
}

/// Replaces a non-recursive reference to the head nonterminal with a call
/// passing 0 (precedence) and, if the head carries `e`, an additional 0
/// (cleared exclusion: a fresh expression context, e.g. inside `'(' E ')'`).
/// Accepts both `Symbol::Identifier` and `Symbol::Call`. The latter shows up
/// when `exclude_desugaring` has already rewritten a bare reference into
/// `Call(E, [bitmask])`, in which case we prepend `0` to the existing args.
fn replace_head_ref(symbol: Symbol, head_name: &str, head_has_e: bool) -> Symbol {
    match symbol {
        Symbol::Identifier(id) if id.name == head_name => {
            let arguments = if head_has_e {
                vec![Expr::Int(0), Expr::Int(0)]
            } else {
                vec![Expr::Int(0)]
            };
            Symbol::Call {
                name: Identifier {
                    name: id.name.clone(),
                    definition: id.definition,
                },
                arguments,
            }
        }
        Symbol::Call { name, arguments } if name.name == head_name => {
            let mut new_args = vec![Expr::Int(0)];
            new_args.extend(arguments);
            Symbol::Call {
                name,
                arguments: new_args,
            }
        }
        Symbol::Labeled { label, symbol } => Symbol::Labeled {
            label,
            symbol: Box::new(replace_head_ref(*symbol, head_name, head_has_e)),
        },
        _ => symbol,
    }
}

/// Rewrites a binary alternative `E op E` at precedence level `pr` into a
/// data-dependent form. The exact rewrite depends on associativity:
///
/// - No associativity (default):
///   `[pr>=p] l=E(p) [l==0||l>=pr] op E(pr) return pr`
///
/// - Left-associative:
///   `[pr>=p] l=E(p) [l==0||l>=pr] op E(pr+1) return pr`
///   (right end gets pr+1 to prevent right-recursive use at same level)
///
/// - Right-associative:
///   `[pr>=p] l=E(p) [l==0||l>=pr+1] op E(pr) return pr`
///   (postcondition uses pr+1 to prevent left-recursive use at same level)
///
/// - Non-associative:
///   `[pr>=p] l=E(p) [l==0||l>=pr+1] op E(pr+1) return pr`
///   (both restrictions)
fn rewrite_binary(
    alt: Alternative,
    pr: i64,
    assoc: Option<Associativity>,
    min_prefix_pr: Option<i64>,
    head_has_e: bool,
    label_idx: Option<i64>,
    names_with_e: &FxHashSet<String>,
) -> Alternative {
    let postcond_threshold = match assoc {
        Some(Associativity::Right | Associativity::NonAssoc) => pr + 1,
        _ => pr,
    };

    let right_arg = match assoc {
        Some(Associativity::Left | Associativity::NonAssoc) => pr + 1,
        _ => pr,
    };

    let use_min = min_prefix_pr.is_some_and(|min_pr| pr > min_pr);

    // Use the actual identifiers from the alternative's left/right ends, not the
    // recursive name. For indirect recursion the recursive name and the
    // identifier at the end may differ; using the end identifier preserves the
    // original nonterminal identity.
    let left_id = extract_identifier(alt.symbols.first().unwrap()).clone();
    let right_id = extract_identifier(alt.symbols.last().unwrap()).clone();
    let left_has_e = names_with_e.contains(&left_id.name);
    let right_has_e = names_with_e.contains(&right_id.name);
    let left_local_n = extract_local_bitmask(alt.symbols.first().unwrap());
    let right_local_n = extract_local_bitmask(alt.symbols.last().unwrap());

    let mut symbols = Vec::new();

    symbols.push(make_precondition(pr));
    symbols.push(make_left_binding(&left_id, left_has_e));
    symbols.push(make_postcondition(postcond_threshold, left_has_e));
    if left_has_e && left_local_n != 0 {
        symbols.push(make_local_exclusion_postcondition(left_local_n));
    }

    let num_symbols = alt.symbols.len();
    for symbol in alt
        .symbols
        .into_iter()
        .skip(1)
        .take(num_symbols.saturating_sub(2))
    {
        symbols.push(symbol);
    }

    if use_min {
        symbols.push(make_right_binding(
            &right_id,
            right_arg,
            right_has_e,
            right_local_n,
        ));
        symbols.push(make_min_return(pr, label_idx, head_has_e, right_has_e));
    } else {
        symbols.push(Symbol::Call {
            name: right_id,
            arguments: recursive_call_args(Expr::Int(right_arg), right_has_e, right_local_n),
        });
        symbols.push(Symbol::Return(pack_return(
            Expr::Int(pr),
            label_idx,
            head_has_e,
        )));
    }

    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Rewrites a prefix alternative `op E` at precedence level `pr` into:
///   op E(pr) return pr
/// Or with the min trick (when a prefix exists at lower precedence):
///   op r=E(pr) return r == 0 ? pr : min(r, pr)
fn rewrite_prefix(
    recursive_name: &str,
    alt: Alternative,
    pr: i64,
    min_prefix_pr: Option<i64>,
    head_has_e: bool,
    label_idx: Option<i64>,
    names_with_e: &FxHashSet<String>,
) -> Alternative {
    let mut symbols = Vec::new();
    let num_symbols = alt.symbols.len();

    let use_min = min_prefix_pr.is_some_and(|min_pr| pr > min_pr);

    let right_id = extract_identifier(alt.symbols.last().unwrap()).clone();
    let right_has_e = names_with_e.contains(&right_id.name);
    let right_local_n = extract_local_bitmask(alt.symbols.last().unwrap());

    for symbol in alt.symbols.into_iter().take(num_symbols.saturating_sub(1)) {
        symbols.push(replace_head_ref(symbol, recursive_name, head_has_e));
    }

    if use_min {
        symbols.push(make_right_binding(
            &right_id,
            pr,
            right_has_e,
            right_local_n,
        ));
        symbols.push(make_min_return(pr, label_idx, head_has_e, right_has_e));
    } else {
        symbols.push(Symbol::Call {
            name: right_id,
            arguments: recursive_call_args(Expr::Int(pr), right_has_e, right_local_n),
        });
        symbols.push(Symbol::Return(pack_return(
            Expr::Int(pr),
            label_idx,
            head_has_e,
        )));
    }

    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Rewrites a postfix alternative `E op` at precedence level `pr` into:
///   [pr>=p] l=E(p) [l==0||l>=pr] op return 0
fn rewrite_postfix(
    recursive_name: &str,
    alt: Alternative,
    pr: i64,
    head_has_e: bool,
    label_idx: Option<i64>,
    names_with_e: &FxHashSet<String>,
) -> Alternative {
    let left_id = extract_identifier(alt.symbols.first().unwrap()).clone();
    let left_has_e = names_with_e.contains(&left_id.name);
    let left_local_n = extract_local_bitmask(alt.symbols.first().unwrap());

    let mut symbols = Vec::new();

    symbols.push(make_precondition(pr));
    symbols.push(make_left_binding(&left_id, left_has_e));
    symbols.push(make_postcondition(pr, left_has_e));
    if left_has_e && left_local_n != 0 {
        symbols.push(make_local_exclusion_postcondition(left_local_n));
    }

    for symbol in alt.symbols.into_iter().skip(1) {
        symbols.push(replace_head_ref(symbol, recursive_name, head_has_e));
    }

    symbols.push(Symbol::Return(pack_return(
        Expr::Int(0),
        label_idx,
        head_has_e,
    )));

    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Rewrites a non-recursive alternative: replaces any E references with E(0)
/// (or E(0, 0) when E carries `e`) and appends `return 0` (packed with the
/// alternative's label index when the head carries `e`).
fn rewrite_non_recursive(
    head_name: &str,
    alt: Alternative,
    head_has_e: bool,
    label_idx: Option<i64>,
) -> Alternative {
    let mut symbols: Vec<Symbol> = alt
        .symbols
        .into_iter()
        .map(|symbol| replace_head_ref(symbol, head_name, head_has_e))
        .collect();

    symbols.push(Symbol::Return(pack_return(
        Expr::Int(0),
        label_idx,
        head_has_e,
    )));

    Alternative {
        symbols,
        label: alt.label,
    }
}

/// Threads a precedence parameter through a nonterminal on the path from an
/// indirect prefix (`side` = `Right`) or postfix (`side` = `Left`) to the head.
/// Each alternative whose `side` end reaches `head` binds that end to `r` and
/// returns it; an alternative that does not reach `head` returns 0.
fn thread_intermediate(
    rule: SyntaxRule,
    head: &str,
    side: Side,
    names_with_e: &FxHashSet<String>,
    ends: &Ends,
) -> SyntaxRule {
    let SyntaxRule {
        head: rule_head,
        priority_levels,
        layout,
        start,
    } = rule;

    let mut alternatives = Vec::new();
    for alt in priority_levels
        .into_iter()
        .flat_map(|level| level.alternatives)
    {
        let mut symbols = alt.symbols;
        // Intermediates are never exclude targets, so they carry no trailing
        // decoration and the appended return is the alternative's last symbol.
        debug_assert!(
            symbols.last().is_none_or(|symbol| !is_decoration(symbol)),
            "intermediate alternative has a trailing decoration"
        );
        // The `side` end when it reaches `head`, with its index: this end gets
        // threaded with `p` and bound to `r`.
        let head_end = end_index(&symbols, side).and_then(|i| {
            let id = symbols[i].as_identifier()?;
            (id.name == head || ends.reaches(&id.name, head, side)).then(|| (i, id.clone()))
        });
        if let Some((i, id)) = head_end {
            let has_e = names_with_e.contains(&id.name);
            symbols[i] = Symbol::Binding {
                name: "r".to_string(),
                symbol: Box::new(threaded_call(&id, has_e)),
            };
            symbols.push(Symbol::Return(Expr::Ref("r".to_string())));
        } else {
            symbols.push(Symbol::Return(Expr::Int(0)));
        }
        alternatives.push(Alternative {
            symbols,
            label: alt.label,
        });
    }

    let mut parameters = Vec::with_capacity(1 + rule_head.parameters.len());
    parameters.push(Parameter {
        name: "p".to_string(),
        ty: ParamType::I32,
    });
    parameters.extend(rule_head.parameters);

    SyntaxRule {
        head: Nonterminal {
            name: rule_head.name,
            origin: rule_head.origin,
            parameters,
        },
        priority_levels: vec![PriorityLevel::new(alternatives)],
        layout,
        start,
    }
}

/// A call to `id` passing the threaded precedence `p`, plus a cleared exclusion
/// `0` when the target carries `e`, so the call is at the target's full arity.
fn threaded_call(id: &Identifier, has_e: bool) -> Symbol {
    let mut arguments = vec![Expr::Ref("p".to_string())];
    if has_e {
        arguments.push(Expr::Int(0));
    }
    Symbol::Call {
        name: id.clone(),
        arguments,
    }
}

/// Updates references to desugared nonterminals in non-desugared rules:
/// `E` becomes `E(0)` (or `E(0, 0)` if `E` also carries `e` from
/// `exclude_desugaring`). Existing `Call(E, [bitmask])` references (left
/// behind by `exclude_desugaring`) get `0` prepended to become `E(0, bitmask)`.
/// Calls already at the target's full arity are left untouched, which keeps
/// the cascade rule's own desugared body from being re-rewritten on the second
/// pass.
fn update_external_references(
    rule: SyntaxRule,
    desugared_names: &[String],
    names_with_e: &FxHashSet<String>,
) -> SyntaxRule {
    transform_syntax_rule(rule, |symbol| match symbol {
        Symbol::Identifier(id) if desugared_names.contains(&id.name) => {
            let target_arity = target_arity_for(&id.name, names_with_e);
            let arguments = vec![Expr::Int(0); target_arity];
            Symbol::Call {
                name: Identifier {
                    name: id.name.clone(),
                    definition: id.definition,
                },
                arguments,
            }
        }
        Symbol::Call { name, arguments } if desugared_names.contains(&name.name) => {
            let target_arity = target_arity_for(&name.name, names_with_e);
            if arguments.len() < target_arity {
                let missing = target_arity - arguments.len();
                let mut new_args = vec![Expr::Int(0); missing];
                new_args.extend(arguments);
                Symbol::Call {
                    name,
                    arguments: new_args,
                }
            } else {
                Symbol::Call { name, arguments }
            }
        }
        other => other,
    })
}

/// Parameter count of a desugared nonterminal: 1 for `p`, plus 1 for `e` if
/// `exclude_desugaring` added it.
fn target_arity_for(name: &str, names_with_e: &FxHashSet<String>) -> usize {
    if names_with_e.contains(name) { 2 } else { 1 }
}

#[cfg(test)]
mod tests {
    use crate::{
        alternative, bind, call, cond, cond_expr, exclude,
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
        let actual: Grammar = input_grammar().try_into().unwrap();
        let expected: Grammar = expected_grammar().try_into().unwrap();
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
    ///     = 'a' return 0
    ///     | [4>=p] l=E(p) [l==0||l>=4] '!' return 0
    ///     | '-' E(3) return 3
    ///     | [2>=p] l=E(p) [l==0||l>=2] '*' E(2) return 2
    ///     | [1>=p] l=E(p) [l==0||l>=1] '+' E(1) return 1
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
        let actual: Grammar = prefix_postfix_input().try_into().unwrap();
        let expected: Grammar = prefix_postfix_expected().try_into().unwrap();
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

        // E(p) = 'a' return 0 | [1>=p] l=E(p) [l==0||l>=1] '+' E(2) return 1
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
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

        // E(p) = 'a' return 0 | [1>=p] l=E(p) [l==0||l>=2] ';' E(1) return 1
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
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

        // E(p) = 'a' return 0 | [1>=p] l=E(p) [l==0||l>=2] '<' E(2) return 1
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
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
        //   = 'a' return 0
        //   | [3>=p] l=E(p) [l==0||l>=3] '*' E(4) return 3
        //   | [3>=p] l=E(p) [l==0||l>=3] '/' E(4) return 3
        //   | [2>=p] l=E(p) [l==0||l>=2] '+' E(3) return 2
        //   | [2>=p] l=E(p) [l==0||l>=2] '-' E(3) return 2
        //   | [1>=p] l=E(p) [l==0||l>=2] '<' E(2) return 1
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
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
    ///     = 'a' return 0
    ///     | [2>=p] l=E(p) [l==0||l>=2] '+' r=E(2) return r == 0 ? 2 : min(r, 2)
    ///     | 'if' E(0) 'then' E(0) 'else' E(1) return 1
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    /// Indirect recursion: F is a filtered copy of E (e.g., from exclude desugaring).
    /// F's alternatives reference E at their ends, making F indirectly recursive with
    /// recursive name E. When E has an alternative with F at the left end,
    /// `classify_ends` must recognize F as an indirect reference to E.
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
        //   'a' return 0
        //   [2>=p] l=F(p) [l==0||l>=2] '*' 'b' return 0    -- F at left end, preserved
        //   [1>=p] l=E(p) [l==0||l>=1] '+' E(1) return 1
        //
        // F(p):
        //   'a' return 0
        //   [1>=p] l=E(p) [l==0||l>=1] '+' E(1) return 1
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l", call!("F", ref "p")),
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

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    /// Exclude inside a precedence cascade.
    /// `E!B` does not synthesize a separate `E_except_B` rule. Instead, `E`
    /// grows an `e: i32` parameter (a bitmask of excluded label bits), every
    /// labeled alternative gains a `[(BIT_L & e) == 0]` guard and a
    /// `Return(label_index)` that the precedence pass packs together with its
    /// own precedence return value as `(pr << 16) | (label & 0xFFFF)`. The
    /// reference `E!B` becomes `E(p, BIT_B)`.
    ///
    /// The packed return shows up in two places:
    ///   - The final `Return` of each desugared alt encodes `(pr, label)`.
    ///   - The left-binding's postcondition unpacks via `Shr(l, 16)` so the
    ///     precedence comparison still operates on `pr` and not on the
    ///     packed value.
    ///
    ///   E
    ///     = "a"           #A
    ///     > E!B "++"      #C
    ///     > E "+" E       #D
    ///     > E "-" E       #B
    ///
    /// Labels assigned in source order:  A = bit 0 (1), C = bit 1 (2),
    /// D = bit 2 (4), B = bit 3 (8). Precedences (bottom = 1): B = 1, D = 2,
    /// C = 3. Packed returns:
    ///   #A:  (0 << 16) | 0      = 0
    ///   #C:  (0 << 16) | 1      = 1
    ///   #D:  (2 << 16) | 2      = 131074
    ///   #B:  (1 << 16) | 3      = 65539
    #[test]
    fn test_exclude_inside_precedence_cascade() {
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
            Symbol::Condition(Expr::Cond(Cond {
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

        // `l >> 16`. Unpacks the precedence half of a packed return value.
        fn l_pr() -> Expr {
            Expr::Shr(
                Box::new(Expr::Ref("l".to_string())),
                Box::new(Expr::Int(16)),
            )
        }

        // Packed precedence postcondition: `[(l >> 16) == 0 || (l >> 16) >= pr]`.
        fn packed_postcondition(pr: i64) -> Symbol {
            Symbol::Condition(Expr::Or(
                Box::new(Expr::Cond(Cond {
                    left: Box::new(l_pr()),
                    right: Box::new(Expr::Int(0)),
                    op: CondOp::Eq,
                })),
                Box::new(Expr::Cond(Cond {
                    left: Box::new(l_pr()),
                    right: Box::new(Expr::Int(pr)),
                    op: CondOp::Geq,
                })),
            ))
        }

        // Local-exclusion postcondition for the left recursive end:
        // `[(l & 0xFFFF) == 0xFFFF || ((local_n >> (l & 0xFFFF)) & 1) == 0]`.
        fn local_exclusion_postcondition(local_n: i64) -> Symbol {
            let l_label = || {
                Expr::BitAnd(
                    Box::new(Expr::Ref("l".to_string())),
                    Box::new(Expr::Int(0xFFFF)),
                )
            };
            Symbol::Condition(Expr::Or(
                Box::new(Expr::Cond(Cond {
                    left: Box::new(l_label()),
                    right: Box::new(Expr::Int(0xFFFF)),
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
                    alternative!(exclusion_guard(1), lit!("a"), ret!(0); #A),
                    // #C: postfix at pr=3, left has local exclusion BIT_B=8.
                    alternative!(
                        exclusion_guard(2),
                        cond!(3 >= "p"),
                        bind!("l", e_call(Expr::Ref("p".to_string()), 0)),
                        packed_postcondition(3),
                        local_exclusion_postcondition(8),
                        lit!("++"),
                        ret!(1);
                        #C
                    ),
                    alternative!(
                        exclusion_guard(4),
                        cond!(2 >= "p"),
                        bind!("l", e_call(Expr::Ref("p".to_string()), 0)),
                        packed_postcondition(2),
                        lit!("+"),
                        e_call(Expr::Int(2), 0),
                        ret!(131074);
                        #D
                    ),
                    alternative!(
                        exclusion_guard(8),
                        cond!(1 >= "p"),
                        bind!("l", e_call(Expr::Ref("p".to_string()), 0)),
                        packed_postcondition(1),
                        lit!("-"),
                        e_call(Expr::Int(1), 0),
                        ret!(65539);
                        #B
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    /// Right ends follow rightmost symbols transitively, the shape of Java's lambda:
    ///   Expression = 'a' > Lambda
    ///   Lambda = LambdaParameters '->' LambdaBody
    ///   LambdaBody = Expression | Block
    ///   LambdaParameters = Identifier
    /// `Lambda` reaches `Expression` through `LambdaBody`.
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
        use super::Side::Right;

        // Lambda reaches Expression through LambdaBody; LambdaBody directly.
        assert!(ends.reaches("Lambda", "Expression", Right));
        assert!(ends.reaches("LambdaBody", "Expression", Right));
        // LambdaParameters is a left end of Lambda, not a right end of anything here.
        assert!(!ends.reaches("LambdaParameters", "Expression", Right));
        assert!(ends.reaches("Lambda", "LambdaParameters", super::Side::Left));
    }

    /// `recurses_indirectly` reports the lambda alternative at the right (its end
    /// reaches Expression through a nonterminal) but not the direct binary or atom:
    ///   Expression = 'a' > Expression '+' Expression > Lambda
    ///   Lambda = LambdaParameters '->' LambdaBody
    ///   LambdaBody = Expression | Block
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
        let right = |alt| super::recurses_indirectly(alt, "Expression", &ends, super::Side::Right);

        assert!(!right(alternatives[0])); // 'a': no recursion
        assert!(!right(alternatives[1])); // Expression '+' Expression: direct, left to `classify_ends`
        assert!(right(alternatives[2])); // Lambda: right end reaches Expression through a nonterminal
    }

    /// `classify` combines direct and indirect recursion: the atom is
    /// non-recursive, the direct binary is `Binary`, and the lambda alternative
    /// is `IndirectPrefix`.
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

        let kind = |alt| super::classify(alt, "Expression", "Expression", &[], &ends);

        assert_eq!(kind(alternatives[0]), RecursionKind::NonRecursive);
        assert_eq!(kind(alternatives[1]), RecursionKind::Binary);
        assert_eq!(kind(alternatives[2]), RecursionKind::IndirectPrefix);
    }

    /// Desugaring an indirect prefix (the lambda shape):
    ///   E = 'a' > E '+' E > Lambda
    ///   Lambda = 'fn' Body
    ///   Body = E
    /// `Lambda` is ranked as the bottom prefix (level 1), so '+' rises to 2 and
    /// gains the min trick. `Lambda` and `Body` are threaded with `p`.
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

        // E(p) = 'a' return 0
        //      | [2>=p] l=E(p) [l==0||l>=2] '+' r=E(2) return r==0 ? 2 : min(r,2)
        //      | Lambda(1) return 1
        // Lambda(p) = 'fn' r=Body(p) return r
        // Body(p)   = r=E(p) return r
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
                    alternative!(call!("Lambda", 1), ret!(1)),
                )),
                syntax_rule!("Lambda"("p": I32) => priority_level!(
                    alternative!(
                        lit!("fn"),
                        bind!("r", call!("Body", ref "p")),
                        ret!(expr Expr::Ref("r".to_string())),
                    ),
                )),
                syntax_rule!("Body"("p": I32) => priority_level!(
                    alternative!(
                        bind!("r", call!("E", ref "p")),
                        ret!(expr Expr::Ref("r".to_string())),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    /// Desugaring an indirect postfix (the mirror of the lambda shape):
    ///   E = 'a' > E '+' E > Postfix
    ///   Postfix = Body '!'
    ///   Body = E
    /// `Postfix` is ranked as the bottom level (1), '+' rises to 2. `Postfix` and
    /// `Body` are threaded with `p` from their left end.
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

        // E(p) = 'a' return 0
        //      | [2>=p] l=E(p) [l==0||l>=2] '+' E(2) return 2
        //      | [1>=p] l=Postfix(p) [l==0||l>=1] return 0
        // Postfix(p) = r=Body(p) '!' return r
        // Body(p)    = r=E(p) return r
        let expected = grammar_def!("Test",
            syntax: [
                syntax_rule!("E"("p": I32) => priority_level!(
                    alternative!(lit!("a"), ret!(0)),
                    alternative!(
                        cond!(2 >= "p"),
                        bind!("l", call!("E", ref "p")),
                        cond!(("l" == 0) || ("l" >= 2)),
                        lit!("+"),
                        call!("E", 2),
                        ret!(2),
                    ),
                    alternative!(
                        cond!(1 >= "p"),
                        bind!("l", call!("Postfix", ref "p")),
                        cond!(("l" == 0) || ("l" >= 1)),
                        ret!(0),
                    ),
                )),
                syntax_rule!("Postfix"("p": I32) => priority_level!(
                    alternative!(
                        bind!("r", call!("Body", ref "p")),
                        lit!("!"),
                        ret!(expr Expr::Ref("r".to_string())),
                    ),
                )),
                syntax_rule!("Body"("p": I32) => priority_level!(
                    alternative!(
                        bind!("r", call!("E", ref "p")),
                        ret!(expr Expr::Ref("r".to_string())),
                    ),
                )),
            ]
        );

        let actual: Grammar = input.try_into().unwrap();
        let expected: Grammar = expected.try_into().unwrap();
        assert_eq!(actual, expected);
    }
}
