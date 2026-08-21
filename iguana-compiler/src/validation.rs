use std::path::Path;

use iguana_runtime::input::{Input, Span};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::generator::grammar_utils::{parse_tree_builder_ident, parser_ident};
use crate::grammar::{
    def::{GrammarDef, LexicalRule, SyntaxRule},
    symbols::{Identifier, Symbol, Terminal},
};
use crate::spans::GrammarSpans;
use crate::utils::{to_pascal_case, to_snake_case};

/// A grammar error.
#[derive(Debug, Clone)]
pub struct GrammarError {
    pub message: String,
    pub span: Span,
}

/// Renders errors for the command line, one per line, each located in the
/// grammar file as `path:line:column: message`.
///
/// Lines and columns are 1-based.
pub fn render_errors(errors: &[GrammarError], path: &Path, source: &str) -> String {
    let input = Input::from(source);
    errors
        .iter()
        .map(|error| {
            let (line, column) = input.line_column(error.span.left_extent);
            format!(
                "{}:{}:{}: {}",
                path.display(),
                line + 1,
                column + 1,
                error.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Checks a resolved `GrammarDef` and reports all errors.
pub fn validate<'a>(grammar_def: &'a GrammarDef, spans: &GrammarSpans<'a>) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    check_duplicate_definitions(grammar_def, spans, &mut errors);
    check_generated_rule_name_collisions(grammar_def, spans, &mut errors);
    check_unresolved_identifiers(grammar_def, spans, &mut errors);
    check_exclusions(grammar_def, spans, &mut errors);
    check_restriction_targets(grammar_def, spans, &mut errors);
    check_one_label_per_symbol(grammar_def, spans, &mut errors);
    check_reserved_names(grammar_def, spans, &mut errors);
    check_layout_is_not_an_identifier_rule(grammar_def, spans, &mut errors);
    check_grammar_has_a_syntax_rule(grammar_def, &mut errors);
    errors
}

/// A grammar needs at least one syntax rule. Generating from a grammar without
/// one produces a crate with an empty token kind and no start symbol, which
/// does not compile. The error carries the start of the file, since a grammar
/// that defines nothing has no rule to point at.
fn check_grammar_has_a_syntax_rule(grammar_def: &GrammarDef, errors: &mut Vec<GrammarError>) {
    if grammar_def.syntax_rules.is_empty() {
        errors.push(GrammarError {
            message: format!("grammar `{}` has no syntax rules", grammar_def.name),
            span: Span::new(0, 0),
        });
    }
}

/// Rule names must be unique. When two rules have the same name, the error is
/// reported on the later rule.
///
/// Lexical rules are checked before syntax rules, the same order that assigns
/// definition IDs. As a result, a name used by both lexical and syntax rules is
/// reported on the syntax rule.
fn check_duplicate_definitions<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    let mut seen = FxHashSet::default();
    for rule in &grammar_def.lexical_rules {
        if !seen.insert(&rule.head.name) {
            errors.push(GrammarError {
                message: format!("duplicate definition `{}`", rule.head.name),
                span: spans.terminal(&rule.head).span,
            });
        }
    }
    for rule in &grammar_def.syntax_rules {
        if !seen.insert(&rule.head.name) {
            errors.push(GrammarError {
                message: format!("duplicate definition `{}`", rule.head.name),
                span: spans.nonterminal(&rule.head).span,
            });
        }
    }
}

/// Reports syntax rule names that collide after conversion to Rust identifiers.
///
/// Each syntax rule produces two identifiers:
/// - A `PascalCase` parse-tree type.
/// - An `UPPER_SNAKE_CASE` nonterminal ID constant.
///
/// Duplicate source names are reported separately. This check reports only
/// names that become equal during either conversion.
fn check_generated_rule_name_collisions<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    // A map from generated `PascalCase` type names to source rule names.
    let mut type_names = FxHashMap::default();
    // A map from generated `UPPER_SNAKE_CASE` constant names to source rule names.
    let mut constant_names = FxHashMap::default();
    // A collision between the same two rules is reported only once. For example,
    // `Foo` and `foo` collide as both the type `Foo` and the constant `FOO`. The
    // type collision is checked first, so the constant collision is skipped.
    let mut reported = FxHashSet::default();

    for rule in &grammar_def.syntax_rules {
        let name = rule.head.name.as_str();
        let span = spans.nonterminal(&rule.head).span;
        let type_name = to_pascal_case(name);
        if let Some(previous) = type_names.insert(type_name.clone(), name)
            && previous != name
            && reported.insert((previous, name))
        {
            errors.push(GrammarError {
                message: format!(
                    "`{previous}` and `{name}` have the same generated name `{type_name}`"
                ),
                span,
            });
        }

        let constant_name = constant_name(name);
        if let Some(previous) = constant_names.insert(constant_name.clone(), name)
            && previous != name
            && reported.insert((previous, name))
        {
            errors.push(GrammarError {
                message: format!(
                    "`{previous}` and `{name}` have the same generated name `{constant_name}`"
                ),
                span,
            });
        }
    }
}

/// Reports unresolved rule references.
///
/// Resolution assigns definition IDs to references in syntax rules and lexical
/// rule bodies. Restrictions attached directly to lexical rules are resolved
/// only after validation, so this check compares their names with the defined
/// rule names.
fn check_unresolved_identifiers<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    grammar_def.for_each_identifier(&mut |identifier| {
        if identifier.definition.is_none() {
            errors.push(GrammarError {
                message: format!("unresolved identifier `{}`", identifier.name),
                span: spans.identifier(identifier).span,
            });
        }
    });

    let defined = defined_names(grammar_def);
    for rule in &grammar_def.lexical_rules {
        for (_, identifier) in lexical_restrictions(rule) {
            if !defined.contains(identifier.name.as_str()) {
                errors.push(GrammarError {
                    message: format!("unresolved identifier `{}`", identifier.name),
                    span: spans.identifier(identifier).span,
                });
            }
        }
    }
}

