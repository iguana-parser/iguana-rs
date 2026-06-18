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
        if self.start == self.end {
            write!(f, "{}", self.start.escape_debug())
        } else {
            write!(
                f,
                "{}-{}",
                self.start.escape_debug(),
                self.end.escape_debug()
            )
        }
    }
}

impl Regex {
    /// Returns true if this regex can match the empty string.
    pub fn is_nullable(&self) -> bool {
        match self {
            Regex::Char(_) | Regex::CharRange(_) | Regex::CharClass(_) | Regex::Plus(_) => false,
            Regex::Epsilon | Regex::Star(_) | Regex::Opt(_) => true,
            Regex::Seq(parts) => parts.iter().all(|r| r.is_nullable()),
            Regex::Alt(choices) => choices.iter().any(|r| r.is_nullable()),
            Regex::Identifier(_) => {
                unreachable!("Regex::Identifier should be inlined before calling is_nullable")
            }
        }
    }

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
            Regex::Char(c) => write!(f, "{}", c.escape_debug()),
            Regex::CharRange(r) => write!(f, "{}", r),
            Regex::Seq(parts) => {
                for part in parts {
                    write!(f, "{}", part)?;
                }
                Ok(())
            }
            // A single-branch alternation only arises from a rule body that has
            // one alternative. The grammar's nested alternation always has two or
            // more branches, so dropping the parentheses here never under-groups
            // a choice inside a rule.
            Regex::Alt(choices) if choices.len() == 1 => write!(f, "{}", choices[0]),
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

/// Whether a `Star`/`Plus`/`Opt` operand needs parentheses to bind correctly.
/// A multi-element sequence does; an alternation prints its own parentheses, and
/// an atom needs none. Single-element sequences and alternations render as their
/// one child, so the decision sees through to that child.
fn needs_grouping(regex: &Regex) -> bool {
    match regex {
        Regex::Seq(parts) if parts.len() == 1 => needs_grouping(&parts[0]),
        Regex::Alt(choices) if choices.len() == 1 => needs_grouping(&choices[0]),
        Regex::Seq(_) => true,
        _ => false,
    }
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
