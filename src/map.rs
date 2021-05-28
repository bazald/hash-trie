use crate::{traits::*, HashTrieError, hash_trie::HashTrie, MapTransformResult};
use alloc::{borrow::Cow, fmt::Debug};
use core::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
struct MapEntry<K, V> {
    key: K,
    value: V,
}

impl <K, V> MapEntry<K, V> {
    #[must_use]
    fn new(key: K, value: V) -> Self {
        Self {key, value}
    }

    #[must_use]
    fn as_ref(&self) -> (&K, &V) {
        (&self.key, &self.value)
    }
}

impl <'a, K: Clone + Debug, V: Clone + Debug> From<CowMapEntry<'a, K, V>> for MapEntry<K, V> {
    fn from(cow: CowMapEntry<'a, K, V>) -> Self {
        MapEntry::new(cow.key.into_owned(), cow.value.into_owned())
    }
}

impl <B, K, V, H: HasherBv<B, K>> HasherBv<B, MapEntry<K, V>> for H {
    fn hash(&self, entry: &MapEntry<K, V>) -> B {
        H::default().hash(&entry.key)
    }
}

impl <K: Hash, V> Hash for MapEntry<K, V> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.key.hash(hasher)
    }
}

impl <K: Eq, V> Eq for MapEntry<K, V> {}

impl <K: PartialEq, V> PartialEq for MapEntry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl <K: PartialEq, V> PartialEq<K> for MapEntry<K, V> {
    fn eq(&self, other: &K) -> bool {
        self.key == *other
    }
}

impl <K, V> HashLike<K> for MapEntry<K, V> {}
impl <K, V> HashLike<MapEntry<K, V>> for K {}

#[derive(Clone, Debug)]
struct CowMapEntry<'a, K: Clone + Debug + 'static, V: Clone + Debug + 'static> {
    key: Cow<'a, K>,
    value: Cow<'a, V>,
}

impl <'a, K: Clone + Debug + 'static, V: Clone + Debug + 'static> CowMapEntry<'a, K, V> {
    #[must_use]
    fn new(key: Cow<'a, K>, value: Cow<'a, V>) -> Self {
        CowMapEntry {key, value}
    }
}

impl <'a, K: Clone + Debug, V: Clone + Debug> AsRef<K> for CowMapEntry<'a, K, V> {
    fn as_ref(&self) -> &K {
        self.key.as_ref()
    }
}

/// `HashTrieMap` implements a hash map using a hash array mapped trie (HAMT).
/// 
/// # Example Usage
/// 
/// ```
/// use hash_trie::HashTrieMap;
/// use std::{borrow::Cow, collections::hash_map::DefaultHasher};
///
/// let mut hash_map: HashTrieMap<u64, u32, String, String, DefaultHasher> = HashTrieMap::new();
/// let hello: String = "Hello,".into();
/// let world: String = "world!,".into();
///
/// hash_map = hash_map.insert(Cow::Borrowed(&hello), Cow::Borrowed(&world), false).unwrap().0;
/// 
/// // Inserting an already-inserted key returns references to the key and value in the map...
/// assert_eq!(hash_map.insert(Cow::Borrowed(&hello), Cow::Owned("?".into()), false)
///     .unwrap_err(), (&hello, &world));
/// // ... unless you enable replacement.
/// assert!(hash_map.insert(Cow::Borrowed(&hello), Cow::Owned("?".into()), true).is_ok());
///
/// assert_eq!(hash_map.find(&hello).unwrap(), (&hello, &world));
///
/// match hash_map.remove(&hello) {
///     Ok((mutated, key_reference, value_reference)) => {
///         // Removing a key returns references to the key and
///         // value in the set in addition to the mutated map.
///         println!("Value stored in hash_map: {}", value_reference);
///         hash_map = mutated;
///     },
///     Err(_) => panic!(),
/// }
/// ```
#[derive(Clone, Debug)]
#[must_use]
pub struct HashTrieMap <H: Hashword, F: Flagword<H>, K: Value, V: Clone + Debug + Eq + PartialEq + Send + Sync + 'static, M: HasherBv<H, K> + 'static> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    set: HashTrie<H, F, MapEntry<K, V>, M>,
}