/// The names of every rule, lexical and syntax alike.
fn defined_names(grammar_def: &GrammarDef) -> FxHashSet<&str> {
    let mut names = FxHashSet::default();
    for rule in &grammar_def.lexical_rules {
        names.insert(rule.head.name.as_str());
    }
    for rule in &grammar_def.syntax_rules {
        names.insert(rule.head.name.as_str());
    }
    names
}

/// A lexical rule's own restrictions, each paired with its operator.
fn lexical_restrictions(rule: &LexicalRule) -> Vec<(RestrictionKind, &Identifier)> {
    let mut restrictions = Vec::new();
    for identifier in &rule.except {
        restrictions.push((RestrictionKind::Except, identifier));
    }
    for identifier in &rule.follow_restriction {
        restrictions.push((RestrictionKind::Follow, identifier));
    }
    if let Some(identifier) = &rule.precede_restriction {
        restrictions.push((RestrictionKind::Precede, identifier));
    }
    restrictions
}

/// Checks an exclusion (`A!label`): it applies to a reference to a syntax
/// rule, and each label names one of that rule's alternatives.
///
/// Anything else has no alternatives to exclude. Without this check, EBNF
/// expansion lifts a group or repetition into a generated rule, and exclusion
/// desugaring later panics about a label on a name the user did not write.
fn check_exclusions<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    let rules_by_name: FxHashMap<&str, &SyntaxRule> = grammar_def
        .syntax_rules
        .iter()
        .map(|rule| (rule.head.name.as_str(), rule))
        .collect();

    grammar_def.for_each_symbol(&mut |symbol| {
        let Symbol::Exclude {
            symbol: inner,
            labels,
        } = symbol
        else {
            return;
        };
        // The spans sit in source order, parallel to `labels`, and an
        // exclusion has at least one label, so indexing cannot miss.
        let label_spans = spans.label_spans(symbol);
        let first_label_span = label_spans[0];

        let Some(identifier) = inner.as_identifier() else {
            errors.push(GrammarError {
                message: format!(
                    "exclusion `{}` only applies to a syntax rule reference",
                    render_labels(labels)
                ),
                span: first_label_span,
            });
            return;
        };
        let Some(rule) = rules_by_name.get(identifier.name.as_str()) else {
            // An unresolved name is already reported on its own.
            if identifier.definition.is_some() {
                errors.push(GrammarError {
                    message: format!(
                        "exclusion `{}` only applies to a syntax rule reference, and `{}` is a lexical rule",
                        render_labels(labels),
                        identifier.name
                    ),
                    span: first_label_span,
                });
            }
            return;
        };

        for (index, label) in labels.iter().enumerate() {
            if !rule.has_label(label) {
                errors.push(GrammarError {
                    message: format!("unresolved label `{label}`"),
                    span: label_spans[index],
                });
            }
        }
    });
}

