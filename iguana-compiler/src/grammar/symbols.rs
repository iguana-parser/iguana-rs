use core::hash;
use std::{fmt::Display, hash::Hasher};

use itertools::Itertools;
use quote::{ToTokens, quote};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::grammar::def::Grammar;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Definition {
    Terminal(Terminal),
    Nonterminal(Nonterminal),
}

impl Definition {
    pub fn name(&self) -> &str {
        match self {
            Definition::Terminal(terminal) => &terminal.name,
            Definition::Nonterminal(nonterminal) => &nonterminal.name,
        }
    }
    pub fn display_name(&self) -> String {
        match self {
            Definition::Terminal(terminal) => terminal.name.clone(),
            Definition::Nonterminal(nonterminal) =>
            // For normal nonterminals, i.e., the ones that are defined by the user directly,
            // the display_name is the same the nonterminal name.
            // For other nonterminals that are generated during grammar transformations,
            // `display_name` shows a name that reflects the structure, rather than the unique,
            // synthetic name using for the code generation.
            // For example, for the rule S : A (B|C)+ C, the display name is (B|C)+, while the
            // name is S_Plus_0.
            {
                match &nonterminal.origin {
                    Some(symbol) => symbol.to_string(),
                    None => nonterminal.name.clone(),
                }
            }
        }
    }
    pub fn as_nonterminal(&self) -> &Nonterminal {
        match self {
            Definition::Terminal(_) => panic!(),
            Definition::Nonterminal(n) => n,
        }
    }
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Definition::Terminal(t) => write!(f, "{}", t),
            Definition::Nonterminal(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    Labeled {
        label: String,
        symbol: Box<Symbol>,
    },
    Identifier(Identifier),
    Literal(String),
    Group(Vec<Symbol>),
    Opt(Box<Symbol>),
    Alt(Vec<Symbol>),
    Star(Box<Symbol>, Option<Box<Symbol>>), // symbol, separator
    Plus(Box<Symbol>, Option<Box<Symbol>>), // symbol, separator
    // A symbol reference with its restrictions, which correspond to the `\`,
    // `!>>`, `!>>>`, and `!<<` operators in the concrete syntax.
    //
    // In the following, Id is the restricted symbol and X the restriction, always a regular
    // expression:
    //
    // - Except restriction: `Id \ X` disallows a string of Id that is also in the language
    //   of X.
    // - Follow restriction: `Id !>> X` disallows X directly after Id. `Id !>>> X` disallows
    //   X after the layout that follows Id.
    // - Precede restriction: `X !<< Id` disallows X directly before Id.
    Restricted {
        symbol: Box<Symbol>,
        restrictions: Restrictions,
    },
    // Corresponds to the `!` operator in the concrete syntax.
    // `A !label` means use nonterminal A but exclude the alternative labeled `label`.
    // Desugared into a new nonterminal with the excluded alternatives removed.
    Exclude {
        symbol: Box<Symbol>,
        labels: Vec<String>,
    },
    Call {
        name: Identifier,
        arguments: Vec<Expr>,
    },
    // Data-dependent condition
    Condition(Expr),
    Return(Expr),
    Binding {
        name: String,
        symbol: Box<Symbol>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Int(i64),
    Cond(Cond),
    Ref(String),
    Or(Box<Expr>, Box<Expr>),
    BitAnd(Box<Expr>, Box<Expr>),
    BitOr(Box<Expr>, Box<Expr>),
    Shl(Box<Expr>, Box<Expr>),
    Shr(Box<Expr>, Box<Expr>),
    Min(Box<Expr>, Box<Expr>),
    Ternary {
        cond: Box<Expr>,
        then: Box<Expr>,
        r#else: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cond {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub op: CondOp,
}

impl Display for Cond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CondOp {
    Eq,
    Leq,
    Geq,
}

impl Display for CondOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CondOp::Eq => write!(f, "=="),
            CondOp::Leq => write!(f, "<="),
            CondOp::Geq => write!(f, ">="),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{i}"),
            Expr::Cond(cond) => write!(f, "{}", cond),
            Expr::Ref(name) => write!(f, "{}", name),
            Expr::Or(left, right) => write!(f, "{} || {}", Operand(left), Operand(right)),
            Expr::BitAnd(left, right) => write!(f, "{} & {}", Operand(left), Operand(right)),
            Expr::BitOr(left, right) => write!(f, "{} | {}", Operand(left), Operand(right)),
            Expr::Shl(left, right) => write!(f, "{} << {}", Operand(left), Operand(right)),
            Expr::Shr(left, right) => write!(f, "{} >> {}", Operand(left), Operand(right)),
            Expr::Min(left, right) => write!(f, "min({}, {})", left, right),
            Expr::Ternary { cond, then, r#else } => {
                write!(
                    f,
                    "{} ? {} : {}",
                    Operand(cond),
                    Operand(then),
                    Operand(r#else)
                )
            }
        }
    }
}

/// Display wrapper that parenthesizes compound expressions when they
/// appear as operands of another expression. Without this, the bare
/// `Display` for `Expr` produces ambiguous formatted output like
/// `r >> 16 == 0 ? 2 : min(r, 2) << 16 | 12`, which the reader cannot
/// regroup correctly without knowing operator precedence. The generator
/// emits correct parenthesized code regardless; this only affects how
/// expressions appear in grammar-rule comments and debug output.
struct Operand<'a>(&'a Expr);

