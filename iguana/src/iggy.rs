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
        parse_tree::Symbol::Lit { string, .. } => Symbol::Literal(text(input, string.span())),
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
        parse_tree::Symbol::Identifier { identifier, .. } => Symbol::Identifier(Identifier {
            name: text(input, identifier.span()),
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
    let name = text(input, rule.identifier.span());
    let head = Terminal::new(name);

    // TODO: add simplification rules to convert Alt(regex) to regex, and Seq(regex) to regex.
    let regex = Regex::Alt(
        rule.body
            .regexes()
            .map(|inner| Regex::Seq(inner.map(|r| convert_regex(r, input)).collect()))
            .collect(),
    );
    LexicalRule { head, regex }
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
        parse_tree::Regex::Char { char, .. } => Regex::Char(parse_char(&text(input, char.span()))),
        parse_tree::Regex::Group { regexes, .. } => {
            Regex::Seq(regexes.regexes().map(|r| convert_regex(r, input)).collect())
        }
    }
}

fn convert_char_class(char_class: &parse_tree::CharClass, input: &Input) -> Regex {
    let negated = char_class.neg.value().is_some();
    let ranges = char_class
        .ranges
        .range_elements()
        .map(|e| {
            if let Some(range) = e.as_range() {
                let start = parse_range_char(&text(input, range.start.span()));
                let end = parse_range_char(&text(input, range.end.span()));
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
