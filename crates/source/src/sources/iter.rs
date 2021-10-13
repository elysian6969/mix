use crate::Source;
use path::PathBuf;
use std::collections::btree_map;

pub struct Iter<'a> {
    pub(crate) iter: btree_map::Keys<'a, Source, PathBuf>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Source;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
