use iggy::{
    highlighting::{Group, rules, token_rules},
    parse_tree::{Grammar, Layout, Start, Token},
    scanner::IggyScanner,
};
use iguana_compiler::{grammar::def::GrammarDef, spans::GrammarSpans};
use iguana_runtime::{
    arena::Arena,
    highlight::{Highlight, flatten, group_name, highlight, lexical_highlight},
    input::{Input, Span},
    utf16::Utf16Columns,
};
use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};

use crate::name_resolution::NameResolutionIndex;

/// The legend: one token type per group of `Group::ALL`, in that order
/// (`token_type`). The VS Code extension's `package.json` declares the types
/// that are not standard.
pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: Group::ALL
            .iter()
            .map(|&group| SemanticTokenType::from(token_type_name(&group_name(group))))
            .collect(),
        token_modifiers: vec![],
    }
}

/// The LSP token type of a group: its segments joined, each one after the
/// first capitalized, such as `stringEscape` for `string.escape`.
fn token_type_name(group: &str) -> String {
    let mut segments = group.split('.');
    let mut name = segments.next().unwrap_or_default().to_string();
    for segment in segments {
        let mut chars = segment.chars();
        name.extend(chars.next().map(|c| c.to_ascii_uppercase()));
        name.extend(chars);
    }
    name
}

/// The token type of `group`, its index in `Group::ALL`.
pub(crate) fn token_type(group: Group) -> u32 {
    Group::ALL.iter().position(|&known| known == group).unwrap() as u32
}

/// The semantic tokens of a parse tree. Iggy's rules (`iggy::highlighting`)
/// highlight the tree, and a second pass gives each rule name the group of its
/// rule (`name_groups`).
pub(crate) fn semantic_tokens<'a>(
    tree: &Start<&Grammar<'_>, &Layout<'_>>,
    input: &Input,
    columns: &Utf16Columns,
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    names: &NameResolutionIndex,
) -> Vec<SemanticToken> {
    let mut highlights = Vec::new();
    for child in [
        tree.before.as_parse_tree(),
        tree.node.as_parse_tree(),
        tree.after.as_parse_tree(),
    ] {
        highlights.extend(highlight(child, input, rules));
    }
    // A name's group comes after the tree's for the same span, and wins.
    highlights.extend(name_groups(grammar_def, spans, names));
    let mut builder = SemanticTokensBuilder::new();
    for highlight in flatten(&highlights) {
        builder.push_span(highlight.span, highlight.group, input, columns);
    }
    builder.build()
}

/// The highlights of the rule names: each head and resolved reference with
/// the group of its rule, and each name that does not resolve as
/// `Unresolved`.
fn name_groups<'a>(
    grammar_def: &'a GrammarDef,
    spans: &GrammarSpans<'a>,
    names: &NameResolutionIndex,
) -> Vec<Highlight<Group>> {
    let rule_group = |definition| {
        if names.is_lexical_rule(definition) {
            Group::RuleLexical
        } else {
            Group::RuleSyntax
        }
    };
    let mut groups: Vec<_> = names
        .rule_names()
        .iter()
        .map(|&(span, definition)| Highlight::new(span, rule_group(definition)))
        .collect();

    grammar_def.for_each_identifier(&mut |identifier| {
        if identifier.definition.is_none() {
            groups.push(Highlight::new(
                spans.identifier(identifier).span,
                Group::Unresolved,
            ));
        }
    });

    // A lexical rule's own restrictions, such as `\ Keyword`, have no
    // definition yet, because the compiler resolves them after validation,
    // and `for_each_identifier` does not visit them. This loop looks up their
    // names in the symbol table.
    let symbol_table = grammar_def.symbol_table();
    for rule in &grammar_def.lexical_rules {
        let restrictions = rule
            .except
            .iter()
            .chain(&rule.follow_restriction)
            .chain(&rule.precede_restriction);
        for identifier in restrictions {
            let group = symbol_table
                .get(&identifier.name)
                .map_or(Group::Unresolved, rule_group);
            groups.push(Highlight::new(spans.identifier(identifier).span, group));
        }
    }
    groups
}

/// The semantic tokens of a source, calculated only with the scanner, because
/// the parse failed or the parse tree is ambiguous (`lexical_highlight`).
pub(crate) fn semantic_tokens_from_scanner(
    input: &Input,
    columns: &Utf16Columns,
) -> Vec<SemanticToken> {
    let arena = Arena::new();
    let mut scanner = IggyScanner::new(input, &arena);
    let mut builder = SemanticTokensBuilder::new();
    let highlights = lexical_highlight(&mut scanner, input, Token::new, token_rules);
    for highlight in highlights {
        builder.push_span(highlight.span, highlight.group, input, columns);
    }
    builder.build()
}

struct SemanticTokensBuilder {
    prev_line: u32,
    prev_char: u32,
    data: Vec<SemanticToken>,
}

