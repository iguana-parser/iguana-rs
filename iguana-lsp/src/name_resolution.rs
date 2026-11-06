use iguana_compiler::grammar::{
    def::GrammarDef,
    symbols::{DefinitionId, Identifier},
};
use iguana_compiler::spans::GrammarSpans;
use iguana_runtime::input::Span;
use rustc_hash::FxHashMap;

/// A span per definition, the spans of the references to it, and a table of
/// both sorted for lookup by offset.
///
/// `GrammarSpans` answers where a node was written. The name resolution index
/// answers where each resolved name is defined and used.
#[derive(Default)]
pub struct NameResolutionIndex {
    pub definitions: FxHashMap<DefinitionId, Span>,
    pub references: FxHashMap<DefinitionId, Vec<Span>>,
    /// Every rule head and resolved reference in the source, sorted by span.
    /// The spans do not overlap.
    occurrences: Vec<(Span, DefinitionId)>,
}

impl NameResolutionIndex {
    pub fn new(grammar_def: &GrammarDef, spans: &GrammarSpans<'_>) -> Self {
        let mut index = Self::default();

        // The head's own id comes from the symbol table rather than from the
        // rule's position, so the order the ids are assigned in stays in the
        // one place that assigns them.
        let symbol_table = grammar_def.symbol_table();
        for rule in &grammar_def.lexical_rules {
            if let Some(id) = symbol_table.get(&rule.head.name) {
                index
                    .definitions
                    .insert(id, spans.terminal(&rule.head).span);
            }
        }
        for rule in &grammar_def.syntax_rules {
            if let Some(id) = symbol_table.get(&rule.head.name) {
                index
                    .definitions
                    .insert(id, spans.nonterminal(&rule.head).span);
            }
        }

        grammar_def.for_each_identifier(&mut |identifier| index.add_reference(identifier, spans));

        for references in index.references.values_mut() {
            references.sort_by_key(|span| span.left_extent);
        }

        let heads = index
            .definitions
            .iter()
            .map(|(&definition, &span)| (span, definition));
        let references = spans
            .identifiers
            .iter()
            .filter_map(|(identifier, region)| Some((region.span, identifier.definition?)));
        let mut occurrences: Vec<_> = heads.chain(references).collect();
        occurrences.sort_by_key(|(span, _)| span.left_extent);
        debug_assert!(
            occurrences
                .windows(2)
                .all(|pair| pair[0].0.right_extent <= pair[1].0.left_extent),
            "name occurrences overlap",
        );
        index.occurrences = occurrences;
        index
    }

    /// The definition of the name at `offset`, or `None` when `offset` is not
    /// on a resolved name.
    pub fn definition_id_at(&self, offset: u32) -> Option<DefinitionId> {
        let following = self
            .occurrences
            .partition_point(|(span, _)| span.left_extent <= offset);
        let &(span, definition) = self.occurrences[..following].last()?;
        (offset < span.right_extent).then_some(definition)
    }

    fn add_reference(&mut self, identifier: &Identifier, spans: &GrammarSpans<'_>) {
        let Some(definition) = identifier.definition else {
            return;
        };
        self.references
            .entry(definition)
            .or_default()
            .push(spans.identifier(identifier).span);
    }
}
