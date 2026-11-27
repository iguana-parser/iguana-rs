//! Exact keyword matching: a keyword must not be directly preceded or
//! followed by an identifier character.
//!
//! Keyword matching is a problem specific to single-phase parsing.
//! Consider the input `elsea` against a grammar with the keyword `"else"`
//! and an identifier rule `[a-zA-Z][a-zA-Z_0-9]*`. Both terminals match at
//! the start of the input. The keyword matches four characters, and the
//! identifier matches all five. A two-phase parser resolves the conflict in
//! the scanning pass by applying the longest match. Therefore, `elsea`
//! becomes one identifier token and the keyword is not sent to the parser.
//!
//! Iguana's scanner does not apply a global longest match. The parser
//! drives the scanner and matches the terminals the grammar expects at a
//! position. Each terminal takes its own longest match, and no comparison
//! across terminals takes place. A keyword can therefore match where a
//! scanner with a global longest match would report part of an
//! identifier. There are two cases:
//!
//! - A keyword followed by identifier characters. The literal `"else"`
//!   matches the first four characters of `elsea`, and when the grammar
//!   allows an identifier after `else`, the input can parse as `else`
//!   followed by the identifier `a`.
//! - Two keywords without layout between them. When the grammar allows
//!   `"class"` after `"public"` and layout is nullable, the input
//!   `publicclass` can parse as `public` followed by `class`, although a
//!   scanner with a global longest match reports one identifier.
//!
//! The first case can lead to ambiguity in ordinary programs. The
//! following grammar defines a list of statements:
//!
//! ```text
//! Program = Stmt+
//! Stmt = Expr ";" | "if" "(" Expr ")" Stmt ("else" Stmt)?
//! Expr = Id
//! @Regex Id = [a-zA-Z][a-zA-Z_0-9]*
//! @Layout @Regex WS = [\ \n\t]*
//! ```
//!
//! The intended parse tree of the input `if (c) x; elsewhere;` has two
//! statements, the second the expression statement `elsewhere;`:
//!
//! ```text
//! (Stmt+
//!   (Stmt
//!     "if"
//!     "("
//!     (Expr Id)
//!     ")"
//!     (Stmt
//!       (Expr Id)
//!       ";"))
//!   (Stmt
//!     (Expr Id)
//!     ";"))
//! ```
//!
//! Matching `"else"` inside `elsewhere` produces a second parse tree, one
//! if statement whose else branch is the statement `where;`:
//!
//! ```text
//! (Stmt+
//!   (Stmt
//!     "if"
//!     "("
//!     (Expr Id)
//!     ")"
//!     (Stmt
//!       (Expr Id)
//!       ";")
//!     "else"
//!     (Stmt
//!       (Expr Id)
//!       ";")))
//! ```
//!
//! Both parse trees exist, so the input is ambiguous.
//!
//! The fix is a restriction pair on each keyword, as in
//! `[a-zA-Z_0-9] !<< "else" !>> [a-zA-Z_0-9]`. The notation is conceptual
//! and is not valid Iggy syntax, because a restriction operand in Iggy is
//! the name of a lexical rule, not a character class.
//!
//! Writing the pair by hand for every keyword in the grammar is tedious,
//! so Iguana automates it. With `@Identifier` on the identifier rules, the
//! generator derives the pairs and inserts them on literals.
//!
//! # Generating the restrictions from `@Identifier`
//!
//! The idea behind generating restriction pairs for a keyword from an
//! identifier is that a keyword must not extend into an identifier on
//! either side. There is no way to mark a literal as a keyword in Iggy, so
//! every string literal in a grammar is a potential keyword. The generator
//! derives the restrictions for every literal, and a literal that an
//! identifier cannot extend, such as `"++"`, does not get a restriction.
//!
//! - For the follow restriction (`!>>`), the generator finds the longest
//!   suffix of the literal that is the start of an identifier. The
//!   character class of the restriction is the characters that can come
//!   after this suffix in an identifier.
//! - For the precede restriction (`!<<`), the generator finds the longest
//!   prefix of the literal that is the end of an identifier. The character
//!   class of the restriction is the characters that can come directly
//!   before this prefix in an identifier.
//!
//! A literal without such a suffix does not get the follow restriction,
//! and a literal without such a prefix does not get the precede
//! restriction. A literal also does not get a restriction if no character
//! can come after the suffix in an identifier, because the character
//! class is then empty.
//!
//! The following literals show the cases, with the identifier rule
//! `[a-zA-Z][a-zA-Z_0-9]*`:
//!
//! - `"else"` is an identifier, and the literal is its own longest suffix
//!   and prefix. It gets `[a-zA-Z_0-9] !<<` and `!>> [a-zA-Z_0-9]`.
//! - `"non-sealed"` is not an identifier. The suffix is `sealed` and the
//!   prefix is `non`, and the literal gets the same pair as `"else"`.
//! - `"@interface"` has the suffix `interface` and gets the follow
//!   restriction. Every prefix of the literal starts with `@`, and an
//!   identifier does not contain `@`. No prefix is the end of an
//!   identifier, and the literal does not get the precede restriction.
//! - `"++"` does not get a restriction, because an identifier does not
//!   contain `+`.
//!
//! A case-insensitive literal such as `i"else"` matches every spelling of
//! the keyword, such as `else`, `ELSE`, and `eLsE`. With the identifier
//! rule `[a-zA-Z][a-zA-Z_0-9]*`, every spelling is an identifier, and the
//! literal gets the same pair as `"else"`.
//!
//! The restrictions come from the regex of the identifier-annotated rule
//! alone; the rule's `\` excepts are dropped. Dropping them matters when the
//! keywords are excluded from the identifier rule, as in
//! `@Regex Identifier = [a-zA-Z][a-zA-Z_0-9]* \ Keyword`. The exclusion
//! removes `else` from the identifier's language, but `else` still looks
//! like an identifier, and its boundaries must still be enforced.
//!
//! With several identifier-annotated rules, an identifier is a string
//! that at least one of the rules matches. Each restriction of a literal
//! is therefore a single character class for all the rules.
//!
//! The restrictions above attach to literal occurrences in syntax
//! rules, including a syntax rule that is nothing but an alternation of
//! literals. A literal can also appear inside a lexical rule, as in
//! `@Regex Bool = "true" | "false"`. A lexical rule is a single terminal,
//! and the literals in its body are parts of the rule's regex, not
//! occurrences, so no restriction attaches to them individually. A rule
//! of this shape is still a set of keywords, though, and the restrictions
//! belong on the terminal as a whole. When the body is an alternation of
//! literals, every reference to the rule gets the follow restriction if
//! every literal of the body gets it, and the same holds for the precede
//! restriction. The character class of the restriction is the union of
//! the character classes of the literals. One literal that does not
//! get a restriction disqualifies the whole rule for that restriction,
//! because the restriction applies to every match of the rule, and a
//! literal like `++` must keep matching next to identifier characters.
//!
//! A class of the two cases of a letter, such as `[eE]`, counts as a
//! character of a literal, because it is the regex of a letter in a
//! case-insensitive literal. The generator therefore treats
//! `@Regex Exponent = [eE]` the same as `i"e"`.
//!
//! Literals in `@NoLayout` rules are left alone. A `@NoLayout` rule is a
//! character-level composition, and its literals are fragments of a
//! larger token rather than keywords.

