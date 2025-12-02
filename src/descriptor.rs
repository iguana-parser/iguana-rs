use crate::{parser::SlotId, sppf::SPPFNodeId};

pub struct Descriptor {
    pub slot_id: SlotId,
    pub result: Option<SPPFNodeId>,
    pub gss_node_id: usize,
}

impl Descriptor {
    pub fn new(slot_id: SlotId, result: Option<SPPFNodeId>, gss_node_id: usize) -> Self {
        Self {
            slot_id,
            result,
            gss_node_id,
        }
    }
}
