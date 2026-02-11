use serde::{Deserialize, Serialize};
use specta::Type;

use crate::utils::inline_vec::InlineVec;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Type)]
pub struct EnvId(pub u32);

impl EnvId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Default)]
pub struct Env {
    bindings: InlineVec<(&'static str, i32)>,
}

impl Env {
    pub fn bind(&mut self, name: &'static str, value: i32) {
        self.bindings.push((name, value));
    }

    pub fn get(&self, name: &str) -> i32 {
        for (n, v) in self.bindings.iter() {
            if *n == name {
                return *v;
            }
        }
        panic!("binding not found: {name}")
    }
}
