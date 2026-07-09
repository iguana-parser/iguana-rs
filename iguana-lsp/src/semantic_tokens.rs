use crate::{BuildResult, build};
use iggy::parse_tree::{Grammar, Layout, ParseTree, Start, Token};
use iguana_runtime::input::{Input, Span};
use lsp_types::{Position, Range, SemanticToken, SemanticTokenType, SemanticTokensLegend};

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
pub fn semantic_tokens(
    tree: &Start<&Grammar<'_>, &Layout<'_>>,
    input: &Input,
) -> Vec<SemanticToken> {
    let mut builder = SemanticTokensBuilder::new();

    for i in 0..tree.child_count() {
        if let Some(child) = tree.child(i) {
            visit(child, &mut |node| match node {
                ParseTree::Token(token) => {
                    if let Some(token_type) = classify_token(&token) {
                        builder.push(to_range(token.span(), input), token_type, 0);
                    }
                    false
                }
                _ => true,
            });
        }
    }

    builder.build()
}

/// Convenience: parse and tokenize in one call (for tests and simple consumers).
pub fn tokenize(source: &str) -> Vec<SemanticToken> {
    let input = Input::from(source);
    let tree_arena = iguana_runtime::parse_tree::Bump::new();
    match build(&input, &tree_arena) {
        BuildResult::Success { tree, .. } => semantic_tokens(tree, &input),
        BuildResult::Error { .. } | BuildResult::Ambiguous => vec![],
    }
}

fn to_range(span: Span, input: &Input) -> Range {
    let (start_line, start_col) = input.line_column(span.left_extent);
    let (end_line, end_col) = input.line_column(span.right_extent);
    Range {
        start: Position::new(start_line, start_col),
        end: Position::new(end_line, end_col),
    }
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
        "\"@NoLayout\"" | "\"@Layout\"" | "\"@WithLayout\"" | "\"@Start\"" | "\"@Regex\"" => {
            Some(TOKEN_DECORATOR)
        }
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

    fn push(&mut self, range: Range, token_type: u32, modifier_bitset: u32) {
        let mut push_line = range.start.line;
        let mut push_char = range.start.character;

        if !self.data.is_empty() {
            push_line -= self.prev_line;
            if push_line == 0 {
                push_char -= self.prev_char;
            }
        }

        let token_len = range.end.character - range.start.character;

        self.data.push(SemanticToken {
            delta_line: push_line,
            delta_start: push_char,
            length: token_len,
            token_type,
            token_modifiers_bitset: modifier_bitset,
        });

        self.prev_line = range.start.line;
        self.prev_char = range.start.character;
    }

    fn build(self) -> Vec<SemanticToken> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
