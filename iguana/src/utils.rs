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
    s.split(['_', '-'])
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

/// Converts the given string to snake_case by inserting `_` at word
/// boundaries. For PascalCase and camelCase, a boundary occurs where an
/// uppercase letter follows a lowercase one (`LineComment` -> `line_comment`).
/// For all-uppercase runs, letters are kept together as a single word
/// (`WS` -> `ws`, `HTML` -> `html`). A boundary is also inserted where an
/// uppercase run ends before a lowercase letter (`HTMLParser` -> `html_parser`).
/// Strings that are already snake_case pass through unchanged.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            if i > 0 && !result.ends_with('_') && (prev_lower || next_lower) {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("camelCase"), "camel_case");
        assert_eq!(to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(to_snake_case("A_B_C"), "a_b_c");
        assert_eq!(to_snake_case("aAbBcC"), "a_ab_bc_c");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("lowercase"), "lowercase");
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_snake_case("A"), "a");
        assert_eq!(to_snake_case("ABC"), "abc");
        assert_eq!(to_snake_case("WS"), "ws");
        assert_eq!(to_snake_case("HTMLParser"), "html_parser");
        assert_eq!(to_snake_case("LineComment"), "line_comment");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("foo_bar"), "FooBar");
        assert_eq!(to_pascal_case("foo-bar"), "FooBar");
        assert_eq!(to_pascal_case("foo_bar-baz"), "FooBarBaz");
        assert_eq!(to_pascal_case("__foo__"), "Foo");
        assert_eq!(to_pascal_case(""), "");
    }
}
