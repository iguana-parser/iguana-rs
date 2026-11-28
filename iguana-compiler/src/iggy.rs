use iggy::{
    IggyParser, ParseError, parse_tree,
    parse_tree::{OptNode, Start},
};
use iguana_runtime::{arena::Arena, input::Input};

use crate::escapes::{decode_next, unescape};
use crate::grammar::{
    def::{
        Alternative, Associativity, GrammarDef, LayoutStrategy, LexicalRule, PriorityLevel,
        SyntaxRule,
    },
    regex::{CharClass, CharRange, Regex},
    symbols::{
        Identifier, Literal, Nonterminal, RestrictionOperand, Restrictions, Symbol, Terminal,
    },
};
use crate::spans;
use crate::validation::{GrammarError, validate, validate_escapes};

/// Parses a grammar and returns a validated `GrammarDef` with its rule
/// references resolved.
pub fn parse_grammar(source: &str) -> Result<GrammarDef, Vec<GrammarError>> {
    let input = Input::from(source);
    let parser_arena = Arena::new();
    let tree_arena = Arena::new();
    let success = IggyParser::new(&input, &parser_arena)
        .parse_grammar(&tree_arena)
        .map_err(parse_error)?;
    // The escapes are checked on the parse tree, because the grammar has a
    // placeholder for an invalid escape (`build_grammar`).
    let mut errors = validate_escapes(&success.tree, &input);
    let grammar_def = build_grammar(&success.tree, &input).resolve();

    // Validate here because source spans require the parse tree.
    let spans = spans::build_spans(&grammar_def, &success.tree, &input);
    errors.extend(validate(&grammar_def, &spans));
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(grammar_def)
}

/// A parse failure as a grammar error.
fn parse_error(error: ParseError) -> Vec<GrammarError> {
    vec![GrammarError {
        message: error.message,
        span: error.span,
    }]
}

/// Builds the `GrammarDef` of a parse tree.
///
/// An invalid `\u{...}` escape, such as `\u{D800}`, has no character, and the
/// `GrammarDef` has U+FFFD, the replacement character, in its place. The
/// placeholder lets validation check the rest of the grammar, so the errors of
/// the invalid escapes (`validate_escapes`) and of the other checks are
/// reported together. `parse_grammar` rejects a grammar with an invalid escape,
/// so no parser is generated from it.
pub fn build_grammar(
    start_grammar: &Start<&parse_tree::Grammar<'_>, &parse_tree::Layout<'_>>,
    input: &Input,
) -> GrammarDef {
    let grammar = &start_grammar.node;
    let name = input.text(grammar.name().span());
    // With `@IgnoreCase`, a string literal without a prefix ignores case,
    // like a literal with the `i` prefix. A character literal in a lexical
    // rule, such as `'e'`, ignores case too.
    let ignore_case = grammar.ignore_case().value().is_some();

    let mut syntax_rules: Vec<SyntaxRule> = Vec::new();
    let mut lexical_rules: Vec<LexicalRule> = Vec::new();
    let mut layout_name: Option<String> = None;
    let mut identifier_rules: Vec<Identifier> = Vec::new();

    for rule in grammar.rules().rules() {
        match rule {
            parse_tree::Rule::SyntaxRule { syntax_rule, .. } => {
                let converted = convert_syntax_rule(syntax_rule, input, ignore_case);
                if has_layout_annotation(syntax_rule) {
                    layout_name.get_or_insert_with(|| converted.head.name.clone());
                }
                syntax_rules.push(converted);
            }
            parse_tree::Rule::RegexRule { regex_rule, .. } => {
                let converted = convert_regex_rule(regex_rule, input, ignore_case);
                if regex_rule.layout().value().is_some() {
                    layout_name.get_or_insert_with(|| converted.head.name.clone());
                }
                if regex_rule.id_annot().value().is_some() {
                    identifier_rules.push(Identifier {
                        name: converted.head.name.clone(),
                        definition: None,
                    });
                }
                lexical_rules.push(converted);
            }
            parse_tree::Rule::Amb(_) => panic!("unexpected ambiguity"),
        }
    }

    // The grammar's layout is the first rule marked `@Layout`.
    // `validate` rejects a grammar that marks more than one.
    let layout = layout_name.map(|name| {
        Symbol::Identifier(Identifier {
            name,
            definition: None,
        })
    });

    // The layout rule defines the layout context, so it must not have layout
    // inserted between its own symbols.
    if let Some(Symbol::Identifier(layout_id)) = &layout {
        for rule in &mut syntax_rules {
            if rule.head.name == layout_id.name {
                rule.layout = LayoutStrategy::None;
            }
        }
    }

    GrammarDef {
        name,
        syntax_rules,
        lexical_rules,
        layout,
        identifier_rules,
    }
}

/// Whether a syntax rule has the `@Layout` annotation, marking it as the
/// grammar's layout rule.
pub(crate) fn has_layout_annotation(rule: &parse_tree::SyntaxRule) -> bool {
    matches!(
        rule.annotation().value(),
        Some(parse_tree::Annotation::Layout { .. })
    )
}

fn convert_syntax_rule(
    rule: &parse_tree::SyntaxRule,
    input: &Input,
    ignore_case: bool,
) -> SyntaxRule {
    let head_name = input.text(rule.head().span());
    let head = Nonterminal::new(head_name);

    let priority_levels: Vec<PriorityLevel> = rule
        .priority_levels()
        .priority_levels()
        .map(|level| convert_priority_level(level, input, ignore_case))
        .collect();

    let mut layout = LayoutStrategy::Default;

    if let Some(annotation) = rule.annotation().value() {
        match annotation {
            parse_tree::Annotation::NoLayout { .. } => layout = LayoutStrategy::None,
            // `@Layout` marks the grammar's layout rule. build_grammar records
            // its name and suppresses layout inside it, so nothing to do here.
            parse_tree::Annotation::Layout { .. } => {}
            parse_tree::Annotation::Amb(_) => panic!("unexpected ambiguity"),
        }
    }

    SyntaxRule {
        head,
        priority_levels,
        layout,
    }
}

