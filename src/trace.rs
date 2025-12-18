#[cfg(feature = "debug-trace")]
use crate::parser::{Parser, SlotId};
use crate::{sppf::SPPFNodeId};

#[cfg(feature = "debug-trace")]
#[derive(Debug)]
pub struct TraceEvent {
    pub input_index: u32,
    pub slot_id: SlotId,
    pub sppf_node_id: Option<SPPFNodeId>,
    pub gss_node_id: usize,
    pub kind: TraceEventKind,
}

#[cfg(feature = "debug-trace")]
#[derive(Debug, Clone)]
pub enum TraceEventKind {
    ProcessingDescriptor,
    MatchingLeadingLayout,
    MatchingTrailingLayout,
    MatchingTerminal(&'static str),
    MatchSuccess(&'static str, u32),
    MatchFailed(&'static str),
}

#[cfg(feature = "debug-trace")]
impl TraceEvent{
    pub fn message<'i>(&self, parser: &impl Parser<'i>) -> String {
        match self.kind {
            TraceEventKind::ProcessingDescriptor => {
                format!(
                    "Processing ({}, {}, {})",
                    parser.slot_name(self.slot_id),
                    parser.gss_to_string(self.gss_node_id),
                    if let Some(result) = self.sppf_node_id {
                        parser.sppf_node_to_string(parser.sppf_node(result))
                    } else {
                        "$".to_string()
                    }
                )
            }
            TraceEventKind::MatchingLeadingLayout => {
                format!(
                    "Matching leading layout at input index {}",
                    self.input_index
                )
            }
            TraceEventKind::MatchingTrailingLayout => {
                format!(
                    "Matching trailing layout at input index {}",
                    self.input_index
                )
            }
            TraceEventKind::MatchingTerminal(terminal_name) => 
                format!(
                    "Matched terminal {terminal_name} at input index {}", 
                    self.input_index, 
                ),
            TraceEventKind::MatchSuccess(terminal_name, match_end) => 
                format!(
                    "Matched terminal {terminal_name} at input index {}. Match length: {}", 
                    self.input_index, 
                    match_end - self.input_index
                ),
            TraceEventKind::MatchFailed(terminal_name)  => 
                parser.input().format_error(terminal_name, self.input_index)
        }
    }
}

#[macro_export]
#[cfg(feature = "debug-trace")]
macro_rules! record {  
    (
        $self:expr, 
        $kind:expr,
        ($input_index:expr, $slot_id:expr, $sppf_node_id:expr, $gss_node_id:expr)
    ) => {
        if let Some(trace_events) = &mut $self.trace_events {
            trace_events.push($crate::trace::TraceEvent {
                input_index: $input_index,
                slot_id: $slot_id,
                sppf_node_id: $sppf_node_id,
                gss_node_id: $gss_node_id,
                kind: $kind,
            });
        }
    };
}

#[macro_export]
#[cfg(not(feature = "debug-trace"))]
macro_rules! record {
    (
        $self:expr, 
        $kind:expr, 
        ($input_index:expr, $slot_id:expr, $sppf_node_id:expr, $gss_node_id:expr)
    ) => {};
}
