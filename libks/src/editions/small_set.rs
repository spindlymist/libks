use std::{
    collections::HashSet,
    hash::Hash,
};

pub struct SmallSet<T>(SmallSetInner<T>);

const MAX_VEC_LEN: usize = 20;

enum SmallSetInner<T> {
    Vec(Vec<T>),
    Hash(HashSet<T>),
}

impl<T> SmallSet<T>
where
    T: Eq + Hash + Clone
{
    pub fn new(values: &[T]) -> Self {
        let inner =
            if values.len() > MAX_VEC_LEN {
                let set = HashSet::from_iter(values.iter().cloned());
                SmallSetInner::Hash(set)
            }
            else {
                SmallSetInner::Vec(values.to_vec())
            };

        Self(inner)
    }

    pub fn has(&self, value: &T) -> bool {
        match &self.0 {
            SmallSetInner::Vec(arr) => arr.contains(value),
            SmallSetInner::Hash(set) => set.contains(value),
        }
    }
}

macro_rules! static_set_lowercase {
    ( $($entry:literal),* $(,)? ) => {
        {
            use std::sync::OnceLock;
            use const_str::convert_ascii_case;
            use $crate::editions::small_set::SmallSet;

            static STATIC_SET: OnceLock<SmallSet<&'static str>> = OnceLock::new();
            STATIC_SET.get_or_init(|| {
                let entries = [$(
                    convert_ascii_case!(lower, $entry)
                ),*];
                SmallSet::new(&entries)
            })
        }
    };
}
pub(crate) use static_set_lowercase;

macro_rules! static_set_lowercase_from_file {
    ( $path:literal ) => {
        {
            use std::sync::OnceLock;
            use const_str::{convert_ascii_case, split};
            use $crate::editions::small_set::SmallSet;

            static STATIC_SET: OnceLock<SmallSet<&'static str>> = OnceLock::new();
            STATIC_SET.get_or_init(|| {
                let entries = split!(
                    convert_ascii_case!(
                        lower,
                        include_str!($path)
                    ),
                    "\n"
                );
                SmallSet::new(&entries)
            })
        }
    };
}
pub(crate) use static_set_lowercase_from_file;
