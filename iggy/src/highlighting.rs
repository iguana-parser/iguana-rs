//! The highlighting rules of Iggy (`iguana_runtime::highlight`). The
//! generated `lib.rs` declares this file as the module `highlighting`.

use iguana_runtime::{
    highlight::{Highlight, backslash_escapes},
    input::Input,
    serde::Serialize,
};

use crate::parse_tree::{ParseTree, PostCondition, Symbol, Token, TokenKind};

/// The highlight groups of Iggy. The rules in this file use all of them
/// except three, which need name resolution. The language server assigns
/// those in a second pass: it highlights the name of a syntax rule as
/// `RuleSyntax`, the name of a lexical rule as `RuleLexical`, and a name that
/// does not resolve as `Unresolved`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(crate = "iguana_runtime::serde", rename_all = "lowercase")]
pub enum Group {
    Keyword,
    Operator,
    Comment,
    Decorator,
    String,
    #[serde(rename = "string.escape")]
    StringEscape,
    Regexp,
    Label,
    Property,
    Rule,
    #[serde(rename = "rule.syntax")]
    RuleSyntax,
    #[serde(rename = "rule.lexical")]
    RuleLexical,
    Unresolved,
}

impl Group {
    pub const ALL: [Group; 13] = [
        Group::Keyword,
        Group::Operator,
        Group::Comment,
        Group::Decorator,
        Group::String,
        Group::StringEscape,
        Group::Regexp,
        Group::Label,
        Group::Property,
        Group::Rule,
        Group::RuleSyntax,
        Group::RuleLexical,
        Group::Unresolved,
    ];
}

/// Pushes the highlights of `token` from its terminal alone, for
/// `lexical_highlight` and for each token in `rules`.
///
/// A token kind is the id of its terminal. The test
/// `the_token_kinds_are_the_terminals_of_the_rules` checks their names.
pub fn token_rules(token: Token, input: &Input, highlights: &mut Vec<Highlight<Group>>) {
    match token.kind {
        // "grammar", "left", "right", "none"
        TokenKind::T11 | TokenKind::T22 | TokenKind::T23 | TokenKind::T24 => {
            push(highlights, token, Group::Keyword)
        }
        // "=", ">", "|", "!<<", "\\", "!>>", "(", ")", "{", "}", "*", "+", "?", ":",
        // "!>>>", "!", "[", "]", "-"
        TokenKind::T12
        | TokenKind::T13
        | TokenKind::T18
        | TokenKind::T19
        | TokenKind::T20
        | TokenKind::T21
        | TokenKind::T25
        | TokenKind::T26
        | TokenKind::T27
        | TokenKind::T28
        | TokenKind::T29
        | TokenKind::T30
        | TokenKind::T31
        | TokenKind::T32
        | TokenKind::T33
        | TokenKind::T34
        | TokenKind::T36
        | TokenKind::T37
        | TokenKind::T38 => push(highlights, token, Group::Operator),
        // "@NoLayout", "@Layout", "@Identifier", "@Regex"
        TokenKind::T14 | TokenKind::T15 | TokenKind::T16 | TokenKind::T17 => {
            push(highlights, token, Group::Decorator)
        }
        // Identifier
        TokenKind::T2 => push(highlights, token, Group::Rule),
        // String, Char
        TokenKind::T3 | TokenKind::T4 => {
            let span = token.span();
            highlights.push(Highlight::new(span, Group::String));
            for escape in backslash_escapes(span, input.chars(span)) {
                highlights.push(Highlight::new(escape, Group::StringEscape));
            }
        }
        // "i", the prefix of an `IgnoreCaseString` such as `i"select"`
        TokenKind::T35 => push(highlights, token, Group::String),
        // Label
        TokenKind::T7 => push(highlights, token, Group::Label),
        // LineComment
        TokenKind::T10 => push(highlights, token, Group::Comment),
        _ => {}
    }
}

