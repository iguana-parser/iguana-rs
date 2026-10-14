use serde::{Deserialize, Serialize};
use specta::Type;

use crate::ids::{EnvId, GssNodeId, SPPFNodeId, SlotId};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Descriptor {
    pub input_index: u32,
    pub slot_id: SlotId,
    pub sppf_node_id: Option<SPPFNodeId>,
    pub gss_node_id: GssNodeId,
    pub env_id: Option<EnvId>,
}

impl Descriptor {
    #[inline]
    pub fn new(
        input_index: u32,
        slot_id: SlotId,
        sppf_node_id: Option<SPPFNodeId>,
        gss_node_id: GssNodeId,
        env_id: Option<EnvId>,
    ) -> Self {
        Self {
            input_index,
            slot_id,
            sppf_node_id,
            gss_node_id,
            env_id,
        }
    }
}