impl<'a> Display for Operand<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Expr::Int(_) | Expr::Ref(_) | Expr::Min(..) => write!(f, "{}", self.0),
            _ => write!(f, "({})", self.0),
        }
    }
}

/// Converts literals to `Expr` values: string literals become `Expr::Ref`,
/// integer literals become `Expr::Int`. Used by the `cond_expr!` macro.
pub trait IntoExpr {
    fn into_expr(self) -> Expr;
}

impl IntoExpr for &str {
    fn into_expr(self) -> Expr {
        Expr::Ref(self.into())
    }
}

impl IntoExpr for i32 {
    fn into_expr(self) -> Expr {
        Expr::Int(self as i64)
    }
}

impl IntoExpr for i64 {
    fn into_expr(self) -> Expr {
        Expr::Int(self)
    }
}

impl IntoExpr for Expr {
    fn into_expr(self) -> Expr {
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Restrictions {
    /// The `!<<` restrictions.
    pub precede: Vec<Identifier>,
    /// The `\` restrictions.
    pub excepts: Vec<Identifier>,
    /// The `!>>` restrictions.
    pub follow: Vec<Identifier>,
    /// The `!>>>` restrictions.
    pub layout_aware_follow: Vec<Identifier>,
}

impl Restrictions {
    pub fn is_empty(&self) -> bool {
        self.precede.is_empty()
            && self.excepts.is_empty()
            && self.follow.is_empty()
            && self.layout_aware_follow.is_empty()
    }

    /// True if any restriction is tested at or after the matched span (`\`,
    /// `!>>`, `!>>>`).
    pub fn has_post_conditions(&self) -> bool {
        !self.excepts.is_empty() || !self.follow.is_empty() || !self.layout_aware_follow.is_empty()
    }

    /// The identifiers of all restrictions.
    pub fn ids(&self) -> impl Iterator<Item = &Identifier> {
        self.precede
            .iter()
            .chain(&self.excepts)
            .chain(&self.follow)
            .chain(&self.layout_aware_follow)
    }
}

impl Symbol {
    pub fn literal(name: impl Into<String>) -> Self {
        Symbol::Literal(name.into())
    }

    /// The symbol with `restrictions` attached, or the symbol itself when
    /// `restrictions` is empty. `Restricted` nodes do not nest: a symbol
    /// reference holds all its restrictions in one node.
    pub fn restricted(symbol: Symbol, restrictions: Restrictions) -> Symbol {
        debug_assert!(
            !matches!(symbol, Symbol::Restricted { .. }),
            "Restricted nodes do not nest"
        );
        if restrictions.is_empty() {
            symbol
        } else {
            Symbol::Restricted {
                symbol: Box::new(symbol),
                restrictions,
            }
        }
    }

    /// Returns `true` if this symbol produces a node in the parse tree.
    /// Symbols like `Condition` and `Return` are semantic-only and do not
    /// appear in the parse tree.
    pub fn is_parse_tree_symbol(&self) -> bool {
        !matches!(self, Symbol::Condition(_) | Symbol::Return(_))
    }

    /// Returns true for a repetition symbol (`A*` or `A+`).
    pub fn is_list(&self) -> bool {
        matches!(self, Symbol::Star(_, _) | Symbol::Plus(_, _))
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            Symbol::Labeled { label, .. } => Some(label),
            _ => None,
        }
    }

    /// Returns the inner symbol if this is a wrapper (`Labeled` or `Binding`),
    /// or `self` otherwise.
    pub fn unwrap(&self) -> &Symbol {
        match self {
            Symbol::Labeled { symbol, .. }
            | Symbol::Binding { symbol, .. }
            | Symbol::Restricted { symbol, .. }
            | Symbol::Exclude { symbol, .. } => symbol,
            Symbol::Identifier(_)
            | Symbol::Literal(_)
            | Symbol::Group(_)
            | Symbol::Opt(_)
            | Symbol::Alt(_)
            | Symbol::Star(_, _)
            | Symbol::Plus(_, _)
            | Symbol::Call { .. }
            | Symbol::Condition(_)
            | Symbol::Return(_) => self,
        }
    }

    pub fn resolved_def(&self) -> DefinitionId {
        self.as_identifier()
            .unwrap_or_else(|| panic!("Symbol should be an Identifier or Call but was {}", self))
            .definition
            .expect("Symbol should be resolved")
    }

    pub fn as_identifier(&self) -> Option<&Identifier> {
        match self {
            Symbol::Identifier(identifier) => Some(identifier),
            Symbol::Call { name, .. } => Some(name),
            Symbol::Labeled { symbol, .. }
            | Symbol::Binding { symbol, .. }
            | Symbol::Restricted { symbol, .. }
            | Symbol::Exclude { symbol, .. } => symbol.as_identifier(),
            Symbol::Literal(_)
            | Symbol::Group(_)
            | Symbol::Opt(_)
            | Symbol::Alt(_)
            | Symbol::Star(_, _)
            | Symbol::Plus(_, _)
            | Symbol::Condition(_)
            | Symbol::Return(_) => None,
        }
    }

    /// The inner `Call`'s arguments, reached by unwrapping any `Labeled`,
    /// `Binding`, or restriction wrapper. `&[]` if the inner symbol is a bare
    /// `Identifier`.
    pub fn call_arguments(&self) -> &[Expr] {
        match self {
            Symbol::Call { arguments, .. } => arguments,
            Symbol::Labeled { symbol, .. }
            | Symbol::Binding { symbol, .. }
            | Symbol::Restricted { symbol, .. }
            | Symbol::Exclude { symbol, .. } => symbol.call_arguments(),
            Symbol::Identifier(_)
            | Symbol::Literal(_)
            | Symbol::Group(_)
            | Symbol::Opt(_)
            | Symbol::Alt(_)
            | Symbol::Star(_, _)
            | Symbol::Plus(_, _)
            | Symbol::Condition(_)
            | Symbol::Return(_) => &[],
        }
    }

    /// The inner `Binding`'s name, reached by unwrapping any `Labeled` or
    /// restriction wrapper. `None` if there is no binding.
    pub fn binding_name(&self) -> Option<&str> {
        match self {
            Symbol::Binding { name, .. } => Some(name),
            Symbol::Labeled { symbol, .. }
            | Symbol::Restricted { symbol, .. }
            | Symbol::Exclude { symbol, .. } => symbol.binding_name(),
            Symbol::Identifier(_)
            | Symbol::Call { .. }
            | Symbol::Literal(_)
            | Symbol::Group(_)
            | Symbol::Opt(_)
            | Symbol::Alt(_)
            | Symbol::Star(_, _)
            | Symbol::Plus(_, _)
            | Symbol::Condition(_)
            | Symbol::Return(_) => None,
        }
    }

    /// The restrictions attached to this symbol reference. A `Restricted`
    /// node can sit inside a `Labeled`, `Binding`, or `Exclude` wrapper, so
    /// the lookup unwraps those wrappers.
    pub fn restrictions(&self) -> &Restrictions {
        static NONE: Restrictions = Restrictions {
            precede: Vec::new(),
            excepts: Vec::new(),
            follow: Vec::new(),
            layout_aware_follow: Vec::new(),
        };
        match self {
            Symbol::Restricted { restrictions, .. } => restrictions,
            Symbol::Labeled { symbol, .. }
            | Symbol::Binding { symbol, .. }
            | Symbol::Exclude { symbol, .. } => symbol.restrictions(),
            Symbol::Identifier(_)
            | Symbol::Literal(_)
            | Symbol::Group(_)
            | Symbol::Opt(_)
            | Symbol::Alt(_)
            | Symbol::Star(_, _)
            | Symbol::Plus(_, _)
            | Symbol::Call { .. }
            | Symbol::Condition(_)
            | Symbol::Return(_) => &NONE,
        }
    }

    /// True if a `\`, `!>>`, `!>>>`, or `!<<` restriction applies to this
    /// symbol, at any depth in the wrapper chain.
    pub fn has_restriction(&self) -> bool {
        !self.restrictions().is_empty()
    }

    pub fn display_name(&self, grammar: &Grammar) -> String {
        match self {
            Symbol::Labeled { label, symbol } => {
                format!("{}:{}", label, symbol.display_name(grammar))
            }
            Symbol::Identifier(identifier) => {
                let def_id = identifier.resolve();
                let definition = grammar.definition(def_id);
                definition.display_name()
            }
            Symbol::Literal(_) => self.to_string(),
            Symbol::Group(symbols) => format!(
                "({})",
                symbols.iter().map(|s| s.display_name(grammar)).join(" "),
            ),
            Symbol::Opt(symbol) => format!("{}?", symbol.display_name(grammar)),
            // A Group symbol inside an Alt is a sequence and renders without
            // its own parentheses: (A B | C) rather than ((A B) | C). The
            // two variations are equivalent from the parsing perspective,
            // and the rendering uses the shorter one.
            Symbol::Alt(symbols) => symbols
                .iter()
                .map(|symbol| match symbol {
                    Symbol::Group(seq) => seq.iter().map(|s| s.display_name(grammar)).join(" "),
                    single => single.display_name(grammar),
                })
                .join(" | "),
            Symbol::Star(symbol, sep) => match sep {
                Some(sep) => format!(
                    "{{{} {}}}*",
                    symbol.display_name(grammar),
                    sep.display_name(grammar)
                ),
                None => format!("({})*", symbol.display_name(grammar)),
            },
            Symbol::Plus(symbol, sep) => match sep {
                Some(sep) => format!(
                    "{{{} {}}}+",
                    symbol.display_name(grammar),
                    sep.display_name(grammar)
                ),
                None => format!("({})+", symbol.display_name(grammar)),
            },
            Symbol::Call { name, arguments } => {
                let def_id = name.resolve();
                let definition = grammar.definition(def_id);
                format!(
                    "{}({})",
                    definition.display_name(),
                    arguments.iter().join(", ")
                )
            }
            Symbol::Restricted {
                symbol,
                restrictions,
            } => {
                let (before, after) = display_restrictions(restrictions);
                format!("{before}{}{after}", symbol.display_name(grammar))
            }
            Symbol::Exclude { symbol, labels } => {
                let exclusions = labels.iter().map(|l| format!("!{l}")).join(" ");
                format!("{} {}", symbol.display_name(grammar), exclusions)
            }
            Symbol::Binding { name, symbol } => {
                format!("{}={}", name, symbol.display_name(grammar))
            }
            Symbol::Condition(_) | Symbol::Return(_) => self.to_string(),
        }
    }
}

/// The textual representation of a symbol's restrictions, as a (before, after)
/// pair. Precede restrictions (`!<<`) go before the symbol. Except (`\`)
/// and follow (`!>>` and `!>>>`) restrictions go after the symbol, and the
/// follow restrictions always come after the except ones. Within each
/// restriction kind, the source order is preserved.
fn display_restrictions(restrictions: &Restrictions) -> (String, String) {
    let before = restrictions
        .precede
        .iter()
        .map(|p| format!("{p} !<< "))
        .join("");
    let after = restrictions
        .excepts
        .iter()
        .map(|e| format!(" \\ {e}"))
        .chain(restrictions.follow.iter().map(|r| format!(" !>> {r}")))
        .chain(
            restrictions
                .layout_aware_follow
                .iter()
                .map(|r| format!(" !>>> {r}")),
        )
        .join("");
    (before, after)
}

/// Escapes text for Iggy string literals, using exactly the escape set the
/// Iggy grammar accepts: `\"`, `\\`, `\n`, `\t`, `\r`, and `\f`.
///
/// Every place a literal's text is presented goes through this escaping:
/// terminal names (parse-tree labels, parse errors) and the Display of
/// literal symbols (printed grammars). The presentation text is
/// syntactically valid Iggy notation, so a literal read from a parse tree,
/// an error message, or a printed grammar can be pasted back into a
/// grammar.
pub fn escape_literal(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            '\u{C}' => escaped.push_str("\\f"),
            _ => escaped.push(c),
        }
    }
    escaped
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Labeled { label, symbol } => write!(f, "{label}:{symbol}"),
            Symbol::Literal(literal) => write!(f, "\"{}\"", escape_literal(literal)),
            Symbol::Identifier(identifier) => write!(f, "{}", identifier.name),
            Symbol::Group(symbols) => write!(f, "({})", symbols.iter().join(" ")),
            Symbol::Opt(opt) => write!(f, "{opt}?"),
            // A Group symbol inside an Alt renders without its own
            // parentheses, as in display_name above.
            Symbol::Alt(symbols) => {
                let rendered = symbols
                    .iter()
                    .map(|symbol| match symbol {
                        Symbol::Group(seq) => seq.iter().join(" "),
                        single => single.to_string(),
                    })
                    .join(" | ");
                write!(f, "({rendered})")
            }
            Symbol::Star(symbol, sep) => match sep {
                Some(sep) => write!(f, "{{{symbol} {sep}}}*"),
                None => write!(f, "{symbol}*"),
            },
            Symbol::Plus(symbol, sep) => match sep {
                Some(sep) => write!(f, "{{{symbol} {sep}}}+"),
                None => write!(f, "{symbol}+"),
            },
            Symbol::Call { name, arguments } => {
                write!(f, "{}({})", name, arguments.iter().join(", "))
            }
            Symbol::Restricted {
                symbol,
                restrictions,
            } => {
                let (before, after) = display_restrictions(restrictions);
                write!(f, "{before}{symbol}{after}")
            }
            Symbol::Exclude { symbol, labels } => {
                write!(f, "{}", symbol)?;
                for label in labels {
                    write!(f, " !{}", label)?;
                }
                Ok(())
            }
            Symbol::Condition(expr) => write!(f, "[{}]", expr),
            Symbol::Return(expr) => write!(f, "return {}", expr),
            Symbol::Binding { name, symbol } => write!(f, "{}={}", name, symbol),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefinitionId(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
    pub definition: Option<DefinitionId>,
}

