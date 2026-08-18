//! Post-process the token-stream string from `quote!` into something legible.
//!
//! `quote!`'s `to_string()` emits everything space-separated on a single line.
//! For small grammars that's already readable; for the Java grammar the parser
//! and parse-tree files end up with multi-hundred-thousand-character lines,
//! and any rustc warning prints back the entire line. This module restores
//! line breaks and rewrites a few pseudo-attributes our generators emit:
//!
//! - `#[comment = "..."]` (our pseudo-attribute, not real Rust) → `// ...`,
//!   word-wrapped across several `// ` lines so a comment can be written as one
//!   plain string in the generator.
//! - `#[doc = r"..."]` / `#[doc = r#"..."#]` (quote!'s default rendering of
//!   `///` doc lines) and `#[doc = "..."]` (the rendering of an interpolated
//!   doc string) → `/// ...`.
//! - `} impl` / `} fn` / `} pub fn` → newline before the keyword.
//!
//! `#[comment]` is not a real attribute, so any unrewritten occurrence is a
//! compile error. Run [`post_process`] on every generator output that goes to
//! disk.

use std::sync::LazyLock;

use regex::Regex;

// `[^"\\]` matches newlines, so a comment string that spans several source
// lines is captured whole; `wrap_comment` then re-wraps it.
static COMMENT_ATTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^#\s*\[\s*comment\s*=\s*"((?:[^"\\]|\\.)*)"\s*\]"#).unwrap());

static DOC_RAW_HASH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r##"^#\s*\[\s*doc\s*=\s*r#"(.*?)"#\s*\]"##).unwrap());

static DOC_RAW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^#\s*\[\s*doc\s*=\s*r"([^"]*)"\s*\]"#).unwrap());

// A doc string interpolated into `quote!` (e.g. `#[doc = #text]`) renders as a
// regular escaped literal, not the raw literal `quote!` uses for `///` lines.
static DOC_STR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^#\s*\[\s*doc\s*=\s*"((?:[^"\\]|\\.)*)"\s*\]"#).unwrap());

// Anchored on `}` to distinguish item-level `impl` / `fn` from the same
// keywords appearing inline: `-> impl Trait` in return position, or `fn(...)`
// as a function-pointer type. Both are preceded by `>` or `:`, never by `}`.
static ITEM_BREAK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\}\s+(?:pub\s+)?(?:impl|fn)\b").unwrap());

/// Walks the input once. Bulk-copies until the next trigger byte (`#` or `}`),
/// then tries each rule against the suffix; the first match wins. If no rule
/// matches at a trigger, emits the trigger char and advances one byte.
pub fn post_process(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        match input[i..].bytes().position(|b| b == b'#' || b == b'}') {
            None => {
                out.push_str(&input[i..]);
                break;
            }
            Some(0) => {} // already at a trigger
            Some(n) => {
                out.push_str(&input[i..i + n]);
                i += n;
            }
        }
        let rest = &input[i..];
        if let Some(caps) = COMMENT_ATTR_RE.captures(rest) {
            wrap_comment(&mut out, &unescape(&caps[1]));
            i += caps.get(0).unwrap().end();
        } else if let Some(caps) = DOC_RAW_HASH_RE.captures(rest) {
            push_doc_line(&mut out, &caps[1]);
            i += caps.get(0).unwrap().end();
        } else if let Some(caps) = DOC_RAW_RE.captures(rest) {
            push_doc_line(&mut out, &caps[1]);
            i += caps.get(0).unwrap().end();
        } else if let Some(caps) = DOC_STR_RE.captures(rest) {
            push_doc_line(&mut out, &unescape(&caps[1]));
            i += caps.get(0).unwrap().end();
        } else if let Some(m) = ITEM_BREAK_RE.find(rest) {
            let after = m.as_str().trim_start_matches('}').trim_start();
            out.push_str(&format!("}}\n{after}"));
            i += m.end();
        } else {
            // `#` and `}` are ASCII, so a one-byte step is a valid char step.
            out.push(input.as_bytes()[i] as char);
            i += 1;
        }
    }
    out
}

/// Appends a `///` doc line to `out`. `quote!` separates tokens with spaces, so
/// a doc attribute arrives preceded by a stray space; dropping it and starting a
/// new line only when `out` is not already at a line start keeps consecutive doc
/// lines (each `#[doc = "..."]` renders separately) flush, with no blank line
/// between them.
///
/// The text goes out as it is, so a `#[doc]` written by hand in a generator
/// needs to include its own leading space. `quote!` already includes one when
/// it renders a real `///` line.
fn push_doc_line(out: &mut String, text: &str) {
    while out.ends_with(' ') {
        out.pop();
    }
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("///");
    out.push_str(text);
    out.push('\n');
}

/// Text-width budget for a wrapped `// ` comment line, not counting the
/// indentation rustfmt adds later. Most generated comments sit at shallow
/// indentation, so this keeps rendered lines near the 100-column target without
/// post_process knowing the eventual indent.
const COMMENT_WIDTH: usize = 90;

/// Emit `text` as one or more `// ` lines, wrapping on word boundaries near
/// `COMMENT_WIDTH`. A comment is written as one string in the generator, so this
/// breaks a long one into readable lines; a short one stays on a single line.
fn wrap_comment(out: &mut String, text: &str) {
    let mut col = 0;
    for word in text.split_whitespace() {
        if col != 0 && col + 1 + word.len() > COMMENT_WIDTH {
            col = 0;
        }
        if col == 0 {
            out.push_str("\n// ");
            col = word.len();
        } else {
            out.push(' ');
            col += 1 + word.len();
        }
        out.push_str(word);
    }
    out.push('\n');
}

/// The token-stream renders a `"`-string with its source-text escapes still
/// in place (`\"`, `\\`, `\n`, …). Line comments don't interpret escapes —
/// leaving them as-is would render `#[comment = "He said \"hi\""]` as
/// `// He said \"hi\"` (with literal backslashes) rather than `// He said "hi"`.
/// Decode them back to the real string value before splicing into the comment.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn consecutive_doc_lines_have_no_blank_between() {
        let tokens = quote! {
            /// first line
            /// second line
            struct X;
        };
        let out = post_process(&tokens.to_string());
        assert!(
            out.contains("/// first line\n/// second line\n"),
            "got: {out:?}"
        );
        assert!(
            !out.contains("\n\n///"),
            "blank line between doc lines: {out:?}"
        );
    }

    #[test]
    fn interpolated_doc_string_becomes_doc_line() {
        let text = " Parses `input` as `Grammar`, allocating the parse tree in `tree_arena`.";
        let tokens = quote! {
            #[doc = #text]
            pub fn parse() {}
        };
        let out = post_process(&tokens.to_string());
        assert!(
            out.contains("/// Parses `input` as `Grammar`"),
            "got: {out:?}"
        );
        assert!(!out.contains("#[doc"), "unrewritten doc attr: {out:?}");
    }

    #[test]
    fn wraps_a_multiline_comment_string() {
        // A comment string split across source lines (no `\` continuation) is
        // captured whole and collapses to wrapped `// ` lines.
        let tokens = quote! {
            #[comment = "alpha beta gamma
                delta epsilon zeta"]
            struct X;
        };
        let out = post_process(&tokens.to_string());
        assert!(
            out.contains("// alpha beta gamma delta epsilon zeta"),
            "got: {out:?}"
        );
    }
}
