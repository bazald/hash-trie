use crate::{traits::*, hash_trie::HashTrie, *};
use alloc::fmt::Debug;

/// `HashTrieMap` implements a hash map using a hash array mapped trie (HAMT).
/// 
/// # Example Usage
/// 
/// ```
/// use hash_trie::{HashTrieMap, traits::HashLike};
/// use std::{collections::hash_map::DefaultHasher, string::String};
/// 
/// #[derive(Clone,Debug,Eq,Hash,PartialEq)]
/// struct Str<'a> {
///     s: &'a str
/// }
/// 
/// impl <'a> Str<'a> {
///     fn new(s: &'a str) -> Self {
///         Self {s}
///     }
/// }
/// 
/// impl <'a> Default for Str<'a> {
///     fn default() -> Self {
///         Self {s: ""}
///     }
/// }
/// impl <'a> From<Str<'a>> for String {
///     fn from(s: Str<'a>) -> String {
///         s.s.into()
///     }
/// }
/// impl <'a> PartialEq<Str<'a>> for String {
///     fn eq(&self, other: &Str<'a>) -> bool {
///         *self == other.s
///     }
/// }
/// impl <'a> HashLike<String> for Str<'a> {}
/// impl <'a> HashLike<Str<'a>> for String {}
/// unsafe impl <'a> Send for Str<'a> {}
/// unsafe impl <'a> Sync for Str<'a> {}
/// 
/// let mut hash_map: HashTrieMap<u64, u32, String, String, DefaultHasher> = HashTrieMap::new();
/// let hello = "Hello,";
/// let world = "world!,";
/// 
/// hash_map = hash_map.insert(Str::new(hello), world, false).unwrap().0;
/// 
/// // Inserting an already-inserted key returns references to the key and value in the map...
/// assert!(hash_map.insert(Str::new(hello), "?", false)
///     .map_err(|key_value| *key_value.key() == hello && *key_value.value() == world).unwrap_err());
/// // ... unless you enable replacement.
/// assert!(hash_map.insert(Str::new(hello), "?", true).is_ok());
/// 
/// assert!(hash_map.find(&Str::new(hello)).map(|key_value| *key_value.key() == hello && *key_value.value() == world).unwrap());
/// 
/// match hash_map.remove(&Str::new(hello)) {
///     Ok((mutated, key_value)) => {
///         // Removing a key returns references to the key and
///         // value in the set in addition to the mutated map.
///         println!("Value stored in hash_map: ({}, {})", key_value.key(), key_value.value());
///         hash_map = mutated;
///     },
///     Err(_) => panic!(),
/// }
/// ```
#[derive(Clone, Debug)]
#[must_use]
pub struct HashTrieMap <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    set: HashTrie<H, F, K, V, M>,
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    /// Get a new, empty HashTrieMap.
    pub fn new() -> Self {
        Self {
            set: HashTrie::<H, F, K, V, M>::new()
        }
    }

    /// Get the total number of entries in the map.
    pub fn size(&self) -> usize {
        self.set.size()
    }

    /// Search the HashTrieMap for the given key and return references if found, or `HashTrieError::NotFound` if not found.
    pub fn find<L: Key + HashLike<K>>(&self, key: &L) -> Result<KeyValueRef<K, V>, HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.find(key)
    }

    /// Search the HashTrieMap for the spot to insert the key and return both a mutated map and, if applicable, references to the replaced values.
    /// If found and replacement is disabled, references to the existing values are returned.
    #[allow(clippy::type_complexity)]
    pub fn insert<L: Key + HashLike<K> + Into<K>, W: Into<V>>(&self, key: L, value: W, replace: bool) -> Result<(Self, KeyValueRef<K, V>), KeyValueRef<K, V>>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>
    {
        self.set.insert(key, value, replace).map(|(set, reference)| (Self {set}, reference))
    }

    /// Search the HashTrieMap for the given key to remove and return a mutated map, or `HashTrieError::NotFound` if not found.
    pub fn remove<L: Key + HashLike<K>>(&self, key: &L) -> Result<(Self, KeyValueRef<K, V>), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.remove(key).map(|(set, reference)| (Self {set}, reference))
    }

    /// Run an operation on each entry in the map.
    pub fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&K, &V) {
        self.set.visit(|k,v| op(k, v));
    }

    /// Run a transform operation on each entry in the map. Returns the transformed map and a reduction of the secondary returns of the transform operations.
    pub fn transform<ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K, &V) -> MapTransformResult<K, V, ReduceT> + Clone
    {
        let (set, reduced) = self.set.transform(reduce_op, |k, v| op(k, v));
        (Self{set}, reduced)
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Default for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn default() -> Self {
        Self::new()
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Eq for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> PartialEq for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    
    #[test]
    fn map_transform() {
        let mut map = DefaultHashTrieMap::<i32, i32>::new();
        let mut squared = DefaultHashTrieMap::<i32, i32>::new();

        for i in 1..101 {
            map = map.insert(i, i, false).unwrap().0;
            squared = squared.insert(i, i * i, false).unwrap().0;
        }

        let removed = map.transform(|_,_| (), |_,_| MapTransformResult::Removed(()));
        let tsquared = map.transform(|_,_| (), |k,v| MapTransformResult::Transformed(*k, v * v, ())); // TODO: add checks on *k

        assert_eq!(removed.0.size(), 0);

        for i in 1..101 {
            map.find(&i).unwrap();
            assert_eq!(i * i, *squared.find(&i).unwrap().value());
            assert_eq!(i * i, *tsquared.0.find(&i).unwrap().value());
        }
    }
    
}
