use super::hash_map::HashMap;
use super::vec::Vec;

#[repr(transparent)]
pub struct HashVecMap<K, V, S> {
    inner: HashMap<K, Vec<V>, S>,
}
