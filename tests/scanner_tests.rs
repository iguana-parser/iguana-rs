use iggy::scanner::IggyScanner;
use iguana_runtime::input::Input;

fn match_identifier(input: &str) -> Option<u32> {
    let input = Input::from(input);
    let scanner = IggyScanner::new(&input);
    scanner.match_terminal_1(0)
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
