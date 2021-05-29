use super::hash_map::{DefaultHashBuilder, HashMap};
use super::hash_set::HashSet;
use alloc::alloc::Global;
use core::alloc::Allocator;

#[repr(transparent)]
pub struct HashSetMap<
    K,
    V,
    S = DefaultHashBuilder,
    T = DefaultHashBuilder,
    A: Allocator + Clone = Global,
    B: Allocator + Clone = Global,
> {
    inner: HashMap<K, HashSet<V, T, B>, S, T>,
}

impl<K: Clone, V: Clone, S: Clone, T: Clone, A: Allocator + Clone, B: Allocator + Clone> Clone
    for HashSetMap<K, V, S, T, A, B>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}
