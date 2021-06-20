use fnv::FnvHasher;
use hash_trie::HashTrieSet;
use im::HashSet as ImHashSet;
use std::{collections::{hash_set::HashSet}, sync::{Arc, Mutex}, time::SystemTime, vec::Vec};
use rand::{Rng, seq::SliceRandom};

fn main() {
    let (insertions, searches, removals) = get_values();
    let mut full_hash_set = HashSet::new();

    println!("HashSet: {} µs", hash_set(&insertions, &searches, &removals, &mut full_hash_set));
    println!("HashSetInc: {} µs", hash_set_inc(&insertions, &searches, &removals, &full_hash_set));
    println!("ImHashSet: {} µs", im_hash_set(&insertions, &searches, &removals, &full_hash_set));
    println!("HashTrieSet: {} µs", hash_trie_set(&insertions, &searches, &removals, &full_hash_set));
}

fn hash_set(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_set: &mut HashSet<i32>) -> u128 {
    let mut hash_set = HashSet::new();

    let t0 = SystemTime::now();

    for v in insertions {
        hash_set.insert(*v);
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_set.contains(v);
    }

    let t2 = SystemTime::now();

    *full_hash_set = hash_set.clone();

    let t3 = SystemTime::now();

    for v in removals {
        hash_set.remove(v);
    }
    
    let t4 = SystemTime::now();

    println!("HashSet insertions: {} µs\r\nHashSet searches: {} µs\r\nHashSet removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t4.duration_since(t3).unwrap().as_micros());

    t4.duration_since(t0).unwrap().as_micros() - t3.duration_since(t2).unwrap().as_micros()
}

fn hash_set_inc(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_set: &HashSet<i32>) -> u128 {
    let mut hash_set = HashSet::new();

    let t0 = SystemTime::now();

    for v in insertions {
        let mut hs = hash_set.clone();
        hs.insert(*v);
        hash_set = hs;
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_set.contains(v);
    }

    let t2 = SystemTime::now();

    assert_eq!(hash_set, *full_hash_set);

    let t3 = SystemTime::now();

    for v in removals {
        let mut hs = hash_set.clone();
        hs.remove(v);
        hash_set = hs;
    }
    
    let t4 = SystemTime::now();

    println!("HashSetInc insertions: {} µs\r\nHashSetInc searches: {} µs\r\nHashSetInc removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t4.duration_since(t3).unwrap().as_micros());

    t4.duration_since(t0).unwrap().as_micros() - t3.duration_since(t2).unwrap().as_micros()
}

fn im_hash_set(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_set: &HashSet<i32>) -> u128 {
    let mut hash_set = ImHashSet::new();

    let t0 = SystemTime::now();

    for v in insertions {
        hash_set = hash_set.update(*v);
    }

    let t1 = SystemTime::now();

    for v in searches {
        let _ = hash_set.contains(v);
    }

    let t2 = SystemTime::now();

    let mut cmp = HashSet::new();
    for &k in &hash_set {
        cmp.insert(k);
    }
    assert_eq!(cmp, *full_hash_set);

    let t3 = SystemTime::now();

    for v in removals {
        hash_set = hash_set.without(v);
    }
    
    let t4 = SystemTime::now();

    println!("ImHashSet insertions: {} µs\r\nImHashSet searches: {} µs\r\nImHashSet removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t4.duration_since(t3).unwrap().as_micros());

    t4.duration_since(t0).unwrap().as_micros() - t3.duration_since(t2).unwrap().as_micros()
}

fn hash_trie_set(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_set: &HashSet<i32>) -> u128 {
    let mut hash_set = HashTrieSet::<u64, u32, i32, FnvHasher>::new();

    let t0 = SystemTime::now();

    for v in insertions {
        if let Ok(ht) = hash_set.insert(*v, false) {
            hash_set = ht.0;
        }
    }

    let t1 = SystemTime::now();

    for v in searches {
        let _ = hash_set.find(v);
    }

    let t2 = SystemTime::now();
    
    let cmp = Arc::new(Mutex::new(HashSet::new()));
    hash_set.visit(|&k| {cmp.lock().unwrap().insert(k);});
    assert_eq!(*cmp.lock().unwrap(), *full_hash_set);

    let t3 = SystemTime::now();

    for v in removals {
        if let Ok((ht, _found_v)) = hash_set.remove(v) {
            hash_set = ht;
        }
    }
    
    let t4 = SystemTime::now();

    println!("HashTrieSet insertions: {} µs\r\nHashTrieSet searches: {} µs\r\nHashTrieSet removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t4.duration_since(t3).unwrap().as_micros());

    t4.duration_since(t0).unwrap().as_micros() - t3.duration_since(t2).unwrap().as_micros()
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
