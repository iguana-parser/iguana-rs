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
    pub sppf_node_id: Option<SPPFNodeId>,
    pub gss_node_id: GssNodeId,
    pub env: Option<EnvId>,
}
