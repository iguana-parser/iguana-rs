//! Exact keyword matching: a keyword must not be directly preceded or
//! followed by an identifier character.
//!
//! The problem is specific to single-phase parsing. Consider the input
//! `elsea` against a grammar with the keyword `"else"` and an identifier
//! rule `[a-zA-Z][a-zA-Z_0-9]*`. Both terminals match at the start of the
//! input: the keyword matches four characters, the identifier all five. A
//! two-phase parser resolves this in the scanning pass by applying the
//! longest match. Therefore, `elsea` becomes one identifier token and the
//! keyword is not reported to the parser.
//!
//! Iguana does not have a separate scanning phase that globally applies
//! the longest match. The parser drives the scanner, each position
//! matches the one terminal the grammar expects there, and no comparison
//! across terminals takes place. A keyword can therefore match at a
//! position a scanner would never report, at either end of a word:
//!
//! - The keyword's end. The literal `"else"` matches the first four
//!   characters of `elsea`, and when the grammar allows an identifier
//!   after `else`, the input parses as `else` followed by the identifier
//!   `a`.
//! - The keyword's start. The identifier itself can never end directly
//!   before the keyword, because each terminal takes its own longest
//!   match: in `aelse`, the identifier matches all five characters, never
//!   just `a`. Another keyword can end there, though. With `"if"` also in
//!   the grammar, a scanner reports `ifelse` as one identifier, while
//!   Iguana matches `"if"` where the grammar expects it, and with
//!   nullable layout `"else"` then matches directly after, mid-word.
//!
//! Not having exact keyword matches makes ordinary input ambiguous.
//! Consider `Stmt = Expr ";" | "if" "(" Expr ")" Stmt ("else" Stmt)?`
//! and the input `if (c) x; elsewhere;`. The intended parse is two
//! statements, the second an expression statement:
//!
//! ```text
//! (if (c) x;) (elsewhere;)
//! ```
//!
//! Matching `"else"` inside `elsewhere` produces a second parse, one if
//! statement whose else branch is carved out of the word:
//!
//! ```text
//! (if (c) x; else (where;))
//! ```
//!
//! Both parses exist, so the input is ambiguous.
//!
//! The fix is a restriction pair on each keyword,
//! `[0-9 A-Z _ a-z] !<< "else" !>> [0-9 A-Z _ a-z]`: the keyword must not
//! be directly preceded or followed by an identifier character. Writing
//! the pair by hand for every keyword in the grammar is tedious, so
//! Iguana automates it: the grammar annotates its identifier rules with
//! `@Identifier`, and the generator derives and inserts the pairs.
//!
//! A literal in a syntax rule is *identifier-shaped* when an
//! identifier-annotated rule accepts its text. Every identifier-shaped
//! literal gets the restriction pair, with its two operands, one
//! character class each, derived from the identifier-annotated rules as
//! described below. The result mimics the longest-match scanner that
//! most programming languages are designed around.
//!
//! To decide which literals get the precede and follow restrictions, we
//! answer three questions about each literal, using a DFA built from the
//! identifier-annotated regex rule. The DFA of the identifier rule
//! above, `[a-zA-Z][a-zA-Z_0-9]*`, serves as the running example. It has
//! two states: the start state, with a transition on `[a-zA-Z]`, and an
//! accepting state, with a transition on `[a-zA-Z_0-9]` that loops back
//! to itself.
//!
//! - Is the literal identifier-shaped? The DFA consumes the literal's
//!   text. For `"else"`, the `e` moves the DFA from the start state to
//!   the accepting state, and `l`, `s`, `e` follow the loop, so the walk
//!   ends in an accepting state: `"else"` is identifier-shaped. `"++"` is
//!   not: the start state does not have a transition on `+`, so the walk
//!   stops at the first character, and no restrictions are inserted for
//!   the literal.
//! - Which characters would extend an identifier after the literal's end?
//!   These form the follow class, the operand of the follow restriction
//!   (`!>>`) inserted after the literal. The class is the union of the
//!   labels of the outgoing transitions of the state where the walk
//!   ended. The walk over `else` ends in the accepting state, which has
//!   one outgoing transition, the loop, labeled `[a-zA-Z_0-9]`, so that
//!   is the follow class.
//! - Which characters can an identifier end on directly before the
//!   literal? These form the precede class, the operand of the precede
//!   restriction (`!<<`) inserted before the literal. The class is the
//!   union of the labels of the transitions whose target state has a
//!   transition on the literal's first character. A character in such a
//!   label can occur inside an identifier directly before the literal's
//!   first character. The literal `"else"` starts with `e`. Both
//!   transitions of the example DFA lead to the accepting state, and the
//!   accepting state has a transition on `e` (the loop), so both labels
//!   qualify, and the precede class is their union, again
//!   `[a-zA-Z_0-9]`.
//!
//! The DFA is built from the identifier-annotated rule's regex alone;
//! the rule's `\` excepts are dropped. Dropping them matters when the
//! keywords are excluded from the identifier rule, as in
//! `Identifier = [a-zA-Z][a-zA-Z_0-9]* \ Keyword`: the exclusion removes
//! `else` from the identifier's language, but `else` still looks like an
//! identifier, and its boundaries must still be enforced.
//!
//! With several identifier-annotated rules, a literal is identifier-shaped
//! when at least one rule's DFA accepts it, and each class is the union
//! of the classes over all of them. A rule contributes even when it does
//! not accept the literal, as long as the literal is a prefix of its
//! words. With a second annotated rule `[a-z]+[0-9]`, the walk over
//! `else` ends in a non-accepting state (a digit is still missing), but
//! `else` is a prefix of the word `else1`, so an identifier can extend
//! across the keyword: the ending state's outgoing labels, digits
//! included, belong in the follow class.
//!
//! Literals in `@NoLayout` rules are left alone: a `@NoLayout` rule is a
//! character-level composition, and its literals are fragments of a
//! larger token rather than words.