impl Identifier {
    pub fn resolve(&self) -> DefinitionId {
        self.definition
            .unwrap_or_else(|| panic!("unresolved identifier {}", self))
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A terminal represents a lexical definition.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Terminal {
    pub name: String,
}

impl Terminal {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// The terminal of a literal occurrence, named by its quoted, escaped
    /// text: the terminal for `else` is named `"else"`.
    pub fn literal(text: &str) -> Self {
        Self::new(format!("\"{}\"", escape_literal(text)))
    }

    pub fn is_literal(&self) -> bool {
        self.name.starts_with('"')
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// The `name` uniquely identifies the nonterminal in the grammar.
/// Origin tracks how the nonterminal is created, e.g., from EBNF to BNF conversion.
/// If origin is None, it's not a derived nonterminal
#[derive(Debug, Clone)]
pub struct Nonterminal {
    pub name: String,
    pub origin: Option<Symbol>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: ParamType,
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

#[derive(Debug, Clone)]
pub enum ParamType {
    I32,
}

impl Display for ParamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamType::I32 => write!(f, "i32"),
        }
    }
}

impl ToTokens for ParamType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let quote = match self {
            ParamType::I32 => quote! { i32 },
        };
        tokens.extend(quote);
    }
}

impl Nonterminal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            origin: None,
            parameters: vec![],
        }
    }

    pub fn with_params(name: impl Into<String>, parameters: Vec<Parameter>) -> Self {
        Self {
            name: name.into(),
            origin: None,
            parameters,
        }
    }

    pub fn with_origin(name: impl Into<String>, origin: Symbol) -> Self {
        Self {
            name: name.into(),
            origin: Some(origin),
            parameters: vec![],
        }
    }

    /// Returns true if the nonterminal was generated when converting from an EBNF Plus.
    pub fn is_plus(&self) -> bool {
        match &self.origin {
            Some(s) => matches!(s, Symbol::Plus(_, _)),
            None => false,
        }
    }

    /// Returns true if the nonterminal was generated when converting from an EBNF Star.
    pub fn is_star(&self) -> bool {
        match &self.origin {
            Some(s) => matches!(s, Symbol::Star(_, _)),
            None => false,
        }
    }

    /// Returns true if the nonterminal was generated when converting from an EBNF Group.
    pub fn is_group(&self) -> bool {
        match &self.origin {
            Some(s) => matches!(s, Symbol::Group(_)),
            None => false,
        }
    }

    // For normal nonterminals, i.e., the ones that are defined by the user directly,
    // the display_name is the same the nonterminal name.
    // For other nonterminals that are generated during grammar transformations,
    // `display_name` shows a name that reflects the structure, rather than the unique,
    // synthetic name used for code generation.
    // For example, for the rule S : A (B|C)+ C, the display name is (B|C)+, while the
    // name is S_Plus_0.
    pub fn display_name(&self) -> String {
        match &self.origin {
            Some(symbol) => symbol.to_string(),
            None => self.name.clone(),
        }
    }

    /// Returns true if the nonterminal was generated from an EBNF operator (Plus, Star, Opt, Group, Alt).
    pub fn is_ebnf(&self) -> bool {
        matches!(
            &self.origin,
            Some(
                Symbol::Plus(_, _)
                    | Symbol::Star(_, _)
                    | Symbol::Opt(_)
                    | Symbol::Group(_)
                    | Symbol::Alt(_)
            )
        )
    }

    /// Returns true if the nonterminal is derived, e.g., from the EBNF to BNF conversion.
    pub fn is_derived(&self) -> bool {
        self.origin.is_some()
    }
}

