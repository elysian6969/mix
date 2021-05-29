use super::btree_map::BTreeMap;
use super::btree_set::BTreeSet;

#[repr(transparent)]
pub struct BTreeSetMap<K, V, S> {
    inner: BtreeMap<K, BTreeSet<V>, S>,
}
