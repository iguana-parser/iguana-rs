use std::iter;
use std::path::Path;

use iggy::parse_tree::{self, ParseTree, Start, TokenKind};
use iguana_runtime::input::{Input, Span};
use iguana_runtime::parse_tree::{TreeVisitor, Visit, walk};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::escapes::invalid_escapes;
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
    check_string_literals_are_not_empty(grammar_def, spans, &mut errors);
    check_ignore_case_literals_have_letters(grammar_def, spans, &mut errors);
    check_grammar_has_a_syntax_rule(grammar_def, &mut errors);
    errors
}

/// The code point of a `\u{...}` escape must have at most six hexadecimal
/// digits, and it must be a Unicode scalar value. A scalar value is at most
/// `10FFFF` and is not a surrogate (`D800` to `DFFF`). For example,
/// `"\u{D800}"` and `[\u{110000}]` are errors, because no character has these
/// code points.
///
/// The check runs on the parse tree, because the `GrammarDef` has a
/// placeholder in place of an invalid escape (`build_grammar`).
pub fn validate_escapes(
    tree: &Start<&parse_tree::Grammar<'_>, &parse_tree::Layout<'_>>,
    input: &Input,
) -> Vec<GrammarError> {
    let mut validator = EscapeValidator {
        input,
        errors: Vec::new(),
    };
    walk(tree.node.as_parse_tree(), &mut validator);
    validator.errors
}

/// Collects the errors of the invalid escapes in the tokens it visits
/// (`validate_escapes`).
struct EscapeValidator<'i> {
    input: &'i Input,
    errors: Vec<GrammarError>,
}

impl<'t> TreeVisitor<ParseTree<'t>> for EscapeValidator<'_> {
    fn enter(&mut self, node: ParseTree<'t>) -> Visit {
        let ParseTree::Token(token) = node else {
            return Visit::Children;
        };
        // String, Char, and RangeChar, the tokens that can contain an escape.
        // The escapes of a String or a Char are between the quotes.
        let span = token.span();
        let chars = self.input.chars(span);
        let (start, inner) = match token.kind {
            TokenKind::T3 | TokenKind::T4 => (span.left_extent + 1, &chars[1..chars.len() - 1]),
            TokenKind::T6 => (span.left_extent, chars),
            _ => return Visit::Skip,
        };
        for range in invalid_escapes(inner) {
            self.errors.push(GrammarError {
                message: "the code point of a `\\u{...}` escape must have at most six \
                          hexadecimal digits, be at most `10FFFF`, and not be a surrogate \
                          (`D800` to `DFFF`)"
                    .to_string(),
                span: Span::new(start + range.start as u32, start + range.end as u32),
            });
        }
        Visit::Skip
    }
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

/// The snake_case form of the grammar name must not be a Rust keyword, `std`,
/// or the name of a dependency of the generated crate, because it is the name
/// of the generated crate. For example, `grammar Std` results in a crate named
/// `std`, which would shadow the standard library.
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

/// A grammar must have at least one syntax rule. Without any syntax rules, the
/// generated crate does not compile.
fn check_grammar_has_a_syntax_rule(grammar_def: &GrammarDef, errors: &mut Vec<GrammarError>) {
    if grammar_def.syntax_rules.is_empty() {
        errors.push(GrammarError {
            message: format!("grammar `{}` has no syntax rules", grammar_def.name),
            // There is no syntax rule to report the error at, so it is
            // reported at the start of the file.
            span: Span::new(0, 0),
        });
    }
}

/// Rule names must be unique across all syntax and lexical rules.
///
/// Lexical rules are checked first, in the order definition IDs are assigned,
/// so a name used by both a lexical and a syntax rule is reported at the
/// syntax rule.
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