impl hash::Hash for Nonterminal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Nonterminal {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Nonterminal {}

impl Display for Nonterminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.parameters.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}({})", self.name, self.parameters.iter().join(", "))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Opt {
    symbol: Symbol,
}

impl Display for Opt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?", self.symbol)
    }
}

#[macro_export]
macro_rules! labeled {
    ($label:literal, $symbol:expr) => {
        $crate::grammar::symbols::Symbol::Labeled {
            label: $label.into(),
            symbol: Box::new($symbol),
        }
    };
}

#[macro_export]
macro_rules! id {
    ($name:expr) => {
        $crate::grammar::symbols::Symbol::Identifier($crate::grammar::symbols::Identifier {
            name: $name.into(),
            definition: None,
        })
    };
}

#[macro_export]
macro_rules! lit {
    ($name:literal) => {
        $crate::grammar::symbols::Symbol::literal($name)
    };
}

#[macro_export]
macro_rules! alt {
    ($($symbol:expr),* $(,)?) => {
        $crate::grammar::symbols::Symbol::Alt(vec![$($symbol),*])
    };
}

#[macro_export]
macro_rules! group {
    ($($symbol:expr),* $(,)?) => {
        $crate::grammar::symbols::Symbol::Group(vec![$($symbol),*])
    };
}

