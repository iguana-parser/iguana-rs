//! The escape sequences of Iggy string literals, character literals, and
//! character classes.

use std::{fmt::Write, ops::Range};

/// Replaces each escape in `chars` with the character it represents
/// (`decode_next`). An invalid escape is replaced with U+FFFD, the replacement
/// character, as a placeholder (`build_grammar`).
pub(crate) fn unescape(mut chars: &[char]) -> String {
    let mut text = String::new();
    while !chars.is_empty() {
        text.push(decode_next(&mut chars).unwrap_or(char::REPLACEMENT_CHARACTER));
    }
    text
}

/// The positions of the invalid `\u{...}` escapes in `chars`
/// (`validate_escapes`). `chars` is as in `decode_next`.
pub(crate) fn invalid_escapes(chars: &[char]) -> Vec<Range<usize>> {
    let mut rest = chars;
    let mut invalid = Vec::new();
    while !rest.is_empty() {
        let start = chars.len() - rest.len();
        if decode_next(&mut rest).is_none() {
            invalid.push(start..chars.len() - rest.len());
        }
    }
    invalid
}

/// Tries to decode the escape sequence at the start of `chars`, and returns
/// `None` if the escape is invalid. If `chars` does not start with an escape,
/// returns its first character. `chars` is updated to point after the
/// character or the escape.
///
/// - `\n`, `\r`, `\t`, and `\f` represent a newline, a carriage return, a tab,
///   and a form feed.
/// - `\u{...}` represents the character with the code point between the
///   braces, written in one to six hexadecimal digits.
/// - Any other escape represents the character after the backslash. `\"` is
///   `"`, and `\-` is `-`.
///
/// `chars` must not be empty, and a `\u` in `chars` must start a complete
/// escape: `\u{`, hexadecimal digits, and `}`. The lexer guarantees the escape
/// in the text of a `String`, `Char`, or `RangeChar` token.
pub(crate) fn decode_next(chars: &mut &[char]) -> Option<char> {
    let (c, len) = match **chars {
        ['\\', 'n', ..] => (Some('\n'), 2),
        ['\\', 'r', ..] => (Some('\r'), 2),
        ['\\', 't', ..] => (Some('\t'), 2),
        ['\\', 'f', ..] => (Some('\u{C}'), 2),
        ['\\', 'u', ..] => {
            let len = chars.iter().position(|&c| c == '}').unwrap() + 1;
            let hex: String = chars[3..len - 1].iter().collect();
            // An escape is invalid if it has more than six digits
            // or if its code point is not a Unicode scalar value.
            let c = if hex.len() > 6 {
                None
            } else {
                // `char::from_u32` returns `None` for a surrogate and for a code
                // point above `10FFFF`.
                u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
            };
            (c, len)
        }
        ['\\', c, ..] => (Some(c), 2),
        [c, ..] => (Some(c), 1),
        [] => unreachable!("`chars` must not be empty"),
    };
    *chars = &chars[len..];
    c
}

/// Escapes text for Iggy string literals with the escapes `\"`, `\\`, `\n`,
/// `\t`, `\r`, and `\f`. Any other control character is written as a `\u{...}`
/// escape in lowercase hexadecimal digits, the case that `char::escape_debug`
/// uses, such as `\u{1b}` for ESC. Other characters are written as they are.
///
/// Every place a literal's text is presented goes through this escaping:
/// terminal names (parse-tree labels, parse errors) and the Display of
/// literal symbols (printed grammars). The presentation text is
/// syntactically valid Iggy notation, so a literal read from a parse tree,
/// an error message, or a printed grammar can be pasted back into a
/// grammar.
pub fn escape_literal(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            '\u{C}' => escaped.push_str("\\f"),
            c if c.is_control() => {
                let _ = write!(escaped, "\\u{{{:x}}}", c as u32);
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_control_character_without_a_named_escape_is_a_unicode_escape() {
        let text = "\u{0}a\u{1B}\u{7F}\n\u{C}é";
        let escaped = escape_literal(text);
        assert_eq!(escaped, r"\u{0}a\u{1b}\u{7f}\n\fé");
        let chars: Vec<char> = escaped.chars().collect();
        assert_eq!(unescape(&chars), text);
    }
}
