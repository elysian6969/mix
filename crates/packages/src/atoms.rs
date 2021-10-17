use crate::{packages, Package};
use std::option;

pub enum Atoms<'a> {
    Exact(option::IntoIter<&'a Package>),
    Set(packages::Packages<'a>),
}

impl<'a> Iterator for Atoms<'a> {
    type Item = &'a Package;

    fn next(&mut self) -> Option<Self::Item> {
        use Atoms::*;

        match self {
            Exact(iter) => iter.next(),
            Set(iter) => iter.next(),
        }
    }
}