#[macro_export]
macro_rules! plus {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Plus(Box::new($symbol), None)
    };
    ($symbol:expr, $sep:expr) => {
        $crate::grammar::symbols::Symbol::Plus(Box::new($symbol), Some(Box::new($sep)))
    };
}

#[macro_export]
macro_rules! star {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Star(Box::new($symbol), None)
    };
    ($symbol:expr, $sep:expr) => {
        $crate::grammar::symbols::Symbol::Star(Box::new($symbol), Some(Box::new($sep)))
    };
}

#[macro_export]
macro_rules! opt {
    ($symbol:expr) => {
        $crate::grammar::symbols::Symbol::Opt(Box::new($symbol))
    };
}

#[macro_export]
macro_rules! except {
    ($symbol:expr, $($except:expr),+ $(,)?) => {
        $crate::grammar::symbols::Symbol::restricted(
            $symbol,
            $crate::grammar::symbols::Restrictions {
                excepts: $crate::restriction_ids!($($except),+),
                ..Default::default()
            },
        )
    };
}

#[macro_export]
macro_rules! follow {
    ($symbol:expr, $($restriction:expr),+ $(,)?) => {
        $crate::grammar::symbols::Symbol::restricted(
            $symbol,
            $crate::grammar::symbols::Restrictions {
                follow: $crate::restriction_ids!($($restriction),+),
                ..Default::default()
            },
        )
    };
}

