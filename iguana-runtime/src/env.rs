use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{ids::BindingId, utils::inline_vec::InlineVec};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct EnvId(pub u32);

impl EnvId {
    /// Sentinel for an absent env id. Real ids must be < u32::MAX.
    pub const NONE: Self = Self(u32::MAX);

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Default)]
pub struct Env {
    pub bindings: InlineVec<(BindingId, i32)>,
}

impl Env {
    pub fn bind(&mut self, name: BindingId, value: i32) {
        self.bindings.push((name, value));
    }

    pub fn get(&self, name: BindingId) -> i32 {
        for (n, v) in self.bindings.iter() {
            if *n == name {
                return *v;
            }
        }
        panic!("binding not found: {name}")
    }
}
