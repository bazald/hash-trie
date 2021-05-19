use hash_trie::HashTrieSet;
use im::HashSet as ImHashSet;
use std::{borrow::Cow, collections::{hash_map::DefaultHasher, hash_set::HashSet}, time::SystemTime, vec::Vec};
use rand::{Rng, seq::SliceRandom};

fn main() {
    println!("HashSet: {} µs", hash_set());
    println!("HashSetInc: {} µs", hash_set_inc());
    println!("ImHashSet: {} µs", im_hash_set());
    println!("HashTrieSet: {} µs", hash_trie_set());
}

fn hash_set() -> u128 {
    let (insertions, searches, removals) = get_values();

    let mut hash_set = HashSet::new();

    let t0 = SystemTime::now();

    for v in insertions {
        hash_set.insert(v);
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_set.contains(&v);
    }

    let t2 = SystemTime::now();

    for v in removals {
        hash_set.remove(&v);
    }
    
    let t3 = SystemTime::now();

    println!("HashSet insertions: {} µs\r\nHashSet searches: {} µs\r\nHashSet removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t3.duration_since(t2).unwrap().as_micros());

    t3.duration_since(t0).unwrap().as_micros()
}

fn hash_set_inc() -> u128 {
    let (insertions, searches, removals) = get_values();

    let mut hash_set = HashSet::new();

    let t0 = SystemTime::now();

    for v in insertions {
        let mut hs = hash_set.clone();
        hs.insert(v);
        hash_set = hs;
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_set.contains(&v);
    }

    let t2 = SystemTime::now();

    for v in removals {
        let mut hs = hash_set.clone();
        hs.remove(&v);
        hash_set = hs;
    }
    
    let t3 = SystemTime::now();

    println!("HashSetInc insertions: {} µs\r\nHashSetInc searches: {} µs\r\nHashSetInc removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t3.duration_since(t2).unwrap().as_micros());

    t3.duration_since(t0).unwrap().as_micros()
}

fn im_hash_set() -> u128 {
    let (insertions, searches, removals) = get_values();

    let mut hash_set = ImHashSet::new();

    let t0 = SystemTime::now();

    for v in insertions {
        hash_set = hash_set.update(v);
    }

    let t1 = SystemTime::now();

    for v in searches {
        let _ = hash_set.contains(&v);
    }

    let t2 = SystemTime::now();

    for v in removals {
        hash_set = hash_set.without(&v);
    }
    
    let t3 = SystemTime::now();

    println!("ImHashSet insertions: {} µs\r\nImHashSet searches: {} µs\r\nImHashSet removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t3.duration_since(t2).unwrap().as_micros());

    t3.duration_since(t0).unwrap().as_micros()
}

fn hash_trie_set() -> u128 {
    let (insertions, searches, removals) = get_values();

    let mut hash_trie = HashTrieSet::<u64, u32, i32, DefaultHasher>::new();

    let t0 = SystemTime::now();

    for v in insertions {
        if let Ok(ht) = hash_trie.insert(Cow::Owned(v), false) {
            hash_trie = ht.0;
        }
    }

    let t1 = SystemTime::now();

    for v in searches {
        let _ = hash_trie.find(&v);
    }

    let t2 = SystemTime::now();

    for v in removals {
        if let Ok((ht, _found_v)) = hash_trie.remove(&v) {
            hash_trie = ht;
        }
    }
    
    let t3 = SystemTime::now();

    println!("HashTrieSet insertions: {} µs\r\nHashTrieSet searches: {} µs\r\nHashTrieSet removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t3.duration_since(t2).unwrap().as_micros());

    t3.duration_since(t0).unwrap().as_micros()
}

fn get_values() -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut rng = rand::thread_rng();
    let insertions = (0..25000).map(|_| rng.gen_range(0..100000)).collect::<Vec<i32>>();
    let mut searches = (1..100000).collect::<Vec<i32>>();
    searches.shuffle(&mut rng);
    let mut removals: Vec<i32> = searches.clone();
    removals.shuffle(&mut rng);
    (insertions, searches, removals)
}