/// Renders an exclusion's labels the way they are written, `!a!b`.
fn render_labels(labels: &[String]) -> String {
    labels.iter().map(|label| format!("!{label}")).collect()
}

/// One of the four restriction operators.
#[derive(Clone, Copy, PartialEq)]
enum RestrictionKind {
    Except,
    Follow,
    LayoutAwareFollow,
    Precede,
}

impl RestrictionKind {
    /// What to call the operator in a message.
    fn name(self) -> &'static str {
        match self {
            RestrictionKind::Except => "except",
            RestrictionKind::Follow => "follow restriction",
            RestrictionKind::LayoutAwareFollow => "layout-aware follow restriction",
            RestrictionKind::Precede => "precede restriction",
        }
    }
}

/// Reports invalid restriction operands:
///
/// - **Lexical operand.** Every operand must name a lexical rule. Missing names
///   are reported separately. This check reports operands that name syntax
///   rules. The scanner matches lexical rules, not syntax rules.
/// - **Unrestricted `\` operand.** A `\` operand must name a lexical rule
///   without restrictions of its own. The compiler subtracts only the
///   operand's regex. The subtraction would silently ignore any restrictions
///   on the operand.
fn check_restriction_targets<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    let lexical_rules: FxHashMap<&str, &LexicalRule> = grammar_def
        .lexical_rules
        .iter()
        .map(|rule| (rule.head.name.as_str(), rule))
        .collect();

    // Resolution assigns definition IDs to restriction operands on symbols.
    // Skip unresolved operands here because `check_unresolved_identifiers`
    // reports them.
    grammar_def.for_each_symbol(&mut |symbol| {
        let Symbol::Restricted { restrictions, .. } = symbol else {
            return;
        };
        for (kind, identifiers) in [
            (RestrictionKind::Except, &restrictions.excepts),
            (RestrictionKind::Follow, &restrictions.follow),
            (
                RestrictionKind::LayoutAwareFollow,
                &restrictions.layout_aware_follow,
            ),
            (RestrictionKind::Precede, &restrictions.precede),
        ] {
            for identifier in identifiers.iter().filter(|id| id.definition.is_some()) {
                check_restriction_target(kind, identifier, &lexical_rules, spans, errors);
            }
        }
    });

    // Resolution does not assign definition IDs to restriction operands on
    // lexical rules. Check only operands whose names belong to defined rules.
    // `check_unresolved_identifiers` reports missing names.
    let defined = defined_names(grammar_def);
    for rule in &grammar_def.lexical_rules {
        for (kind, identifier) in lexical_restrictions(rule) {
            if defined.contains(identifier.name.as_str()) {
                check_restriction_target(kind, identifier, &lexical_rules, spans, errors);
            }
        }
    }
}

/// Checks one restriction operand against the rule it names.
fn check_restriction_target<'a>(
    kind: RestrictionKind,
    identifier: &'a Identifier,
    lexical_rules: &FxHashMap<&str, &LexicalRule>,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    let span = spans.identifier(identifier).span;
    let Some(rule) = lexical_rules.get(identifier.name.as_str()) else {
        errors.push(GrammarError {
            message: format!(
                "{} `{}` must name a lexical rule, not a syntax rule",
                kind.name(),
                identifier.name
            ),
            span,
        });
        return;
    };
    let restricted = !rule.except.is_empty()
        || !rule.follow_restriction.is_empty()
        || rule.precede_restriction.is_some();
    if kind == RestrictionKind::Except && restricted {
        errors.push(GrammarError {
            message: format!(
                "except `{}` must name a lexical rule with no restrictions of its own",
                identifier.name
            ),
            span,
        });
    }
}

/// Reports a label nested directly inside another label (`outer:inner:A`).
///
/// One label is valid. Direct nesting gives the same symbol two labels, and
/// the generator uses only the outer one, so the inner label would silently
/// disappear.
fn check_one_label_per_symbol<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    grammar_def.for_each_symbol(&mut |symbol| {
        let Symbol::Labeled {
            label,
            symbol: inner,
        } = symbol
        else {
            return;
        };
        let Symbol::Labeled {
            label: inner_label, ..
        } = inner.as_ref()
        else {
            return;
        };
        errors.push(GrammarError {
            message: format!(
                "labels cannot be nested directly: `{label}:{inner_label}:` applies two labels to one symbol"
            ),
            span: spans.symbol(symbol).span,
        });
    });
}

