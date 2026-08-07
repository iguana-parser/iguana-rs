use iggy::{
    ParseError, parse_tree,
    parse_tree::{OptNode, Start},
};
use iguana_runtime::{arena::Arena, input::Input};

use crate::grammar::{
    def::{
        Alternative, Associativity, GrammarDef, LayoutStrategy, LexicalRule, PriorityLevel,
        SyntaxRule,
    },
    regex::{CharClass, CharRange, Regex},
    symbols::{Identifier, Nonterminal, Restrictions, Symbol, Terminal},
};

pub fn parse_grammar(source: &str) -> Result<GrammarDef, ParseError> {
    let input = Input::from(source);
    let tree_arena = Arena::new();
    let success = iggy::parse_grammar(&input, &tree_arena)?;
    build_grammar(success.tree, &input)
}

pub fn build_grammar(
    start_grammar: &Start<&parse_tree::Grammar<'_>, &parse_tree::Layout<'_>>,
    input: &Input,
) -> Result<GrammarDef, ParseError> {
    let grammar = &start_grammar.node;
    let name = input.text(grammar.name().span());

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
    let head_name = input.text(rule.head().span());
    let head = Nonterminal::new(head_name);

    let priority_levels: Vec<PriorityLevel> = rule
        .priority_levels()
        .priority_levels()
        .map(|level| convert_priority_level(level, input))
        .collect();

    let mut layout = LayoutStrategy::Default;

    if let Some(annotation) = rule.annotation().value() {
        match annotation {
            parse_tree::Annotation::NoLayout { .. } => layout = LayoutStrategy::None,
            // `@Layout` marks the grammar's layout rule. build_grammar records
            // its name and suppresses layout inside it, so nothing to do here.
            parse_tree::Annotation::Layout { .. } => {}
            parse_tree::Annotation::WithLayout { identifier, .. } => {
                let name = input.text(identifier.span());
                layout = LayoutStrategy::Custom(Identifier {
                    name,
                    definition: None,
                });
            }
            parse_tree::Annotation::Amb(_) => panic!("unexpected ambiguity"),
        }
    }

    SyntaxRule {
        head,
        priority_levels,
        layout,
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

fn convert_alternative(alt: &parse_tree::Alternative, input: &Input) -> Alternative {
    let (symbols, label): (Vec<Symbol>, _) = match alt {
        parse_tree::Alternative::Symbols { symbols, label, .. } => (
            symbols
                .symbols()
                .map(|sym| convert_symbol(sym, input))
                .collect(),
            label,
        ),
        parse_tree::Alternative::Empty { label, .. } => (Vec::new(), label),
        parse_tree::Alternative::Amb(_) => panic!("unexpected ambiguity"),
    };

    // Extract label, stripping the # prefix
    let label = label.value().map(|token| {
        let label_text = input.text(token.span());
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
        // #Paren is the parenthesized form: at the syntax level one shape,
        // a list of symbol sequences separated by "|", covers both groups
        // and alternations. One sequence constructs a Group. Several
        // construct an Alt: a one-symbol sequence stays as is, and a longer
        // one is wrapped in a Group, because Alt holds a list of symbols,
        // not a list of sequences.
        parse_tree::Symbol::Paren { seqs, .. } => {
            let mut seqs: Vec<Vec<Symbol>> = seqs
                .symbols()
                .map(|seq| seq.map(|s| convert_symbol(s, input)).collect())
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
        parse_tree::Symbol::Lit { string, .. } => {
            let raw = input.text(string.span());
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
        parse_tree::Symbol::Labeled { label, symbol, .. } => Symbol::Labeled {
            label: input.text(label.span()),
            symbol: Box::new(convert_symbol(symbol, input)),
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
                    add_restriction(&mut restrictions.precede, condition.identifier(), input);
                }
                node = symbol;
            }
            let mut labels = Vec::new();
            if let parse_tree::Symbol::PostCondition {
                conditions, symbol, ..
            } = node
            {
                for condition in conditions.post_conditions() {
                    match condition {
                        parse_tree::PostCondition::Except { identifier, .. } => {
                            add_restriction(&mut restrictions.excepts, *identifier, input);
                        }
                        parse_tree::PostCondition::FollowRestriction { identifier, .. } => {
                            add_restriction(&mut restrictions.follow, *identifier, input);
                        }
                        parse_tree::PostCondition::LayoutAwareFollowRestriction {
                            identifier,
                            ..
                        } => {
                            add_restriction(
                                &mut restrictions.layout_aware_follow,
                                *identifier,
                                input,
                            );
                        }
                        parse_tree::PostCondition::Exclude { identifier, .. } => {
                            labels.push(input.text(identifier.span()));
                        }
                        parse_tree::PostCondition::Amb(_) => panic!("unexpected ambiguity"),
                    }
                }
                node = symbol;
            }
            let mut converted = convert_symbol(node, input);
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

/// Appends the restriction unless `list` already holds it.
fn add_restriction(list: &mut Vec<Identifier>, token: parse_tree::Token, input: &Input) {
    let id = Identifier {
        name: input.text(token.span()),
        definition: None,
    };
    if !list.contains(&id) {
        list.push(id);
    }
}

fn convert_regex_rule(rule: &parse_tree::RegexRule, input: &Input) -> LexicalRule {
    let name = input.text(rule.identifier().span());
    let head = Terminal::new(name);

    let regex = Regex::Alt(
        rule.body()
            .regexes()
            .map(|inner| Regex::Seq(inner.map(|r| convert_regex(r, input)).collect()))
            .collect(),
    );
    let mut lexical_rule = LexicalRule::new(head, regex);
    for post_condition in rule.regex_post_conditions().regex_post_conditions() {
        match post_condition {
            parse_tree::RegexPostCondition::Except { identifier, .. } => {
                lexical_rule.except.push(Identifier {
                    name: input.text(identifier.span()),
                    definition: None,
                });
            }
            parse_tree::RegexPostCondition::FollowRestriction { identifier, .. } => {
                lexical_rule.follow_restriction.push(Identifier {
                    name: input.text(identifier.span()),
                    definition: None,
                });
            }
            parse_tree::RegexPostCondition::Amb(_) => panic!("unexpected ambiguity"),
        }
    }
    if let Some(condition) = rule.regex_pre_condition().value() {
        lexical_rule.precede_restriction = Some(Identifier {
            name: input.text(condition.identifier().span()),
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
        // #Paren is the parenthesized form: at the syntax level one shape,
        // a list of regex sequences separated by "|", covers both groups
        // and alternations. One sequence constructs a Seq. Several
        // construct an Alt: a one-regex sequence stays as is, and a longer
        // one is wrapped in a Seq, because Alt holds a list of regexes,
        // not a list of sequences.
        parse_tree::Regex::Paren { seqs, .. } => {
            let mut seqs: Vec<Vec<Regex>> = seqs
                .regexes()
                .map(|seq| seq.map(|r| convert_regex(r, input)).collect())
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
            let raw = input.text(char.span());
            Regex::Char(parse_char(&raw[1..raw.len() - 1]))
        }
        parse_tree::Regex::String { string, .. } => {
            let raw = input.text(string.span());
            let unescaped = unescape_string(&raw[1..raw.len() - 1]);
            let regexes = unescaped.chars().map(Regex::Char).collect();
            Regex::Seq(regexes)
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
                let start = parse_range_char(&input.text(range.start().span()));
                let end = parse_range_char(&input.text(range.end().span()));
                CharRange { start, end }
            } else if let Some(range_char) = e.as_range_char() {
                let ch = parse_range_char(&input.text(range_char.span()));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The first symbol of the first alternative, for a grammar with one
    /// rule.
    fn first_symbol(source: &str) -> Symbol {
        let grammar = parse_grammar(source).expect("the grammar should parse");
        grammar.syntax_rules[0].priority_levels[0].alternatives[0].symbols[0].clone()
    }

    fn names(ids: &[Identifier]) -> Vec<&str> {
        ids.iter().map(|id| id.name.as_str()).collect()
    }

    fn restrictions(symbol: &Symbol) -> &Restrictions {
        match symbol {
            Symbol::Restricted { restrictions, .. } => restrictions,
            other => panic!("expected a restricted symbol, got {other:?}"),
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

    #[test]
    fn test_restriction_written_twice_is_recorded_once() {
        let symbol = first_symbol("grammar g S = A !>> B \\ K !>> B C");
        assert_eq!(names(&restrictions(&symbol).follow), ["B"]);
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
}
