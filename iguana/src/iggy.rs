use iggy::{
    ParseError, parse_tree,
    parse_tree::{OptNode, Start},
};
use iguana_runtime::{input::Input, parse_tree::ParseContext, sppf::Span};

use crate::grammar::{
    def::{
        Alternative, Associativity, GrammarDef, LayoutStrategy, LexicalRule, PriorityLevel,
        SyntaxRule,
    },
    regex::{CharClass, CharRange, Regex},
    symbols::{Identifier, Nonterminal, Symbol, Terminal},
};

pub fn parse_grammar(source: &str) -> Result<GrammarDef, ParseError> {
    let input = Input::from(source);
    let ctx = ParseContext::new();
    let success = iggy::parse_grammar(&input, &ctx)?;
    build_grammar(success.tree, &input)
}

fn text(input: &Input, span: Span) -> String {
    input.substring(span.left_extent, span.right_extent)
}

pub fn build_grammar(
    start_grammar: &Start<&parse_tree::Grammar<'_>, &parse_tree::Layout<'_>>,
    input: &Input,
) -> Result<GrammarDef, ParseError> {
    let grammar = &start_grammar.node;
    let name = text(input, grammar.name().span());

    let mut syntax_rules: Vec<SyntaxRule> = Vec::new();
    let mut lexical_rules: Vec<LexicalRule> = Vec::new();
    let mut layout_name: Option<String> = None;

    for rule in grammar.rules().rules() {
        match rule {
            parse_tree::Rule::SyntaxRule { syntax_rule, .. } => {
                let converted = convert_syntax_rule(syntax_rule, input);
                if has_layout_annotation(syntax_rule) {
                    layout_name = Some(converted.head.name.clone());
                }
                syntax_rules.push(converted);
            }
            parse_tree::Rule::RegexRule { regex_rule, .. } => {
                let converted = convert_regex_rule(regex_rule, input);
                if regex_rule.layout().value().is_some() {
                    layout_name = Some(converted.head.name.clone());
                }
                lexical_rules.push(converted);
            }
            parse_tree::Rule::Amb(_) => panic!("unexpected ambiguity"),
        }
    }

    // The grammar's layout is the rule marked `@Layout`.
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

    Ok(GrammarDef {
        name,
        syntax_rules,
        lexical_rules,
        layout,
    })
}

/// Whether a syntax rule carries the `@Layout` annotation, marking it as the
/// grammar's layout rule.
fn has_layout_annotation(rule: &parse_tree::SyntaxRule) -> bool {
    matches!(
        rule.annotation().value(),
        Some(parse_tree::Annotation::Layout { .. })
    )
}

fn convert_syntax_rule(rule: &parse_tree::SyntaxRule, input: &Input) -> SyntaxRule {
    let head_name = text(input, rule.head().span());
    let head = Nonterminal::new(head_name);

    let priority_levels: Vec<PriorityLevel> = rule
        .priority_levels()
        .priority_levels()
        .map(|level| convert_priority_level(level, input))
        .collect();

    let mut layout = LayoutStrategy::Default;
    let mut start = false;

    if let Some(annotation) = rule.annotation().value() {
        match annotation {
            parse_tree::Annotation::NoLayout { .. } => layout = LayoutStrategy::None,
            // `@Layout` marks the grammar's layout rule. build_grammar records
            // its name and suppresses layout inside it, so nothing to do here.
            parse_tree::Annotation::Layout { .. } => {}
            parse_tree::Annotation::WithLayout { identifier, .. } => {
                let name = text(input, identifier.span());
                layout = LayoutStrategy::Custom(Identifier {
                    name,
                    definition: None,
                });
            }
            parse_tree::Annotation::Start { .. } => start = true,
            parse_tree::Annotation::Amb(_) => panic!("unexpected ambiguity"),
        }
    }

    SyntaxRule {
        head,
        priority_levels,
        layout,
        start,
    }
}

