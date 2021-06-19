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
///     .map_err(|key_value| *key_value.0 == hello && *key_value.1 == world).unwrap_err());
/// // ... unless you enable replacement.
/// assert!(hash_map.insert(Str::new(hello), "?", true).is_ok());
/// 
/// assert!(hash_map.find(&Str::new(hello)).map(|key_value| *key_value.0 == hello && *key_value.1 == world).unwrap());
/// 
/// match hash_map.remove(&Str::new(hello)) {
///     Ok((mutated, key, value)) => {
///         // Removing a key returns references to the key and
///         // value in the set in addition to the mutated map.
///         println!("Value stored in hash_map: ({}, {})", key, value);
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
    pub fn find<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<(&'a K, &'a V), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.find(key)
    }

    /// Search the HashTrieMap for the spot to insert the key and return both a mutated map and, if applicable, references to the replaced values.
    /// If found and replacement is disabled, references to the existing values are returned.
    #[allow(clippy::type_complexity)]
    pub fn insert<'a, L: Key + HashLike<K> + Into<K>, W: Into<V>>(&'a self, key: L, value: W, replace: bool) -> Result<(Self, *const K, *const V, Option<(&'a K, &'a V)>), (&'a K, &'a V)>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>
    {
        self.set.insert(key, value, replace).map(|(set, key, value, prev)| (Self {set}, key, value, prev))
    }

    /// Search the HashTrieMap for the given key to remove and return a mutated map, or `HashTrieError::NotFound` if not found.
    pub fn remove<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<(Self, &'a K, &'a V), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.remove(key).map(|(set, key, value)| (Self {set}, key, value))
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
        Op: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone
    {
        let (set, reduced) = self.set.transform(reduce_op, |k, v| op(k, v));
        (Self{set}, reduced)
    }

    /// Run a transmute operation on each entry in the map. Returns the transmuted map and a reduction of the secondary returns of the transmute operations.
    pub unsafe fn transmute<S: Key + HashLike<K>, X: Value, ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> (HashTrieMap<H, F, S, X, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        K: HashLike<S>,
        K: PartialEq<S>,
        M: HasherBv<H, S>,
    {
        let (set, reduced) = self.set.transmute(reduce_op, |k, v| op(k, v));
        (HashTrieMap{set}, reduced)
    }

    /// Run a transmute operation on each entry or pair of entries in the sets. Returns the transmuted set and a reduction of the secondary returns of the transmute operations.
    pub unsafe fn joint_transmute<L: Key + HashLike<K>, W: Value, S: Key + HashLike<K>, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieMap<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp) -> (HashTrieMap<H, F, S, X, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        K: HashLike<L>,
        K: PartialEq<L>,
        K: HashLike<S>,
        K: PartialEq<S>,
        L: HashLike<K>,
        L: PartialEq<K>,
        L: HashLike<S>,
        L: PartialEq<S>,
        M: HasherBv<H, L>,
        M: HasherBv<H, S>,
    {
        let (set, reduced) = self.set.joint_transmute(&right.set, reduce_op, both_op, left_op, right_op);
        (HashTrieMap{set}, reduced)
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
    fn map_transmute() {
        let mut map = DefaultHashTrieMap::<i32, i32>::new();
        let mut squared = DefaultHashTrieMap::<i32, i32>::new();

        for i in 1..101 {
            map = map.insert(i, i, false).unwrap().0;
            squared = squared.insert(i, i * i, false).unwrap().0;
        }

        let removed = map.transform(|_,_| (), |_,_| MapTransformResult::Removed(()));
        let tsquared = unsafe { map.transmute(|_,_| (), |k,v| MapTransmuteResult::Transmuted(*k, v * v, ())) };

        assert_eq!(removed.0.size(), 0);

        for i in 1..101 {
            map.find(&i).unwrap();
            assert_eq!(i * i, *squared.find(&i).unwrap().1);
            assert_eq!(i * i, *tsquared.0.find(&i).unwrap().1);
        }
    }
    
}
