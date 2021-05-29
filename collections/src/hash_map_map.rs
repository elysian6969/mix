use super::hash_map::HashMap;

#[repr(transparent)]
pub struct HashSetMap<K, L, V, S, T> {
    inner: HashMap<K, HashMap<L, V, S>, T>,
}
