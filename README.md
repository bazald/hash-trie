# HashTrie

HashTrie provides containers based on Hash Array Mapped Tries (HAMT). They are concurrency and functional programming friendly. While the data structures are immutable, the vast majority of memory is shared between instances as they are modified.

`HashTrieSet` implements a hash set. Here's a quick usage example.
```
use hash_trie::HashTrieSet;
use std::{borrow::Cow, collections::hash_map::DefaultHasher};

let mut hash_set: HashTrieSet<u64, u32, String, DefaultHasher> = HashTrieSet::new();
let hello_world: String = "Hello, world!".into();

hash_set = hash_set.insert(Cow::Borrowed(&hello_world), false).unwrap().0;

// Inserting an already-inserted value returns a reference to the value in the set...
assert_eq!(*hash_set.insert(Cow::Borrowed(&hello_world), false).unwrap_err(), hello_world);
// ... unless you enable replacement.
assert!(hash_set.insert(Cow::Borrowed(&hello_world), true).is_ok());

assert_eq!(*hash_set.find(&hello_world).unwrap(), hello_world);

match hash_set.remove(&hello_world) {
    Ok((mutated, reference)) => {
        // Removing a value returns a reference to the value
        // in the set in addition to the mutated set.
        println!("Value stored in hash_set: {}", reference);
        hash_set = mutated;
    },
    Err(_) => panic!(),
}
```

`HashTrieMap` implements a hash map with comparable syntax.
