use crate::{flag::*, traits::*, node::*, result::*, HashTrieError};

use alloc::{borrow::Cow, fmt::Debug};

/// `HashTrieSet` implements a hash set using a hash array mapped trie (HAMT).
/// 
/// # Example Usage
/// 
/// ```
/// use hash_trie::HashTrieSet;
/// use std::{borrow::Cow, collections::hash_map::DefaultHasher};
///
/// let mut hash_set: HashTrieSet<u64, String, DefaultHasher> = HashTrieSet::new();
/// let hello_world: String = "Hello, world!".into();
///
/// hash_set = hash_set.insert(Cow::Borrowed(&hello_world)).unwrap();
/// 
/// // Inserting an already-inserted value returns a reference to the value in the set.
/// assert_eq!(*hash_set.insert(Cow::Borrowed(&hello_world)).unwrap_err(), hello_world);
///
/// assert_eq!(*hash_set.find(&hello_world).unwrap(), hello_world);
///
/// match hash_set.remove(&hello_world) {
///     Ok((mutated, reference)) => {
///         // Removing a value returns a reference to the value
///         // in the set in addition to the mutated set.
///         println!("Value stored in hash_set: {}", reference);
///         hash_set = mutated;
///     },
///     Err(_) => panic!(),
/// }
/// ```
#[derive(Clone, Debug)]
pub struct HashTrieSet <B: Bits, V: Value, H: HasherBv<B, V>> {
    root: MNode<B, V, H>,
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> HashTrieSet<B, V, H> {
    /// Get a new, empty HashTrieSet.
    pub fn new() -> Self {
        Self {
            root: MNode::<B, V, H>::default()
        }
    }

    fn singleton(mnode: MNode<B, V, H>) -> Self {
        Self {
            root: mnode
        }
    }

    /// Search the HashTrieSet for the given value and return a reference if found, or `HashTrieError::NotFound` if not found.
    pub fn find(&self, value: &V) -> Result<&V, HashTrieError> {
        match self.root.find(value, Some(Flag::new(H::default().hash(value)))) {
            FindResult::NotFound => Err(HashTrieError::NotFound),
            FindResult::Found(found) => Ok(found)
        }
    }

    /// Search the HashTrieSet for the spot to insert the value and return a mutated set, or a reference to the existing value if found.
    pub fn insert(&self, value: Cow<V>) -> Result<Self, &V> {
        let flag = Flag::from(H::default().hash(value.as_ref()));
        match self.root.insert(value, Some(flag)) {
            InsertResult::Found(found) => Err(found),
            InsertResult::InsertedC(cnode) => Ok(Self::singleton(MNode::C(cnode))),
            InsertResult::InsertedL(lnode) => Ok(Self::singleton(MNode::L(lnode))),
        }
    }

    /// Search the HashTrieSet for the given value to remove and return a mutated set, or `HashTrieError::NotFound` if not found.
    pub fn remove(&self, value: &V) -> Result<(Self, &V), HashTrieError> {
        match self.root.remove(value, Some(Flag::from(H::default().hash(value)))) {
            RemoveResult::NotFound => Err(HashTrieError::NotFound),
            RemoveResult::RemovedC(cnode, removed) => Ok((Self::singleton(MNode::C(cnode)), removed)),
            RemoveResult::RemovedL(lnode, removed) => Ok((Self::singleton(MNode::L(lnode)), removed)),
            RemoveResult::RemovedS(snode, removed) => Ok((Self::singleton(MNode::S(snode)), removed)),
            RemoveResult::RemovedZ(removed) => Ok((Self::default(), removed))
        }
    }
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> Default for HashTrieSet<B, V, H> {
    fn default() -> Self {
        Self {
            root: MNode::<B, V, H>::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HashTrieSet;
    use std::{borrow::Cow, collections::{hash_map::DefaultHasher, hash_set::HashSet}, vec::Vec};
    use rand::{Rng, seq::SliceRandom};

    #[test]
    fn hash_trie_test_insert_find_remove_10000() {
        let mut rng = rand::thread_rng();
        let mut hash_set = HashSet::new();
        let mut hash_trie = HashTrieSet::<u64, i32, DefaultHasher>::new();

        for _ in 0..10000 {
            let v = rng.gen_range(0..50000);
            let inserted = hash_set.insert(v);
            match hash_trie.insert(Cow::Owned(v)) {
                Ok(ht) => {
                    assert!(inserted);
                    hash_trie = ht;
                },
                Err(reference) => assert_eq!(*reference, v),
            };
        }

        for v in 0..50000 {
            let found = hash_set.contains(&v);
            match hash_trie.find(&v) {
                Ok(found_v) => {
                    assert_eq!(v, *found_v);
                    assert!(found);
                },
                Err(_) => assert!(!found),
            }
        }

        let mut values: Vec<i32> = (1..50000).collect::<Vec<i32>>();
        values.shuffle(&mut rng);
        for v in values {
            let found = hash_set.contains(&v);
            match hash_trie.remove(&v) {
                Ok((ht, found_v)) => {
                    assert_eq!(v, *found_v);
                    assert!(found);
                    hash_trie = ht;
                },
                Err(_) => assert!(!found)
            }
        }
    }

}
