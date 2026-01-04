use std::borrow::Cow;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::grammar::def::Alternative;

pub fn to_first_uppercase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

pub fn to_first_lowercase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_lowercase(), chars.as_str()),
        None => String::new(),
    }
}

pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

pub fn to_snake_case(s: &str) -> String {
    s.to_lowercase()
}

pub fn alternative_label(alternative: &Alternative, index: usize) -> Cow<'_, str> {
    match &alternative.label {
        Some(label) => Cow::Borrowed(label),
        None => Cow::Owned(format!("Alt{}", index)),
    }
}

pub fn rustfmt(code: &str) -> String {
    let mut child = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn rustfmt");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}