impl <H: Hashword, F: Flagword<H>, K: Value, V: Clone + Debug + Eq + PartialEq + Send + Sync + 'static, M: HasherBv<H, K> + 'static> HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    /// Get a new, empty HashTrieMap.
    pub fn new() -> Self {
        Self {
            set: HashTrie::<H, F, MapEntry<K, V>, M>::new()
        }
    }

    /// Get the total number of entries in the map.
    pub fn size(&self) -> usize {
        self.set.size()
    }

    /// Search the HashTrieMap for the given key and return references if found, or `HashTrieError::NotFound` if not found.
    pub fn find(&self, key: &K) -> Result<(&K, &V), HashTrieError> {
        self.set.find(key).map(|entry| entry.as_ref())
    }

    /// Search the HashTrieMap for the spot to insert the key and return both a mutated map and, if applicable, references to the replaced values.
    /// If found and replacement is disabled, references to the existing values are returned.
    #[allow(clippy::type_complexity)]
    pub fn insert<'a>(&'a self, key: Cow<'a, K>, value: Cow<'a, V>, replace: bool) -> Result<(Self, Option<(&'a K, &'a V)>), (&'a K, &'a V)> {
        self.set.insert(CowMapEntry::new(key, value), replace).map(|(set, reference)| (Self {set}, reference.map(|entry| entry.as_ref()))).map_err(|entry| entry.as_ref())
    }

    /// Search the HashTrieMap for the given key to remove and return a mutated map, or `HashTrieError::NotFound` if not found.
    pub fn remove(&self, key: &K) -> Result<(Self, &K, &V), HashTrieError> {
        self.set.remove(key).map(|(set, entry)| (Self {set}, entry)).map(|(map, entry)| (map, &entry.key, &entry.value))
    }

    /// Run an operation on each entry in the map.
    pub fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&K, &V) {
        self.set.visit(|e| op(&e.key, &e.value));
    }

    /// Run a transform operation on each entry in the map. Returns the transformed map and a reduction of the secondary returns of the transform operations.
    pub fn transform<ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(ReduceT, ReduceT) -> ReduceT + Clone,
        Op: Fn(&K, &V) -> (MapTransformResult<V>, ReduceT) + Clone
    {
        let (set, reduced) = self.set.transform(reduce_op, |e| {
            let (result, reduced) = op(&e.key, &e.value);
            (match result {
                MapTransformResult::Unchanged => MapTransformResult::Unchanged,
                MapTransformResult::Changed(value) => MapTransformResult::Changed(MapEntry::new(e.key.clone(), value)),
                MapTransformResult::Removed => MapTransformResult::Removed,
            }, reduced)
        });
        (Self{set}, reduced)
    }
}

impl <H: Hashword, F: Flagword<H>, K: Value, V: Clone + Debug + Eq + PartialEq + Send + Sync + 'static, M: HasherBv<H, K> + 'static> Default for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn default() -> Self {
        Self::new()
    }
}

impl <H: Hashword, F: Flagword<H>, K: Value, V: Clone + Debug + Eq + PartialEq + Send + Sync + 'static, M: HasherBv<H, K> + 'static> Eq for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {}

impl <H: Hashword, F: Flagword<H>, K: Value, V: Clone + Debug + Eq + PartialEq + Send + Sync + 'static, M: HasherBv<H, K> + 'static> PartialEq for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::borrow::Cow;
    
    #[test]
    fn map_transform() {
        let mut map = DefaultHashTrieMap::<i32, i32>::new();
        let mut squared = DefaultHashTrieMap::<i32, i32>::new();

        for i in 1..101 {
            map = map.insert(Cow::Owned(i), Cow::Owned(i), false).unwrap().0;
            squared = squared.insert(Cow::Owned(i), Cow::Owned(i * i), false).unwrap().0;
        }

        let same = map.transform(|_,_| (), |_,_| (MapTransformResult::Unchanged, ()));
        let removed = map.transform(|_,_| (), |_,_| (MapTransformResult::Removed, ()));
        let tsquared = map.transform(|_,_| (), |_,v| (MapTransformResult::Changed(v * v), ()));

        assert_eq!(map, same.0);
        assert_eq!(removed.0.size(), 0);

        for i in 1..101 {
            map.find(&i).unwrap();
            assert_eq!(i * i, *squared.find(&i).unwrap().1);
            assert_eq!(i * i, *tsquared.0.find(&i).unwrap().1);
        }
    }
}