use rustc_hash::FxHashMap;

use crate::dfa::Dfa;
use crate::grammar::def::{LayoutStrategy, LexicalRule, SyntaxRule};
use crate::grammar::regex::{CharClass, CharRange, Regex, merge_ranges};
use crate::grammar::symbols::{Identifier, Restrictions, Symbol, Terminal};
use crate::grammar::transformations::{transform_symbol, transform_syntax_rule};

/// Inserts the derived restriction pair on every keyword reference in
/// `syntax_rules`, appending each operand's rule to `lexical_rules` on
/// first use. Without identifier-annotated rules the grammar is returned
/// unchanged.
pub fn transform(
    syntax_rules: Vec<SyntaxRule>,
    mut lexical_rules: Vec<LexicalRule>,
    identifier_rules: &[Identifier],
) -> (Vec<SyntaxRule>, Vec<LexicalRule>) {
    if identifier_rules.is_empty() {
        return (syntax_rules, lexical_rules);
    }
    // One regex for all the identifier-annotated rules. An identifier is a
    // string that one of the rules matches.
    let identifier = Regex::Alt(
        identifier_rules
            .iter()
            .map(|id| {
                let rule = lexical_rules
                    .iter()
                    .find(|rule| rule.head.name == id.name)
                    .unwrap_or_else(|| panic!("identifier rule `{}` has no lexical rule", id.name));
                rule.regex.clone()
            })
            .collect(),
    );
    let dfa = Dfa::from_regex(&identifier);
    let reversed_dfa = Dfa::from_regex(&identifier.reversed());

    // The derived restrictions of every keyword rule, keyed by rule name.
    // Rule names are unique across the grammar, so a reference with a
    // matching name can only refer to the keyword rule.
    let keywords: FxHashMap<String, KeywordRestrictions> = lexical_rules
        .iter()
        .filter_map(|rule| {
            let derived = keyword_restrictions(&dfa, &reversed_dfa, &rule.regex)?;
            if derived.precede.is_none() && derived.follow.is_none() {
                None
            } else {
                Some((rule.head.name.clone(), derived))
            }
        })
        .collect();

    let mut rewrite = |symbol: Symbol| match symbol {
        Symbol::Identifier(id) => match keywords.get(&id.name) {
            Some(derived) => {
                let restrictions = Restrictions {
                    precede: restriction_operand(derived.precede.as_ref(), &mut lexical_rules),
                    follow: restriction_operand(derived.follow.as_ref(), &mut lexical_rules),
                    ..Default::default()
                };
                Symbol::restricted(Symbol::Identifier(id), restrictions)
            }
            None => Symbol::Identifier(id),
        },
        // The walk is bottom-up: a literal inside a handwritten
        // `Restricted` node arrives freshly wrapped. The merge folds the
        // derived pair into the handwritten node, because `Restricted`
        // nodes do not nest. The derived operands are synthesized names,
        // so the two sides never share an entry.
        Symbol::Restricted {
            symbol,
            restrictions,
        } => match *symbol {
            Symbol::Restricted {
                symbol: literal,
                restrictions: derived,
            } => Symbol::Restricted {
                symbol: literal,
                restrictions: merge(restrictions, derived),
            },
            inner => Symbol::Restricted {
                symbol: Box::new(inner),
                restrictions,
            },
        },
        other => other,
    };

    let syntax_rules = syntax_rules
        .into_iter()
        .map(|rule| {
            // A `@NoLayout` rule is a character-level composition: a
            // literal there is a fragment of a larger token, so
            // restricting it does not make sense.
            if rule.layout == LayoutStrategy::None {
                return rule;
            }
            transform_syntax_rule(rule, |symbol| transform_symbol(symbol, &mut rewrite))
        })
        .collect();
    (syntax_rules, lexical_rules)
}