fn convert_priority_level(
    level: &parse_tree::PriorityLevel,
    input: &Input,
    ignore_case: bool,
) -> PriorityLevel {
    let alternatives: Vec<Alternative> = level
        .alternatives()
        .alternatives()
        .map(|alt| convert_alternative(alt, input, ignore_case))
        .collect();

    let associativity =
        level
            .associativity()
            .value()
            .map(
                |assoc: &parse_tree::Associativity<'_>| match input.text(assoc.span()).as_str() {
                    "left" => Associativity::Left,
                    "right" => Associativity::Right,
                    "none" => Associativity::NonAssoc,
                    other => panic!("Unknown associativity: {other}"),
                },
            );

    PriorityLevel::with_associativity(alternatives, associativity)
}

fn convert_alternative(
    alt: &parse_tree::Alternative,
    input: &Input,
    ignore_case: bool,
) -> Alternative {
    let (symbols, label): (Vec<Symbol>, _) = match alt {
        parse_tree::Alternative::Symbols { symbols, label, .. } => (
            symbols
                .symbols()
                .map(|sym| convert_symbol(sym, input, ignore_case))
                .collect(),
            label,
        ),
        parse_tree::Alternative::Empty { label, .. } => (Vec::new(), label),
        parse_tree::Alternative::Amb(_) => panic!("unexpected ambiguity"),
    };

    // Extract label, stripping the # prefix
    let label = label.value().map(|token| {
        let label_text = input.text(token.span());
        label_text.trim_prefix('#').to_string()
    });

    Alternative { symbols, label }
}

fn convert_symbol(symbol: &parse_tree::Symbol, input: &Input, ignore_case: bool) -> Symbol {
    match symbol {
        parse_tree::Symbol::Star { symbol, .. } => {
            Symbol::Star(Box::new(convert_symbol(symbol, input, ignore_case)), None)
        }
        parse_tree::Symbol::Plus { symbol, .. } => {
            Symbol::Plus(Box::new(convert_symbol(symbol, input, ignore_case)), None)
        }
        parse_tree::Symbol::Opt { symbol, .. } => {
            Symbol::Opt(Box::new(convert_symbol(symbol, input, ignore_case)))
        }
        // #Paren is the parenthesized form: at the syntax level one shape,
        // a list of symbol sequences separated by "|", covers both groups
        // and alternations. One sequence constructs a Group. Several
        // construct an Alt: a one-symbol sequence stays as is, and a longer
        // one is wrapped in a Group, because Alt holds a list of symbols,
        // not a list of sequences.
        parse_tree::Symbol::Paren { seqs, .. } => {
            let mut seqs: Vec<Vec<Symbol>> = seqs
                .symbols()
                .map(|seq| seq.map(|s| convert_symbol(s, input, ignore_case)).collect())
                .collect();
            if seqs.len() == 1 {
                Symbol::Group(seqs.pop().unwrap())
            } else {
                Symbol::Alt(
                    seqs.into_iter()
                        .map(|mut seq| {
                            if seq.len() == 1 {
                                seq.pop().unwrap()
                            } else {
                                Symbol::Group(seq)
                            }
                        })
                        .collect(),
                )
            }
        }
        parse_tree::Symbol::Literal { literal, .. } => {
            Symbol::Literal(convert_literal(literal, input, ignore_case))
        }
        parse_tree::Symbol::StarSep { symbol, sep, .. } => Symbol::Star(
            Box::new(convert_symbol(symbol, input, ignore_case)),
            Some(Box::new(convert_symbol(sep, input, ignore_case))),
        ),
        parse_tree::Symbol::PlusSep { symbol, sep, .. } => Symbol::Plus(
            Box::new(convert_symbol(symbol, input, ignore_case)),
            Some(Box::new(convert_symbol(sep, input, ignore_case))),
        ),
        parse_tree::Symbol::Labeled { label, symbol, .. } => Symbol::Labeled {
            label: input.text(label.span()),
            symbol: Box::new(convert_symbol(symbol, input, ignore_case)),
        },
        parse_tree::Symbol::Identifier { identifier, .. } => Symbol::Identifier(Identifier {
            name: input.text(identifier.span()),
            definition: None,
        }),
        // `PostCondition` has higher precedence than `PreCondition`, so a
        // post-condition node can nest inside a pre-condition node. The two
        // cases are therefore converted together, building one
        // `Restrictions` record for both.
        parse_tree::Symbol::PostCondition { .. } | parse_tree::Symbol::PreCondition { .. } => {
            let mut restrictions = Restrictions::default();
            let mut node = symbol;
            if let parse_tree::Symbol::PreCondition {
                conditions, symbol, ..
            } = node
            {
                for condition in conditions.pre_conditions() {
                    let operand = convert_operand(condition.operand(), input, ignore_case);
                    restrictions.precede.push(operand);
                }
                node = symbol;
            }
            let mut labels = Vec::new();
            if let parse_tree::Symbol::PostCondition {
                conditions, symbol, ..
            } = node
            {
                for condition in conditions.post_conditions() {
                    let (list, operand) = match condition {
                        parse_tree::PostCondition::Except { operand, .. } => {
                            (&mut restrictions.excepts, operand)
                        }
                        parse_tree::PostCondition::Require { operand, .. } => {
                            (&mut restrictions.required, operand)
                        }
                        parse_tree::PostCondition::FollowRestriction { operand, .. } => {
                            (&mut restrictions.follow, operand)
                        }
                        parse_tree::PostCondition::LayoutAwareFollowRestriction {
                            operand, ..
                        } => (&mut restrictions.layout_aware_follow, operand),
                        parse_tree::PostCondition::Exclude { identifier, .. } => {
                            labels.push(input.text(identifier.span()));
                            continue;
                        }
                        parse_tree::PostCondition::Amb(_) => panic!("unexpected ambiguity"),
                    };
                    list.push(convert_operand(operand, input, ignore_case));
                }
                node = symbol;
            }
            let mut converted = convert_symbol(node, input, ignore_case);
            if !labels.is_empty() {
                converted = Symbol::Exclude {
                    symbol: Box::new(converted),
                    labels,
                };
            }
            Symbol::restricted(converted, restrictions)
        }
        parse_tree::Symbol::Amb(_) => panic!("unexpected ambiguity"),
    }
}