use crate::dfa::Dfa;
use crate::grammar::def::{LayoutStrategy, LexicalRule, SyntaxRule};
use crate::grammar::regex::{CharClass, CharRange, Regex};
use crate::grammar::symbols::{Definition, Identifier, Restrictions, Symbol, Terminal};
use crate::grammar::transformations::{transform_symbol, transform_syntax_rule};

/// Inserts the derived restriction pair on every identifier-shaped literal
/// reference in `syntax_rules`, appending each operand's rule to
/// `lexical_rules` on first use. Without identifier-annotated rules the
/// grammar is returned unchanged.
pub fn transform(
    syntax_rules: Vec<SyntaxRule>,
    mut lexical_rules: Vec<LexicalRule>,
    definitions: &[Definition],
    identifier_rules: &[Identifier],
) -> (Vec<SyntaxRule>, Vec<LexicalRule>) {
    if identifier_rules.is_empty() {
        return (syntax_rules, lexical_rules);
    }
    let dfas: Vec<Dfa> = identifier_rules
        .iter()
        .map(|id| {
            let rule = lexical_rules
                .iter()
                .find(|rule| rule.head.name == id.name)
                .unwrap_or_else(|| panic!("identifier rule `{}` has no lexical rule", id.name));
            Dfa::from_regex(&rule.regex)
        })
        .collect();

    let mut rewrite = |symbol: Symbol| match symbol {
        Symbol::Identifier(id) => {
            let Definition::Terminal(terminal) = &definitions[id.resolve().0 as usize] else {
                return Symbol::Identifier(id);
            };
            let Some(text) = &terminal.literal else {
                return Symbol::Identifier(id);
            };
            match derive_restrictions(&dfas, text) {
                Some(derived) => {
                    let restrictions = Restrictions {
                        precede: restriction_operand(derived.precede, &mut lexical_rules),
                        follow: restriction_operand(derived.follow, &mut lexical_rules),
                        ..Default::default()
                    };
                    Symbol::restricted(Symbol::Identifier(id), restrictions)
                }
                None => Symbol::Identifier(id),
            }
        }
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
/// class, synthesized on first use and named by its rendering. An empty
/// class yields an empty list: no rule, no restriction.
fn restriction_operand(
    ranges: Vec<CharRange>,
    lexical_rules: &mut Vec<LexicalRule>,
) -> Vec<Identifier> {
    if ranges.is_empty() {
        return vec![];
    }
    let class = Regex::CharClass(CharClass {
        ranges,
        negated: false,
    });
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
        layout_aware_follow: [first.layout_aware_follow, second.layout_aware_follow].concat(),
    }
}

/// The restrictions derived for an identifier-shaped literal.
pub struct KeywordRestrictions {
    pub precede: Vec<CharRange>,
    pub follow: Vec<CharRange>,
}

/// Derives `literal`'s restrictions from the identifier-annotated rules'
/// DFAs, or `None` when no DFA accepts the literal's text (the literal is
/// not identifier-shaped).
pub fn derive_restrictions(dfas: &[Dfa], literal: &str) -> Option<KeywordRestrictions> {
    let first_char = literal.chars().next()?;
    let identifier_shaped = dfas.iter().any(|dfa| {
        dfa.state_after(literal)
            .is_some_and(|state| dfa.states[state].accept.is_some())
    });
    if !identifier_shaped {
        return None;
    }

    let mut precede = Vec::new();
    let mut follow = Vec::new();
    for dfa in dfas {
        if let Some(state) = dfa.state_after(literal) {
            follow.extend(
                dfa.states[state]
                    .transitions
                    .iter()
                    .map(|(range, _)| *range),
            );
        }
        for state in &dfa.states {
            for (range, target) in &state.transitions {
                let consumes_first = dfa.states[*target]
                    .transitions
                    .iter()
                    .any(|(r, _)| r.start <= first_char && first_char <= r.end);
                if consumes_first {
                    precede.push(*range);
                }
            }
        }
    }
    Some(KeywordRestrictions {
        precede: merge_ranges(precede),
        follow: merge_ranges(follow),
    })
}

/// Sorts `ranges` and merges overlapping and adjacent ones. The result is
/// canonical: it names the synthesized class rule, so equal classes
/// resolve to one rule.
fn merge_ranges(mut ranges: Vec<CharRange>) -> Vec<CharRange> {
    ranges.sort_by_key(|r| (r.start, r.end));
    let mut merged: Vec<CharRange> = Vec::new();
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.start as u32 <= last.end as u32 + 1 => {
                if range.end > last.end {
                    last.end = range.end;
                }
            }
            _ => merged.push(range),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::regex::Regex;

    fn dfa(regex: Regex) -> Dfa {
        Dfa::from_regex(&regex)
    }

    fn class(ranges: &[(char, char)]) -> Vec<CharRange> {
        ranges
            .iter()
            .map(|&(start, end)| CharRange { start, end })
            .collect()
    }

    /// `[a-zA-Z][a-zA-Z_0-9]*`
    fn identifier() -> Dfa {
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

    #[test]
    fn keyword_derives_the_identifier_character_class() {
        let dfas = [identifier()];
        let classes = derive_restrictions(&dfas, "else").unwrap();
        let expected = class(&[('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')]);
        assert_eq!(classes.follow, expected);
        assert_eq!(classes.precede, expected);
    }

    #[test]
    fn non_identifier_shaped_literals_derive_nothing() {
        let dfas = [identifier()];
        assert!(derive_restrictions(&dfas, "++").is_none());
        assert!(derive_restrictions(&dfas, "else if").is_none());
        assert!(derive_restrictions(&dfas, "").is_none());
    }

    #[test]
    fn literal_longer_than_the_identifier_language_is_not_shaped() {
        let two_letters = dfa(Regex::seq(vec![
            Regex::range('a', 'z'),
            Regex::range('a', 'z'),
        ]));
        let dfas = [two_letters];
        assert!(derive_restrictions(&dfas, "else").is_none());
        assert!(derive_restrictions(&dfas, "el").is_some());
    }

    /// `[a-z]+ [0-9]` does not accept `else`, but it can extend across it
    /// with a digit, so the digit lands in the follow-class union.
    #[test]
    fn non_accepting_rule_contributes_to_the_follow_class() {
        let letters = dfa(Regex::plus(Regex::range('a', 'z')));
        let letters_then_digit = dfa(Regex::seq(vec![
            Regex::plus(Regex::range('a', 'z')),
            Regex::range('0', '9'),
        ]));
        let classes = derive_restrictions(&[letters, letters_then_digit], "else").unwrap();
        assert_eq!(classes.follow, class(&[('0', '9'), ('a', 'z')]));
    }

    /// `[a-z][0-9]*`: a letter occurs only at a word start, so nothing can
    /// precede the literal `a` mid-word, while digits extend it.
    #[test]
    fn precede_and_follow_classes_differ_in_an_asymmetric_automaton() {
        let id = dfa(Regex::seq(vec![
            Regex::range('a', 'z'),
            Regex::star(Regex::range('0', '9')),
        ]));
        let classes = derive_restrictions(&[id], "a").unwrap();
        assert!(classes.precede.is_empty());
        assert_eq!(classes.follow, class(&[('0', '9')]));
    }

    #[test]
    fn adjacent_and_overlapping_ranges_merge() {
        assert_eq!(
            merge_ranges(class(&[('n', 'z'), ('a', 'm'), ('c', 'f')])),
            class(&[('a', 'z')])
        );
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
        let (definitions, table) = create_symbol_table(&rules, &lexical);
        rules[0].priority_levels[0].alternatives[0].symbols = symbols(&table);
        transform(rules, lexical, &definitions, identifier_rules)
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
    fn non_identifier_shaped_literals_stay_untouched() {
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
        let (definitions, table) = create_symbol_table(&rules, &lexical);
        rules[0].priority_levels[0].alternatives[0].symbols = vec![refer("\"go\"", &table)];
        rules[0].layout = LayoutStrategy::None;
        let (out, lexical) = transform(rules, lexical, &definitions, &id_rules());
        assert!(matches!(first_symbol(&out), Symbol::Identifier(_)));
        assert_eq!(lexical.len(), 2, "no operand rule should be synthesized");
    }

    /// `[a-z][0-9]*` and the literal `a`: nothing can precede a word start,
    /// so only the follow restriction is inserted.
    #[test]
    fn empty_precede_class_inserts_only_the_follow_restriction() {
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
}
