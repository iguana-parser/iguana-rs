use iggy::parse_tree::{Grammar, Layout, ParseTree, Start, Token};
use iguana_runtime::{
    input::{Input, Span},
    parse_tree::ParseTreeNode,
};
use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: vec![],
    }
}

const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,   // 0
    SemanticTokenType::TYPE,      // 1
    SemanticTokenType::STRING,    // 2
    SemanticTokenType::REGEXP,    // 3
    SemanticTokenType::OPERATOR,  // 4
    SemanticTokenType::DECORATOR, // 5
    SemanticTokenType::COMMENT,   // 6 - used for labels (#Name)
];

const TOKEN_KEYWORD: u32 = 0;
const TOKEN_TYPE: u32 = 1;
const TOKEN_STRING: u32 = 2;
const TOKEN_REGEXP: u32 = 3;
const TOKEN_OPERATOR: u32 = 4;
const TOKEN_DECORATOR: u32 = 5;
const TOKEN_LABEL: u32 = 6;

/// Extract semantic tokens from a parse tree.
pub(crate) fn semantic_tokens(
    tree: &Start<&Grammar<'_>, &Layout<'_>>,
    input: &Input,
) -> Vec<SemanticToken> {
    let mut builder = SemanticTokensBuilder::new();

    for child in [
        tree.before.as_parse_tree(),
        tree.node.as_parse_tree(),
        tree.after.as_parse_tree(),
    ] {
        visit(child, &mut |node| match node {
            ParseTree::Token(token) => {
                if let Some(token_type) = classify_token(&token) {
                    builder.push_span(token.span(), token_type, input);
                }
                false
            }
            _ => true,
        });
    }

    builder.build()
}

fn visit<'a, F>(node: ParseTree<'a>, f: &mut F)
where
    F: FnMut(ParseTree<'a>) -> bool,
{
    if f(node) {
        for child in node.children() {
            visit(child, f);
        }
    }
}

fn classify_token(token: &Token) -> Option<u32> {
    match token.kind.name() {
        // Keywords
        "Keyword" | "\"grammar\"" | "\"left\"" | "\"right\"" | "\"none\"" => Some(TOKEN_KEYWORD),
        // Decorators
        "\"@NoLayout\"" | "\"@Layout\"" | "\"@Identifier\"" | "\"@Regex\"" => Some(TOKEN_DECORATOR),
        // Identifier
        "Identifier" => Some(TOKEN_TYPE),
        // String and Char literals (now include quotes)
        "String" | "Char" => Some(TOKEN_STRING),
        // Label
        "Label" => Some(TOKEN_LABEL),
        // Operators
        "\"=\"" | "\">\"" | "\"|\"" | "\"\\\"" | "\"!>>\"" | "\"!<<\"" | "\"*\"" | "\"+\""
        | "\"?\"" | "\"!\"" | "\":\"" | "\"-\"" => Some(TOKEN_OPERATOR),
        // Punctuation
        "\"(\"" | "\")\"" | "\"{\"" | "\"}\"" => Some(TOKEN_OPERATOR),
        // Regex: character class brackets and RangeChar
        "\"[\"" | "\"]\"" | "RangeChar" => Some(TOKEN_REGEXP),
        // Whitespace, layout, comments, escape chars — skip
        "WS" | "Comment" | "EscapeChar" | "Layout" => None,
        _ => None,
    }
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

    /// Pushes one token per line of `span`, because an LSP client does not have
    /// to support a token that spans lines. A string literal can contain a
    /// newline.
    fn push_span(&mut self, span: Span, token_type: u32, input: &Input) {
        let mut start = span.left_extent;
        while start < span.right_extent {
            let end = input.line_end(start).min(span.right_extent);
            if start < end {
                let (line, column) = input.line_column(start);
                self.push(line, column, end - start, token_type);
            }
            start = end + 1;
        }
    }

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

    #[test]
    fn test_simple_grammar() {
        let tokens = tokenize("grammar Test\n\nRule\n  = \"hello\"\n");
        assert!(!tokens.is_empty());

        // First token should be "grammar" keyword at line 0, col 0
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].delta_start, 0);
        assert_eq!(tokens[0].length, 7); // "grammar"
        assert_eq!(tokens[0].token_type, TOKEN_KEYWORD);
    }

    #[test]
    fn multiline_literal_has_one_token_per_line() {
        let tokens = tokenize("grammar T\nS = \"abc\n\nx\" | \"y\"\n");
        let strings: Vec<_> = tokens
            .iter()
            .filter(|token| token.token_type == TOKEN_STRING)
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

    #[test]
    fn test_invalid_returns_empty() {
        let tokens = tokenize("not a valid grammar {{{");
        assert!(tokens.is_empty());
    }
}
