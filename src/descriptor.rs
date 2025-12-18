use crate::{parser::SlotId, sppf::SPPFNodeId};

pub struct Descriptor {
    pub input_index: u32,
    pub slot_id: SlotId,
    pub sppf_node_id: Option<SPPFNodeId>,
    pub gss_node_id: usize,
}
