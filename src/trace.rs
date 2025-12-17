use crate::{parser::SlotId, sppf::SPPFNodeId};

#[cfg(feature = "debug-trace")]
#[derive(Debug, Clone)]
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
    MatchLeadingLayout,
}

#[cfg(feature = "debug-trace")]
impl TraceEvent {
    pub fn message(&self) -> String {
        match self.kind {
            TraceEventKind::MatchLeadingLayout => {
                format!(
                    "Matching leading layout at input index {}",
                    self.input_index
                )
            }
        }
    }
}

#[macro_export]
#[cfg(feature = "debug-trace")]
macro_rules! trace_event {
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
macro_rules! trace_event {
    (
        $self:expr, 
        $kind:expr, 
        ($input_index:expr, $slot_id:expr, $sppf_node_id:expr, $gss_node_id:expr)
    ) => {};
}
