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

#[test]
fn test_valid_identifiers() {
    assert_eq!(match_identifier("a"), Some(1));
    assert_eq!(match_identifier("_start"), Some(6));
    assert_eq!(match_identifier("myVar123"), Some(8));
    assert_eq!(match_identifier("__init__"), Some(8));
}

#[test]
fn test_invalid_identifiers() {
    assert_eq!(match_identifier("0abc"), None);
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