/// Names of types and traits imported or defined by the generated parse tree.
///
/// Keep this list synchronized with generated-code type and trait names in
/// `generator/parse_tree_gen.rs` that are not derived from the grammar.
const RESERVED_TYPE_NAMES: &[&str] = &[
    "Arena",
    "DisplayOptions",
    "IntoIter",
    "ListNode",
    "NodeKind",
    "NonterminalId",
    "NonterminalNode",
    "OneOrMany",
    "OptNode",
    "Origin",
    "ParseTree",
    "ParseTreeBuilder",
    "ParseTreeNode",
    "SPPFNodeId",
    "SlotId",
    "Span",
    "Start",
    "TerminalId",
    "TerminalNode",
    "Token",
    "TokenKind",
];

/// Names of constants generated alongside upper-snake-case nonterminal IDs.
///
/// Keep this list synchronized with generated-code constant names in
/// `generator/grammar_data_gen.rs` that are not derived from the grammar.
const RESERVED_CONSTANT_NAMES: &[&str] = &[
    "NONTERMINALS",
    "NONTERMINAL_DISPLAY_ORDER",
    "SLOTS",
    "TERMINALS",
];

const AMBIGUITY_VARIANT_NAME: &str = "Amb";

const ALTERNATIVE_SPAN_FIELD_NAME: &str = "span";

/// Reports rule and label names that collide with generated Rust identifiers.
///
/// The check compares each name after applying the generator's case conversion.
/// Rule names become PascalCase types and UPPER_SNAKE_CASE constants.
/// Alternative labels become PascalCase variants, and symbol labels become
/// snake_case fields.
fn check_reserved_names<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    let layout_name = grammar_def
        .layout
        .as_ref()
        .and_then(Symbol::as_identifier)
        .map(|identifier| identifier.name.as_str());
    // Every syntax rule except the layout rule gets a `StartX` wrapper. Compare
    // constant names so different spellings that generate the same constant, such
    // as `start_s` and `StartS`, are also detected.
    let start_wrappers: FxHashMap<String, &str> = grammar_def
        .syntax_rules
        .iter()
        .map(|rule| rule.head.name.as_str())
        .filter(|name| Some(*name) != layout_name)
        .map(|name| (constant_name(&format!("Start{name}")), name))
        .collect();
    let parser_type = parser_ident(&grammar_def.name);
    let parse_tree_builder_type = parse_tree_builder_ident(&grammar_def.name);

    for rule in &grammar_def.syntax_rules {
        let name = rule.head.name.as_str();
        let span = spans.nonterminal(&rule.head).span;

        if let Some(wrapped) = start_wrappers.get(&constant_name(name)) {
            errors.push(GrammarError {
                message: format!(
                    "`{name}` collides with the generated start wrapper for `{wrapped}`"
                ),
                span,
            });
        }
        let type_name = to_pascal_case(name);
        if RESERVED_TYPE_NAMES.contains(&type_name.as_str())
            || parser_type == type_name
            || parse_tree_builder_type == type_name
        {
            errors.push(GrammarError {
                message: format!("`{name}` is a reserved name in the generated parse tree"),
                span,
            });
        }
        if RESERVED_CONSTANT_NAMES.contains(&constant_name(name).as_str()) {
            errors.push(GrammarError {
                message: format!("`{name}` is a reserved name in the generated grammar data"),
                span,
            });
        }

        let mut variant_names = FxHashMap::default();
        for alternative in rule.alternatives() {
            let Some(label) = &alternative.label else {
                continue;
            };
            let variant_name = to_pascal_case(label);
            let alternative_span = spans.alternative(alternative).span;
            if variant_name == AMBIGUITY_VARIANT_NAME {
                errors.push(GrammarError {
                    message: format!(
                        "`#{label}` becomes `{AMBIGUITY_VARIANT_NAME}`, which is reserved for ambiguous alternatives"
                    ),
                    span: alternative_span,
                });
            }
            // Reserve the whole `Alt<n>` family. Desugaring can add or reorder
            // alternatives. A source position therefore does not determine the
            // generated name of an unlabeled variant.
            if let Some(index) = variant_name.strip_prefix("Alt")
                && index.parse::<usize>().is_ok()
            {
                errors.push(GrammarError {
                    message: format!(
                        "`#{label}` becomes `{variant_name}`, which is reserved for unlabeled alternatives"
                    ),
                    span: alternative_span,
                });
            }
            if let Some(previous) = variant_names.insert(variant_name.clone(), label.as_str()) {
                errors.push(GrammarError {
                    message: format!(
                        "`#{previous}` and `#{label}` both become the generated variant `{variant_name}`"
                    ),
                    span: alternative_span,
                });
            }
        }
    }

    // A symbol label becomes a snake_case field name. An unlabeled rule reference
    // derives its field name from the referenced rule, but repeated names gain
    // numeric indexes. Whether an unlabeled field collides with `span` therefore
    // cannot be decided from the rule name alone.
    grammar_def.for_each_symbol(&mut |symbol| {
        if let Symbol::Labeled { label, .. } = symbol
            && to_snake_case(label) == ALTERNATIVE_SPAN_FIELD_NAME
        {
            errors.push(GrammarError {
                message: format!(
                    "`{label}` becomes the field `{ALTERNATIVE_SPAN_FIELD_NAME}`, which the generator uses for an alternative's span"
                ),
                span: spans.symbol(symbol).span,
            });
        }
    });
}

