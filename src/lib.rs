#![cfg_attr(not(test), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[macro_use]
mod result;
mod bit_indexed_array;
mod flag;
pub mod functions;
mod hash_trie;
mod map;
mod node;
mod set;
pub mod traits;

pub use result::BitError as BitError;
pub use result::HashTrieError as HashTrieError;

pub use functions::SetJointTransform as SetJointTransform;
pub use functions::SetTransform as SetTransform;
pub use functions::SetTransmute as SetTransmute;

pub use functions::MapJointTransform as MapJointTransform;
pub use functions::MapTransform as MapTransform;
pub use functions::MapTransmute as MapTransmute;

pub use result::SetTransformResult as SetTransformResult;
pub use result::SetJointTransformResult as SetJointTransformResult;
pub use result::SetTransmuteResult as SetTransmuteResult;

pub use result::MapTransformResult as MapTransformResult;
pub use result::MapJointTransformResult as MapJointTransformResult;
pub use result::MapTransmuteResult as MapTransmuteResult;

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
