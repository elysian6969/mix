use crate::{atoms, packages, Package};
use std::option;

pub struct Matches<'a> {
    pub(crate) iter: atoms::Atoms<'a>,
    pub(crate) requirement: &'a mix_atom::Requirement,
}

impl<'a> Iterator for Matches<'a> {
    type Item = &'a Package;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
