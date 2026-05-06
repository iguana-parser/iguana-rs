//! Post-process the token-stream string from `quote!` into something legible.
//!
//! `quote!`'s `to_string()` emits everything space-separated on a single line.
//! For small grammars that's already readable; for the Java grammar the parser
//! and parse-tree files end up with multi-hundred-thousand-character lines,
//! and any rustc warning prints back the entire line. This module restores
//! line breaks and rewrites a few pseudo-attributes our generators emit:
//!
//! - `#[comment = "..."]` (our pseudo-attribute, not real Rust) → `// ...`.
//! - `#[doc = r"..."]` / `#[doc = r#"..."#]` (quote!'s default rendering of
//!   `///` doc lines) → `/// ...`.
//! - `} impl` / `} fn` / `} pub fn` → newline before the keyword.
//!
//! `#[comment]` is not a real attribute, so any unrewritten occurrence is a
//! compile error. Run [`post_process`] on every generator output that goes to
//! disk.

use std::sync::LazyLock;

use regex::Regex;

static COMMENT_ATTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^#\s*\[\s*comment\s*=\s*"((?:[^"\\]|\\.)*)"\s*\]"#).unwrap());

static DOC_RAW_HASH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r##"^#\s*\[\s*doc\s*=\s*r#"(.*?)"#\s*\]"##).unwrap());

static DOC_RAW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^#\s*\[\s*doc\s*=\s*r"([^"]*)"\s*\]"#).unwrap());

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
            out.push_str(&format!("\n// {}\n", unescape(&caps[1])));
            i += caps.get(0).unwrap().end();
        } else if let Some(caps) = DOC_RAW_HASH_RE.captures(rest) {
            out.push_str(&format!("\n///{}\n", &caps[1]));
            i += caps.get(0).unwrap().end();
        } else if let Some(caps) = DOC_RAW_RE.captures(rest) {
            out.push_str(&format!("\n///{}\n", &caps[1]));
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