/// The operand list for a class: a reference to the rule defining the
/// class, synthesized on first use and named by its rendering. The list is
/// empty if there is no class.
fn restriction_operand(
    class: Option<&CharClass>,
    lexical_rules: &mut Vec<LexicalRule>,
) -> Vec<Identifier> {
    let Some(class) = class else {
        return vec![];
    };
    let class = Regex::CharClass(class.clone());
    let name = class.to_string();
    if !lexical_rules.iter().any(|rule| rule.head.name == name) {
        lexical_rules.push(LexicalRule::new(Terminal::new(name.clone()), class));
    }
    vec![Identifier {
        name,
        definition: None,
    }]
}

/// The two restriction sets combined: each list is the concatenation in
/// argument order.
fn merge(first: Restrictions, second: Restrictions) -> Restrictions {
    Restrictions {
        precede: [first.precede, second.precede].concat(),
        follow: [first.follow, second.follow].concat(),
        excepts: [first.excepts, second.excepts].concat(),
        required: [first.required, second.required].concat(),
        layout_aware_follow: [first.layout_aware_follow, second.layout_aware_follow].concat(),
    }
}

/// The character classes of the two restrictions derived for a keyword
/// rule. A character class is `None` if the rule does not get the
/// restriction.
struct KeywordRestrictions {
    precede: Option<CharClass>,
    follow: Option<CharClass>,
}

