#[cfg(feature = "debug-trace")]
use std::time::Duration;

#[cfg(feature = "debug-trace")]
use crate::parser::Parser;
#[cfg(feature = "debug-trace")]
use crate::sppf::SPPFNodeId;
#[cfg(feature = "debug-trace")]
use crate::{
    ids::{NonterminalId, SlotId, TerminalId},
    sppf::Span,
};

#[cfg(feature = "debug-trace")]
#[derive(Debug)]
pub enum TraceEvent {
    ProcessingDescriptor(SlotId, u32, usize, Option<SPPFNodeId>),
    DescriptorAdded(SlotId, u32, usize, Option<SPPFNodeId>),
    MatchingLeadingLayout(u32),
    MatchingTrailingLayout(u32),
    MatchingTerminal(&'static str, u32),  // terminal_name
    MatchSuccess(&'static str, u32, u32), // terminal_name, next_input match
    MatchFailed(&'static str, u32),       // terminal_name
    MatchedLayout(Option<u32>),           // next_input match
    GSSNodeCreated(NonterminalId, u32),
    GSSNodeFound(NonterminalId, u32),
    GSSNodeNotFound(NonterminalId, u32),
    GSSNodeAdded(usize, usize, SlotId),
    TerminalNodeCreated(TerminalId, Span),
    NonterminalNodeCreated(NonterminalId, Span),
    IntermediateNodeCreated(SlotId, Span),
    TerminalNodeFound(SPPFNodeId),
    NonterminalNodeFound(SPPFNodeId),
    IntermediateNodeFound(SPPFNodeId),
    Pop(usize, SPPFNodeId),
    AddToPoppedElements(usize, SPPFNodeId),
    NodeAlreadyInPoppedElements,
    Call(Option<SPPFNodeId>, usize, SlotId),
    ParseSuccess(Duration),
    ParseFailed(Duration),
}

#[cfg(feature = "debug-trace")]
impl TraceEvent {
    pub fn message<'i>(&self, parser: &impl Parser<'i>) -> String {
        match *self {
            TraceEvent::ProcessingDescriptor(slot_id, input_index, gss_node_id, sppf_node_id) => {
                format!(
                    "Processing ({}, {}, {}, {})",
                    parser.slot_name(slot_id),
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
                    parser.slot_name(slot_id),
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
            TraceEvent::MatchingTerminal(terminal_name, input_index) => format!(
                "Matched terminal {terminal_name} at input index {}",
                input_index,
            ),
            TraceEvent::MatchSuccess(terminal_name, input_index, matched_index) => format!(
                "Matched terminal {terminal_name} at input index {}. Match length: {}",
                input_index,
                matched_index - input_index
            ),
            TraceEvent::MatchFailed(terminal_name, input_index) => {
                parser.input().format_error(terminal_name, input_index)
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
                parser.nonterminal_name(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeFound(nonterminal_id, input_index) => format!(
                "GSS node ({},{}) found",
                parser.nonterminal_name(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeNotFound(nonterminal_id, input_index) => format!(
                "GSS node ({},{}) not found",
                parser.nonterminal_name(nonterminal_id),
                input_index
            ),
            TraceEvent::GSSNodeAdded(origin_gss_node_id, dest_gss_node_id, return_slot) => format!(
                "GSS edge added from {} to {} with return label {}",
                parser.gss_to_string(origin_gss_node_id),
                parser.gss_to_string(dest_gss_node_id),
                parser.slot_name(return_slot)
            ),
            TraceEvent::TerminalNodeCreated(terminal_id, span) => format!(
                "({}, {}, {})",
                parser.terminal_name(terminal_id),
                span.left_extent,
                span.right_extent
            ),
            TraceEvent::NonterminalNodeCreated(nonterminal_id, span) => format!(
                "({}, {}, {})",
                parser.nonterminal_name(nonterminal_id),
                span.left_extent,
                span.right_extent
            ),
            TraceEvent::IntermediateNodeCreated(slot_id, span) => format!(
                "({}, {}, {})",
                parser.slot_name(slot_id),
                span.left_extent,
                span.right_extent
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
            TraceEvent::Pop(gss_node_id, sppf_node_id) => format!(
                "Pop: {} with result {}",
                parser.gss_to_string(gss_node_id),
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
                parser.slot_name(slot_id)
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
        $parser.add_trace_event($crate::trace::TraceEvent::MatchingLeadingLayout($input_index));
    };
    ($parser:expr, MatchingTrailingLayout, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchingTrailingLayout($input_index));
    };
    ($parser:expr, MatchingTerminal, $terminal_name:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchingTerminal($terminal_name, $input_index));
    };
    ($parser:expr, MatchSuccess, $terminal_name:expr, $input_index:expr, $next_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchSuccess(
            $terminal_name,
            $input_index,
            $next_index,
        ));
    };
    ($parser:expr, MatchFailed, $terminal_name:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchFailed($terminal_name, $input_index));
    };
    ($parser:expr, MatchedLayout, $match_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::MatchedLayout($match_index));
    };
    ($parser:expr, GSSNodeCreated, $nonterminal_id:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeCreated($nonterminal_id, $input_index));
    };
    ($parser:expr, GSSNodeFound, $nonterminal_id:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeFound($nonterminal_id, $input_index));
    };
    ($parser:expr, GSSNodeNotFound, $nonterminal_id:expr, $input_index:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeNotFound($nonterminal_id, $input_index));
    };
    ($parser:expr, GSSNodeAdded, $origin_gss_node_id:expr, $dest_gss_node_id:expr, $return_slot:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::GSSNodeAdded(
            $origin_gss_node_id,
            $dest_gss_node_id,
            $return_slot,
        ));
    };
    ($parser:expr, TerminalNodeCreated, $terminal_id:expr, $span:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::TerminalNodeCreated($terminal_id, $span));
    };
    ($parser:expr, NonterminalNodeCreated, $nonterminal_id:expr, $span:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::NonterminalNodeCreated($nonterminal_id, $span));
    };
    ($parser:expr, IntermediateNodeCreated, $slot_id:expr, $span:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::IntermediateNodeCreated($slot_id, $span));
    };
    ($parser:expr, TerminalNodeFound, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::TerminalNodeFound($sppf_node_id));
    };
    ($parser:expr, NonterminalNodeFound, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::NonterminalNodeFound($sppf_node_id));
    };
    ($parser:expr, IntermediateNodeFound, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::IntermediateNodeFound($sppf_node_id));
    };
    ($parser:expr, Pop, $gss_node_id:expr, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::Pop($gss_node_id, $sppf_node_id));
    };
    ($parser:expr, AddToPoppedElements, $gss_node_id:expr, $sppf_node_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::AddToPoppedElements($gss_node_id, $sppf_node_id));
    };
    ($parser:expr, NodeAlreadyInPoppedElements) => {
        $parser.add_trace_event($crate::trace::TraceEvent::NodeAlreadyInPoppedElements);
    };
    ($parser:expr, Call, $sppf_node_id:expr, $gss_node_id:expr, $slot_id:expr) => {
        $parser.add_trace_event($crate::trace::TraceEvent::Call($sppf_node_id, $gss_node_id, $slot_id));
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
