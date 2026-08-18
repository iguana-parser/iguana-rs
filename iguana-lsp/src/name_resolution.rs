use iguana_compiler::grammar::{
    def::GrammarDef,
    symbols::{DefinitionId, Identifier},
};
use iguana_runtime::input::Span;
use rustc_hash::FxHashMap;

use crate::spans::GrammarSpans;

/// A span per definition, and the spans of the references to it.
///
/// `GrammarSpans` answers where a node was written. The name resolution index
/// answers where each resolved name is defined and used. Only the language
/// server features that navigate definitions and references build the index.
#[derive(Default)]
pub struct NameResolutionIndex {
    pub definitions: FxHashMap<DefinitionId, Span>,
    pub references: FxHashMap<DefinitionId, Vec<Span>>,
}

impl NameResolutionIndex {
    pub fn new(grammar_def: &GrammarDef, spans: &GrammarSpans<'_>) -> Self {
        let mut index = Self::default();

        // The head's own id comes from the symbol table rather than from the
        // rule's position, so the order the ids are assigned in stays in the
        // one place that assigns them.
        let symbol_table = grammar_def.symbol_table();
        for rule in &grammar_def.lexical_rules {
            if let Some(id) = symbol_table.get(&rule.head.name)
                && let Some(region) = spans.terminal(&rule.head)
            {
                index.definitions.insert(id, region.span);
            }
        }
        for rule in &grammar_def.syntax_rules {
            if let Some(id) = symbol_table.get(&rule.head.name)
                && let Some(region) = spans.nonterminal(&rule.head)
            {
                index.definitions.insert(id, region.span);
            }
        }

        grammar_def.for_each_identifier(&mut |identifier| index.add_reference(identifier, spans));

        for references in index.references.values_mut() {
            references.sort_by_key(|span| span.left_extent);
        }
        index
    }

    fn add_reference(&mut self, identifier: &Identifier, spans: &GrammarSpans<'_>) {
        let Some(definition) = identifier.definition else {
            return;
        };
        let Some(region) = spans.identifier(identifier) else {
            return;
        };
        self.references
            .entry(definition)
            .or_default()
            .push(region.span);
    }
}
