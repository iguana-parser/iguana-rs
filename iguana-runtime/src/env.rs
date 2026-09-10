use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    arena::{Arena, ArenaVec},
    ids::BindingId,
};

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
pub struct Env<'arena> {
    pub bindings: Bindings<'arena>,
}

const INLINE_CAPACITY: usize = if cfg!(target_pointer_width = "64") {
    4
} else {
    3
};

/// A list of bindings with four inline entries on 64-bit targets and three on
/// 32-bit targets. Further entries move into an arena vector. The shared
/// `InlineVec` keeps three inline entries so that collections with larger
/// elements, such as GSS edges, do not grow.
#[derive(Clone, Debug)]
pub enum Bindings<'arena> {
    Inline {
        len: u8,
        values: [(BindingId, i32); INLINE_CAPACITY],
    },
    Multiple(ArenaVec<'arena, (BindingId, i32)>),
}

impl Default for Bindings<'_> {
    fn default() -> Self {
        Self::Inline {
            len: 0,
            values: [(BindingId(0), 0); INLINE_CAPACITY],
        }
    }
}

impl<'arena> Bindings<'arena> {
    pub fn push(&mut self, binding: (BindingId, i32), arena: &'arena Arena) {
        match self {
            Self::Inline { len, values } if usize::from(*len) < values.len() => {
                values[usize::from(*len)] = binding;
                *len += 1;
            }
            Self::Inline { values, .. } => {
                let mut bindings = arena.vec_with_capacity(8);
                bindings.extend_from_slice(values);
                bindings.push(binding);
                *self = Self::Multiple(bindings);
            }
            Self::Multiple(bindings) => bindings.push(binding),
        }
    }

    pub fn as_slice(&self) -> &[(BindingId, i32)] {
        match self {
            Self::Inline { len, values } => &values[..usize::from(*len)],
            Self::Multiple(bindings) => bindings,
        }
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (BindingId, i32)> {
        self.as_slice().iter()
    }
}

impl<'arena> Env<'arena> {
    /// Appends a binding, shadowing any earlier binding of the same name.
    pub fn bind(&mut self, name: BindingId, value: i32, arena: &'arena Arena) {
        self.bindings.push((name, value), arena);
    }

    /// Reads the latest binding of this name.
    pub fn get(&self, name: BindingId) -> i32 {
        for (n, v) in self.bindings.iter().rev() {
            if *n == name {
                return *v;
            }
        }
        panic!("binding not found: {name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn four_bindings_stay_inline_and_the_fifth_spills() {
        let arena = Arena::new();
        let mut env = Env::default();
        for i in 0..4 {
            env.bind(BindingId(i), i as i32, &arena);
        }
        assert!(matches!(env.bindings, Bindings::Inline { len: 4, .. }));
        let inline = env.clone();
        env.bind(BindingId(4), 4, &arena);
        assert!(matches!(env.bindings, Bindings::Multiple(_)));
        assert_eq!(inline.bindings.len(), 4);
        assert_eq!(env.bindings.len(), 5);
        for i in 0..4 {
            assert_eq!(inline.get(BindingId(i)), i as i32);
            assert_eq!(env.get(BindingId(i)), i as i32);
        }
        assert_eq!(env.get(BindingId(4)), 4);
    }

    #[test]
    fn inline_bindings_do_not_enlarge_the_environment() {
        const {
            assert!(
                size_of::<Env>()
                    == size_of::<crate::utils::inline_vec::InlineVec<(BindingId, i32)>>()
            );
        }
    }

    #[test]
    fn later_bindings_shadow_earlier_ones_in_inline_and_spilled_environments() {
        let arena = Arena::new();
        let mut env = Env::default();
        let x = BindingId(0);
        let y = BindingId(1);
        env.bind(y, 42, &arena);
        for value in 0..8 {
            let previous = env.clone();
            env.bind(x, value, &arena);
            assert_eq!(env.get(x), value);
            assert_eq!(env.get(y), 42);
            if value > 0 {
                assert_eq!(previous.get(x), value - 1);
            }
            assert_eq!(previous.get(y), 42);
        }
    }
}
