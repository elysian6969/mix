use crate::{atoms, Package};

pub struct Matches<'a> {
    pub(crate) iter: atoms::Atoms<'a>,
}

impl<'a> Iterator for Matches<'a> {
    type Item = &'a Package;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