impl SemanticTokensBuilder {
    fn new() -> Self {
        SemanticTokensBuilder {
            prev_line: 0,
            prev_char: 0,
            data: Vec::new(),
        }
    }

    /// Pushes one token of `group` per line of `span`, because an LSP client
    /// does not have to support a token that spans lines. A string literal
    /// can contain a newline.
    fn push_span(&mut self, span: Span, group: Group, input: &Input, columns: &Utf16Columns) {
        let token_type = token_type(group);
        let mut start = span.left_extent;
        while start < span.right_extent {
            let end = input.line_end(start).min(span.right_extent);
            if start < end {
                let (line, column) = input.line_column(start);
                let utf16_start = columns.utf16_column(line, column);
                let utf16_end = columns.utf16_column(line, column + (end - start));
                self.push(line, utf16_start, utf16_end - utf16_start, token_type);
            }
            start = end + 1;
        }
    }

    /// Pushes a token at `line` and the column in UTF-16 `column`, `length`
    /// code units long, and encodes it relative to the previous token.
    fn push(&mut self, line: u32, column: u32, length: u32, token_type: u32) {
        let mut delta_line = line;
        let mut delta_start = column;

        if !self.data.is_empty() {
            delta_line -= self.prev_line;
            if delta_line == 0 {
                delta_start -= self.prev_char;
            }
        }

        self.data.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset: 0,
        });

        self.prev_line = line;
        self.prev_char = column;
    }

    fn build(self) -> Vec<SemanticToken> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::Analysis;

    fn tokenize(source: &str) -> Vec<SemanticToken> {
        Analysis::new(source).semantic_tokens().to_vec()
    }

    /// The text and group of each token of `source`, in order. A token's
    /// column and length are in UTF-16 code units.
    fn tokens_of(source: &str) -> Vec<(String, String)> {
        let lines: Vec<Vec<u16>> = source
            .split('\n')
            .map(|line| line.encode_utf16().collect())
            .collect();
        let (mut line, mut column) = (0, 0);
        tokenize(source)
            .iter()
            .map(|token| {
                if token.delta_line > 0 {
                    column = 0;
                }
                line += token.delta_line;
                column += token.delta_start;
                let units = &lines[line as usize][column as usize..][..token.length as usize];
                (
                    String::from_utf16(units).unwrap(),
                    group_name(Group::ALL[token.token_type as usize]),
                )
            })
            .collect()
    }

    fn expected(tokens: &[(&str, &str)]) -> Vec<(String, String)> {
        tokens
            .iter()
            .map(|&(text, group)| (text.to_string(), group.to_string()))
            .collect()
    }

    /// The tokens of `source` whose text is one of `texts`.
    fn tokens_with_text(source: &str, texts: &[&str]) -> Vec<(String, String)> {
        tokens_of(source)
            .into_iter()
            .filter(|(text, _)| texts.contains(&text.as_str()))
            .collect()
    }

    #[test]
    fn a_group_of_several_segments_is_one_token_type() {
        assert_eq!(token_type_name("keyword"), "keyword");
        assert_eq!(token_type_name("string.escape"), "stringEscape");
        assert_eq!(token_type_name("rule.lexical"), "ruleLexical");
    }

    #[test]
    fn test_simple_grammar() {
        let tokens = tokenize("grammar Test\n\nRule\n  = \"hello\"\n");
        assert!(!tokens.is_empty());

        // First token should be "grammar" keyword at line 0, col 0
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].delta_start, 0);
        assert_eq!(tokens[0].length, 7); // "grammar"
        assert_eq!(tokens[0].token_type, token_type(Group::Keyword));
    }

    // The tree's groups of Iggy tokens, and the rule names of the second
    // pass.
    #[test]
    fn a_grammar_has_the_tree_and_name_groups() {
        let source =
            "grammar G\n// c\n@Layout @Regex\nWS = [\\ ]*\nE = E \"+\" E #Add > left E \"*\" E";
        assert_eq!(
            tokens_of(source),
            expected(&[
                ("grammar", "keyword"),
                ("G", "rule"),
                ("// c", "comment"),
                ("@Layout", "decorator"),
                ("@Regex", "decorator"),
                ("WS", "rule.lexical"),
                ("=", "operator"),
                ("[", "operator"),
                ("\\ ", "string.escape"),
                ("]", "operator"),
                ("*", "operator"),
                ("E", "rule.syntax"),
                ("=", "operator"),
                ("E", "rule.syntax"),
                ("\"+\"", "string"),
                ("E", "rule.syntax"),
                ("#Add", "label"),
                (">", "operator"),
                ("left", "keyword"),
                ("E", "rule.syntax"),
                ("\"*\"", "string"),
                ("E", "rule.syntax"),
            ])
        );
    }

    // A rule name has the group of its rule, in its head and in each
    // reference, including a reference before the rule.
    #[test]
    fn a_rule_name_has_the_group_of_its_rule() {
        let source = "grammar G\nS = Id Missing S\n@Regex\nId = [a-z]+";
        assert_eq!(
            tokens_with_text(source, &["S", "Id", "Missing"]),
            expected(&[
                ("S", "rule.syntax"),
                ("Id", "rule.lexical"),
                ("Missing", "unresolved"),
                ("S", "rule.syntax"),
                ("Id", "rule.lexical"),
            ])
        );
    }

    // In a character class, the negation `!`, the brackets, and the `-` of a
    // range are operators. The name before `:` in a symbol is a field,
    // highlighted as `property`.
    #[test]
    fn a_negation_is_an_operator_and_a_field_is_a_property() {
        let source = "grammar G\nS = lhs:S Id\n@Regex\nId = ![a-z]";
        assert_eq!(
            tokens_of(source),
            expected(&[
                ("grammar", "keyword"),
                ("G", "rule"),
                ("S", "rule.syntax"),
                ("=", "operator"),
                ("lhs", "property"),
                (":", "operator"),
                ("S", "rule.syntax"),
                ("Id", "rule.lexical"),
                ("@Regex", "decorator"),
                ("Id", "rule.lexical"),
                ("=", "operator"),
                ("!", "operator"),
                ("[", "operator"),
                ("a", "regexp"),
                ("-", "operator"),
                ("z", "regexp"),
                ("]", "operator"),
            ])
        );
    }

    // The restrictions of a lexical rule refer to rules too, and the name
    // after `!` in an exclusion is a label.
    #[test]
    fn restriction_operands_are_rules_and_an_exclusion_operand_is_a_label() {
        let source = "grammar G\nE = E!Add \"+\" #Add | Id\n@Regex\nId = Kw !<< [a]+ \\ Kw \\ Nope\n@Regex\nKw = \"if\"";
        assert_eq!(
            tokens_with_text(source, &["Add", "Kw", "Nope"]),
            expected(&[
                ("Add", "label"),
                ("Kw", "rule.lexical"),
                ("Kw", "rule.lexical"),
                ("Nope", "unresolved"),
                ("Kw", "rule.lexical"),
            ])
        );
    }

    // An escape sequence in a string has its own group, with the tree and
    // without it.
    #[test]
    fn an_escape_in_a_string_has_its_own_group() {
        for source in ["grammar G\nS = \"a\\\"b\"", "S = \"a\\\"b\""] {
            assert_eq!(
                tokens_with_text(source, &["\"a", "\\\"", "b\""]),
                expected(&[
                    ("\"a", "string"),
                    ("\\\"", "string.escape"),
                    ("b\"", "string"),
                ]),
                "{source}"
            );
        }
    }

    // The prefix of an `i` literal and its string are two string tokens, with
    // the tree and without it.
    #[test]
    fn an_ignore_case_literal_is_a_string() {
        for source in ["grammar G\nS = i\"select\" x", "S = i\"select\" x"] {
            assert_eq!(
                tokens_with_text(source, &["i", "\"select\""]),
                expected(&[("i", "string"), ("\"select\"", "string")]),
                "{source}"
            );
        }
    }

    // A column and a length are in UTF-16 code units. The emoji is one
    // character and two code units, so the tokens after it are one column
    // further than in characters.
    #[test]
    fn a_column_is_in_utf16_code_units() {
        let source = "grammar G\nS = \"😀\" #A | \"x\" // 😀😀 end";
        assert_eq!(
            tokens_of(source)[2..],
            expected(&[
                ("S", "rule.syntax"),
                ("=", "operator"),
                ("\"😀\"", "string"),
                ("#A", "label"),
                ("|", "operator"),
                ("\"x\"", "string"),
                ("// 😀😀 end", "comment"),
            ])
        );
    }

    // Without a tree, `token_rules` highlights the scanner's tokens
    // (`Scanner::tokens`). The source does not parse, because its `grammar`
    // header is missing. A character class has no `regexp` without the tree.
    #[test]
    fn a_source_that_does_not_parse_has_the_scanner_tokens() {
        let source = "E = a \"+\" grammar [b] // c";
        assert_eq!(
            tokens_of(source),
            expected(&[
                ("E", "rule"),
                ("=", "operator"),
                ("a", "rule"),
                ("\"+\"", "string"),
                ("grammar", "keyword"),
                ("[", "operator"),
                ("b", "rule"),
                ("]", "operator"),
                ("// c", "comment"),
            ])
        );
    }

    #[test]
    fn multiline_literal_has_one_token_per_line() {
        let tokens = tokenize("grammar T\nS = \"abc\n\nx\" | \"y\"\n");
        let strings: Vec<_> = tokens
            .iter()
            .filter(|token| token.token_type == token_type(Group::String))
            .map(|token| (token.delta_line, token.delta_start, token.length))
            .collect();
        // The deltas are relative to the previous token, which is `=` before
        // `"abc` and `|` before `"y"`. The empty line in the literal does not
        // have a token.
        assert_eq!(strings, [(0, 2, 4), (2, 0, 2), (0, 2, 3)]);
    }

    #[test]
    fn test_empty_returns_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }
}
