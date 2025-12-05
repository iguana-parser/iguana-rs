#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Regex {
    Char(char),
    CharRange { start: char, end: char },
    Seq(Vec<Regex>),
    Alt(Vec<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>),
}

impl Regex {
    pub fn char(c: char) -> Self {
        Regex::Char(c)
    }

    pub fn range(start: char, end: char) -> Self {
        Regex::CharRange { start, end }
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

    pub fn literal(s: &str) -> Self {
        Regex::Seq(s.chars().map(Regex::Char).collect())
    }
}

impl std::fmt::Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Regex::Char(c) => write!(f, "{}", c),
            Regex::CharRange { start, end } => write!(f, "[{}-{}]", start, end),
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
        }
    }
}

fn needs_grouping(regex: &Regex) -> bool {
    matches!(regex, Regex::Seq(_) | Regex::Alt(_))
}
