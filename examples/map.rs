use hash_trie::HashTrieMap;
use std::{borrow::Cow, collections::{hash_map::DefaultHasher, hash_map::HashMap}, time::SystemTime, vec::Vec};
use rand::{Rng, seq::SliceRandom};

fn main() {
    println!("HashMap: {} µs", hash_map());
    println!("HashMapInc: {} µs", hash_map_inc());
    println!("HashTrieMap: {} µs", hash_trie_map());
}

fn hash_map() -> u128 {
    let (insertions, searches, removals) = get_values();

    let mut hash_map = HashMap::new();

    let t0 = SystemTime::now();

    for v in insertions {
        hash_map.insert(v, 0x42);
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_map.get(&v);
    }

    let t2 = SystemTime::now();

    for v in removals {
        hash_map.remove(&v);
    }
    
    let t3 = SystemTime::now();

    println!("HashMap insertions: {} µs\r\nHashMap searches: {} µs\r\nHashMap removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t3.duration_since(t2).unwrap().as_micros());

    t3.duration_since(t0).unwrap().as_micros()
}

fn hash_map_inc() -> u128 {
    let (insertions, searches, removals) = get_values();

    let mut hash_map = HashMap::new();

    let t0 = SystemTime::now();

    for v in insertions {
        let mut ht = hash_map.clone();
        ht.insert(v, 0x42);
        hash_map = ht;
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_map.get(&v);
    }

    let t2 = SystemTime::now();

    for v in removals {
        let mut ht = hash_map.clone();
        ht.remove(&v);
        hash_map = ht;
    }
    
    let t3 = SystemTime::now();

    println!("HashMapInc insertions: {} µs\r\nHashMapInc searches: {} µs\r\nHashMapInc removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t3.duration_since(t2).unwrap().as_micros());

    t3.duration_since(t0).unwrap().as_micros()
}

fn hash_trie_map() -> u128 {
    let (insertions, searches, removals) = get_values();

    let mut hash_trie = HashTrieMap::<u64, u32, i32, i32, DefaultHasher>::new();

    let t0 = SystemTime::now();

    for v in insertions {
        if let Ok(ht) = hash_trie.insert(Cow::Owned(v), Cow::Owned(0x42), false) {
            hash_trie = ht.0;
        }
    }

    let t1 = SystemTime::now();

    for v in searches {
        let _ = hash_trie.find(&v);
    }

    let t2 = SystemTime::now();

    for v in removals {
        if let Ok((ht, _found_k, _found_v)) = hash_trie.remove(&v) {
            hash_trie = ht;
        }
    }
    
    let t3 = SystemTime::now();

    println!("HashTrieMap insertions: {} µs\r\nHashTrieMap searches: {} µs\r\nHashTrieMap removals: {} µs",
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
