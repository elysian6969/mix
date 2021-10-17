use crate::{set, Package, Set};
use std::{iter, option};

pub(crate) type MapSetToIter<'a> = fn(&'a Set) -> set::Iter<'a>;

pub(crate) fn map_set_to_iter<'a>(set: &'a Set) -> set::Iter<'a> {
    set.iter()
}

pub struct Packages<'a> {
    pub(crate) iter: iter::FlatMap<option::IntoIter<&'a Set>, set::Iter<'a>, MapSetToIter<'a>>,
}

impl<'a> Iterator for Packages<'a> {
    type Item = &'a Package;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
