use std::collections::HashMap;
use std::hash::Hash;

pub struct BiHashMap<K, V> {
    key_to_val: HashMap<K, V>,
    val_to_key: HashMap<V, K>,
}

impl<K, V> BiHashMap<K, V> {
    /// Create an empty `BiHashMap`.
    pub fn new() -> Self {
        Self {
            key_to_val: HashMap::new(),
            val_to_key: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key_to_val.is_empty()
    }

    pub fn len(&self) -> usize {
        self.key_to_val.len()
    }
}

impl<K, V> BiHashMap<K, V>
where
    K: Copy + Eq + Hash,
    V: Copy + Eq + Hash,
{
    pub fn insert(&mut self, key: K, val: V) {
        self.key_to_val.insert(key, val);
        self.val_to_key.insert(val, key);
    }
}

impl<K, V> BiHashMap<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone + Eq + Hash,
{
    pub fn insert_cloned(&mut self, key: K, val: V) {
        self.key_to_val.insert(key.clone(), val.clone());
        self.val_to_key.insert(val, key);
    }
}