/// The names of two nonterminals must not have the same PascalCase form, such
/// as `LineComment` and `line_comment`, because the nonterminals would get the
/// same names in the generated parser. `LineComment`, for example, gets:
///
/// - the type `LineComment` and the method `ParseTree::unwrap_line_comment` in
///   `parse_tree.rs`,
/// - the nonterminal id constant `LINE_COMMENT` in `grammar.rs`,
/// - the entry point `parse_line_comment` in `parser.rs` and, if `LineComment`
///   is an LL(1) nonterminal, the methods `ll1_parse_line_comment` and
///   `ll1_body_line_comment`.
fn check_generated_rule_name_collisions<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    // A map from generated type names to source rule names.
    let mut type_names = FxHashMap::default();

    for rule in &grammar_def.syntax_rules {
        let name = rule.head.name.as_str();
        // Two names with different snake_case forms can have the same
        // PascalCase form (`to_pascal_case`), so this check compares the
        // PascalCase forms.
        let type_name = to_pascal_case(name);
        // Nonterminals with the same name in the grammar are reported by
        // `check_duplicate_definitions`.
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

/// Every reference to a rule must resolve to a rule in the grammar.
fn check_unresolved_identifiers<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    // Resolution assigns definition IDs to references in syntax rules and
    // lexical rule bodies.
    grammar_def.for_each_identifier(&mut |identifier| {
        if identifier.definition.is_none() {
            errors.push(GrammarError {
                message: format!("unresolved identifier `{}`", identifier.name),
                span: spans.identifier(identifier).span,
            });
        }
    });

    // Restrictions attached directly to lexical rules are resolved only after
    // validation, so this check compares their names with the defined rule
    // names.
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

/// An exclusion (`A!label`) must apply to a reference to a syntax rule. Each
/// exclusion label must be the label of an alternative of that rule.
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

/// - The operand of a restriction (`\`, `!>>`, `!>>>`, or `!<<`) must refer to
///   a lexical rule, because the scanner matches only lexical rules.
/// - The operand of an except (`\`) must refer to a lexical rule without
///   restrictions of its own, because an except uses only the regex of its
///   operand. For example, `Id \ Keyword` is an error when
///   `Keyword = "if" !>> Tail`.
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

/// Checks one restriction operand against the rule it refers to.
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

/// A symbol must have at most one label. For example, `outer:inner:A` is an
/// error, because the generator would use only the outer label, and the inner
/// one would silently disappear.
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

/// Two field names in an alternative must not have the same snake_case form,
/// because each becomes a field of the generated type. For example,
/// `x:"a" x:"b"` and `fooBar:A foo_bar:B` are errors.
///
/// Field names inside a group or a list do not conflict with field names
/// outside it, because the group or the list becomes a rule of its own. For
/// example, `x:A (x:B C)` is valid, but `(x:B x:C)` is an error.
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
    // Validation runs before EBNF-to-BNF conversion, because an error needs the
    // source span of its symbol and only the parsed grammar (GrammarDef) has
    // spans. Groups and lists are therefore still nested in their alternatives
    // here. An alternation element or an option body is a sequence of one
    // symbol, which cannot repeat a label.
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

/// The prefixes of the names of the nonterminals derived from EBNF symbols.
const EBNF_PREFIXES: &[&str] = &["Opt", "Star", "Plus", "Group", "Alt"];

/// Whether `name` can conflict with the name of a nonterminal derived from an
/// EBNF symbol, such as `Opt_0`. The check compares PascalCase forms, so
/// `Opt_0`, `opt_0`, and `Opt0` all conflict with `Opt_0`.
fn conflicts_with_ebnf_name(name: &str) -> bool {
    let pascal_name = to_pascal_case(name);
    EBNF_PREFIXES.iter().any(|prefix| {
        pascal_name
            .strip_prefix(prefix)
            .is_some_and(|number| number.parse::<usize>().is_ok())
    })
}

/// Rule names, field names, and alternative labels must not conflict with
/// identifiers in the generated code. Before the comparison, each of these
/// names is converted the way the generator converts it: to PascalCase when it
/// becomes a type or an enum variant, and to snake_case when it becomes a field
/// or a method. For example, a nonterminal named `TokenKind`, `token_kind`, or
/// `TOKEN_KIND` results in a type named `TokenKind`, which conflicts with the
/// type `TokenKind` in `parse_tree.rs`.
///
/// A rule name or a field name also must not become the field `self`, `super`,
/// or `crate`, which Rust does not accept even as a raw identifier.
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

    // The name of a syntax rule must not:
    // - have a reserved nonterminal name as its snake_case form
    //   (`RESERVED_NONTERMINAL_NAMES`).
    // - have the type of a start wrapper, such as `StartS`, or of the parse
    //   tree builder as its PascalCase form.
    // - conflict with the name of a nonterminal derived from an EBNF symbol,
    //   such as `Opt_0`, because both would become the type `Opt0`.
    // - have a reserved field name as its snake_case form
    //   (`RESERVED_FIELD_NAMES`).
    // - have `self`, `super`, or `crate` as its snake_case form
    //   (`UNESCAPABLE_FIELD_NAMES`).
    for rule in &grammar_def.syntax_rules {
        let name = rule.head.name.as_str();
        let span = spans.nonterminal(&rule.head).span;

        let type_name = to_pascal_case(name);
        let snake_name = to_snake_case(name);
        if RESERVED_NONTERMINAL_NAMES.contains(&snake_name.as_str())
            || start_wrappers.contains(&type_name)
            || type_name == parse_tree_builder
            || conflicts_with_ebnf_name(name)
        {
            errors.push(name_conflict(name, &type_name, span));
        } else if RESERVED_FIELD_NAMES.contains(&snake_name.as_str()) {
            errors.push(name_conflict(name, &snake_name, span));
        }
        if UNESCAPABLE_FIELD_NAMES.contains(&snake_name.as_str()) {
            errors.push(unescapable_field(name, &snake_name, span));
        }

        // The PascalCase form of an alternative label must not be one of these
        // names (a label becomes an enum variant, which is in PascalCase):
        // - `Self`, a Rust keyword that cannot be a raw identifier.
        // - `Amb`, the variant of ambiguous alternatives.
        // - `Cycle`, the variant of cyclic references.
        // - `Alt` followed by a number, the variants of unlabeled alternatives.
        // - the label of another alternative of the same rule, such as `#if`
        //   and `#If`.
        let mut variant_names = FxHashMap::default();
        for alternative in rule.alternatives() {
            let Some(label) = &alternative.label else {
                continue;
            };
            let variant_name = to_pascal_case(label);
            let alternative_span = spans.alternative(alternative).span;
            // `Self` is the only Rust keyword in PascalCase.
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

    // The name of a lexical rule must not:
    // - conflict with the name of a nonterminal derived from an EBNF symbol,
    //   such as `Opt_0`, because a reference to the lexical rule would refer to
    //   that nonterminal instead. The check compares PascalCase forms, so it
    //   also rejects `Opt0`, which does not conflict.
    // - have a reserved field name as its snake_case form
    //   (`RESERVED_FIELD_NAMES`).
    // - have `self`, `super`, or `crate` as its snake_case form
    //   (`UNESCAPABLE_FIELD_NAMES`).
    for rule in &grammar_def.lexical_rules {
        let name = rule.head.name.as_str();
        let span = spans.terminal(&rule.head).span;
        let field_name = to_snake_case(name);
        if conflicts_with_ebnf_name(name) {
            errors.push(name_conflict(name, name, span));
        } else if RESERVED_FIELD_NAMES.contains(&field_name.as_str()) {
            errors.push(name_conflict(name, &field_name, span));
        }
        if UNESCAPABLE_FIELD_NAMES.contains(&field_name.as_str()) {
            errors.push(unescapable_field(name, &field_name, span));
        }
    }

    // A field name must not:
    // - have a reserved field name as its snake_case form
    //   (`RESERVED_FIELD_NAMES`).
    // - have `self`, `super`, or `crate` as its snake_case form
    //   (`UNESCAPABLE_FIELD_NAMES`).
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

/// A grammar must have at most one layout rule (`@Layout`). Every layout rule
/// after the first one is an error.
fn check_duplicate_layout_rules(spans: &GrammarSpans<'_>, errors: &mut Vec<GrammarError>) {
    // Rule names are unique here, because `validate` returns early on a
    // duplicate definition.
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

/// A string literal must not be empty. For example, `""` and `i""` are
/// errors in both syntax and lexical rules. An optional part is written
/// `A?` instead of `A | ""`.
fn check_string_literals_are_not_empty<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    let mut report = |span| {
        errors.push(GrammarError {
            message: "a string literal must not be empty".to_string(),
            span,
        });
    };
    grammar_def.for_each_symbol(&mut |symbol| {
        if matches!(symbol, Symbol::Literal { text, .. } if text.is_empty()) {
            report(spans.symbol(symbol).span);
        }
    });
    spans.empty_strings.iter().copied().for_each(report);
}

/// An `i` literal of a syntax rule must have at least one ASCII letter. For
/// example, `i"+"` is an error, because the prefix changes only how ASCII
/// letters match, and `i"+"` would be a second terminal for the text of
/// `"+"`. An `i` literal of a lexical rule is not a terminal, so it is not
/// checked. An empty `i` literal is an error of
/// `check_string_literals_are_not_empty`.
///
/// An `i` literal with U+FFFD is not checked, because U+FFFD is the
/// placeholder of an invalid escape (`build_grammar`). The escape can be a
/// mistyped letter, such as `\u{0000041}` for `A`, and `validate_escapes`
/// reports it. An `i` literal with a U+FFFD of its own and no letter is
/// therefore not reported.
fn check_ignore_case_literals_have_letters<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    errors: &mut Vec<GrammarError>,
) {
    grammar_def.for_each_symbol(&mut |symbol| {
        let Symbol::Literal {
            text,
            ignore_ascii_case: true,
        } = symbol
        else {
            return;
        };
        if !text.is_empty()
            && !text.contains(char::REPLACEMENT_CHARACTER)
            && !text.contains(|c: char| c.is_ascii_alphabetic())
        {
            errors.push(GrammarError {
                message: "a literal with the `i` prefix must have at least one ASCII letter"
                    .to_string(),
                span: spans.symbol(symbol).span,
            });
        }
    });
}

/// A rule must not be both the layout rule (`@Layout`) and an identifier rule
/// (`@Identifier`). Keyword exactness derives word boundaries from the
/// identifier rules, and layout is what separates the words, so one rule
/// cannot serve as both.
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
    fn rules_named_like_ebnf_types_are_rejected() {
        let source = r#"grammar G

S = Opt0 Star_0 alt12 Optional Alt Plus_1 Group3
Opt0 = "a"
Star_0 = "b"
alt12 = "c"
Optional = "d"
Alt = "e"
@Regex
Plus_1 = [p]
@Regex
Group3 = [g]
"#;
        let errors = parse_grammar(source).unwrap_err();
        let messages: Vec<&str> = errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "`Opt0` conflicts with a name in the generated code",
                "`Star_0` conflicts with `Star0` in the generated code",
                "`alt12` conflicts with `Alt12` in the generated code",
                "`Plus_1` conflicts with a name in the generated code",
                "`Group3` conflicts with a name in the generated code",
            ]
        );
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
    fn an_ignore_case_literal_without_a_letter_is_rejected() {
        let source = "grammar G\n\nS = i\"+\" i\"a1\" T\n\n@Regex\nT = i\"\\n\" | i\"_x\"\n";
        let errors = parse_grammar(source).unwrap_err();
        let spans: Vec<String> = errors
            .iter()
            .map(|error| {
                let span = error.span;
                source[span.left_extent as usize..span.right_extent as usize].to_string()
            })
            .collect();
        assert_eq!(spans, ["i\"+\""]);
        assert_eq!(
            errors[0].message,
            "a literal with the `i` prefix must have at least one ASCII letter"
        );
    }

    #[test]
    fn an_empty_string_literal_is_rejected() {
        let source =
            "grammar G\n\nS = \"\" i\"\" \"a\" T\n\n@Regex\nT = \"\" \"b\" | (i\"\" \"c\")+\n";
        let errors = parse_grammar(source).unwrap_err();
        let spans: Vec<String> = errors
            .iter()
            .map(|error| {
                let span = error.span;
                source[span.left_extent as usize..span.right_extent as usize].to_string()
            })
            .collect();
        assert_eq!(spans, ["\"\"", "i\"\"", "\"\"", "i\"\""]);
        for error in &errors {
            assert_eq!(error.message, "a string literal must not be empty");
        }
    }

    #[test]
    fn a_rule_cannot_be_named_i() {
        assert!(parse_grammar("grammar G\n\nS = i\n\ni = \"a\"\n").is_err());
        assert!(parse_grammar("grammar G\n\nS = i \"a\"\n").is_err());
        assert!(parse_grammar("grammar G\n\nS = x\"a\"\n\nx = \"b\"\n").is_ok());
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
