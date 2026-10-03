use rustc_hash::{FxHashMap, FxHashSet};

use crate::grammar::{def::Grammar, regex::CharRange, symbols::Terminal};

pub struct FirstChars {
    /// A map from each terminal to its first characters.
    first_chars: FxHashMap<Terminal, Vec<CharRange>>,
    /// The terminals that match the empty string. A nullable terminal matches
    /// at every position regardless of its first characters.
    nullables: FxHashSet<Terminal>,
}

impl FirstChars {
    pub fn new(grammar: &Grammar) -> Self {
        let mut first_chars = FxHashMap::default();
        let mut nullables = FxHashSet::default();
        for terminal in grammar.terminals() {
            let rule = grammar
                .lexical_rule(terminal)
                .unwrap_or_else(|| panic!("Terminal {} is not defined", terminal.name));
            // An except does not remove first characters. For `T = [ab] \ K` with
            // `K = "a"`, `T` matches only `b`, but its first characters are `a` and
            // `b`. The scanner's first-character test rejects only a character
            // outside the first characters, so an extra character only lets the
            // scan run, and the scan then fails.
            first_chars.insert(terminal.clone(), rule.regex.first_chars());
            if rule.regex.is_nullable() {
                nullables.insert(terminal.clone());
            }
        }
        FirstChars {
            first_chars,
            nullables,
        }
    }

    /// The characters that can begin a match of `terminal`, as sorted,
    /// disjoint ranges.
    pub fn first_chars(&self, terminal: &Terminal) -> &[CharRange] {
        self.first_chars
            .get(terminal)
            .unwrap_or_else(|| panic!("unknown terminal: {}", terminal))
    }

    pub fn is_nullable(&self, terminal: &Terminal) -> bool {
        self.nullables.contains(terminal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iggy::parse_grammar;

    fn char_range(start: char, end: char) -> CharRange {
        CharRange { start, end }
    }

    fn first_chars_of(source: &str, name: &str) -> (Vec<CharRange>, bool) {
        let grammar: Grammar = parse_grammar(source).unwrap().try_into().unwrap();
        let first_chars = FirstChars::new(&grammar);
        let terminal = grammar.terminal(name).unwrap();
        (
            first_chars.first_chars(terminal).to_vec(),
            first_chars.is_nullable(terminal),
        )
    }

    #[test]
    fn first_chars_come_from_the_regex_of_each_terminal() {
        let source = r#"
            grammar G
            S = Num Id
            @Regex
            Num = [\-]? [0-9]+
            @Regex
            Id = [a-z] [a-z 0-9]*
            "#;
        assert_eq!(
            first_chars_of(source, "Num"),
            (vec![char_range('-', '-'), char_range('0', '9')], false)
        );
        assert_eq!(
            first_chars_of(source, "Id"),
            (vec![char_range('a', 'z')], false)
        );
    }

    #[test]
    fn an_except_keeps_the_first_chars_of_its_operand() {
        let source = r#"
            grammar G
            S = Id
            @Regex
            Id = [a-z]+ \ Kw
            @Regex
            Kw = "if"
            "#;
        assert_eq!(
            first_chars_of(source, "Id"),
            (vec![char_range('a', 'z')], false)
        );
    }

    #[test]
    fn a_terminal_is_nullable_when_its_regex_is() {
        let source = r#"
            grammar G
            S = As Id
            @Regex
            As = [a]*
            @Regex
            Id = [a-z]* \ Kw
            @Regex
            Kw = "if"
            "#;
        assert_eq!(
            first_chars_of(source, "As"),
            (vec![char_range('a', 'a')], true)
        );
        assert_eq!(
            first_chars_of(source, "Id"),
            (vec![char_range('a', 'z')], true)
        );
    }
}
