use by_address::ByAddress;
use iggy::parse_tree::{self, Layout, ParseTree};
use iguana::grammar::{
    def::{Alternative, GrammarDef, LexicalRule, SyntaxRule},
    symbols::{DefinitionId, Identifier, Symbol},
};
use iguana_runtime::{input::Input, sppf::Span};
use rustc_hash::FxHashMap;

use crate::layout::{leading_comments, trailing_comment};

/// Walk the right spine of `node`, skipping `Layout` children, and return the
/// `right_extent` of the rightmost non-layout `Token` leaf. Parse tree nodes
/// fold trailing layout into their `.span()`, so this walk finds the actual
/// content end. Cost is O(tree depth) per call.
fn rightmost_token_end(node: ParseTree<'_>) -> Option<u32> {
    if let ParseTree::Token(t) = node {
        return Some(t.span().right_extent);
    }
    for child in node.children().iter().rev() {
        if matches!(child, ParseTree::Layout(_)) {
            continue;
        }
        if let Some(end) = rightmost_token_end(*child) {
            return Some(end);
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub span: Option<Span>,
    pub leading_comments: Vec<Span>,
    pub trailing_comment: Option<Span>,
}

#[derive(Default)]
pub struct GrammarSpans<'a> {
    pub syntax_rules: FxHashMap<ByAddress<&'a SyntaxRule>, Metadata>,
    pub lexical_rules: FxHashMap<ByAddress<&'a LexicalRule>, Metadata>,
    pub alternatives: FxHashMap<ByAddress<&'a Alternative>, Metadata>,
    pub symbols: FxHashMap<ByAddress<&'a Symbol>, Metadata>,
    pub identifiers: FxHashMap<ByAddress<&'a Identifier>, Span>,
    /// Maps a DefinitionId to its rule head span.
    pub definition_spans: FxHashMap<DefinitionId, Span>,
    /// Maps a DefinitionId to all the spans where it is referenced in rule bodies.
    pub reference_spans: FxHashMap<DefinitionId, Vec<Span>>,
}

impl<'a> GrammarSpans<'a> {
    pub fn symbol_span(&self, symbol: &'a Symbol) -> Option<Span> {
        self.symbols
            .get(&ByAddress(symbol))
            .and_then(|meta| meta.span)
    }

    pub fn identifier_span(&self, id: &'a Identifier) -> Option<Span> {
        self.identifiers.get(&ByAddress(id)).copied()
    }
}

/// Walks the parse tree top-down to populate GrammarSpans. The walk mirrors
/// the structure of GrammarDef: syntax_idx and lexical_idx track which
/// GrammarDef rule corresponds to the current parse tree rule. This relies
/// on both structures being in the same source order, which is guaranteed
/// by construction (build_grammar in iggy.rs builds GrammarDef from the
/// same parse tree, in the same order).
struct SpanBuilder<'a, 'b> {
    grammar_def: &'a GrammarDef,
    input: &'b Input,
    spans: GrammarSpans<'a>,
    /// The most recently visited non-empty Layout node. Used to attach
    /// leading comments (before a rule) and trailing comments (after recursing
    /// into a rule's children).
    last_layout: Option<&'b Layout<'b>>,
    /// Index into grammar_def.syntax_rules, advanced each time we visit a SyntaxRule.
    syntax_idx: usize,
    /// Index into grammar_def.lexical_rules, advanced each time we visit a RegexRule.
    lexical_idx: usize,
}

impl<'a, 'b> SpanBuilder<'a, 'b> {
    fn walk(&mut self, node: ParseTree<'b>) {
        match node {
            ParseTree::Layout(l) => {
                if !l.span().is_empty() {
                    self.last_layout = Some(l);
                }
            }
            ParseTree::Rule(rule) => match rule {
                parse_tree::Rule::SyntaxRule { syntax_rule, .. } => {
                    let gr_rule = &self.grammar_def.syntax_rules[self.syntax_idx];
                    let head_start = syntax_rule.head.span().left_extent;

                    let leading = self
                        .last_layout
                        .map(|l| leading_comments(l, self.input, head_start))
                        .unwrap_or_default()
                        .iter()
                        .map(|t| t.span())
                        .collect();

                    for child in node.children().iter() {
                        self.walk(*child);
                    }

                    let content_end = rightmost_token_end(syntax_rule.as_parse_tree())
                        .unwrap_or(syntax_rule.span.right_extent);
                    let trailing = self
                        .last_layout
                        .and_then(|l| trailing_comment(l, self.input, content_end))
                        .map(|t| t.span());
                    let rule_span = Span::new(syntax_rule.span.left_extent, content_end);

                    self.spans.syntax_rules.insert(
                        ByAddress(gr_rule),
                        Metadata {
                            span: Some(rule_span),
                            leading_comments: leading,
                            trailing_comment: trailing,
                        },
                    );
                    let head_span = syntax_rule.head.span();
                    let def_id = DefinitionId(
                        (self.grammar_def.lexical_rules.len() + self.syntax_idx) as u16,
                    );
                    self.spans.definition_spans.insert(def_id, head_span);
                    self.syntax_idx += 1;
                }
                parse_tree::Rule::RegexRule { regex_rule, .. } => {
                    let gr_rule = &self.grammar_def.lexical_rules[self.lexical_idx];
                    let head_start = regex_rule.identifier.span().left_extent;

                    let leading = self
                        .last_layout
                        .map(|l| leading_comments(l, self.input, head_start))
                        .unwrap_or_default()
                        .iter()
                        .map(|t| t.span())
                        .collect();

                    for child in node.children().iter() {
                        self.walk(*child);
                    }

                    let content_end = rightmost_token_end(regex_rule.as_parse_tree())
                        .unwrap_or(regex_rule.span.right_extent);
                    let trailing = self
                        .last_layout
                        .and_then(|l| trailing_comment(l, self.input, content_end))
                        .map(|t| t.span());
                    let rule_span = Span::new(regex_rule.span.left_extent, content_end);

                    self.spans.lexical_rules.insert(
                        ByAddress(gr_rule),
                        Metadata {
                            span: Some(rule_span),
                            leading_comments: leading,
                            trailing_comment: trailing,
                        },
                    );
                    let head_span = regex_rule.identifier.span();
                    let def_id = DefinitionId(self.lexical_idx as u16);
                    self.spans.definition_spans.insert(def_id, head_span);
                    self.lexical_idx += 1;
                }
                parse_tree::Rule::Amb(_) => panic!("unexpected ambiguity"),
            },
            _ => {
                for child in node.children().iter() {
                    self.walk(*child);
                }
            }
        }
    }
}

pub fn build_spans<'a>(
    grammar_def: &'a GrammarDef,
    parse_tree: &parse_tree::Start<&parse_tree::Grammar<'_>, &parse_tree::Layout<'_>>,
    input: &Input,
) -> GrammarSpans<'a> {
    let grammar = &parse_tree.node;
    let mut builder = SpanBuilder {
        grammar_def,
        input,
        spans: GrammarSpans::default(),
        last_layout: None,
        syntax_idx: 0,
        lexical_idx: 0,
    };
    builder.walk(grammar.as_parse_tree());

    // Now collect the finer-grained spans (alternatives, symbols) by walking
    // the GrammarDef and parse tree in parallel.
    let mut syntax_idx = 0;
    let mut lexical_idx = 0;
    for rule in grammar.rules.rules() {
        match rule {
            parse_tree::Rule::SyntaxRule { syntax_rule, .. } => {
                let gr_rule = &grammar_def.syntax_rules[syntax_idx];
                collect_syntax_rule_spans(gr_rule, &syntax_rule, &mut builder.spans);
                syntax_idx += 1;
            }
            parse_tree::Rule::RegexRule { .. } => {
                lexical_idx += 1;
            }
            parse_tree::Rule::Amb(_) => panic!("unexpected ambiguity"),
        }
    }

    builder.spans
}

