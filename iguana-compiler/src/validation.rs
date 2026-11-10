use std::iter;
use std::path::Path;

use iguana_runtime::input::{Input, Span};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::generator::grammar_utils::parse_tree_builder_ident;
use crate::generator::utils::is_rust_keyword;
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

/// Checks a resolved grammar for semantic errors in rule definitions, symbol
/// references, and annotations, reporting each error at its source location.
///
/// Duplicate rule names stop the remaining checks because those checks assume
/// unique names.
pub fn validate<'a>(grammar_def: &'a GrammarDef, spans: &GrammarSpans<'a>) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    check_grammar_name(grammar_def, spans, &mut errors);
    check_duplicate_definitions(grammar_def, spans, &mut errors);
    if !errors.is_empty() {
        return errors;
    }
    check_generated_rule_name_collisions(grammar_def, spans, &mut errors);
    check_unresolved_identifiers(grammar_def, spans, &mut errors);
    check_exclusions(grammar_def, spans, &mut errors);
    check_restriction_targets(grammar_def, spans, &mut errors);
    check_one_label_per_symbol(grammar_def, spans, &mut errors);
    check_duplicate_field_labels(grammar_def, spans, &mut errors);
    check_reserved_names(grammar_def, spans, &mut errors);
    check_duplicate_layout_rules(spans, &mut errors);
    check_layout_is_not_an_identifier_rule(grammar_def, spans, &mut errors);
    check_grammar_has_a_syntax_rule(grammar_def, &mut errors);
    errors
}

/// The dependencies of every generated crate shape, as crate names. Keep in
/// sync with the `[dependencies]` sections written by `cargo_toml_gen`.
const GENERATED_CRATE_DEPENDENCIES: &[&str] = &["iguana_runtime", "rustc_hash", "serde_json"];

/// Returns why a grammar named `grammar_name` cannot generate a crate, or
/// `None` when it can. The generated crate takes the snake_case form of the
/// grammar name, and the scaffold imports the crate by that name. A Rust
/// keyword cannot be imported, `std` shadows the standard library, and a
/// dependency's name makes the import ambiguous.
pub fn crate_name_error(grammar_name: &str) -> Option<String> {
    let crate_name = to_snake_case(grammar_name);
    let reason = if is_rust_keyword(&crate_name) {
        "is a Rust keyword"
    } else if crate_name == "std" {
        "shadows the standard library"
    } else if GENERATED_CRATE_DEPENDENCIES.contains(&crate_name.as_str()) {
        "is a dependency of the generated crate"
    } else {
        return None;
    };
    Some(format!(
        "the generated crate would be named `{crate_name}`, which {reason}"
    ))
}

