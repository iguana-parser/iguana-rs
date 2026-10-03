use serde::{Deserialize, Serialize};

use crate::ids::{GssNodeId, NonterminalId, SlotId, TerminalId};
use crate::input::Span;
use crate::parser::GLLFailureKind;
use crate::sppf::SPPFNodeId;

#[cfg(feature = "debug-trace")]
use crate::{grammar::Grammar, parser::Parser};

/// The kind of a failure in a trace. The terminal ids are stored as an owned
/// vector, instead of a pointer to the static in `GLLFailureKind`. This is
/// because a program that reads a trace file, such as Terrarium, does not
/// have that static.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceFailureKind {
    /// Terminal match failed: expected one of these terminals at this position.
    UnexpectedToken { expected: Vec<TerminalId> },
    /// Nonterminal except (`\`): matched a nonterminal but it was excluded.
    ExcludedMatch { excluded_by: Vec<TerminalId> },
    /// Follow restriction (`!>>`): the symbol after the match is forbidden.
    ForbiddenFollow { forbidden: Vec<TerminalId> },
}

impl From<GLLFailureKind> for TraceFailureKind {
    fn from(kind: GLLFailureKind) -> Self {
        let terminals = kind.terminals().to_vec();
        match kind {
            GLLFailureKind::UnexpectedToken(_) => Self::UnexpectedToken {
                expected: terminals,
            },
            GLLFailureKind::ExcludedMatch(_) => Self::ExcludedMatch {
                excluded_by: terminals,
            },
            GLLFailureKind::ForbiddenFollow(_) => Self::ForbiddenFollow {
                forbidden: terminals,
            },
        }
    }
}

/// Trace events emitted during GLL parsing. Serializable and deserializable
/// in every build; recording them requires the `debug-trace` feature.
#[derive(Debug, Serialize, Deserialize)]
pub enum TraceEvent {
    ProcessingDescriptor(SlotId, u32, GssNodeId, Option<SPPFNodeId>),
    DescriptorAdded(SlotId, u32, GssNodeId, Option<SPPFNodeId>),
    MatchingLeadingLayout(u32),
    MatchingTrailingLayout(u32),
    MatchingTerminal(TerminalId, u32),  // terminal, input_index
    MatchSuccess(TerminalId, u32, u32), // terminal, input_index, next_input match
    GLLFailure(u32, SlotId, Option<GssNodeId>, TraceFailureKind), // input_index, slot, gss_node, kind
    MatchedLayout(Option<u32>),                                   // next_input match
    GSSNodeCreated(NonterminalId, u32),
    GSSNodeFound(NonterminalId, u32),
    GSSNodeNotFound(NonterminalId, u32),
    GSSNodeAdded(GssNodeId, GssNodeId, SlotId), // (src, dest)
    TerminalNodeCreated(TerminalId, Span),
    NonterminalNodeCreated(NonterminalId, Span, SPPFNodeId),
    IntermediateNodeCreated(SlotId, Span, SPPFNodeId, SPPFNodeId), // (slot_id, span, left_child, right_child)
    TerminalNodeFound(SPPFNodeId),
    NonterminalNodeFound(SPPFNodeId),
    IntermediateNodeFound(SPPFNodeId),
    Pop(GssNodeId, SlotId, SPPFNodeId, Option<i32>),
    AddToPoppedElements(GssNodeId, SPPFNodeId, Option<i32>),
    NodeAlreadyInPoppedElements,
    Call(Option<SPPFNodeId>, GssNodeId, SlotId),
}

