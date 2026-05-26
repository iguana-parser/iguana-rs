use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    env::EnvId,
    ids::{GssNodeId, SlotId},
    sppf::SPPFNodeId,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Descriptor {
    pub input_index: u32,
    pub slot_id: SlotId,
    sppf_node_id: SPPFNodeId,
    pub gss_node_id: GssNodeId,
    env_id: EnvId,
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
            sppf_node_id: sppf_node_id.unwrap_or(SPPFNodeId::NONE),
            gss_node_id,
            env_id: env_id.unwrap_or(EnvId::NONE),
        }
    }

    #[inline]
    pub fn sppf_node_id(&self) -> Option<SPPFNodeId> {
        if self.sppf_node_id == SPPFNodeId::NONE {
            None
        } else {
            Some(self.sppf_node_id)
        }
    }

    #[inline]
    pub fn env_id(&self) -> Option<EnvId> {
        if self.env_id == EnvId::NONE {
            None
        } else {
            Some(self.env_id)
        }
    }
}