fn convert_priority_level(level: &parse_tree::PriorityLevel, input: &Input) -> PriorityLevel {
    let alternatives: Vec<Alternative> = level
        .alternatives()
        .alternatives()
        .map(|alt| convert_alternative(alt, input))
        .collect();

    let associativity =
        level
            .associativity()
            .value()
            .map(|assoc: &parse_tree::Associativity<'_>| {
                match text(input, assoc.span()).as_str() {
                    "left" => Associativity::Left,
                    "right" => Associativity::Right,
                    "none" => Associativity::NonAssoc,
                    other => panic!("Unknown associativity: {other}"),
                }
            });

    PriorityLevel::with_associativity(alternatives, associativity)
}

fn convert_alternative(alt: &parse_tree::Alternative, input: &Input) -> Alternative {
    let symbols: Vec<Symbol> = alt
        .symbols()
        .symbols()
        .map(|sym| convert_symbol(sym, input))
        .collect();

    // Extract label, stripping the # prefix
    let label = alt.label().value().map(|token| {
        let label_text = text(input, token.span());
        label_text
            .strip_prefix('#')
            .unwrap_or(&label_text)
            .to_string()
    });

    Alternative { symbols, label }
}

fn convert_symbol(symbol: &parse_tree::Symbol, input: &Input) -> Symbol {
    match symbol {
        parse_tree::Symbol::Star { symbol, .. } => {
            Symbol::Star(Box::new(convert_symbol(symbol, input)), None)
        }
        parse_tree::Symbol::Plus { symbol, .. } => {
            Symbol::Plus(Box::new(convert_symbol(symbol, input)), None)
        }
        parse_tree::Symbol::Opt { symbol, .. } => {
            Symbol::Opt(Box::new(convert_symbol(symbol, input)))
        }
        parse_tree::Symbol::Alt { first, rest, .. } => {
            let mut symbols = vec![convert_symbol(first, input)];
            let rest: Vec<Symbol> = rest.symbols().map(|s| convert_symbol(s, input)).collect();
            symbols.extend(rest);
            Symbol::Alt(symbols)
        }
        parse_tree::Symbol::Lit { string, .. } => {
            let raw = text(input, string.span());
            Symbol::Literal(unescape_string(&raw[1..raw.len() - 1]))
        }
        parse_tree::Symbol::StarSep { symbol, sep, .. } => Symbol::Star(
            Box::new(convert_symbol(symbol, input)),
            Some(Box::new(convert_symbol(sep, input))),
        ),
        parse_tree::Symbol::PlusSep { symbol, sep, .. } => Symbol::Plus(
            Box::new(convert_symbol(symbol, input)),
            Some(Box::new(convert_symbol(sep, input))),
        ),
        parse_tree::Symbol::Group { symbols, .. } => Symbol::Group(
            symbols
                .symbols()
                .map(|s| convert_symbol(s, input))
                .collect(),
        ),
        parse_tree::Symbol::Labeled { label, symbol, .. } => Symbol::Labeled {
            label: text(input, label.span()),
            symbol: Box::new(convert_symbol(symbol, input)),
        },
        parse_tree::Symbol::Identifier { identifier, .. } => Symbol::Identifier(Identifier {
            name: text(input, identifier.span()),
            definition: None,
        }),
        parse_tree::Symbol::Except {
            symbol, excepts, ..
        } => Symbol::Except {
            symbol: Box::new(convert_symbol(symbol, input)),
            except: excepts
                .identifiers()
                .map(|token| Identifier {
                    name: text(input, token.span()),
                    definition: None,
                })
                .collect(),
        },
        parse_tree::Symbol::FollowRestriction {
            symbol,
            restrictions,
            ..
        } => Symbol::FollowRestriction {
            symbol: Box::new(convert_symbol(symbol, input)),
            restrictions: restrictions
                .identifiers()
                .map(|token| Identifier {
                    name: text(input, token.span()),
                    definition: None,
                })
                .collect(),
        },
        parse_tree::Symbol::PrecedeRestriction {
            symbol, identifier, ..
        } => Symbol::PrecedeRestriction {
            symbol: Box::new(convert_symbol(symbol, input)),
            restriction: Identifier {
                name: text(input, identifier.span()),
                definition: None,
            },
        },
        parse_tree::Symbol::Exclude { symbol, labels, .. } => {
            let labels = labels
                .identifiers()
                .map(|token| text(input, token.span()))
                .collect();
            Symbol::Exclude {
                symbol: Box::new(convert_symbol(symbol, input)),
                labels,
            }
        }
        parse_tree::Symbol::Amb(_) => panic!("unexpected ambiguity"),
    }
}

