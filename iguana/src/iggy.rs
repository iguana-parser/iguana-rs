use iggy::{
    parse_tree,
    parse_tree::{IggyParseTreeBuilder, OptNode, ParseTree, StartGrammar, create_parse_tree},
    parser::IggyParser,
};
use iguana_runtime::{
    input::Input,
    parser::{ParseResult, Parser},
    sppf::Span,
};

use crate::grammar::{
    def::{Alternative, GrammarDef, LexicalRule, PriorityLevel, SyntaxRule},
    regex::{CharClass, CharRange, Regex},
    symbols::{Identifier, Nonterminal, Symbol, Terminal},
};

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

pub fn parse_grammar(source: &str) -> Result<GrammarDef, Error> {
    let input = Input::from(source);
    let parse_tree = parse(&input)?;
    build_grammar(&parse_tree, &input)
}

fn parse(input: &Input) -> Result<StartGrammar, Error> {
    let start_nonterminal_name = "StartGrammar";
    let start_nonterminal_id = IggyParser::nonterminal_id(start_nonterminal_name).unwrap();
    let mut parser = IggyParser::new(input, start_nonterminal_id);
    let result = parser.run();
    match result {
        ParseResult::Success(success) => {
            let parse_tree = create_parse_tree(
                success.sppf_node_id,
                start_nonterminal_name,
                &parser,
                &IggyParseTreeBuilder,
            );
            let ParseTree::StartGrammar(start_grammar) = parse_tree else {
                unreachable!()
            };
            Ok(start_grammar)
        }
        ParseResult::Failure() => Err(Error {
            message: "Parse error".into(),
        }),
    }
}

fn text(input: &Input, span: Span) -> String {
    input.substring(span.left_extent, span.right_extent)
}

fn build_grammar(
    start_grammar: &parse_tree::StartGrammar,
    input: &Input,
) -> Result<GrammarDef, Error> {
    let grammar = &start_grammar.start;
    let name = text(input, grammar.name.span());

    let syntax_rules: Vec<SyntaxRule> = grammar
        .syntax_rules
        .syntax_rules()
        .map(|r| convert_syntax_rule(r, input))
        .collect();

    let lexical_rules: Vec<LexicalRule> = grammar
        .regex_block
        .value()
        .map(|block| convert_regex_block(block, input))
        .unwrap_or_default();

    Ok(GrammarDef {
        name,
        syntax_rules,
        lexical_rules,
        layout_def: vec![],
    })
}

fn convert_syntax_rule(rule: &parse_tree::SyntaxRule, input: &Input) -> SyntaxRule {
    let head_name = text(input, rule.head.span());
    let head = Nonterminal::new(head_name);

    let priority_levels: Vec<PriorityLevel> = rule
        .priority_levels
        .priority_levels()
        .map(|level| convert_priority_level(level, input))
        .collect();

    SyntaxRule::new(head, priority_levels)
}

fn convert_priority_level(level: &parse_tree::PriorityLevel, input: &Input) -> PriorityLevel {
    let alternatives: Vec<Alternative> = level
        .alternatives
        .alternatives()
        .map(|alt| convert_alternative(alt, input))
        .collect();

    PriorityLevel::new(alternatives)
}

fn convert_alternative(alt: &parse_tree::Alternative, input: &Input) -> Alternative {
    let symbols: Vec<Symbol> = alt
        .symbols
        .symbols()
        .map(|sym| convert_symbol(sym, input))
        .collect();

    Alternative {
        symbols,
        label: None,
    }
}

fn convert_symbol(symbol: &parse_tree::Symbol, input: &Input) -> Symbol {
    match symbol {
        parse_tree::Symbol::Star { symbol_0, .. } => {
            Symbol::Star(Box::new(convert_symbol(symbol_0, input)), None)
        }
        parse_tree::Symbol::Plus { symbol_0, .. } => {
            Symbol::Plus(Box::new(convert_symbol(symbol_0, input)), None)
        }
        parse_tree::Symbol::Opt { symbol_0, .. } => {
            Symbol::Opt(Box::new(convert_symbol(symbol_0, input)))
        }
        parse_tree::Symbol::Alt {
            symbol_2,
            symbol_6,
            ..
        } => Symbol::Alt(vec![
            convert_symbol(symbol_2, input),
            convert_symbol(symbol_6, input),
        ]),
        parse_tree::Symbol::Lit { string_2, .. } => Symbol::Literal(text(input, string_2.span())),
        parse_tree::Symbol::StarSep {
            symbol_2,
            symbol_4,
            ..
        } => Symbol::Star(
            Box::new(convert_symbol(symbol_2, input)),
            Some(Box::new(convert_symbol(symbol_4, input))),
        ),
        parse_tree::Symbol::PlusSep {
            symbol_2,
            symbol_4,
            ..
        } => Symbol::Plus(
            Box::new(convert_symbol(symbol_2, input)),
            Some(Box::new(convert_symbol(symbol_4, input))),
        ),
        parse_tree::Symbol::Group { symbols, .. } => {
            Symbol::Group(symbols.symbols().map(|s| convert_symbol(s, input)).collect())
        }
        parse_tree::Symbol::Identifier { identifier_0, .. } => Symbol::Identifier(Identifier {
            name: text(input, identifier_0.span()),
            definition: None,
        }),
    }
}

