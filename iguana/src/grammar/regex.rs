use std::fmt::Display;

use itertools::Itertools;

use super::symbols::Identifier;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Regex {
    Char(char),
    CharRange(CharRange),
    CharClass(CharClass),
    Seq(Vec<Regex>),
    Alt(Vec<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>),
    Opt(Box<Regex>),
    Epsilon,
    /// A reference to another named `@regex` rule — inlined during grammar compilation.
    Identifier(Identifier),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CharRange {
    pub start: char,
    pub end: char,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CharClass {
    pub ranges: Vec<CharRange>,
    pub negated: bool,
}

impl Display for CharRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl Regex {
    pub fn char(c: char) -> Self {
        Regex::Char(c)
    }

    pub fn range(start: char, end: char) -> Self {
        Regex::CharRange(CharRange { start, end })
    }

    pub fn seq(parts: Vec<Regex>) -> Self {
        Regex::Seq(parts)
    }

    pub fn alt(choices: Vec<Regex>) -> Self {
        Regex::Alt(choices)
    }

    pub fn star(regex: Regex) -> Self {
        Regex::Star(Box::new(regex))
    }

    pub fn plus(regex: Regex) -> Self {
        Regex::Plus(Box::new(regex))
    }

    pub fn char_class(ranges: Vec<CharRange>, negated: bool) -> Self {
        Regex::CharClass(CharClass { ranges, negated })
    }

    pub fn literal(s: &str) -> Self {
        Regex::Seq(s.chars().map(Regex::Char).collect())
    }
}

impl std::fmt::Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Regex::Char(c) => write!(f, "{}", c),
            Regex::CharRange(r) => write!(f, "{}", r),
            Regex::Seq(parts) => {
                for part in parts {
                    write!(f, "{}", part)?;
                }
                Ok(())
            }
            Regex::Alt(choices) => {
                write!(f, "(")?;
                for (i, choice) in choices.iter().enumerate() {
                    if i > 0 {
                        write!(f, "|")?;
                    }
                    write!(f, "{}", choice)?;
                }
                write!(f, ")")
            }
            Regex::Star(inner) => {
                if needs_grouping(inner) {
                    write!(f, "({})*", inner)
                } else {
                    write!(f, "{}*", inner)
                }
            }
            Regex::Plus(inner) => {
                if needs_grouping(inner) {
                    write!(f, "({})+", inner)
                } else {
                    write!(f, "{}+", inner)
                }
            }
            Regex::CharClass(cc) => {
                let ranges_to_string = cc.ranges.iter().map(|r| r.to_string()).join(" ");
                if cc.negated {
                    write!(f, "![{}]", ranges_to_string)
                } else {
                    write!(f, "[{}]", ranges_to_string)
                }
            }
            Regex::Opt(inner) => {
                if needs_grouping(inner) {
                    write!(f, "({})?", inner)
                } else {
                    write!(f, "{}?", inner)
                }
            }
            Regex::Epsilon => write!(f, "{}", "ε"),
            Regex::Identifier(id) => write!(f, "{}", id.name),
        }
    }
}

fn needs_grouping(regex: &Regex) -> bool {
    matches!(regex, Regex::Seq(_) | Regex::Alt(_))
}

#[macro_export]
macro_rules! c {
    ($c:literal) => {
        $crate::grammar::regex::Regex::Char($c)
    };
}

#[macro_export]
macro_rules! r {
    [$start:literal - $end:literal] => {
        $crate::grammar::regex::Regex::CharRange($crate::grammar::regex::CharRange {
            start: $start,
            end: $end,
        })
    };
}

#[macro_export]
macro_rules! r_seq {
    ($($part:expr),* $(,)?) => {
        $crate::grammar::regex::Regex::Seq(vec![$($part),*])
    };
}

#[macro_export]
macro_rules! r_alt {
    ($($choice:expr),* $(,)?) => {
        $crate::grammar::regex::Regex::Alt(vec![$($choice),*])
    };
}

#[macro_export]
macro_rules! r_star {
    ($inner:expr) => {
        $crate::grammar::regex::Regex::Star(Box::new($inner))
    };
}

#[macro_export]
macro_rules! r_plus {
    ($inner:expr) => {
        $crate::grammar::regex::Regex::Plus(Box::new($inner))
    };
}

#[macro_export]
macro_rules! r_opt {
    ($inner:expr) => {
        $crate::grammar::regex::Regex::Opt(Box::new($inner))
    };
}

#[macro_export]
macro_rules! cc {
    ([$($start:literal - $end:literal),* $(,)?]) => {
        $crate::grammar::regex::Regex::CharClass($crate::grammar::regex::CharClass {
            ranges: vec![
                $($crate::grammar::regex::CharRange { start: $start, end: $end }),*
            ],
            negated: false,
        })
    };
    (![$($start:literal - $end:literal),* $(,)?]) => {
        $crate::grammar::regex::Regex::CharClass($crate::grammar::regex::CharClass {
            ranges: vec![
                $($crate::grammar::regex::CharRange { start: $start, end: $end }),*
            ],
            negated: true,
        })
    };
}