/// Calculates the restrictions of the keyword `regex`. `dfa` is the DFA of
/// the identifier, and `reversed_dfa` is the DFA of the reversed
/// identifier. Returns `None` if `regex` is not a literal or an
/// alternation of literals.
///
/// The character class of the follow restriction comes from the literals
/// and `dfa`. The character class of the precede restriction comes from
/// the same computation, with the reversed literals and `reversed_dfa`.
/// This DFA consumes an identifier from its last character to its first.
/// A suffix of the reversed literal is a prefix of the literal in the
/// opposite order, and the characters that the DFA can consume after it
/// are the characters that can come directly before the prefix in an
/// identifier.
fn keyword_restrictions(
    dfa: &Dfa,
    reversed_dfa: &Dfa,
    regex: &Regex,
) -> Option<KeywordRestrictions> {
    let follow = restriction_class(dfa, &literals(regex)?);
    let reversed = regex.reversed();
    let precede = restriction_class(reversed_dfa, &literals(&reversed)?);
    Some(KeywordRestrictions { precede, follow })
}

/// The character class of one restriction of a keyword rule with the
/// literals `literals`, calculated with `dfa`.
///
/// A restriction applies to every match of the rule, so the rule gets the
/// restriction only if every literal gets it. The character class is then
/// the union of the character classes of the literals. The result is
/// `None` if the rule does not get the restriction.
fn restriction_class(dfa: &Dfa, literals: &[&Regex]) -> Option<CharClass> {
    let mut ranges = Vec::new();
    for literal in literals {
        ranges.extend(next_chars_after_longest_suffix(dfa, literal)?);
    }
    Some(CharClass {
        ranges: merge_ranges(ranges),
        negated: false,
    })
}

/// The characters that `dfa` can consume after the longest suffix of
/// `literal` that it can consume from its start state. For the literal
/// `non-sealed` and the DFA of an identifier, the suffix is `sealed`. The
/// result is `None` if `dfa` cannot consume a suffix of `literal`, or if
/// it cannot consume a character after the suffix.
fn next_chars_after_longest_suffix(dfa: &Dfa, literal: &Regex) -> Option<Vec<CharRange>> {
    let keyword = Dfa::from_regex(literal);
    // A literal is a sequence of single characters, or one character.
    let len = match literal {
        Regex::Seq(chars) => chars.len(),
        _ => 1,
    };
    // The loop tries the suffixes of the literal from the whole literal to
    // its last character. A suffix is the literal without its first `skip`
    // characters. `next_chars_after` returns `None` if `dfa` cannot consume
    // the suffix, and the loop goes on to the next suffix.
    //
    // The loop stops at the first suffix that `dfa` can consume. A shorter
    // suffix can be the start of a different identifier. With the rules
    // `[a-z]+` and `"r#" [a-z]+`, the suffix `r` of `for` is the start of a
    // raw identifier, but `#` cannot come after `for` in an identifier.
    for skip in 0..len {
        if let Some(chars) = dfa.next_chars_after(&keyword, skip) {
            return if chars.is_empty() { None } else { Some(chars) };
        }
    }
    None
}