#[macro_export]
macro_rules! precede {
    ($symbol:expr, $($restriction:expr),+ $(,)?) => {
        $crate::grammar::symbols::Symbol::restricted(
            $symbol,
            $crate::grammar::symbols::Restrictions {
                precede: $crate::restriction_ids!($($restriction),+),
                ..Default::default()
            },
        )
    };
}

/// The identifier list of one restriction kind.
#[macro_export]
macro_rules! restriction_ids {
    ($($name:expr),+ $(,)?) => {
        vec![
            $(
                $crate::grammar::symbols::Identifier {
                    name: $name.into(),
                    definition: None,
                },
            )+
        ]
    };
}

#[macro_export]
macro_rules! exclude {
    ($symbol:expr, $($label:expr),+ $(,)?) => {
        $crate::grammar::symbols::Symbol::Exclude {
            symbol: Box::new($symbol),
            labels: vec![$($label.into()),+],
        }
    };
}

#[macro_export]
macro_rules! call {
    ($name:expr, ref $arg:literal) => {
        $crate::grammar::symbols::Symbol::Call {
            name: $crate::grammar::symbols::Identifier {
                name: $name.into(),
                definition: None,
            },
            arguments: vec![$crate::grammar::symbols::Expr::Ref($arg.into())],
        }
    };
    ($name:expr, $($arg:expr),* $(,)?) => {
        $crate::grammar::symbols::Symbol::Call {
            name: $crate::grammar::symbols::Identifier {
                name: $name.into(),
                definition: None,
            },
            arguments: vec![$($crate::grammar::symbols::Expr::Int($arg)),*],
        }
    };
}

