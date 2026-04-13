use by_address::ByAddress;
use iguana::grammar::def::GrammarDef;
use iguana::grammar::symbols::{DefinitionId, Symbol};

use crate::spans::GrammarSpans;

/// Determine the DefinitionId of the symbol at the given byte offset.
/// First checks rule heads (definitions), then descends into rule bodies.
pub fn find_definition_at_offset(
    grammar_def: &GrammarDef,
    spans: &GrammarSpans<'_>,
    offset: u32,
) -> Option<DefinitionId> {
    let num_lexical = grammar_def.lexical_rules.len();

    // Check syntax rule heads.
    for (i, _) in grammar_def.syntax_rules.iter().enumerate() {
        let def_id = DefinitionId((num_lexical + i) as u16);
        if let Some(head_span) = spans.definition_spans.get(&def_id) {
            if offset >= head_span.left_extent && offset < head_span.right_extent {
                return Some(def_id);
            }
        }
    }

    // Check lexical rule heads.
    for (i, _) in grammar_def.lexical_rules.iter().enumerate() {
        let def_id = DefinitionId(i as u16);
        if let Some(head_span) = spans.definition_spans.get(&def_id) {
            if offset >= head_span.left_extent && offset < head_span.right_extent {
                return Some(def_id);
            }
        }
    }

    // Descend into syntax rule bodies.
    for rule in &grammar_def.syntax_rules {
        for level in &rule.priority_levels {
            for alt in &level.alternatives {
                for sym in &alt.symbols {
                    if let Some(def_id) = find_identifier_at_offset(sym, spans, offset) {
                        return Some(def_id);
                    }
                }
            }
        }
    }

    None
}

/// Recursively descend into a Symbol tree to find the DefinitionId at `offset`.
fn find_identifier_at_offset(
    sym: &Symbol,
    spans: &GrammarSpans<'_>,
    offset: u32,
) -> Option<DefinitionId> {
    let meta = spans.symbols.get(&ByAddress(sym))?;
    let span = meta.span?;
    if offset < span.left_extent || offset >= span.right_extent {
        return None;
    }

    match sym {
        Symbol::Identifier(id) => id.definition,
        Symbol::Call { name, .. } => name.definition,
        Symbol::Labeled { symbol, .. }
        | Symbol::Binding { symbol, .. }
        | Symbol::Opt(symbol)
        | Symbol::Except { symbol, .. }
        | Symbol::FollowRestriction { symbol, .. }
        | Symbol::PrecedeRestriction { symbol, .. }
        | Symbol::Exclude { symbol, .. } => find_identifier_at_offset(symbol, spans, offset),
        Symbol::Star(inner, sep) | Symbol::Plus(inner, sep) => {
            if let Some(def_id) = find_identifier_at_offset(inner, spans, offset) {
                return Some(def_id);
            }
            if let Some(sep) = sep {
                return find_identifier_at_offset(sep, spans, offset);
            }
            None
        }
        Symbol::Group(syms) | Symbol::Alt(syms) => {
            for s in syms {
                if let Some(def_id) = find_identifier_at_offset(s, spans, offset) {
                    return Some(def_id);
                }
            }
            None
        }
        Symbol::Literal(_) | Symbol::Condition(_) | Symbol::Return(_) => None,
    }
}
