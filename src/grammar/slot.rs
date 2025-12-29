use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Slot {
    pub name: String,
}

impl Slot {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