/// Converts a rule name to the upper snake case used for grammar-data constants.
fn constant_name(rule_name: &str) -> String {
    to_snake_case(rule_name).to_uppercase()
}

/// Reports a rule annotated both `@Layout` and `@Identifier`. Keyword
/// exactness derives word boundaries from the identifier rules, and layout is
/// what separates the words, so one rule cannot serve as both.
fn check_layout_is_not_an_identifier_rule(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    errors: &mut Vec<GrammarError>,
) {
    let Some(Symbol::Identifier(layout)) = &grammar_def.layout else {
        return;
    };
    for identifier in &grammar_def.identifier_rules {
        if identifier.name == layout.name {
            // `resolve` leaves the identifier rules unresolved, so the rule
            // is found by name. Every identifier rule is recorded from a
            // lexical rule, so the lookup cannot miss.
            let head = lexical_head(grammar_def, &identifier.name)
                .expect("an identifier rule names a lexical rule");
            errors.push(GrammarError {
                message: format!(
                    "`{}` cannot be both the layout rule and an identifier rule",
                    identifier.name
                ),
                span: spans.terminal(head).span,
            });
        }
    }
}

/// The head of the lexical rule named `name`.
fn lexical_head<'a>(grammar_def: &'a GrammarDef, name: &str) -> Option<&'a Terminal> {
    grammar_def
        .lexical_rules
        .iter()
        .find(|rule| rule.head.name == name)
        .map(|rule| &rule.head)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_spanned_error_at_a_one_based_line_and_column() {
        let source = r#"grammar G

S = Missing
"#;
        let start = source.find("Missing").unwrap() as u32;
        let errors = [GrammarError {
            message: "unresolved identifier `Missing`".to_string(),
            span: Span::new(start, start + "Missing".len() as u32),
        }];

        assert_eq!(
            render_errors(&errors, Path::new("grammar.iggy"), source),
            "grammar.iggy:3:5: unresolved identifier `Missing`"
        );
    }

    #[test]
    fn renders_multiple_errors_one_per_line() {
        let source = r#"grammar G

S = Foo Bar
"#;
        let foo = source.find("Foo").unwrap() as u32;
        let bar = source.find("Bar").unwrap() as u32;
        let errors = [
            GrammarError {
                message: "unresolved identifier `Foo`".to_string(),
                span: Span::new(foo, foo + 3),
            },
            GrammarError {
                message: "unresolved identifier `Bar`".to_string(),
                span: Span::new(bar, bar + 3),
            },
        ];

        assert_eq!(
            render_errors(&errors, Path::new("grammar.iggy"), source),
            r#"grammar.iggy:3:5: unresolved identifier `Foo`
grammar.iggy:3:9: unresolved identifier `Bar`"#
        );
    }
}