#[cfg(feature = "debug-trace")]
impl TraceEvent {
    pub fn message<'i, 'arena, P: Parser<'i, 'arena>>(&self, parser: &P) -> String {
        match *self {
            TraceEvent::ProcessingDescriptor(slot_id, input_index, gss_node_id, sppf_node_id) => {
                format!(
                    "Processing ({}, {}, {}, {})",
                    P::Grammar::slot_name(slot_id),
                    input_index,
                    parser.gss_to_string(gss_node_id),
                    if let Some(sppf_node_id) = sppf_node_id {
                        parser.sppf_node_to_string(parser.sppf_node(sppf_node_id))
                    } else {
                        "$".to_string()
                    }
                )
            }
            TraceEvent::DescriptorAdded(slot_id, input_index, gss_node_id, sppf_node_id) => {
                format!(
                    "Descriptor ({}, {}, {}, {}) added.",
                    P::Grammar::slot_name(slot_id),
                    input_index,
                    parser.gss_to_string(gss_node_id),
                    if let Some(sppf_node_id) = sppf_node_id {
                        parser.sppf_node_to_string(parser.sppf_node(sppf_node_id))
                    } else {
                        "$".to_string()
                    }
                )
            }
            TraceEvent::MatchingLeadingLayout(input_index) => {
                format!("Matching leading layout at input index {}", input_index)
            }
            TraceEvent::MatchingTrailingLayout(input_index) => {
                format!("Matching trailing layout at input index {}", input_index)
            }
            TraceEvent::MatchingTerminal(terminal_id, input_index) => format!(
                "Matched terminal {} at input index {}",
                P::Grammar::terminal_name(terminal_id),
                input_index,
            ),
            TraceEvent::MatchSuccess(terminal_id, input_index, matched_index) => format!(
                "Matched terminal {} at input index {}. Match length: {}",
                P::Grammar::terminal_name(terminal_id),
                input_index,
                matched_index - input_index
            ),
            TraceEvent::GLLFailure(input_index, slot_id, gss_node_id, ref kind) => {
                let gss = gss_node_id
                    .map(|id| parser.gss_to_string(id))
                    .unwrap_or_else(|| "?".to_string());
                let (description, terminals) = match kind {
                    TraceFailureKind::UnexpectedToken { expected } => ("expected", expected),
                    TraceFailureKind::ExcludedMatch { excluded_by } => ("excluded by", excluded_by),
                    TraceFailureKind::ForbiddenFollow { forbidden } => {
                        ("forbidden follow", forbidden)
                    }
                };
                let names: Vec<&str> = terminals
                    .iter()
                    .map(|id| P::Grammar::terminal_name(*id))
                    .collect();
                let kind_str = format!("{description} {}", names.join(", "));
                format!(
                    "Parse error at input index {} (slot: {}, GSS node: {}): {}",
                    input_index,
                    P::Grammar::slot_name(slot_id),
                    gss,
                    kind_str
                )
            }
            TraceEvent::MatchedLayout(matched_index) => {
                if let Some(matched_index) = matched_index {
                    format!("Matched layout. New input index is: {}", matched_index)
                } else {
                    "No layout found".into()
                }
            }
            TraceEvent::GSSNodeCreated(nonterminal_id, input_index) => format!(
                "GSS node ({},{}) created",
                P::Grammar::nonterminal_display_name(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeFound(nonterminal_id, input_index) => format!(
                "GSS node ({},{}) found",
                P::Grammar::nonterminal_display_name(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeNotFound(nonterminal_id, input_index) => format!(
                "GSS node ({},{}) not found",
                P::Grammar::nonterminal_display_name(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeAdded(origin_gss_node_id, dest_gss_node_id, return_slot) => format!(
                "GSS edge added from {} to {} with return label {}",
                parser.gss_to_string(origin_gss_node_id),
                parser.gss_to_string(dest_gss_node_id),
                P::Grammar::slot_name(return_slot)
            ),
            TraceEvent::TerminalNodeCreated(terminal_id, span) => format!(
                "Terminal node created: ({}, {}, {})",
                P::Grammar::terminal_name(terminal_id),
                span.left_extent,
                span.right_extent
            ),
            TraceEvent::NonterminalNodeCreated(nonterminal_id, span, child) => format!(
                "Nonterminal node created: ({}, {}, {}, {})",
                P::Grammar::nonterminal_display_name(nonterminal_id),
                span.left_extent,
                span.right_extent,
                parser.sppf_node_to_string(parser.sppf_node(child)),
            ),
            TraceEvent::IntermediateNodeCreated(slot_id, span, left_child, right_child) => format!(
                "Intermediate node created: ({}, {}, {}, {}, {})",
                P::Grammar::slot_name(slot_id),
                span.left_extent,
                span.right_extent,
                parser.sppf_node_to_string(parser.sppf_node(left_child)),
                parser.sppf_node_to_string(parser.sppf_node(right_child))
            ),
            TraceEvent::TerminalNodeFound(sppf_node_id) => format!(
                "Terminal node found: {}",
                parser.sppf_node_to_string(parser.sppf_node(sppf_node_id))
            ),
            TraceEvent::NonterminalNodeFound(sppf_node_id) => format!(
                "Nonterminal node found: {}",
                parser.sppf_node_to_string(parser.sppf_node(sppf_node_id))
            ),
            TraceEvent::IntermediateNodeFound(sppf_node_id) => format!(
                "Intermediate node found: {}",
                parser.sppf_node_to_string(parser.sppf_node(sppf_node_id))
            ),
            TraceEvent::Pop(gss_node_id, slot_id, nonterminal_node_id, return_value) => {
                match return_value {
                    Some(return_value) => format!(
                        "Pop GSS node {} for the slot {} with SPPF node {} and return value {}",
                        parser.gss_to_string(gss_node_id),
                        P::Grammar::slot_name(slot_id),
                        parser.sppf_node_to_string(parser.sppf_node(nonterminal_node_id)),
                        return_value
                    ),
                    None => format!(
                        "Pop GSS node {} for the slot {} with SPPF node {}",
                        parser.gss_to_string(gss_node_id),
                        P::Grammar::slot_name(slot_id),
                        parser.sppf_node_to_string(parser.sppf_node(nonterminal_node_id))
                    ),
                }
            }
            TraceEvent::AddToPoppedElements(gss_node_id, nonterminal_node_id, return_value) => {
                match return_value {
                    Some(return_value) => format!(
                        "Added {} to {}'s popped elements with return value {}",
                        parser.sppf_node_to_string(parser.sppf_node(nonterminal_node_id)),
                        parser.gss_to_string(gss_node_id),
                        return_value
                    ),
                    None => format!(
                        "Added {} to {}'s popped elements",
                        parser.sppf_node_to_string(parser.sppf_node(nonterminal_node_id)),
                        parser.gss_to_string(gss_node_id)
                    ),
                }
            }
            TraceEvent::NodeAlreadyInPoppedElements => {
                "Node already in popped elements".to_string()
            }
            TraceEvent::Call(sppf_node_id, gss_node_id, slot_id) => format!(
                "Call {}, {}, {}",
                sppf_node_id
                    .map(|sppf_node_id| parser.sppf_node_to_string(parser.sppf_node(sppf_node_id)))
                    .unwrap_or("$".to_owned()),
                parser.gss_to_string(gss_node_id),
                P::Grammar::slot_name(slot_id)
            ),
        }
    }
}

#[macro_export]
#[cfg(feature = "debug-trace")]
macro_rules! record {
    ($parser:expr, ProcessingDescriptor, $input_index:expr, $slot_id:expr, $sppf_node_id:expr, $gss_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::ProcessingDescriptor(
            $slot_id,
            $input_index,
            $gss_node_id,
            $sppf_node_id,
        ));
    };
    ($parser:expr, DescriptorAdded, $input_index:expr, $slot_id:expr, $sppf_node_id:expr, $gss_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::DescriptorAdded(
            $slot_id,
            $input_index,
            $gss_node_id,
            $sppf_node_id,
        ));
    };
    ($parser:expr, MatchingLeadingLayout, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchingLeadingLayout(
            $input_index,
        ));
    };
    ($parser:expr, MatchingTrailingLayout, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchingTrailingLayout(
            $input_index,
        ));
    };
    ($parser:expr, MatchingTerminal, $terminal_id:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchingTerminal(
            $terminal_id,
            $input_index,
        ));
    };
    ($parser:expr, MatchSuccess, $terminal_id:expr, $input_index:expr, $next_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchSuccess(
            $terminal_id,
            $input_index,
            $next_index,
        ));
    };
    ($parser:expr, GLLFailure, $input_index:expr, $slot_id:expr, $gss_node_id:expr, $kind:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GLLFailure(
            $input_index,
            $slot_id,
            $gss_node_id,
            $crate::trace::TraceFailureKind::from($kind),
        ));
    };
    ($parser:expr, MatchedLayout, $match_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchedLayout($match_index));
    };
    ($parser:expr, GSSNodeCreated, $nonterminal_id:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeCreated(
            $nonterminal_id,
            $input_index,
        ));
    };
    ($parser:expr, GSSNodeFound, $nonterminal_id:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeFound(
            $nonterminal_id,
            $input_index,
        ));
    };
    ($parser:expr, GSSNodeNotFound, $nonterminal_id:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeNotFound(
            $nonterminal_id,
            $input_index,
        ));
    };
    ($parser:expr, GSSNodeAdded, $origin_gss_node_id:expr, $dest_gss_node_id:expr, $return_slot:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeAdded(
            $origin_gss_node_id,
            $dest_gss_node_id,
            $return_slot,
        ));
    };
    ($parser:expr, TerminalNodeCreated, $terminal_id:expr, $span:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::TerminalNodeCreated(
            $terminal_id,
            $span,
        ));
    };
    ($parser:expr, NonterminalNodeCreated, $nonterminal_id:expr, $span:expr, $child:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::NonterminalNodeCreated(
            $nonterminal_id,
            $span,
            $child,
        ));
    };
    ($parser:expr, IntermediateNodeCreated, $slot_id:expr, $span:expr, $left_child:expr, $right_child:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::IntermediateNodeCreated(
            $slot_id,
            $span,
            $left_child,
            $right_child,
        ));
    };
    ($parser:expr, TerminalNodeFound, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::TerminalNodeFound($sppf_node_id));
    };
    ($parser:expr, NonterminalNodeFound, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::NonterminalNodeFound(
            $sppf_node_id,
        ));
    };
    ($parser:expr, IntermediateNodeFound, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::IntermediateNodeFound(
            $sppf_node_id,
        ));
    };
    ($parser:expr, Pop, $gss_node_id:expr, $slot_id:expr, $sppf_node_id:expr, $return_value:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::Pop(
            $gss_node_id,
            $slot_id,
            $sppf_node_id,
            $return_value,
        ));
    };
    ($parser:expr, AddToPoppedElements, $gss_node_id:expr, $sppf_node_id:expr, $return_value:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::AddToPoppedElements(
            $gss_node_id,
            $sppf_node_id,
            $return_value,
        ));
    };
    ($parser:expr, NodeAlreadyInPoppedElements) => {
        $parser.add_trace_event($crate::trace::TraceEvent::NodeAlreadyInPoppedElements);
    };
    ($parser:expr, Call, $sppf_node_id:expr, $gss_node_id:expr, $slot_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::Call(
            $sppf_node_id,
            $gss_node_id,
            $slot_id,
        ));
    };
}

#[macro_export]
#[cfg(not(feature = "debug-trace"))]
macro_rules! record {
    ($self:expr, $kind:ident $(, $args:expr)*) => {};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::TerminalSet;

    static SET: TerminalSet = TerminalSet {
        id: 0,
        terminals: &[TerminalId(1), TerminalId(2)],
    };

    #[test]
    fn a_failure_event_reads_back_with_its_terminals() {
        let kind = TraceFailureKind::from(GLLFailureKind::ExcludedMatch(&SET));
        let events = vec![TraceEvent::GLLFailure(3, SlotId(4), None, kind.clone())];
        let json = serde_json::to_string(&events).unwrap();
        let read: Vec<TraceEvent> = serde_json::from_str(&json).unwrap();
        let [TraceEvent::GLLFailure(3, SlotId(4), None, read_kind)] = read.as_slice() else {
            panic!("expected the failure event back, got {read:?}");
        };
        assert_eq!(*read_kind, kind);
        assert_eq!(
            kind,
            TraceFailureKind::ExcludedMatch {
                excluded_by: vec![TerminalId(1), TerminalId(2)],
            }
        );
    }
}