fn convert_operand(
    operand: &parse_tree::Operand,
    input: &Input,
    ignore_case: bool,
) -> RestrictionOperand {
    match operand {
        parse_tree::Operand::Identifier { identifier, .. } => {
            RestrictionOperand::Identifier(Identifier {
                name: input.text(identifier.span()),
                definition: None,
            })
        }
        parse_tree::Operand::Literal { literal, .. } => {
            RestrictionOperand::Literal(convert_literal(literal, input, ignore_case))
        }
        parse_tree::Operand::Amb(_) => panic!("unexpected ambiguity"),
    }
}

/// Converts a string literal, such as `"else"`, `i"select"`, or `s"0x"`.
///
/// In a grammar with `@IgnoreCase`, a string without a prefix becomes an `i`
/// literal if it has an ASCII letter. For example, `"else"` becomes
/// `i"else"`. But `"+"` stays `"+"`, because `i"+"` is an error in a syntax
/// rule.
fn convert_literal(literal: &parse_tree::Literal, input: &Input, ignore_case: bool) -> Literal {
    match literal {
        parse_tree::Literal::Plain { string, .. } => {
            let text = string_text(*string, input);
            if ignore_case && text.contains(|c: char| c.is_ascii_alphabetic()) {
                Literal::ignore_case(&text)
            } else {
                Literal::exact(text)
            }
        }
        parse_tree::Literal::IgnoreCase {
            ignore_case_string, ..
        } => Literal::ignore_case(&string_text(ignore_case_string.string(), input)),
        parse_tree::Literal::Exact { exact_string, .. } => {
            Literal::exact(string_text(exact_string.string(), input))
        }
        parse_tree::Literal::Amb(_) => panic!("unexpected ambiguity"),
    }
}

fn convert_regex_rule(
    rule: &parse_tree::RegexRule,
    input: &Input,
    ignore_case: bool,
) -> LexicalRule {
    let name = input.text(rule.identifier().span());
    let head = Terminal::new(name);

    let regex = Regex::Alt(
        rule.body()
            .regexes()
            .map(|inner| {
                Regex::Seq(
                    inner
                        .map(|r| convert_regex(r, input, ignore_case))
                        .collect(),
                )
            })
            .collect(),
    );
    let mut lexical_rule = LexicalRule::new(head, regex);
    for post_condition in rule.regex_post_conditions().regex_post_conditions() {
        let (list, operand) = match post_condition {
            parse_tree::RegexPostCondition::Except { regex_operand, .. } => {
                (&mut lexical_rule.except, regex_operand)
            }
            parse_tree::RegexPostCondition::FollowRestriction { regex_operand, .. } => {
                (&mut lexical_rule.follow_restriction, regex_operand)
            }
            parse_tree::RegexPostCondition::Amb(_) => panic!("unexpected ambiguity"),
        };
        list.push(convert_regex_operand(operand, input, ignore_case));
    }
    if let Some(condition) = rule.regex_pre_condition().value() {
        lexical_rule.precede_restriction = Some(convert_regex_operand(
            condition.regex_operand(),
            input,
            ignore_case,
        ));
    }
    lexical_rule
}

fn convert_regex_operand(
    operand: &parse_tree::RegexOperand,
    input: &Input,
    ignore_case: bool,
) -> Regex {
    match operand {
        parse_tree::RegexOperand::CharClass { char_class, .. } => {
            convert_char_class(char_class, input)
        }
        parse_tree::RegexOperand::Char { char, .. } => {
            let c = char_text(*char, input);
            if ignore_case {
                Regex::ignore_case_char(c)
            } else {
                Regex::Char(c)
            }
        }
        parse_tree::RegexOperand::Literal { literal, .. } => {
            convert_literal(literal, input, ignore_case).regex()
        }
        parse_tree::RegexOperand::Identifier { identifier, .. } => Regex::Identifier(Identifier {
            name: input.text(identifier.span()),
            definition: None,
        }),
        parse_tree::RegexOperand::Amb(_) => panic!("unexpected ambiguity"),
    }
}

fn convert_regex(regex: &parse_tree::Regex, input: &Input, ignore_case: bool) -> Regex {
    match regex {
        parse_tree::Regex::Plus { regex, .. } => {
            Regex::Plus(Box::new(convert_regex(regex, input, ignore_case)))
        }
        parse_tree::Regex::Star { regex, .. } => {
            Regex::Star(Box::new(convert_regex(regex, input, ignore_case)))
        }
        parse_tree::Regex::Opt { regex, .. } => {
            Regex::Opt(Box::new(convert_regex(regex, input, ignore_case)))
        }
        // #Paren is the parenthesized form: at the syntax level one shape,
        // a list of regex sequences separated by "|", covers both groups
        // and alternations. One sequence constructs a Seq. Several
        // construct an Alt: a one-regex sequence stays as is, and a longer
        // one is wrapped in a Seq, because Alt holds a list of regexes,
        // not a list of sequences.
        parse_tree::Regex::Paren { seqs, .. } => {
            let mut seqs: Vec<Vec<Regex>> = seqs
                .regexes()
                .map(|seq| seq.map(|r| convert_regex(r, input, ignore_case)).collect())
                .collect();
            if seqs.len() == 1 {
                Regex::Seq(seqs.pop().unwrap())
            } else {
                Regex::Alt(
                    seqs.into_iter()
                        .map(|mut seq| {
                            if seq.len() == 1 {
                                seq.pop().unwrap()
                            } else {
                                Regex::Seq(seq)
                            }
                        })
                        .collect(),
                )
            }
        }
        parse_tree::Regex::CharClass { char_class, .. } => convert_char_class(char_class, input),
        parse_tree::Regex::Char { char, .. } => {
            let c = char_text(*char, input);
            if ignore_case {
                Regex::ignore_case_char(c)
            } else {
                Regex::Char(c)
            }
        }
        parse_tree::Regex::Literal { literal, .. } => {
            convert_literal(literal, input, ignore_case).regex()
        }
        parse_tree::Regex::Identifier { identifier, .. } => {
            let name = input.text(identifier.span());
            Regex::Identifier(Identifier {
                name,
                definition: None,
            })
        }
        parse_tree::Regex::Amb(_) => panic!("unexpected ambiguity"),
    }
}