fn convert_regex_rule(rule: &parse_tree::RegexRule, input: &Input) -> LexicalRule {
    let name = text(input, rule.identifier().span());
    let head = Terminal::new(name);

    let regex = Regex::Alt(
        rule.body()
            .regexes()
            .map(|inner| Regex::Seq(inner.map(|r| convert_regex(r, input)).collect()))
            .collect(),
    );
    let mut lexical_rule = LexicalRule::new(head, regex);
    for post_condition in rule.post_conditions().post_conditions() {
        match post_condition {
            parse_tree::PostCondition::Except { identifier, .. } => {
                lexical_rule.except.push(Identifier {
                    name: text(input, identifier.span()),
                    definition: None,
                });
            }
            parse_tree::PostCondition::FollowRestriction { identifier, .. } => {
                assert!(
                    lexical_rule.follow_restriction.is_none(),
                    "Duplicate follow restriction on terminal {}",
                    lexical_rule.head
                );
                lexical_rule.follow_restriction = Some(Identifier {
                    name: text(input, identifier.span()),
                    definition: None,
                });
            }
            parse_tree::PostCondition::Amb(_) => panic!("unexpected ambiguity"),
        }
    }
    if let Some(pc) = rule.pre_condition().value() {
        lexical_rule.precede_restriction = Some(Identifier {
            name: text(input, pc.identifier().span()),
            definition: None,
        });
    }
    lexical_rule
}

fn convert_regex(regex: &parse_tree::Regex, input: &Input) -> Regex {
    match regex {
        parse_tree::Regex::Plus { regex, .. } => Regex::Plus(Box::new(convert_regex(regex, input))),
        parse_tree::Regex::Star { regex, .. } => Regex::Star(Box::new(convert_regex(regex, input))),
        parse_tree::Regex::Opt { regex, .. } => Regex::Opt(Box::new(convert_regex(regex, input))),
        parse_tree::Regex::Alt { first, rest, .. } => {
            let mut regexes = vec![convert_regex(first, input)];
            let rest_regexes: Vec<Regex> =
                rest.regexes().map(|r| convert_regex(r, input)).collect();
            regexes.extend(rest_regexes);
            Regex::Alt(regexes)
        }
        parse_tree::Regex::CharClass { char_class, .. } => convert_char_class(char_class, input),
        parse_tree::Regex::Char { char, .. } => {
            let raw = text(input, char.span());
            Regex::Char(parse_char(&raw[1..raw.len() - 1]))
        }
        parse_tree::Regex::Group { regexes, .. } => {
            Regex::Seq(regexes.regexes().map(|r| convert_regex(r, input)).collect())
        }
        parse_tree::Regex::String { string, .. } => {
            let raw = text(input, string.span());
            let unescaped = unescape_string(&raw[1..raw.len() - 1]);
            let regexes = unescaped.chars().map(Regex::Char).collect();
            Regex::Seq(regexes)
        }
        parse_tree::Regex::Identifier { identifier, .. } => {
            let name = text(input, identifier.span());
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
                let start = parse_range_char(&text(input, range.start().span()));
                let end = parse_range_char(&text(input, range.end().span()));
                CharRange { start, end }
            } else if let Some(range_char) = e.as_range_char() {
                let ch = parse_range_char(&text(input, range_char.span()));
                CharRange { start: ch, end: ch }
            } else {
                unreachable!()
            }
        })
        .collect();
    Regex::CharClass(CharClass { ranges, negated })
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                let escaped = match next {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'f' => '\x0c', // form feed
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                };
                result.push(escaped);
                chars.next();
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_char(s: &str) -> char {
    if s.starts_with('\\') && s.len() > 1 {
        match s.chars().nth(1) {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('f') => '\x0c', // form feed
            Some('\\') => '\\',
            Some('"') => '"',
            Some(c) => c,
            None => s.chars().next().unwrap(),
        }
    } else {
        s.chars().next().unwrap()
    }
}

fn parse_range_char(s: &str) -> char {
    parse_char(s)
}