/// The grammar name parses as an identifier but also names the generated
/// crate, which has its own restrictions. The error points at the header name.
fn check_grammar_name(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    errors: &mut Vec<GrammarError>,
) {
    if let Some(reason) = crate_name_error(&grammar_def.name) {
        errors.push(GrammarError {
            message: format!(
                "`{}` cannot be the grammar name: {reason}",
                grammar_def.name
            ),
            span: spans.grammar_name,
        });
    }
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

/// Checks that syntax and lexical rule names are unique. Each repeated
/// definition produces an error at its rule name.
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

/// Reports nonterminals whose names collide in the generated parser, such as
/// `LineComment` and `line_comment`.
///
/// Each nonterminal produces these names, shown for `LineComment`:
///
/// - the type `LineComment` and the method `ParseTree::unwrap_line_comment` in
///   `parse_tree.rs`,
/// - the nonterminal id constant `LINE_COMMENT` in `grammar.rs`,
/// - the entry point `parse_line_comment` in `parser.rs` and, if `LineComment`
///   is an LL(1) nonterminal, the methods `ll1_parse_line_comment` and
///   `ll1_body_line_comment`.
///
/// Two names with different snake_case forms can have the same PascalCase form
/// (`to_pascal_case`), so this check compares the PascalCase forms.
///
/// Nonterminals with the same name in the grammar are reported by
/// `check_duplicate_definitions`.
fn check_generated_rule_name_collisions<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    // A map from generated type names to source rule names.
    let mut type_names = FxHashMap::default();

    for rule in &grammar_def.syntax_rules {
        let name = rule.head.name.as_str();
        let type_name = to_pascal_case(name);
        if let Some(previous) = type_names.insert(type_name.clone(), name)
            && previous != name
        {
            errors.push(GrammarError {
                message: format!(
                    "`{previous}` and `{name}` have the same generated name `{type_name}`"
                ),
                span: spans.nonterminal(&rule.head).span,
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

        // An exclusion wraps the reference directly. A field label or a
        // restriction wraps the exclusion instead.
        let Symbol::Identifier(identifier) = inner.as_ref() else {
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

/// Reports two labeled symbols in an alternative whose labels become the same
/// field name, such as `x:"a" x:"b"` or `fooBar:A foo_bar:B`.
///
/// Validation runs before EBNF-to-BNF conversion, because an error needs the
/// source span of its symbol and only the parsed grammar (GrammarDef) has
/// spans. At this point a parenthesized group or a list is still a symbol
/// nested inside its enclosing alternative, and EBNF-to-BNF later turns it
/// into a rule of its own. The check therefore also visits every nested
/// sequence, the symbols of a group or a list element together with its
/// separator, and compares each sequence's labels on their own, as they will
/// be fields of one alternative in the generated parser.
fn check_duplicate_field_labels<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    for rule in &grammar_def.syntax_rules {
        for alternative in rule.alternatives() {
            check_sequence_field_labels(alternative.symbols.iter(), spans, errors);
        }
    }
    // An alternation element or an option body is a sequence of one symbol,
    // which cannot repeat a label.
    grammar_def.for_each_symbol(&mut |symbol| match symbol {
        Symbol::Group(symbols) => check_sequence_field_labels(symbols.iter(), spans, errors),
        Symbol::Star(element, separator) | Symbol::Plus(element, separator) => {
            check_sequence_field_labels(
                iter::once(element.as_ref()).chain(separator.as_deref()),
                spans,
                errors,
            );
        }
        _ => {}
    });
}

/// Compares the labels of the symbols of one sequence. A label is the
/// outermost wrapper of a parsed symbol, so the label of the symbol itself is
/// the one the generator names the field after.
fn check_sequence_field_labels<'a>(
    symbols: impl Iterator<Item = &'a Symbol>,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    // A map from generated field names to source labels.
    let mut field_names: FxHashMap<String, &str> = FxHashMap::default();
    for symbol in symbols {
        let Some(label) = symbol.label() else {
            continue;
        };
        let field_name = to_snake_case(label);
        if let Some(previous) = field_names.insert(field_name.clone(), label) {
            let message = if previous == label {
                format!("duplicate label `{label}`")
            } else {
                format!("`{previous}` and `{label}` both become the field `{field_name}`")
            };
            errors.push(GrammarError {
                message,
                span: spans.symbol(symbol).span,
            });
        }
    }
}

/// Names that nonterminals cannot use because they would conflict with types
/// in `parse_tree.rs`:
///
/// - `ParseTree`, `Token`, `TokenKind`, and `OptNode`, defined there,
/// - `Start`, re-exported there,
/// - `SlotId`, imported there,
/// - `Option`, `Vec`, and `Iterator`, the prelude types the generated code uses.
///
/// Other runtime items are accessed through their modules, so they do not need
/// to be reserved. The runtime types `Span` and `Origin` are covered by
/// `RESERVED_FIELD_NAMES`, which applies to every rule.
const RESERVED_NONTERMINAL_NAMES: &[&str] = &[
    "iterator",
    "opt_node",
    "option",
    "parse_tree",
    "slot_id",
    "start",
    "token",
    "token_kind",
    "vec",
];

/// Names that nonterminals, lexical rules, and labels cannot use because they
/// would conflict with the methods every parse tree type has. A reference to a
/// rule becomes a field named after the rule, or after its label if it has
/// one. The type of a rule with a single alternative gets an accessor method
/// for each field. `span` also conflicts with the span field of every
/// alternative.
const RESERVED_FIELD_NAMES: &[&str] = &[
    "span",
    "display_name",
    "origin",
    "child",
    "child_count",
    "as_parse_tree",
];

const AMBIGUITY_VARIANT_NAME: &str = "Amb";

/// The Rust keywords that cannot be escaped as raw identifiers. Every other
/// keyword becomes a raw identifier when it is a field name, such as `r#type`
/// for `type:A`.
const UNESCAPABLE_FIELD_NAMES: &[&str] = &["self", "super", "crate"];

/// Reports rule and label names that collide with generated Rust identifiers
/// or cannot be Rust identifiers.
///
/// The check compares each name after applying the generator's case conversion:
///
/// - Reserved names are compared in snake_case. `TokenKind`, `token_kind`, and
///   `TOKEN_KIND` all conflict with `token_kind`.
/// - The start wrappers and the parse tree builder are types, compared in
///   PascalCase. `start_s` conflicts with `StartS`, the start wrapper of `S`.
/// - Alternative labels become PascalCase variants.
fn check_reserved_names<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    // Every syntax rule `X` gets a start wrapper `StartX`.
    let start_wrappers: FxHashSet<String> = grammar_def
        .syntax_rules
        .iter()
        .map(|rule| to_pascal_case(&format!("Start{}", rule.head.name)))
        .collect();
    let parse_tree_builder = parse_tree_builder_ident(&grammar_def.name).to_string();

    for rule in &grammar_def.syntax_rules {
        let name = rule.head.name.as_str();
        let span = spans.nonterminal(&rule.head).span;

        let type_name = to_pascal_case(name);
        let snake_name = to_snake_case(name);
        if RESERVED_NONTERMINAL_NAMES.contains(&snake_name.as_str())
            || start_wrappers.contains(&type_name)
            || type_name == parse_tree_builder
        {
            errors.push(name_conflict(name, &type_name, span));
        } else if RESERVED_FIELD_NAMES.contains(&snake_name.as_str()) {
            errors.push(name_conflict(name, &snake_name, span));
        }
        if UNESCAPABLE_FIELD_NAMES.contains(&snake_name.as_str()) {
            errors.push(unescapable_field(name, &snake_name, span));
        }

        let mut variant_names = FxHashMap::default();
        for alternative in rule.alternatives() {
            let Some(label) = &alternative.label else {
                continue;
            };
            let variant_name = to_pascal_case(label);
            let alternative_span = spans.alternative(alternative).span;
            // Alternative labels become PascalCase Rust enum variants:
            // `#type` becomes `Type`, and `#match` becomes `Match`. Rust keywords
            // are case-sensitive, so those variant names are valid. However,
            // `#self` and `#Self` both become `Self`, which is a Rust keyword
            // meaning the current type. Rust lets us escape most keywords with
            // `r#`, as in `r#type`, but explicitly forbids `r#Self`. We therefore
            // reject labels that become `Self`.
            if is_rust_keyword(&variant_name) {
                errors.push(GrammarError {
                    message: format!(
                        "`#{label}` becomes the variant `{variant_name}`, which is a Rust keyword"
                    ),
                    span: alternative_span,
                });
            }
            if variant_name == AMBIGUITY_VARIANT_NAME {
                errors.push(GrammarError {
                    message: format!(
                        "`#{label}` becomes `{AMBIGUITY_VARIANT_NAME}`, which is reserved for ambiguous alternatives"
                    ),
                    span: alternative_span,
                });
            }
            if variant_name == "Cycle" {
                errors.push(GrammarError {
                    message: format!(
                        "`#{label}` becomes `Cycle`, which is reserved for cyclic references"
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

    // A reference to a lexical rule becomes a token field named after the rule.
    for rule in &grammar_def.lexical_rules {
        let name = rule.head.name.as_str();
        let span = spans.terminal(&rule.head).span;
        let field_name = to_snake_case(name);
        if RESERVED_FIELD_NAMES.contains(&field_name.as_str()) {
            errors.push(name_conflict(name, &field_name, span));
        }
        if UNESCAPABLE_FIELD_NAMES.contains(&field_name.as_str()) {
            errors.push(unescapable_field(name, &field_name, span));
        }
    }

    // A symbol label becomes a snake_case field name.
    grammar_def.for_each_symbol(&mut |symbol| {
        let Symbol::Labeled { label, .. } = symbol else {
            return;
        };
        let span = spans.symbol(symbol).span;
        let field_name = to_snake_case(label);
        if RESERVED_FIELD_NAMES.contains(&field_name.as_str()) {
            errors.push(name_conflict(label, &field_name, span));
        }
        if UNESCAPABLE_FIELD_NAMES.contains(&field_name.as_str()) {
            errors.push(unescapable_field(label, &field_name, span));
        }
    });
}

/// The error for a rule or label `name` that becomes `generated`, a name the
/// generated code already uses. The message names `generated` only when it is
/// spelled differently from `name`.
fn name_conflict(name: &str, generated: &str, span: Span) -> GrammarError {
    let message = if generated == name {
        format!("`{name}` conflicts with a name in the generated code")
    } else {
        format!("`{name}` conflicts with `{generated}` in the generated code")
    };
    GrammarError { message, span }
}

/// The error for a rule or label `name` that becomes the field `field_name`,
/// which Rust does not accept even as a raw identifier.
fn unescapable_field(name: &str, field_name: &str, span: Span) -> GrammarError {
    GrammarError {
        message: format!(
            "`{name}` becomes the field `{field_name}`, which cannot be a Rust field name"
        ),
        span,
    }
}

/// Reports every rule after the first marked `@Layout`, at its rule head,
/// naming the first in source order. A grammar has at most one layout rule.
///
/// Rule names are unique here, because `validate` returns early on a duplicate
/// definition.
fn check_duplicate_layout_rules(spans: &GrammarSpans<'_>, errors: &mut Vec<GrammarError>) {
    let Some(((first, _), rest)) = spans.layout_rules.split_first() else {
        return;
    };
    for (name, span) in rest {
        errors.push(GrammarError {
            message: format!(
                "`{name}` is marked `@Layout`, but `{first}` is already the layout rule"
            ),
            span: *span,
        });
    }
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

    use crate::iggy::parse_grammar;

    #[test]
    fn only_the_parse_tree_builder_type_is_rejected_as_a_rule_name() {
        let source = r#"grammar Foo

S = FooGrammar FooParser FooParseTreeBuilder
FooGrammar = "a"
FooParser = "b"
FooParseTreeBuilder = "c"
"#;
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            ["`FooParseTreeBuilder` conflicts with a name in the generated code"]
        );
    }

    #[test]
    fn a_grammar_name_that_snake_cases_to_a_rust_keyword_is_rejected() {
        let source = "grammar Match\n\nS = \"a\"\n";
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "`Match` cannot be the grammar name: the generated crate would be named `match`, which is a Rust keyword"
            ]
        );
        let start = source.find("Match").unwrap() as u32;
        assert_eq!(
            errors[0].span,
            Span::new(start, start + "Match".len() as u32)
        );
    }

    #[test]
    fn crate_name_error_names_the_reserved_crate_names() {
        for name in ["Calculator", "MyMatch", "Core", "Alloc", "Test", "Build"] {
            assert_eq!(crate_name_error(name), None, "{name}");
        }
        assert_eq!(
            crate_name_error("Std").as_deref(),
            Some("the generated crate would be named `std`, which shadows the standard library")
        );
        assert_eq!(
            crate_name_error("RustcHash").as_deref(),
            Some(
                "the generated crate would be named `rustc_hash`, which is a dependency of the generated crate"
            )
        );
    }

    #[test]
    fn rules_named_like_prelude_types_are_rejected() {
        let source = r#"grammar G

S = Option Vec Iterator
Option = "a"
Vec = "b"
Iterator = "c"
"#;
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "`Option` conflicts with a name in the generated code",
                "`Vec` conflicts with a name in the generated code",
                "`Iterator` conflicts with a name in the generated code",
            ]
        );
    }

    #[test]
    fn rules_and_labels_named_like_node_methods_are_rejected() {
        let source = r#"grammar G

S = Span DisplayName AsParseTree Origin child:"a" child_count:"b"
Span = "c"
DisplayName = "d"
AsParseTree = "e"
@Regex
Origin = [a-z]+
"#;
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "`Span` conflicts with `span` in the generated code",
                "`DisplayName` conflicts with `display_name` in the generated code",
                "`AsParseTree` conflicts with `as_parse_tree` in the generated code",
                "`Origin` conflicts with `origin` in the generated code",
                "`child` conflicts with a name in the generated code",
                "`child_count` conflicts with a name in the generated code",
            ]
        );
    }

    #[test]
    fn a_reserved_name_conflicts_in_every_spelling() {
        for (name, expected) in [
            (
                "OptNode",
                "`OptNode` conflicts with a name in the generated code",
            ),
            (
                "opt_node",
                "`opt_node` conflicts with `OptNode` in the generated code",
            ),
            (
                "optNode",
                "`optNode` conflicts with `OptNode` in the generated code",
            ),
            (
                "Opt_Node",
                "`Opt_Node` conflicts with `OptNode` in the generated code",
            ),
            (
                "OPT_NODE",
                "`OPT_NODE` conflicts with `OptNode` in the generated code",
            ),
            (
                "START",
                "`START` conflicts with `Start` in the generated code",
            ),
            (
                "Span_",
                "`Span_` conflicts with `span` in the generated code",
            ),
        ] {
            let source = format!("grammar G\n\nS = {name}\n{name} = \"a\"\n");
            let errors = parse_grammar(&source).unwrap_err();
            let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
            assert_eq!(messages, [expected]);
        }
    }

    #[test]
    fn names_that_do_not_conflict_are_accepted() {
        // `StartFOOBAR` becomes the type `StartFoobar`, and the start wrapper
        // of `FOO_BAR` becomes `StartFooBar`.
        let source = r#"grammar G

S = Spans ALl1 BLl1Body StartWithoutLayout ChildAt:"a" StartFOOBAR FOO_BAR
Spans = "b"
ALl1 = "c"
BLl1Body = "d"
StartWithoutLayout = "e"
StartFOOBAR = "f"
FOO_BAR = "g"
"#;
        let result = parse_grammar(source);
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn rules_that_become_the_same_type_are_rejected() {
        let source = "grammar G\n\nS = A_1 A1\nA_1 = \"a\"\nA1 = \"b\"\n";
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            ["`A_1` and `A1` have the same generated name `A1`"]
        );

        // The start wrapper of `X_1` is `StartX1`.
        let source = "grammar G\n\nS = X_1 StartX1\nX_1 = \"a\"\nStartX1 = \"b\"\n";
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            ["`StartX1` conflicts with a name in the generated code"]
        );
    }

    #[test]
    fn a_rule_named_self_is_rejected() {
        let source = "grammar G\n\nS = Self\nSelf = \"a\"\n";
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            ["`Self` becomes the field `self`, which cannot be a Rust field name"]
        );
    }

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
