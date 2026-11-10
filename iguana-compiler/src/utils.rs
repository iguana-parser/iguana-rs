pub fn to_first_lowercase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_lowercase(), chars.as_str()),
        None => String::new(),
    }
}

/// Converts a name to PascalCase.
///
/// Types and enum variants in generated parsers are in PascalCase. The
/// PascalCase form of a name is its snake_case form (`to_snake_case`) with the
/// `_` and `-` removed and each word starting with an uppercase letter. For
/// example, `line_comment`, `LINE_COMMENT`, and `LineComment` all become
/// `LineComment`, and `SPPFNode` becomes `SppfNode`.
///
/// Two names with different snake_case forms can have the same PascalCase form.
/// For example, the snake_case form of `A_1` is `a_1` and that of `A1` is `a1`,
/// but both have the PascalCase form `A1`.
pub fn to_pascal_case(s: &str) -> String {
    to_snake_case(s)
        .split(['_', '-'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Converts a name to snake_case.
///
/// Fields and methods in generated parsers are in snake_case, and constants use
/// the same form but in uppercase.
///
/// A name is converted to snake_case as follows:
///
/// - An uppercase letter after a lowercase letter starts a new word.
///   `LineComment` becomes `line_comment`.
/// - In a sequence of uppercase letters, the last letter starts a new word if a
///   lowercase letter follows it. `SPPFNode` becomes `sppf_node`. If no
///   lowercase letter follows, the sequence stays one word. `WS` becomes `ws`.
/// - A digit does not start a new word. `A1` becomes `a1`.
/// - Several `_` in a row become one `_`. `line__comment` becomes
///   `line_comment`.
/// - A trailing `_` is removed. `span_` becomes `span`.
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
        } else if c != '_' || !result.ends_with('_') {
            result.push(c);
        }
    }
    if result.ends_with('_') {
        result.pop();
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
        assert_eq!(to_snake_case("line__comment"), "line_comment");
        assert_eq!(to_snake_case("Line__Comment"), "line_comment");
        assert_eq!(to_snake_case("span_"), "span");
        assert_eq!(to_snake_case("Span__"), "span");
        assert_eq!(to_snake_case("A_1"), "a_1");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("foo_bar"), "FooBar");
        assert_eq!(to_pascal_case("foo-bar"), "FooBar");
        assert_eq!(to_pascal_case("foo_bar-baz"), "FooBarBaz");
        assert_eq!(to_pascal_case("line__comment"), "LineComment");
        assert_eq!(to_pascal_case("LINE_COMMENT"), "LineComment");
        assert_eq!(to_pascal_case("LineComment"), "LineComment");
        assert_eq!(to_pascal_case("lineComment"), "LineComment");
        assert_eq!(to_pascal_case("HTMLParser"), "HtmlParser");
        assert_eq!(to_pascal_case("SPPFNode"), "SppfNode");
        assert_eq!(to_pascal_case("WS"), "Ws");
        assert_eq!(to_pascal_case("Star_0"), "Star0");
        assert_eq!(to_pascal_case("My-Grammar"), "MyGrammar");
        assert_eq!(to_pascal_case("span_"), "Span");
        assert_eq!(to_pascal_case("A_1"), "A1");
        assert_eq!(to_pascal_case("foo-"), "Foo");
        assert_eq!(to_pascal_case(""), "");
    }
}