/// Pushes the highlights of `node` of a parse tree.
pub fn rules(node: ParseTree<'_>, input: &Input, highlights: &mut Vec<Highlight<Group>>) {
    match node {
        ParseTree::Token(token) => {
            token_rules(token, input, highlights);
            // RangeChar, a character of a character class, such as `a` in
            // `[a-z]`. Only the tree colors it, because it matches almost any
            // character and the scan would color stray characters.
            if let TokenKind::T6 = token.kind {
                push(highlights, token, range_char_group(token, input));
            }
        }
        // The field in `lhs:Expr`. The symbol's rule runs after the name's,
        // and overrides `Rule`.
        ParseTree::Symbol(Symbol::Labeled { label, .. }) => {
            push(highlights, *label, Group::Property)
        }
        // The label in the exclusion `Expr!Add`.
        ParseTree::PostCondition(PostCondition::Exclude { identifier, .. }) => {
            push(highlights, *identifier, Group::Label)
        }
        _ => {}
    }
}

fn push(highlights: &mut Vec<Highlight<Group>>, token: Token, group: Group) {
    highlights.push(Highlight::new(token.span(), group));
}

/// A `RangeChar` is one character or one escape, so a token that starts with a
/// backslash is an escape.
fn range_char_group(token: Token, input: &Input) -> Group {
    if input.chars(token.span()).first() == Some(&'\\') {
        Group::StringEscape
    } else {
        Group::Regexp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_kinds_are_the_terminals_of_the_rules() {
        let names = [
            (TokenKind::T2, "Identifier"),
            (TokenKind::T3, "String"),
            (TokenKind::T4, "Char"),
            (TokenKind::T6, "RangeChar"),
            (TokenKind::T7, "Label"),
            (TokenKind::T10, "LineComment"),
            (TokenKind::T11, r#""grammar""#),
            (TokenKind::T12, r#""=""#),
            (TokenKind::T13, r#"">""#),
            (TokenKind::T14, r#""@NoLayout""#),
            (TokenKind::T15, r#""@Layout""#),
            (TokenKind::T16, r#""@Identifier""#),
            (TokenKind::T17, r#""@Regex""#),
            (TokenKind::T18, r#""|""#),
            (TokenKind::T19, r#""!<<""#),
            (TokenKind::T20, r#""\\""#),
            (TokenKind::T21, r#""!>>""#),
            (TokenKind::T22, r#""left""#),
            (TokenKind::T23, r#""right""#),
            (TokenKind::T24, r#""none""#),
            (TokenKind::T25, r#""(""#),
            (TokenKind::T26, r#"")""#),
            (TokenKind::T27, r#""{""#),
            (TokenKind::T28, r#""}""#),
            (TokenKind::T29, r#""*""#),
            (TokenKind::T30, r#""+""#),
            (TokenKind::T31, r#""?""#),
            (TokenKind::T32, r#"":""#),
            (TokenKind::T33, r#""!>>>""#),
            (TokenKind::T34, r#""!""#),
            (TokenKind::T35, r#""i""#),
            (TokenKind::T36, r#""[""#),
            (TokenKind::T37, r#""]""#),
            (TokenKind::T38, r#""-""#),
        ];
        for (kind, name) in names {
            assert_eq!(kind.name(), name);
        }
    }

    // `ALL` lists each group once, at the position of its arm here. The match
    // has no wildcard, so a new group does not compile until it has an arm,
    // but the test does not check that `ALL` lists it.
    #[test]
    fn all_lists_each_group_once() {
        let position = |group| match group {
            Group::Keyword => 0,
            Group::Operator => 1,
            Group::Comment => 2,
            Group::Decorator => 3,
            Group::String => 4,
            Group::StringEscape => 5,
            Group::Regexp => 6,
            Group::Label => 7,
            Group::Property => 8,
            Group::Rule => 9,
            Group::RuleSyntax => 10,
            Group::RuleLexical => 11,
            Group::Unresolved => 12,
        };
        for (index, &group) in Group::ALL.iter().enumerate() {
            assert_eq!(position(group), index, "{group:?}");
        }
    }
}