fn convert_char_class(char_class: &parse_tree::CharClass, input: &Input) -> Regex {
    let negated = char_class.neg().value().is_some();
    let ranges = char_class
        .range_elements()
        .range_elements()
        .map(|e| {
            if let Some(range) = e.as_range() {
                let start = range_char(range.start(), input);
                let end = range_char(range.end(), input);
                CharRange { start, end }
            } else if let Some(token) = e.as_range_char() {
                let ch = range_char(*token, input);
                CharRange { start: ch, end: ch }
            } else {
                unreachable!()
            }
        })
        .collect();
    Regex::CharClass(CharClass { ranges, negated })
}

/// The text of a `String` token: the token without its quotes, with each
/// escape sequence replaced by the character it represents. For example,
/// the text of `"\n"` is one newline character, not a backslash and an `n`.
fn string_text(string: parse_tree::Token, input: &Input) -> String {
    let chars = input.chars(string.span());
    unescape(&chars[1..chars.len() - 1])
}

/// The character of a `Char` token, such as `'a'` or `'\u{3C0}'`.
fn char_text(char: parse_tree::Token, input: &Input) -> char {
    let chars = input.chars(char.span());
    single_char(&chars[1..chars.len() - 1])
}

/// The character of a `RangeChar` token, such as `a` or `\-` in `[a\-z]`.
fn range_char(token: parse_tree::Token, input: &Input) -> char {
    single_char(input.chars(token.span()))
}

