use crate::{traits::*, HashTrieError, hash_trie::HashTrie};
use alloc::{borrow::Cow, fmt::Debug};
use core::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
struct SetEntry<V> {
    value: V,
}

impl <V> SetEntry<V> {
    fn new(value: V) -> Self {
        Self {value}
    }
}

impl <V> AsRef<V> for SetEntry<V> {
    fn as_ref(&self) -> &V {
        &self.value
    }
}

impl <'a, V: Clone + Debug> From<Cow<'a, V>> for SetEntry<V> {
    fn from(cow: Cow<'a, V>) -> Self {
        SetEntry::new(cow.into_owned())
    }
}

impl <B, V, H: HasherBv<B, V>> HasherBv<B, SetEntry<V>> for H {
    fn hash(&self, entry: &SetEntry<V>) -> B {
        H::default().hash(&entry.value)
    }
}

impl <V: Hash> Hash for SetEntry<V> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.value.hash(hasher)
    }
}

impl <V: Eq> Eq for SetEntry<V> {}

impl <V: PartialEq> PartialEq for SetEntry<V> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl <V: PartialEq> PartialEq<V> for SetEntry<V> {
    fn eq(&self, other: &V) -> bool {
        self.value == *other
    }
}

impl <V> HashLike<V> for SetEntry<V> {}
impl <V> HashLike<SetEntry<V>> for V {}

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
/// hash_set = hash_set.insert(Cow::Borrowed(&hello_world), false).unwrap().0;
/// 
/// // Inserting an already-inserted value returns a reference to the value in the set...
/// assert_eq!(*hash_set.insert(Cow::Borrowed(&hello_world), false).unwrap_err(), hello_world);
/// // ... unless you enable replacement.
/// assert!(hash_set.insert(Cow::Borrowed(&hello_world), true).is_ok());
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
pub struct HashTrieSet <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    set: HashTrie<H, F, SetEntry<V>, M>,
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> HashTrieSet<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    /// Get a new, empty HashTrieSet.
    pub fn new() -> Self {
        Self {
            set: HashTrie::<H, F, SetEntry<V>, M>::new()
        }
    }

    /// Search the HashTrieSet for the given value and return a reference if found, or `HashTrieError::NotFound` if not found.
    pub fn find(&self, value: &V) -> Result<&V, HashTrieError> {
        self.set.find(value).map(|entry| entry.as_ref())
    }

    /// Search the HashTrieSet for the spot to insert the value and return both a mutated set and, if applicable, a reference to the replaced value.
    /// If found and replacement is disabled, a reference to the existing value is returned.
    pub fn insert<'a>(&'a self, value: Cow<'a, V>, replace: bool) -> Result<(Self, Option<&'a V>), &'a V> {
        self.set.insert(value, replace).map(|(set, reference)| (Self {set}, reference.map(|entry| entry.as_ref()))).map_err(|entry| entry.as_ref())
    }

    /// Search the HashTrieSet for the given value to remove and return a mutated set, or `HashTrieError::NotFound` if not found.
    pub fn remove(&self, value: &V) -> Result<(Self, &V), HashTrieError> {
        self.set.remove(value).map(|(set, entry)| (Self {set}, entry)).map(|(map, entry)| (map, &entry.value))
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Default for HashTrieSet<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn default() -> Self {
        Self::new()
    }
}