fn collect_syntax_rule_spans<'a>(
    gr_rule: &'a SyntaxRule,
    pt_rule: &parse_tree::SyntaxRule,
    spans: &mut GrammarSpans<'a>,
) {
    for (gr_level, pt_level) in gr_rule
        .priority_levels
        .iter()
        .zip(pt_rule.priority_levels.priority_levels())
    {
        for (gr_alt, pt_alt) in gr_level
            .alternatives
            .iter()
            .zip(pt_level.alternatives.alternatives())
        {
            let alt_end = rightmost_token_end(pt_alt.as_parse_tree())
                .unwrap_or(pt_alt.span.right_extent);
            let alt_span = Span::new(pt_alt.span.left_extent, alt_end);
            spans.alternatives.insert(
                ByAddress(gr_alt),
                Metadata {
                    span: Some(alt_span),
                    ..Default::default()
                },
            );
            for (gr_sym, pt_sym) in gr_alt.symbols.iter().zip(pt_alt.symbols.symbols()) {
                collect_symbol_spans(gr_sym, pt_sym, spans);
            }
        }
    }
}

fn collect_symbol_spans<'a>(
    gr_sym: &'a Symbol,
    pt_sym: &parse_tree::Symbol,
    spans: &mut GrammarSpans<'a>,
) {
    let sym_span = pt_sym.span();
    spans.symbols.insert(
        ByAddress(gr_sym),
        Metadata {
            span: Some(sym_span),
            ..Default::default()
        },
    );
    match gr_sym {
        Symbol::Identifier(id) => {
            spans.identifiers.insert(ByAddress(id), sym_span);
            if let Some(def_id) = id.definition {
                spans.reference_spans.entry(def_id).or_default().push(sym_span);
            }
        }
        Symbol::Call { name, .. } => {
            spans.identifiers.insert(ByAddress(name), sym_span);
            if let Some(def_id) = name.definition {
                spans.reference_spans.entry(def_id).or_default().push(sym_span);
            }
        }
        _ => {}
    }
    match (gr_sym, pt_sym) {
        (Symbol::Star(gr_inner, None), parse_tree::Symbol::Star { symbol, .. })
        | (Symbol::Plus(gr_inner, None), parse_tree::Symbol::Plus { symbol, .. })
        | (Symbol::Opt(gr_inner), parse_tree::Symbol::Opt { symbol, .. }) => {
            collect_symbol_spans(gr_inner, symbol, spans);
        }
        (Symbol::Star(gr_inner, Some(gr_sep)), parse_tree::Symbol::StarSep { symbol, sep, .. })
        | (Symbol::Plus(gr_inner, Some(gr_sep)), parse_tree::Symbol::PlusSep { symbol, sep, .. }) =>
        {
            collect_symbol_spans(gr_inner, symbol, spans);
            collect_symbol_spans(gr_sep, sep, spans);
        }
        (Symbol::Alt(gr_syms), parse_tree::Symbol::Alt { first, rest, .. }) => {
            let pt_syms: Vec<&parse_tree::Symbol> =
                std::iter::once(*first).chain(rest.symbols()).collect();
            for (ir, pt) in gr_syms.iter().zip(pt_syms) {
                collect_symbol_spans(ir, pt, spans);
            }
        }
        (Symbol::Group(gr_syms), parse_tree::Symbol::Group { symbols, .. }) => {
            for (ir, pt) in gr_syms.iter().zip(symbols.symbols()) {
                collect_symbol_spans(ir, pt, spans);
            }
        }
        (Symbol::Labeled { symbol: gr_inner, .. }, parse_tree::Symbol::Labeled { symbol, .. })
        | (
            Symbol::Exclude { symbol: gr_inner, .. },
            parse_tree::Symbol::Exclude { symbol, .. },
        ) => {
            collect_symbol_spans(gr_inner, symbol, spans);
        }
        (
            Symbol::Except { symbol: gr_inner, except, .. },
            parse_tree::Symbol::Except { symbol, excepts, .. },
        ) => {
            collect_symbol_spans(gr_inner, symbol, spans);
            for (id, token) in except.iter().zip(excepts.identifiers()) {
                spans.identifiers.insert(ByAddress(id), token.span());
            }
        }
        (
            Symbol::FollowRestriction { symbol: gr_inner, restrictions: gr_restrictions, .. },
            parse_tree::Symbol::FollowRestriction { symbol, restrictions, .. },
        ) => {
            collect_symbol_spans(gr_inner, symbol, spans);
            for (id, token) in gr_restrictions.iter().zip(restrictions.identifiers()) {
                spans.identifiers.insert(ByAddress(id), token.span());
            }
        }
        (
            Symbol::PrecedeRestriction { symbol: gr_inner, restriction, .. },
            parse_tree::Symbol::PrecedeRestriction { symbol, identifier, .. },
        ) => {
            collect_symbol_spans(gr_inner, symbol, spans);
            spans.identifiers.insert(ByAddress(restriction), identifier.span());
        }
        _ => {}
    }
}