/// The character of `chars`, the text of one character or one escape. An
/// invalid escape is U+FFFD, the replacement character, as a placeholder
/// (`build_grammar`).
fn single_char(mut chars: &[char]) -> char {
    let c = decode_next(&mut chars).unwrap_or(char::REPLACEMENT_CHARACTER);
    debug_assert!(
        chars.is_empty(),
        "`chars` is more than one character or escape"
    );
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lit;
    use iguana_runtime::input::Span;
    use std::fmt::Display;

    /// Parses without validating. The fragments below exercise one construct
    /// each and leave the names they mention undefined.
    fn parse_unvalidated(source: &str) -> GrammarDef {
        let input = Input::from(source);
        let parser_arena = Arena::new();
        let tree_arena = Arena::new();
        let success = IggyParser::new(&input, &parser_arena)
            .parse_grammar(&tree_arena)
            .expect("the grammar should parse");
        build_grammar(&success.tree, &input)
    }

    /// The first symbol of the first alternative, for a grammar with one
    /// rule.
    fn first_symbol(source: &str) -> Symbol {
        let grammar = parse_unvalidated(source);
        grammar.syntax_rules[0].priority_levels[0].alternatives[0].symbols[0].clone()
    }

    fn names(items: &[impl Display]) -> Vec<String> {
        items.iter().map(ToString::to_string).collect()
    }

    fn messages(errors: &[GrammarError]) -> Vec<&str> {
        errors.iter().map(|error| error.message.as_str()).collect()
    }

    fn restrictions(symbol: &Symbol) -> &Restrictions {
        match symbol {
            Symbol::Restricted { restrictions, .. } => restrictions,
            other => panic!("expected a restricted symbol, got {other:?}"),
        }
    }

    /// A failure at the end of the input gets the empty span at the input
    /// length, so the span stays a valid source range.
    #[test]
    fn test_parse_error_at_end_of_input_has_an_empty_span() {
        let errors = parse_grammar("grammar").expect_err("the grammar is incomplete");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].span, Span::new(7, 7));
    }

    #[test]
    fn test_parse_error_suppresses_layout_expectations() {
        let errors = parse_grammar("grammar g\n\nS = \"a\" @")
            .expect_err("the grammar has an incomplete annotation");
        assert_eq!(errors.len(), 1);
        let message = &errors[0].message;
        assert!(message.contains("Identifier"), "{message}");
        assert!(!message.contains("WS"), "{message}");
        assert!(!message.contains("LineComment"), "{message}");
    }

    #[test]
    fn test_with_layout_is_rejected_at_the_annotation() {
        let source = "grammar g\n\n@WithLayout(WS)\nS = \"a\" \"b\"\n\n@Regex\nWS = [\\ ]*";
        let errors = parse_grammar(source).expect_err("custom layout is not part of Iggy");
        assert_eq!(errors.len(), 1);
        let start = source.find('@').unwrap() as u32;
        assert_eq!(errors[0].span, Span::new(start, start + 1));
        assert!(errors[0].message.starts_with("Expected "), "{errors:?}");
        assert!(!errors[0].message.contains("WithLayout"), "{errors:?}");
    }

    #[test]
    fn test_identifier_annotation_marks_the_rule() {
        let grammar = parse_grammar("grammar g S = Id @Identifier @Regex Id = [a-z]+")
            .expect("the grammar should parse");
        assert_eq!(names(&grammar.identifier_rules), ["Id"]);
    }

    #[test]
    fn test_multiple_identifier_rules_are_recorded() {
        let grammar = parse_grammar(
            r#"grammar g

S = VarId TypeId

@Identifier @Regex
VarId = [a-z]+

@Identifier @Regex
TypeId = [A-Z][a-z]*
"#,
        )
        .expect("the grammar should parse");
        assert_eq!(names(&grammar.identifier_rules), ["VarId", "TypeId"]);
    }

    #[test]
    fn test_grammar_without_a_syntax_rule_is_rejected() {
        let errors =
            parse_grammar("grammar g").expect_err("the grammar declares a name and nothing else");
        assert_eq!(messages(&errors), ["grammar `g` has no syntax rules"]);
    }

    #[test]
    fn test_layout_rule_cannot_be_an_identifier_rule() {
        let errors = parse_grammar("grammar g S = \"a\" @Layout @Identifier @Regex WS = [\\ ]+")
            .expect_err("the layout rule is also an identifier rule");
        assert_eq!(
            messages(&errors),
            ["`WS` cannot be both the layout rule and an identifier rule"]
        );
    }

    #[test]
    fn test_duplicate_layout_rules_are_rejected_in_source_order() {
        for first in [r#"@Layout Spaces = " "*"#, r"@Layout @Regex Spaces = [\ ]*"] {
            for second in [r#"@Layout Tabs = "\t"*"#, r"@Layout @Regex Tabs = [\t]*"] {
                let source = format!(
                    r#"grammar g
S = "a"
{first}
{second}
"#
                );
                let errors = parse_grammar(&source).expect_err("two rules are marked @Layout");
                assert_eq!(
                    messages(&errors),
                    ["`Tabs` is marked `@Layout`, but `Spaces` is already the layout rule"],
                    "{source}"
                );
                let start = source.find("Tabs").unwrap() as u32;
                assert_eq!(errors[0].span, Span::new(start, start + 4), "{source}");
            }
        }
    }

    #[test]
    fn test_duplicate_layout_name_only_reports_duplicate_definition() {
        for annotation in ["@Layout", "@Layout @Regex"] {
            let source = format!(
                r#"grammar g
S = "a"
{annotation} WS = " "*
{annotation} WS = "\t"*
"#
            );
            let errors = parse_grammar(&source).expect_err("WS is defined twice");
            assert_eq!(messages(&errors), ["duplicate definition `WS`"]);
            let start = source.rfind("WS").unwrap() as u32;
            assert_eq!(errors[0].span, Span::new(start, start + 2));
        }
    }

    #[test]
    fn test_duplicate_definitions_stop_semantic_validation() {
        for rules in [
            r#"Item = "a" #One
Item = "b" #Two"#,
            r#"@Regex Item = [a]
@Regex Item = [b]"#,
        ] {
            let source = format!(
                r#"grammar g
S = Item!MissingLabel Undefined
{rules}
@Layout @Identifier @Regex WS = [a-z]*
@Layout Tabs = "\t"*
"#
            );
            let errors = parse_grammar(&source).expect_err("Item is defined twice");
            assert_eq!(messages(&errors), ["duplicate definition `Item`"]);
            let start = source.rfind("Item").unwrap() as u32;
            assert_eq!(errors[0].span, Span::new(start, start + 4));
        }
    }

    #[test]
    fn test_all_duplicate_definitions_are_reported_before_other_errors() {
        let source = r#"grammar g
S = A Missing
A = "a"
A = "b"
A = "c"
@Regex B = [a]
@Regex B = [b]
"#;
        let errors = parse_grammar(source).expect_err("A and B have duplicate definitions");
        assert_eq!(
            messages(&errors),
            [
                "duplicate definition `B`",
                "duplicate definition `A`",
                "duplicate definition `A`",
            ]
        );
        for (error, declaration) in errors.iter().zip(["B = [b]", "A = \"b\"", "A = \"c\""]) {
            let start = source.find(declaration).unwrap() as u32;
            assert_eq!(error.span, Span::new(start, start + 1));
        }
    }

    #[test]
    fn test_duplicate_names_across_syntax_and_lexical_rules() {
        for rules in [
            r#"A = "a"
@Regex A = [a]
"#,
            r#"@Regex A = [a]
A = "a"
"#,
        ] {
            let source = format!(
                r#"grammar g
S = A Missing
{rules}"#
            );
            let errors =
                parse_grammar(&source).expect_err("A names both a syntax and lexical rule");
            assert_eq!(messages(&errors), ["duplicate definition `A`"]);
            let start = source.find("A = \"a\"").unwrap() as u32;
            assert_eq!(errors[0].span, Span::new(start, start + 1));
        }
    }

    #[test]
    fn test_duplicate_layout_does_not_hide_the_first_rules_identifier_conflict() {
        for annotation in ["@Layout", "@Layout @Regex"] {
            let source = format!(
                r#"grammar g
S = "a"
@Layout @Identifier @Regex WS = [a-z]*
{annotation} Tabs = "\t"*
"#
            );
            let errors =
                parse_grammar(&source).expect_err("both layout checks should report errors");
            assert_eq!(
                messages(&errors),
                [
                    "`Tabs` is marked `@Layout`, but `WS` is already the layout rule",
                    "`WS` cannot be both the layout rule and an identifier rule",
                ]
            );
            for (error, name) in errors.iter().zip(["Tabs", "WS"]) {
                let start = source.find(name).unwrap() as u32;
                assert_eq!(error.span, Span::new(start, start + name.len() as u32));
            }
        }
    }

    #[test]
    fn test_every_additional_layout_rule_is_reported() {
        let source = r#"grammar g
S = "a"
@Layout @Regex Spaces = [\ ]*
@Layout Tabs = "\t"*
@Layout @Regex Newlines = [\n]*
"#;
        let errors = parse_grammar(source).expect_err("three rules are marked @Layout");
        assert_eq!(
            messages(&errors),
            [
                "`Tabs` is marked `@Layout`, but `Spaces` is already the layout rule",
                "`Newlines` is marked `@Layout`, but `Spaces` is already the layout rule",
            ]
        );
        for (error, name) in errors.iter().zip(["Tabs", "Newlines"]) {
            let start = source.find(name).unwrap() as u32;
            assert_eq!(error.span, Span::new(start, start + name.len() as u32));
        }
    }

    #[test]
    fn test_zero_or_one_layout_rule_is_accepted() {
        for layout_rule in ["", r#"@Layout WS = " "*"#, r"@Layout @Regex WS = [\ ]*"] {
            let source = format!(
                r#"grammar g
@NoLayout S = "a"
{layout_rule}
"#
            );
            let grammar = parse_grammar(&source).expect("at most one rule is marked @Layout");
            assert_eq!(
                grammar
                    .layout
                    .as_ref()
                    .and_then(Symbol::as_identifier)
                    .map(|id| id.name.as_str()),
                if layout_rule.is_empty() {
                    None
                } else {
                    Some("WS")
                }
            );
        }
    }

    #[test]
    fn test_restriction_kinds_of_one_symbol_land_in_one_node() {
        let symbol = first_symbol("grammar g S = Id \\ Kw !>> B C");
        let restrictions = restrictions(&symbol);
        assert_eq!(names(&restrictions.excepts), ["Kw"]);
        assert_eq!(names(&restrictions.follow), ["B"]);
        assert!(matches!(symbol.unwrap(), Symbol::Identifier(id) if id.name == "Id"));
    }

    #[test]
    fn test_required_matches_land_in_the_node_of_the_other_restrictions() {
        let symbol = first_symbol("grammar g S = Id \\ Kw & T & U !>> B C");
        let restrictions = restrictions(&symbol);
        assert_eq!(names(&restrictions.excepts), ["Kw"]);
        assert_eq!(names(&restrictions.required), ["T", "U"]);
        assert_eq!(names(&restrictions.follow), ["B"]);
        assert!(matches!(symbol.unwrap(), Symbol::Identifier(id) if id.name == "Id"));
    }

    #[test]
    fn test_precede_and_follow_restrictions_land_in_one_node() {
        let symbol = first_symbol("grammar g S = X !<< A !>> B C");
        let restrictions = restrictions(&symbol);
        assert_eq!(names(&restrictions.precede), ["X"]);
        assert_eq!(names(&restrictions.follow), ["B"]);
    }

    #[test]
    fn test_restrictions_keep_their_source_order() {
        let symbol = first_symbol("grammar g S = X !<< Y !<< A !>> B !>> C \\ K1 \\ K2");
        let restrictions = restrictions(&symbol);
        assert_eq!(names(&restrictions.precede), ["X", "Y"]);
        assert_eq!(names(&restrictions.follow), ["B", "C"]);
        assert_eq!(names(&restrictions.excepts), ["K1", "K2"]);
    }

    /// The conversion keeps a repeated operand, and `restriction_dedup`
    /// removes it.
    #[test]
    fn test_restriction_written_twice_is_recorded_twice() {
        let symbol = first_symbol("grammar g S = A !>> B \\ K !>> B C");
        assert_eq!(names(&restrictions(&symbol).follow), ["B", "B"]);
    }

    #[test]
    fn test_literal_operands_convert_like_literal_symbols() {
        let symbol = first_symbol(r#"grammar g S = A !>> "+" !>> i"if" !>> s"Do" B"#);
        assert_eq!(
            restrictions(&symbol).follow,
            [
                RestrictionOperand::Literal(Literal::exact("+")),
                RestrictionOperand::Literal(Literal::ignore_case("if")),
                RestrictionOperand::Literal(Literal::exact("Do")),
            ]
        );
    }

    #[test]
    fn test_a_literal_operand_ignores_case_in_an_ignore_case_grammar() {
        let symbol = first_symbol(r#"@IgnoreCase grammar g S = A !>> "Else" !>> "+" B"#);
        assert_eq!(
            names(&restrictions(&symbol).follow),
            [r#"i"else""#, r#""+""#]
        );
    }

    #[test]
    fn test_layout_aware_and_plain_follow_restrictions_stay_apart() {
        let symbol = first_symbol("grammar g S = A !>> B !>>> C D");
        let restrictions = restrictions(&symbol);
        assert_eq!(names(&restrictions.follow), ["B"]);
        assert_eq!(names(&restrictions.layout_aware_follow), ["C"]);
    }

    #[test]
    fn test_label_stays_above_the_restrictions() {
        let symbol = first_symbol("grammar g S = x:A !>> B C");
        let Symbol::Labeled { label, symbol } = &symbol else {
            panic!("expected a labeled symbol, got {symbol:?}");
        };
        assert_eq!(label, "x");
        assert_eq!(names(&restrictions(symbol).follow), ["B"]);
    }

    #[test]
    fn test_restrictions_of_a_parenthesized_symbol_apply_to_the_group() {
        let symbol = first_symbol("grammar g S = (A B) !>> C D");
        assert_eq!(names(&restrictions(&symbol).follow), ["C"]);
        assert!(matches!(symbol.unwrap(), Symbol::Group(symbols) if symbols.len() == 2));
    }

    #[test]
    fn test_exclusion_stays_below_the_restrictions() {
        for source in [
            "grammar g S = A!label !>> B C",
            "grammar g S = A !>> B !label C",
        ] {
            let symbol = first_symbol(source);
            assert_eq!(names(&restrictions(&symbol).follow), ["B"], "{source}");
            assert!(
                matches!(symbol.unwrap(), Symbol::Exclude { labels, .. } if labels == &["label"]),
                "{source}",
            );
        }
    }

    /// A name reaches the generated code through a case conversion, so a
    /// reserved name has to be recognized in the form the generator writes it
    /// rather than in the spelling the grammar uses. Each case below compiled
    /// to a rustc error in generated code before the conversion was applied.
    #[test]
    fn test_reserved_names_are_matched_after_case_conversion() {
        for (source, expected) in [
            (
                "grammar g S = parse_tree parse_tree = \"a\"",
                "`parse_tree` conflicts with `ParseTree` in the generated code",
            ),
            (
                "grammar g S = g_parse_tree_builder g_parse_tree_builder = \"a\"",
                "`g_parse_tree_builder` conflicts with `GParseTreeBuilder` in the generated code",
            ),
            (
                "grammar g S = Span:\"a\"",
                "`Span` conflicts with `span` in the generated code",
            ),
            (
                "grammar g S = start_s start_s = \"a\"",
                "`start_s` conflicts with `StartS` in the generated code",
            ),
            (
                "grammar g S = \"a\" #amb",
                "`#amb` becomes `Amb`, which is reserved for ambiguous alternatives",
            ),
            (
                "grammar g S = \"a\" #cycle",
                "`#cycle` becomes `Cycle`, which is reserved for cyclic references",
            ),
            (
                "grammar g S = \"a\" #alt_0",
                "`#alt_0` becomes `Alt0`, which is reserved for unlabeled alternatives",
            ),
            (
                "grammar g S = \"a\" #self",
                "`#self` becomes the variant `Self`, which is a Rust keyword",
            ),
            (
                "grammar g S = Self:\"a\"",
                "`Self` becomes the field `self`, which cannot be a Rust field name",
            ),
            (
                "grammar g S = crate:\"a\"",
                "`crate` becomes the field `crate`, which cannot be a Rust field name",
            ),
        ] {
            let errors = parse_grammar(source).expect_err(source);
            assert!(
                messages(&errors).contains(&expected),
                "{source}\n  expected: {expected}\n  got: {:?}",
                messages(&errors),
            );
        }
    }

    /// A name starts with a letter, so `_` is a parse error rather than a
    /// label that the validator has to reject as a field name.
    #[test]
    fn test_names_cannot_start_with_an_underscore() {
        for source in [
            "grammar g S = _:\"a\"",
            "grammar g S = \"a\" #_x",
            "grammar g _s = \"a\"",
        ] {
            let errors = parse_grammar(source).expect_err(source);
            assert!(
                messages(&errors)[0].starts_with("Expected"),
                "{source}: {:?}",
                messages(&errors)
            );
        }
    }

    /// A keyword label becomes a raw identifier in the generated code, so it
    /// is accepted. The rejected labels are the ones raw identifier syntax
    /// cannot escape.
    #[test]
    fn test_keyword_labels_that_raw_identifiers_can_escape_are_accepted() {
        for source in [
            "grammar g S = type:\"a\"",
            "grammar g S = do:\"a\"",
            "grammar g S = \"a\" #type",
        ] {
            parse_grammar(source).unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
        }
    }

    /// Two labels in one sequence that become the same field name generate a
    /// struct with two fields of that name. Labels in different sequences,
    /// including the alternatives of one parenthesized alternation, become
    /// fields of different types.
    #[test]
    fn test_field_labels_must_be_distinct_within_a_sequence() {
        for (source, expected) in [
            ("grammar g S = x:\"a\" x:\"b\"", "duplicate label `x`"),
            (
                "grammar g S = fooBar:\"a\" foo_bar:\"b\"",
                "`fooBar` and `foo_bar` both become the field `foo_bar`",
            ),
            (
                "grammar g S = \"c\" (x:\"a\" x:\"b\")",
                "duplicate label `x`",
            ),
            (
                "grammar g S = (\"c\" | x:\"a\" x:\"b\")",
                "duplicate label `x`",
            ),
            ("grammar g S = (x:\"a\" x:\"b\")*", "duplicate label `x`"),
            ("grammar g S = {x:\"a\" x:\",\"}*", "duplicate label `x`"),
            ("grammar g S = (x:\"a\" x:\"b\")?", "duplicate label `x`"),
        ] {
            let errors = parse_grammar(source).expect_err(source);
            assert_eq!(messages(&errors), [expected], "{source}");
        }
        for source in [
            "grammar g S = x:\"a\" | x:\"b\"",
            "grammar g S = x:\"a\" (x:\"b\")",
            "grammar g S = (x:\"a\" | x:\"b\")",
            "grammar g S = x:\"a\" {x:\"b\" \",\"}*",
            "grammar g S = x:\"a\" y = x:\"b\"",
        ] {
            parse_grammar(source).unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
        }
    }

    #[test]
    fn test_generated_names_must_be_distinct_after_case_conversion() {
        let errors = parse_grammar("grammar g A = \"a\" a = \"b\"")
            .expect_err("the rule names generate duplicate Rust identifiers");
        assert_eq!(
            messages(&errors),
            ["`A` and `a` have the same generated name `A`"]
        );

        let errors = parse_grammar("grammar g S = \"a\" #foo_bar | \"b\" #FooBar")
            .expect_err("the labels generate duplicate Rust variants");
        assert_eq!(
            messages(&errors),
            ["`#foo_bar` and `#FooBar` both become the generated variant `FooBar`"]
        );
    }

    #[test]
    fn test_restriction_must_name_a_lexical_rule() {
        let errors = parse_grammar("grammar g S = A !>> B A = \"a\" B = \"b\"")
            .expect_err("a restriction cannot name a syntax rule");
        assert_eq!(
            messages(&errors),
            ["follow restriction `B` must refer to a lexical rule, not a syntax rule"]
        );

        let errors = parse_grammar("grammar g S = Token @Regex Token = [a-z]+ !>> S")
            .expect_err("a lexical rule restriction cannot name a syntax rule");
        assert_eq!(
            messages(&errors),
            ["follow restriction `S` must refer to a lexical rule, not a syntax rule"]
        );
    }

    #[test]
    fn test_restriction_operand_must_not_be_a_literal() {
        let source = r#"grammar g S = A !>> "+" A = "a""#;
        let errors = parse_grammar(source).expect_err("a literal operand is not supported");
        assert_eq!(
            messages(&errors),
            [
                "follow restriction must be a rule name: a string literal is not supported as a restriction operand"
            ]
        );
        let start = source.find(r#""+""#).unwrap() as u32;
        assert_eq!(errors[0].span, Span::new(start, start + 3));
    }

    #[test]
    fn test_lexical_operands_convert_like_regexes() {
        let grammar = parse_unvalidated(
            r#"@IgnoreCase grammar g S = Kw @Regex Kw = 'a' !<< "if" \ s"If" !>> [0-9] !>> 'x'"#,
        );
        let rule = &grammar.lexical_rules[0];
        assert_eq!(rule.precede_restriction, Some(Regex::ignore_case_char('a')));
        assert_eq!(rule.except, [Regex::exact_literal("If")]);
        assert_eq!(
            rule.follow_restriction,
            [
                Regex::char_class(
                    vec![CharRange {
                        start: '0',
                        end: '9'
                    }],
                    false
                ),
                Regex::ignore_case_char('x')
            ]
        );
    }

    #[test]
    fn test_lexical_restriction_operand_must_be_a_rule_name() {
        let source = r#"grammar g S = Kw @Regex Kw = "if" !>> [a-z]"#;
        let errors = parse_grammar(source).expect_err("a class operand is not supported");
        assert_eq!(
            messages(&errors),
            [
                "follow restriction must be a rule name: a character, a string, or a character class is not supported as a restriction operand"
            ]
        );
        let start = source.find("[a-z]").unwrap() as u32;
        assert_eq!(errors[0].span, Span::new(start, start + 5));
    }

    #[test]
    fn test_except_operand_cannot_have_restrictions() {
        let errors = parse_grammar(
            r#"grammar g

S = Identifier

@Regex
Identifier = [a-z]+ \ Keyword

@Regex
Keyword = "if" !>> Tail

@Regex
Tail = [a-z]
"#,
        )
        .expect_err("an except operand cannot have restrictions");
        assert_eq!(
            messages(&errors),
            ["except `Keyword` must refer to a lexical rule with no restrictions of its own"]
        );
    }

    #[test]
    fn test_required_match_operand_cannot_have_restrictions() {
        let errors = parse_grammar(
            r#"grammar g

S = Word & Keyword

@Regex
Word = [a-z]+

@Regex
Keyword = "if" !>> Tail

@Regex
Tail = [a-z]
"#,
        )
        .expect_err("a required match operand cannot have restrictions");
        assert_eq!(
            messages(&errors),
            [
                "required match `Keyword` must refer to a lexical rule with no restrictions of its own"
            ]
        );
    }

    #[test]
    fn test_required_match_operand_must_be_a_lexical_rule() {
        let errors = parse_grammar(
            r#"grammar g

S = Word & Name

Name = Word

@Regex
Word = [a-z]+
"#,
        )
        .expect_err("a required match operand must be a lexical rule");
        assert_eq!(
            messages(&errors),
            ["required match `Name` must refer to a lexical rule, not a syntax rule"]
        );
    }

    #[test]
    fn test_lexical_rule_cannot_have_a_required_match() {
        let source = r#"grammar g

S = Word

@Regex
Word = [a-z]+ & Lower

@Regex
Lower = [a-z]+
"#;
        let errors = parse_grammar(source).expect_err("`&` is not part of the lexical rule syntax");
        assert_eq!(errors.len(), 1);
        let start = source.find('&').unwrap() as u32;
        assert_eq!(errors[0].span, Span::new(start, start + 1));
        assert!(errors[0].message.starts_with("Expected "), "{errors:?}");
    }

    #[test]
    fn test_exclusion_only_applies_to_a_rule_reference() {
        for source in [
            "grammar g S = (A)!label A = \"a\" #label",
            "grammar g S = A+!label A = \"a\" #label",
        ] {
            let errors = parse_grammar(source)
                .expect_err("an exclusion cannot be applied to a group or repetition");
            assert_eq!(
                messages(&errors),
                ["exclusion `!label` only applies to a syntax rule reference"],
                "{source}",
            );
        }
    }

    #[test]
    fn test_unicode_escapes_are_decoded() {
        let grammar = parse_unvalidated(
            r#"grammar g

S = "\u{3C0}\u{000041}b" "\\u{41}"

@Regex
T = '\u{0}' [\u{41}-\u{5A}\u{3C0}] "\u{1F600}"
"#,
        );
        let symbols = &grammar.syntax_rules[0].priority_levels[0].alternatives[0].symbols;
        assert_eq!(symbols[0], lit!("πAb"));
        assert_eq!(symbols[1], lit!(r"\u{41}"));
        let class = CharClass {
            ranges: vec![
                CharRange {
                    start: 'A',
                    end: 'Z',
                },
                CharRange {
                    start: 'π',
                    end: 'π',
                },
            ],
            negated: false,
        };
        assert_eq!(
            grammar.lexical_rules[0].regex,
            Regex::Alt(vec![Regex::Seq(vec![
                Regex::Char('\0'),
                Regex::CharClass(class),
                Regex::exact_literal("😀"),
            ])])
        );
    }

    /// The other checks still run, so the unresolved `Missing` is reported
    /// after the escapes. The placeholder of `\u{0000041}` does not make
    /// `i"\u{0000041}"` an `i` literal without a letter.
    #[test]
    fn test_invalid_unicode_escapes_are_rejected_at_the_escape() {
        let source = r#"grammar g

S = "a\u{D800}" i"b\u{DFFF}" i"\u{0000041}" T Missing

@Regex
T = [\u{110000}] '\u{1234567}' "\u{10FFFF}"
"#;
        let errors = parse_grammar(source).expect_err("the grammar has invalid escapes");
        let spans: Vec<&str> = errors
            .iter()
            .map(|error| &source[error.span.left_extent as usize..error.span.right_extent as usize])
            .collect();
        assert_eq!(
            spans,
            [
                r"\u{D800}",
                r"\u{DFFF}",
                r"\u{0000041}",
                r"\u{110000}",
                r"\u{1234567}",
                "Missing"
            ]
        );
        for error in &errors[..5] {
            assert_eq!(
                error.message,
                "the code point of a `\\u{...}` escape must have at most six hexadecimal \
                 digits, be at most `10FFFF`, and not be a surrogate (`D800` to `DFFF`)"
            );
        }
    }

    #[test]
    fn test_plain_literal_ignores_case_with_the_ignore_case_annotation() {
        let grammar = parse_unvalidated(
            r#"@IgnoreCase
grammar G

S = "Select" "(" i"from" s"0x"

@Regex
A = "ab" s"cd" 'e' '_' [f]
"#,
        );
        assert_eq!(
            grammar.syntax_rules[0].priority_levels[0].alternatives[0].symbols,
            [
                Symbol::Literal(Literal::ignore_case("select")),
                lit!("("),
                Symbol::Literal(Literal::ignore_case("from")),
                lit!("0x"),
            ]
        );
        assert_eq!(
            grammar.lexical_rules[0].regex,
            Regex::Alt(vec![Regex::Seq(vec![
                Regex::ignore_case_literal("ab"),
                Regex::exact_literal("cd"),
                Regex::ignore_case_char('e'),
                Regex::Char('_'),
                Regex::char_class(
                    vec![CharRange {
                        start: 'f',
                        end: 'f'
                    }],
                    false
                ),
            ])])
        );
    }

    #[test]
    fn test_plain_literal_is_exact_without_the_ignore_case_annotation() {
        let grammar = parse_unvalidated(
            r#"grammar G

S = "Select" i"from" s"0x"
"#,
        );
        assert_eq!(
            grammar.syntax_rules[0].priority_levels[0].alternatives[0].symbols,
            [
                lit!("Select"),
                Symbol::Literal(Literal::ignore_case("from")),
                lit!("0x"),
            ]
        );
    }

    #[test]
    fn test_labels_cannot_be_nested_directly() {
        let errors = parse_grammar("grammar g S = outer:inner:\"a\"")
            .expect_err("labels cannot be nested directly");
        assert_eq!(
            messages(&errors),
            ["labels cannot be nested directly: `outer:inner:` applies two labels to one symbol"]
        );
    }
}
