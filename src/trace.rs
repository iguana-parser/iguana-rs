use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::ids::{GssNodeId, NonterminalId, SlotId, TerminalId};
use crate::sppf::{SPPFNodeId, Span};

#[cfg(feature = "debug-trace")]
use crate::parser::Parser;

/// Trace events emitted during GLL parsing.
/// Always available for deserialization; runtime tracing requires `debug-trace` feature.
#[derive(Debug, Serialize, Deserialize)]
pub enum TraceEvent {
    ProcessingDescriptor(SlotId, u32, GssNodeId, Option<SPPFNodeId>),
    DescriptorAdded(SlotId, u32, GssNodeId, Option<SPPFNodeId>),
    MatchingLeadingLayout(u32),
    MatchingTrailingLayout(u32),
    MatchingTerminal(String, u32),  // terminal_name
    MatchSuccess(String, u32, u32), // terminal_name, next_input match
    MatchFailed(String, u32, SlotId, GssNodeId, Option<SPPFNodeId>), // terminal_name, input_index, slot, gss_node, sppf_node
    MatchedLayout(Option<u32>),     // next_input match
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
    Pop(GssNodeId, SlotId, SPPFNodeId),
    AddToPoppedElements(GssNodeId, SPPFNodeId),
    NodeAlreadyInPoppedElements,
    Call(Option<SPPFNodeId>, GssNodeId, SlotId),
    ParseSuccess(Duration),
    ParseFailed(Duration),
}

#[cfg(feature = "debug-trace")]
impl TraceEvent {
    pub fn message<'i, P: Parser<'i>>(&self, parser: &P) -> String {
        match *self {
            TraceEvent::ProcessingDescriptor(slot_id, input_index, gss_node_id, sppf_node_id) => {
                format!(
                    "Processing ({}, {}, {}, {})",
                    P::slot(slot_id).name,
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
                    P::slot(slot_id).name,
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
            TraceEvent::MatchingTerminal(ref terminal_name, input_index) => format!(
                "Matched terminal {terminal_name} at input index {}",
                input_index,
            ),
            TraceEvent::MatchSuccess(ref terminal_name, input_index, matched_index) => format!(
                "Matched terminal {terminal_name} at input index {}. Match length: {}",
                input_index,
                matched_index - input_index
            ),
            TraceEvent::MatchFailed(ref terminal_name, input_index, slot_id, gss_node_id, sppf_node_id) => {
                format!(
                    "Match failed for terminal {} at input index {} (slot: {}, GSS node: {}, SPPF node: {})",
                    terminal_name,
                    input_index,
                    P::slot(slot_id).name,
                    parser.gss_to_string(gss_node_id),
                    sppf_node_id
                        .map(|id| parser.sppf_node_to_string(parser.sppf_node(id)))
                        .unwrap_or_else(|| "$".to_string())
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
                P::nonterminal(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeFound(nonterminal_id, input_index) => format!(
                "GSS node ({},{}) found",
                P::nonterminal(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeNotFound(nonterminal_id, input_index) => format!(
                "GSS node ({},{}) not found",
                P::nonterminal(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeAdded(origin_gss_node_id, dest_gss_node_id, return_slot) => format!(
                "GSS edge added from {} to {} with return label {}",
                parser.gss_to_string(origin_gss_node_id),
                parser.gss_to_string(dest_gss_node_id),
                P::slot(return_slot).name
            ),
            TraceEvent::TerminalNodeCreated(terminal_id, span) => format!(
                "Terminal node created: ({}, {}, {})",
                P::terminal(terminal_id),
                span.left_extent,
                span.right_extent
            ),
            TraceEvent::NonterminalNodeCreated(nonterminal_id, span, child) => format!(
                "Nonterminal node created: ({}, {}, {}, {})",
                P::nonterminal(nonterminal_id),
                span.left_extent,
                span.right_extent,
                parser.sppf_node_to_string(parser.sppf_node(child)),
            ),
            TraceEvent::IntermediateNodeCreated(slot_id, span, left_child, right_child) => format!(
                "Intermediate node created: ({}, {}, {}, {}, {})",
                P::slot(slot_id).name,
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
            TraceEvent::Pop(gss_node_id, slot_id, sppf_node_id) => format!(
                "Pop GSS node {} for the slot {} with SPPF node {}",
                parser.gss_to_string(gss_node_id),
                P::slot(slot_id).name,
                parser.sppf_node_to_string(parser.sppf_node(sppf_node_id))
            ),
            TraceEvent::AddToPoppedElements(gss_node_id, sppf_node_id) => format!(
                "Added {} to {}'s popped elements",
                parser.sppf_node_to_string(parser.sppf_node(sppf_node_id)),
                parser.gss_to_string(gss_node_id)
            ),
            TraceEvent::NodeAlreadyInPoppedElements => {
                "Node already in popped elements".to_string()
            }
            TraceEvent::Call(sppf_node_id, gss_node_id, slot_id) => format!(
                "Call {}, {}, {}",
                sppf_node_id
                    .map(|sppf_node_id| parser.sppf_node_to_string(parser.sppf_node(sppf_node_id)))
                    .unwrap_or("$".to_owned()),
                parser.gss_to_string(gss_node_id),
                P::slot(slot_id).name
            ),
            TraceEvent::ParseSuccess(duration) => {
                format!("Parse succeeded in {:?} ms", duration.as_millis())
            }
            TraceEvent::ParseFailed(duration) => {
                format!("Parse failed in {:?}", duration.as_millis())
            }
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
    ($parser:expr, MatchingTerminal, $terminal_name:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchingTerminal(
            $terminal_name.into(),
            $input_index,
        ));
    };
    ($parser:expr, MatchSuccess, $terminal_name:expr, $input_index:expr, $next_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchSuccess(
            $terminal_name.into(),
            $input_index,
            $next_index,
        ));
    };
    ($parser:expr, MatchFailed, $terminal_name:expr, $input_index:expr, $slot_id:expr, $gss_node_id:expr, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchFailed(
            $terminal_name.into(),
            $input_index,
            $slot_id,
            $gss_node_id,
            $sppf_node_id,
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
    ($parser:expr, Pop, $gss_node_id:expr, $slot_id:expr, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::Pop(
            $gss_node_id,
            $slot_id,
            $sppf_node_id,
        ));
    };
    ($parser:expr, AddToPoppedElements, $gss_node_id:expr, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::AddToPoppedElements(
            $gss_node_id,
            $sppf_node_id,
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
    ($parser:expr, ParseSuccess, $duration:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::ParseSuccess($duration));
    };
    ($parser:expr, ParseFailed, $duration:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::ParseFailed($duration));
    };
}

#[macro_export]
#[cfg(not(feature = "debug-trace"))]
macro_rules! record {
    ($self:expr, $kind:ident $(, $args:expr)*) => {};
}
