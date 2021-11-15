#![cfg_attr(not(test), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

extern crate futures;

#[macro_use]
pub mod results;
mod bit_indexed_array;
mod enums;
mod flag;
mod hash_trie;
mod map;
mod node;
mod set;
pub mod traits;
pub mod transformations;

pub use enums::ParallelismStrategy;

pub use set::HashTrieSet as HashTrieSet;
pub use map::HashTrieMap as HashTrieMap;

/// A HashTrieSet using 64-bit hashes, 32-bit flags, and FnvHasher
pub type DefaultHashTrieSet<V> = set::HashTrieSet<u64, u32, V, fnv::FnvHasher>;

/// A HashTrieMap using 64-bit hashes, 32-bit flags, and FnvHasher
pub type DefaultHashTrieMap<K, V> = map::HashTrieMap<u64, u32, K, V, fnv::FnvHasher>;

#[cfg(test)]
mod tests {
    use crate::{DefaultHashTrieMap, DefaultHashTrieSet};
    use alloc::string::String;

    #[test]
    fn default_test() {
        let _hash_set = DefaultHashTrieSet::<i32>::new().insert(42, false);
        let _hash_map = DefaultHashTrieMap::<i32, String>::new().insert(42, "Hello, world!", false);
    }

}
