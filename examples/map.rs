use fnv::FnvHasher;
use hash_trie::HashTrieMap;
use im::HashMap as ImHashMap;
use std::{collections::hash_map::HashMap, sync::{Arc, Mutex}, time::SystemTime, vec::Vec};
use rand::{Rng, seq::SliceRandom};

fn main() {
    let (insertions, searches, removals) = get_values();
    let mut full_hash_map = HashMap::new();

    println!("HashMap: {} µs", hash_map(&insertions, &searches, &removals, &mut full_hash_map));
    println!("HashMapInc: {} µs", hash_map_inc(&insertions, &searches, &removals, &full_hash_map));
    println!("ImHashMap: {} µs", im_hash_map(&insertions, &searches, &removals, &full_hash_map));
    println!("HashTrieMap: {} µs", hash_trie_map(&insertions, &searches, &removals, &full_hash_map));
}

fn hash_map(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_map: &mut HashMap<i32, i32>) -> u128 {
    let mut hash_map = HashMap::new();

    let t0 = SystemTime::now();

    for v in insertions {
        hash_map.insert(*v, 0x42);
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_map.get(v);
    }

    let t2 = SystemTime::now();
    
    *full_hash_map = hash_map.clone();

    let t3 = SystemTime::now();

    for v in removals {
        hash_map.remove(v);
    }
    
    let t4 = SystemTime::now();

    println!("HashMap insertions: {} µs\r\nHashMap searches: {} µs\r\nHashMap removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t4.duration_since(t3).unwrap().as_micros());

    t4.duration_since(t0).unwrap().as_micros() - t3.duration_since(t2).unwrap().as_micros()
}

fn hash_map_inc(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_map: &HashMap<i32, i32>) -> u128 {
    let mut hash_map = HashMap::new();

    let t0 = SystemTime::now();

    for v in insertions {
        let mut ht = hash_map.clone();
        ht.insert(*v, 0x42);
        hash_map = ht;
    }

    let t1 = SystemTime::now();

    for v in searches {
        hash_map.get(v);
    }

    let t2 = SystemTime::now();

    assert_eq!(hash_map, *full_hash_map);

    let t3 = SystemTime::now();

    for v in removals {
        let mut ht = hash_map.clone();
        ht.remove(v);
        hash_map = ht;
    }
    
    let t4 = SystemTime::now();

    println!("HashMapInc insertions: {} µs\r\nHashMapInc searches: {} µs\r\nHashMapInc removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t4.duration_since(t3).unwrap().as_micros());

    t4.duration_since(t0).unwrap().as_micros() - t3.duration_since(t2).unwrap().as_micros()
}

fn im_hash_map(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_map: &HashMap<i32, i32>) -> u128 {
    let mut hash_map = ImHashMap::new();

    let t0 = SystemTime::now();

    for v in insertions {
        hash_map = hash_map.update(*v, 0x42);
    }

    let t1 = SystemTime::now();

    for v in searches {
        let _ = hash_map.get(v);
    }

    let t2 = SystemTime::now();

    let mut cmp = HashMap::new();
    for (&k, &v) in &hash_map {
        cmp.insert(k, v);
    }
    assert_eq!(cmp, *full_hash_map);

    let t3 = SystemTime::now();

    for v in removals {
        hash_map = hash_map.without(v);
    }
    
    let t4 = SystemTime::now();

    println!("ImHashMap insertions: {} µs\r\nImHashMap searches: {} µs\r\nImHashMap removals: {} µs",
        t1.duration_since(t0).unwrap().as_micros(),
        t2.duration_since(t1).unwrap().as_micros(),
        t4.duration_since(t3).unwrap().as_micros());

    t4.duration_since(t0).unwrap().as_micros() - t3.duration_since(t2).unwrap().as_micros()
}

fn hash_trie_map(insertions: &[i32], searches: &[i32], removals: &[i32], full_hash_map: &HashMap<i32, i32>) -> u128 {
    let mut hash_map = HashTrieMap::<u64, u32, i32, i32, FnvHasher>::new();

    let t0 = SystemTime::now();

    for v in insertions {
        if let Ok(ht) = hash_map.insert(*v, 0x42, false) {
            hash_map = ht.0;
        }
    }

    let t1 = SystemTime::now();

    for v in searches {
        let _ = hash_map.find(v);
    }

    let t2 = SystemTime::now();

    let cmp = Arc::new(Mutex::new(HashMap::new()));
    hash_map.visit(|&k, &v| {cmp.lock().unwrap().insert(k, v);});
    assert_eq!(*cmp.lock().unwrap(), *full_hash_map);


    let t3 = SystemTime::now();

    for v in removals {
        if let Ok((ht, _key, _value)) = hash_map.remove(v) {
            hash_map = ht;
        }
    }
    
    let t4 = SystemTime::now();

    println!("HashTrieMap insertions: {} µs\r\nHashTrieMap searches: {} µs\r\nHashTrieMap removals: {} µs",
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
