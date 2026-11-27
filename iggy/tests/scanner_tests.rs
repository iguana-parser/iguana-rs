use iggy::scanner::IggyScanner;
use iguana_runtime::arena::Arena;
use iguana_runtime::input::Input;

fn match_identifier(input: &str) -> Option<u32> {
    match_identifier_at(input, 0)
}

fn match_identifier_at(input: &str, index: u32) -> Option<u32> {
    let input = Input::from(input);
    let arena = Arena::new();
    let scanner = IggyScanner::new(&input, &arena);
    scanner.match_terminal_2(index)
}

/// Matches a `String` (terminal 3) at the start of `input`. Returns the
/// offset where the match ends, or `None` when `input` does not start with a
/// `String`.
fn match_string(input: &str) -> Option<u32> {
    let input = Input::from(input);
    let arena = Arena::new();
    IggyScanner::new(&input, &arena).match_terminal_3(0)
}

/// Matches a `Char` (terminal 4) at the start of `input`. Returns the offset
/// where the match ends, or `None` when `input` does not start with a `Char`.
fn match_char(input: &str) -> Option<u32> {
    let input = Input::from(input);
    let arena = Arena::new();
    IggyScanner::new(&input, &arena).match_terminal_4(0)
}

#[test]
fn test_valid_identifiers() {
    assert_eq!(match_identifier("a"), Some(1));
    assert_eq!(match_identifier("my_var"), Some(6));
    assert_eq!(match_identifier("myVar123"), Some(8));
    assert_eq!(match_identifier("init__"), Some(6));
}

#[test]
fn test_invalid_identifiers() {
    assert_eq!(match_identifier("0abc"), None);
    assert_eq!(match_identifier("_start"), None);
    assert_eq!(match_identifier(" "), None);
    assert_eq!(match_identifier("@var"), None);
}

#[test]
fn test_identifier_partial_match() {
    assert_eq!(match_identifier("abc def"), Some(3));
    assert_eq!(match_identifier("var@123"), Some(3));
}

#[test]
fn test_identifier_cannot_start_after_a_name_character() {
    // The precede restriction on Identifier: no match right after a name
    // character, so a keyword match cannot split an identifier like `leftx`.
    assert_eq!(match_identifier_at("leftx", 4), None);
    assert_eq!(match_identifier_at("a b", 2), Some(3));
    assert_eq!(match_identifier_at("(x)", 1), Some(2));
}

#[test]
fn test_string_is_on_one_line() {
    assert_eq!(match_string(r#""a\nb""#), Some(6));
    assert_eq!(match_string("\"a\nb\""), None);
    assert_eq!(match_string("\"a\rb\""), None);
    assert_eq!(match_string("\"a\r\nb\""), None);
}

#[test]
fn test_char_is_on_one_line() {
    assert_eq!(match_char(r"'\n'"), Some(4));
    assert_eq!(match_char("'\n'"), None);
    assert_eq!(match_char("'\r'"), None);
}

#[test]
fn test_unicode_escape_has_hexadecimal_digits_in_braces() {
    assert_eq!(match_string(r#""\u{41}""#), Some(8));
    assert_eq!(match_string(r#""a\u{1F600}b""#), Some(13));
    assert_eq!(match_char(r"'\u{3C0}'"), Some(9));
    assert_eq!(match_string(r#""\u{}""#), None);
    assert_eq!(match_string(r#""\u{4G}""#), None);
    assert_eq!(match_string(r#""\u41""#), None);
    assert_eq!(match_char(r"'\u{41'"), None);
}