#[macro_export]
macro_rules! cond_expr {
    ($left:literal == $right:literal) => {
        $crate::grammar::symbols::Expr::Cond($crate::grammar::symbols::Cond {
            left: Box::new($crate::grammar::symbols::IntoExpr::into_expr($left)),
            right: Box::new($crate::grammar::symbols::IntoExpr::into_expr($right)),
            op: $crate::grammar::symbols::CondOp::Eq,
        })
    };
    ($left:literal <= $right:literal) => {
        $crate::grammar::symbols::Expr::Cond($crate::grammar::symbols::Cond {
            left: Box::new($crate::grammar::symbols::IntoExpr::into_expr($left)),
            right: Box::new($crate::grammar::symbols::IntoExpr::into_expr($right)),
            op: $crate::grammar::symbols::CondOp::Leq,
        })
    };
    ($left:literal >= $right:literal) => {
        $crate::grammar::symbols::Expr::Cond($crate::grammar::symbols::Cond {
            left: Box::new($crate::grammar::symbols::IntoExpr::into_expr($left)),
            right: Box::new($crate::grammar::symbols::IntoExpr::into_expr($right)),
            op: $crate::grammar::symbols::CondOp::Geq,
        })
    };
}

#[macro_export]
macro_rules! cond {
    (($($c1:tt)*) || ($($c2:tt)*)) => {
        $crate::grammar::symbols::Symbol::Condition($crate::grammar::symbols::Expr::Or(
            Box::new($crate::cond_expr!($($c1)*)),
            Box::new($crate::cond_expr!($($c2)*)),
        ))
    };
    ($left:literal == $right:literal) => {
        $crate::grammar::symbols::Symbol::Condition($crate::cond_expr!($left == $right))
    };
    ($left:literal <= $right:literal) => {
        $crate::grammar::symbols::Symbol::Condition($crate::cond_expr!($left <= $right))
    };
    ($left:literal >= $right:literal) => {
        $crate::grammar::symbols::Symbol::Condition($crate::cond_expr!($left >= $right))
    };
}

