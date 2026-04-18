use iguana_runtime::input::Input;

fn parse_error(source: &str) -> String {
    let input = Input::from(source);
    match iggy::parse_grammar(&input) {
        Err(e) => e.message,
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn not_a_grammar() {
    assert_eq!(parse_error("a"), "Expected \"grammar\" but found 'a'");
}

#[test]
fn grammar_without_name() {
    assert_eq!(parse_error("grammar"), "Expected Identifier but found EOF");
}

#[test]
fn grammar_with_trailing_space() {
    // The parser consumed "grammar " and tries to parse more layout at EOF.
    // Ideally this would say "Expected Identifier" but the GLL error reports
    // the layout parse failure at the furthest position.
    assert_eq!(
        parse_error("grammar "),
        "Expected one of LineComment, WS but found EOF"
    );
}
