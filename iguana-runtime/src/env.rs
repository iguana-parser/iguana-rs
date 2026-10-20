use crate::{
    arena::{Arena, ArenaVec},
    ids::BindingId,
    utils::id_vec::Placeholder,
};

#[derive(Clone, Debug, Default)]
pub struct Env<'arena> {
    pub bindings: Bindings<'arena>,
}

impl Placeholder for Env<'_> {
    fn placeholder() -> Self {
        Env::default()
    }
}

const INLINE_CAPACITY: usize = if cfg!(target_pointer_width = "64") {
    7
} else {
    5
};

/// A list of bindings with seven inline entries on 64-bit targets and five on
/// 32-bit targets. Further entries move into an arena vector. The inline
/// entries keep names and values in separate arrays, so a one-byte name is not
/// padded to the alignment of its value. The shared `InlineVec` keeps three
/// inline entries so that collections with larger elements, such as GSS edges,
/// do not grow.
#[derive(Clone, Debug)]
pub enum Bindings<'arena> {
    Inline {
        len: u8,
        names: [BindingId; INLINE_CAPACITY],
        values: [i32; INLINE_CAPACITY],
    },
    Multiple(ArenaVec<'arena, (BindingId, i32)>),
}

impl Default for Bindings<'_> {
    fn default() -> Self {
        Self::Inline {
            len: 0,
            names: [BindingId(0); INLINE_CAPACITY],
            values: [0; INLINE_CAPACITY],
        }
    }
}

impl<'arena> Bindings<'arena> {
    pub fn push(&mut self, binding: (BindingId, i32), arena: &'arena Arena) {
        match self {
            Self::Inline { len, names, values } if usize::from(*len) < INLINE_CAPACITY => {
                let index = usize::from(*len);
                names[index] = binding.0;
                values[index] = binding.1;
                *len += 1;
            }
            Self::Inline { names, values, .. } => {
                let mut bindings = arena.vec_with_capacity(2 * INLINE_CAPACITY);
                bindings.extend(names.iter().copied().zip(values.iter().copied()));
                bindings.push(binding);
                *self = Self::Multiple(bindings);
            }
            Self::Multiple(bindings) => bindings.push(binding),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => usize::from(*len),
            Self::Multiple(bindings) => bindings.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'arena> Env<'arena> {
    /// Appends a binding, shadowing any earlier binding of the same name.
    pub fn bind(&mut self, name: BindingId, value: i32, arena: &'arena Arena) {
        self.bindings.push((name, value), arena);
    }

    /// Reads the latest binding of this name.
    pub fn get(&self, name: BindingId) -> i32 {
        let value = match &self.bindings {
            Bindings::Inline { len, names, values } => names[..usize::from(*len)]
                .iter()
                .rposition(|&n| n == name)
                .map(|index| values[index]),
            Bindings::Multiple(bindings) => bindings
                .iter()
                .rev()
                .find(|(n, _)| *n == name)
                .map(|&(_, v)| v),
        };
        value.unwrap_or_else(|| panic!("binding not found: {name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn seven_bindings_stay_inline_and_the_eighth_spills() {
        let arena = Arena::new();
        let mut env = Env::default();
        for i in 0..7 {
            env.bind(BindingId(i), i as i32, &arena);
        }
        assert!(matches!(env.bindings, Bindings::Inline { len: 7, .. }));
        let inline = env.clone();
        env.bind(BindingId(7), 7, &arena);
        assert!(matches!(env.bindings, Bindings::Multiple(_)));
        assert_eq!(inline.bindings.len(), 7);
        assert_eq!(env.bindings.len(), 8);
        for i in 0..7 {
            assert_eq!(inline.get(BindingId(i)), i as i32);
            assert_eq!(env.get(BindingId(i)), i as i32);
        }
        assert_eq!(env.get(BindingId(7)), 7);
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
