use crate::Package;
use std::borrow::Borrow;
use std::cmp::Ord;
use std::collections::{btree_set, BTreeSet};
use std::{fmt, mem};

#[derive(Clone)]
pub struct Set {
    set: BTreeSet<Package>,
}

impl Set {
    /// Create a new set.
    pub fn new() -> Self {
        let set = BTreeSet::new();

        Self { set }
    }

    /// Add a package into this set.
    pub fn insert(&mut self, package: impl Into<Package>) {
        self.set.insert(package.into());
    }

    /// Get a package from this set if it exists.
    pub fn get<Q>(&self, package: &Q) -> Option<&Package>
    where
        Package: Borrow<Q>,
        Q: Ord,
    {
        self.set.get(package)
    }

    /// Get a package from this set if it exists.
    pub fn get_mut<Q>(&mut self, package: &Q) -> Option<&mut Package>
    where
        Package: Borrow<Q>,
        Q: Ord,
    {
        // SAFETY: I don't care.
        unsafe { mem::transmute(self.set.get(package)) }
    }

    /// Return an iterator over the packages within this set.
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        let iter = self.set.iter();

        Iter { iter }
    }

    /// Remove a package from this set.
    pub fn remove<Q>(&mut self, package: &Q)
    where
        Package: Borrow<Q>,
        Q: Ord,
    {
        self.set.remove(package);
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
}

pub struct Iter<'a> {
    iter: btree_set::Iter<'a, Package>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Package;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.set.iter()).finish()
    }
}
