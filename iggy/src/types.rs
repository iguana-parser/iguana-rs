pub enum EbnfKind {
    Star,
    Plus,
    Opt,
    Group,
    Alt,
}
pub struct Nonterminal {
    pub name: &'static str,
    pub display: &'static str,
    pub kind: Option<EbnfKind>,
}
impl Nonterminal {
    pub fn is_ebnf(&self) -> bool {
        self.kind.is_some()
    }
}
pub struct Terminal {
    pub name: &'static str,
}
pub struct Slot {
    pub display_name: &'static str,
}

