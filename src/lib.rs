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

#[derive(Clone, Debug)]
pub struct Zst {}

pub use result::BitError as BitError;
pub use result::HashTrieError as HashTrieError;

pub use result::KeyRef as KeyRef;
pub use result::KeyValueRef as KeyValueRef;

pub use result::SetTransformResult as SetTransformResult;
pub use result::MapTransformResult as MapTransformResult;

pub use set::HashTrieSet as HashTrieSet;
pub use map::HashTrieMap as HashTrieMap;

#[cfg(any(feature = "std", test))]
pub type DefaultHashTrieSet<V> = set::HashTrieSet<u64, u32, V, std::collections::hash_map::DefaultHasher>;

#[cfg(any(feature = "std", test))]
pub type DefaultHashTrieMap<K, V> = map::HashTrieMap<u64, u32, K, V, std::collections::hash_map::DefaultHasher>;

#[cfg(test)]
mod tests {
    use crate::{DefaultHashTrieMap, DefaultHashTrieSet};
    use alloc::string::String;

    #[test]
    fn std_test() {
        let _hash_set = DefaultHashTrieSet::<i32>::new().insert(42, false);
        let _hash_map = DefaultHashTrieMap::<i32, String>::new().insert(42, "Hello, world!", false);
    }

}
