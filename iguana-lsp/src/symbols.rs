use iguana_compiler::grammar::symbols::DefinitionId;

use crate::spans::GrammarSpans;

/// Determine the DefinitionId of the symbol at the given byte offset.
/// Checks rule heads first, then all identifier references.
pub fn find_definition_at_offset(spans: &GrammarSpans<'_>, offset: u32) -> Option<DefinitionId> {
    for (&def_id, &span) in &spans.definition_spans {
        if offset >= span.left_extent && offset < span.right_extent {
            return Some(def_id);
        }
    }

    for (id, &span) in &spans.identifiers {
        if offset >= span.left_extent && offset < span.right_extent {
            return id.definition;
        }
    }

    None
}
