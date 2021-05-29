pub mod btree_map;
pub mod btree_map_map;
pub mod btree_set;
pub mod btree_set_map;
pub mod btree_vec_map;
pub mod hash_map;
pub mod hash_map_map;
pub mod hash_set;
pub mod hash_vec_map;
pub mod vec;

/*use std::borrow::Borrow;
use std::collections::hash_map::{HashMap, HashSet, RandomState};
use std::hash::Hash;
use std::rc::Rc;

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ref<T>(Rc<T>);

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for Ref<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct BiMultiHashMap<L, R> {
    left: HashMap<Ref<L>, HashSet<Ref<R>>>,
    right: HashMap<Ref<R>, HashSet<Ref<L>>>,
}

impl<L, R> BiHashMap<L, R> {
    /// Create an empty `BiMultiHashMap`.
    ///
    ///
    /// The map is initially created with a capacity of 0, so it
    /// will not allocate until it is first inserted into.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bimap::BiHashMap;
    ///
    /// let mut map: BiHashMap<&str, usize> = BiHashMap::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            left: HashMap::new(),
            right: HashMap::new(),
        }
    }

    /// Creates an empty `BiMultiHashMap` with the specified capacities.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bimap::BiHashMap;
    ///
    /// let mut map: BiHashMap<&str, usize> = BiHashMap::with_capacity(10, 20);
    /// ```
    #[inline]
    pub fn with_capacity(left: usize, right: usize) -> Self {
        Self {
            left: HashMap::with_capacity(left),
            right: HashMap::with_capacity(right),
        }
    }

    /// Clears the map, removing all left-right pairs. Keeps the
    /// allocated memory for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
    }

    #[inline]
    pub fn contains_left<Q>(&self, left: &Q) -> Option<&R>
    where
        L: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
    {
        self.left.contains_key(left)
    }

    #[inline]
    pub fn contains_right<Q>(&self, right: &Q) -> Option<&L>
    where
        R: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
    {
        self.right.contains_right(right)
    }

    #[inline]
    pub fn get_right<Q>(&self, left: &Q) -> Option<&R>
    where
        L: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
    {
        self.left.get(left)
    }

    #[inline]
    pub fn get_left<Q>(&self, right: &Q) -> Option<&L>
    where
        R: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
    {
        self.right.get(right)
    }

    #[inline]
    pub fn get_left_right<Q>(&self, left: &Q) -> Option<(&L, &R)>
    where
        R: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
    {
        self.left.get_key_value(left)
    }

    #[inline]
    pub fn get_right_left<Q>(&self, right: &Q) -> Option<(&R, &L)>
    where
        R: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
    {
        self.right.get_key_value(right)
    }

    #[inline]
    pub fn insert(&mut self, left: L, right: R)
    where
        L: Copy + Eq + Hash,
        R: Copy + Eq + Hash,
    {
        self.left.insert(left, right);
        self.right.insert(right, left);
    }

    #[inline]
    pub fn insert_cloned(&mut self, left: L, right: R)
    where
        R: Clone + Eq + Hash,
        L: Clone + Eq + Hash,
    {
        self.left.insert(left.clone(), right.clone());
        self.right.insert(right, left);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.left.is_empty()
    }

    #[inline]
    pub fn len(&self) -> (usize, usize) {
        (self.left.len(), self.right.len())
    }

    /// Returns the number of elements the map can hold without
    /// reallocating in each direction.
    ///
    /// These numbers are a lower bound; the BiMultiHashMap<K, V>
    /// might be able to hold more, but is guaranteed to be able
    /// to hold at least this many.
    #[inline]
    pub fn capacity(&self) -> (usize, usize) {
        (self.left.capacity(), self.right.capacity())
    }
}*/
