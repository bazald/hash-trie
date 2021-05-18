#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[macro_use]
mod result;
mod bit_indexed_array;
mod flag;
mod hash_trie;
mod map;
mod node;
mod set;
pub mod traits;

/// The only error you'll find is `NotFound`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HashTrieError {
    /// The value was not found.
    NotFound,
}

pub use set::HashTrieSet as HashTrieSet;
pub use map::HashTrieMap as HashTrieMap;

#[cfg(any(feature = "std", test))]
pub type DefaultHashTrieSet<V> = set::HashTrieSet<u32, V, std::collections::hash_map::DefaultHasher>;

#[cfg(any(feature = "std", test))]
pub type DefaultHashTrieMap<K, V> = map::HashTrieMap<u32, K, V, std::collections::hash_map::DefaultHasher>;

#[cfg(test)]
mod tests {
    use crate::{DefaultHashTrieMap, DefaultHashTrieSet};
    use alloc::string::String;
    use std::borrow::Cow;

    #[test]
    fn std_test() {
        let _hash_set = DefaultHashTrieSet::<i32>::new().insert(Cow::Owned(42), false);
        let _hash_map = DefaultHashTrieMap::<i32, String>::new().insert(Cow::Owned(42), Cow::Owned("Hello, world!".into()), false);
    }

}
