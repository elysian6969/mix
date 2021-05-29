use super::btree_map::BTreeMap;

#[repr(transparent)]
pub struct BTreeSetMap<K, L, V, S, T> {
    inner: BtreeMap<K, BTreeMap<L, V, S>, T>,
}