fn convert_regex_block(block: &parse_tree::RegexBlock, input: &Input) -> Vec<LexicalRule> {
    block
        .regex_rules
        .regex_rules()
        .map(|rule| convert_regex_rule(rule, input))
        .collect()
}

fn convert_regex_rule(rule: &parse_tree::RegexRule, input: &Input) -> LexicalRule {
    let name = text(input, rule.identifier_0.span());
    let head = Terminal::new(name);

    let alternatives = collect_regex_alternatives(&rule.regex_rule_plus_3_4, input);

    let regex = if alternatives.len() == 1 {
        alternatives.into_iter().next().unwrap()
    } else {
        Regex::Alt(alternatives)
    };

    LexicalRule { head, regex }
}

fn collect_regex_alternatives(
    plus3: &parse_tree::RegexRulePlus3,
    input: &Input,
) -> Vec<Regex> {
    match plus3 {
        parse_tree::RegexRulePlus3::Alt0 {
            regex_rule_plus_3_0,
            regexes,
            ..
        } => {
            let mut alts = collect_regex_alternatives(regex_rule_plus_3_0, input);
            alts.push(collect_regex_sequence(regexes, input));
            alts
        }
        parse_tree::RegexRulePlus3::Alt1 { regexes, .. } => {
            vec![collect_regex_sequence(regexes, input)]
        }
    }
}

fn collect_regex_sequence(plus4: &parse_tree::RegexRulePlus4, input: &Input) -> Regex {
    let regexes: Vec<Regex> = plus4.regexes().map(|r| convert_regex(r, input)).collect();

    if regexes.len() == 1 {
        regexes.into_iter().next().unwrap()
    } else {
        Regex::Seq(regexes)
    }
}

fn convert_regex(regex: &parse_tree::Regex, input: &Input) -> Regex {
    match regex {
        parse_tree::Regex::Plus { regex_0, .. } => {
            Regex::Plus(Box::new(convert_regex(regex_0, input)))
        }
        parse_tree::Regex::Star { regex_0, .. } => {
            Regex::Star(Box::new(convert_regex(regex_0, input)))
        }
        parse_tree::Regex::Opt { regex_0, .. } => {
            Regex::Opt(Box::new(convert_regex(regex_0, input)))
        }
        parse_tree::Regex::Alt { regex_star_5_2, .. } => {
            let alternatives = collect_regex_star5_alternatives(regex_star_5_2, input);
            if alternatives.is_empty() {
                Regex::Epsilon
            } else if alternatives.len() == 1 {
                alternatives.into_iter().next().unwrap()
            } else {
                Regex::Alt(alternatives)
            }
        }
        parse_tree::Regex::CharClass { char_class_0, .. } => convert_char_class(char_class_0, input),
        parse_tree::Regex::Char { char_2, .. } => {
            Regex::Char(parse_char(&text(input, char_2.span())))
        }
    }
}

fn collect_regex_star5_alternatives(
    star5: &parse_tree::RegexStar5,
    input: &Input,
) -> Vec<Regex> {
    match star5.regex_opt_6_0.value() {
        Some(plus3) => collect_regex_alternatives(plus3, input),
        None => vec![],
    }
}

fn convert_char_class(char_class: &parse_tree::CharClass, input: &Input) -> Regex {
    let negated = char_class.char_class_opt_7_0.value().is_some();
    let ranges = collect_char_class_ranges(&char_class.char_class_plus_7_4, input);

    Regex::CharClass(CharClass { ranges, negated })
}

fn collect_char_class_ranges(plus7: &parse_tree::CharClassPlus7, input: &Input) -> Vec<CharRange> {
    match plus7 {
        parse_tree::CharClassPlus7::Alt0 {
            char_class_plus_7_0,
            char_class_alt_0_2,
            ..
        } => {
            let mut ranges = collect_char_class_ranges(char_class_plus_7_0, input);
            if let Some(range) = convert_char_class_alt0(char_class_alt_0_2, input) {
                ranges.push(range);
            }
            ranges
        }
        parse_tree::CharClassPlus7::Alt1 {
            char_class_alt_0_0, ..
        } => {
            let mut ranges = Vec::new();
            if let Some(range) = convert_char_class_alt0(char_class_alt_0_0, input) {
                ranges.push(range);
            }
            ranges
        }
    }
}

fn convert_char_class_alt0(
    alt0: &parse_tree::CharClassAlt0,
    input: &Input,
) -> Option<CharRange> {
    match alt0 {
        parse_tree::CharClassAlt0::Alt0 { range_0, .. } => {
            let start = parse_range_char(&text(input, range_0.range_char_0.span()));
            let end = parse_range_char(&text(input, range_0.range_char_4.span()));
            Some(CharRange { start, end })
        }
        parse_tree::CharClassAlt0::Alt1 { range_char_0, .. } => {
            let ch = parse_range_char(&text(input, range_char_0.span()));
            Some(CharRange { start: ch, end: ch })
        }
    }
}

fn parse_char(s: &str) -> char {
    if s.starts_with('\\') && s.len() > 1 {
        match s.chars().nth(1) {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
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