#[macro_export]
macro_rules! ternary {
    ($cond:expr, $then:expr, $else:expr) => {
        $crate::grammar::symbols::Expr::Ternary {
            cond: Box::new($cond),
            then: Box::new($crate::grammar::symbols::IntoExpr::into_expr($then)),
            r#else: Box::new($crate::grammar::symbols::IntoExpr::into_expr($else)),
        }
    };
}

#[macro_export]
macro_rules! min {
    ($a:expr, $b:expr) => {
        $crate::grammar::symbols::Expr::Min(
            Box::new($crate::grammar::symbols::IntoExpr::into_expr($a)),
            Box::new($crate::grammar::symbols::IntoExpr::into_expr($b)),
        )
    };
}

#[macro_export]
macro_rules! ret {
    (expr $e:expr) => {
        $crate::grammar::symbols::Symbol::Return($e)
    };
    ($value:expr) => {
        $crate::grammar::symbols::Symbol::Return($crate::grammar::symbols::Expr::Int($value))
    };
}

#[macro_export]
macro_rules! bind {
    ($name:literal, $symbol:expr) => {
        $crate::grammar::symbols::Symbol::Binding {
            name: $name.into(),
            symbol: Box::new($symbol),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(ids: &[Identifier]) -> Vec<&str> {
        ids.iter().map(|id| id.name.as_str()).collect()
    }

    #[test]
    fn test_bare_identifier_has_no_restrictions() {
        let symbol = id!("A");
        assert!(symbol.restrictions().is_empty());
        assert!(!symbol.has_restriction());
    }

    #[test]
    fn test_empty_restrictions_add_no_node() {
        assert_eq!(
            Symbol::restricted(id!("A"), Restrictions::default()),
            id!("A")
        );
    }

    #[test]
    fn test_label_over_follow_restriction() {
        let symbol = labeled!("x", follow!(id!("A"), "B"));
        let restrictions = symbol.restrictions();
        assert_eq!(names(&restrictions.follow), ["B"]);
        assert!(restrictions.excepts.is_empty());
        assert!(restrictions.has_post_conditions());
        assert!(symbol.has_restriction());
    }

    #[test]
    fn test_precede_alone_is_no_post_condition() {
        let symbol = precede!(id!("A"), "X");
        let restrictions = symbol.restrictions();
        assert!(!restrictions.has_post_conditions());
        assert!(!restrictions.is_empty());
    }

    #[test]
    fn test_layout_aware_and_plain_follow_are_kept_separate() {
        let symbol = Symbol::restricted(
            id!("X"),
            Restrictions {
                follow: restriction_ids!("B"),
                layout_aware_follow: restriction_ids!("A"),
                ..Default::default()
            },
        );
        let restrictions = symbol.restrictions();
        assert_eq!(names(&restrictions.layout_aware_follow), ["A"]);
        assert_eq!(names(&restrictions.follow), ["B"]);
    }

    #[test]
    fn test_binding_and_exclude_are_looked_through() {
        let symbol = bind!("n", exclude!(except!(id!("A"), "Kw"), "lbl"));
        let restrictions = symbol.restrictions();
        assert_eq!(names(&restrictions.excepts), ["Kw"]);
        assert!(symbol.has_restriction());
    }

    #[test]
    fn test_display_orders_the_restriction_kinds() {
        let symbol = Symbol::restricted(
            id!("Id"),
            Restrictions {
                precede: restriction_ids!("Digit"),
                excepts: restriction_ids!("Kw"),
                follow: restriction_ids!("Eq"),
                layout_aware_follow: restriction_ids!("Else"),
            },
        );
        assert_eq!(symbol.to_string(), "Digit !<< Id \\ Kw !>> Eq !>>> Else");
    }
}
