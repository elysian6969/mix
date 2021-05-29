use super::btree_map::BTreeMap;
use super::vec::Vec;

#[repr(transparent)]
pub struct BTreeVecMap<K, V, S> {
    inner: BTreeMap<K, Vec<V>, S>,
}
