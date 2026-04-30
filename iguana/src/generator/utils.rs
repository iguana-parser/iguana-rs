use std::borrow::Cow;

use crate::grammar::def::Alternative;

pub fn alternative_label(alternative: &Alternative, index: usize) -> Cow<'_, str> {
    match &alternative.label {
        Some(label) => Cow::Borrowed(label),
        None => Cow::Owned(format!("Alt{}", index)),
    }
}

pub fn is_valid_rust_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    for (i, c) in s.chars().enumerate() {
        if c.is_alphabetic() || c == '_' {
            continue;
        }
        if i > 0 && c.is_ascii_digit() {
            continue;
        }
        return false;
    }
    true
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

pub fn is_rust_keyword(s: &str) -> bool {
    RUST_KEYWORDS.contains(&s)
}

/// Creates an identifier that is safe to use in generated Rust code.
/// If the name is a Rust keyword, uses raw identifier syntax (r#keyword).
pub fn safe_ident(name: &str) -> proc_macro2::Ident {
    if is_rust_keyword(name) {
        proc_macro2::Ident::new_raw(name, proc_macro2::Span::call_site())
    } else {
        proc_macro2::Ident::new(name, proc_macro2::Span::call_site())
    }
}