/// The literals of a regex that is an alternation of literals (a single
/// literal counts), or `None` for any other regex.
fn literals(regex: &Regex) -> Option<Vec<&Regex>> {
    let literals: Vec<&Regex> = match regex {
        Regex::Alt(arms) => arms.iter().collect(),
        other => vec![other],
    };
    if literals.iter().all(|literal| literal.is_literal()) {
        Some(literals)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The DFA of the identifier `regex` and the DFA of the reversed
    /// `regex`.
    fn dfa(regex: Regex) -> (Dfa, Dfa) {
        (Dfa::from_regex(&regex), Dfa::from_regex(&regex.reversed()))
    }

    fn class(ranges: &[(char, char)]) -> Option<CharClass> {
        let ranges = ranges
            .iter()
            .map(|&(start, end)| CharRange { start, end })
            .collect();
        Some(CharClass {
            ranges,
            negated: false,
        })
    }

    /// `[a-zA-Z][a-zA-Z_0-9]*`
    fn identifier() -> (Dfa, Dfa) {
        dfa(Regex::seq(vec![
            Regex::alt(vec![Regex::range('A', 'Z'), Regex::range('a', 'z')]),
            Regex::star(Regex::alt(vec![
                Regex::range('A', 'Z'),
                Regex::range('a', 'z'),
                Regex::char('_'),
                Regex::range('0', '9'),
            ])),
        ]))
    }

    /// The restrictions of the literal `text`.
    fn derive((dfa, reversed_dfa): &(Dfa, Dfa), text: &str) -> KeywordRestrictions {
        keyword_restrictions(dfa, reversed_dfa, &Regex::literal(text)).unwrap()
    }

    /// The restrictions of the case-insensitive literal `text`.
    fn derive_ignore_case((dfa, reversed_dfa): &(Dfa, Dfa), text: &str) -> KeywordRestrictions {
        keyword_restrictions(dfa, reversed_dfa, &Regex::ignore_case_literal(text)).unwrap()
    }

    #[test]
    fn keyword_derives_the_identifier_character_class() {
        let dfas = identifier();
        let classes = derive(&dfas, "else");
        let expected = class(&[('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')]);
        assert_eq!(classes.follow, expected);
        assert_eq!(classes.precede, expected);
    }

    #[test]
    fn a_literal_without_identifier_characters_at_its_ends_derives_nothing() {
        let dfas = identifier();
        for text in ["++", "+else+"] {
            let classes = derive(&dfas, text);
            assert!(classes.precede.is_none(), "{text}");
            assert!(classes.follow.is_none(), "{text}");
        }
    }

    /// An identifier can end with `0` but cannot start with it.
    #[test]
    fn a_literal_that_can_only_end_an_identifier_derives_the_precede_char_class() {
        let classes = derive(&identifier(), "0");
        let expected = class(&[('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')]);
        assert_eq!(classes.precede, expected);
        assert!(classes.follow.is_none());
    }

    /// `non-sealed` is not an identifier, but `non` can be the end of an
    /// identifier and `sealed` can be the start of an identifier.
    #[test]
    fn a_literal_with_a_non_identifier_character_inside_derives_both_classes() {
        let dfas = identifier();
        let expected = class(&[('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')]);
        for text in ["non-sealed", "else if"] {
            let classes = derive(&dfas, text);
            assert_eq!(classes.precede, expected, "{text}");
            assert_eq!(classes.follow, expected, "{text}");
        }
    }

    /// An identifier can continue after `@interface` but not before it,
    /// and before `else+` but not after it.
    #[test]
    fn each_end_of_a_literal_is_decided_on_its_own() {
        let dfas = identifier();
        let expected = class(&[('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')]);

        let classes = derive(&dfas, "@interface");
        assert!(classes.precede.is_none());
        assert_eq!(classes.follow, expected);

        let classes = derive(&dfas, "else+");
        assert_eq!(classes.precede, expected);
        assert!(classes.follow.is_none());
    }

    /// `[a-z][a-z0-9]*` has only lowercase letters, and `i"Go"` gets both
    /// character classes from its lowercase spelling. With uppercase
    /// letters in the identifier, `i"select"` gets them from every
    /// spelling.
    #[test]
    fn a_case_insensitive_literal_gets_the_char_classes_of_its_consumed_spellings() {
        let lowercase = dfa(Regex::seq(vec![
            Regex::range('a', 'z'),
            Regex::star(Regex::alt(vec![
                Regex::range('a', 'z'),
                Regex::range('0', '9'),
            ])),
        ]));
        let expected = class(&[('0', '9'), ('a', 'z')]);
        let classes = derive_ignore_case(&lowercase, "Go");
        assert_eq!(classes.precede, expected);
        assert_eq!(classes.follow, expected);

        let classes = derive_ignore_case(&identifier(), "select");
        let expected = class(&[('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')]);
        assert_eq!(classes.precede, expected);
        assert_eq!(classes.follow, expected);
    }

    /// `[a-z]+ | "r#" [a-z]+`: `#` is a character of an identifier, but an
    /// identifier does not start or end with it. The literal `#` does not
    /// get a restriction, and a letter can directly follow it.
    #[test]
    fn a_character_inside_an_identifier_does_not_restrict_its_literal() {
        let id = dfa(Regex::alt(vec![
            Regex::plus(Regex::range('a', 'z')),
            Regex::seq(vec![
                Regex::literal("r#"),
                Regex::plus(Regex::range('a', 'z')),
            ]),
        ]));
        let classes = derive(&id, "#");
        assert!(classes.precede.is_none());
        assert!(classes.follow.is_none());
    }

    /// With identifiers of exactly two letters, the longest prefix of
    /// `else` that ends an identifier is `el`, and the longest suffix that
    /// starts one is `se`. Both are whole identifiers, so no character can
    /// come before `el` or after `se`.
    #[test]
    fn the_char_classes_come_from_the_longest_prefix_and_suffix() {
        let dfas = dfa(Regex::seq(vec![
            Regex::range('a', 'z'),
            Regex::range('a', 'z'),
        ]));
        let classes = derive(&dfas, "else");
        assert!(classes.precede.is_none());
        assert!(classes.follow.is_none());
        let classes = derive(&dfas, "e");
        assert_eq!(classes.precede, class(&[('a', 'z')]));
        assert_eq!(classes.follow, class(&[('a', 'z')]));
    }

    /// `[a-z][0-9]*`: a letter occurs only at an identifier start, so
    /// nothing can precede the literal `a` mid-identifier, while digits
    /// extend it.
    #[test]
    fn precede_and_follow_char_classes_differ_in_an_asymmetric_automaton() {
        let id = dfa(Regex::seq(vec![
            Regex::range('a', 'z'),
            Regex::star(Regex::range('0', '9')),
        ]));
        let classes = derive(&id, "a");
        assert!(classes.precede.is_none());
        assert_eq!(classes.follow, class(&[('0', '9')]));
    }

    use crate::grammar::def::{SymbolTable, create_symbol_table};
    use crate::{alternative, follow, opt, priority_level, syntax_rule};

    /// `Id = [a-z][a-z0-9]*`; both derived classes come out `[0-9 a-z]`.
    fn identifier_rule() -> LexicalRule {
        LexicalRule::new(
            Terminal::new("Id"),
            Regex::seq(vec![
                Regex::range('a', 'z'),
                Regex::star(Regex::alt(vec![
                    Regex::range('a', 'z'),
                    Regex::range('0', '9'),
                ])),
            ]),
        )
    }

    fn literal_rule(text: &str) -> LexicalRule {
        LexicalRule::new(Terminal::literal(text), Regex::literal(text))
    }

    fn id_rules() -> Vec<Identifier> {
        vec![Identifier {
            name: "Id".into(),
            definition: None,
        }]
    }

    /// A resolved reference, as the transform receives them.
    fn refer(name: &str, table: &SymbolTable) -> Symbol {
        Symbol::Identifier(Identifier {
            name: name.into(),
            definition: table.get(name),
        })
    }

    /// Builds `S = <symbols>` with references resolved against `lexical`,
    /// and runs the transform on it.
    fn run_transform(
        symbols: impl Fn(&SymbolTable) -> Vec<Symbol>,
        lexical: Vec<LexicalRule>,
        identifier_rules: &[Identifier],
    ) -> (Vec<SyntaxRule>, Vec<LexicalRule>) {
        let mut rules = vec![syntax_rule!("S" => priority_level!(alternative!()))];
        let (_, table) = create_symbol_table(&rules, &lexical);
        rules[0].priority_levels[0].alternatives[0].symbols = symbols(&table);
        transform(rules, lexical, identifier_rules)
    }

    fn first_symbol(rules: &[SyntaxRule]) -> &Symbol {
        &rules[0].priority_levels[0].alternatives[0].symbols[0]
    }

    fn names(ids: &[Identifier]) -> Vec<&str> {
        ids.iter().map(|id| id.name.as_str()).collect()
    }

    fn restrictions(symbol: &Symbol) -> &Restrictions {
        match symbol {
            Symbol::Restricted { restrictions, .. } => restrictions,
            other => panic!("expected a restricted symbol, got {other:?}"),
        }
    }

    #[test]
    fn keywords_get_the_pair_and_share_one_operand_rule() {
        let (out, lexical) = run_transform(
            |table| vec![refer("\"else\"", table), refer("\"if\"", table)],
            vec![identifier_rule(), literal_rule("else"), literal_rule("if")],
            &id_rules(),
        );
        for symbol in &out[0].priority_levels[0].alternatives[0].symbols {
            let restrictions = restrictions(symbol);
            assert_eq!(names(&restrictions.precede), ["[0-9 a-z]"]);
            assert_eq!(names(&restrictions.follow), ["[0-9 a-z]"]);
        }
        // One synthesized rule serves both keywords and both operands.
        assert_eq!(lexical.len(), 4);
        assert_eq!(lexical[3].head.name, "[0-9 a-z]");
        assert_eq!(lexical[3].regex.to_string(), "[0-9 a-z]");
    }

    #[test]
    fn derived_restrictions_merge_after_the_authors() {
        let (out, _) = run_transform(
            |table| vec![follow!(refer("\"else\"", table), "X")],
            vec![identifier_rule(), literal_rule("else")],
            &id_rules(),
        );
        let restrictions = restrictions(first_symbol(&out));
        assert_eq!(names(&restrictions.follow), ["X", "[0-9 a-z]"]);
        assert_eq!(names(&restrictions.precede), ["[0-9 a-z]"]);
    }

    #[test]
    fn literals_nested_in_ebnf_symbols_are_rewritten() {
        let (out, _) = run_transform(
            |table| vec![opt!(refer("\"else\"", table))],
            vec![identifier_rule(), literal_rule("else")],
            &id_rules(),
        );
        let Symbol::Opt(inner) = first_symbol(&out) else {
            panic!("expected Opt");
        };
        assert_eq!(names(&restrictions(inner).follow), ["[0-9 a-z]"]);
    }

    #[test]
    fn a_literal_without_restrictions_stays_untouched() {
        let (out, lexical) = run_transform(
            |table| vec![refer("\"++\"", table)],
            vec![identifier_rule(), literal_rule("++")],
            &id_rules(),
        );
        assert!(matches!(first_symbol(&out), Symbol::Identifier(_)));
        assert_eq!(lexical.len(), 2, "no operand rule should be synthesized");
    }

    #[test]
    fn nonterminal_and_terminal_references_stay_untouched() {
        let (out, _) = run_transform(
            |table| vec![refer("Id", table), refer("S", table)],
            vec![identifier_rule(), literal_rule("else")],
            &id_rules(),
        );
        let symbols = &out[0].priority_levels[0].alternatives[0].symbols;
        assert!(symbols.iter().all(|s| matches!(s, Symbol::Identifier(_))));
    }

    #[test]
    fn without_identifier_rules_the_grammar_is_unchanged() {
        let (out, lexical) = run_transform(
            |table| vec![refer("\"else\"", table)],
            vec![identifier_rule(), literal_rule("else")],
            &[],
        );
        assert!(matches!(first_symbol(&out), Symbol::Identifier(_)));
        assert_eq!(lexical.len(), 2);
    }

    #[test]
    fn literals_in_no_layout_rules_stay_untouched() {
        let lexical = vec![identifier_rule(), literal_rule("go")];
        let mut rules = vec![syntax_rule!("S" => priority_level!(alternative!()))];
        let (_, table) = create_symbol_table(&rules, &lexical);
        rules[0].priority_levels[0].alternatives[0].symbols = vec![refer("\"go\"", &table)];
        rules[0].layout = LayoutStrategy::None;
        let (out, lexical) = transform(rules, lexical, &id_rules());
        assert!(matches!(first_symbol(&out), Symbol::Identifier(_)));
        assert_eq!(lexical.len(), 2, "no operand rule should be synthesized");
    }

    /// `[a-z][0-9]*` and the literal `a`: nothing can precede an
    /// identifier start,
    /// so only the follow restriction is inserted.
    #[test]
    fn empty_precede_char_class_inserts_only_the_follow_restriction() {
        let (out, lexical) = run_transform(
            |table| vec![refer("\"a\"", table)],
            vec![
                LexicalRule::new(
                    Terminal::new("Id"),
                    Regex::seq(vec![
                        Regex::range('a', 'z'),
                        Regex::star(Regex::range('0', '9')),
                    ]),
                ),
                literal_rule("a"),
            ],
            &id_rules(),
        );
        let restrictions = restrictions(first_symbol(&out));
        assert!(restrictions.precede.is_empty());
        assert_eq!(names(&restrictions.follow), ["[0-9]"]);
        assert_eq!(lexical[2].head.name, "[0-9]");
    }

    /// `Bool = "true" | "false"`: a rule that is a set of literals is a
    /// keyword set, and references to it get the pair.
    #[test]
    fn references_to_a_rule_of_literals_get_the_pair() {
        let bool_rule = LexicalRule::new(
            Terminal::new("Bool"),
            Regex::alt(vec![Regex::literal("true"), Regex::literal("false")]),
        );
        let (out, lexical) = run_transform(
            |table| vec![refer("Bool", table)],
            vec![identifier_rule(), bool_rule],
            &id_rules(),
        );
        let restrictions = restrictions(first_symbol(&out));
        assert_eq!(names(&restrictions.precede), ["[0-9 a-z]"]);
        assert_eq!(names(&restrictions.follow), ["[0-9 a-z]"]);
        assert_eq!(lexical[2].head.name, "[0-9 a-z]");
    }

    /// `Id = [a-z]+` and `Id2 = [a-z]+ [0-9]`: an identifier is a string of
    /// either rule. A letter or a digit can come after `else`, and only a
    /// letter can come before it.
    #[test]
    fn several_identifier_rules_give_one_char_class_per_restriction() {
        let letters = || Regex::plus(Regex::range('a', 'z'));
        let id = LexicalRule::new(Terminal::new("Id"), letters());
        let id2 = LexicalRule::new(
            Terminal::new("Id2"),
            Regex::seq(vec![letters(), Regex::range('0', '9')]),
        );
        let identifier_rules: Vec<Identifier> = ["Id", "Id2"]
            .iter()
            .map(|name| Identifier {
                name: name.to_string(),
                definition: None,
            })
            .collect();
        let (out, _) = run_transform(
            |table| vec![refer("\"else\"", table)],
            vec![id, id2, literal_rule("else")],
            &identifier_rules,
        );
        let restrictions = restrictions(first_symbol(&out));
        assert_eq!(names(&restrictions.precede), ["[a-z]"]);
        assert_eq!(names(&restrictions.follow), ["[0-9 a-z]"]);
    }

    /// `Op = "true" | "++"`: a restriction would apply to `++` matches too,
    /// so a rule with a literal that does not get a restriction stays
    /// untouched.
    #[test]
    fn a_rule_with_a_literal_without_restrictions_stays_untouched() {
        let op_rule = LexicalRule::new(
            Terminal::new("Op"),
            Regex::alt(vec![Regex::literal("true"), Regex::literal("++")]),
        );
        let (out, lexical) = run_transform(
            |table| vec![refer("Op", table)],
            vec![identifier_rule(), op_rule],
            &id_rules(),
        );
        assert!(matches!(first_symbol(&out), Symbol::Identifier(_)));
        assert_eq!(lexical.len(), 2, "no operand rule should be synthesized");
    }

    /// `Reserved = i"go" | "end"`: a case-insensitive literal is a keyword
    /// like a plain one.
    #[test]
    fn a_rule_with_a_case_insensitive_literal_gets_the_pair() {
        let reserved = LexicalRule::new(
            Terminal::new("Reserved"),
            Regex::alt(vec![
                Regex::ignore_case_literal("go"),
                Regex::literal("end"),
            ]),
        );
        let (out, _) = run_transform(
            |table| vec![refer("Reserved", table)],
            vec![identifier_rule(), reserved],
            &id_rules(),
        );
        let restrictions = restrictions(first_symbol(&out));
        assert_eq!(names(&restrictions.precede), ["[0-9 a-z]"]);
        assert_eq!(names(&restrictions.follow), ["[0-9 a-z]"]);
    }

    /// `Op = "true" | "+x"`: `true` and `x` can each be the start of an
    /// identifier, but `+x` cannot be the end of an identifier. References
    /// get only the follow restriction.
    #[test]
    fn a_rule_gets_only_the_restriction_that_all_its_literals_get() {
        let op_rule = LexicalRule::new(
            Terminal::new("Op"),
            Regex::alt(vec![Regex::literal("true"), Regex::literal("+x")]),
        );
        let (out, _) = run_transform(
            |table| vec![refer("Op", table)],
            vec![identifier_rule(), op_rule],
            &id_rules(),
        );
        let restrictions = restrictions(first_symbol(&out));
        assert!(restrictions.precede.is_empty());
        assert_eq!(names(&restrictions.follow), ["[0-9 a-z]"]);
    }

    /// `Else = "else"`: a named single-literal rule is a keyword set of one.
    #[test]
    fn a_named_single_literal_rule_gets_the_pair() {
        let else_rule = LexicalRule::new(Terminal::new("Else"), Regex::literal("else"));
        let (out, _) = run_transform(
            |table| vec![refer("Else", table)],
            vec![identifier_rule(), else_rule],
            &id_rules(),
        );
        let restrictions = restrictions(first_symbol(&out));
        assert_eq!(names(&restrictions.precede), ["[0-9 a-z]"]);
        assert_eq!(names(&restrictions.follow), ["[0-9 a-z]"]);
    }
}
